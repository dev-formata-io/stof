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

mod value;
use std::ops::Deref;

use bytes::Bytes;
use js_sys::Uint8Array;
use rustc_hash::FxHashSet;
use wasm_bindgen::prelude::*;
use crate::{js::value::to_stof_value, model::{import::parse_json_object_value, Graph}, runtime::{Runtime, Val}};


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
}


#[wasm_bindgen]
/// Stof Document.
/// This is the entire interface for wasm/js (Runtime + Graph).
pub struct Stof {
    graph: Graph,
}
impl From<Graph> for Stof {
    fn from(graph: Graph) -> Self {
        Self { graph }
    }
}
#[wasm_bindgen]
impl Stof {
    #[wasm_bindgen(constructor)]
    /// Construct a new document.
    pub fn new() -> Self {
        Self { graph: Graph::default() }
    }


    /*****************************************************************************
     * Runtime.
     *****************************************************************************/
    
    /// Run functions with the given attribute(s) in this document.
    /// Attributes defaults to #[main] functions if null or undefined.
    pub fn run(&mut self, attributes: JsValue) -> Result<String, String> {
        let mut attrs = FxHashSet::default();
        match to_stof_value(attributes, &self) {
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
        Runtime::run_attribute_functions(&mut self.graph, None, &Some(attrs), true)
    }

    /// Call a singular function in the document (by path).
    /// If no arguments, pass undefined as args.
    /// Otherwise, pass an array of arguments as args.
    pub fn call(&mut self, path: &str, args: JsValue) -> Result<JsValue, String> {
        let mut arguments = vec![];
        match to_stof_value(args, &self) {
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
        match Runtime::call(&mut self.graph, path, arguments) {
            Ok(res) => Ok(JsValue::from(res)),
            Err(err) => Err(err.to_string())
        }
    }


    /*****************************************************************************
     * I/O
     *****************************************************************************/
    
    /// Parse Stof into this document, optionally within the specified node (pass null for root node).
    pub fn parse(&mut self, stof: &str, node: JsValue) -> Result<bool, String> {
        self.string_import(stof, "stof", node)
    }

    #[wasm_bindgen(js_name = objImport)]
    /// Import a JS object value.
    pub fn js_obj_import(&mut self, js_obj: JsValue, node: JsValue) -> Result<bool, String> {
        if let Ok(value) = serde_wasm_bindgen::from_value::<serde_json::Value>(js_obj) {
            let val = to_stof_value(node, &self);
            let mut parse_node = self.graph.ensure_main_root();
            match val {
                Val::Obj(node) => {
                    if node.node_exists(&self.graph) {
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
            parse_json_object_value(&mut self.graph, &parse_node, value);
            return Ok(true);
        }
        Err(format!("failed to import js object"))
    }

    #[wasm_bindgen(js_name = stringImport)]
    /// String import, using a format of choice (including stof).
    pub fn string_import(&mut self, src: &str, format: &str, node: JsValue) -> Result<bool, String> {
        let val = to_stof_value(node, &self);
        let mut parse_node = self.graph.ensure_main_root();
        match val {
            Val::Obj(node) => {
                if node.node_exists(&self.graph) {
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
        match self.graph.string_import(format, src, Some(parse_node)) {
            Ok(_) => Ok(true),
            Err(err) => Err(err.to_string())
        }
    }
    
    #[wasm_bindgen(js_name = binaryImport)]
    /// Binary import (Uint8Array), using a format of choice.
    /// Format can also be a content type (for HTTP-like situations).
    pub fn binary_import(&mut self, bytes: JsValue, format: &str, node: JsValue) -> Result<bool, String> {
        let val = to_stof_value(node, &self);
        let mut parse_node = self.graph.ensure_main_root();
        match val {
            Val::Obj(node) => {
                if node.node_exists(&self.graph) {
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
        match self.graph.binary_import(format, bytes, Some(parse_node)) {
            Ok(_) => Ok(true),
            Err(err) => Err(err.to_string())
        }
    }

    #[wasm_bindgen(js_name = stringExport)]
    /// String export, using a format of choice.
    pub fn string_export(&self, format: &str, node: JsValue) -> Result<String, String> {
        let val = to_stof_value(node, &self);
        let exp_node;
        match val {
            Val::Obj(node) => {
                if node.node_exists(&self.graph) {
                    exp_node = node;
                } else {
                    return Err(format!("export node not found"));
                }
            },
            Val::Null |
            Val::Void => {
                if let Some(root) = self.graph.main_root() {
                    exp_node = root;
                } else {
                    return Err(format!("export node not found"));
                }
            },
            _ => {
                return Err(format!("export node not found"));
            }
        }
        match self.graph.string_export(format, Some(exp_node)) {
            Ok(val) => Ok(val),
            Err(err) => Err(err.to_string())
        }
    }

    #[wasm_bindgen(js_name = binaryExport)]
    /// Binary export (Uint8Array), using a format of choice.
    /// Format can also be a content type (for HTTP-like situations).
    pub fn binary_export(&self, format: &str, node: JsValue) -> Result<JsValue, String> {
        let val = to_stof_value(node, &self);
        let exp_node;
        match val {
            Val::Obj(node) => {
                if node.node_exists(&self.graph) {
                    exp_node = node;
                } else {
                    return Err(format!("export node not found"));
                }
            },
            Val::Null |
            Val::Void => {
                if let Some(root) = self.graph.main_root() {
                    exp_node = root;
                } else {
                    return Err(format!("export node not found"));
                }
            },
            _ => {
                return Err(format!("export node not found"));
            }
        }
        match self.graph.binary_export(format, Some(exp_node)) {
            Ok(bytes) => Ok(JsValue::from(Uint8Array::from(bytes.as_ref()))),
            Err(err) => Err(err.to_string())
        }
    }
}
