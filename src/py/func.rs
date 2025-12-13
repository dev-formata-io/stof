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
use pyo3::{Py, PyAny, Python, exceptions::PyValueError};
use serde::{Deserialize, Serialize};
use crate::{model::{Graph, LibFunc, stof_std::THROW}, py::value::{py_any_to_val, val_to_py}, runtime::{Val, Variable, instruction::{Instruction, Instructions}, instructions::Base, proc::ProcEnv}};


thread_local! {
    static PY_FUNCTIONS: RefCell<BTreeMap<String, BTreeMap<String, Py<PyAny>>>> = RefCell::new(BTreeMap::default());
}


/// Create and insert a python function as a Stof library function.
pub fn set_py_lib_func(graph: &mut Graph, lib: &str, name: &str, func: Py<PyAny>, is_async: bool) {
    PY_FUNCTIONS.with_borrow_mut(|map| {
        if let Some(lib) = map.get_mut(lib) {
            lib.insert(name.into(), func);
        } else {
            let mut inner = BTreeMap::new();
            inner.insert(name.into(), func);
            map.insert(lib.into(), inner);
        }
    });

    let clib = lib.to_string();
    let nm = name.to_string();
    let func = LibFunc {
        library: lib.into(),
        name: name.into(),
        is_async,
        docs: String::default(),
        params: vector![],
        unbounded_args: true,
        return_type: None,
        args_to_symbol_table: false,
        func: Arc::new(move |_as_ref, arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(PyLibFuncIns::Call(arg_count, clib.clone(), nm.clone())));
            Ok(instructions)
        }),
    };
    graph.insert_libfunc(func);
}


#[derive(Debug, Clone, Serialize, Deserialize)]
enum PyLibFuncIns {
    Call(usize, String, String),
}
#[typetag::serde(name = "PyLibFuncIns")]
impl Instruction for PyLibFuncIns {
    fn exec(&self, env: &mut ProcEnv, graph: &mut Graph) -> Result<Option<Instructions>, crate::runtime::Error> {
        match self {
            Self::Call(arg_count, library, name) => {
                let res = PY_FUNCTIONS.with_borrow(|map| {
                    Python::attach(|py| {
                        if let Some(lib) = map.get(library) {
                            if let Some(py_func) = lib.get(name) {
                                match arg_count {
                                    0 => {
                                        py_func.call0(py)
                                    },
                                    1 => {
                                        let arg = val_to_py(py, env.stack.pop().unwrap().val.read().clone());
                                        py_func.call1(py, (arg,))
                                    },
                                    2 => {
                                        let one = val_to_py(py, env.stack.pop().unwrap().val.read().clone());
                                        let zer = val_to_py(py, env.stack.pop().unwrap().val.read().clone());
                                        py_func.call1(py, (zer, one))
                                    },
                                    3 => {
                                        let two = val_to_py(py, env.stack.pop().unwrap().val.read().clone());
                                        let one = val_to_py(py, env.stack.pop().unwrap().val.read().clone());
                                        let zer = val_to_py(py, env.stack.pop().unwrap().val.read().clone());
                                        py_func.call1(py, (zer, one, two))
                                    },
                                    4 => {
                                        let thr = val_to_py(py, env.stack.pop().unwrap().val.read().clone());
                                        let two = val_to_py(py, env.stack.pop().unwrap().val.read().clone());
                                        let one = val_to_py(py, env.stack.pop().unwrap().val.read().clone());
                                        let zer = val_to_py(py, env.stack.pop().unwrap().val.read().clone());
                                        py_func.call1(py, (zer, one, two, thr))
                                    },
                                    5 => {
                                        let foy = val_to_py(py, env.stack.pop().unwrap().val.read().clone());
                                        let thr = val_to_py(py, env.stack.pop().unwrap().val.read().clone());
                                        let two = val_to_py(py, env.stack.pop().unwrap().val.read().clone());
                                        let one = val_to_py(py, env.stack.pop().unwrap().val.read().clone());
                                        let zer = val_to_py(py, env.stack.pop().unwrap().val.read().clone());
                                        py_func.call1(py, (zer, one, two, thr, foy))
                                    },
                                    6 => {
                                        let six = val_to_py(py, env.stack.pop().unwrap().val.read().clone());
                                        let foy = val_to_py(py, env.stack.pop().unwrap().val.read().clone());
                                        let thr = val_to_py(py, env.stack.pop().unwrap().val.read().clone());
                                        let two = val_to_py(py, env.stack.pop().unwrap().val.read().clone());
                                        let one = val_to_py(py, env.stack.pop().unwrap().val.read().clone());
                                        let zer = val_to_py(py, env.stack.pop().unwrap().val.read().clone());
                                        py_func.call1(py, (zer, one, two, thr, foy, six))
                                    },
                                    7 => {
                                        let sev = val_to_py(py, env.stack.pop().unwrap().val.read().clone());
                                        let six = val_to_py(py, env.stack.pop().unwrap().val.read().clone());
                                        let foy = val_to_py(py, env.stack.pop().unwrap().val.read().clone());
                                        let thr = val_to_py(py, env.stack.pop().unwrap().val.read().clone());
                                        let two = val_to_py(py, env.stack.pop().unwrap().val.read().clone());
                                        let one = val_to_py(py, env.stack.pop().unwrap().val.read().clone());
                                        let zer = val_to_py(py, env.stack.pop().unwrap().val.read().clone());
                                        py_func.call1(py, (zer, one, two, thr, foy, six, sev))
                                    },
                                    8 => {
                                        let eig = val_to_py(py, env.stack.pop().unwrap().val.read().clone());
                                        let sev = val_to_py(py, env.stack.pop().unwrap().val.read().clone());
                                        let six = val_to_py(py, env.stack.pop().unwrap().val.read().clone());
                                        let foy = val_to_py(py, env.stack.pop().unwrap().val.read().clone());
                                        let thr = val_to_py(py, env.stack.pop().unwrap().val.read().clone());
                                        let two = val_to_py(py, env.stack.pop().unwrap().val.read().clone());
                                        let one = val_to_py(py, env.stack.pop().unwrap().val.read().clone());
                                        let zer = val_to_py(py, env.stack.pop().unwrap().val.read().clone());
                                        py_func.call1(py, (zer, one, two, thr, foy, six, sev, eig))
                                    },
                                    9 => {
                                        let nin = val_to_py(py, env.stack.pop().unwrap().val.read().clone());
                                        let eig = val_to_py(py, env.stack.pop().unwrap().val.read().clone());
                                        let sev = val_to_py(py, env.stack.pop().unwrap().val.read().clone());
                                        let six = val_to_py(py, env.stack.pop().unwrap().val.read().clone());
                                        let foy = val_to_py(py, env.stack.pop().unwrap().val.read().clone());
                                        let thr = val_to_py(py, env.stack.pop().unwrap().val.read().clone());
                                        let two = val_to_py(py, env.stack.pop().unwrap().val.read().clone());
                                        let one = val_to_py(py, env.stack.pop().unwrap().val.read().clone());
                                        let zer = val_to_py(py, env.stack.pop().unwrap().val.read().clone());
                                        py_func.call1(py, (zer, one, two, thr, foy, six, sev, eig, nin))
                                    },
                                    _ => {
                                        Err(PyValueError::new_err(format!("outnumbered allotted argument count for Python/Stof interop")))
                                    }
                                }
                            } else {
                                Err(PyValueError::new_err(format!("Python/Stof Function not found: {library}.{name}")))
                            }
                        } else {
                            Err(PyValueError::new_err(format!("JS/Stof Function not found: {library}.{name}")))
                        }
                    })
                });
                match res {
                    Ok(result) => {
                        let val = Python::attach(|py| {
                            py_any_to_val(result.bind(py), &graph)
                        });
                        env.stack.push(Variable::val(val));
                        Ok(None)
                    },
                    Err(error) => {
                        let mut instructions = Instructions::default();
                        instructions.push(Arc::new(Base::Literal(Val::Str(error.to_string().into()))));
                        instructions.push(THROW.clone());
                        return Ok(Some(instructions));
                    }
                }
            },
        }
    }
}
