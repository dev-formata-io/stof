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

use std::{collections::BTreeMap, sync::Arc};
use bytes::Bytes;
use imbl::Vector;
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use crate::{model::{DataRef, Graph, NodeRef, SId, SPath, StofData}, runtime::{expr::Expr, instruction::{Instruction, Instructions}, Type, Variable}};


/// Attribute used to denote a main function.
pub const MAIN_FUNC_ATTR: SId = SId(Bytes::from_static(b"main"));

/// Attribute used to denote a test function.
pub const TEST_FUNC_ATTR: SId = SId(Bytes::from_static(b"test"));


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Function.
pub struct Func {
    pub params: Vec<Param>,
    pub return_type: Type,
    pub attributes: FxHashMap<SId, Variable>,
    pub instructions: Vector<Arc<dyn Instruction>>,
}

#[typetag::serde(name = "_Func")]
impl StofData for Func {
    fn core_data(&self) -> bool {
        return true;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Parameter.
pub struct Param {
    pub name: SId,
    pub param_type: Type,
    pub default_expr: Option<Expr>,
}

impl Func {
    /// Create a new function.
    pub fn new(params: Vec<Param>, return_type: Type, instructions: Instructions, attrs: Option<FxHashMap<SId, Variable>>) -> Self {
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
        if let Some(id_path) = spath.to_id_path(&graph, start) {
            if let Some(node) = id_path.node(&graph) {
                return Self::func(graph, &node, &func_name);
            }
        }
        None
    }
    
    #[inline]
    /// Func lookup.
    pub fn func(graph: &Graph, node: &NodeRef, func_name: &SId) -> Option<DataRef> {
        if let Some(node) = node.node(graph) {
            if let Some(dref) = node.data.get(func_name) {
                if dref.type_of::<Self>(&graph) {
                    return Some(dref.clone());
                }
            }
        }
        None
    }

    /// Get all functions on a node, optionally filtered by attributes and optionally recursively.
    pub fn functions(graph: &Graph, node: &NodeRef, attrs: &Option<FxHashSet<SId>>, recursive: bool) -> BTreeMap<SId, DataRef> {
        let mut funcs = BTreeMap::default();
        if let Some(node) = node.node(&graph) {
            for (name, dref) in &node.data {
                if let Some(func) = graph.get_stof_data::<Self>(dref) {
                    if let Some(attrs) = &attrs {
                        for att in attrs {
                            if func.attributes.contains_key(att) {
                                funcs.insert(name.clone(), dref.clone());
                                break;
                            }
                        }
                    } else {
                        funcs.insert(name.clone(), dref.clone());
                    }
                }
            }

            if recursive {
                for child in &node.children {
                    funcs.append(&mut Self::functions(graph, child, attrs, recursive));
                }
            }
        }
        funcs
    }

    /// Get all functions in a graph, optionally filtered by attribute.
    pub fn all_functions(graph: &Graph, attrs: &Option<FxHashSet<SId>>) -> BTreeMap<SId, DataRef> {
        let mut funcs = BTreeMap::default();
        for root in &graph.roots {
            funcs.append(&mut Self::functions(graph, root, attrs, true));
        }
        funcs
    }

    /// Get all main functions in a graph.
    pub fn main_functions(graph: &Graph) -> BTreeMap<SId, DataRef> {
        let mut set = FxHashSet::default();
        set.insert(MAIN_FUNC_ATTR);
        Self::all_functions(graph, &Some(set))
    }

    /// Get all test functions in a graph.
    pub fn test_functions(graph: &Graph) -> BTreeMap<SId, DataRef> {
        let mut set = FxHashSet::default();
        set.insert(TEST_FUNC_ATTR);
        Self::all_functions(graph, &Some(set))
    }
}
