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
use crate::{js::{func::StofFunc, value::to_graph_value}, model::{Graph, NodeRef, Profile, import::parse_json_object_value, seal::SealOptions}, runtime::{JsTask, Runtime, Val, Variable, instruction::Instruction, instructions::Base, proc::ProcEnv}};


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


#[wasm_bindgen(js_name = stofOutline)]
/// The fields and functions in a Stof text, as JSON (see `parser::edit::outline_json`).
pub fn stof_outline(text: &str) -> Result<String, String> {
    let defs = crate::parser::edit::outline(text).map_err(|error| error.to_string())?;
    Ok(crate::parser::edit::outline_json(text, &defs).to_string())
}

#[wasm_bindgen(js_name = stofGenerateKey)]
/// A new secret key (base64, 32 bytes from the system's secure random source).
pub fn stof_generate_key() -> String {
    crate::model::seal::generate_key()
}

#[wasm_bindgen(js_name = stofPublicKey)]
/// The public key (base64) for a secret key.
pub fn stof_public_key(secret_key: &str) -> Result<String, String> {
    crate::model::seal::public_key(secret_key).map_err(|error| error.to_string())
}

#[wasm_bindgen(js_name = stofSign)]
/// Sign a message (a string's UTF-8 bytes, or bytes) under a context with a secret key: the signature (base64).
pub fn stof_sign(secret_key: &str, context: &str, message: JsValue) -> Result<String, String> {
    let bytes = message_bytes(&message)?;
    crate::model::seal::sign(secret_key, context, &bytes).map_err(|error| error.to_string())
}

#[wasm_bindgen(js_name = stofVerify)]
/// Is this a signature over this message (a string or bytes), under this context, by this public key?
pub fn stof_verify(public_key: &str, context: &str, message: JsValue, signature: &str) -> bool {
    match message_bytes(&message) {
        Ok(bytes) => crate::model::seal::verify(public_key, context, &bytes, signature),
        Err(_) => false,
    }
}

/// A message's bytes: a string's UTF-8, or a Uint8Array's bytes.
fn message_bytes(message: &JsValue) -> Result<Vec<u8>, String> {
    if let Some(text) = message.as_string() { return Ok(text.into_bytes()); }
    if message.is_instance_of::<js_sys::Uint8Array>() { return Ok(js_sys::Uint8Array::from(message.clone()).to_vec()); }
    Err("a message must be a string or a Uint8Array (the exact bytes that are signed)".into())
}

#[wasm_bindgen(js_name = stofEdit)]
/// Edit a Stof text in place, with edits as JSON text (see `parser::edit::edits_from_json`).
pub fn stof_edit(text: &str, edits: &str) -> Result<String, String> {
    crate::parser::edit::edit_json(text, edits).map_err(|error| error.to_string())
}


#[wasm_bindgen]
/// Stof Document.
/// This is the entire interface for wasm/js (Runtime + Graph).
pub struct Stof {
    docid: String,
    graph: RefCell<Graph>,
}
#[wasm_bindgen]
/// A call in progress (see `Stof::step`): the host runs it one step at a time, so a trap in the engine ends the
/// call with an error the host can catch.
pub struct StofTask {
    task: JsTask,
}

impl From<Graph> for Stof {
    fn from(graph: Graph) -> Self {
        Self {
            docid: nanoid!(10),
            graph: RefCell::new(graph),
        }
    }
}
/// While Stof runs a step, the document is borrowed. A library function can call back into it with the async methods
/// (`await doc.call(..)`): those run after the step, once the document is free. A synchronous method called during
/// the step (Ex. `doc.get` before the function's first await) finds it busy: this error, instead of crashing the engine.
const BUSY: &str = "the document is busy running a step: from a library function, call back with the async methods (await doc.call(..), doc.callSource(..)), or call synchronous ones (get, set, sync_call, ..) after an await";

impl Stof {
    #[inline]
    fn graph_mut(&self) -> Result<std::cell::RefMut<'_, Graph>, String> {
        self.graph.try_borrow_mut().map_err(|_| BUSY.to_string())
    }

    #[inline]
    fn graph_ref(&self) -> Result<std::cell::Ref<'_, Graph>, String> {
        self.graph.try_borrow().map_err(|_| BUSY.to_string())
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
    pub fn get(&self, path: &str, start: JsValue) -> Result<JsValue, String> {
        let mut proc_env = ProcEnv::default();
        let mut graph = self.graph_mut()?;
        if let Some(main) = graph.main_root() {
            proc_env.self_stack.push(main);
        }
        match to_graph_value(start, &graph) {
            Val::Obj(start) => {
                proc_env.self_stack.push(start);
            },
            _ => {}
        }
        let path = graph.host_path(path, proc_env.self_stack.last());
        let instruction: Arc<dyn Instruction> = Arc::new(Base::LoadVariable(path.into(), false, false));

        let _ = instruction.exec(&mut proc_env, &mut *graph); // don't care about res
        
        if let Some(var) = proc_env.stack.pop() {
            Ok(JsValue::from(var.val.read().clone()))
        } else {
            Ok(JsValue::NULL)
        }
    }

    /// Set a value onto this graph using the Stof runtime.
    pub fn set(&self, path: &str, value: JsValue, start: JsValue) -> Result<bool, String> {
        let mut proc_env = ProcEnv::default();
        let mut graph = self.graph_mut()?;
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
        let path = graph.host_path(path, proc_env.self_stack.last());
        let instruction: Arc<dyn Instruction> = Arc::new(Base::SetVariable(path.into()));
        match instruction.exec(&mut proc_env, &mut *graph) {
            Ok(_res) => Ok(true),
            Err(_err) => Ok(false)
        }
    }


    /*****************************************************************************
     * Runtime.
     *****************************************************************************/
    
    /// Start running functions with the given attribute(s) in this document (#[main] by default), as a task the
    /// host runs with `step` and then `report` (doc.ts `run`).
    pub fn start_run(&self, attributes: JsValue) -> Result<StofTask, String> {
        let mut attrs = FxHashSet::default();
        {
            let graph = self.graph_ref()?;
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
        let mut graph = self.graph_mut()?;
        Ok(StofTask { task: JsTask::run(&mut *graph, &Some(attrs), true) })
    }

    /// Synchronous run functions with the given attribute(s) in this document.
    /// Attributes defaults to #[main] functions if null or undefined.
    /// Async TS lib functions will not work with this, but it will be faster.
    pub fn sync_run(&self, attributes: JsValue) -> Result<String, String> {
        let mut attrs = FxHashSet::default();
        let mut graph = self.graph_mut()?;
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

    /// Start a call to a function in the document (by path), as a task the host runs with `step` and then
    /// `result` (doc.ts `call`). Pass an array of arguments, or undefined for none.
    pub fn start_call(&self, path: &str, args: JsValue) -> Result<StofTask, String> {
        let mut graph = self.graph_mut()?;
        let arguments = call_arguments(args, &graph);
        Ok(StofTask { task: JsTask::call(&mut *graph, path, arguments) })
    }

    /// Synchronous call a singular function in the document (by path).
    /// If no arguments, pass undefined as args.
    /// Otherwise, pass an array of arguments as args.
    /// Async TS lib functions will not work with this, but it will be faster.
    pub fn sync_call(&self, path: &str, args: JsValue) -> Result<JsValue, String> {
        let mut arguments = vec![];
        let mut graph = self.graph_mut()?;
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


    /// Start a call to a function from Stof source without adding it to the document, as a task the host runs
    /// with `step` and then `result` (doc.ts `callSource`). The source's first function runs with self as the node
    /// (an object path or ID, the root by default), and everything the source declares is removed after the call.
    pub fn start_call_source(&self, src: &str, args: JsValue, node: JsValue) -> Result<StofTask, String> {
        let mut graph = self.graph_mut()?;
        let arguments = call_arguments(args, &graph);
        let context = context_node(node, &graph)?;
        JsTask::call_source(&mut *graph, src, context, arguments)
            .map(|task| StofTask { task })
            .map_err(|err| err.to_string())
    }

    /// Run one step of a task: 0 when it's finished, 1 for more, 2 for more after yielding to the event loop.
    /// A document that's busy (another step holds it) ends the task with that error (returns 0).
    pub fn step(&self, task: &mut StofTask) -> u8 {
        match self.graph.try_borrow_mut() {
            Ok(mut graph) => task.task.step(&mut *graph),
            Err(_) => {
                task.task.fail("the document is busy: another call holds it while the call gate is acquired".into());
                0
            },
        }
    }

    /// End a task with this error (Ex. the host's call gate failed): `result` or `report` returns it.
    pub fn fail(&self, task: &mut StofTask, message: String) {
        task.task.fail(message);
    }

    /// A finished call's result (`start_call`, `start_call_source`).
    pub fn result(&self, task: &mut StofTask) -> Result<JsValue, String> {
        let mut graph = self.graph_mut()?;
        match task.task.result(&mut *graph) {
            Ok(res) => Ok(JsValue::from(res)),
            Err(err) => Err(err.to_string()),
        }
    }

    /// A finished run's report (`start_run`).
    pub fn report(&self, task: &mut StofTask) -> Result<String, String> {
        let graph = self.graph_ref()?;
        task.task.report(&graph)
    }

    /// Synchronous call a function from Stof source without adding it to the document.
    /// Async TS lib functions will not work with this, but it will be faster.
    pub fn sync_call_source(&self, src: &str, args: JsValue, node: JsValue) -> Result<JsValue, String> {
        let mut graph = self.graph_mut()?;
        let arguments = call_arguments(args, &graph);
        let context = context_node(node, &graph)?;
        match Runtime::call_source(&mut *graph, src, context, arguments) {
            Ok(res) => Ok(JsValue::from(res)),
            Err(err) => Err(err.to_string())
        }
    }

    /// Remove a field, function, or object by path (Ex. "config.staging"), like drop() in Stof.
    /// A field that holds an object drops the object too, unless shallow (then only the field is removed).
    /// Returns whether anything was removed.
    pub fn remove(&self, path: &str, shallow: bool) -> Result<bool, String> {
        Ok(self.graph_mut()?.remove(path, shallow))
    }


    /*****************************************************************************
     * Interop.
     *****************************************************************************/
    
    /// Insert a JS function as a library function, available in Stof.
    pub fn js_library_function(&self, func: StofFunc) -> Result<(), String> {
        let mut graph = self.graph_mut()?;
        graph.insert_libfunc(func.get_func());
        Ok(())
    }

    #[wasm_bindgen(js_name = allowHttp)]
    /// Give this document network access (the Http library, using fetch). Off by default.
    pub fn allow_http(&self) -> Result<(), String> {
        self.graph_mut()?.allow_http();
        Ok(())
    }

    #[wasm_bindgen(js_name = allowHtmlTags)]
    /// Elements the html format may write (Ex. the components your page renders): only these, instead of the
    /// standard HTML elements and custom elements. Documents can't change it. Calling it again adds elements.
    pub fn allow_html_tags(&self, tags: Vec<String>) -> Result<(), String> {
        self.graph_mut()?.allow_html_tags(tags);
        Ok(())
    }

    /// Seal an object (path, or a type as "<Name>") and everything under it: its functions can be called but not
    /// changed (see `model::seal`). With `fields`, its fields are frozen too. With `expected`, only if its digest is
    /// that. With `public_key` and `signature`, only if the digest is signed by that key. Returns the digest.
    /// With `removable`, it can still be removed whole. With `reads` (paths, looked up each time the part is entered),
    /// its code reaches only itself and those objects; with `hosts`, its Http.fetch reaches only those hosts; with
    /// `libs`, it calls only those libraries (host libraries included); with `time` (ms), it runs for at most that
    /// long each time it's entered; with `calls` (function paths), those functions are its gates: it may call them,
    /// and they run unconfined (see `model::seal::Confine`).
    pub fn seal(&self, path: &str, fields: Option<bool>, expected: Option<String>, public_key: Option<String>, signature: Option<String>, removable: Option<bool>, reads: Option<Vec<String>>, hosts: Option<Vec<String>>, libs: Option<Vec<String>>, time: Option<f64>, calls: Option<Vec<String>>) -> Result<String, String> {
        let mut graph = self.graph_mut()?;
        let calls = match calls {
            Some(paths) => Some(graph.seal_calls(&paths).map_err(|error| error.to_string())?),
            None => None,
        };
        let time = match time {
            Some(ms) if !(ms > 0.0) || !ms.is_finite() => return Err("sealed: seal time must be more than 0 ms".into()),
            Some(ms) => Some(web_time::Duration::from_micros((ms * 1000.0) as u64)),
            None => None,
        };
        let options = SealOptions { fields: fields.unwrap_or(false), removable: removable.unwrap_or(false), reads, hosts, libs, time, calls };
        let res = match (public_key, signature) {
            (Some(key), Some(signature)) => {
                if let Some(expected) = &expected {
                    let digest = graph.digest_path(path, options.fields).map_err(|e| e.to_string())?;
                    if !digest.eq_ignore_ascii_case(expected.trim()) {
                        return Err(format!("sealed: {path} isn't what was expected: its digest is {digest}, not {expected}"));
                    }
                }
                graph.seal_signed_with(path, &options, &key, &signature)
            },
            (None, None) => graph.seal_path_with(path, &options, expected.as_deref()),
            _ => return Err("a signature needs both a public key and a signature".into()),
        };
        res.map_err(|error| error.to_string())
    }

    /// How an object (a path, or a type as "<Name>") is sealed, as JSON: { part, fields, removable, confined:
    /// [{ reads, hosts, libs, time, calls }] }, or null when it isn't sealed.
    pub fn sealed(&self, path: &str) -> Result<Option<String>, String> {
        let graph = self.graph_ref()?;
        let info = graph.seal_info_path(path).map_err(|error| error.to_string())?;
        Ok(info.map(|info| info.to_json(&graph).to_string()))
    }

    /// Unseal an object that `seal` sealed (path, or a type as "<Name>").
    pub fn unseal(&self, path: &str) -> Result<(), String> {
        self.graph_mut()?.unseal_path(path).map_err(|error| error.to_string())
    }

    /// The seal digest of an object (path, or a type as "<Name>"), with its fields' values when `fields`.
    pub fn digest(&self, path: &str, fields: Option<bool>) -> Result<String, String> {
        self.graph_ref()?.digest_path(path, fields.unwrap_or(false)).map_err(|error| error.to_string())
    }

    #[wasm_bindgen(js_name = allowHtmlAttributes)]
    /// Attributes the html format writes only when allowed: `style`, `action`, and `formaction` (off by default).
    /// Documents can't change it. Calling it again adds attributes.
    pub fn allow_html_attributes(&self, attributes: Vec<String>) -> Result<(), String> {
        self.graph_mut()?.allow_html_attributes(attributes);
        Ok(())
    }

    #[wasm_bindgen(js_name = keepSource)]
    /// Keep each function's source text from now on (off by default), so the readable `stof` export writes
    /// functions as Stof. It costs the memory of the text.
    pub fn keep_source(&self, keep: Option<bool>) -> Result<(), String> {
        self.graph_mut()?.keep_source(keep.unwrap_or(true));
        Ok(())
    }

    /// Print nothing to the console (off by default): no parse warnings, test progress, or pln/err/dbg output.
    /// Results and errors still come back (Ex. `test` still returns its report).
    pub fn quiet(&self, quiet: Option<bool>) -> Result<(), String> {
        self.graph_mut()?.quiet(quiet.unwrap_or(true));
        Ok(())
    }

    #[wasm_bindgen(js_name = setMaxExecutionTime)]
    /// How long a call can run, in milliseconds, before it fails with an execution timeout (null for no limit).
    /// Defaults to 120,000 (2 minutes).
    pub fn set_max_execution_time(&self, ms: Option<f64>) -> Result<(), String> {
        self.graph_mut()?.max_execution_time = ms.map(|ms| std::time::Duration::from_millis(ms.max(0.) as u64));
        Ok(())
    }

    #[wasm_bindgen(js_name = setMaxMemory)]
    /// How much memory a call can use, in bytes, before it fails with a memory limit error (null for no limit).
    /// Defaults to 268,435,456 (256 MB).
    pub fn set_max_memory(&self, bytes: Option<f64>) -> Result<(), String> {
        self.graph_mut()?.max_memory = bytes.map(|bytes| bytes.max(0.) as usize);
        Ok(())
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
            let mut graph = self.graph_mut()?;
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
        let mut graph = self.graph_mut()?;
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
            Val::Str(path) => {
                // a path to an object (Ex. 'plugins.weather'): an error when there's none, never a quiet no-op
                match graph.host_object(&path) {
                    Some(node) => parse_node = node,
                    None => return Err(format!("there's no object at '{path}' to parse into")),
                }
            },
            _ => {
                return Ok(false);
            }
        }

        let profile = match profile {
            "prod" => Profile::prod(),
            "test" => Profile::test(),
            "prod_docs" => Profile::docs(false),
            "untrusted" => Profile::untrusted(),
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
        let mut graph = self.graph_mut()?;
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
            Val::Str(path) => {
                // a path to an object (Ex. 'plugins.weather'): an error when there's none, never a quiet no-op
                match graph.host_object(&path) {
                    Some(node) => parse_node = node,
                    None => return Err(format!("there's no object at '{path}' to parse into")),
                }
            },
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
            "untrusted" => Profile::untrusted(),
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
        let graph = self.graph_ref()?;
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
            // a path, like get, set, and call take (Ex. "config.production")
            Val::Str(path) => {
                match graph.host_node(&path) {
                    Some(node) => exp_node = node,
                    None => return Err(format!("export node not found")),
                }
            },
            _ => {
                return Err(format!("export node not found"));
            }
        }
        drop(graph);
        // computed values (Ex. #[attr] functions in html) are called first, when the document isn't busy
        let res = match self.graph.try_borrow_mut() {
            Ok(mut graph) => graph.stringify(format, Some(exp_node)),
            Err(_) => self.graph_ref()?.string_export(format, Some(exp_node)),
        };
        match res {
            Ok(val) => Ok(val),
            Err(err) => Err(err.to_string())
        }
    }

    #[wasm_bindgen(js_name = binaryExport)]
    /// Binary export (Uint8Array), using a format of choice.
    /// Format can also be a content type (for HTTP-like situations).
    pub fn binary_export(&self, format: &str, node: JsValue) -> Result<JsValue, String> {
        let graph = self.graph_ref()?;
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
            // a path, like get, set, and call take (Ex. "config.production")
            Val::Str(path) => {
                match graph.host_node(&path) {
                    Some(node) => exp_node = node,
                    None => return Err(format!("export node not found")),
                }
            },
            _ => {
                return Err(format!("export node not found"));
            }
        }
        drop(graph);
        // computed values (Ex. #[attr] functions in html) are called first, when the document isn't busy
        let res = match self.graph.try_borrow_mut() {
            Ok(mut graph) => graph.blobify(format, Some(exp_node)),
            Err(_) => self.graph_ref()?.binary_export(format, Some(exp_node)),
        };
        match res {
            Ok(bytes) => Ok(JsValue::from(Uint8Array::from(bytes.as_ref()))),
            Err(err) => Err(err.to_string())
        }
    }
}


/// Call arguments from JS: an array of arguments, a single value, or undefined for none.
fn call_arguments(args: JsValue, graph: &Graph) -> Vec<Val> {
    match to_graph_value(args, graph) {
        Val::List(vals) => vals.into_iter().map(|val| val.read().clone()).collect(),
        Val::Void => vec![],
        val => vec![val],
    }
}

/// The object a call runs on: an object path or ID, or null/undefined for the main root.
fn context_node(node: JsValue, graph: &Graph) -> Result<Option<NodeRef>, String> {
    match to_graph_value(node, graph) {
        Val::Null | Val::Void => Ok(None),
        Val::Obj(node) if node.node_exists(graph) => Ok(Some(node)),
        Val::Str(path) => match graph.host_node(&path) {
            Some(node) => Ok(Some(node)),
            None => Err(format!("object '{path}' not found")),
        },
        _ => Err(format!("node must be an object path or ID")),
    }
}
