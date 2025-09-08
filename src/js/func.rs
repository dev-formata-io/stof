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

use std::{cell::RefCell, collections::BTreeMap, sync::Arc};
use imbl::vector;
use js_sys::Function;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use crate::{js::value::to_graph_value, model::{stof_std::THROW, Graph, LibFunc}, runtime::{instruction::{Instruction, Instructions}, instructions::Base, proc::ProcEnv, Error, Val, Variable}};


thread_local! {
    static JS_FUNCTIONS: RefCell<BTreeMap<String, BTreeMap<String, Function>>> = RefCell::new(BTreeMap::default());
}


#[wasm_bindgen]
/// JS Library Function.
pub struct StofFunc {
    func: LibFunc,
}
impl StofFunc {
    pub fn get_func(self) -> LibFunc {
        self.func
    }

    fn set_js_func(lib: &str, name: &str, func: Function) {
        JS_FUNCTIONS.with_borrow_mut(|map| {
            if let Some(lib) = map.get_mut(lib) {
                lib.insert(name.into(), func);
            } else {
                let mut inner = BTreeMap::new();
                inner.insert(name.into(), func);
                map.insert(lib.into(), inner);
            }
        });
    }
}
#[wasm_bindgen]
impl StofFunc {
    #[wasm_bindgen(constructor)]
    /// Create a new Stof function from a JS function.
    pub fn new(library: &str, name: &str, js_function: JsValue) -> Self {
        let js_function = Function::from(js_function);
        Self::set_js_func(library, name, js_function);

        let lib = library.to_string();
        let nm = name.to_string();
        let func = LibFunc {
            library: library.into(),
            name: name.into(),
            is_async: false,
            docs: String::default(),
            params: vector![],
            unbounded_args: true,
            return_type: None,
            args_to_symbol_table: false,
            func: Arc::new(move |_as_ref, arg_count, _env, _graph| {
                let mut instructions = Instructions::default();
                instructions.push(Arc::new(JsLibFuncIns::Call(arg_count, lib.clone(), nm.clone())));
                Ok(instructions)
            }),
        };

        Self { func }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// JS Library Function Instructions.
enum JsLibFuncIns {
    Call(usize, String, String),
}
#[typetag::serde(name = "JsLibFuncIns")]
impl Instruction for JsLibFuncIns {
    fn exec(&self, env: &mut ProcEnv, graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        match self {
            Self::Call(arg_count, library, name) => {
                let context = JsValue::from(Val::Obj(env.self_ptr()));
                let res = JS_FUNCTIONS.with_borrow(|map| {
                    if let Some(lib) = map.get(library) {
                        if let Some(js_func) = lib.get(name) {
                            match arg_count {
                                0 => {
                                    js_func.call0(&context)
                                },
                                1 => {
                                    js_func.call1(&context, &env.stack.pop().unwrap().val.read().clone().into())
                                },
                                2 => {
                                    let one = env.stack.pop().unwrap().val.read().clone();
                                    let zer = env.stack.pop().unwrap().val.read().clone();
                                    js_func.call2(&context, &zer.into(), &one.into())
                                },
                                3 => {
                                    let two = env.stack.pop().unwrap().val.read().clone();
                                    let one = env.stack.pop().unwrap().val.read().clone();
                                    let zer = env.stack.pop().unwrap().val.read().clone();
                                    js_func.call3(&context, &zer.into(), &one.into(), &two.into())
                                },
                                4 => {
                                    let thr = env.stack.pop().unwrap().val.read().clone();
                                    let two = env.stack.pop().unwrap().val.read().clone();
                                    let one = env.stack.pop().unwrap().val.read().clone();
                                    let zer = env.stack.pop().unwrap().val.read().clone();
                                    js_func.call4(&context, &zer.into(), &one.into(), &two.into(), &thr.into())
                                },
                                5 => {
                                    let foy = env.stack.pop().unwrap().val.read().clone();
                                    let thr = env.stack.pop().unwrap().val.read().clone();
                                    let two = env.stack.pop().unwrap().val.read().clone();
                                    let one = env.stack.pop().unwrap().val.read().clone();
                                    let zer = env.stack.pop().unwrap().val.read().clone();
                                    js_func.call5(&context, &zer.into(), &one.into(), &two.into(), &thr.into(), &foy.into())
                                },
                                6 => {
                                    let six = env.stack.pop().unwrap().val.read().clone();
                                    let foy = env.stack.pop().unwrap().val.read().clone();
                                    let thr = env.stack.pop().unwrap().val.read().clone();
                                    let two = env.stack.pop().unwrap().val.read().clone();
                                    let one = env.stack.pop().unwrap().val.read().clone();
                                    let zer = env.stack.pop().unwrap().val.read().clone();
                                    js_func.call6(&context, &zer.into(), &one.into(), &two.into(), &thr.into(), &foy.into(), &six.into())
                                },
                                7 => {
                                    let sev = env.stack.pop().unwrap().val.read().clone();
                                    let six = env.stack.pop().unwrap().val.read().clone();
                                    let foy = env.stack.pop().unwrap().val.read().clone();
                                    let thr = env.stack.pop().unwrap().val.read().clone();
                                    let two = env.stack.pop().unwrap().val.read().clone();
                                    let one = env.stack.pop().unwrap().val.read().clone();
                                    let zer = env.stack.pop().unwrap().val.read().clone();
                                    js_func.call7(&context, &zer.into(), &one.into(), &two.into(), &thr.into(), &foy.into(), &six.into(), &sev.into())
                                },
                                8 => {
                                    let eig = env.stack.pop().unwrap().val.read().clone();
                                    let sev = env.stack.pop().unwrap().val.read().clone();
                                    let six = env.stack.pop().unwrap().val.read().clone();
                                    let foy = env.stack.pop().unwrap().val.read().clone();
                                    let thr = env.stack.pop().unwrap().val.read().clone();
                                    let two = env.stack.pop().unwrap().val.read().clone();
                                    let one = env.stack.pop().unwrap().val.read().clone();
                                    let zer = env.stack.pop().unwrap().val.read().clone();
                                    js_func.call8(&context, &zer.into(), &one.into(), &two.into(), &thr.into(), &foy.into(), &six.into(), &sev.into(), &eig.into())
                                },
                                9 => {
                                    let nin = env.stack.pop().unwrap().val.read().clone();
                                    let eig = env.stack.pop().unwrap().val.read().clone();
                                    let sev = env.stack.pop().unwrap().val.read().clone();
                                    let six = env.stack.pop().unwrap().val.read().clone();
                                    let foy = env.stack.pop().unwrap().val.read().clone();
                                    let thr = env.stack.pop().unwrap().val.read().clone();
                                    let two = env.stack.pop().unwrap().val.read().clone();
                                    let one = env.stack.pop().unwrap().val.read().clone();
                                    let zer = env.stack.pop().unwrap().val.read().clone();
                                    js_func.call9(&context, &zer.into(), &one.into(), &two.into(), &thr.into(), &foy.into(), &six.into(), &sev.into(), &eig.into(), &nin.into())
                                },
                                _ => {
                                    Err(JsValue::from_str("outnumbered allotted argument count for JS/Stof interop"))
                                }
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
                        env.stack.push(Variable::val(to_graph_value(result, &graph)));
                    },
                    Err(error) => {
                        let mut instructions = Instructions::default();
                        instructions.push(Arc::new(Base::Literal(to_graph_value(error, &graph))));
                        instructions.push(THROW.clone());
                        return Ok(Some(instructions));
                    }
                }
            },
        }
        Ok(None)
    }
}
