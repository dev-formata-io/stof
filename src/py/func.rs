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

use std::sync::Arc;
use pyo3::{Py, PyAny, Python, exceptions::PyValueError, types::PyTuple};
use serde::{Deserialize, Serialize};
use crate::{model::{Graph, LibFunc, stof_std::THROW}, py::value::{py_any_to_val, val_to_py}, runtime::{Val, Variable, instruction::{Instruction, Instructions}, instructions::Base, proc::ProcEnv}};


/// Create and insert a python function as a Stof library function.
///
/// The Python callable lives in this document's own library table (inside the LibFunc), so each document keeps
/// its own functions (two documents can both register `App.handler`), and any thread can call the document.
/// Unlike JS functions, Python callables are Send + Sync, so no side table keyed by document or thread is needed.
pub fn set_py_lib_func(graph: &mut Graph, lib: &str, name: &str, func: Py<PyAny>, is_async: bool) {
    let callable = Arc::new(func);
    let clib = lib.to_string();
    let nm = name.to_string();
    let func = LibFunc {
        library: lib.into(),
        name: name.into(),
        is_async,
        docs: String::default(),
        params: crate::params![],
        unbounded_args: true,
        return_type: None,
        args_to_symbol_table: false,
        func: Arc::new(move |_as_ref, arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(PyLibFuncIns::Call { arg_count, library: clib.clone(), name: nm.clone(), func: Some(callable.clone()) }));
            Ok(instructions)
        }),
    };
    graph.insert_libfunc(func);
}


#[derive(Debug, Clone, Serialize, Deserialize)]
enum PyLibFuncIns {
    Call {
        arg_count: usize,
        library: String,
        name: String,
        /// The Python callable (runtime only: a Python function can't be serialized).
        #[serde(skip)]
        func: Option<Arc<Py<PyAny>>>,
    },
}
#[typetag::serde(name = "PyLibFuncIns")]
impl Instruction for PyLibFuncIns {
    fn exec(&self, env: &mut ProcEnv, graph: &mut Graph) -> Result<Option<Instructions>, crate::runtime::Error> {
        match self {
            Self::Call { arg_count, library, name, func } => {
                let res = Python::attach(|py| {
                    let Some(py_func) = func else {
                        return Err(PyValueError::new_err(format!("Python/Stof Function not found: {library}.{name}")));
                    };
                    // the arguments are on the stack in order (the last one on top): any count
                    if env.stack.len() < *arg_count {
                        return Err(PyValueError::new_err(format!("{library}.{name}: expected {arg_count} arguments on the stack")));
                    }
                    let args = env.stack.split_off(env.stack.len() - *arg_count);
                    let args: Vec<_> = args.into_iter().map(|arg| val_to_py(py, arg.val.read().clone())).collect();
                    match PyTuple::new(py, args) {
                        Ok(args) => py_func.call1(py, args),
                        Err(error) => Err(error),
                    }
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
