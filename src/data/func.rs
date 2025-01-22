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

use std::collections::{BTreeMap, HashMap};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use crate::{Data, IntoDataRef, SData, SDataRef, SDoc, SGraph, SNodeRef};
use super::{lang::{Expr, Statements, StatementsRes}, SType, SVal};


/// Stof function kind.
pub const FUNC_KIND: &str = "fnc";


/// Stof function.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SFunc {
    /// ID of this function.
    /// This will also be the SDataRef ID.
    pub id: String,

    pub name: String,
    pub params: Vec<SParam>,
    pub statements: Statements,
    
    /// Return type.
    pub rtype: SType,

    /// Attributes.
    pub attributes: BTreeMap<String, SVal>,
}
impl IntoDataRef for SFunc {
    fn data_ref(&self) -> SDataRef {
        SDataRef::from(&self.id)
    }
}
impl Data for SFunc {
    fn kind(&self) -> String {
        FUNC_KIND.to_string()
    }
    fn set_ref(&mut self, to_ref: impl IntoDataRef) {
        self.id = to_ref.data_ref().id;
    }
}
impl SFunc {
    /// New function.
    /// AFunction::new("myFunc", vec![("name", SType::string()).into()], SType::string(), vec![].into());
    pub fn new(name: &str, params: Vec<SParam>, rtype: SType, statements: Statements) -> Self {
        Self {
            name: name.to_owned(),
            params,
            rtype,
            statements,
            id: Default::default(),
            attributes: Default::default(),
        }
    }

    /// Get all functions in a graph.
    pub fn all_funcs(graph: &SGraph) -> Vec<Self> {
        let mut funcs = Vec::new();
        for root in &graph.roots {
            funcs.append(&mut Self::recursive_funcs(graph, root));
        }
        funcs
    }

    /// Get all functions on a node recursively.
    pub fn recursive_funcs(graph: &SGraph, node: &SNodeRef) -> Vec<Self> {
        let mut res = Vec::new();
        if let Some(node) = node.node(graph) {
            for dref in node.recursive_prefix_selection(graph, FUNC_KIND) {
                if let Ok(func) = SData::data::<SFunc>(graph, dref) {
                    res.push(func);
                }
            }
        }
        res
    }

    /// Get all functions on a node.
    pub fn funcs(graph: &SGraph, node: &SNodeRef) -> Vec<Self> {
        let mut res = Vec::new();
        if let Some(node) = node.node(graph) {
            for dref in node.prefix_selection(FUNC_KIND) {
                if let Ok(func) = SData::data::<SFunc>(graph, dref) {
                    res.push(func);
                }
            }
        }
        res
    }

    /// Get all functions on a node as a hashmap.
    pub fn func_map(graph: &SGraph, node: &SNodeRef) -> HashMap<String, Self> {
        let mut res = HashMap::new();
        if let Some(node) = node.node(graph) {
            for dref in node.prefix_selection(FUNC_KIND) {
                if let Ok(func) = SData::data::<SFunc>(graph, dref) {
                    res.insert(func.name.clone(), func);
                }
            }
        }
        res
    }

    /// Get an adjacent func to this func.
    pub fn adjacent(&self, graph: &SGraph, path: &str, sep: char) -> Option<Self> {
        if let Some(data) = self.data_ref().data(graph) {
            for node_ref in &data.nodes {
                let func = Self::func(graph, path, sep, Some(node_ref));
                if func.is_some() {
                    return func;
                }
            }
        }
        None
    }

    /// Get the first func that matches a path given.
    pub fn first_match(graph: &SGraph, paths: Vec<&str>, sep: char, start: Option<&SNodeRef>) -> Option<Self> {
        for path in paths {
            let func = Self::func(graph, path, sep, start);
            if func.is_some() {
                return func;
            }
        }
        None
    }

    /// Get a func from a path with the given separator.
    /// Last name in the path is the func name.
    /// If path is only the func, will search on start if any or search each root in the graph.
    pub fn func(graph: &SGraph, path: &str, sep: char, start: Option<&SNodeRef>) -> Option<Self> {
        let mut items: Vec<&str> = path.split(sep).collect();

        let func_name = items.pop().unwrap();
        if items.len() > 0 {
            if let Some(node) = graph.node_from(&items.join("/"), start) {
                for dref in node.prefix_selection(FUNC_KIND) {
                    if let Ok(func) = SData::data::<SFunc>(graph, dref) {
                        if func.name == func_name {
                            return Some(func);
                        }
                    }
                }
            }
        } else {
            if let Some(start) = start {
                if let Some(node) = start.node(graph) {
                    for dref in node.prefix_selection(FUNC_KIND) {
                        if let Ok(func) = SData::data::<SFunc>(graph, dref) {
                            if func.name == func_name {
                                return Some(func);
                            }
                        }
                    }
                }
            } else {
                for root_ref in &graph.roots {
                    if let Some(node) = root_ref.node(graph) {
                        for dref in node.prefix_selection(FUNC_KIND) {
                            if let Ok(func) = SData::data::<SFunc>(graph, dref) {
                                if func.name == func_name {
                                    return Some(func);
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Call this function with the given doc.
    /// Parameters get put onto the doc stack, and statements get executed.
    pub fn call(&self, pid: &str, doc: &mut SDoc, mut parameters: Vec<SVal>, add_self: bool) -> Result<SVal> {
        // Validate the number of parameters required to call this function
        if self.params.len() != parameters.len() {
            let mut index = parameters.len();
            while index < self.params.len() {
                let param = &self.params[index];
                if let Some(default) = &param.default {
                    let value = default.exec(pid, doc)?;
                    parameters.push(value);
                } else {
                    break;
                }
                index += 1;
            }
        }
        if self.params.len() != parameters.len() {
            return Err(anyhow!("Gave incorrect parameters for function: {}", &self.name));
        }

        // Add self to doc self stack
        if add_self {
            if let Some(data) = doc.graph.data_from_ref(&self.data_ref()) {
                if let Some(nref) = data.nodes.last() {
                    doc.push_self(pid, nref.clone());
                } else {
                    // Data isn't in the graph, so add main as root
                    if doc.graph.roots.len() < 1 {
                        doc.graph.insert_root("root");
                    }
                    let main_ref = doc.graph.main_root().unwrap();
                    doc.push_self(pid, main_ref);
                }
            } else {
                // Data isn't in the graph, so add main as root
                if doc.graph.roots.len() < 1 {
                    doc.graph.insert_root("root");
                }
                let main_ref = doc.graph.main_root().unwrap();
                doc.push_self(pid, main_ref);
            }
        }

        // Validate the types of parameters given as we push them to the doc stack
        let mut added = Vec::new();
        parameters.reverse();
        for i in 0..parameters.len() {
            let mut arg_val = parameters.pop().unwrap();
            let mut arg_type = arg_val.stype(&doc.graph);
            let param = &self.params[i];

            if arg_type != param.ptype {
                arg_val = arg_val.cast(param.ptype.clone(), pid, doc)?;
                arg_type = param.ptype.clone(); // for null, etc..
            }

            if arg_type == param.ptype {
                let name = &param.name;
                added.push(name.clone());
                doc.add_variable(pid, name, arg_val);
            } else {
                for name in added {
                    doc.drop(pid, &name);
                }
                return Err(anyhow!("Failed to match parameter types for function: {}", &self.name));
            }
        }

        // Execute all of the statements with this doc in a scope (block)
        doc.push_call_stack(pid, self.data_ref());
        doc.new_scope(pid);
        let statements_res = self.statements.exec(pid, doc);
        doc.end_scope(pid);
        doc.pop_call_stack(pid);

        // Pop the self stack!
        if add_self {
            doc.pop_self(pid);
        }

        // Validate the return/result of this function
        let mut res = None;
        match statements_res {
            Ok(statements_res) => {
                match statements_res {
                    StatementsRes::Return(on_stack) => {
                        if on_stack { res = doc.pop(pid); }
                    },
                    _ => {}
                }
            },
            Err(error) => {
                return Err(error);
            }
        }
        if self.rtype.is_void() && res.is_none() {
            return Ok(SVal::Void);
        } else if res.is_some() {
            let mut res = res.unwrap();
            let mut res_type = res.stype(&doc.graph);

            // Try casting result to our return type if needed
            if res_type != self.rtype {
                if let Ok(new_res) = res.cast(self.rtype.clone(), pid, doc) {
                    res = new_res;
                    res_type = self.rtype.clone();
                }
            }

            if res_type == self.rtype {
                return Ok(res);
            }
            return Err(anyhow!("Failed to validate return of function: {}. Expected: {:?}, received: {:?}: {:?}", &self.name, self.rtype, res, res.stype(&doc.graph)));
        }
        Err(anyhow!("Failed to validate return of function: {}. Expected: {:?}, received: {:?}", &self.name, self.rtype, res))
    }
}


/// Function parameter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SParam {
    pub name: String,
    pub ptype: SType,
    pub default: Option<Expr>,
}
impl SParam {
    /// New parameter.
    pub fn new(name: &str, ptype: SType, default: Option<Expr>) -> Self {
        Self {
            name: name.into(),
            ptype,
            default,
        }
    }
}
impl From<(&str, SType)> for SParam {
    fn from((name, atype): (&str, SType)) -> Self {
        Self::new(name, atype, None)
    }
}
impl From<(&str, &str)> for SParam {
    fn from((name, atype): (&str, &str)) -> Self {
        Self::new(name, SType::from(atype), None)
    }
}
impl From<(&str, &str, SVal)> for SParam {
    fn from((name, atype, default): (&str, &str, SVal)) -> Self {
        Self::new(name, SType::from(atype), Some(Expr::Literal(default)))
    }
}
