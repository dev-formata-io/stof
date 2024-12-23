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

use std::{collections::HashSet, fs, sync::{Arc, RwLock}, time::SystemTime};
use anyhow::{anyhow, Result};
use bytes::Bytes;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use crate::{bytes::BYTES, text::TEXT, SField, SFunc, SVal, BSTOF, STOF};
use super::{runtime::{DocPermissions, Library, Symbol, SymbolTable}, ArrayLibrary, CustomTypes, Format, FunctionLibrary, IntoDataRef, NumberLibrary, ObjectLibrary, SFormats, SGraph, SLibraries, SNodeRef, StdLibrary, StringLibrary, TupleLibrary};

#[cfg(feature = "js")]
use crate::js::StofLibFunc;
#[cfg(feature = "js")]
use std::collections::BTreeMap;

#[cfg(feature = "json")]
use crate::json::JSON;
#[cfg(feature = "json")]
use crate::json::NDJSON;

#[cfg(feature = "toml")]
use crate::toml::TOML;

#[cfg(feature = "yaml")]
use crate::yaml::YAML;

#[cfg(feature = "xml")]
use crate::xml::XML;

#[cfg(feature = "urlencoded")]
use crate::urlencoded::URLENC;


/// Stof Document.
/// Holds a Graph, containing the data contained within this document as well as
/// any permissions, libraries, etc... needed for this document to manipulate itself.
#[derive(Clone, Serialize, Deserialize)]
pub struct SDoc {
    pub graph: SGraph,
    pub perms: DocPermissions,
    pub types: CustomTypes,

    /// Formats that this Doc supports.
    /// It is up to the local Doc to provide formats.
    #[serde(skip)]
    pub formats: SFormats,

    /// Libraries that this Doc can call when scripting.
    /// It is up to the local Doc to provide libraries.
    #[serde(skip)]
    pub libraries: SLibraries,
    
    #[serde(skip)]
    pub(crate) self_stack: Vec<SNodeRef>,

    #[serde(skip)]
    pub(crate) stack: Vec<SVal>,

    #[serde(skip)]
    pub(crate) table: SymbolTable,

    #[serde(skip)]
    pub(crate) bubble_control_flow: u8,

    #[cfg(feature = "js")]
    #[serde(skip)]
    pub libfuncs: Arc<RwLock<BTreeMap<String, BTreeMap<String, StofLibFunc>>>>,
}
impl Default for SDoc {
    fn default() -> Self {
        let mut def_graph = SGraph::default();
        def_graph.insert_root("root");
        Self::new(def_graph)
    }
}
impl SDoc {
    /// New document from a graph.
    pub fn new(graph: SGraph) -> Self {
        let mut doc = Self {
            graph,
            types: Default::default(),
            stack: Default::default(),
            table: Default::default(),
            self_stack: Default::default(),
            libraries: Default::default(),
            formats: Default::default(),
            perms: Default::default(),
            bubble_control_flow: 0,

            #[cfg(feature = "js")]
            libfuncs: Default::default(),
        };
        doc.load_std_formats();
        doc.load_std_lib();
        doc
    }

    /// New document from a string import format.
    pub fn src(src: &str, format: &str) -> Result<Self> {
        let mut doc = Self::default();
        doc.string_import(format, src, "")?;
        Ok(doc)
    }

    /// New document from a file import.
    pub fn file(path: &str, format: &str) -> Result<Self> {
        let mut doc = Self::default();
        doc.file_import(format, path, format, "")?;
        Ok(doc)
    }

    /// New document from bytes.
    pub fn bytes(mut bytes: Bytes, format: &str) -> Result<Self> {
        let mut doc = Self::default();
        doc.header_import(format, format, &mut bytes, "")?;
        Ok(doc)
    }

    /// Export this document to a text file at "path" using "format".
    pub fn text_file_out(&self, path: &str, format: &str) -> Result<()> {
        if let Ok(out) = self.export_min_string(format, None) {
            fs::write(path, out)?;
            return Ok(());
        }
        Err(anyhow!("Could not export to a text file"))
    }

    /// Export this document to a binary file at "path" using "format".
    pub fn bin_file_out(&self, path: &str, format: &str) -> Result<()> {
        if let Ok(out) = self.export_bytes(format, None) {
            fs::write(path, out)?;
            return Ok(());
        }
        Err(anyhow!("Could not export to a binary file"))
    }


    /*****************************************************************************
     * Formats.
     *****************************************************************************/
    
    /// Load the Stof standard formats.
    fn load_std_formats(&mut self) {
        self.load_format(Arc::new(TEXT{}));
        self.load_format(Arc::new(BYTES{}));

        // STOF format ".stof" text files
        self.load_format(Arc::new(STOF{}));

        // BSTOF format ".bstof" binary files
        self.load_format(Arc::new(BSTOF{}));

        // JSON format ".json" files and NDJSON (newlines between json)
        #[cfg(feature = "json")]
        self.load_format(Arc::new(JSON{}));
        #[cfg(feature = "json")]
        self.load_format(Arc::new(NDJSON{}));

        // TOML format ".toml" files
        #[cfg(feature = "toml")]
        self.load_format(Arc::new(TOML{}));

        // YAML format ".yaml" files
        #[cfg(feature = "yaml")]
        self.load_format(Arc::new(YAML{}));

        // XML format ".xml" files
        #[cfg(feature = "xml")]
        self.load_format(Arc::new(XML{}));

        // URL encoding "urlencoded" format
        #[cfg(feature = "urlencoded")]
        self.load_format(Arc::new(URLENC{}));
    }

    /// Load a format into this document.
    pub fn load_format(&mut self, format: Arc<dyn Format>) {
        self.formats.insert(format);
    }

    /// Available formats
    pub fn available_formats(&self) -> HashSet<String> {
        self.formats.available()
    }

    /// Content type for a format.
    pub fn format_content_type(&self, format: &str) -> Option<String> {
        self.formats.content_type(format)
    }

    /// Header import (content type with bytes).
    pub fn header_import(&mut self, format: &str, content_type: &str, bytes: &mut Bytes, as_name: &str) -> Result<()> {
        self.formats.clone().header_import(format, self, content_type, bytes, as_name)
    }

    /// String import.
    pub fn string_import(&mut self, format: &str, src: &str, as_name: &str) -> Result<()> {
        self.formats.clone().string_import(format, self, src, as_name)
    }

    /// File import.
    /// Stof Syntax: 'import <format> "<path>.<extension>" as <as_name>;'
    /// If <format> isn't supplied, "format" will be "extension".
    /// If <as_name> isn't supplied, the data should be imported into the current doc scope (or main root).
    pub fn file_import(&mut self, format: &str, full_path: &str, extension: &str, as_name: &str) -> Result<()> {
        self.formats.clone().file_import(format, self, full_path, extension, as_name)
    }

    /// Export document string.
    pub fn export_string(&self, format: &str, node: Option<&SNodeRef>) -> Result<String> {
        self.formats.export_string(format, self, node)
    }

    /// Export document min string.
    pub fn export_min_string(&self, format: &str, node: Option<&SNodeRef>) -> Result<String> {
        self.formats.export_min_string(format, self, node)
    }

    /// Export document bytes.
    pub fn export_bytes(&self, format: &str, node: Option<&SNodeRef>) -> Result<Bytes> {
        self.formats.export_bytes(format, self, node)
    }


    /*****************************************************************************
     * Libraries.
     *****************************************************************************/

    /// Load the Stof standard library.
    fn load_std_lib(&mut self) {
        self.load_lib(Arc::new(RwLock::new(StdLibrary::default())));
        self.load_lib(Arc::new(RwLock::new(ObjectLibrary::default())));
        self.load_lib(Arc::new(RwLock::new(ArrayLibrary::default())));
        self.load_lib(Arc::new(RwLock::new(FunctionLibrary::default())));
        self.load_lib(Arc::new(RwLock::new(NumberLibrary::default())));
        self.load_lib(Arc::new(RwLock::new(StringLibrary::default())));
        self.load_lib(Arc::new(RwLock::new(TupleLibrary::default())));
    }
    
    /// Load a library into this document.
    pub fn load_lib(&mut self, library: Arc<RwLock<dyn Library>>) {
        self.libraries.insert(library);
    }

    /// Get a library in this doc.
    pub fn library(&mut self, lib: &str) -> Option<Arc<RwLock<dyn Library>>> {
        if let Some(library) = self.libraries.get(lib) {
            return Some(library.clone());
        }
        None
    }

    /// Available libraries
    pub fn available_libraries(&self) -> HashSet<String> {
        self.libraries.available()
    }


    /*****************************************************************************
     * General data interface.
     *****************************************************************************/
    
    /// Get a value from this document by path.
    /// If the path points to a field, the value will be retrieved.
    /// If the path points to a function, it will be called.
    pub fn get(&mut self, path: &str, start: Option<&SNodeRef>) -> Option<SVal> {
        if let Some(field) = self.field(path, start) {
            return Some(field.value);
        } else if let Some(func) = self.func(path, start) {
            let mut parameters = Vec::new();
            if let Some(params) = func.attributes.get("get") {
                parameters.push(params.clone());
            }
            if let Ok(res) = func.call(self, parameters, true) {
                return Some(res);
            }
        }
        None
    }


    /*****************************************************************************
     * Field interface.
     *****************************************************************************/
    
    /// Find a field in this document with a path.
    /// Path is dot '.' separated.
    pub fn field(&self, path: &str, start: Option<&SNodeRef>) -> Option<SField> {
        SField::field(&self.graph, path, '.', start)
    }


    /*****************************************************************************
     * Function interface.
     *****************************************************************************/

    /// Run the main functions on a node or within this document.
    /// Main functions are denoted with a #[main] attribute in the text format.
    pub fn run(&mut self, context: Option<&SNodeRef>) -> Vec<(SFunc, SVal)> {
        let mut results = Vec::new();
        let functions;
        if context.is_some() {
            functions = SFunc::recursive_funcs(&self.graph, context.unwrap());
        } else {
            functions = SFunc::all_funcs(&self.graph);
        }
        for func in functions {
            if let Some(attr_val) = func.attributes.get("main") {
                let result;
                if attr_val.is_empty() {
                    result = func.call(self, vec![], true);
                } else {
                    result = func.call(self, vec![attr_val.clone()], true);
                }
                if let Ok(res) = result {
                    results.push((func, res));
                }
            }
        }
        results
    }
    
    /// Find a function in this document with a path.
    /// Path is dot '.' separated.
    pub fn func(&self, path: &str, start: Option<&SNodeRef>) -> Option<SFunc> {
        SFunc::func(&self.graph, path, '.', start)
    }

    /// Call a function in this document with a path.
    pub fn call_func(&mut self, path: &str, start: Option<&SNodeRef>, params: Vec<SVal>) -> Result<SVal> {
        if let Some(func) = self.func(path, start) {
            return func.call(self, params, true);
        }
        Err(anyhow!("Did not find a function at path '{}' to call", path))
    }
    
    /// Call a function on this document.
    /// Function does not have to be contained within this document.
    pub fn call(&mut self, func: &SFunc, params: Vec<SVal>) -> Result<SVal> {
        func.call(self, params, true)
    }

    /// Test some Stof in a file.
    /// Will run all #[test] functions.
    /// Set 'throw' to true if you want this function to panic if tests fail.
    pub fn test_file(file: &str, throw: bool) {
        let compile_start = SystemTime::now();
        let doc_res = Self::file(file, "stof");
        let dur = (compile_start.elapsed().unwrap().as_secs_f32() * 100.0).round() / 100.0;
        println!("{} {}s", "stof compiled in".dimmed(), dur);
        let mut doc;
        match doc_res {
            Ok(dr) => doc = dr,
            Err(err) => {
                let message = format!("{}: {}", "parse error".red(), err.to_string());
                if throw {
                    panic!("{message}");
                } else {
                    println!("{message}");
                    return;
                }
            },
        }

        let res = doc.run_tests(throw, None);
        match res {
            Ok(res) => {
                println!("{res}");
            },
            Err(err) => {
                panic!("{err}");
            }
        }
    }

    /// Test some Stof.
    /// Will run all #[test] functions in 'stof'.
    /// Set 'throw' to true if you want this function to panic if tests fail.
    pub fn test(stof: &str, throw: bool, context: Option<&SNodeRef>) {
        let doc_res = Self::src(stof, "stof");
        let mut doc;
        match doc_res {
            Ok(dr) => doc = dr,
            Err(err) => {
                let message = format!("{}: {}", "parse error".red(), err.to_string());
                if throw {
                    panic!("{message}");
                } else {
                    println!("{message}");
                    return;
                }
            },
        }

        let res = doc.run_tests(throw, context);
        match res {
            Ok(res) => {
                println!("{res}");
            },
            Err(err) => {
                panic!("{err}");
            }
        }
    }

    /// Run the test functions on a node or within this document, throwing an error if any fail.
    /// Test functions are denoted with a #[test(<result_expr_eq>)] attribute.
    pub fn run_tests(&mut self, throw: bool, context: Option<&SNodeRef>) -> Result<String, String> {
        let mut functions;
        if context.is_some() {
            functions = SFunc::recursive_funcs(&self.graph, context.unwrap());
        } else {
            functions = SFunc::all_funcs(&self.graph);
        }
        functions.retain(|f| f.attributes.contains_key("test"));

        let total = functions.len();
        println!("{} {} {}", "running".bold(), total, "Stof tests".bold());
        let mut failures = Vec::new();
        let mut profiles = Vec::new();
        let start = SystemTime::now();
        for func in functions {
            if let Some(res_test_val) = func.attributes.get("test") {
                let silent = func.attributes.contains_key("silent");
                let mut result = func.call(self, vec![], true);

                let func_nodes = func.data_ref().nodes(&self.graph);
                let func_path;
                if func_nodes.len() > 0 {
                    func_path = func_nodes.first().unwrap().path(&self.graph);
                } else {
                    func_path = String::from("<unknown>");
                }

                if let Some(error_val) = func.attributes.get("errors") {
                    if result.is_err() {
                        result = Ok(error_val.clone());
                    } else {
                        result = Err(anyhow!("Expected function to throw an error"));
                    }
                }

                match result {
                    Ok(res_val) => {
                        let name = func.name.clone();
                        if !res_test_val.is_empty() && &res_val != res_test_val {
                            if !silent {
                                println!("{} {} {} ... {}", "test".purple(), func_path.italic().dimmed(), name.blue(), "failed".bold().red());
                            }

                            let err_str = format!("{:?} does not equal {:?}", res_val, res_test_val);
                            failures.push((func, format!("\t{}: {} at {}: {}", "failed".bold().red(), name.blue(), func_path.italic().dimmed(), err_str.bold())));
                        } else {
                            if !silent {
                                println!("{} {} {} ... {}", "test".purple(), func_path.italic().dimmed(), name.blue(), "ok".bold().green());
                            }
                            
                            // This is a successful running of the test! Now check if we should profile the function
                            if let Some(profile_val) = func.attributes.get("profile") {
                                if profile_val.is_empty() || profile_val.truthy() {
                                    let mut iterations = 100;
                                    match profile_val {
                                        SVal::Number(num) => {
                                            iterations = num.int() as u128;
                                        },
                                        _ => {}
                                    }

                                    let profile_start = SystemTime::now();
                                    for _ in 0..iterations {
                                        let _ = func.call(self, vec![], true);
                                    }
                                    let total_duration = profile_start.elapsed().unwrap();
                                    let total_ns = total_duration.as_nanos();
                                    let each_ns = total_ns / iterations;
                                    
                                    let dur = (total_duration.as_secs_f32() * 100.0).round() / 100.0;
                                    profiles.push(format!("\t{} {} ... {} iterations; {}s ({}ms); {}ns per call", func_path.italic().dimmed(), name.blue(), iterations, dur, total_duration.as_millis(), each_ns));
                                }
                            }
                        }
                    },
                    Err(err) => {
                        let name = func.name.clone();
                        if !silent {
                            println!("{} {} {} ... {}", "test".purple(), func_path.italic().dimmed(), name.blue(), "failed".bold().red());
                        }

                        let err_str = err.to_string();
                        failures.push((func, format!("\t{}: {} at {}: {}", "failed".bold().red(), name.blue(), func_path.italic().dimmed(), err_str.bold())));
                    }
                }
            }
        }

        let duration = start.elapsed().unwrap();
        let mut output = "\n".to_string();
        let mut result = "ok".bold().green();
        if failures.len() > 0 {
            result = "failed".bold().red();
            output.push_str(&format!("{} failures:\n", failures.len()));
            for failure in &failures {
                output.push_str(&format!("{}\n", failure.1));
            }
            output.push('\n');
        }
        if profiles.len() > 0 {
            output.push_str(&format!("{} profiles:\n", profiles.len()));
            for profile in &profiles {
                output.push_str(&format!("{}\n", profile));
            }
            output.push('\n');
        }
        let passed = total - failures.len();
        let dur = (duration.as_secs_f32() * 100.0).round() / 100.0;
        output.push_str(&format!("test result: {}. {} passed; {} failed; finished in {}s", result, passed, failures.len(), dur));

        if throw && failures.len() > 0 {
            return Err(output);
        }
        return Ok(output);
    }


    /*****************************************************************************
     * Stof Parser/Runtime Interface.
     *****************************************************************************/

    /// Self pointer.
    pub(crate) fn self_ptr(&self) -> Option<SNodeRef> {
        if let Some(last) = self.self_stack.last() {
            if last.exists(&self.graph) {
                return Some(last.clone());
            }
        }
        None
    }

    /// New table.
    /// Returns the current table, replacing it with a new one.
    /// This happens for function calls.
    pub(crate) fn new_table(&mut self) -> SymbolTable {
        let current = self.table.clone();
        self.table = SymbolTable::default();
        return current;
    }

    /// Set table.
    pub(crate) fn set_table(&mut self, table: SymbolTable) {
        self.table = table;
    }

    /// Add a variable to the current scope.
    pub(crate) fn add_variable<T>(&mut self, name: &str, value: T) where T: Into<SVal> {
        let symbol = Symbol::Variable(value.into());
        self.table.insert(name, symbol);
    }

    /// Set a variable.
    /// Will not add the variable if not already present.
    /// Sets current scope or above variables!
    pub(crate) fn set_variable<T>(&mut self, name: &str, value: T) -> bool where T: Into<SVal> {
        self.table.set_variable(name, &value.into(), &mut self.graph)
    }

    /// Drop a symbol from the current scope.
    pub(crate) fn drop(&mut self, name: &str) -> Option<Symbol> {
        self.table.remove(name)
    }

    /// Get a symbol from the current scope or above.
    pub(crate) fn get_symbol(&mut self, name: &str) -> Option<&Symbol> {
        self.table.get(name)
    }

    /// Has a symbol from the current scope or above.
    pub(crate) fn has_symbol(&mut self, name: &str) -> bool {
        self.table.get(name).is_some()
    }

    /// Push a value onto the stack.
    pub(crate) fn push<T>(&mut self, value: T) where T: Into<SVal> {
        let val: SVal = value.into();
        if !val.is_void() { // Prevent void from being pushed to the stack!
            self.stack.push(val);
        }
    }

    /// Pop a value from the stack.
    pub(crate) fn pop(&mut self) -> Option<SVal> {
        self.stack.pop()
    }

    /// Clean for scripting.
    pub(crate) fn clean(&mut self) {
        self.stack.clear();
        self.table = Default::default();
        self.self_stack.clear();
        self.bubble_control_flow = 0;
    }
}
