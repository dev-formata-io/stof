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

use std::fs;
use anyhow::{anyhow, Result};
use crate::{Format, IntoNodeRef, SDoc, SField, SGraph};


/// Stof TEXT interface.
pub struct TEXT;
impl TEXT {
    /// Parse some text into a new document.
    pub fn parse_new(text: &str) -> SDoc {
        SDoc::new(Self::parse(text))
    }

    /// Parse some text into a new graph.
    pub fn parse(text: &str) -> SGraph {
        let mut graph = SGraph::default();
        let root = graph.insert_root("root");
        SField::new_string(&mut graph, "text", text, &root);
        graph
    }

    /// Stringify.
    pub fn stringify(graph: &SGraph) -> Result<String> {
        if let Some(field) = SField::field(graph, "text", '.', graph.main_root().as_ref()) {
            return Ok(field.to_string());
        }
        Err(anyhow!("Stof TEXT Error: Did not find a 'text' field on the main root of this graph"))
    }

    /// Stringify node.
    pub fn stringify_node(graph: &SGraph, node: impl IntoNodeRef) -> Result<String> {
        if let Some(field) = SField::field(graph, "text", '.', Some(&node.node_ref())) {
            return Ok(field.to_string());
        }
        Err(anyhow!("Stof TEXT Error: Did not find a 'text' field on this node"))
    }
}

impl Format for TEXT {
    /// Format getter.
    fn format(&self) -> String {
        "text".to_string()
    }

    /// Content type.
    fn content_type(&self) -> String {
        "text/plain".to_string()
    }

    /// Header import.
    fn header_import(&self, doc: &mut crate::SDoc, _content_type: &str, bytes: &mut bytes::Bytes, as_name: &str) -> Result<()> {
        let str = std::str::from_utf8(bytes.as_ref())?;
        self.string_import(doc, str, as_name)
    }

    /// String import.
    fn string_import(&self, doc: &mut crate::SDoc, src: &str, as_name: &str) -> Result<()> {
        let mut graph = TEXT::parse(src);
        if as_name.len() > 0 && as_name != "root" {
            let mut path = as_name.replace(".", "/");
            if as_name.starts_with("self") || as_name.starts_with("super") {
                if let Some(ptr) = doc.self_ptr() {
                    path = format!("{}/{}", ptr.path(&doc.graph), path);
                }
            }

            // as_name is really a location, so ensure the nodes and move it there
            let mut loc_graph = SGraph::default();
            let loc = loc_graph.ensure_nodes(&path, '/', true, None);
            if let Some(main) = graph.main_root() {
                if let Some(main) = main.node(&graph) {
                    loc_graph.absorb_external_node(&graph, main, &loc);
                }
            }
            graph = loc_graph;
        }
        doc.graph.default_absorb_merge(graph)
    }

    /// File import.
    fn file_import(&self, doc: &mut crate::SDoc, _format: &str, full_path: &str, _extension: &str, as_name: &str) -> Result<()> {
        let src = fs::read_to_string(full_path)?;
        self.string_import(doc, &src, as_name)
    }

    /// Export string.
    fn export_string(&self, doc: &crate::SDoc, node: Option<&crate::SNodeRef>) -> Result<String> {
        if node.is_some() {
            TEXT::stringify_node(&doc.graph, node)
        } else {
            TEXT::stringify(&doc.graph)
        }
    }
}
