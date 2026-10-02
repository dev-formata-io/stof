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

use std::{cell::RefCell, collections::BTreeMap, sync::Arc};
use imbl::vector;
use js_sys::{Function, Promise};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use crate::{js::value::{to_graph_value, to_raw_value}, model::{Graph, LibFunc, Param, stof_std::THROW}, runtime::{Error, Type, Val, ValRef, Variable, WakeRef, instruction::{Instruction, Instructions}, instructions::Base, proc::ProcEnv, wake}};


thread_local! {
    static JS_FUNCTIONS: RefCell<BTreeMap<String, BTreeMap<String, BTreeMap<String, Function>>>> = RefCell::new(BTreeMap::default());
}


#[wasm_bindgen]
/// JS Library Function.
pub struct StofFunc {
    docid: String,
    func: LibFunc,
}
impl StofFunc {
    pub fn get_func(self) -> LibFunc {
        self.func
    }

    fn set_js_func(docid: &str, lib: &str, name: &str, func: Function) {
        JS_FUNCTIONS.with_borrow_mut(|map| {
            if let Some(doc) = map.get_mut(docid) {
                if let Some(lib) = doc.get_mut(lib) {
                    lib.insert(name.into(), func);
                } else {
                    let mut funcs = BTreeMap::new();
                    funcs.insert(name.into(), func);
                    doc.insert(lib.into(), funcs);
                }
            } else {
                let mut funcs = BTreeMap::new();
                funcs.insert(name.into(), func);
                let mut libs = BTreeMap::new();
                libs.insert(lib.into(), funcs);
                map.insert(docid.into(), libs);
            }
        });
    }
}
#[wasm_bindgen]
impl StofFunc {
    #[wasm_bindgen(constructor)]
    /// Create a new Stof function from a JS function.
    pub fn new(docid: &str, library: &str, name: &str, js_function: JsValue, is_async: bool) -> Self {
        let js_function = Function::from(js_function);
        Self::set_js_func(docid, library, name, js_function);

        let lib = library.to_string();
        let nm = name.to_string();
        let did = docid.to_string();
        let func = LibFunc {
            library: library.into(),
            name: name.into(),
            is_async,
            docs: String::default(),
            params: vector![],
            unbounded_args: true,
            return_type: None,
            args_to_symbol_table: false,
            func: Arc::new(move |_as_ref, arg_count, _env, _graph| {
                let mut instructions = Instructions::default();
                instructions.push(Arc::new(JsLibFuncIns::Call(did.clone(), arg_count, lib.clone(), nm.clone())));
                Ok(instructions)
            }),
        };

        Self { docid: docid.into(), func }
    }

    /// Doc id for this function.
    pub fn docid(&self) -> String {
        self.docid.clone()
    }

    #[wasm_bindgen(js_name = setParams)]
    /// Name this function's parameters, so Stof can call it with named arguments (Ex. `Http.fetch(url, bearer = 'x')`).
    /// Parameters are optional: anything not passed arrives as null.
    pub fn set_params(&mut self, names: Vec<String>) {
        self.func.params = names.into_iter()
            .map(|name| Param { name: name.into(), param_type: Type::Void, default: Some(Arc::new(Base::Literal(Val::Null))) })
            .collect();
    }
}

/// A JS error as a Stof value: its message when it's an Error or a string.
fn js_error_value(error: &JsValue) -> Val {
    if let Some(message) = error.as_string() {
        return Val::Str(message.into());
    }
    if error.is_instance_of::<js_sys::Error>() {
        let error = js_sys::Error::from(error.clone());
        return Val::Str(String::from(error.message()).into());
    }
    to_raw_value(error.clone())
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// JS Library Function Instructions.
enum JsLibFuncIns {
    Call(String, usize, String, String),
    /// After an async call: throw the result if the JS promise was rejected.
    ThrowIfRejected(ValRef<Val>),
}
#[typetag::serde(name = "JsLibFuncIns")]
impl Instruction for JsLibFuncIns {
    fn exec(&self, env: &mut ProcEnv, graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        match self {
            Self::Call(docid, arg_count, library, name) => {
                let context = JsValue::from(Val::Obj(env.self_ptr()));
                let res = JS_FUNCTIONS.with_borrow(|map| {
                    if let Some(doc) = map.get(docid) {
                        if let Some(lib) = doc.get(library) {
                            if let Some(js_func) = lib.get(name) {
                                if env.stack.len() < *arg_count {
                                    Err(JsValue::from_str(&format!("{library}.{name}: missing arguments")))
                                } else {
                                    // arguments are on the stack in order (last on top)
                                    let mut args: Vec<JsValue> = Vec::with_capacity(*arg_count);
                                    for _ in 0..*arg_count {
                                        if let Some(var) = env.stack.pop() { args.push(var.val.read().clone().into()); }
                                    }
                                    args.reverse();
                                    match args.len() {
                                        0 => js_func.call0(&context),
                                        1 => js_func.call1(&context, &args[0]),
                                        2 => js_func.call2(&context, &args[0], &args[1]),
                                        3 => js_func.call3(&context, &args[0], &args[1], &args[2]),
                                        _ => {
                                            // any number of arguments (was capped at 9)
                                            let array = js_sys::Array::new();
                                            for arg in &args { array.push(arg); }
                                            js_func.apply(&context, &array)
                                        },
                                    }
                                }
                            } else {
                                Err(JsValue::from_str(&format!("JS/Stof Function not found: {library}.{name}")))
                            }
                        } else {
                            Err(JsValue::from_str(&format!("JS/Stof Function not found: {library}.{name}")))
                        }
                    } else {
                        Err(JsValue::from_str(&format!("JS/Stof Function not found: {library}.{name}")))
                    }
                });
                match res {
                    Ok(result) => {
                        // if the result is a promise, do the async things
                        if result.is_instance_of::<Promise>() {
                            let promise = Promise::from(result);
                            
                            let wake_ref = WakeRef::default(); // when to return
                            let placeholder = ValRef::new(Val::Null); // the return value
                            let rejected = ValRef::new(Val::Bool(false)); // did the promise reject?

                            // start the JS promise in the background
                            let wake_clone = wake_ref.clone();
                            let ret_val = placeholder.clone();
                            let rejected_flag = rejected.clone();
                            wasm_bindgen_futures::spawn_local(async move {
                                match JsFuture::from(promise).await {
                                    Ok(result) => {
                                        let mut ret = ret_val.write();
                                        *ret = to_raw_value(result);
                                    },
                                    Err(error) => {
                                        *ret_val.write() = js_error_value(&error);
                                        *rejected_flag.write() = Val::Bool(true);
                                    }
                                }
                                //web_sys::console::log_1(&"calling wake".into());
                                wake(&wake_clone); // wake the Stof process after complete
                            });

                            let mut instructions = Instructions::default();
                            instructions.push(Arc::new(Base::Variable(Variable::refval(placeholder.clone()))));
                            instructions.push(Arc::new(Base::CtrlSleepRef(wake_ref)));
                            instructions.push(Arc::new(JsLibFuncIns::ThrowIfRejected(rejected)));
                            return Ok(Some(instructions));
                        } else {
                            env.stack.push(Variable::val(to_graph_value(result, &graph)));
                        }
                    },
                    Err(error) => {
                        let mut instructions = Instructions::default();
                        instructions.push(Arc::new(Base::Literal(js_error_value(&error))));
                        instructions.push(THROW.clone());
                        return Ok(Some(instructions));
                    }
                }
            },
            Self::ThrowIfRejected(rejected) => {
                if rejected.read().truthy() {
                    if let Some(error) = env.stack.pop() {
                        let mut instructions = Instructions::default();
                        instructions.push(Arc::new(Base::Literal(error.val.read().clone())));
                        instructions.push(THROW.clone());
                        return Ok(Some(instructions));
                    }
                }
            },
        }
        Ok(None)
    }
}
