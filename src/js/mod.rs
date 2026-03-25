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

mod value;
mod func;
use std::{cell::RefCell, ops::Deref, sync::Arc};

use bytes::Bytes;
use js_sys::Uint8Array;
use nanoid::nanoid;
use rustc_hash::FxHashSet;
use wasm_bindgen::prelude::*;
use crate::{js::{func::StofFunc, value::to_graph_value}, model::{Graph, Profile, import::parse_json_object_value}, runtime::{Runtime, Val, Variable, instruction::Instruction, instructions::Base, proc::ProcEnv}};


// Workaround for Wasm-Pack Error
#[cfg(target_family = "wasm")]
mod wasm_workaround {
    unsafe extern "C" {
        pub(super) fn __wasm_call_ctors();
    }
}
#[wasm_bindgen(start)]
fn start() {
    // stof::data::field::_::__ctor::h5fcded453a464929: Read a negative address value from the stack. Did we run out of memory?
    #[cfg(target_family = "wasm")]
    unsafe { wasm_workaround::__wasm_call_ctors() };
    console_error_panic_hook::set_once();
}


#[wasm_bindgen]
/// Stof Document.
/// This is the entire interface for wasm/js (Runtime + Graph).
pub struct Stof {
    docid: String,
    graph: RefCell<Graph>,
}
impl From<Graph> for Stof {
    fn from(graph: Graph) -> Self {
        Self {
            docid: nanoid!(10),
            graph: RefCell::new(graph),
        }
    }
}
impl Stof {
    #[inline]
    fn graph_mut(&self) -> std::cell::RefMut<'_, Graph> {
        self.graph.borrow_mut()
    }
}
#[wasm_bindgen]
impl Stof {
    #[wasm_bindgen(constructor)]
    /// Construct a new document.
    pub fn new() -> Self {
        Self {
            docid: nanoid!(10),
            graph: RefCell::new(Graph::default()),
        }
    }

    /// Get the ID of this document as a string.
    pub fn docid(&self) -> String {
        self.docid.clone()
    }
    
    /// Get a value from this graph using the Stof runtime (all language features supported).
    pub fn get(&self, path: &str, start: JsValue) -> JsValue {
        let instruction: Arc<dyn Instruction> = Arc::new(Base::LoadVariable(path.into(), false, false));
        let mut proc_env = ProcEnv::default();
        let mut graph = self.graph_mut();
        if let Some(main) = graph.main_root() {
            proc_env.self_stack.push(main);
        }
        match to_graph_value(start, &graph) {
            Val::Obj(start) => {
                proc_env.self_stack.push(start);
            },
            _ => {}
        }

        let _ = instruction.exec(&mut proc_env, &mut *graph); // don't care about res
        
        if let Some(var) = proc_env.stack.pop() {
            JsValue::from(var.val.read().clone())
        } else {
            JsValue::NULL
        }
    }

    /// Set a value onto this graph using the Stof runtime.
    pub fn set(&self, path: &str, value: JsValue, start: JsValue) -> bool {
        let mut proc_env = ProcEnv::default();
        let mut graph = self.graph_mut();
        if let Some(main) = graph.main_root() {
            proc_env.self_stack.push(main);
        }
        match to_graph_value(start, &graph) {
            Val::Obj(start) => {
                proc_env.self_stack.push(start);
            },
            _ => {}
        }
        proc_env.stack.push(Variable::val(to_graph_value(value, &graph)));
        let instruction: Arc<dyn Instruction> = Arc::new(Base::SetVariable(path.into()));
        match instruction.exec(&mut proc_env, &mut *graph) {
            Ok(_res) => true,
            Err(_err) => false
        }
    }


    /*****************************************************************************
     * Runtime.
     *****************************************************************************/
    
    /// Run functions with the given attribute(s) in this document.
    /// Attributes defaults to #[main] functions if null or undefined.
    pub async fn run_with_gate(&self, attributes: JsValue, acquire: &js_sys::Function, release: &js_sys::Function) -> Result<String, String> {
        let mut attrs = FxHashSet::default();
        {
            let graph = self.graph.borrow();
            match to_graph_value(attributes, &graph) {
                Val::Str(attribute) => {
                    attrs.insert(attribute.to_string());
                },
                Val::List(vals) => {
                    for val in vals {
                        match val.read().deref() {
                            Val::Str(att) => { attrs.insert(att.to_string()); },
                            _ => {}
                        }
                    }
                },
                Val::Set(set) => {
                    for val in set {
                        match val.read().deref() {
                            Val::Str(att) => { attrs.insert(att.to_string()); },
                            _ => {}
                        }
                    }
                },
                _ => {
                    attrs.insert("main".into());
                }
            }
        }
        Runtime::async_run_attribute_functions_with_gate(&self.graph, None, &Some(attrs), true, acquire, release).await
    }

    /// Synchronous run functions with the given attribute(s) in this document.
    /// Attributes defaults to #[main] functions if null or undefined.
    /// Async TS lib functions will not work with this, but it will be faster.
    pub fn sync_run(&self, attributes: JsValue) -> Result<String, String> {
        let mut attrs = FxHashSet::default();
        let mut graph = self.graph_mut();
        match to_graph_value(attributes, &graph) {
            Val::Str(attribute) => {
                attrs.insert(attribute.to_string());
            },
            Val::List(vals) => {
                for val in vals {
                    match val.read().deref() {
                        Val::Str(att) => { attrs.insert(att.to_string()); },
                        _ => {}
                    }
                }
            },
            Val::Set(set) => {
                for val in set {
                    match val.read().deref() {
                        Val::Str(att) => { attrs.insert(att.to_string()); },
                        _ => {}
                    }
                }
            },
            _ => {
                attrs.insert("main".into());
            }
        }
        Runtime::run_attribute_functions(&mut *graph, None, &Some(attrs), true)
    }

    /// Call a singular function in the document (by path).
    /// If no arguments, pass undefined as args.
    /// Otherwise, pass an array of arguments as args.
    pub async fn call_with_gate(&self, path: &str, args: JsValue, acquire: &js_sys::Function, release: &js_sys::Function) -> Result<JsValue, String> {
        let mut arguments = vec![];
        {
            let graph = self.graph.borrow();
            match to_graph_value(args, &graph) {
                Val::List(vals) => {
                    for val in vals {
                        arguments.push(val.read().clone());
                    }
                },
                Val::Void => { /* Undefined value. */ },
                val => {
                    arguments.push(val);
                }
            }
        }
        match Runtime::async_call_with_gate(&self.graph, path, arguments, acquire, release).await {
            Ok(res) => Ok(JsValue::from(res)),
            Err(err) => Err(err.to_string())
        }
    }

    /// Synchronous call a singular function in the document (by path).
    /// If no arguments, pass undefined as args.
    /// Otherwise, pass an array of arguments as args.
    /// Async TS lib functions will not work with this, but it will be faster.
    pub fn sync_call(&self, path: &str, args: JsValue) -> Result<JsValue, String> {
        let mut arguments = vec![];
        let mut graph = self.graph_mut();
        match to_graph_value(args, &graph) {
            Val::List(vals) => {
                for val in vals {
                    arguments.push(val.read().clone());
                }
            },
            Val::Void => { /* Undefined value. */ },
            val => {
                arguments.push(val);
            }
        }
        match Runtime::call(&mut *graph, path, arguments) {
            Ok(res) => Ok(JsValue::from(res)),
            Err(err) => Err(err.to_string())
        }
    }


    /*****************************************************************************
     * Interop.
     *****************************************************************************/
    
    /// Insert a JS function as a library function, available in Stof.
    pub fn js_library_function(&self, func: StofFunc) {
        let mut graph = self.graph_mut();
        graph.insert_libfunc(func.get_func());
    }


    /*****************************************************************************
     * I/O
     *****************************************************************************/
    
    /// Parse Stof into this document, optionally within the specified node (pass null for root node).
    pub fn parse(&self, stof: &str, node: JsValue, profile: &str) -> Result<bool, String> {
        self.string_import(stof, "stof", node, profile)
    }

    #[wasm_bindgen(js_name = objImport)]
    /// Import a JS object value.
    pub fn js_obj_import(&self, js_obj: JsValue, node: JsValue) -> Result<bool, String> {
        if let Ok(value) = serde_wasm_bindgen::from_value::<serde_json::Value>(js_obj) {
            let mut graph = self.graph_mut();
            let val = to_graph_value(node, &graph);
            let mut parse_node = graph.ensure_main_root();
            match val {
                Val::Obj(node) => {
                    if node.node_exists(&graph) {
                        parse_node = node;
                    } else {
                        return Ok(false);
                    }
                },
                Val::Null |
                Val::Void => {},
                _ => {
                    return Ok(false);
                }
            }
            parse_json_object_value(&mut *graph, &parse_node, value);
            return Ok(true);
        }
        Err(format!("failed to import js object"))
    }

    #[wasm_bindgen(js_name = stringImport)]
    /// String import, using a format of choice (including stof).
    pub fn string_import(&self, src: &str, format: &str, node: JsValue, profile: &str) -> Result<bool, String> {
        let mut graph = self.graph_mut();
        let val = to_graph_value(node, &graph);
        let mut parse_node = graph.ensure_main_root();
        match val {
            Val::Obj(node) => {
                if node.node_exists(&graph) {
                    parse_node = node;
                } else {
                    return Ok(false);
                }
            },
            Val::Null |
            Val::Void => {},
            _ => {
                return Ok(false);
            }
        }

        let profile = match profile {
            "prod" => Profile::prod(),
            "test" => Profile::test(),
            "prod_docs" => Profile::docs(false),
            "docs" => Profile::docs(true),
            _ => Profile::default(),
        };

        match graph.string_import(format, src, Some(parse_node), &profile) {
            Ok(_) => Ok(true),
            Err(err) => Err(err.to_string())
        }
    }
    
    #[wasm_bindgen(js_name = binaryImport)]
    /// Binary import (Uint8Array), using a format of choice.
    /// Format can also be a content type (for HTTP-like situations).
    pub fn binary_import(&self, bytes: JsValue, format: &str, node: JsValue, profile: &str) -> Result<bool, String> {
        let mut graph = self.graph_mut();
        let val = to_graph_value(node, &graph);
        let mut parse_node = graph.ensure_main_root();
        match val {
            Val::Obj(node) => {
                if node.node_exists(&graph) {
                    parse_node = node;
                } else {
                    return Ok(false);
                }
            },
            Val::Null |
            Val::Void => {},
            _ => {
                return Ok(false);
            }
        }
        let array = Uint8Array::from(bytes);
        let bytes = Bytes::from(array.to_vec());
        let profile = match profile {
            "prod" => Profile::prod(),
            "test" => Profile::test(),
            "prod_docs" => Profile::docs(false),
            "docs" => Profile::docs(true),
            _ => Profile::default(),
        };
        match graph.binary_import(format, bytes, Some(parse_node), &profile) {
            Ok(_) => Ok(true),
            Err(err) => Err(err.to_string())
        }
    }

    #[wasm_bindgen(js_name = stringExport)]
    /// String export, using a format of choice.
    pub fn string_export(&self, format: &str, node: JsValue) -> Result<String, String> {
        let graph = self.graph.borrow();
        let val = to_graph_value(node, &graph);
        let exp_node;
        match val {
            Val::Obj(node) => {
                if node.node_exists(&graph) {
                    exp_node = node;
                } else {
                    return Err(format!("export node not found"));
                }
            },
            Val::Null |
            Val::Void => {
                if let Some(root) = graph.main_root() {
                    exp_node = root;
                } else {
                    return Err(format!("export node not found"));
                }
            },
            _ => {
                return Err(format!("export node not found"));
            }
        }
        match graph.string_export(format, Some(exp_node)) {
            Ok(val) => Ok(val),
            Err(err) => Err(err.to_string())
        }
    }

    #[wasm_bindgen(js_name = binaryExport)]
    /// Binary export (Uint8Array), using a format of choice.
    /// Format can also be a content type (for HTTP-like situations).
    pub fn binary_export(&self, format: &str, node: JsValue) -> Result<JsValue, String> {
        let graph = self.graph.borrow();
        let val = to_graph_value(node, &graph);
        let exp_node;
        match val {
            Val::Obj(node) => {
                if node.node_exists(&graph) {
                    exp_node = node;
                } else {
                    return Err(format!("export node not found"));
                }
            },
            Val::Null |
            Val::Void => {
                if let Some(root) = graph.main_root() {
                    exp_node = root;
                } else {
                    return Err(format!("export node not found"));
                }
            },
            _ => {
                return Err(format!("export node not found"));
            }
        }
        match graph.binary_export(format, Some(exp_node)) {
            Ok(bytes) => Ok(JsValue::from(Uint8Array::from(bytes.as_ref()))),
            Err(err) => Err(err.to_string())
        }
    }
}
