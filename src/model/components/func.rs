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

use std::sync::Arc;
use arcstr::{literal, ArcStr};
use imbl::Vector;
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use crate::{parser::source::SrcLoc, model::{DataRef, Graph, NodeRef, SId, SPath, StofData}, runtime::{instruction::{Instruction, Instructions}, Type, Val, Variable}};


/// Attribute used to denote a main function.
pub const MAIN_FUNC_ATTR: ArcStr = literal!("main");

/// Attribute used to denote a test function.
pub const TEST_FUNC_ATTR: ArcStr = literal!("test");

/// Attribute used to denote an async function.
pub const ASYNC_FUNC_ATTR: ArcStr = literal!("async");

/// If present, the function will not add its location to the self stack when called.
pub const UNSELF_FUNC_ATTR: ArcStr = literal!("unself");

/// Arrow function attribute (set by the parser on arrow functions).
/// Arrow functions see the variables of the function they're called from (normally the one that defines
/// them, Ex. `(x) => x + offset`), and after those, the variables that were visible where they were last
/// evaluated (so they work when passed to or stored for another function); named functions only see their
/// own (lexical boundary).
pub const ARROW_FUNC_ATTR: ArcStr = literal!("arrow");


/// A function's parameters, in order. Shared (cloning is a reference count), and sized to the parameters: an
/// `imbl::Vector` allocates a 64-slot chunk even for one parameter, a large share of a document's memory.
/// Serialized as a sequence, the same as before, so BSTF is unchanged.
pub type Params = Arc<[Param]>;

#[macro_export]
/// Parameters for a function (Ex. `params![Param { .. }, Param { .. }]`), like `vec!`.
macro_rules! params {
    ($($param:expr),* $(,)?) => { $crate::model::Params::from([$($param),*]) };
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// Function.
pub struct Func {
    pub params: Params,
    pub return_type: Type,
    pub attributes: FxHashMap<String, Val>,
    pub instructions: Vector<Arc<dyn Instruction>>,

    /// Where this function is defined, for error messages.
    /// Set by the parser and never stored in documents (BSTF stays the same).
    #[serde(skip)]
    pub src: Option<SrcLoc>,

    /// The function's source text as written (from `fn`, `async`, or an arrow's parameters to the end of its
    /// body), for the readable Stof export. Set by the parser only when the host asks (`Graph::keep_source`), and
    /// never stored in documents (a function loaded from BSTF has none).
    #[serde(skip)]
    pub source: Option<ArcStr>,

    /// Arrow functions: the variables visible where the arrow was last evaluated (shared with them, so an
    /// assignment in the arrow is seen by the variable it captured). Runtime only, never stored in documents.
    #[serde(skip)]
    pub captured: Option<Arc<FxHashMap<String, Variable>>>,

    /// Arrow functions: the outer variable names the body uses, so only those are captured. Set by the parser and
    /// never stored in documents (None captures every visible variable).
    #[serde(skip)]
    pub capture_names: Option<Arc<Vec<String>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuncDoc {
    pub func: DataRef,
    pub docs: String,
}
impl FuncDoc {
    /// Func docs on a node.
    pub fn docs(graph: &Graph, node: &NodeRef) -> FxHashMap<DataRef, String> {
        let mut docs = FxHashMap::default();
        if let Some(node) = node.node(graph) {
            for (_, dref) in &node.data {
                if let Some(doc) = graph.get_stof_data::<Self>(dref) {
                    docs.insert(doc.func.clone(), doc.docs.clone());
                }
            }
        }
        docs
    }
}

#[typetag::serde(name = "Fn")]
impl StofData for Func {
    fn core_data(&self) -> bool {
        return true;
    }
}
#[typetag::serde(name = "FnDoc")]
impl StofData for FuncDoc {
    fn core_data(&self) -> bool {
        return true;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Parameter.
pub struct Param {
    pub name: SId,
    pub param_type: Type,
    pub default: Option<Arc<dyn Instruction>>,
}

impl Func {
    /// Create a new function.
    pub fn new(params: Params, return_type: Type, instructions: Instructions, attrs: Option<FxHashMap<String, Val>>) -> Self {
        let mut attributes = FxHashMap::default();
        if let Some(attr) = attrs {
            attributes = attr;
        }
        Self {
            params,
            return_type,
            instructions: instructions.instructions.into_iter().collect(),
            attributes,
            src: None,
            source: None,
            captured: None,
            capture_names: None,
        }
    }

    /// Get a func from a dot separated name path string.
    /// Ex. "root.hello" -> root object with a func named "hello".
    pub fn func_from_path(graph: &Graph, path: &str, start: Option<NodeRef>) -> Option<DataRef> {
        let mut spath = SPath::from(path);
        if spath.path.is_empty() { return None; }
        
        let func_name = spath.path.pop().unwrap();
        if let Some(node) = SPath::node(&graph, spath, start) {
            return Self::func(graph, &node, &func_name);
        }
        None
    }
    
    #[inline]
    /// Func lookup.
    pub fn func(graph: &Graph, node: &NodeRef, func_name: &SId) -> Option<DataRef> {
        if let Some(node) = node.node(graph) {
            if let Some(dref) = node.data.get(func_name.as_ref()) {
                if dref.type_of::<Self>(&graph) {
                    return Some(dref.clone());
                }
            }
        }
        None
    }

    /// Get all functions on a node, optionally filtered by attributes and optionally recursively, in definition order
    /// (the object's own, then its children's).
    pub fn functions(graph: &Graph, node: &NodeRef, attrs: &Option<FxHashSet<String>>, recursive: bool) -> Vec<DataRef> {
        let mut funcs = Vec::new();
        let mut seen = FxHashSet::default();
        Self::collect_functions(graph, node, attrs, recursive, &mut funcs, &mut seen);
        funcs
    }

    fn collect_functions(graph: &Graph, node: &NodeRef, attrs: &Option<FxHashSet<String>>, recursive: bool, funcs: &mut Vec<DataRef>, seen: &mut FxHashSet<DataRef>) {
        if let Some(node) = node.node(&graph) {
            for (_name, dref) in &node.data {
                if let Some(func) = graph.get_stof_data::<Self>(dref) {
                    if func.has_any_attr(attrs) && seen.insert(dref.clone()) {
                        funcs.push(dref.clone());
                    }
                }
            }
            if recursive {
                for child in &node.children {
                    Self::collect_functions(graph, child, attrs, recursive, funcs, seen);
                }
            }
        }
    }

    /// Get all functions in a graph, optionally filtered by attribute, in definition order.
    /// One pass over the graph's data: no walk of every node (documents can hold many objects, Ex. customers). Only
    /// the functions found are then ordered, by where they are in the document.
    pub fn all_functions(graph: &Graph, attrs: &Option<FxHashSet<String>>) -> Vec<DataRef> {
        let mut keyed = Vec::new();
        for (dref, data) in &graph.data {
            if let Some(func) = data.data.as_dyn_any().downcast_ref::<Self>() {
                if func.has_any_attr(attrs) {
                    keyed.push((Self::definition_key(graph, dref), dref.clone()));
                }
            }
        }
        keyed.sort();
        keyed.into_iter().map(|(_, dref)| dref).collect()
    }

    /// Where a function is in the document, for ordering: its root (the main root first, then by name), the position
    /// of each object down to it (as a field of its parent), and its position in its object.
    fn definition_key(graph: &Graph, dref: &DataRef) -> Vec<usize> {
        let mut best: Option<Vec<usize>> = None;
        let Some(data) = graph.data.get(dref) else { return vec![usize::MAX]; };
        for node in &data.nodes {
            let mut key = Vec::new();
            if let Some(obj) = node.node(graph) {
                key.push(obj.data.get_index_of(data.name.as_ref()).unwrap_or(usize::MAX));
            }
            let mut current = node.clone();
            let mut depth = 0;
            loop {
                depth += 1;
                if depth > 512 { break; } // a parent cycle shouldn't exist; never loop on one
                let Some(obj) = current.node(graph) else { break; };
                match &obj.parent {
                    Some(parent) => {
                        let index = parent.node(graph).map(|p| {
                            p.data.get_index_of(obj.name.as_ref())
                                .unwrap_or_else(|| p.data.len() + p.children.get_index_of(&current).unwrap_or(usize::MAX / 2))
                        }).unwrap_or(usize::MAX);
                        key.push(index);
                        current = parent.clone();
                    },
                    None => {
                        let rank = if graph.main_root().as_ref() == Some(&current) { 0 } else {
                            let mut names: Vec<&str> = graph.roots.iter().filter_map(|r| r.node(graph).map(|n| n.name.as_ref())).collect();
                            names.sort();
                            1 + names.iter().position(|n| *n == obj.name.as_ref()).unwrap_or(names.len())
                        };
                        key.push(rank);
                        break;
                    },
                }
            }
            key.reverse();
            if best.as_ref().is_none_or(|b| key < *b) { best = Some(key); }
        }
        best.unwrap_or_else(|| vec![usize::MAX])
    }

    /// Does this function have any of these attributes (or are there no attributes to filter by)?
    fn has_any_attr(&self, attrs: &Option<FxHashSet<String>>) -> bool {
        match attrs {
            Some(attrs) => attrs.iter().any(|attr| self.attributes.contains_key(attr)),
            None => true,
        }
    }

    /// Get all main functions in a graph.
    pub fn main_functions(graph: &Graph) -> Vec<DataRef> {
        let mut set = FxHashSet::default();
        set.insert(MAIN_FUNC_ATTR.to_string());
        Self::all_functions(graph, &Some(set))
    }

    /// Get all test functions in a graph.
    pub fn test_functions(graph: &Graph) -> Vec<DataRef> {
        let mut set = FxHashSet::default();
        set.insert(TEST_FUNC_ATTR.to_string());
        Self::all_functions(graph, &Some(set))
    }
}
