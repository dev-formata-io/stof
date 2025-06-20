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

use std::{collections::HashSet, mem::swap, sync::Arc, time::SystemTime};
use bytes::Bytes;
use colored::Colorize;
use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};
use crate::{bytes::BYTES, gitbook::Gitbook, lang::SError, text::TEXT, SData, SField, SFunc, SVal, BSTOF, STOF};
use super::{runtime::{DocPermissions, Library, Symbol, SymbolTable}, ArrayLibrary, BlobLibrary, SemVerLibrary, BoolLibrary, CustomTypes, DataLibrary, Format, FunctionLibrary, IntoDataRef, IntoNodeRef, MapLibrary, NumberLibrary, ObjectLibrary, SDataRef, SFormats, SGraph, SLibraries, SNodeRef, SProcesses, SetLibrary, StdLibrary, StringLibrary, TupleLibrary};

#[cfg(not(feature = "wasm"))]
use super::FileSystemLibrary;

#[cfg(not(feature = "wasm"))]
use super::TimeLibrary;

#[cfg(feature = "async")]
use super::TokioLibrary;

#[cfg(feature = "thread")]
use super::ThreadLibrary;

#[cfg(feature = "http")]
use super::HttpLibrary;

#[cfg(feature = "pkg")]
use crate::pkg::PKG;

#[cfg(feature = "markdown")]
use crate::text::markdown::MD;
#[cfg(feature = "markdown-lib")]
use crate::text::md_lib::MDLibrary;

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

#[cfg(feature = "docx")]
use crate::docx::DOCX;

#[cfg(feature = "image")]
use crate::image::load_image_formats;
#[cfg(feature = "image")]
use crate::image::library::SImageLibrary;

#[cfg(feature = "pdf")]
use crate::pdf::PDF;
#[cfg(feature = "pdf")]
use crate::pdf::library::SPDFLibrary;


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
    pub processes: SProcesses,

    #[serde(skip)]
    pub env_compiled_paths: FxHashSet<String>,
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
            libraries: Default::default(),
            formats: Default::default(),
            perms: Default::default(),
            processes: SProcesses::new(),
            env_compiled_paths: Default::default(),
        };
        doc.load_std_formats();
        doc.load_std_lib();
        doc
    }

    /// New document from a string import format.
    pub fn src(src: &str, format: &str) -> Result<Self, SError> {
        let mut doc = Self::default();
        doc.string_import("main", format, src, "")?;
        Ok(doc)
    }

    /// New document from a file import.
    pub fn file(path: &str, format: &str) -> Result<Self, SError> {
        let mut doc = Self::default();
        doc.file_import("main", format, path, format, "")?;
        Ok(doc)
    }

    /// New document from bytes.
    pub fn bytes(mut bytes: Bytes, format: &str) -> Result<Self, SError> {
        let mut doc = Self::default();
        doc.header_import("main", format, format, &mut bytes, "")?;
        Ok(doc)
    }

    /// Export this document to a text file at "path" using "format".
    pub fn text_file_out(&mut self, path: &str, format: &str) -> Result<(), SError> {
        if let Ok(out) = self.export_min_string("main", format, None) {
            self.fs_write_string("main", path, &out)?;
            return Ok(());
        }
        Err(SError::filesys("main", &self, "text_file_out", "could not export to a text file"))
    }

    /// Export this document to a binary file at "path" using "format".
    pub fn bin_file_out(&mut self, path: &str, format: &str) -> Result<(), SError> {
        if let Ok(out) = self.export_bytes("main", format, None) {
            self.fs_write_blob("main", path, out.to_vec())?;
            return Ok(());
        }
        Err(SError::filesys("main", &self, "bin_file_out", "could not export to a binary file"))
    }


    /*****************************************************************************
     * Formats.
     *****************************************************************************/
    
    /// Load the Stof standard formats.
    pub fn load_std_formats(&mut self) {
        self.load_format(Arc::new(TEXT{})); // "text" | .txt import "text" field
        self.load_format(Arc::new(BYTES{}));// "bytes" import "bytes" field

        // STOF format ".stof" text files
        self.load_format(Arc::new(STOF(false)));

        // BSTOF format ".bstof" binary files
        self.load_format(Arc::new(BSTOF{}));

        // PKG format
        #[cfg(feature = "pkg")]
        self.load_format(Arc::new(PKG::default()));

        #[cfg(feature = "markdown")]
        self.load_format(Arc::new(MD{}));   // .md import "markdown" field

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

        // DOCX format (.docx)
        #[cfg(feature = "docx")]
        self.load_format(Arc::new(DOCX{}));

        // IMAGE formats (.jpg, .png, .bmp, .ico, .tiff, .tif, .gif, .webp)
        #[cfg(feature = "image")]
        load_image_formats(self);

        // PDF format (.pdf)
        #[cfg(feature = "pdf")]
        self.load_format(Arc::new(PDF{}));
    }

    /// Load StofDocs format.
    /// Replaces the STOF format with a version that gererates documentation information.
    pub fn load_stof_docs(&mut self) {
        self.load_format(Arc::new(STOF(true)));
    }

    /// Load a format into this document.
    pub fn load_format(&mut self, format: Arc<dyn Format>) {
        self.formats.insert(format);
    }

    /// Available formats.
    pub fn available_formats(&self) -> HashSet<String> {
        self.formats.available()
    }

    /// Remove a format.
    /// If this format is registered as many formats, all are removed.
    pub fn remove_format(&mut self, format: &str) -> bool {
        self.formats.remove_all(format)
    }

    /// Remove a singular format.
    /// The format referenced may still exist under other formats.
    pub fn remove_singular_format(&mut self, format: &str) -> bool {
        self.formats.remove(format)
    }

    /// Content type for a format.
    pub fn format_content_type(&self, format: &str) -> Option<String> {
        self.formats.content_type(format)
    }

    /// Header import (content type with bytes).
    pub fn header_import(&mut self, pid: &str, format: &str, content_type: &str, bytes: &mut Bytes, as_name: &str) -> Result<(), SError> {
        self.formats.clone().header_import(format, pid, self, content_type, bytes, as_name)
    }

    /// String import.
    pub fn string_import(&mut self, pid: &str, format: &str, src: &str, as_name: &str) -> Result<(), SError> {
        self.formats.clone().string_import(format, pid, self, src, as_name)
    }

    /// File import.
    /// Stof Syntax: 'import <format> "<path>.<extension>" as <as_name>;'
    /// If <format> isn't supplied, "format" will be "extension".
    /// If <as_name> isn't supplied, the data should be imported into the current doc scope (or main root).
    pub fn file_import(&mut self, pid: &str, format: &str, full_path: &str, extension: &str, as_name: &str) -> Result<(), SError> {
        self.formats.clone().file_import(format, pid, self, full_path, extension, as_name)
    }

    /// Export document string.i
    pub fn export_string(&self, pid: &str, format: &str, node: Option<&SNodeRef>) -> Result<String, SError> {
        self.formats.export_string(format, pid, self, node)
    }

    /// Export document min string.
    pub fn export_min_string(&self, pid: &str, format: &str, node: Option<&SNodeRef>) -> Result<String, SError> {
        self.formats.export_min_string(format, pid, self, node)
    }

    /// Export document bytes.
    pub fn export_bytes(&self, pid: &str, format: &str, node: Option<&SNodeRef>) -> Result<Bytes, SError> {
        self.formats.export_bytes(format, pid, self, node)
    }


    /*****************************************************************************
     * Libraries.
     *****************************************************************************/

    /// Load the Stof standard library.
    pub fn load_std_lib(&mut self) {
        #[cfg(not(feature = "wasm"))]
        self.load_lib(Arc::new(FileSystemLibrary::default()));

        #[cfg(not(feature = "wasm"))]
        self.load_lib(Arc::new(TimeLibrary::default()));

        #[cfg(feature = "thread")]
        self.load_lib(Arc::new(ThreadLibrary::default()));

        #[cfg(feature = "async")]
        self.load_lib(Arc::new(TokioLibrary::default()));

        self.load_lib(Arc::new(StdLibrary::default()));
        self.load_lib(Arc::new(ObjectLibrary::default()));
        self.load_lib(Arc::new(ArrayLibrary::default()));
        self.load_lib(Arc::new(MapLibrary::default()));
        self.load_lib(Arc::new(SetLibrary::default()));
        self.load_lib(Arc::new(FunctionLibrary::default()));
        self.load_lib(Arc::new(NumberLibrary::default()));
        self.load_lib(Arc::new(StringLibrary::default()));
        self.load_lib(Arc::new(TupleLibrary::default()));
        self.load_lib(Arc::new(BoolLibrary::default()));
        self.load_lib(Arc::new(BlobLibrary::default()));
        self.load_lib(Arc::new(SemVerLibrary::default()));
        self.load_lib(Arc::new(DataLibrary::default()));

        #[cfg(feature = "http")]
        self.load_lib(Arc::new(HttpLibrary::default()));

        #[cfg(feature = "markdown-lib")]
        self.load_lib(Arc::new(MDLibrary::default()));

        #[cfg(feature = "image")]
        self.load_lib(Arc::new(SImageLibrary::default()));

        #[cfg(feature = "pdf")]
        self.load_lib(Arc::new(SPDFLibrary::default()));
    }
    
    /// Load a library into this document.
    pub fn load_lib(&mut self, library: Arc<dyn Library>) {
        self.libraries.insert(library);
    }

    /// Get a library in this doc.
    pub fn library(&self, lib: &str) -> Option<Arc<dyn Library>> {
        if let Some(library) = self.libraries.get(lib) {
            return Some(library.clone());
        }
        None
    }

    /// Available libraries.
    pub fn available_libraries(&self) -> HashSet<String> {
        self.libraries.available()
    }

    /// Remove a library.
    pub fn remove_library(&mut self, lib: &str) -> bool {
        self.libraries.remove(lib)
    }

    /// Write a string to a file using the fs library.
    pub fn fs_write_string(&mut self, pid: &str, path: &str, contents: &str) -> Result<(), SError> {
        if let Some(fs) = self.library("fs") {
            fs.call(pid, self, "write", &mut vec![SVal::String(path.to_owned()), SVal::String(contents.to_owned())])?;
            return Ok(());
        }
        Err(SError::filesys("main", &self, "fs_write_string", "no FileSystem 'fs' library loaded into this document"))
    }

    /// Write a blob to a file using the fs library.
    pub fn fs_write_blob(&mut self, pid: &str, path: &str, contents: Vec<u8>) -> Result<(), SError> {
        if let Some(fs) = self.library("fs") {
            fs.call(pid, self, "write_blob", &mut vec![SVal::String(path.to_owned()), SVal::Blob(contents)])?;
            return Ok(());
        }
        Err(SError::filesys("main", &self, "fs_write_blob", "no FileSystem 'fs' library loaded into this document"))
    }

    /// Read a file to a string using the fs library.
    pub fn fs_read_string(&mut self, pid: &str, path: &str) -> Result<String, SError> {
        if let Some(fs) = self.library("fs") {
            let res = fs.call(pid, self, "read", &mut vec![SVal::String(path.to_owned())])?;
            return Ok(res.owned_to_string());
        }
        Err(SError::filesys("main", &self, "fs_read_string", "no FileSystem 'fs' library loaded into this document"))
    }

    /// Read a file to a blob using the fs library.
    pub fn fs_read_blob(&mut self, pid: &str, path: &str) -> Result<Bytes, SError> {
        if let Some(fs) = self.library("fs") {
            let res = fs.call(pid, self, "read_blob", &mut vec![SVal::String(path.to_owned())])?;
            match res {
                SVal::Blob(blob) => {
                    return Ok(Bytes::from(blob));
                },
                _ => {}
            }
        }
        Err(SError::filesys("main", &self, "fs_read_blob", "no FileSystem 'fs' library loaded into this document"))
    }


    /*****************************************************************************
     * General data interface.
     *****************************************************************************/
    
    /// Get a value from this document by path.
    /// If the path points to a field, the value will be retrieved.
    /// If the path points to a function, it will be called.
    pub fn get(&mut self, path: &str, start: Option<&SNodeRef>) -> Option<SVal> {
        if let Some(field) = self.field(path, start) {
            return Some(field.value.clone());
        } else if let Some(func_ref) = self.func(path, start) {
            let mut parameters = Vec::new();
            if let Some(func) = SData::get::<SFunc>(&self.graph, &func_ref) {
                if let Some(params) = func.attributes.get("get") {
                    parameters.push(params.clone());
                }
            }
            if let Ok(res) = SFunc::call(&func_ref, "main", self, parameters, true, false) {
                self.clean("main");
                return Some(res);
            }
            self.clean("main");
        }
        None
    }


    /*****************************************************************************
     * Field interface.
     *****************************************************************************/
    
    /// Find a field in this document with a path.
    /// Path is dot '.' separated.
    pub fn field(&self, path: &str, start: Option<&SNodeRef>) -> Option<&SField> {
        SField::field(&self.graph, path, '.', start)
    }


    /*****************************************************************************
     * Docs interface.
     *****************************************************************************/
    
    /// Generate docs from a file.
    pub fn gitbook_md_from_file(file: &str) -> Result<String, String> {
        let mut doc = Self::default();
        doc.load_stof_docs();
        doc.load_format(Arc::new(Gitbook{}));
        let doc_res = doc.file_import("main", "stof", file, "stof", "");
        match doc_res {
            Ok(_) => {
                match doc.export_string("main", "gitbook", None) {
                    Ok(md) => {
                        Ok(md)
                    },
                    Err(err) => {
                        Err(err.to_string(&doc.graph))
                    }
                }
            },
            Err(err) => {
                let message = format!("{}: {} - {}", "parse error".red(), err.error_type.to_string().blue(), err.message.dimmed());
                Err(message)
            },
        }
    }


    #[cfg(feature = "markdown-lib")]
    /// gitbook HTML.
    pub fn gitbook_html_from_file(file: &str) -> Result<String, String> {
        let md = Self::gitbook_md_from_file(file)?;
        Ok(markdown::to_html(&md))
    }


    /*****************************************************************************
     * Function interface.
     *****************************************************************************/

    /// Run some Stof in a file.
    /// Will run all #[main] functions.
    /// Set 'throw' to true if you want this function to panic if tests fail.
    pub fn run_file(file: &str, throw: bool) {
        let doc_res = Self::file(file, "stof");
        let mut doc;
        match doc_res {
            Ok(dr) => doc = dr,
            Err(err) => {
                let message = format!("{}: {} - {}", "parse error".red(), err.error_type.to_string().blue(), err.message.dimmed());
                if throw {
                    panic!("{message}");
                } else {
                    println!("{message}");
                    return;
                }
            },
        }

        let res = doc.run(None, None);
        match res {
            Ok(_) => {
                // Don't do anything here
            },
            Err(err) => {
                if throw {
                    panic!("{err}");
                } else {
                    println!("{err}");
                }
            }
        }
    }

    #[cfg(feature = "async")]
    /// Run some Stof in a file.
    /// Will run all #[main] functions.
    /// Set 'throw' to true if you want this function to panic if tests fail.
    pub async fn run_file_async(file: &str, throw: bool) {
        let doc_res = Self::file(file, "stof");
        let doc;
        match doc_res {
            Ok(dr) => doc = dr,
            Err(err) => {
                let message = format!("{}: {} - {}", "parse error".red(), err.error_type.to_string().blue(), err.message.dimmed());
                if throw {
                    panic!("{message}");
                } else {
                    println!("{message}");
                    return;
                }
            },
        }

        let res = Self::run_blocking_async(doc, None, None).await;
        match res {
            Ok(_) => {
                // Don't do anything here
            },
            Err(err) => {
                if throw {
                    panic!("{err}");
                } else {
                    println!("{err}");
                }
            }
        }
    }

    #[cfg(feature = "async")]
    /// Async run a document, taking ownership of the document.
    /// Must be ran from within a tokio runtime.
    /// Creates a new blocking thread and does not prevent other async tasks from progressing.
    pub async fn run_blocking_async(mut doc: Self, context: Option<SNodeRef>, attribute: Option<String>) -> Result<(), String> {
        use tokio::runtime::Handle;
        let current = Handle::current();
        let res = current.spawn_blocking(move || {
            doc.run(context.as_ref(), attribute)
        }).await;
        match res {
            Ok(res) => res,
            Err(error) => Err(error.to_string()),
        }
    }

    /// Run the main functions on a node or within this document.
    /// Main functions are denoted with an #[main] attribute in the text format. This is the default attribute to run.
    pub fn run(&mut self, context: Option<&SNodeRef>, attribute: Option<String>) -> Result<(), String> {
        let mut search_attr = String::from("main");
        if let Some(att) = attribute {
            search_attr = att;
        }

        let functions;
        if context.is_some() {
            functions = SFunc::recursive_func_refs(&self.graph, context.unwrap());
        } else {
            functions = SFunc::all_funcs(&self.graph);
        }
        let mut errors = Vec::new();
        for func_ref in functions {
            if let Some(func) = SData::get::<SFunc>(&self.graph, &func_ref).cloned() {
                if let Some(attr_val) = func.attributes.get(&search_attr) {
                    let result;
                    if attr_val.is_empty() {
                        result = SFunc::call_internal(&func_ref, "main", self, vec![], true, &func.params, &func.statements, &func.rtype, false);
                    } else {
                        result = SFunc::call_internal(&func_ref, "main", self, vec![attr_val.clone()], true, &func.params, &func.statements, &func.rtype, false);
                    }
                    self.clean("main");
                    match result {
                        Ok(_) => {
                            // Nada... keep going!
                        },
                        Err(error) => {
                            let func_nodes = func_ref.nodes(&self.graph);
                            let func_path;
                            if func_nodes.len() > 0 {
                                func_path = func_nodes.first().unwrap().path(&self.graph);
                            } else {
                                func_path = String::from("<unknown>");
                            }

                            errors.push(format!("{} {} ...\n{}", func_path.italic().dimmed(), func.name.blue(), error.to_string(&self.graph)));
                        },
                    }
                }
            }
        }
        if errors.len() > 0 {
            let mut error = String::default();
            for err in errors {
                error.push_str(&format!("\n{} @ {}\n", "error".bold().red(), err));
            }
            Err(error)
        } else {
            Ok(())
        }
    }
    
    /// Find a function in this document with a path.
    /// Path is dot '.' separated.
    pub fn func(&self, path: &str, start: Option<&SNodeRef>) -> Option<SDataRef> {
        SFunc::func_ref(&self.graph, path, '.', start)
    }

    /// Call a function in this document with a path.
    pub fn call_func(&mut self, path: &str, start: Option<&SNodeRef>, params: Vec<SVal>) -> Result<SVal, SError> {
        if let Some(func_ref) = SFunc::func_ref(&self.graph, path, '.', start) {
            let res = SFunc::call(&func_ref, "main", self, params, true, true);
            self.clean("main");
            return res;
        }
        Err(SError::call("main", &self, &format!("did not find a function at the path '{}' to call", path)))
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
                let message = format!("{}: {} - {}", "parse error".red(), err.error_type.to_string().blue(), err.message.dimmed());
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

    #[cfg(feature = "async")]
    /// Test some Stof in a file.
    /// Will run all #[test] functions.
    /// Set 'throw' to true if you want this function to panic if tests fail.
    pub async fn test_file_async(file: &str, throw: bool) {
        let compile_start = SystemTime::now();
        let doc_res = Self::file(file, "stof");
        let dur = (compile_start.elapsed().unwrap().as_secs_f32() * 100.0).round() / 100.0;
        println!("{} {}s", "stof compiled in".dimmed(), dur);
        let doc;
        match doc_res {
            Ok(dr) => doc = dr,
            Err(err) => {
                let message = format!("{}: {} - {}", "parse error".red(), err.error_type.to_string().blue(), err.message.dimmed());
                if throw {
                    panic!("{message}");
                } else {
                    println!("{message}");
                    return;
                }
            },
        }

        let res = Self::test_blocking_async(doc, throw, None).await;
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
                let message = format!("{}: {} - {}", "parse error".red(), err.error_type.to_string().blue(), err.message.dimmed());
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

    #[cfg(feature = "async")]
    /// Async test a document, taking ownership of the document.
    /// Must be ran from within a tokio runtime.
    /// Creates a new blocking thread and does not prevent other async tasks from progressing.
    pub async fn test_blocking_async(mut doc: Self, throw: bool, context: Option<SNodeRef>) -> Result<String, String> {
        use tokio::runtime::Handle;
        let current = Handle::current();
        let res = current.spawn_blocking(move || {
            doc.run_tests(throw, context.as_ref())
        }).await;
        match res {
            Ok(res) => res,
            Err(error) => Err(error.to_string()),
        }
    }

    /// Run the test functions on a node or within this document, throwing an error if any fail.
    /// Test functions are denoted with a #[test(<result_expr_eq>)] attribute.
    pub fn run_tests(&mut self, throw: bool, context: Option<&SNodeRef>) -> Result<String, String> {
        let mut functions;
        if context.is_some() {
            functions = SFunc::recursive_func_refs(&self.graph, context.unwrap());
        } else {
            functions = SFunc::all_funcs(&self.graph);
        }
        functions.retain(|f| {
            if let Some(func) = SData::get::<SFunc>(&self.graph, f) {
                func.attributes.contains_key("test")
            } else {
                false
            }
        });
        let mut functions: Vec<SDataRef> = functions.into_iter().collect();
        functions.sort_by(|a, b| {
            a.first_path(&self.graph).cmp(&b.first_path(&self.graph))
        });

        let total = functions.len();
        println!("{} {} {}", "running".bold(), total, "Stof tests".bold());
        let mut failures = Vec::new();
        let mut profiles = Vec::new();
        let start = SystemTime::now();
        for func_ref in functions {
            if let Some(func) = SData::get::<SFunc>(&self.graph, &func_ref).cloned() {
                if let Some(res_test_val) = func.attributes.get("test") {
                    let silent = func.attributes.contains_key("silent");
                    let mut result = SFunc::call_internal(&func_ref, "main", self, vec![], true, &func.params, &func.statements, &func.rtype, false);

                    let func_nodes = func_ref.nodes(&self.graph);
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
                            result = Err(SError::custom("main", &self, "TestError", "expected function to throw an error"));
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
                                failures.push((func, format!("\t{}: {} @ {}: {}", "failed".bold().red(), name.blue(), func_path.italic().dimmed(), err_str.bold())));
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
                                            let res = SFunc::call_internal(&func_ref, "main", self, vec![], true, &func.params, &func.statements, &func.rtype, false);
                                            if res.is_err() { self.clean("main"); }
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

                            let err_str = err.to_string(&self.graph);
                            failures.push((func, format!("{}: {} @ {} ...\n{}", "failed".bold().red(), name.blue(), func_path.italic().dimmed(), err_str.bold())));
                            self.clean("main");
                        }
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
                output.push_str(&format!("{}\n\n", failure.1));
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
     * Split.
     *****************************************************************************/
    
    /// Split this document into another.
    pub fn split(&self) -> Self {
        let mut split = self.clone();
        split.graph.flush_all();
        split
    }

    /// Context split this document into another.
    /// When used well, can prevent a lot of the data from being cloned.
    pub fn context_split(&self, mut context: HashSet<SNodeRef>) -> Self {
        let mut split = Self::new(SGraph::default());
        
        // Make sure all of the prototype data exists
        if let Some(nref) = self.graph.node_ref("__stof__", None) {
            context.insert(nref);
        }
        split.graph = self.graph.context_clone(context);
        split.graph.flush_all();

        split.perms = self.perms.clone();
        split.types = self.types.clone();
        split.formats = self.formats.clone();
        split.libraries = self.libraries.clone();
        split.processes = self.processes.clone();
        split.env_compiled_paths = self.env_compiled_paths.clone();
        split
    }

    /// Join a document with this document.
    /// Validates all changed of other, as if it were 'split' from this document.
    pub fn join(&mut self, other: &mut Self) {
        self.graph.flush_join_graph(&mut other.graph);
        self.perms.merge(&other.perms);
        self.types.merge(&other.types);
    }


    /*****************************************************************************
     * Stof Parser/Runtime Interface.
     *****************************************************************************/

    /// Self pointer.
    pub(crate) fn self_ptr(&self, pid: &str) -> Option<SNodeRef> {
        if let Some(process) = self.processes.get(pid) {
            return process.self_ptr();
        }
        None
    }

    /// Push a node ref to the self stack of a process.
    pub(crate) fn push_self(&mut self, pid: &str, node: impl IntoNodeRef) {
        if let Some(process) = self.processes.get_mut(pid) {
            process.self_stack.push(node.node_ref());
        }
    }

    /// Pop a node ref from the self stack of a process.
    pub(crate) fn pop_self(&mut self, pid: &str) -> Option<SNodeRef> {
        if let Some(process) = self.processes.get_mut(pid) {
            process.self_stack.pop()
        } else {
            None
        }
    }

    /// New object pointer.
    pub(crate) fn new_obj_ptr(&self, pid: &str) -> Option<SNodeRef> {
        if let Some(process) = self.processes.get(pid) {
            return process.new_obj_ptr();
        }
        None
    }

    /// Push a node ref to the new object stack of a process.
    pub(crate) fn push_new_obj(&mut self, pid: &str, node: impl IntoNodeRef) {
        if let Some(process) = self.processes.get_mut(pid) {
            process.new_obj_stack.push(node.node_ref());
        }
    }

    /// Pop a node ref from the new object stack of a process.
    pub(crate) fn pop_new_obj(&mut self, pid: &str) -> Option<SNodeRef> {
        if let Some(process) = self.processes.get_mut(pid) {
            process.new_obj_stack.pop()
        } else {
            None
        }
    }

    /// New table.
    /// Returns the current table, replacing it with a new one.
    /// This happens for function calls.
    pub(crate) fn new_table(&mut self, pid: &str) -> SymbolTable {
        if let Some(processes) = self.processes.get_mut(pid) {
            return processes.new_table();
        }
        SymbolTable::default()
    }

    /// Push a new scope to the symbol table.
    pub(crate) fn new_scope(&mut self, pid: &str) {
        if let Some(process) = self.processes.get_mut(pid) {
            process.table.new_scope();
        }
    }

    /// Has a variable with this name in the current scope of the symbol table?
    pub(crate) fn has_var_with_name_in_current(&mut self, pid: &str, name: &str) -> bool {
        if let Some(process) = self.processes.get_mut(pid) {
            process.table.has_in_current(name)
        } else {
            false
        }
    }

    /// End a scope in the table.
    pub(crate) fn end_scope(&mut self, pid: &str) {
        if let Some(process) = self.processes.get_mut(pid) {
            process.table.end_scope();
        }
    }

    /// Set table.
    pub(crate) fn set_table(&mut self, pid: &str, table: SymbolTable) {
        if let Some(process) = self.processes.get_mut(pid) {
            process.set_table(table);
        }
    }

    /// Push a function reference to the call stack.
    pub(crate) fn push_call_stack(&mut self, pid: &str, dref: impl IntoDataRef) {
        if let Some(process) = self.processes.get_mut(pid) {
            process.push_call_stack(dref);
        }
    }

    /// Pop a function reference from the call stack.
    pub(crate) fn pop_call_stack(&mut self, pid: &str) {
        if let Some(process) = self.processes.get_mut(pid) {
            process.pop_call_stack();
        }
    }

    /// Add a variable to the current scope.
    pub(crate) fn add_variable<T>(&mut self, pid: &str, name: &str, value: T, is_const: bool) where T: Into<SVal> {
        if let Some(process) = self.processes.get_mut(pid) {
            process.add_variable(name, value, is_const);
        }
    }

    /// Set a variable.
    /// Will not add the variable if not already present.
    /// Sets current scope or above variables!
    pub(crate) fn set_variable(&mut self, pid: &str, name: &str, value: &SVal, force: bool) -> Result<bool, String> {
        if let Some(process) = self.processes.get_mut(pid) {
            process.set_variable(name, value, force)
        } else {
            Ok(false)
        }
    }

    /// Drop a symbol from the current scope.
    pub(crate) fn drop(&mut self, pid: &str, name: &str) -> Option<Symbol> {
        if let Some(processes) = self.processes.get_mut(pid) {
            processes.drop(name)
        } else {
            None
        }
    }

    /// Get a symbol from the current scope or above.
    pub(crate) fn get_symbol(&mut self, pid: &str, name: &str) -> Option<&Symbol> {
        if let Some(process) = self.processes.get_mut(pid) {
            process.get_symbol(name)
        } else {
            None
        }
    }

    /// Push a value onto the stack.
    pub(crate) fn push<T>(&mut self, pid: &str, value: T) where T: Into<SVal> {
        if let Some(process) = self.processes.get_mut(pid) {
            process.push(value);
        }
    }

    /// Pop a value from the stack.
    pub(crate) fn pop(&mut self, pid: &str) -> Option<SVal> {
        if let Some(process) = self.processes.get_mut(pid) {
            process.pop()
        } else {
            None
        }
    }

    /// Clean for scripting.
    pub(crate) fn clean(&mut self, pid: &str) {
        if let Some(processes) = self.processes.get_mut(pid) {
            processes.clean();
        }
    }

    /// Bubble control flow?
    pub(crate) fn bubble_control_flow(&self, pid: &str) -> bool {
        if let Some(process) = self.processes.get(pid) {
            process.bubble_control_flow > 0
        } else {
            false
        }
    }

    /// Increment bubble control flow.
    pub(crate) fn inc_bubble_control(&mut self, pid: &str) {
        if let Some(process) = self.processes.get_mut(pid) {
            process.bubble_control_flow += 1;
        }
    }

    /// Deincrement bubble control flow.
    pub(crate) fn dinc_bubble_control(&mut self, pid: &str) {
        if let Some(process) = self.processes.get_mut(pid) {
            process.bubble_control_flow -= 1;
        }
    }

    /// Funcstart bubble control.
    pub(crate) fn funcstart_bubble_control(&mut self, pid: &str, v: &mut u8) {
        if let Some(process) = self.processes.get_mut(pid) {
            swap(&mut process.bubble_control_flow, v);
        }
    }
}
