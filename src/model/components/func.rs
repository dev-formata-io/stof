//
// Copyright 2024 Formata, Inc. All rights reserved.
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
use crate::{model::{DataRef, Graph, NodeRef, SId, SPath, StofData}, runtime::{instruction::{Instruction, Instructions}, Type, Val}};


/// Attribute used to denote a main function.
pub const MAIN_FUNC_ATTR: ArcStr = literal!("main");

/// Attribute used to denote a test function.
pub const TEST_FUNC_ATTR: ArcStr = literal!("test");

/// Attribute used to denote an async function.
pub const ASYNC_FUNC_ATTR: ArcStr = literal!("async");

/// If present, the function will not add its location to the self stack when called.
pub const UNSELF_FUNC_ATTR: ArcStr = literal!("unself");


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// Function.
pub struct Func {
    pub params: Vector<Param>,
    pub return_type: Type,
    pub attributes: FxHashMap<String, Val>,
    pub instructions: Vector<Arc<dyn Instruction>>,
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
    pub fn new(params: Vector<Param>, return_type: Type, instructions: Instructions, attrs: Option<FxHashMap<String, Val>>) -> Self {
        let mut attributes = FxHashMap::default();
        if let Some(attr) = attrs {
            attributes = attr;
        }
        Self {
            params,
            return_type,
            instructions: instructions.instructions,
            attributes,
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

    /// Get all functions on a node, optionally filtered by attributes and optionally recursively.
    pub fn functions(graph: &Graph, node: &NodeRef, attrs: &Option<FxHashSet<String>>, recursive: bool) -> FxHashSet<DataRef> {
        let mut funcs = FxHashSet::default();
        if let Some(node) = node.node(&graph) {
            for (_name, dref) in &node.data {
                if let Some(func) = graph.get_stof_data::<Self>(dref) {
                    if let Some(attrs) = &attrs {
                        for att in attrs {
                            if func.attributes.contains_key(att) {
                                funcs.insert(dref.clone());
                                break;
                            }
                        }
                    } else {
                        funcs.insert(dref.clone());
                    }
                }
            }

            if recursive {
                for child in &node.children {
                    for dref in Self::functions(graph, child, attrs, recursive) {
                        funcs.insert(dref);
                    }
                }
            }
        }
        funcs
    }

    /// Get all functions in a graph, optionally filtered by attribute.
    pub fn all_functions(graph: &Graph, attrs: &Option<FxHashSet<String>>) -> FxHashSet<DataRef> {
        let mut funcs = FxHashSet::default();
        for root in &graph.roots {
            for dref in Self::functions(graph, root, attrs, true) {
                funcs.insert(dref);
            }
        }
        funcs
    }

    /// Get all main functions in a graph.
    pub fn main_functions(graph: &Graph) -> FxHashSet<DataRef> {
        let mut set = FxHashSet::default();
        set.insert(MAIN_FUNC_ATTR.to_string());
        Self::all_functions(graph, &Some(set))
    }

    /// Get all test functions in a graph.
    pub fn test_functions(graph: &Graph) -> FxHashSet<DataRef> {
        let mut set = FxHashSet::default();
        set.insert(TEST_FUNC_ATTR.to_string());
        Self::all_functions(graph, &Some(set))
    }
}
