//
// Copyright 2025 Formata, Inc. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

//! Sealing: a host freezes the behavior in part of a document, so code that runs later (an agent's function, a
//! customer's rule, a document from somewhere else) can call it but can't change it.
//!
//! `Graph::seal(node, fields)` seals an object and everything under it. From then on, in that part of the document:
//! - functions can't be added, removed, renamed, replaced, or moved, and neither can prototypes (an object's type)
//!   or fields that hold a function or a sealed object (the paths sealed code calls through)
//! - sealed objects can't be removed, moved, renamed, or have their attributes changed
//! - no other type can take a sealed type's name (it would be found first by code closer to it)
//! - with `fields`, every field is frozen too (off by default: a sealed object can still keep state)
//!
//! It's enforced where the document changes, for Stof code and host calls alike (`unseal` is the way back, and only
//! the host has it), so a call costs nothing more. A refused change fails the instruction (or host call) that made it
//! with `Error::Sealed`. Arrow functions evaluated by sealed code still work.
//!
//! What it guarantees: the code in the sealed part doesn't change. Stof finds functions by name when they're called,
//! so a sealed function that calls into an unsealed part of the document calls whatever is there; seal everything
//! a call chain relies on. Library functions can't be changed by documents, so calls to them are covered.
//!
//! `Graph::digest(node)` is a SHA-256 over the sealed part's functions (names, parameters, return types,
//! attributes, and instructions) and structure (the objects, types, and the fields that hold functions or objects),
//! plus every field's value with `fields`. It's the same whether the document was parsed from source or loaded from
//! BSTF. A host checks it against the digest it expects, or verifies an Ed25519 `signature` over it (`verify`), Ex.
//! for an engine shipped as a document. It's computed only when asked for (a few milliseconds for a large part).

use std::{cell::Cell, collections::BTreeMap, sync::Arc};
use rustc_hash::{FxHashMap, FxHashSet};
use serde::Serialize;
use sha2::{Digest, Sha256};
use crate::{model::{ARROW_FUNC_ATTR, DataRef, Field, Func, Graph, NodeRef, Prototype}, runtime::{Error, Val}};


thread_local! {
    /// Set while computing a digest: arrow function IDs (random per parse) are left out and attributes are sorted,
    /// so the same functions always give the same bytes. Never set otherwise, so BSTF is unchanged.
    static DIGESTING: Cell<bool> = const { Cell::new(false) };
}

#[doc(hidden)]
/// While alive, serialization is in digest mode (for tools and tests that inspect digest bytes).
pub struct DigestScope(bool);
impl DigestScope {
    pub fn new() -> Self { Self(DIGESTING.with(|digesting| digesting.replace(true))) }
}
impl Drop for DigestScope {
    fn drop(&mut self) { DIGESTING.with(|digesting| digesting.set(self.0)); }
}

/// Is a digest being computed (see `DIGESTING`)?
pub(crate) fn digesting() -> bool {
    DIGESTING.with(|digesting| digesting.get())
}

/// Serialize an arrow function's ID, or nothing while digesting (`FuncLit`).
pub(crate) fn serialize_arrow_id<S: serde::Serializer>(dref: &DataRef, serializer: S) -> Result<S::Ok, S::Error> {
    if digesting() { "".serialize(serializer) } else { dref.serialize(serializer) }
}

/// Serialize attributes, sorted while digesting (`Func`, `Field`).
pub(crate) fn serialize_attributes<S: serde::Serializer>(attributes: &FxHashMap<String, Val>, serializer: S) -> Result<S::Ok, S::Error> {
    if digesting() {
        attributes.iter().collect::<BTreeMap<_, _>>().serialize(serializer)
    } else {
        attributes.serialize(serializer)
    }
}


#[derive(Debug, Clone, Default)]
/// The sealed parts of a graph (host runtime state: never stored in documents).
pub struct Seals {
    /// Every sealed object, and whether its fields are frozen too.
    pub nodes: FxHashMap<NodeRef, bool>,
    /// The objects `seal` was called on.
    pub roots: FxHashSet<NodeRef>,
    /// Type names of sealed types (no other type may take them).
    pub types: FxHashSet<String>,
    /// Why the last change was refused, until the instruction or host call that made it reports it.
    pub violation: Option<String>,
}


impl Graph {
    /// Seal an object and everything under it (see the module docs), with its fields frozen too when `fields`.
    /// Objects added under it later are not sealed. Returns the part's digest.
    pub fn seal(&mut self, node: &NodeRef, fields: bool) -> Result<String, Error> {
        if !node.node_exists(self) { return Err(Error::Sealed("there's no object to seal".into())); }
        let digest = self.digest(node, fields)?;
        let nodes = self.all_child_nodes(node, true);
        let mut types = FxHashSet::default();
        for (name, typed) in &self.typemap {
            if typed.iter().any(|ty| nodes.contains(ty)) { types.insert(name.clone()); }
        }
        let seals = Arc::make_mut(&mut self.seals);
        for nref in nodes {
            let frozen = seals.nodes.get(&nref).copied().unwrap_or(false) || fields;
            seals.nodes.insert(nref, frozen);
        }
        seals.roots.insert(node.clone());
        seals.types.extend(types);
        Ok(digest)
    }

    /// Seal an object (by path, or a type as `<Name>`), checking first that its digest is `expected` when given.
    pub fn seal_path(&mut self, path: &str, fields: bool, expected: Option<&str>) -> Result<String, Error> {
        let node = self.seal_target(path)?;
        if let Some(expected) = expected {
            let digest = self.digest(&node, fields)?;
            if !digest.eq_ignore_ascii_case(expected.trim()) {
                return Err(Error::Sealed(format!("{path} isn't what was expected: its digest is {digest}, not {expected}")));
            }
        }
        self.seal(&node, fields)
    }

    /// The digest of an object by path (or a type as `<Name>`), with its fields' values when `fields`.
    pub fn digest_path(&self, path: &str, fields: bool) -> Result<String, Error> {
        let node = self.seal_target(path)?;
        self.digest(&node, fields)
    }

    /// Unseal an object by path (or a type as `<Name>`) that `seal` was called on.
    pub fn unseal_path(&mut self, path: &str) -> Result<(), Error> {
        let node = self.seal_target(path)?;
        self.unseal(&node);
        Ok(())
    }

    /// Unseal an object sealed with `seal` (and everything `seal` sealed under it). Host only: documents can't.
    pub fn unseal(&mut self, node: &NodeRef) {
        if !self.seals.roots.contains(node) { return; }
        let nodes = self.all_child_nodes(node, true);
        let mut roots = self.seals.roots.clone();
        roots.remove(node);
        // what's still under another seal stays sealed
        let release: Vec<NodeRef> = nodes.into_iter()
            .filter(|nref| !roots.iter().any(|root| nref == root || nref.child_of(self, root)))
            .collect();
        let seals = Arc::make_mut(&mut self.seals);
        seals.roots = roots;
        for nref in &release { seals.nodes.remove(nref); }
        let types = self.typemap.iter()
            .filter(|(_, typed)| typed.iter().any(|ty| self.seals.nodes.contains_key(ty)))
            .map(|(name, _)| name.clone())
            .collect();
        Arc::make_mut(&mut self.seals).types = types;
    }

    /// The object a host path names (Ex. "policy.rules"), or a type as `<Name>` (Ex. "<Limitr>").
    pub fn seal_target(&self, path: &str) -> Result<NodeRef, Error> {
        let path = path.trim();
        let found = if let Some(name) = path.strip_prefix('<').and_then(|rest| rest.strip_suffix('>')) {
            match name.rsplit_once('.') {
                Some((qualifier, name)) => self.find_qualified_type(qualifier, name, None),
                None => self.find_type(name, None),
            }
        } else {
            self.host_node(path)
        };
        found.ok_or_else(|| Error::Sealed(format!("there's no object or type '{path}' to seal")))
    }

    /// Run this with the seals lifted, for the few changes Stof itself makes on sealed objects: inserting and updating
    /// arrow functions that sealed code evaluates (they're runtime values, under new random names), and attaching a
    /// host's `call_source` functions for the call. Nothing a document can reach.
    pub(crate) fn without_seals<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        if self.seals.nodes.is_empty() { return f(self); }
        let seals = std::mem::take(&mut self.seals);
        let res = f(self);
        let violation = self.take_seal_violation();
        self.seals = seals;
        if violation.is_some() { Arc::make_mut(&mut self.seals).violation = violation; }
        res
    }

    #[inline]
    /// Is this object sealed?
    pub fn is_sealed(&self, node: &NodeRef) -> bool {
        !self.seals.nodes.is_empty() && self.seals.nodes.contains_key(node)
    }


    /*****************************************************************************
     * Checks, from the places the graph changes.
     *****************************************************************************/

    /// Refuse a change: remember why (reported as `Error::Sealed` by the instruction or host call), return false.
    pub(crate) fn refuse(&mut self, why: String) -> bool {
        if self.seals.violation.is_none() {
            Arc::make_mut(&mut self.seals).violation = Some(why);
        }
        false
    }

    /// The reason a change was refused since the last check, if one was (and clear it).
    pub fn take_seal_violation(&mut self) -> Option<String> {
        if self.seals.violation.is_none() { return None; }
        Arc::make_mut(&mut self.seals).violation.take()
    }

    /// Is this data on this node sealed (it can't change)?
    pub(crate) fn sealed_data(&self, node: &NodeRef, dref: &DataRef) -> bool {
        let Some(fields) = self.seals.nodes.get(node) else { return false };
        *fields || self.behavior(dref)
    }

    /// Is this data sealed on any node that holds it?
    pub(crate) fn sealed_anywhere(&self, dref: &DataRef) -> bool {
        if self.seals.nodes.is_empty() { return false; }
        dref.data(self).is_some_and(|data| data.nodes.iter().any(|node| self.sealed_data(node, dref)))
    }

    /// Is this data behavior a seal protects: a function, a prototype, or a field holding a function or a sealed
    /// object?
    fn behavior(&self, dref: &DataRef) -> bool {
        let Some(data) = dref.data(self) else { return false };
        let any = data.data.as_dyn_any();
        if any.is::<Func>() || any.is::<Prototype>() { return true; }
        if let Some(field) = any.downcast_ref::<Field>() {
            return match &*field.value.val.read() {
                Val::Fn(_) => true,
                Val::Obj(obj) => self.seals.nodes.contains_key(obj),
                _ => false,
            };
        }
        false
    }

    /// May this data be put on this node (as `name`)? Refuses (see `refuse`) when not.
    pub(crate) fn may_insert(&mut self, node: &NodeRef, name: &str, data: &crate::model::Data) -> bool {
        if self.seals.nodes.is_empty() { return true; }
        // the same ID replaces data that's already in the graph, wherever it is
        if self.data.contains_key(&data.id) && self.sealed_anywhere(&data.id) {
            return self.refuse(format!("'{name}' is sealed and can't be replaced"));
        }
        let Some(fields) = self.seals.nodes.get(node).copied() else { return true };
        if let Some(existing) = node.node(self).and_then(|n| n.data.get(name)).cloned() {
            if existing != data.id && self.sealed_data(node, &existing) {
                return self.refuse(format!("'{name}' is sealed and can't be replaced"));
            }
        }
        let any = data.data.as_dyn_any();
        let behavior = any.is::<Func>() || any.is::<Prototype>() || any.downcast_ref::<Field>()
            .is_some_and(|field| matches!(&*field.value.val.read(), Val::Fn(_)) || field.value.try_obj().is_some_and(|obj| self.seals.nodes.contains_key(&obj)));
        if fields || behavior {
            return self.refuse(format!("can't add '{name}' to a sealed object"));
        }
        true
    }

    /// May this data be removed (from this node, or from every node)? Refuses when not.
    pub(crate) fn may_remove(&mut self, dref: &DataRef, node: Option<&NodeRef>) -> bool {
        if self.seals.nodes.is_empty() { return true; }
        let sealed = match node {
            Some(node) => self.sealed_data(node, dref),
            None => self.sealed_anywhere(dref),
        };
        if sealed {
            let name = dref.data_name(self).map(|name| name.to_string()).unwrap_or_default();
            return self.refuse(format!("'{name}' is sealed and can't be removed"));
        }
        true
    }

    /// May this data change (renamed, set, or written to)? Refuses when not.
    pub(crate) fn may_change(&mut self, dref: &DataRef) -> bool {
        if self.seals.nodes.is_empty() || !self.sealed_anywhere(dref) { return true; }
        let name = dref.data_name(self).map(|name| name.to_string()).unwrap_or_default();
        self.refuse(format!("'{name}' is sealed and can't be changed"))
    }

    /// May this object be removed, moved, or renamed (it isn't sealed and holds nothing sealed)? Refuses when not.
    pub(crate) fn may_move_node(&mut self, node: &NodeRef) -> bool {
        if self.seals.nodes.is_empty() { return true; }
        let sealed = self.seals.nodes.contains_key(node)
            || self.seals.roots.iter().any(|root| root.child_of(self, node));
        if sealed {
            let name = node.node(self).map(|n| n.name.to_string()).unwrap_or_default();
            return self.refuse(format!("'{name}' is sealed (or holds a sealed object) and can't be removed, moved, or renamed"));
        }
        true
    }

    /// May this object's attributes change? Refuses when it's sealed.
    pub(crate) fn may_change_node(&mut self, node: &NodeRef) -> bool {
        if !self.is_sealed(node) { return true; }
        let name = node.node(self).map(|n| n.name.to_string()).unwrap_or_default();
        self.refuse(format!("'{name}' is sealed and its attributes can't change"))
    }

    /// May a child object be added to this one (not when its fields are frozen)? Refuses when not.
    pub(crate) fn may_add_child(&mut self, parent: &NodeRef) -> bool {
        if self.seals.nodes.get(parent).copied() != Some(true) { return true; }
        let name = parent.node(self).map(|n| n.name.to_string()).unwrap_or_default();
        self.refuse(format!("'{name}' is sealed with its fields and can't have objects added"))
    }

    /// May this object become a type named `name`? Refuses a sealed type's name (it would shadow it).
    pub(crate) fn may_insert_type(&mut self, name: &str, node: &NodeRef) -> bool {
        if self.seals.types.is_empty() || !self.seals.types.contains(name) || self.seals.nodes.contains_key(node) { return true; }
        self.refuse(format!("the type name '{name}' is sealed: another type can't take it"))
    }


    /*****************************************************************************
     * Digest.
     *****************************************************************************/

    /// A SHA-256 (hex) over this object's functions and structure, and its fields' values with `fields` (see the
    /// module docs). The same for the same document, parsed or loaded from BSTF.
    pub fn digest(&self, node: &NodeRef, fields: bool) -> Result<String, Error> {
        if !node.node_exists(self) { return Err(Error::Sealed("there's no object to digest".into())); }
        let previous = DIGESTING.with(|digesting| digesting.replace(true));
        let mut hasher = Sha256::new();
        hasher.update(b"stof-seal-v1");
        let res = self.digest_node(node, node, fields, &mut hasher, &mut FxHashSet::default());
        DIGESTING.with(|digesting| digesting.set(previous));
        res?;
        Ok(hex(&hasher.finalize()))
    }

    fn digest_node(&self, root: &NodeRef, nref: &NodeRef, fields: bool, hasher: &mut Sha256, seen: &mut FxHashSet<NodeRef>) -> Result<(), Error> {
        if !seen.insert(nref.clone()) { return Ok(()); }
        let Some(node) = nref.node(self) else { return Ok(()) };
        put(hasher, b"node", node.name.as_ref().as_bytes());
        put(hasher, b"attrs", &self.attrs_bytes(root, &node.attributes)?);

        let mut data: Vec<(&String, &DataRef)> = node.data.iter().collect();
        data.sort_by(|a, b| a.0.cmp(b.0));
        for (name, dref) in data {
            let Some(item) = dref.data(self) else { continue };
            let any = item.data.as_dyn_any();
            if let Some(func) = any.downcast_ref::<Func>() {
                // arrow functions are runtime values (named by random IDs, made when evaluated): a field that holds
                // one carries its body instead
                if func.attributes.contains_key(ARROW_FUNC_ATTR.as_str()) { continue; }
                put(hasher, b"fn", name.as_bytes());
                put(hasher, b"body", &bytes(func)?);
            } else if let Some(proto) = any.downcast_ref::<Prototype>() {
                put(hasher, b"proto", name.as_bytes());
                put(hasher, b"type", self.path_from(root, &proto.node).as_bytes());
            } else if let Some(field) = any.downcast_ref::<Field>() {
                let value = field.value.val.read();
                match &*value {
                    Val::Fn(func) if self.get_stof_data::<Func>(func).is_some_and(|f| f.attributes.contains_key(ARROW_FUNC_ATTR.as_str())) => {
                        put(hasher, b"field-arrow", name.as_bytes());
                        put(hasher, b"body", &bytes(self.get_stof_data::<Func>(func).unwrap())?);
                    },
                    Val::Fn(func) => {
                        put(hasher, b"field-fn", name.as_bytes());
                        let at = func.data(self).and_then(|f| f.nodes.iter().next().cloned());
                        let target = match (at, func.data_name(self)) {
                            (Some(at), Some(fname)) => format!("{}.{}", self.path_from(root, &at), fname),
                            _ => String::new(),
                        };
                        put(hasher, b"target", target.as_bytes());
                    },
                    Val::Obj(obj) => {
                        put(hasher, b"field-obj", name.as_bytes());
                        put(hasher, b"target", self.path_from(root, obj).as_bytes());
                    },
                    other if fields => {
                        put(hasher, b"field", name.as_bytes());
                        put(hasher, b"value", &self.value_bytes(root, other)?);
                    },
                    _ => {},
                }
                if fields { put(hasher, b"field-attrs", &self.attrs_bytes(root, &field.attributes)?); }
            }
        }

        let mut children: Vec<(String, NodeRef)> = node.children.iter()
            .map(|child| (child.node(self).map(|c| c.name.to_string()).unwrap_or_default(), child.clone()))
            .collect();
        children.sort_by(|a, b| a.0.cmp(&b.0));
        for (_, child) in children {
            self.digest_node(root, &child, fields, hasher, seen)?;
        }
        put(hasher, b"end", b"");
        Ok(())
    }

    /// Attributes for the digest: sorted, with values as `value_bytes` writes them.
    fn attrs_bytes(&self, root: &NodeRef, attributes: &FxHashMap<String, Val>) -> Result<Vec<u8>, Error> {
        let mut sorted: Vec<(&String, &Val)> = attributes.iter().collect();
        sorted.sort_by(|a, b| a.0.cmp(b.0));
        let mut out = Vec::new();
        for (name, value) in sorted {
            out.extend(bytes(name)?);
            out.extend(self.value_bytes(root, value)?);
        }
        Ok(out)
    }

    /// A value for the digest. Objects and functions are written by path (an arrow function by its body), never by
    /// ID (random per parse), and sets and maps in that order, so the same document always gives the same bytes.
    fn value_bytes(&self, root: &NodeRef, value: &Val) -> Result<Vec<u8>, Error> {
        let tagged = |tag: &str, body: Vec<u8>| -> Result<Vec<u8>, Error> { let mut out = bytes(tag)?; out.extend(bytes(&body)?); Ok(out) };
        match value {
            Val::Obj(obj) => tagged("obj", self.path_from(root, obj).into_bytes()),
            Val::Fn(func) | Val::Data(func) => match self.get_stof_data::<Func>(func) {
                Some(f) if f.attributes.contains_key(ARROW_FUNC_ATTR.as_str()) => tagged("arrow", bytes(f)?),
                _ => {
                    let at = func.data(self).and_then(|d| d.nodes.iter().next().cloned());
                    let name = func.data_name(self).map(|n| n.to_string()).unwrap_or_default();
                    let path = at.map(|at| self.path_from(root, &at)).unwrap_or_default();
                    tagged(if matches!(value, Val::Fn(_)) { "fn" } else { "data" }, format!("{path}.{name}").into_bytes())
                },
            },
            Val::List(items) | Val::Tup(items) => {
                let mut body = Vec::new();
                for item in items { body.extend(self.value_bytes(root, &item.read())?); }
                tagged(if matches!(value, Val::List(_)) { "list" } else { "tup" }, body)
            },
            Val::Set(items) => {
                let mut each = Vec::new();
                for item in items { each.push(self.value_bytes(root, &item.read())?); }
                each.sort();
                tagged("set", each.concat())
            },
            Val::Map(map) => {
                let mut each = Vec::new();
                for (k, v) in map {
                    let mut pair = self.value_bytes(root, &k.read())?;
                    pair.extend(self.value_bytes(root, &v.read())?);
                    each.push(pair);
                }
                each.sort();
                tagged("map", each.concat())
            },
            other => bytes(other),
        }
    }

    /// A path for the digest: relative to the sealed root when inside it (so where the root sits doesn't matter),
    /// else from the graph root.
    fn path_from(&self, root: &NodeRef, node: &NodeRef) -> String {
        let Some(path) = node.node_path(self, true) else { return String::new() };
        let names: Vec<String> = path.path.iter().map(|name| name.to_string()).collect();
        if node == root || node.child_of(self, root) {
            if let Some(root_path) = root.node_path(self, true) {
                return format!("~{}", names[root_path.path.len().min(names.len())..].join("."));
            }
        }
        names.join(".")
    }
}

/// Add a labeled, length-prefixed piece to the digest (so pieces can't run together).
fn put(hasher: &mut Sha256, label: &[u8], bytes: &[u8]) {
    hasher.update((label.len() as u64).to_le_bytes());
    hasher.update(label);
    hasher.update((bytes.len() as u64).to_le_bytes());
    hasher.update(bytes);
}

fn bytes<T: Serialize + ?Sized>(value: &T) -> Result<Vec<u8>, Error> {
    bincode::serialize(value).map_err(|error| Error::Sealed(format!("couldn't digest: {error}")))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn unhex(text: &str) -> Option<Vec<u8>> {
    let text = text.trim();
    if text.len() % 2 != 0 { return None; }
    (0..text.len()).step_by(2).map(|i| u8::from_str_radix(text.get(i..i + 2)?, 16).ok()).collect()
}


/*****************************************************************************
 * Signatures (Ed25519 over a digest).
 *****************************************************************************/

/// What's signed: the digest, labeled, so a signature over a digest can't be used for anything else.
fn signed_message(digest: &str) -> Vec<u8> {
    format!("stof-seal-v1:{}", digest.trim().to_ascii_lowercase()).into_bytes()
}

/// Key and signature bytes from base64 (standard or URL-safe) or hex.
pub fn key_bytes(text: &str) -> Option<Vec<u8>> {
    use base64::Engine;
    let text = text.trim();
    if let Some(bytes) = unhex(text) { if bytes.len() == 32 || bytes.len() == 64 { return Some(bytes); } }
    base64::engine::general_purpose::STANDARD.decode(text).ok()
        .or_else(|| base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(text.trim_end_matches('=')).ok())
}

fn base64(bytes: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

/// The public key (base64) for a secret key (base64 or hex, 32 bytes).
pub fn public_key(secret_key: &str) -> Result<String, Error> {
    let key = signing_key(secret_key)?;
    Ok(base64(key.verifying_key().as_bytes()))
}

/// Sign a digest with a secret key (base64 or hex, 32 bytes): the signature, base64.
pub fn sign(digest: &str, secret_key: &str) -> Result<String, Error> {
    use ed25519_dalek::Signer;
    let key = signing_key(secret_key)?;
    Ok(base64(&key.sign(&signed_message(digest)).to_bytes()))
}

/// Is this a valid signature (base64 or hex) over this digest by this public key (base64 or hex)?
pub fn verify(digest: &str, public_key: &str, signature: &str) -> bool {
    let (Some(key), Some(sig)) = (key_bytes(public_key), key_bytes(signature)) else { return false };
    let (Ok(key), Ok(sig)) = (<[u8; 32]>::try_from(key.as_slice()), <[u8; 64]>::try_from(sig.as_slice())) else { return false };
    let Ok(key) = ed25519_dalek::VerifyingKey::from_bytes(&key) else { return false };
    key.verify_strict(&signed_message(digest), &ed25519_dalek::Signature::from_bytes(&sig)).is_ok()
}

fn signing_key(secret_key: &str) -> Result<ed25519_dalek::SigningKey, Error> {
    let bytes = key_bytes(secret_key).ok_or_else(|| Error::Sealed("the secret key isn't base64 or hex".into()))?;
    let seed = <[u8; 32]>::try_from(bytes.as_slice()).map_err(|_| Error::Sealed("the secret key must be 32 bytes".into()))?;
    Ok(ed25519_dalek::SigningKey::from_bytes(&seed))
}

impl Graph {
    /// Seal an object (path or `<Type>`) only if its digest is signed by this public key: checks the signature
    /// over its current digest, then seals it. Returns the digest.
    pub fn seal_signed(&mut self, path: &str, fields: bool, public_key: &str, signature: &str) -> Result<String, Error> {
        let node = self.seal_target(path)?;
        let digest = self.digest(&node, fields)?;
        if !verify(&digest, public_key, signature) {
            return Err(Error::Sealed(format!("{path} isn't signed by this key: its digest {digest} doesn't match the signature")));
        }
        self.seal(&node, fields)
    }
}
