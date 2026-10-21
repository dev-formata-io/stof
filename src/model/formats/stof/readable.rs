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

//! The readable Stof export (`stringify('stof', obj)`): a document as Stof a person or a model can read and edit.
//!
//! - Objects are objects (`name: { .. }`), with their type (`Plan pro: { .. }`), attributes, and doc comments.
//! - Values are Stof literals: units (`1GB`), versions, lists, tuples, and blobs, with `set(..)` and `map(..)` for
//!   sets and maps (a field whose value starts with `{` is an object).
//! - Functions are written as their source text when the parser kept it (the host's `Graph::keep_source`, off by
//!   default). A function without it (not kept, or loaded from BSTF) is written as a `data@v1` block instead (the
//!   exact export), so nothing is lost.
//! - Objects in values (Ex. a list of records) are written inline as `new Type { .. }` when they only hold values,
//!   else as an object of their own (`#[no-field]`) before the field, referenced by path.
//! - A reference to an object elsewhere is its path, so it resolves when the object is written before it.
//!
//! The exact export (IDs, references, compiled functions) is `stof:graph`.

use std::ops::Deref;
use rustc_hash::FxHashSet;
use crate::{model::{DataRef, Field, FieldDoc, Func, FuncDoc, Graph, NodeRef, Prototype, ARROW_FUNC_ATTR, ASYNC_FUNC_ATTR, NOEXPORT_FIELD_ATTR, NOFIELD_FIELD_ATTR, PROTOTYPE_EXTENDS_ATTR, PROTOTYPE_TYPE_ATTR}, parser::edit::{code_lf, dedent, reindent}, runtime::{Num, Type, Units, Val}};

const UNIT: &str = "    ";


/// A node (its contents), or the whole graph (each root).
pub fn readable_export(graph: &Graph, node: Option<NodeRef>) -> String {
    let mut export = Readable { graph, out: String::new(), seen: FxHashSet::default(), root: node.clone().or_else(|| graph.main_root()), current: Vec::new(), renamed: Default::default() };
    match node {
        Some(node) => {
            export.seen.insert(node.clone());
            export.body(&node, "");
        },
        None => {
            let main = graph.main_root();
            let mut roots: Vec<NodeRef> = graph.roots.iter().cloned().collect();
            roots.sort_by_key(|root| (Some(root) != main.as_ref(), root.node_name(graph).map(|n| n.as_ref().to_string()).unwrap_or_default()));
            for root in roots {
                export.seen.insert(root.clone());
                if Some(&root) == main.as_ref() {
                    export.body(&root, "");
                } else {
                    let name = root.node_name(graph).map(|n| n.as_ref().to_string()).unwrap_or_default();
                    export.line("", &format!("root {} {{", name_text(&name)));
                    export.body(&root, UNIT);
                    export.line("", "}");
                }
            }
        },
    }
    let mut out = export.out.trim_start_matches('\n').to_string();
    if !out.is_empty() { out.push('\n'); }
    out
}


struct Readable<'a> {
    graph: &'a Graph,
    out: String,
    seen: FxHashSet<NodeRef>,
    /// What's being exported (references inside it are relative: `self.a`, `super.b`).
    root: Option<NodeRef>,
    /// The objects being written, innermost last (`self` where a value is parsed).
    current: Vec<NodeRef>,
    /// Names written for objects whose own names can't be in a path (Ex. `new {}` objects' generated names).
    renamed: rustc_hash::FxHashMap<NodeRef, String>,
}

impl<'a> Readable<'a> {
    fn line(&mut self, indent: &str, text: &str) {
        self.out.push('\n');
        if !text.is_empty() { self.out.push_str(indent); }
        self.out.push_str(text);
    }

    /// The contents of an object.
    fn body(&mut self, node: &NodeRef, indent: &str) {
        self.current.push(node.clone());
        self.body_inner(node, indent);
        self.current.pop();
    }

    fn body_inner(&mut self, node: &NodeRef, indent: &str) {
        let graph = self.graph;
        let Some(n) = node.node(graph) else { return; };

        // doc comments, by what they document
        let mut docs: Vec<(DataRef, String)> = Vec::new();
        for dref in n.data.values() {
            if let Some(doc) = graph.get_stof_data::<FieldDoc>(dref) { docs.push((doc.field.clone(), doc.docs.clone())); }
            if let Some(doc) = graph.get_stof_data::<FuncDoc>(dref) { docs.push((doc.func.clone(), doc.docs.clone())); }
        }
        let docs_for = |dref: &DataRef| docs.iter().find(|(target, _)| target == dref).map(|(_, docs)| docs.clone());

        // objects that fields hold
        let mut owned: FxHashSet<NodeRef> = FxHashSet::default();
        for dref in n.data.values() {
            if let Some(field) = graph.get_stof_data::<Field>(dref) {
                if let Some(obj) = field.value.try_obj() {
                    if obj.node(graph).and_then(|o| o.parent.clone()).as_ref() == Some(node) { owned.insert(obj); }
                }
            }
        }

        // types are used by name when parsed, so objects with type definitions in them go first
        let mut entries: Vec<(&String, &DataRef)> = n.data.iter().collect();
        entries.sort_by_key(|(_, dref)| {
            let defines = graph.get_stof_data::<Field>(dref).and_then(|field| field.value.try_obj()).is_some_and(|obj| self.defines_types(&obj, node, 0));
            !defines
        });

        let mut first = true;
        // objects no field holds that have types in them, first too
        let typed_children: Vec<NodeRef> = n.children.iter().filter(|child| !owned.contains(*child) && self.defines_types(child, node, 0)).cloned().collect();
        for child in typed_children {
            if self.seen.contains(&child) { continue; }
            self.loose_object(&child, indent, &mut first);
        }
        for (name, dref) in entries {
            if let Some(field) = graph.get_stof_data::<Field>(dref) {
                if self.hidden(field, node) { continue; }
                // objects in this value that can't be written inline go first, as objects of their own
                let mut standalone = Vec::new();
                self.standalone_in(&field.value.val.read(), node, &owned, &mut standalone);
                for obj in standalone { self.loose_object(&obj, indent, &mut first); }

                if !first && self.is_block(field, node) { self.out.push('\n'); }
                first = false;
                if let Some(docs) = docs_for(dref) { self.docs(indent, &docs); }
                self.field(name, field, node, indent);
            } else if let Some(func) = graph.get_stof_data::<Func>(dref) {
                if func.attributes.contains_key(NOEXPORT_FIELD_ATTR.as_str()) { continue; }
                if func.attributes.contains_key(ARROW_FUNC_ATTR.as_str()) { continue; } // written where it's used
                if !first { self.out.push('\n'); }
                first = false;
                if let Some(docs) = docs_for(dref) { self.docs(indent, &docs); }
                self.func(dref, func, indent);
            } else if graph.get_stof_data::<Prototype>(dref).is_some()
                || graph.get_stof_data::<FieldDoc>(dref).is_some()
                || graph.get_stof_data::<FuncDoc>(dref).is_some() {
                continue; // types are written with their objects, docs with what they document
            } else {
                self.data_block(dref, indent);
            }
        }

        // objects that aren't any field's (Ex. #[no-field] objects)
        let children: Vec<NodeRef> = n.children.iter().cloned().collect();
        for child in children {
            if owned.contains(&child) || self.seen.contains(&child) { continue; }
            self.loose_object(&child, indent, &mut first);
        }
    }

    /// Is this field left out? `#[no-export]` fields are, but imports (Ex. JSON) mark a field that holds an
    /// object `#[no-export]` without marking the object: that object is still written, as the field.
    fn hidden(&self, field: &Field, node: &NodeRef) -> bool {
        if !field.attributes.contains_key(NOEXPORT_FIELD_ATTR.as_str()) { return false; }
        match field.value.try_obj() {
            Some(obj) => match obj.node(self.graph) {
                Some(o) => o.parent.as_ref() != Some(node) || o.attributes.contains_key(NOEXPORT_FIELD_ATTR.as_str()),
                None => true,
            },
            None => true,
        }
    }

    /// Is this object (held by `parent`) a type definition, or does it have one in it?
    fn defines_types(&self, obj: &NodeRef, parent: &NodeRef, depth: usize) -> bool {
        if depth > 64 { return false; }
        let Some(o) = obj.node(self.graph) else { return false; };
        if o.parent.as_ref() != Some(parent) { return false; }
        if o.attributes.contains_key(PROTOTYPE_TYPE_ATTR.as_str()) { return true; }
        o.children.iter().any(|child| self.defines_types(child, obj, depth + 1))
    }

    /// Does this field write a block (an object)? Blocks get a blank line before them.
    fn is_block(&self, field: &Field, node: &NodeRef) -> bool {
        field.value.try_obj().map(|obj| obj.node(self.graph).and_then(|o| o.parent.clone()).as_ref() == Some(node)).unwrap_or(false)
    }

    fn docs(&mut self, indent: &str, docs: &str) {
        for line in docs.trim().lines() {
            let line = line.trim();
            self.line(indent, &if line.is_empty() { "///".to_string() } else { format!("/// {line}") });
        }
    }

    fn attributes(&mut self, indent: &str, attributes: &rustc_hash::FxHashMap<String, Val>, skip: &[&str]) {
        let mut names: Vec<&String> = attributes.keys().collect();
        names.sort();
        for name in names {
            if skip.contains(&name.as_str()) { continue; }
            let value = &attributes[name];
            if name == PROTOTYPE_EXTENDS_ATTR.as_str() {
                if let Val::Obj(proto) = value {
                    let path = self.reference(proto); // a type by its path (Ex. `#[extends(self.Base)]`)
                    self.line(indent, &format!("#[{name}({path})]"));
                    continue;
                }
            }
            if matches!(value, Val::Null | Val::Void) {
                self.line(indent, &format!("#[{name}]"));
            } else {
                let text = self.value(value, indent);
                self.line(indent, &format!("#[{name}({text})]"));
            }
        }
    }

    fn field(&mut self, name: &str, field: &Field, node: &NodeRef, indent: &str) {
        let graph = self.graph;
        self.attributes(indent, &field.attributes, &[NOEXPORT_FIELD_ATTR.as_str()]); // not hidden: see hidden()
        let constant = if field.value.mutable { "" } else { "const " };
        let val = field.value.val.read().deref().clone();
        if let Val::Obj(obj) = &val {
            if obj.node(graph).and_then(|o| o.parent.clone()).as_ref() == Some(node) && !self.seen.contains(obj) {
                self.seen.insert(obj.clone());
                let declared = field.value.vtype.as_ref().filter(|t| !matches!(t, Type::Void)).map(|t| type_text(t, graph));
                let is_type = field.attributes.contains_key(PROTOTYPE_TYPE_ATTR.as_str()) || field.attributes.contains_key(PROTOTYPE_EXTENDS_ATTR.as_str());
                let typename = declared.or_else(|| if is_type { None } else { self.prototype_name(obj) });
                let prefix = typename.map(|t| format!("{t} ")).unwrap_or_default();
                let empty = obj.node(graph).map(|o| o.children.is_empty() && o.data.values().all(|d| graph.get_stof_data::<Prototype>(d).is_some())).unwrap_or(true);
                if empty {
                    self.line(indent, &format!("{constant}{prefix}{}: {{}}", name_text(name)));
                } else {
                    self.line(indent, &format!("{constant}{prefix}{}: {{", name_text(name)));
                    self.body(obj, &format!("{indent}{UNIT}"));
                    self.line(indent, "}");
                }
                return;
            }
        }
        let declared = field.value.vtype.as_ref().filter(|t| !matches!(t, Type::Void)).map(|t| format!("{} ", type_text(t, graph))).unwrap_or_default();
        let value = self.value(&val, indent);
        self.line(indent, &format!("{constant}{declared}{}: {value}", name_text(name)));
    }

    fn func(&mut self, dref: &DataRef, func: &Func, indent: &str) {
        let Some(source) = &func.source else {
            // loaded from BSTF: no text, so the exact form (it has its attributes)
            self.data_block(dref, indent);
            return;
        };
        let async_attr = ASYNC_FUNC_ATTR;
        let skip = [async_attr.as_str()];
        let skip: &[&str] = if source.trim_start().starts_with("async") { &skip } else { &[] };
        self.attributes(indent, &func.attributes, skip);
        let text = function_text(source, indent);
        self.line(indent, &text);
    }

    /// An object no field holds, written as `#[no-field] name: { .. }`.
    fn loose_object(&mut self, obj: &NodeRef, indent: &str, first: &mut bool) {
        let graph = self.graph;
        if self.seen.contains(obj) { return; }
        self.seen.insert(obj.clone());
        let Some(o) = obj.node(graph) else { return; };
        if !*first { self.out.push('\n'); }
        *first = false;
        // a field node (Ex. `import .. as self.x`) is a field; anything else is an object only
        let mut attributes = o.attributes.clone();
        let is_field = attributes.remove("field").is_some();
        if !is_field { attributes.insert(NOFIELD_FIELD_ATTR.to_string(), Val::Null); }
        self.attributes(indent, &attributes, &[]);
        let is_type = o.attributes.contains_key(PROTOTYPE_TYPE_ATTR.as_str()) || o.attributes.contains_key(PROTOTYPE_EXTENDS_ATTR.as_str());
        let prefix = if is_type { String::new() } else { self.prototype_name(obj).map(|t| format!("{t} ")).unwrap_or_default() };
        let mut name = o.name.as_ref().to_string();
        if !is_ident(&name) {
            // a generated name (Ex. from `new {}`) can't be in a path: one that can
            name = format!("_{}", name.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect::<String>());
            self.renamed.insert(obj.clone(), name.clone());
        }
        self.line(indent, &format!("{prefix}{name}: {{"));
        self.body(obj, &format!("{indent}{UNIT}"));
        self.line(indent, "}");
    }

    /// The type name of an object's prototype, when it's a declared type (`#[type]`). A prototype that's an
    /// object (Ex. a type's object field, copied into each instance) can't be named in Stof, so it isn't written.
    fn prototype_name(&self, obj: &NodeRef) -> Option<String> {
        let proto = Prototype::prototype_nodes(self.graph, obj, false).into_iter().next()?;
        if !proto.node(self.graph).is_some_and(|p| p.attributes.contains_key(PROTOTYPE_TYPE_ATTR.as_str())) { return None; }
        Some(Type::Obj(proto.as_ref().into()).rt_type_of(self.graph).to_string())
    }

    fn data_block(&mut self, dref: &DataRef, indent: &str) {
        if let Some(data) = dref.data(self.graph) {
            if let Ok(bytes) = bincode::serialize(data) {
                let text = format!("{bytes:?}");
                self.line(indent, &format!("data@v1 |{}|;", text.trim_start_matches('[').trim_end_matches(']')));
            }
        }
    }

    /// Objects in a value that can't be written inline (they hold more than values) and that no field holds.
    fn standalone_in(&self, val: &Val, node: &NodeRef, owned: &FxHashSet<NodeRef>, out: &mut Vec<NodeRef>) {
        match val {
            Val::Obj(obj) => {
                let child = obj.node(self.graph).and_then(|o| o.parent.clone()).as_ref() == Some(node);
                if child && !owned.contains(obj) && !self.seen.contains(obj) && !self.plain(obj, 0) && !out.contains(obj) {
                    out.push(obj.clone());
                }
            },
            Val::List(items) | Val::Tup(items) => for item in items { self.standalone_in(&item.read(), node, owned, out); },
            Val::Set(items) => for item in items { self.standalone_in(&item.read(), node, owned, out); },
            Val::Map(map) => for (k, v) in map { self.standalone_in(&k.read(), node, owned, out); self.standalone_in(&v.read(), node, owned, out); },
            _ => {},
        }
    }

    /// Does an object only hold values (so it can be written inline as `new Type { .. }`)?
    fn plain(&self, obj: &NodeRef, depth: usize) -> bool {
        if depth > 32 { return false; }
        let graph = self.graph;
        let Some(o) = obj.node(graph) else { return false; };
        if !o.attributes.is_empty() { return false; }
        for dref in o.data.values() {
            if graph.get_stof_data::<Prototype>(dref).is_some() { continue; }
            let Some(field) = graph.get_stof_data::<Field>(dref) else { return false; };
            if !field.attributes.is_empty() || !field.value.mutable || field.value.vtype.as_ref().is_some_and(|t| !matches!(t, Type::Void)) { return false; }
            if !self.plain_value(&field.value.val.read(), obj, depth) { return false; }
        }
        // nested objects must be the fields'
        o.children.iter().all(|child| o.data.values().any(|dref| graph.get_stof_data::<Field>(dref).and_then(|f| f.value.try_obj()).as_ref() == Some(child)))
    }

    fn plain_value(&self, val: &Val, owner: &NodeRef, depth: usize) -> bool {
        match val {
            Val::Obj(obj) => obj.node(self.graph).and_then(|o| o.parent.clone()).as_ref() == Some(owner) && self.plain(obj, depth + 1),
            Val::Fn(_) | Val::Data(_) => false,
            Val::List(items) | Val::Tup(items) => items.iter().all(|item| self.plain_value(&item.read(), owner, depth)),
            Val::Set(items) => items.iter().all(|item| self.plain_value(&item.read(), owner, depth)),
            Val::Map(map) => map.iter().all(|(k, v)| self.plain_value(&k.read(), owner, depth) && self.plain_value(&v.read(), owner, depth)),
            _ => true,
        }
    }

    /// A value as Stof source.
    fn value(&mut self, val: &Val, indent: &str) -> String {
        let graph = self.graph;
        match val {
            Val::Void | Val::Null | Val::Promise(..) => "null".into(),
            Val::Bool(b) => b.to_string(),
            Val::Num(num) => num_text(num),
            Val::Str(s) => string_text(s.as_str()),
            Val::Prompt(p) => string_text(&p.to_string()),
            Val::Ver(..) => val.to_string(),
            Val::Blob(blob) => format!("|{}|", blob.iter().map(|b| b.to_string()).collect::<Vec<_>>().join(", ")),
            Val::Fn(dref) => {
                let Some(func) = graph.get_stof_data::<Func>(dref) else { return "null".into(); };
                if func.attributes.contains_key(ARROW_FUNC_ATTR.as_str()) {
                    if let Some(source) = &func.source { return function_text(source, indent); }
                } else if let Some(data) = dref.data(graph) {
                    // a named function: its path (Ex. `self.check`)
                    let mut owners: Vec<&NodeRef> = data.nodes.iter().collect();
                    owners.sort();
                    if let Some(owner) = owners.first() {
                        return format!("{}.{}", self.reference(owner), data.name.as_ref());
                    }
                }
                format!("Data.from_id('{}') as fn", dref.as_ref())
            },
            Val::Data(dref) => format!("Data.from_id('{}')", dref.as_ref()),
            Val::Obj(obj) => {
                let here = self.current.last().cloned();
                let child = here.is_some() && obj.node(graph).and_then(|o| o.parent.clone()) == here;
                if child && !self.seen.contains(obj) && self.plain(obj, 0) {
                    self.seen.insert(obj.clone());
                    self.current.push(obj.clone());
                    let text = self.inline_object(obj, indent);
                    self.current.pop();
                    return text;
                }
                self.reference(obj)
            },
            Val::List(items) => {
                let items: Vec<String> = items.iter().map(|item| self.value(&item.read(), &format!("{indent}{UNIT}"))).collect();
                list_text("[", "]", items, indent)
            },
            Val::Tup(items) => {
                let items: Vec<String> = items.iter().map(|item| self.value(&item.read(), indent)).collect();
                format!("({})", items.join(", "))
            },
            // constructors, since `{ .. }` is an object where a field's value starts
            Val::Set(items) => {
                let items: Vec<String> = items.iter().map(|item| self.value(&item.read(), indent)).collect();
                format!("set({})", items.join(", "))
            },
            Val::Map(map) => {
                let items: Vec<String> = map.iter().map(|(k, v)| format!("({}, {})", self.value(&k.read(), indent), self.value(&v.read(), indent))).collect();
                format!("map({})", items.join(", "))
            },
        }
    }

    /// A path to an object: relative (`self.a.b`, `super.c`) inside what's exported, else from its root.
    fn reference(&self, obj: &NodeRef) -> String {
        let graph = self.graph;
        let chain = |node: &NodeRef| -> Vec<NodeRef> {
            let mut out = vec![node.clone()];
            let mut cur = node.clone();
            while let Some(parent) = cur.node(graph).and_then(|n| n.parent.clone()) {
                out.push(parent.clone());
                cur = parent;
                if out.len() > 512 { break; }
            }
            out.reverse();
            out
        };
        let target = chain(obj);
        let inside = self.root.as_ref().is_some_and(|root| target.contains(root));
        if let (true, Some(here)) = (inside, self.current.last()) {
            let from = chain(here);
            let common = from.iter().zip(target.iter()).take_while(|(a, b)| a == b).count();
            if common > 0 {
                let ups = from.len() - common;
                let parts: Vec<String> = if ups == 0 { vec!["self".into()] } else { vec!["super".into(); ups] };
                let names: Vec<String> = target[common..].iter().map(|node| self.renamed.get(node).cloned().or_else(|| node.node_name(graph).map(|n| n.as_ref().to_string())).unwrap_or_default()).collect();
                return path_text(parts, names);
            }
        }
        match obj.node_path(graph, true) {
            Some(path) => {
                let names: Vec<String> = path.path.iter().map(|name| name.as_ref().to_string()).collect();
                let Some((first, rest)) = names.split_first() else { return "null".into(); };
                path_text(vec![first.clone()], rest.to_vec())
            },
            None => "null".into(),
        }
    }

    /// `new Type { a: 1, b: 2 }`, one field per line when it doesn't fit on one.
    fn inline_object(&mut self, obj: &NodeRef, indent: &str) -> String {
        let graph = self.graph;
        let typename = self.prototype_name(obj).map(|t| format!("{t} ")).unwrap_or_default();
        let Some(o) = obj.node(graph) else { return "null".into(); };
        let inner = format!("{indent}{UNIT}");
        let mut fields = Vec::new();
        let data: Vec<(String, DataRef)> = o.data.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        for (name, dref) in data {
            let Some(field) = graph.get_stof_data::<Field>(&dref) else { continue; };
            let value = field.value.val.read().deref().clone();
            let text = self.value(&value, &inner);
            fields.push(format!("{}: {text}", name_text(&name)));
        }
        if fields.is_empty() { return format!("new {typename}{{}}"); }
        let one_line = format!("new {typename}{{ {} }}", fields.join(", "));
        if one_line.len() <= 80 && !one_line.contains('\n') { return one_line; }
        format!("new {typename}{{\n{}\n{indent}}}", fields.iter().map(|f| format!("{inner}{f},")).collect::<Vec<_>>().join("\n"))
    }
}


/// A function's source at an indent: its own indent (that of its last line, the closing brace) is replaced.
fn function_text(source: &str, indent: &str) -> String {
    // the export's lines end with \n, whatever the parsed text used (Ex. CRLF files on Windows); line breaks
    // inside strings are part of their values, so they stay
    let source = code_lf(source);
    let source = source.trim();
    let own = source.rsplit('\n').next().filter(|_| source.contains('\n')).map(|last| last.chars().take_while(|c| *c == ' ' || *c == '\t').collect::<String>()).unwrap_or_default();
    reindent(&dedent(source, &own), indent)
}

/// A path from its start: names that can't be in a path (Ex. generated IDs) are looked up with `get`.
fn path_text(start: Vec<String>, names: Vec<String>) -> String {
    let mut out = start.join(".");
    for name in names {
        if is_ident(&name) { out.push('.'); out.push_str(&name); }
        else { out.push_str(&format!(".get({})", string_text(&name))); }
    }
    out
}

fn list_text(open: &str, close: &str, items: Vec<String>, indent: &str) -> String {
    if items.is_empty() { return format!("{open}{close}"); }
    let one_line = format!("{open}{}{close}", items.join(", "));
    if one_line.len() <= 100 && !one_line.contains('\n') { return one_line; }
    format!("{open}\n{}\n{indent}{close}", items.iter().map(|item| format!("{indent}{UNIT}{item},")).collect::<Vec<_>>().join("\n"))
}

fn type_text(t: &Type, graph: &Graph) -> String {
    match t {
        Type::NotNull(inner) => format!("{}!", type_text(inner, graph)),
        Type::Union(types) => types.iter().map(|t| type_text(t, graph)).collect::<Vec<_>>().join(" | "),
        _ => t.rt_type_of(graph).to_string(),
    }
}

fn num_text(num: &Num) -> String {
    match num {
        Num::Int(v) => v.to_string(),
        Num::Float(v) => float_text(*v),
        Num::Units(v, units) => {
            if matches!(units, Units::None | Units::Undefined) || !v.is_finite() { return float_text(*v); }
            format!("{v}{}", units.to_string())
        },
    }
}

fn float_text(v: f64) -> String {
    if v.is_nan() { return "nan()".into(); }
    if v.is_infinite() { return if v > 0. { "inf()".into() } else { "-inf()".into() }; }
    format!("{v:?}") // keeps a decimal point (2.0), so it reads back as a float
}

fn string_text(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('\'');
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\'' => out.push_str("\\'"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch => out.push(ch),
        }
    }
    out.push('\'');
    out
}

fn is_ident(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_') && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn name_text(name: &str) -> String {
    if is_ident(name) { name.to_string() } else { string_text(name) }
}


#[cfg(test)]
mod tests {
    use crate::{model::{Graph, Profile}, runtime::{Runtime, Val}};

    const DOC: &str = r#"
/// A plan.
#[type]
Plan: {
    str label: ''
    /// Its price.
    fn price() -> float { 10 }
}

policy: {
    Plan pro: {
        label: 'Pro'
        #[custom]
        fn price() -> float {
            20
        }
    }
    limits: [new { value: 1GB }]
}

root Other {
    x: 1
}
"#;

    #[test]
    fn whole_graph_with_roots_types_and_docs() {
        let mut graph = Graph::default();
        graph.keep_source(true);
        graph.parse_stof_src(DOC, None, Profile::docs(false)).unwrap();
        let text = graph.string_export("stof", None).unwrap();
        assert!(text.starts_with("/// A plan.\n#[type]\nPlan: {\n    str label: ''\n\n    /// Its price.\n    fn price() -> float { 10 }\n}\n"), "{text}");
        assert!(text.contains("\npolicy: {\n    Plan pro: {\n        label: 'Pro'\n\n        #[custom]\n        fn price() -> float {\n            20\n        }\n    }\n    limits: [new { value: 1GB }]\n}\n"), "{text}");
        assert!(text.ends_with("\nroot Other {\n    x: 1\n}\n"), "{text}");

        let mut back = Graph::default();
        back.keep_source(true);
        back.parse_stof_src(&text, None, Profile::docs(false)).unwrap();
        assert_eq!(back.string_export("stof", None).unwrap(), text);
        let price = Runtime::call(&mut back, "policy.pro.price", vec![]).unwrap();
        assert_eq!(price, Val::from(20.0));
    }

    #[test]
    fn source_is_kept_only_when_asked() {
        // by default, functions keep no text: written as data blocks (exact), and they still work
        let mut graph = Graph::default();
        graph.parse_stof_src("obj: { n: 2\nfn double() -> int { self.n * 2 } }", None, Profile::default()).unwrap();
        let text = graph.string_export("stof", None).unwrap();
        assert!(text.contains("n: 2") && text.contains("data@v1 |") && !text.contains("fn double"), "{text}");
        let mut back = Graph::default();
        back.parse_stof_src(&text, None, Profile::default()).unwrap();
        assert_eq!(Runtime::call(&mut back, "obj.double", vec![]).unwrap(), Val::from(4));

        let mut kept = Graph::default();
        kept.keep_source(true);
        kept.parse_stof_src("obj: { n: 2\nfn double() -> int { self.n * 2 } }", None, Profile::default()).unwrap();
        assert!(kept.string_export("stof", None).unwrap().contains("fn double() -> int { self.n * 2 }"));
    }

    #[test]
    fn functions_from_bstf_are_exact() {
        // functions loaded from BSTF have no source text: written as data blocks, so nothing is lost
        let mut graph = Graph::default();
        graph.parse_stof_src("obj: { n: 2\nfn double() -> int { self.n * 2 } }", None, Profile::default()).unwrap();
        let bytes = graph.binary_export("bstf", None).unwrap();
        let mut loaded = Graph::default();
        loaded.binary_import("bstf", bytes, None, &Profile::default()).unwrap();
        let text = loaded.string_export("stof", None).unwrap();
        assert!(text.contains("obj: {\n    n: 2\n\n    data@v1 |"), "{text}");

        let mut back = Graph::default();
        back.parse_stof_src(&text, None, Profile::default()).unwrap();
        assert_eq!(Runtime::call(&mut back, "obj.double", vec![]).unwrap(), Val::from(4));
    }

    #[test]
    fn the_text_can_be_outlined_and_edited() {
        let mut graph = Graph::default();
        graph.keep_source(true);
        graph.parse_stof_src(DOC, None, Profile::default()).unwrap();
        let text = graph.string_export("stof", None).unwrap();
        let out = crate::parser::edit::edit(&text, &[crate::parser::edit::Edit::Set { path: vec!["policy".into(), "pro".into(), "label".into()], value: "'Pro plus'".into() }]).unwrap();
        assert!(out.contains("        label: 'Pro plus'\n"), "{out}");
    }
}

#[cfg(test)]
mod corpus {
    use crate::model::{Graph, Profile};

    #[test]
    /// Every .stof file in this repo that parses on its own: its readable export parses, and exporting that gives
    /// the same text.
    fn corpus_reads_back() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut files = Vec::new();
        let mut stack = vec![root.join("src"), root.join("web")];
        while let Some(dir) = stack.pop() {
            let Ok(entries) = std::fs::read_dir(&dir) else { continue; };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() { if !path.ends_with("node_modules") { stack.push(path); } }
                else if path.extension().map(|ext| ext == "stof").unwrap_or(false) { files.push(path); }
            }
        }
        files.sort();
        let mut checked = 0;
        let mut failures = Vec::new();
        for file in files {
            let text = std::fs::read_to_string(&file).unwrap();
            let mut graph = Graph::default();
            graph.keep_source(true);
            if graph.parse_stof_src(&text, None, Profile::default()).is_err() { continue; }
            checked += 1;
            let exported = graph.string_export("stof", None).unwrap();
            let mut back = Graph::default();
            back.keep_source(true);
            if let Err(error) = back.parse_stof_src(&exported, None, Profile::default()) {
                failures.push(format!("{}: doesn't parse: {:?}", file.display(), error));
                continue;
            }
            let again = back.string_export("stof", None).unwrap();
            // data blocks carry IDs, which a parse gives new ones
            let strip = |text: &str| text.lines().filter(|line| !line.trim_start().starts_with("data@v1")).collect::<Vec<_>>().join("\n");
            let (again, exported) = (strip(&again), strip(&exported));
            if again != exported {
                let line = again.lines().zip(exported.lines()).position(|(a, b)| a != b).unwrap_or(0);
                failures.push(format!("{}: changed at line {}: {:?} vs {:?}", file.display(), line + 1, exported.lines().nth(line), again.lines().nth(line)));
            }
        }
        assert!(checked >= 20, "only {checked} files");
        assert!(failures.is_empty(), "{} of {checked} files:\n{}", failures.len(), failures.join("\n"));
    }
}

#[cfg(test)]
mod suite {
    use crate::model::{Graph, Profile};

    #[test]
    /// The whole Stof suite, written with the readable export and parsed from that, still passes. Only the tests of
    /// declared object IDs fail: the readable form doesn't keep IDs (`stof:graph` does).
    fn suite_from_export() {
        let mut graph = Graph::default();
        graph.allow_system();
        graph.keep_source(true);
        graph.parse_stof_file("stof", "src/model/formats/stof/tests/tests.stof", None, Profile::test()).unwrap();
        let text = graph.string_export("stof", None).unwrap();
        let mut back = Graph::default();
        back.allow_system();
        back.keep_source(true);
        back.parse_stof_src(&text, None, Profile::test()).unwrap();
        if let Err(err) = back.test(None, true) {
            // the report can be colored (terminal escape codes), so they're taken out first
            let mut plain = String::with_capacity(err.len());
            let mut chars = err.chars().peekable();
            while let Some(ch) = chars.next() {
                if ch == '\u{1b}' && chars.peek() == Some(&'[') {
                    while let Some(c) = chars.next() { if c.is_ascii_alphabetic() { break; } }
                } else {
                    plain.push(ch);
                }
            }
            let failed: Vec<&str> = plain.lines().filter(|line| line.starts_with("failed:")).collect();
            assert!(!failed.is_empty() && failed.iter().all(|line| line.contains("has_correct_custom_id")), "{err}");
        }
    }
}
