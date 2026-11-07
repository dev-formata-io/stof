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
//! A seal can also be:
//! - `removable`: the sealed object can still be removed whole (dropped), though nothing in it can change and it
//!   can't be moved or renamed. For code that comes and goes (Ex. a provider's capabilities, removed when access
//!   ends).
//! - confined (`reads`, `hosts`, `libs`, `time`, `calls`): code in the sealed part (and everything it calls) reaches
//!   only the sealed part itself and the objects listed in `reads`, changes only the sealed part, the Http library
//!   reaches only the listed `hosts`, it calls only the libraries listed in `libs` (host libraries included), it runs
//!   for at most `time` each time it's entered, and the functions in `calls` (gates) are its only way out: they run
//!   unconfined, as themselves. See `Confine`. For code from somewhere else that shouldn't see the rest of the
//!   document, or use what the host gave the document, except through the API the host chose.
//!
//! `Graph::digest(node)` is a SHA-256 over the sealed part's functions (names, parameters, return types,
//! attributes, and instructions) and structure (the objects, types, and the fields that hold functions or objects),
//! plus every field's value with `fields`. It's the same whether the document was parsed from source or loaded from
//! BSTF, and wherever the part sits (its own name and place aren't part of it), so a part sent to another document
//! has the same digest there. A host checks it against the digest it expects, or verifies an Ed25519 `signature`
//! over it (`verify`), Ex. for an engine shipped as a document. It's computed only when asked for (a few milliseconds
//! for a large part).
//!
//! Signatures (`sign`, `verify`, `generate_key`, `public_key`) are Ed25519 over a message's exact bytes, under a
//! context that names what they're for (Ex. "orders/v1"), so one made for one purpose is never valid for another. A
//! seal signature is one over the part's digest under "stof/seal" (`SEAL_CONTEXT`).

use std::{cell::Cell, collections::BTreeMap, sync::Arc};
use arcstr::ArcStr;
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use crate::{model::{ARROW_FUNC_ATTR, DataRef, Field, Func, Graph, NodeRef, Prototype, SId}, runtime::{Error, Val, ValRef, Variable, instruction::{Instruction, Instructions}, proc::ProcEnv}};


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
    /// Sealed objects (of `roots`) that may still be removed whole.
    pub removable: FxHashSet<NodeRef>,
    /// What the code in a sealed part may reach, by the object `seal` was called on (see `Confine`).
    pub confines: FxHashMap<NodeRef, Vec<Arc<Confine>>>,
    /// Type names of sealed types (no other type may take them).
    pub types: FxHashSet<String>,
    /// Why the last change was refused, until the instruction or host call that made it reports it.
    pub violation: Option<String>,
    /// Objects made by a change that was refused (Ex. `new {}` in a part sealed with its fields): they're removed when
    /// the refusal is reported, so a refused change never grows the document.
    pub refused: Vec<NodeRef>,
}


#[derive(Debug, Clone)]
/// How an object is sealed (`Graph::seal_info`).
pub struct SealInfo {
    /// The object `seal` was called on (this one, or one around it).
    pub part: NodeRef,
    /// Its fields are frozen.
    pub fields: bool,
    /// It can be removed whole (it's the part, sealed `removable`, and no other seal holds it).
    pub removable: bool,
    /// The confinements its code runs under (empty: not confined).
    pub confines: Vec<Arc<Confine>>,
}
impl SealInfo {
    /// As JSON: `{ part, fields, removable, confined: [{ reads, hosts, libs, time, calls }] }` (for hosts; `time` in
    /// ms, `calls` as function paths).
    pub fn to_json(&self, graph: &Graph) -> serde_json::Value {
        let confined: Vec<serde_json::Value> = self.confines.iter().map(|confine| {
            let mut hosts: Option<Vec<String>> = confine.hosts.as_ref().map(|hosts| hosts.iter().cloned().collect());
            if let Some(hosts) = &mut hosts { hosts.sort(); }
            let mut libs: Option<Vec<String>> = confine.libs.as_ref().map(|libs| libs.iter().cloned().collect());
            if let Some(libs) = &mut libs { libs.sort(); }
            let time = confine.time.map(|time| time.as_millis() as u64);
            let mut calls: Option<Vec<String>> = confine.calls.as_ref().map(|calls| calls.iter().map(|func| graph.func_path(func)).collect());
            if let Some(calls) = &mut calls { calls.sort(); }
            serde_json::json!({ "reads": confine.reads, "hosts": hosts, "libs": libs, "time": time, "calls": calls })
        }).collect();
        serde_json::json!({
            "part": self.part.node_path(graph, true).map(|path| path.join(".")).unwrap_or_default(),
            "fields": self.fields,
            "removable": self.removable,
            "confined": confined,
        })
    }

    /// As a Stof map: `part` (path), `fields`, `removable`, `confined` (a list of `{ reads, hosts, libs, time, calls }`
    /// maps, `time` in ms, `calls` as function paths).
    pub fn to_val(&self, graph: &Graph) -> Val {
        use imbl::{OrdMap, Vector};
        use crate::runtime::ValRef;
        let list = |items: &Option<Vec<String>>| match items {
            Some(items) => Val::List(items.iter().map(|item| ValRef::new(Val::Str(item.as_str().into()))).collect::<Vector<_>>()),
            None => Val::Null,
        };
        let mut confined = Vector::new();
        for confine in &self.confines {
            let mut hosts: Option<Vec<String>> = confine.hosts.as_ref().map(|hosts| hosts.iter().cloned().collect());
            if let Some(hosts) = &mut hosts { hosts.sort(); }
            let mut libs: Option<Vec<String>> = confine.libs.as_ref().map(|libs| libs.iter().cloned().collect());
            if let Some(libs) = &mut libs { libs.sort(); }
            let time = match confine.time {
                Some(time) => Val::Num(crate::runtime::Num::Units(time.as_millis() as f64, crate::runtime::Units::Milliseconds)),
                None => Val::Null,
            };
            let mut map = OrdMap::new();
            map.insert(ValRef::new(Val::Str("reads".into())), ValRef::new(list(&confine.reads)));
            map.insert(ValRef::new(Val::Str("hosts".into())), ValRef::new(list(&hosts)));
            map.insert(ValRef::new(Val::Str("libs".into())), ValRef::new(list(&libs)));
            map.insert(ValRef::new(Val::Str("time".into())), ValRef::new(time));
            let mut calls: Option<Vec<String>> = confine.calls.as_ref().map(|calls| calls.iter().map(|func| graph.func_path(func)).collect());
            if let Some(calls) = &mut calls { calls.sort(); }
            map.insert(ValRef::new(Val::Str("calls".into())), ValRef::new(list(&calls)));
            confined.push_back(ValRef::new(Val::Map(map)));
        }
        let part = self.part.node_path(graph, true).map(|path| path.join(".")).unwrap_or_default();
        let mut map = OrdMap::new();
        map.insert(ValRef::new(Val::Str("part".into())), ValRef::new(Val::Str(part.into())));
        map.insert(ValRef::new(Val::Str("fields".into())), ValRef::new(Val::Bool(self.fields)));
        map.insert(ValRef::new(Val::Str("removable".into())), ValRef::new(Val::Bool(self.removable)));
        map.insert(ValRef::new(Val::Str("confined".into())), ValRef::new(Val::List(confined)));
        Val::Map(map)
    }
}


#[derive(Debug, Clone, Default)]
/// How to seal (see the module docs). `Default` is a plain seal: functions frozen, fields free.
pub struct SealOptions {
    /// Freeze every field too.
    pub fields: bool,
    /// The sealed object may still be removed whole.
    pub removable: bool,
    /// Confine the part's code to itself and the objects at these paths (or types as `<Name>`), looked up each time
    /// the part is entered, so an object replaced at a path is still the one it reads (Some, even empty: it reads
    /// only itself).
    pub reads: Option<Vec<String>>,
    /// Confine the part's network access to these hosts (Some, even empty: no network).
    pub hosts: Option<Vec<String>>,
    /// Confine the part's code to these libraries, by name (Ex. "Str", "Http", or a host's own library), host
    /// libraries included (Some, even empty: no library calls at all).
    pub libs: Option<Vec<String>>,
    /// Confine how long the part's code runs each time it's entered (and everything it calls).
    pub time: Option<web_time::Duration>,
    /// Gates: functions outside the part its code may call, which run unconfined, as themselves (the part's only way
    /// to anything else; see `Confine`). The functions themselves, not their paths: a function later put at the same
    /// path isn't a gate. `Graph::seal_calls` finds them by path.
    pub calls: Option<Vec<DataRef>>,
}


#[derive(Debug, Clone)]
/// What the code in a sealed part may reach, while it runs (and everything it calls while it runs).
///
/// With `reads`, the code finds only the sealed part and the listed objects (and everything under them): a path to
/// anything else finds nothing, calling a function elsewhere fails, and so does handing an object elsewhere to a
/// library function (Ex. `stringify`), or a library function handing one back (Ex. `Obj.parent`). It changes only
/// the sealed part (the listed objects are read-only to it) and makes no new roots. With `hosts`, `Http.fetch`
/// reaches only those hosts (and the document's own allowlist still applies). With `libs`, it calls only the listed
/// libraries (Ex. `Str`, `Http`), the host's own libraries included, so a document's code can't use a host function
/// (Ex. one that signs with the host's key) it wasn't given. With `time`, it runs for at most that long each time
/// it's entered: wall-clock time from entering, waits included (Ex. an `Http.fetch`), checked as it runs, like the
/// process's own limit.
///
/// With `calls` (gates), it may call those functions outside its reach, and they run unconfined, as themselves: a
/// controlled API the host gives it (Ex. one that signs a request for it, which it couldn't do itself). The host hands
/// them to it as values (a field of the part holding the function, or an argument), since a path to them finds nothing.
/// A gate is a door: it runs with everything the document has, so it must check what it's given. A call is a gate's
/// only when every confinement the code runs under lists it, so a part can't make one of its own (sealing itself
/// again with `calls` only narrows). Code a gate calls in a confined part (Ex. a function value it was handed) runs
/// confined again.
///
/// Objects out of its reach stay opaque to it, even ones it's handed (an argument, or a field the host put in it):
/// it can hold one and hand it back, but not look inside, change it, or export it. Give it values (strings, numbers,
/// lists, maps), or list the objects in `reads`. Confined code calling code that's confined too runs under both.
pub struct Confine {
    /// The object `seal` was called on.
    pub root: NodeRef,
    /// Paths (or `<Type>`s) of what the code may read besides its own part (None: reads and changes aren't limited).
    pub reads: Option<Vec<String>>,
    /// Hosts `Http.fetch` may reach (None: not limited beyond the document's allowlist).
    pub hosts: Option<Arc<FxHashSet<String>>>,
    /// Libraries the code may call (None: any).
    pub libs: Option<Arc<FxHashSet<String>>>,
    /// How long the code may run each time the part is entered (None: only the process's own limit).
    pub time: Option<web_time::Duration>,
    /// Gates: functions outside its reach the code may call, which run unconfined (None: none).
    pub calls: Option<Arc<FxHashSet<DataRef>>>,
}
impl Confine {
    /// What it reaches now: its `reads` paths looked up (a path with nothing there reaches nothing), and when its
    /// time runs out.
    pub fn resolve(&self, graph: &Graph, owner: SId) -> Reach {
        Reach {
            owner,
            ended: Default::default(),
            root: self.root.clone(),
            reads: self.reads.as_ref().map(|paths| paths.iter().filter_map(|path| graph.seal_target(path).ok()).collect()),
            hosts: self.hosts.clone(),
            libs: self.libs.clone(),
            deadline: self.time.map(|time| web_time::Instant::now() + time),
            calls: self.calls.clone(),
        }
    }
}


#[derive(Debug, Clone)]
/// A `Confine` while its code runs: the objects its `reads` paths point to now.
pub struct Reach {
    /// The process that entered the part (and leaves it).
    pub owner: SId,
    /// Has that process left the part? A process the part's code spawned (Ex. an `async` block it didn't await) still
    /// runs under this, and ends at its next instruction when it has: confined code runs only while it's called.
    pub ended: Arc<std::sync::atomic::AtomicBool>,
    /// The object `seal` was called on.
    pub root: NodeRef,
    /// What the code may read besides its own part (None: reads and changes aren't limited).
    pub reads: Option<Vec<NodeRef>>,
    /// Hosts `Http.fetch` may reach.
    pub hosts: Option<Arc<FxHashSet<String>>>,
    /// Libraries the code may call (None: any).
    pub libs: Option<Arc<FxHashSet<String>>>,
    /// When the code's time runs out (None: no limit of its own).
    pub deadline: Option<web_time::Instant>,
    /// Gates: functions outside its reach the code may call, which run unconfined.
    pub calls: Option<Arc<FxHashSet<DataRef>>>,
}
impl Reach {
    /// Is this function one of its gates?
    pub fn gate(&self, func: &DataRef) -> bool {
        self.calls.as_ref().is_some_and(|calls| calls.contains(func))
    }

    /// May the code read this data (on an object it may read)?
    pub fn reaches_data(&self, graph: &Graph, dref: &DataRef) -> bool {
        dref.data(graph).is_some_and(|data| data.nodes.iter().any(|node| self.reaches(graph, node)))
    }

    /// May the code call this library?
    pub fn lib_allowed(&self, lib: &str) -> bool {
        match &self.libs {
            Some(libs) => libs.contains(lib),
            None => true,
        }
    }

    /// Has the process that entered the part left it (see `ended`)?
    pub fn has_ended(&self) -> bool {
        self.ended.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Has the code run past its time limit?
    pub fn out_of_time(&self) -> bool {
        self.deadline.is_some_and(|deadline| web_time::Instant::now() > deadline)
    }

    /// May the code read this object?
    pub fn reaches(&self, graph: &Graph, node: &NodeRef) -> bool {
        let Some(reads) = &self.reads else { return true };
        self.changes(graph, node) || reads.iter().any(|read| node == read || node.child_of(graph, read))
    }

    /// May the code change this object (its own part)?
    pub fn changes(&self, graph: &Graph, node: &NodeRef) -> bool {
        self.reads.is_none() || node == &self.root || node.child_of(graph, &self.root)
    }

    /// May the code send a request to this URL's host?
    pub fn host_allowed(&self, url: &str) -> bool {
        let Some(hosts) = &self.hosts else { return true };
        host_listed(url, hosts)
    }
}

/// The objects a value holds (in lists, maps, ... too).
fn held_objects(val: &Val, out: &mut Vec<NodeRef>, depth: usize) {
    if depth > 64 { return; }
    match val {
        Val::Obj(nref) => out.push(nref.clone()),
        Val::List(items) | Val::Tup(items) => for item in items { held_objects(&item.read(), out, depth + 1); },
        Val::Set(items) => for item in items { held_objects(&item.read(), out, depth + 1); },
        Val::Map(map) => for (k, v) in map { held_objects(&k.read(), out, depth + 1); held_objects(&v.read(), out, depth + 1); },
        _ => {},
    }
}

/// Is this URL's host (or host and port) in this list?
pub fn host_listed(url: &str, hosts: &FxHashSet<String>) -> bool {
    let Some((host, port)) = url_host(url) else { return false };
    if hosts.contains(&host) { return true; }
    match port {
        Some(port) => hosts.contains(&format!("{host}:{port}")),
        None => false,
    }
}

/// A URL's host (lowercase) and port (or the scheme's default), without a URL parser (every build has this).
fn url_host(url: &str) -> Option<(String, Option<u16>)> {
    let (scheme, rest) = url.trim().split_once("://")?;
    let authority = rest.split(|c| c == '/' || c == '?' || c == '#').next()?;
    let authority = authority.rsplit_once('@').map(|(_, host)| host).unwrap_or(authority);
    let (host, port) = if let Some(rest) = authority.strip_prefix('[') {
        let (host, after) = rest.split_once(']')?;
        (format!("[{host}]"), after.strip_prefix(':'))
    } else {
        match authority.rsplit_once(':') {
            Some((host, port)) => (host.to_string(), Some(port)),
            None => (authority.to_string(), None),
        }
    };
    if host.is_empty() { return None; }
    let port = match port {
        Some(port) => Some(port.parse::<u16>().ok()?),
        None => match scheme.to_ascii_lowercase().as_str() { "http" | "ws" => Some(80), "https" | "wss" => Some(443), _ => None },
    };
    Some((host.to_lowercase(), port))
}


impl Graph {
    /// Seal an object and everything under it (see the module docs), with its fields frozen too when `fields`.
    /// Objects added under it later are not sealed. Returns the part's digest.
    pub fn seal(&mut self, node: &NodeRef, fields: bool) -> Result<String, Error> {
        self.seal_with(node, &SealOptions { fields, ..Default::default() })
    }

    /// Seal an object and everything under it with these options (see `SealOptions`). Sealing again only adds to a
    /// seal: frozen fields stay frozen, an object sealed without `removable` stays that way, and confinements add up.
    /// Returns the part's digest.
    pub fn seal_with(&mut self, node: &NodeRef, options: &SealOptions) -> Result<String, Error> {
        if !node.node_exists(self) { return Err(Error::Sealed("there's no object to seal".into())); }
        // gates are functions outside the part: one inside it would run the part's own code unconfined
        for gate in options.calls.iter().flatten() {
            let Some(data) = gate.data(self) else { return Err(Error::Sealed("a seal's gate (calls) must be a function in the document".into())) };
            if !gate.type_of::<crate::model::Func>(self) { return Err(Error::Sealed("a seal's gate (calls) must be a function".into())); }
            if data.nodes.iter().any(|on| on == node || on.child_of(self, node)) {
                let name = gate.data_name(self).map(|name| name.to_string()).unwrap_or_default();
                return Err(Error::Sealed(format!("a seal's gate (calls) must be outside the part it's for ('{name}' is inside it, and would run its code unconfined)")));
            }
        }
        let digest = self.digest(node, options.fields)?;
        let nodes = self.all_child_nodes(node, true);
        let mut types = FxHashSet::default();
        for (name, typed) in &self.typemap {
            if typed.iter().any(|ty| nodes.contains(ty)) { types.insert(name.clone()); }
        }
        let confine = if options.reads.is_some() || options.hosts.is_some() || options.libs.is_some() || options.time.is_some() || options.calls.is_some() {
            Some(Arc::new(Confine {
                root: node.clone(),
                reads: options.reads.clone(),
                hosts: options.hosts.as_ref().map(|hosts| Arc::new(hosts.iter().map(|host| host.trim().to_lowercase()).collect())),
                libs: options.libs.as_ref().map(|libs| Arc::new(libs.iter().map(|lib| lib.trim().to_string()).collect())),
                time: options.time,
                calls: options.calls.as_ref().map(|calls| Arc::new(calls.iter().cloned().collect())),
            }))
        } else {
            None
        };
        let seals = Arc::make_mut(&mut self.seals);
        for nref in nodes {
            let frozen = seals.nodes.get(&nref).copied().unwrap_or(false) || options.fields;
            seals.nodes.insert(nref, frozen);
        }
        let sealed_before = !seals.roots.insert(node.clone());
        if options.removable {
            if !sealed_before { seals.removable.insert(node.clone()); }
        } else {
            seals.removable.remove(node);
        }
        if let Some(confine) = confine {
            seals.confines.entry(node.clone()).or_default().push(confine);
        }
        seals.types.extend(types);
        Ok(digest)
    }

    /// Seal an object (by path, or a type as `<Name>`), checking first that its digest is `expected` when given.
    pub fn seal_path(&mut self, path: &str, fields: bool, expected: Option<&str>) -> Result<String, Error> {
        self.seal_path_with(path, &SealOptions { fields, ..Default::default() }, expected)
    }

    /// Seal an object (by path, or a type as `<Name>`) with these options, checking first that its digest is
    /// `expected` when given.
    pub fn seal_path_with(&mut self, path: &str, options: &SealOptions, expected: Option<&str>) -> Result<String, Error> {
        let node = self.seal_target(path)?;
        if let Some(expected) = expected {
            let digest = self.digest(&node, options.fields)?;
            if !digest.eq_ignore_ascii_case(expected.trim()) {
                return Err(Error::Sealed(format!("{path} isn't what was expected: its digest is {digest}, not {expected}")));
            }
        }
        self.seal_with(&node, options)
    }

    /// The objects for host paths: a path, or a type as `<Name>`.
    pub fn seal_targets(&self, paths: &[String]) -> Result<Vec<NodeRef>, Error> {
        paths.iter().map(|path| self.seal_target(path)).collect()
    }

    /// How an object is sealed (None when it isn't): see `SealInfo`.
    pub fn seal_info(&self, node: &NodeRef) -> Option<SealInfo> {
        let fields = *self.seals.nodes.get(node)?;
        let mut part = None;
        let mut confines: Vec<Arc<Confine>> = Vec::new();
        let mut current = Some(node.clone());
        while let Some(nref) = current {
            if part.is_none() && self.seals.roots.contains(&nref) { part = Some(nref.clone()); }
            if let Some(found) = self.seals.confines.get(&nref) { confines.extend(found.iter().cloned()); }
            current = nref.node_parent(self);
        }
        let part = part?;
        let removable = &part == node && self.removable_whole(node);
        Some(SealInfo { part, fields, removable, confines })
    }

    /// How an object is sealed, by path (or a type as `<Name>`): None when it isn't.
    pub fn seal_info_path(&self, path: &str) -> Result<Option<SealInfo>, Error> {
        let node = self.seal_target(path)?;
        Ok(self.seal_info(&node))
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
        seals.removable.remove(node);
        seals.confines.remove(node);
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
        let confined = std::mem::take(&mut self.confined);
        let res = f(self);
        let violation = self.take_seal_violation();
        self.seals = seals;
        self.confined = confined;
        if violation.is_some() { Arc::make_mut(&mut self.seals).violation = violation; }
        res
    }

    /// Forget the seals on objects that were removed from the graph (after `remove_node`).
    pub(crate) fn forget_seals(&mut self, removed: &[NodeRef]) {
        if self.seals.nodes.is_empty() || !removed.iter().any(|nref| self.seals.nodes.contains_key(nref)) { return; }
        let seals = Arc::make_mut(&mut self.seals);
        for nref in removed {
            seals.nodes.remove(nref);
            seals.roots.remove(nref);
            seals.removable.remove(nref);
            seals.confines.remove(nref);
        }
        let types = self.typemap.iter()
            .filter(|(_, typed)| typed.iter().any(|ty| self.seals.nodes.contains_key(ty)))
            .map(|(name, _)| name.clone())
            .collect();
        Arc::make_mut(&mut self.seals).types = types;
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
        let seals = Arc::make_mut(&mut self.seals);
        let why = seals.violation.take();
        let refused = std::mem::take(&mut seals.refused);
        for nref in refused { self.discard_refused_node(&nref); }
        why
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

    /// Would a change to this field be refused: it's sealed, or outside what the running code may change?
    /// Values read from such a field are copies (see `Graph::field_var`), so changing a list, map, set, or tuple
    /// in place (Ex. `push_back`) can't change the field behind the seal checks. Never refuses.
    pub(crate) fn guarded(&self, dref: &DataRef) -> bool {
        if self.seals.nodes.is_empty() { return false; }
        if !self.confined.is_empty() && !dref.data(self).is_some_and(|data| data.nodes.iter().all(|node| self.confined_changes(node))) {
            return true;
        }
        self.sealed_anywhere(dref)
    }

    /// The variable to load for a field's value: the value itself (by reference when asked), or a copy of it when
    /// the field is guarded (sealed, or outside what the running code may change).
    pub fn field_var(&self, dref: &DataRef, value: &Variable, by_ref: bool) -> Variable {
        if !self.guarded(dref) { return value.stack_var(by_ref); }
        let mut copy = value.clone();
        copy.val = ValRef::new(value.val.read().detached());
        copy.shared = false;
        copy
    }

    /// The value to load for a field (see `field_var`).
    pub fn field_val(&self, dref: &DataRef, value: &Variable, by_ref: bool) -> ValRef<Val> {
        if !self.guarded(dref) { return value.val.duplicate(by_ref); }
        ValRef::new(value.val.read().detached())
    }

    /*****************************************************************************
     * Confinement: what the running code may reach (see `Confine`).
     *****************************************************************************/

    #[inline]
    /// May the running code read this object (always, unless confined)?
    pub fn confined_reaches(&self, node: &NodeRef) -> bool {
        self.confined.is_empty() || self.confined.iter().all(|confine| confine.reaches(self, node))
    }

    /// May the running code read this data (a field or function: on an object it may read)?
    pub fn confined_reaches_data(&self, dref: &DataRef) -> bool {
        if self.confined.is_empty() { return true; }
        dref.data(self).is_some_and(|data| data.nodes.iter().any(|node| self.confined_reaches(node)))
    }

    #[inline]
    /// May the running code change this object (always, unless confined)?
    pub fn confined_changes(&self, node: &NodeRef) -> bool {
        self.confined.is_empty() || self.confined.iter().all(|confine| confine.changes(self, node))
    }

    /// May the running code read this value: every object and function in it (lists, maps, ... too)?
    pub fn confined_reaches_val(&self, val: &Val) -> bool {
        if self.confined.is_empty() { return true; }
        self.reaches_val(val, 0)
    }

    fn reaches_val(&self, val: &Val, depth: usize) -> bool {
        if depth > 64 { return true; } // values nested that deep hold no new references worth the walk
        match val {
            Val::Obj(nref) => self.confined_reaches(nref),
            Val::Fn(dref) | Val::Data(dref) => self.confined_reaches_data(dref),
            Val::List(items) | Val::Tup(items) => items.iter().all(|item| self.reaches_val(&item.read(), depth + 1)),
            Val::Set(items) => items.iter().all(|item| self.reaches_val(&item.read(), depth + 1)),
            Val::Map(map) => map.iter().all(|(k, v)| self.reaches_val(&k.read(), depth + 1) && self.reaches_val(&v.read(), depth + 1)),
            _ => true,
        }
    }

    /// May the running code send a request to this URL (the confined parts' hosts)?
    pub fn confined_host(&self, url: &str) -> bool {
        self.confined.iter().all(|confine| confine.host_allowed(url))
    }

    /// The host lists of the confined parts running now (each must allow a request).
    pub fn confined_hosts(&self) -> Vec<Arc<FxHashSet<String>>> {
        self.confined.iter().filter_map(|confine| confine.hosts.clone()).collect()
    }

    /// The confinements of the sealed parts a function belongs to (it's on an object in or under them).
    pub fn func_confines(&self, func: &DataRef) -> Vec<Arc<Confine>> {
        let mut found: Vec<Arc<Confine>> = Vec::new();
        if self.seals.confines.is_empty() { return found; }
        let Some(data) = func.data(self) else { return found };
        for node in &data.nodes {
            let mut current = Some(node.clone());
            while let Some(nref) = current {
                if let Some(confines) = self.seals.confines.get(&nref) {
                    for confine in confines {
                        if !found.iter().any(|existing| Arc::ptr_eq(existing, confine)) { found.push(confine.clone()); }
                    }
                }
                current = nref.node_parent(self);
            }
        }
        found
    }

    /// May the running code export this object: it, and every object it holds (as children or in fields), within
    /// reach? Exporting everything (no object) isn't, when confined to what it reads.
    pub fn confined_export(&self, node: &Option<NodeRef>) -> Result<(), Error> {
        if self.confined.is_empty() || self.confined.iter().all(|confine| confine.reads.is_none()) { return Ok(()); }
        let Some(node) = node else {
            return Err(Error::Sealed("this sealed part can only export what it may read: give it an object".into()));
        };
        let mut seen = FxHashSet::default();
        let mut pending = vec![node.clone()];
        while let Some(nref) = pending.pop() {
            if !seen.insert(nref.clone()) { continue; }
            if !self.confined_reaches(&nref) {
                return Err(Error::Sealed("this sealed part can't export an object outside what it may read (or one holding it)".into()));
            }
            let Some(current) = nref.node(self) else { continue };
            pending.extend(current.children.iter().cloned());
            for dref in current.data.values() {
                if let Some(field) = self.get_stof_data::<Field>(dref) {
                    let value = field.value.val.read();
                    // functions and data held as values (not the object's own) must be within reach too
                    if !self.confined_reaches_val(&value) {
                        return Err(Error::Sealed("this sealed part can't export an object holding something outside what it may read".into()));
                    }
                    held_objects(&value, &mut pending, 0);
                }
            }
        }
        Ok(())
    }

    /// Refuse a change outside the confined part (true when it's allowed).
    fn confined_change(&mut self, node: &NodeRef) -> bool {
        if self.confined_changes(node) { return true; }
        let name = node.node(self).map(|n| n.name.to_string()).unwrap_or_default();
        self.refuse(format!("'{name}' is outside what this sealed part may change"))
    }

    /// Refuse a change to data outside the confined part (true when it's allowed).
    fn confined_change_data(&mut self, dref: &DataRef, node: Option<&NodeRef>) -> bool {
        if self.confined.is_empty() { return true; }
        let allowed = match node {
            Some(node) => self.confined_changes(node),
            None => dref.data(self).is_some_and(|data| data.nodes.iter().all(|node| self.confined_changes(node))),
        };
        if allowed { return true; }
        let name = dref.data_name(self).map(|name| name.to_string()).unwrap_or_default();
        self.refuse(format!("'{name}' is outside what this sealed part may change"))
    }

    /// May a new root be made (not by confined code)? Refuses when not.
    pub(crate) fn may_add_root(&mut self) -> bool {
        if self.confined.iter().all(|confine| confine.reads.is_none()) { return true; }
        self.refuse("this sealed part can't make new roots".into())
    }

    /// May this data be put on this node (as `name`)? Refuses (see `refuse`) when not.
    pub(crate) fn may_insert(&mut self, node: &NodeRef, name: &str, data: &crate::model::Data) -> bool {
        if self.seals.nodes.is_empty() { return true; }
        if !self.confined.is_empty() && !self.confined_change(node) { return false; }
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
        if !self.confined_change_data(dref, node) { return false; }
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
        if self.seals.nodes.is_empty() { return true; }
        if !self.confined_change_data(dref, None) { return false; }
        if !self.sealed_anywhere(dref) { return true; }
        let name = dref.data_name(self).map(|name| name.to_string()).unwrap_or_default();
        self.refuse(format!("'{name}' is sealed and can't be changed"))
    }

    /// May this object be removed? Not when it's sealed or holds something sealed, unless it was sealed
    /// `removable` (and nothing else seals it or anything under it). Refuses when not.
    pub(crate) fn may_remove_node(&mut self, node: &NodeRef) -> bool {
        if self.seals.nodes.is_empty() { return true; }
        if !self.confined.is_empty() && !self.confined_change(node) { return false; }
        if self.removable_whole(node) || self.holds_only_removable(node) { return true; }
        self.may_move_node(node)
    }

    /// Is this object unsealed, with every seal inside it removable (Ex. a plugin's container)? Removing it removes
    /// them, as dropping each would.
    fn holds_only_removable(&self, node: &NodeRef) -> bool {
        !self.seals.nodes.contains_key(node)
            && !self.seals.roots.iter().any(|root| node.child_of(self, root))
            && self.seals.roots.iter().filter(|root| root.child_of(self, node)).all(|root| self.seals.removable.contains(root))
    }

    /// Was this object sealed `removable`, with no other seal covering it (one around it, or one inside it that
    /// isn't removable)?
    fn removable_whole(&self, node: &NodeRef) -> bool {
        self.seals.removable.contains(node)
            && !self.seals.roots.iter().any(|root| root != node && node.child_of(self, root))
            && !self.seals.roots.iter().any(|root| root != node && root.child_of(self, node) && !self.seals.removable.contains(root))
    }

    /// May this object be removed, moved, or renamed (it isn't sealed and holds nothing sealed)? Refuses when not.
    pub(crate) fn may_move_node(&mut self, node: &NodeRef) -> bool {
        if self.seals.nodes.is_empty() { return true; }
        if !self.confined.is_empty() && !self.confined_change(node) { return false; }
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
        if !self.confined.is_empty() && !self.confined_change(node) { return false; }
        if !self.is_sealed(node) { return true; }
        let name = node.node(self).map(|n| n.name.to_string()).unwrap_or_default();
        self.refuse(format!("'{name}' is sealed and its attributes can't change"))
    }

    /// May a child object be added to this one (not when its fields are frozen)? Refuses when not.
    pub(crate) fn may_add_child(&mut self, parent: &NodeRef) -> bool {
        if !self.confined.is_empty() && !self.confined_change(parent) { return false; }
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
        hasher.update(b"stof-seal-v2");
        let res = self.digest_node(node, node, "~", fields, &mut hasher, &mut FxHashSet::default());
        DIGESTING.with(|digesting| digesting.set(previous));
        res?;
        Ok(hex(&hasher.finalize()))
    }

    /// An object for the digest, by `name`: the field that holds it (objects made with `new` have random names),
    /// and "~" for the part itself (its own name isn't part of it: where it's put decides that).
    fn digest_node(&self, root: &NodeRef, nref: &NodeRef, name: &str, fields: bool, hasher: &mut Sha256, seen: &mut FxHashSet<NodeRef>) -> Result<(), Error> {
        if !seen.insert(nref.clone()) { return Ok(()); }
        let Some(node) = nref.node(self) else { return Ok(()) };
        put(hasher, b"node", name.as_bytes());
        put(hasher, b"attrs", &self.attrs_bytes(root, &without_internal(&node.attributes), seen)?);

        // Objects this one holds as fields: the same whether a field was written for it (Ex. parsed from Stof) or
        // it's a field object without one yet (Ex. from JSON, given a field when first looked up)
        let mut objects: BTreeMap<String, FxHashMap<String, Val>> = BTreeMap::new();
        for child in &node.children {
            if let Some(child_node) = child.node(self) {
                if child_node.is_field() { objects.insert(child_node.name.to_string(), without_internal(&child_node.attributes)); }
            }
        }

        let mut held: FxHashMap<NodeRef, String> = FxHashMap::default(); // a child object's name: its field's
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
                            (Some(at), Some(fname)) => format!("{}fn{}:{fname}", self.path_from(root, &at), fname.len()),
                            _ => String::new(),
                        };
                        put(hasher, b"target", target.as_bytes());
                    },
                    Val::Obj(obj) if obj.node_parent(self).as_ref() == Some(nref) => {
                        objects.insert(name.clone(), without_internal(&field.attributes));
                        held.entry(obj.clone()).or_insert_with(|| name.clone());
                        continue;
                    },
                    Val::Obj(obj) => {
                        put(hasher, b"field-obj", name.as_bytes());
                        put(hasher, b"target", self.path_from(root, obj).as_bytes());
                    },
                    other if fields => {
                        put(hasher, b"field", name.as_bytes());
                        put(hasher, b"value", &self.value_bytes(root, other, seen)?);
                    },
                    _ => {},
                }
                if fields { put(hasher, b"field-attrs", &self.attrs_bytes(root, &field.attributes, seen)?); }
            }
        }
        for (name, attributes) in objects {
            put(hasher, b"field-child", name.as_bytes());
            if fields { put(hasher, b"field-attrs", &self.attrs_bytes(root, &attributes, seen)?); }
        }

        let mut children: Vec<(String, NodeRef)> = node.children.iter()
            .map(|child| (held.get(child).cloned().unwrap_or_else(|| child.node(self).map(|c| c.name.to_string()).unwrap_or_default()), child.clone()))
            .collect();
        children.sort_by(|a, b| a.0.cmp(&b.0));
        for (name, child) in children {
            self.digest_node(root, &child, &name, fields, hasher, seen)?;
        }
        put(hasher, b"end", b"");
        Ok(())
    }

    /// Attributes for the digest: sorted, with values as `value_bytes` writes them.
    fn attrs_bytes(&self, root: &NodeRef, attributes: &FxHashMap<String, Val>, seen: &mut FxHashSet<NodeRef>) -> Result<Vec<u8>, Error> {
        let mut sorted: Vec<(&String, &Val)> = attributes.iter().collect();
        sorted.sort_by(|a, b| a.0.cmp(b.0));
        let mut out = Vec::new();
        for (name, value) in sorted {
            out.extend(bytes(name)?);
            out.extend(self.value_bytes(root, value, seen)?);
        }
        Ok(out)
    }

    /// A value for the digest. Objects and functions are written by path (an arrow function by its body), never by
    /// ID (random per parse), and sets and maps in that order, so the same document always gives the same bytes.
    /// An object inside the part that a value holds (Ex. objects in a list, from JSON) is written by its contents:
    /// its name is random, so its path would be too. It's then seen, so it isn't digested again as a child.
    fn value_bytes(&self, root: &NodeRef, value: &Val, seen: &mut FxHashSet<NodeRef>) -> Result<Vec<u8>, Error> {
        let tagged = |tag: &str, body: Vec<u8>| -> Result<Vec<u8>, Error> { let mut out = bytes(tag)?; out.extend(bytes(&body)?); Ok(out) };
        match value {
            Val::Obj(obj) if obj != root && obj.child_of(self, root) => {
                if seen.contains(obj) { return tagged("obj-seen", Vec::new()); } // a cycle, or already written
                let mut sub = Sha256::new();
                self.digest_node(root, obj, "~", true, &mut sub, seen)?;
                tagged("obj-in", sub.finalize().to_vec())
            },
            Val::Obj(obj) => tagged("obj", self.path_from(root, obj).into_bytes()),
            Val::Fn(func) | Val::Data(func) => match self.get_stof_data::<Func>(func) {
                Some(f) if f.attributes.contains_key(ARROW_FUNC_ATTR.as_str()) => tagged("arrow", bytes(f)?),
                _ => {
                    let at = func.data(self).and_then(|d| d.nodes.iter().next().cloned());
                    let name = func.data_name(self).map(|n| n.to_string()).unwrap_or_default();
                    let path = at.map(|at| self.path_from(root, &at)).unwrap_or_default();
                    tagged(if matches!(value, Val::Fn(_)) { "fn" } else { "data" }, format!("{path}fn{}:{name}", name.len()).into_bytes())
                },
            },
            Val::List(items) | Val::Tup(items) => {
                let mut body = Vec::new();
                for item in items { body.extend(self.value_bytes(root, &item.read(), seen)?); }
                tagged(if matches!(value, Val::List(_)) { "list" } else { "tup" }, body)
            },
            Val::Set(items) => {
                let mut each = Vec::new();
                for item in items { each.push(self.value_bytes(root, &item.read(), seen)?); }
                each.sort();
                tagged("set", each.concat())
            },
            Val::Map(map) => {
                let mut each = Vec::new();
                for (k, v) in map {
                    let mut pair = self.value_bytes(root, &k.read(), seen)?;
                    pair.extend(self.value_bytes(root, &v.read(), seen)?);
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
    /// A node's path for the digest, unambiguous: "in" and the names under the part for nodes inside it, "out" and
    /// the full path otherwise, each name length-prefixed (names can hold any character, dots included).
    fn path_from(&self, root: &NodeRef, node: &NodeRef) -> String {
        let Some(path) = node.node_path(self, true) else { return String::new() };
        let names: Vec<String> = path.path.iter().map(|name| name.to_string()).collect();
        let encode = |names: &[String]| names.iter().map(|name| format!("{}:{name}", name.len())).collect::<String>();
        if node == root || node.child_of(self, root) {
            if let Some(root_path) = root.node_path(self, true) {
                return format!("in{}", encode(&names[root_path.path.len().min(names.len())..]));
            }
        }
        format!("out{}", encode(&names))
    }
}

/// Attributes without the ones Stof keeps for itself (a field object's marker, and the no-export mark on the field
/// made for one when it's first looked up), which say how an object was made, not what it is.
fn without_internal(attributes: &FxHashMap<String, Val>) -> FxHashMap<String, Val> {
    if !attributes.contains_key("field") && !attributes.contains_key("no-export") { return attributes.clone(); }
    attributes.iter()
        .filter(|(name, _)| name.as_str() != "field" && name.as_str() != "no-export")
        .map(|(name, value)| (name.clone(), value.clone()))
        .collect()
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
 * Keys and signatures (Ed25519).
 *
 * One scheme for everything: a signature is over a message's exact bytes, under a context that names what it's
 * for (Ex. "orders/v1"), so it's only valid for that purpose. A seal signature is a signature over the part's
 * digest under the context "stof/seal" (`SEAL_CONTEXT`). Keys and signatures are base64 strings (hex is read too).
 * The same functions are the `Key` library in Stof and `StofDoc` / `Doc` functions on hosts.
 *****************************************************************************/

/// The context seal signatures are made under: `sign(secret, SEAL_CONTEXT, digest)`.
pub const SEAL_CONTEXT: &str = "stof/seal";

/// What's signed: labeled, then the context (length-prefixed), then the exact message bytes.
fn signed_bytes(context: &str, message: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(32 + context.len() + message.len());
    out.extend_from_slice(b"stof-signature-v1");
    out.extend_from_slice(&(context.len() as u64).to_le_bytes());
    out.extend_from_slice(context.as_bytes());
    out.extend_from_slice(message);
    out
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

/// A new secret key (base64, 32 bytes from the system's secure random source). Share only its `public_key`.
pub fn generate_key() -> String {
    base64(&nanoid::rngs::default(32))
}

/// The public key (base64) for a secret key.
pub fn public_key(secret_key: &str) -> Result<String, Error> {
    let key = signing_key(secret_key)?;
    Ok(base64(key.verifying_key().as_bytes()))
}

/// Sign a message (its exact bytes) under a context with a secret key: the signature (base64).
pub fn sign(secret_key: &str, context: &str, message: &[u8]) -> Result<String, Error> {
    use ed25519_dalek::Signer;
    let key = signing_key(secret_key)?;
    Ok(base64(&key.sign(&signed_bytes(context, message)).to_bytes()))
}

/// Is this a valid signature over this message, under this context, by this public key? False for anything that
/// isn't (a malformed key or signature included).
pub fn verify(public_key: &str, context: &str, message: &[u8], signature: &str) -> bool {
    let (Some(key), Some(sig)) = (key_bytes(public_key), key_bytes(signature)) else { return false };
    let (Ok(key), Ok(sig)) = (<[u8; 32]>::try_from(key.as_slice()), <[u8; 64]>::try_from(sig.as_slice())) else { return false };
    let Ok(key) = ed25519_dalek::VerifyingKey::from_bytes(&key) else { return false };
    key.verify_strict(&signed_bytes(context, message), &ed25519_dalek::Signature::from_bytes(&sig)).is_ok()
}

fn signing_key(secret_key: &str) -> Result<ed25519_dalek::SigningKey, Error> {
    let bytes = key_bytes(secret_key).ok_or_else(|| Error::Key("a secret key is base64 (or hex): 32 bytes, Ex. from Key.generate()".into()))?;
    let seed = <[u8; 32]>::try_from(bytes.as_slice()).map_err(|_| Error::Key(format!("a secret key is 32 bytes (this one is {})", bytes.len())))?;
    Ok(ed25519_dalek::SigningKey::from_bytes(&seed))
}

impl Graph {
    /// Seal an object (path or `<Type>`) only if its digest is signed by this public key: checks the signature
    /// over its current digest, then seals it. Returns the digest.
    pub fn seal_signed(&mut self, path: &str, fields: bool, public_key: &str, signature: &str) -> Result<String, Error> {
        self.seal_signed_with(path, &SealOptions { fields, ..Default::default() }, public_key, signature)
    }

    /// Seal an object (path or `<Type>`) with these options, only if its digest is signed by this public key.
    pub fn seal_signed_with(&mut self, path: &str, options: &SealOptions, public_key: &str, signature: &str) -> Result<String, Error> {
        let node = self.seal_target(path)?;
        let digest = self.digest(&node, options.fields)?;
        if !verify(public_key, SEAL_CONTEXT, digest.as_bytes(), signature) {
            return Err(Error::Sealed(format!("{path} isn't signed by this key: no signature over its digest {digest} under '{SEAL_CONTEXT}'")));
        }
        self.seal_with(&node, options)
    }

    /// The functions at these paths (host paths, Ex. "link.request"), for a seal's gates (`SealOptions::calls`).
    /// An error for a path with no function.
    pub fn seal_calls(&self, paths: &[String]) -> Result<Vec<DataRef>, Error> {
        let mut calls = Vec::new();
        for path in paths {
            let search = self.host_path(path, None);
            match crate::model::Func::func_from_path(self, &search, self.main_root()) {
                Some(func) => calls.push(func),
                None => return Err(Error::Sealed(format!("a seal's gate (calls) must be a function: there's none at '{path}'"))),
            }
        }
        Ok(calls)
    }

    /// A function's path (Ex. "root.link.request"), for showing a seal's gates.
    pub(crate) fn func_path(&self, func: &DataRef) -> String {
        let name = func.data_name(self).map(|name| name.to_string()).unwrap_or_default();
        let owner = func.data(self).and_then(|data| data.nodes.iter().next().cloned());
        match owner.and_then(|node| node.node_path(self, true)) {
            Some(path) => format!("{}.{}", path.join("."), name),
            None => name,
        }
    }
}


/*****************************************************************************
 * Confinement in the runtime.
 *****************************************************************************/

impl ProcEnv {
    /// Entering a function: it must be within reach of the code that's confined now (or a gate every confinement
    /// lists, which then runs unconfined until it returns), and code in a confined part runs confined from here
    /// (until the call returns). Called right after the function is on the call stack.
    pub(crate) fn enter_confined(&mut self, func: &DataRef, graph: &mut Graph) -> Result<(), Error> {
        let depth = self.call_stack.len();
        if !self.confines.is_empty() {
            // a gate: listed by every confinement it's called under (so a part can't make one of its own)
            if self.confines.iter().all(|(_, _, reach)| reach.gate(func)) {
                // out of time, a part calls nothing more (its gates run on the clock too)
                if self.confines.iter().any(|(_, _, reach)| reach.out_of_time()) {
                    return Err(Error::Sealed("this sealed part ran past its time limit".into()));
                }
                let confines = std::mem::take(&mut self.confines);
                self.suspended.push((depth, confines));
                graph.confined = Vec::new();
            } else if !graph.confined_reaches_data(func) {
                let name = func.data_name(graph).map(|name| name.to_string()).unwrap_or_default();
                return Err(Error::Sealed(format!("this sealed part can't call '{name}': it's outside what the part may reach")));
            }
        }
        let mut added = false;
        for confine in graph.func_confines(func) {
            // entering the part (not already running in it): its reads are looked up now
            if !self.confines.iter().any(|(_, existing, _)| Arc::ptr_eq(existing, &confine)) {
                let reach = Arc::new(confine.resolve(graph, self.pid.clone()));
                self.confines.push((depth, confine, reach));
                added = true;
            }
        }
        if added { graph.confined = self.confined(); }
        Ok(())
    }

    /// Leaving functions: confinements entered deeper than the call stack is now are done, and a gate that returned
    /// (or that an error unwound) gives its caller's confinements back.
    pub(crate) fn leave_confined(&mut self, graph: &mut Graph) {
        let before = self.confines.len();
        let mut changed = false;
        loop {
            while self.confines.last().is_some_and(|(depth, _, _)| *depth > self.call_stack.len()) {
                if let Some((_, _, reach)) = self.confines.pop() {
                    // left by the process that entered it: anything its code spawned that's still running ends too
                    if reach.owner == self.pid { reach.ended.store(true, std::sync::atomic::Ordering::Relaxed); }
                }
                changed = true;
            }
            if self.suspended.last().is_some_and(|(depth, _)| *depth > self.call_stack.len()) {
                // anything the gate entered is done (popped above); its caller's confinements come back
                let (_, confines) = self.suspended.pop().unwrap();
                self.confines = confines;
                changed = true;
                continue; // the restored ones may be done too (Ex. an error unwound past the caller as well)
            }
            break;
        }
        if changed || self.confines.len() != before { graph.confined = self.confined(); }
    }

    /// The confinements this process runs under now.
    pub(crate) fn confined(&self) -> Vec<Arc<Reach>> {
        self.confines.iter().map(|(_, _, reach)| reach.clone()).collect()
    }
}


/// Std functions confined code can never call: exit() (it would end the unconfined code that called the part, past
/// its try/catch), and the host's environment (env, env_vars, set_env, remove_env, with allow_system).
const CONFINED_STD_REFUSED: [&str; 5] = ["exit", "env", "env_vars", "set_env", "remove_env"];


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Checks for a library call made by confined code (added to the call only while confined; never stored).
pub enum ConfineIns {
    /// The library must be one every confined part allows, the call's arguments (the top `count` values) must be
    /// within reach, and `Http.fetch` must go to a host the confined parts allow.
    Args { count: usize, lib: ArcStr, func: ArcStr },
    /// What the call returned (above `base` on the stack) must be within reach.
    Result { base: usize, lib: ArcStr, func: ArcStr },
}
#[typetag::serde(name = "ConfineIns")]
impl Instruction for ConfineIns {
    fn exec(&self, env: &mut ProcEnv, graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        if graph.confined.is_empty() { return Ok(None); }
        match self {
            Self::Args { count, lib, func } => {
                if graph.confined.iter().any(|reach| !reach.lib_allowed(lib)) {
                    return Err(Error::Sealed(format!("this sealed part can't call {lib}.{func}: the {lib} library isn't one it may use")));
                }
                if graph.confined.iter().any(|reach| reach.out_of_time()) {
                    return Err(Error::Sealed("this sealed part ran past its time limit".into()));
                }
                if lib.as_str() == "Std" && CONFINED_STD_REFUSED.contains(&func.as_str()) {
                    // ending the process would end the code that called the part too (past its try/catch), and the
                    // environment is the host's (with allow_system), not something a part can be given
                    return Err(Error::Sealed(format!("this sealed part can't call {lib}.{func}")));
                }
                let start = env.stack.len().saturating_sub(*count);
                for var in &env.stack[start..] {
                    if !graph.confined_reaches_val(&var.val.read()) {
                        return Err(Error::Sealed(format!("this sealed part can't give {lib}.{func} an object outside what it may reach")));
                    }
                }
                if lib.as_str() == "Http" && func.as_str() == "fetch" {
                    if let Some(var) = env.stack.get(start) {
                        let url = var.val.read().to_string();
                        if !graph.confined_host(&url) {
                            return Err(Error::HttpSendError(format!("{url} is not a host this sealed part may reach")));
                        }
                    }
                }
            },
            Self::Result { base, lib, func } => {
                if env.stack.len() > *base {
                    if let Some(var) = env.stack.last() {
                        if !graph.confined_reaches_val(&var.val.read()) {
                            return Err(Error::Sealed(format!("{lib}.{func} gave this sealed part an object outside what it may reach")));
                        }
                    }
                }
            },
        }
        Ok(None)
    }
}
