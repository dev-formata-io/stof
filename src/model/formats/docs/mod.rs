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

use std::{fs, path::PathBuf};
use imbl::OrdMap;
use crate::{model::{FS_LIB, Field, FieldDoc, Format, Func, FuncDoc, Graph, InnerDoc, LibFunc, NodeRef}, runtime::{Error, Val}};
pub mod libs;


#[derive(Debug)]
pub struct MdDocsFormat;
impl MdDocsFormat {
    /// Create documentation pages for a location (or the whole graph).
    pub fn docs(graph: &Graph, node: Option<NodeRef>) -> Vec<(String, String)> {
        let mut pages = vec![];
        pages.append(&mut Self::node_pages(graph, node));
        pages.append(&mut Self::lib_pages(graph));
        pages
    }


    /*****************************************************************************
     * Pages.
     *****************************************************************************/
    
    /// Node pages.
    /// Should show all fields and functions in a Stof structure - showing relationships at all levels.
    /// Then it should show all fields with doc comments, types, etc.
    /// Then it should show all functions with doc comments, signature, etc.
    fn node_pages(graph: &Graph, node: Option<NodeRef>) -> Vec<(String, String)> {
        let mut docs = Vec::new();
        if let Some(node) = node {
            if node.node_exists(graph) {
                let name = node.node_name(graph).unwrap();
                let out = Self::node_docs(graph, node);
                if out.len() > 0 {
                    docs.push((format!("{}_obj.md", name.as_ref()), out));
                }
            }
        } else {
            for node in &graph.roots {
                if node.node_exists(graph) {
                    let name = node.node_name(graph).unwrap();
                    let out = Self::node_docs(graph, node.clone());
                    if out.len() > 0 {
                        docs.push((format!("{}_obj.md", name.as_ref()), out));
                    }
                }
            }
        }
        docs
    }

    /// Library pages.
    /// One page per library.
    fn lib_pages(graph: &Graph) -> Vec<(String, String)> {
        let mut pages = vec![];
        for (lib, funcs) in &graph.libfuncs {
            let mut docs = String::default();
            if funcs.len() > 0 {
                // Library page header
                if let Some(libdoc) = graph.libdocs.get(lib) {
                    docs.push_str(libdoc);
                } else {
                    docs.push_str(&format!("# {lib}\n"));
                }

                // Docs for each function in the library
                docs.push('\n');
                for (_, func) in funcs.iter().collect::<OrdMap<&String, &LibFunc>>() {
                    docs.push_str(&func.docs);
                    docs.push_str("\n\n");
                }
            }
            if docs.len() > 0 { pages.push((format!("{lib}.md"), docs)); }
        }
        pages
    }


    /*****************************************************************************
     * Nodes.
     *****************************************************************************/
    
    /// Node documentation.
    ///
    /// Layout:
    /// - Node Inner Docs/header section
    /// - Fields & Funcs (structure as Stof only)
    /// - Field docs
    /// - Func docs
    fn node_docs(graph: &Graph, node: NodeRef) -> String {
        let mut docs = InnerDoc::docs(graph, &node);
        if docs.len() < 1 {
            docs.push_str(&format!("# {}", node.node_name(graph).unwrap().as_ref()));
        }
        docs.push('\n');

        let mut structure = String::from("```stof\n");
        let mut field_docs = String::default();
        let mut func_docs = String::default();
        Self::node_field_func_structure(graph, &node, &mut structure, &mut field_docs, &mut func_docs);
        structure.push_str("```");

        docs.push_str(&structure);
        docs.push('\n');
        docs.push_str(&field_docs);
        docs.push('\n');
        docs.push_str(&func_docs);

        docs
    }

    /// Field and function structure for a node.
    fn node_field_func_structure(graph: &Graph, node: &NodeRef, structure: &mut String, field_docs: &mut String, func_docs: &mut String) {
        // Note: fields have a mix of data refs and node refs for child fields
        let mut fields = OrdMap::default();
        let mut field_doc_map = FieldDoc::docs(graph, node);

        let mut funcs = OrdMap::default();
        let mut func_doc_map = FuncDoc::docs(graph, node);

        let path = node.node_path(graph, true).unwrap();
        let indent_count = i32::max(path.path.len() as i32 - 1, 0);
        let mut indent = String::new();
        for _ in 0..indent_count { indent.push('\t'); }
        let node_path = path.join(".");

        if let Some(node) = node.node(graph) {
            for (name, dref) in &node.data {
                if let Some(_field) = graph.get_stof_data::<Field>(dref) {
                    fields.insert(name.clone(), dref.clone());
                } else if let Some(_func) = graph.get_stof_data::<Func>(dref) {
                    funcs.insert(name.clone(), dref.clone());
                }
            }
            for child in &node.children {
                if let Some(child) = child.node(graph) {
                    if child.is_field() && !fields.contains_key(child.name.as_ref()) {
                        fields.insert(child.name.to_string(), child.id.clone());
                    }
                }
            }
        }

        for (name, id) in fields {
            if id.data_exists(graph) {
                // field field (could be a node too)
                if let Some(field) = graph.get_stof_data::<Field>(&id) {
                    // Field Doc
                    if let Some(doc) = field_doc_map.remove(&id) {
                        field_docs.push_str(&format!("# Field {node_path}.{name}\n"));
                        
                        if field.attributes.len() > 0 {
                            field_docs.push_str("## Attributes\n");
                            for (k, v) in &field.attributes {
                                if v.null() {
                                    field_docs.push_str(&format!("- #[{k}]\n"));
                                } else {
                                    field_docs.push_str(&format!("- #[{k}({})]\n", v.print(graph)));
                                }
                            }
                        }

                        field_docs.push_str(&doc);

                        if let Some(obj) = field.value.try_obj() {
                            if obj.child_of(graph, node) && &obj != node {
                                let inner = InnerDoc::docs(graph, &obj);
                                if inner.len() > 0 {
                                    field_docs.push('\n');
                                    for line in inner.split('\n') {
                                        if line.starts_with('#') { // make it a sub-comment
                                            field_docs.push_str(&format!("#{line}\n"));
                                        } else {
                                            field_docs.push_str(line);
                                            field_docs.push('\n');
                                        }
                                    }
                                }
                            }
                        }

                        field_docs.push_str("\n\n");
                    }

                    // Field structure
                    if field.attributes.len() > 0 {
                        structure.push('\n');
                        for (k, v) in &field.attributes {
                            if v.null() {
                                structure.push_str(&format!("{indent}#[{k}]\n"));
                            } else {
                                structure.push_str(&format!("{indent}#[{k}({})]\n", v.print(graph)));
                            }
                        }
                    }
                    if !field.value.mutable {
                        structure.push_str(&format!("{indent}const "));
                    } else {
                        structure.push_str(&indent);
                    }
                    structure.push_str(&field.value.val.read().spec_type(graph).rt_type_of(graph));
                    structure.push(' ');
                    structure.push_str(&name);
                    if let Some(obj) = field.value.try_obj() {
                        if obj.child_of(graph, node) && &obj != node {
                            structure.push_str(": {\n");
                            Self::node_field_func_structure(graph, &obj, structure, field_docs, func_docs);
                            structure.push_str(&format!("{indent}}}"));
                        }
                    }
                    structure.push('\n');
                }
            } else if let Some(child) = id.node(graph) {
                // Field docs
                let inner = InnerDoc::docs(graph, &id);
                if inner.len() > 0 {
                    field_docs.push_str(&format!("# Field {node_path}.{name}\n"));       
                    if child.attributes.len() > 0 {
                        field_docs.push_str("## Attributes\n");
                        for (k, v) in &child.attributes {
                            if v.null() {
                                field_docs.push_str(&format!("- #[{k}]\n"));
                            } else {
                                field_docs.push_str(&format!("- #[{k}({})]\n", v.print(graph)));
                            }
                        }
                    }
                    field_docs.push('\n');
                    for line in inner.split('\n') {
                        if line.starts_with('#') { // make it a sub-comment
                            field_docs.push_str(&format!("#{line}\n"));
                        } else {
                            field_docs.push_str(line);
                            field_docs.push('\n');
                        }
                    }
                    field_docs.push_str("\n\n");
                }

                // Field structure
                if child.attributes.len() > 0 {
                    structure.push('\n');
                    for (k, v) in &child.attributes {
                        if v.null() {
                            structure.push_str(&format!("{indent}#[{k}]\n"));
                        } else {
                            structure.push_str(&format!("{indent}#[{k}({})]\n", v.print(graph)));
                        }
                    }
                }
                structure.push_str(&indent);
                structure.push_str(&Val::Obj(id.clone()).spec_type(graph).rt_type_of(graph));
                structure.push(' ');
                structure.push_str(&name);
                structure.push_str(": {\n");
                Self::node_field_func_structure(graph, &id, structure, field_docs, func_docs);
                structure.push_str(&format!("{indent}}}\n"));
            }
        }
        for (name, fref) in funcs {
            if let Some(func) = graph.get_stof_data::<Func>(&fref) {
                // Func Doc
                if let Some(doc) = func_doc_map.remove(&fref) {
                    func_docs.push_str(&format!("# Func {node_path}.{name}\n"));
                    if func.attributes.len() > 0 {
                        func_docs.push_str("## Attributes\n");
                        for (k, v) in &func.attributes {
                            if v.null() {
                                func_docs.push_str(&format!("- #[{k}]\n"));
                            } else {
                                func_docs.push_str(&format!("- #[{k}({})]\n", v.print(graph)));
                            }
                        }
                    }
                    func_docs.push_str(&doc);
                    func_docs.push_str("\n\n");
                }

                // Func structure
                if func.attributes.len() > 0 {
                    structure.push('\n');
                    for (k, v) in &func.attributes {
                        if v.null() {
                            structure.push_str(&format!("{indent}#[{k}]\n"));
                        } else {
                            structure.push_str(&format!("{indent}#[{k}({})]\n", v.print(graph)));
                        }
                    }
                }
                structure.push_str(&format!("{indent}fn {name}("));
                let mut first = true;
                for param in &func.params {
                    if first {
                        first = false;
                    } else {
                        structure.push_str(", ");
                    }
                    if param.default.is_some() {
                        structure.push_str(&format!("{}?: {}", param.name.as_ref(), param.param_type.rt_type_of(graph)));
                    } else {
                        structure.push_str(&format!("{}: {}", param.name.as_ref(), param.param_type.rt_type_of(graph)));
                    }
                }
                structure.push_str(&format!(") -> {};\n", func.return_type.rt_type_of(graph)));
            }
        }
    }
}
impl Format for MdDocsFormat {
    fn identifiers(&self) -> Vec<String> {
        vec!["docs".into()]
    }
    fn content_type(&self) -> String {
        "stof/docs+md".into()
    }
    fn string_export(&self, graph: &Graph, _format: &str, node: Option<NodeRef>) -> Result<String, Error> {
        let docs = Self::docs(graph, node);
        let docs = docs
            .into_iter()
            .map(|d| d.1)
            .collect::<Vec<_>>()
            .join("\n");
        Ok(docs)
    }
    /// Takes a directory path and populates the directory with documentation for this graph & node.
    fn file_export(&self, graph: &Graph, _format: &str, path: &str, node: Option<NodeRef>) -> Result<(), Error> {
        if let Some(_lib) = graph.libfunc(&FS_LIB, "write") {
            let buf = PathBuf::from(path);
            if buf.is_dir() {
                // nada
            } else if let Err(err) = fs::create_dir_all(path) {
                return Err(Error::FormatFileExportFsError(err.to_string()));
            }

            for (name, content) in Self::docs(graph, node) {
                let path = format!("{path}/{name}");
                match fs::write(path, content) {
                    Ok(_) => {},
                    Err(err) => {
                        return Err(Error::FormatFileExportFsError(err.to_string()));
                    }
                }
            }
            return Ok(());
        }
        Err(Error::FormatFileExportNotAllowed)
    }
}
