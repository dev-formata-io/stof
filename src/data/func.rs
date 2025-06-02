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
impl Data for SFunc {
    fn core_data(&self) -> bool {
        return true;
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
            attributes: Default::default(),
        }
    }

    /// Get all functions in a graph.
    pub fn all_funcs(graph: &SGraph) -> HashSet<SDataRef> {
        let mut funcs = HashSet::new();
        for (_, node) in &graph.nodes.store {
            for func in node.data_refs::<Self>(graph) {
                funcs.insert(func);
            }
        }
        funcs
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

    /// Call this function.
    pub fn call(dref: &SDataRef, pid: &str, doc: &mut SDoc, parameters: Vec<SVal>, add_self: bool, allow_async: bool) -> Result<SVal, SError> {
        let params;
        let statements;
        let rtype;
        if let Some(func) = SData::get::<Self>(&doc.graph, dref) {
            params = func.params.clone();
            statements = func.statements.clone();
            rtype = func.rtype.clone();
        } else {
            return Err(SError::call(pid, &doc, "data reference given was not a function"));
        }
        Self::call_internal(dref, pid, doc, parameters, add_self, &params, &statements, &rtype, allow_async)
    }

    /// Internal call with this data reference.
    #[allow(unused)]
    pub fn call_internal(dref: &SDataRef, pid: &str, doc: &mut SDoc, mut parameters: Vec<SVal>, add_self: bool, params: &Vec<SParam>, statements: &Statements, rtype: &SType, allow_async: bool) -> Result<SVal, SError> {
        #[cfg(feature = "async")]
        // Async function call returns a string handle instead of the resulting value
        {
            use crate::TokioPool;
            if allow_async && doc.libraries.libraries.contains_key("Async") {
                use tokio::runtime::Handle;
                if Handle::try_current().is_ok() {
                    if let Some(func) = SData::get::<Self>(&doc.graph, dref) {
                        if let Some(async_context) = func.attributes.get("async") {
                            let mut contexts = HashSet::new();

                            let data = dref.data(&doc.graph).unwrap();
                            for node in &data.nodes {
                                if node.exists(&doc.graph) {
                                    contexts.insert(node.clone());
                                }
                            }

                            match async_context {
                                SVal::Object(nref) => {
                                    contexts.insert(nref.clone());
                                    return Ok(SVal::String(TokioPool::spawn(&doc, vec![(dref.clone(), parameters)], Some(contexts), None)));
                                },
                                SVal::Array(vals) => {
                                    for val in vals {
                                        match val {
                                            SVal::Object(nref) => {
                                                contexts.insert(nref.clone());
                                            },
                                            _ => {}
                                        }
                                    }
                                    return Ok(SVal::String(TokioPool::spawn(&doc, vec![(dref.clone(), parameters)], Some(contexts), None)));
                                },
                                SVal::String(mode) => {
                                    match mode.as_str() {
                                        "none" |
                                        "scope" |
                                        "local" |
                                        "scoped" => {
                                            return Ok(SVal::String(TokioPool::spawn(&doc, vec![(dref.clone(), parameters)], Some(contexts), None)));
                                        },
                                        _ => {
                                            return Ok(SVal::String(TokioPool::spawn(&doc, vec![(dref.clone(), parameters)], None, None)));
                                        }
                                    }
                                },
                                _ => {
                                    return Ok(SVal::String(TokioPool::spawn(&doc, vec![(dref.clone(), parameters)], None, None)));
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Add self to doc self stack
        if add_self {
            if let Some(data) = doc.graph.data_from_ref(dref) {
                let mut set_self = None;
                for nref in data.nodes.iter().rev() {
                    if nref.exists(&doc.graph) {
                        set_self = Some(nref.clone());
                        break;
                    }
                }

                if let Some(nref) = set_self {
                    doc.push_self(pid, nref);
                } else {
                    // Data isn't in the graph, so add main root as self
                    if doc.graph.roots.len() < 1 {
                        doc.graph.insert_root("root");
                    }
                    let main_ref = doc.graph.main_root().unwrap();
                    doc.push_self(pid, main_ref);
                }
            } else {
                // Data isn't in the graph, so add main root as selt
                if doc.graph.roots.len() < 1 {
                    doc.graph.insert_root("root");
                }
                let main_ref = doc.graph.main_root().unwrap();
                doc.push_self(pid, main_ref);
            }
        }
        
        // Validate the number of parameters required to call this function
        if params.len() != parameters.len() {
            let mut index = parameters.len();
            while index < params.len() {
                let param = &params[index];
                if let Some(default) = &param.default {
                    let value = default.exec(pid, doc);
                    match value {
                        Ok(val) => parameters.push(val),
                        Err(error) => {
                            if add_self {
                                doc.pop_self(pid);
                            }
                            return Err(error);
                        }
                    }
                } else {
                    break;
                }
                index += 1;
            }
        }
        if params.len() != parameters.len() {
            doc.push_call_stack(pid, dref);
            let error = SError::call(pid, &doc, &format!("received incorrect parameters for function call, expecting ({:?})", &params));
            doc.pop_call_stack(pid);
            if add_self {
                doc.pop_self(pid);
            }
            return Err(error);
        }

        // Validate the types of parameters given as we push them to the doc stack
        let mut added = Vec::new();
        parameters.reverse();
        for i in 0..parameters.len() {
            let mut arg_val = parameters.pop().unwrap();
            let mut arg_type = arg_val.stype(&doc.graph);
            let param = &params[i];

            if arg_type != param.ptype {
                let arg_val_res = arg_val.cast(param.ptype.clone(), pid, doc);
                match arg_val_res {
                    Ok(val) => arg_val = val,
                    Err(error) => {
                        if add_self {
                            doc.pop_self(pid);
                        }
                        return Err(error);
                    }
                }
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
                let error = SError::call(pid, &doc, &format!("arguments do not match expected parameter types and cannot be converted, expecting ({:?})", &params));
                doc.pop_call_stack(pid);
                if add_self {
                    doc.pop_self(pid);
                }
                return Err(error);
            }
        }

        // Execute all of the statements with this doc in a scope (block)
        doc.push_call_stack(pid, dref);
        let bcs = doc.funcstart_bubble_control(pid);
        doc.new_scope(pid);
        let statements_res = statements.exec(pid, doc);
        doc.end_scope(pid);
        doc.funcset_bubble_control(pid, bcs);
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
        if rtype.is_void() && res.is_none() {
            return Ok(SVal::Void);
        } else if res.is_some() {
            let mut res = res.unwrap();
            let mut res_type = res.stype(&doc.graph);

            // Try casting result to our return type if needed
            if res_type != *rtype {
                if let Ok(new_res) = res.cast(rtype.clone(), pid, doc) {
                    res = new_res;
                    res_type = rtype.clone();
                }
            }

            if res_type == *rtype {
                return Ok(res);
            }
            let error = SError::call(pid, &doc, &format!("return type ({:?}) does not match the expected type ({:?})", res_type, rtype));
            return Err(error);
        }
        Err(SError::call(pid, &doc, &format!("return value ({:?}) does not match the expected type ({:?})", res, rtype)))
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
