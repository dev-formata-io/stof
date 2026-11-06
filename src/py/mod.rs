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

    #[pyo3(signature = (src, args = None, node = None))]
    /// Call a function from Stof source without adding it to the document (Ex. a one-off script or check).
    /// The source's first function runs as if it were defined on node (an object path or ID, the root by
    /// default), and everything the source declares is removed after the call.
    /// Ex. `doc.call_source('fn total(qty: int) -> float { self.price * qty }', [3])`
    pub fn call_source<'py>(&mut self, py: Python<'py>, src: &str, args: Option<&Bound<'py, PyAny>>, node: Option<&Bound<'py, PyAny>>) -> PyResult<Bound<'py, PyAny>> {
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
        let context = match node.map(|node| py_any_to_val(node, &self.graph)).unwrap_or(Val::Null) {
            Val::Null | Val::Void => None,
            Val::Obj(node) if node.node_exists(&self.graph) => Some(node),
            Val::Str(path) => match self.graph.host_node(&path) {
                Some(node) => Some(node),
                None => return Err(PyValueError::new_err(format!("object '{path}' not found"))),
            },
            _ => return Err(PyValueError::new_err("node must be an object path or ID")),
        };
        match Runtime::call_source(&mut self.graph, src, context, arguments) {
            Ok(res) => Ok(val_to_py(py, res)),
            Err(err) => Err(PyValueError::new_err(err.to_string()))
        }
    }

    #[pyo3(signature = (path, shallow = false))]
    /// Remove a field, function, or object by path (Ex. "config.staging"), like drop() in Stof.
    /// A field that holds an object drops the object too, unless shallow (then only the field is removed).
    /// Returns whether anything was removed.
    pub fn remove(&mut self, path: &str, shallow: bool) -> bool {
        self.graph.remove(path, shallow)
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

    /// Let documents allowed any host (allow_http) reach this machine and private networks too (off by default).
    pub fn allow_private_network(&mut self) {
        self.graph.allow_private_network();
    }

    /// Give this document network access to these hosts only: requests (and redirects) to any other host fail.
    /// An entry is a host name (any port, Ex. "api.example.com") or a host and port ("localhost:8080").
    pub fn allow_http_hosts(&mut self, hosts: Vec<String>) {
        self.graph.allow_http_hosts(hosts);
    }

    /// Stop allowing these hosts (listed with allow_http_hosts): requests to them fail from now on. Removing every
    /// listed host leaves none allowed. Returns False when any host is allowed (allow_http without a list).
    pub fn disallow_http_hosts(&mut self, hosts: Vec<String>) -> bool {
        self.graph.disallow_http_hosts(hosts)
    }

    /// Elements the html format may write (Ex. the components your page renders): only these, instead of the
    /// standard HTML elements and custom elements. Documents can't change it. Calling it again adds elements.
    pub fn allow_html_tags(&mut self, tags: Vec<String>) {
        self.graph.allow_html_tags(tags);
    }

    /// Attributes the html format writes only when allowed: style, action, and formaction (off by default).
    /// Documents can't change it. Calling it again adds attributes.
    pub fn allow_html_attributes(&mut self, attributes: Vec<String>) {
        self.graph.allow_html_attributes(attributes);
    }

    #[pyo3(signature = (keep = true))]
    /// Keep each function's source text from now on (off by default), so the readable `stof` export writes
    /// functions as Stof. It costs the memory of the text.
    pub fn keep_source(&mut self, keep: bool) {
        self.graph.keep_source(keep);
    }

    #[pyo3(signature = (quiet = true))]
    /// Print nothing to the console (off by default): no parse warnings, test progress, or pln/err/dbg output.
    /// Results and errors still come back (Ex. `test` still returns its report).
    pub fn quiet(&mut self, quiet: bool) {
        self.graph.quiet(quiet);
    }

    /// How long a call can run, in seconds, before it fails with an execution timeout (None for no limit).
    /// Defaults to 120 (2 minutes).
    pub fn set_max_execution_time(&mut self, seconds: Option<f64>) {
        self.graph.max_execution_time = seconds.map(|seconds| std::time::Duration::from_secs_f64(seconds.max(0.)));
    }

    /// How much memory a call can use, in bytes, before it fails with a memory limit error (None for no limit).
    /// Defaults to 268,435,456 (256 MB).
    pub fn set_max_memory(&mut self, bytes: Option<usize>) {
        self.graph.max_memory = bytes;
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

    #[pyo3(signature = (path, fields = false, digest = None, public_key = None, signature = None, removable = false, reads = None, hosts = None))]
    /// Seal an object (path, or a type as "<Name>") and everything under it: its functions can be called but not
    /// changed. With fields, its fields are frozen too. With digest, only if its digest is that; with public_key and
    /// signature, only if the digest is signed by that key (a signature under "stof/seal": Doc.sign(secret,
    /// "stof/seal", digest)). With removable, it can still be removed whole. With reads (paths, looked up each time
    /// the part is entered), its code reaches only itself and those objects; with hosts, its Http.fetch reaches only
    /// those hosts. Returns the digest.
    pub fn seal(&mut self, path: &str, fields: bool, digest: Option<&str>, public_key: Option<&str>, signature: Option<&str>, removable: bool, reads: Option<Vec<String>>, hosts: Option<Vec<String>>) -> PyResult<String> {
        let options = crate::model::seal::SealOptions { fields, removable, reads, hosts };
        let res = match (public_key, signature) {
            (Some(key), Some(signature)) => {
                if let Some(expected) = digest {
                    let actual = self.graph.digest_path(path, fields).map_err(|e| PyValueError::new_err(e.to_string()))?;
                    if !actual.eq_ignore_ascii_case(expected.trim()) {
                        return Err(PyValueError::new_err(format!("sealed: {path} isn't what was expected: its digest is {actual}, not {expected}")));
                    }
                }
                self.graph.seal_signed_with(path, &options, key, signature)
            },
            (None, None) => self.graph.seal_path_with(path, &options, digest),
            _ => return Err(PyValueError::new_err("a signature needs both a public key and a signature")),
        };
        res.map_err(|error| PyValueError::new_err(error.to_string()))
    }

    /// Unseal an object that seal sealed (path, or a type as "<Name>").
    pub fn unseal(&mut self, path: &str) -> PyResult<()> {
        self.graph.unseal_path(path).map_err(|error| PyValueError::new_err(error.to_string()))
    }

    #[pyo3(signature = (path, fields = false))]
    /// The seal digest of an object (path, or a type as "<Name>"), with its fields' values when fields.
    pub fn digest(&self, path: &str, fields: bool) -> PyResult<String> {
        self.graph.digest_path(path, fields).map_err(|error| PyValueError::new_err(error.to_string()))
    }

    /// How an object (path, or a type as "<Name>") is sealed: a dict with part, fields, removable, and confined (a
    /// list of {reads, hosts}), or None when it isn't sealed.
    pub fn sealed<'py>(&self, py: Python<'py>, path: &str) -> PyResult<Bound<'py, PyAny>> {
        let info = self.graph.seal_info_path(path).map_err(|error| PyValueError::new_err(error.to_string()))?;
        match info {
            Some(info) => py.import("json")?.call_method1("loads", (info.to_json(&self.graph).to_string(),)),
            None => Ok(py.None().into_bound(py)),
        }
    }

    #[staticmethod]
    /// A new secret key (base64, 32 bytes from the system's secure random source). Share only its public key.
    pub fn generate_key() -> String {
        crate::model::seal::generate_key()
    }

    #[staticmethod]
    /// The public key (base64) for a secret key.
    pub fn public_key(secret_key: &str) -> PyResult<String> {
        crate::model::seal::public_key(secret_key).map_err(|error| PyValueError::new_err(error.to_string()))
    }

    #[staticmethod]
    /// Sign a message (str as UTF-8, or bytes) under a context (Ex. "orders/v1") with a secret key: the signature
    /// (base64). It verifies only under the same context (documents: Key.verify(public, context, message, sig)).
    pub fn sign(secret_key: &str, context: &str, message: &Bound<'_, PyAny>) -> PyResult<String> {
        let bytes = py_message_bytes(message)?;
        crate::model::seal::sign(secret_key, context, &bytes).map_err(|error| PyValueError::new_err(error.to_string()))
    }

    #[staticmethod]
    /// Is this a signature over this message (str or bytes), under this context, by this public key?
    pub fn verify(public_key: &str, context: &str, message: &Bound<'_, PyAny>, signature: &str) -> PyResult<bool> {
        let bytes = py_message_bytes(message)?;
        Ok(crate::model::seal::verify(public_key, context, &bytes, signature))
    }

    #[staticmethod]
    /// The fields and functions in a Stof text: a list of dicts with kind ("field" or "fn"), path, name, object,
    /// line, col, and value (its source).
    pub fn outline<'py>(py: Python<'py>, text: &str) -> PyResult<Bound<'py, PyAny>> {
        let defs = crate::parser::edit::outline(text).map_err(|error| PyValueError::new_err(error.to_string()))?;
        let json = crate::parser::edit::outline_json(text, &defs).to_string();
        py.import("json")?.call_method1("loads", (json,))
    }

    #[staticmethod]
    /// Edit a Stof text in place: a list of edits like {"set": "a.b", "value": 1}, {"set": "a.b", "stof": "1GB"},
    /// {"set_fn": "a.f", "stof": "fn f() {}"}, {"remove": "a.b"}, or {"rename": "a.b", "to": "c"}.
    pub fn edit<'py>(py: Python<'py>, text: &str, edits: &Bound<'py, PyAny>) -> PyResult<String> {
        let edits: String = py.import("json")?.call_method1("dumps", (edits,))?.extract()?;
        crate::parser::edit::edit_json(text, &edits).map_err(|error| PyValueError::new_err(error.to_string()))
    }

    #[pyo3(signature = (format = "json", node = None))]
    /// String export, using a format of choice.
    pub fn string_export(&mut self, format: &str, node: Option<&Bound<'_, PyAny>>) -> PyResult<String> {
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
            // a path, like get, set, and call take (Ex. "config.production")
            Val::Str(path) => {
                match self.graph.host_node(&path) {
                    Some(node) => exp_node = node,
                    None => return Err(PyValueError::new_err("export node not found")),
                }
            },
            _ => {
                return Err(PyValueError::new_err("export node not found"));
            }
        }
        match self.graph.stringify(format, Some(exp_node)) { // calls computed values (Ex. #[attr] functions in html)
            Ok(val) => Ok(val),
            Err(err) => Err(PyValueError::new_err(err.to_string()))
        }
    }

    #[pyo3(signature = (format = "bstf", node = None))]
    /// Binary export, using a format of choice (the main root by default).
    pub fn binary_export<'py>(&mut self, py: Python<'py>, format: &str, node: Option<&Bound<'py, PyAny>>) -> PyResult<Bound<'py, PyAny>> {
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
            // a path, like get, set, and call take (Ex. "config.production")
            Val::Str(path) => {
                match self.graph.host_node(&path) {
                    Some(node) => exp_node = node,
                    None => return Err(PyValueError::new_err("export node not found")),
                }
            },
            _ => {
                return Err(PyValueError::new_err("export node not found"));
            }
        }
        match self.graph.blobify(format, Some(exp_node)) { // calls computed values (Ex. #[attr] functions in html)
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


/// A message's bytes: a str's UTF-8, or bytes.
fn py_message_bytes(message: &Bound<'_, PyAny>) -> PyResult<Vec<u8>> {
    if let Ok(text) = message.extract::<String>() { return Ok(text.into_bytes()); }
    if let Ok(bytes) = message.extract::<Vec<u8>>() { return Ok(bytes); }
    Err(PyValueError::new_err("a message must be a str or bytes (the exact bytes that are signed)"))
}
