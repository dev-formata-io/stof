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

pub mod value;
pub mod func;
use std::sync::Arc;
use std::ops::Deref;
use pyo3::{exceptions::PyValueError, prelude::*};
use rustc_hash::FxHashSet;
use crate::{model::{Graph, Profile}, py::{func::set_py_lib_func, value::{py_any_to_val, val_to_py}}, runtime::{Runtime, Val, Variable, instruction::Instruction, instructions::Base, proc::ProcEnv}};


#[pyclass]
pub struct Doc {
    graph: Graph,
}


#[pymethods]
impl Doc {
    #[new]
    /// Create a new document.
    fn new() -> Self {
        Self { graph: Graph::default() }
    }

    #[pyo3(signature = (path, start = None))]
    /// Get a value from this document by path with an optional starting object (string obj id).
    pub fn get<'py>(&mut self, py: Python<'py>, path: &str, start: Option<&Bound<'py, PyAny>>) -> PyResult<Bound<'py, PyAny>> {
        let mut proc_env = ProcEnv::default();
        if let Some(main) = self.graph.main_root() {
            proc_env.self_stack.push(main);
        }
        if let Some(Val::Obj(start)) = start.map(|start| py_any_to_val(start, &self.graph)) {
            proc_env.self_stack.push(start);
        }
        let path = self.graph.host_path(path, proc_env.self_stack.last());
        let instruction: Arc<dyn Instruction> = Arc::new(Base::LoadVariable(path.into(), false, false));
        let _ = instruction.exec(&mut proc_env, &mut self.graph); // don't care about res
        if let Some(var) = proc_env.stack.pop() {
            Ok(val_to_py(py, var.val.read().clone()))
        } else {
            Ok(val_to_py(py, Val::Null))
        }
    }

    #[pyo3(signature = (path, value, start = None))]
    /// Set a value in this document by path with an optional starting object (string obj id).
    pub fn set<'py>(&mut self, path: &str, value: &Bound<'py, PyAny>, start: Option<&Bound<'py, PyAny>>) -> PyResult<bool> {
        let mut proc_env = ProcEnv::default();
        if let Some(main) = self.graph.main_root() {
            proc_env.self_stack.push(main);
        }
        if let Some(Val::Obj(start)) = start.map(|start| py_any_to_val(start, &self.graph)) {
            proc_env.self_stack.push(start);
        }
        proc_env.stack.push(Variable::val(py_any_to_val(value, &self.graph)));
        let path = self.graph.host_path(path, proc_env.self_stack.last());
        let instruction: Arc<dyn Instruction> = Arc::new(Base::SetVariable(path.into()));
        match instruction.exec(&mut proc_env, &mut self.graph) {
            Ok(_res) => Ok(true),
            Err(_err) => Ok(false)
        }
    }


    /*****************************************************************************
     * Runtime.
     *****************************************************************************/
    
    #[pyo3(signature = (attributes = None))]
    /// Run functions with the given attribute(s).
    /// Attribute defaults to #[main] if None.
    pub fn run(&mut self, attributes: Option<&Bound<'_, PyAny>>) -> PyResult<String> {
        let mut attrs = FxHashSet::default();
        let val;
        if let Some(v) = attributes {
            val = py_any_to_val(v, &self.graph);
        } else {
            val = Val::Null;
        }
        match val {
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
        match Runtime::run_attribute_functions(&mut self.graph, None, &Some(attrs), true) {
            Ok(res) => Ok(res),
            Err(err) => Err(PyValueError::new_err(err)),
        }
    }

    #[pyo3(signature = (path, args = None))]
    /// Call a singular function in the document by path.
    /// Arguments are a list (Ex. `doc.call('total', [21])`), a single value, or omitted/None for no arguments.
    pub fn call<'py>(&mut self, py: Python<'py>, path: &str, args: Option<&Bound<'py, PyAny>>) -> PyResult<Bound<'py, PyAny>> {
        let mut arguments = vec![];
        match args.map(|args| py_any_to_val(args, &self.graph)).unwrap_or(Val::Null) {
            Val::List(vals) => {
                for val in vals {
                    arguments.push(val.read().clone());
                }
            },
            Val::Null |
            Val::Void => { /* None value. */ },
            val => {
                arguments.push(val);
            }
        }
        match Runtime::call(&mut self.graph, path, arguments) {
            Ok(res) => Ok(val_to_py(py, res)),
            Err(err) => Err(PyValueError::new_err(err.to_string()))
        }
    }


    /*****************************************************************************
     * Interop
     *****************************************************************************/
    
    /// Give this document file system and environment access: the fs library, env functions, and file
    /// imports. Off by default, so a document can only read and change itself.
    pub fn allow_system(&mut self) {
        self.graph.allow_system();
    }

    /// Give this document network access (the Http library). Off by default.
    pub fn allow_http(&mut self) {
        self.graph.allow_http();
    }

    #[pyo3(signature = (lib, name, func, is_async = false))]
    /// Add a python function as a stof library function.
    pub fn lib(&mut self, lib: &str, name: &str, func: Py<PyAny>, is_async: bool) {
        set_py_lib_func(&mut self.graph, lib, name, func, is_async);
    }


    /*****************************************************************************
     * I/O
     *****************************************************************************/
    
    #[pyo3(signature = (stof, node = None, profile = "prod"))]
    /// Parse some Stof into this document.
    pub fn parse(&mut self, stof: &str, node: Option<&Bound<'_, PyAny>>, profile: &str) -> PyResult<bool> {
        self.string_import(stof, "stof", node, profile)
    }
    
    #[pyo3(signature = (src, format = "stof", node = None, profile = "prod"))]
    /// String import, using a format of choice.
    pub fn string_import(&mut self, src: &str, format: &str, node: Option<&Bound<'_, PyAny>>, profile: &str) -> PyResult<bool> {
        let val;
        if let Some(node) = node {
            val = py_any_to_val(node, &self.graph);
        } else {
            val = Val::Null;
        }

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

        let profile = match profile {
            "prod" => Profile::prod(),
            "test" => Profile::test(),
            "prod_docs" => Profile::docs(false),
            "docs" => Profile::docs(true),
            _ => Profile::default(),
        };

        match self.graph.string_import(format, src, Some(parse_node), &profile) {
            Ok(_) => Ok(true),
            Err(err) => Err(PyValueError::new_err(err.to_string()))
        }
    }

    #[pyo3(signature = (bytes, format = "bstf", node = None, profile = "prod"))]
    /// Binary import, using a format of choice.
    pub fn binary_import(&mut self, bytes: &Bound<'_, PyAny>, format: &str, node: Option<&Bound<'_, PyAny>>, profile: &str) -> PyResult<bool> {
        let val;
        if let Some(node) = node {
            val = py_any_to_val(node, &self.graph);
        } else {
            val = Val::Null;
        }

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

        let profile = match profile {
            "prod" => Profile::prod(),
            "test" => Profile::test(),
            "prod_docs" => Profile::docs(false),
            "docs" => Profile::docs(true),
            _ => Profile::default(),
        };

        let bin;
        match py_any_to_val(bytes, &self.graph) {
            Val::Blob(by) => bin = by,
            _ => { return Err(PyValueError::new_err("Stof binary import requires a binary 'bytes' argument")) }
        }

        match self.graph.binary_import(format, bin, Some(parse_node), &profile) {
            Ok(_) => Ok(true),
            Err(err) => Err(PyValueError::new_err(err.to_string()))
        }
    }

    #[pyo3(signature = (format = "json", node = None))]
    /// String export, using a format of choice.
    pub fn string_export(&self, format: &str, node: Option<&Bound<'_, PyAny>>) -> PyResult<String> {
        let val;
        if let Some(node) = node {
            val = py_any_to_val(node, &self.graph);
        } else {
            val = Val::Null;
        }

        let exp_node;
        match val {
            Val::Obj(node) => {
                if node.node_exists(&self.graph) {
                    exp_node = node;
                } else {
                    return Err(PyValueError::new_err("export node not found"));
                }
            },
            Val::Null |
            Val::Void => {
                if let Some(root) = self.graph.main_root() {
                    exp_node = root;
                } else {
                    return Err(PyValueError::new_err("export node not found"));
                }
            },
            _ => {
                return Err(PyValueError::new_err("export node not found"));
            }
        }
        match self.graph.string_export(format, Some(exp_node)) {
            Ok(val) => Ok(val),
            Err(err) => Err(PyValueError::new_err(err.to_string()))
        }
    }

    #[pyo3(signature = (format = "bstf", node = None))]
    /// Binary export, using a format of choice (the main root by default).
    pub fn binary_export<'py>(&self, py: Python<'py>, format: &str, node: Option<&Bound<'py, PyAny>>) -> PyResult<Bound<'py, PyAny>> {
        let val = node.map(|node| py_any_to_val(node, &self.graph)).unwrap_or(Val::Null);
        let exp_node;
        match val {
            Val::Obj(node) => {
                if node.node_exists(&self.graph) {
                    exp_node = node;
                } else {
                    return Err(PyValueError::new_err("export node not found"));
                }
            },
            Val::Null |
            Val::Void => {
                if let Some(root) = self.graph.main_root() {
                    exp_node = root;
                } else {
                    return Err(PyValueError::new_err("export node not found"));
                }
            },
            _ => {
                return Err(PyValueError::new_err("export node not found"));
            }
        }
        match self.graph.binary_export(format, Some(exp_node)) {
            Ok(val) => Ok(val_to_py(py, Val::Blob(val))),
            Err(err) => Err(PyValueError::new_err(err.to_string()))
        }
    }
}


#[pymodule]
/// Python module implemented in Rust.
/// Name must match the lib name in Cargo.toml
mod pystof {
    #[pymodule_export]
    use super::Doc;
}
