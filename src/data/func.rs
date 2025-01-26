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

use std::collections::{BTreeMap, HashSet};
use serde::{Deserialize, Serialize};
use crate::{Data, SData, SDataRef, SDoc, SGraph, SNodeRef};
use super::{lang::{Expr, SError, Statements, StatementsRes}, SType, SVal};


/// Stof function.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SFunc {
    pub name: String,
    pub params: Vec<SParam>,
    pub statements: Statements,
    pub rtype: SType,
    pub attributes: BTreeMap<String, SVal>,
}

#[typetag::serde(name = "_SFunc")]
impl Data for SFunc {}

impl SFunc {
    /// New function.
    /// AFunction::new("myFunc", vec![("name", SType::string()).into()], SType::string(), vec![].into());
    pub fn new(name: &str, params: Vec<SParam>, rtype: SType, statements: Statements) -> Self {
        Self {
            name: name.to_owned(),
            params,
            rtype,
            statements,
            attributes: Default::default(),
        }
    }

    /// Get all functions in a graph.
    pub fn all_funcs(graph: &SGraph) -> HashSet<SDataRef> {
        let mut funcs = HashSet::new();
        for root in &graph.roots {
            for dref in Self::recursive_func_refs(graph, root) {
                funcs.insert(dref);
            }
        }
        funcs
    }

    /// Get all function refs on a node recursively.
    pub fn recursive_func_refs(graph: &SGraph, node: &SNodeRef) -> HashSet<SDataRef> {
        let mut res = Self::func_refs(graph, node);
        if let Some(node) = node.node(graph) {
            for child in &node.children {
                for dref in Self::recursive_func_refs(graph, child) {
                    res.insert(dref);
                }
            }
        }
        res
    }

    /// Get all functions on a node.
    pub fn funcs<'a>(graph: &'a SGraph, node: &SNodeRef) -> Vec<&'a Self> {
        let mut res = Vec::new();
        if let Some(node) = node.node(graph) {
            for func in node.data::<Self>(graph) {
                res.push(func);
            }
        }
        res
    }

    /// Get all function references on a node.
    pub fn func_refs(graph: &SGraph, node: &SNodeRef) -> HashSet<SDataRef> {
        let mut res = HashSet::new();
        if let Some(node) = node.node(graph) {
            for func in node.data_refs::<Self>(graph) {
                res.insert(func);
            }
        }
        res
    }

    /// Get a func from a path with the given separator.
    /// Last name in the path is the func name.
    /// If path is only the func, will search on start if any or search each root in the graph.
    pub fn func<'a>(graph: &'a SGraph, path: &str, sep: char, start: Option<&SNodeRef>) -> Option<&'a Self> {
        let mut items: Vec<&str> = path.split(sep).collect();

        let func_name = items.pop().unwrap();
        if items.len() > 0 {
            if let Some(node) = graph.node_from(&items.join("/"), start) {
                for func in node.data::<Self>(graph) {
                    if func.name == func_name {
                        return Some(func);
                    }
                }
            }
        } else {
            if let Some(start) = start {
                if let Some(node) = start.node(graph) {
                    for func in node.data::<Self>(graph) {
                        if func.name == func_name {
                            return Some(func);
                        }
                    }
                }
            } else {
                for root_ref in &graph.roots {
                    if let Some(node) = root_ref.node(graph) {
                        for func in node.data::<Self>(graph) {
                            if func.name == func_name {
                                return Some(func);
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Get a func ref from a path with the given separator.
    /// Last name in the path is the func name.
    /// If path is only the func, will search on start if any or search each root in the graph.
    pub fn func_ref(graph: &SGraph, path: &str, sep: char, start: Option<&SNodeRef>) -> Option<SDataRef> {
        let mut items: Vec<&str> = path.split(sep).collect();

        let func_name = items.pop().unwrap();
        if items.len() > 0 {
            if let Some(node) = graph.node_from(&items.join("/"), start) {
                for dref in &node.data {
                    if let Some(func) = SData::get::<Self>(graph, dref) {
                        if func.name == func_name {
                            return Some(dref.clone());
                        }
                    }
                }
            }
        } else {
            if let Some(start) = start {
                if let Some(node) = start.node(graph) {
                    for dref in &node.data {
                        if let Some(func) = SData::get::<Self>(graph, dref) {
                            if func.name == func_name {
                                return Some(dref.clone());
                            }
                        }
                    }
                }
            } else {
                for root_ref in &graph.roots {
                    if let Some(node) = root_ref.node(graph) {
                        for dref in &node.data {
                            if let Some(func) = SData::get::<Self>(graph, dref) {
                                if func.name == func_name {
                                    return Some(dref.clone());
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
    pub fn call(&self, dref: &SDataRef, pid: &str, doc: &mut SDoc, mut parameters: Vec<SVal>, add_self: bool) -> Result<SVal, SError> {
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
            doc.push_call_stack(pid, dref);
            let error = SError::call(pid, &doc, &format!("received incorrect parameters for function call, expecting ({:?})", &self.params));
            doc.pop_call_stack(pid);
            return Err(error);
        }

        // Add self to doc self stack
        if add_self {
            if let Some(data) = doc.graph.data_from_ref(dref) {
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
                doc.push_call_stack(pid, dref);
                let error = SError::call(pid, &doc, &format!("arguments do not match expected parameter types and cannot be converted, expecting ({:?})", &self.params));
                doc.pop_call_stack(pid);
                return Err(error);
            }
        }

        // Execute all of the statements with this doc in a scope (block)
        doc.push_call_stack(pid, dref);
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
            let error = SError::call(pid, &doc, &format!("return type ({:?}) does not match the expected type ({:?})", res_type, self.rtype));
            return Err(error);
        }
        Err(SError::call(pid, &doc, &format!("return value ({:?}) does not match the expected type ({:?})", res, self.rtype)))
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
