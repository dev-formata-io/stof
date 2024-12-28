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

pub mod reader;
use reader::read_xml_to_graph;

pub mod writer;
use writer::node_write_xml;
use writer::write_xml;

use anyhow::Result;
use crate::{Format, IntoNodeRef, SDoc, SGraph};


/// Stof XML interface.
pub struct XML;
impl XML {
    /// Parse xml string into a new document.
    pub fn parse_new(xml: &str) -> Result<SDoc> {
        Ok(SDoc::new(Self::parse(xml)?))
    }

    /// Parse xml string.
    pub fn parse(xml: &str) -> Result<SGraph> {
        Ok(read_xml_to_graph(xml))
    }

    /// Stringify a graph as an XML string.
    pub fn stringify(graph: &SGraph) -> Result<String> {
        write_xml(graph)
    }

    /// Stringify a node as an XML string.
    pub fn stringify_node(graph: &SGraph, node: impl IntoNodeRef) -> Result<String> {
        node_write_xml(graph, &node.node_ref())
    }
}

impl Format for XML {
    /// Format getter.
    fn format(&self) -> String {
        "xml".to_string()
    }

    /// Content type.
    fn content_type(&self) -> String {
        "text/xml".to_string()
    }

    /// Header import.
    fn header_import(&self, pid: &str, doc: &mut crate::SDoc, _content_type: &str, bytes: &mut bytes::Bytes, as_name: &str) -> Result<()> {
        let str = std::str::from_utf8(bytes.as_ref())?;
        self.string_import(pid, doc, str, as_name)
    }

    /// String import.
    fn string_import(&self, pid: &str, doc: &mut crate::SDoc, src: &str, as_name: &str) -> Result<()> {
        let mut graph = XML::parse(src)?;
        if as_name.len() > 0 && as_name != "root" {
            let mut path = as_name.replace(".", "/");
            if as_name.starts_with("self") || as_name.starts_with("super") {
                if let Some(ptr) = doc.self_ptr(pid) {
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
    fn file_import(&self, pid: &str, doc: &mut crate::SDoc, _format: &str, full_path: &str, _extension: &str, as_name: &str) -> Result<()> {
        let src = doc.fs_read_string(pid, full_path)?;
        self.string_import(pid, doc, &src, as_name)
    }

    /// Export string.
    fn export_string(&self, _pid: &str, doc: &crate::SDoc, node: Option<&crate::SNodeRef>) -> Result<String> {
        if node.is_some() {
            XML::stringify_node(&doc.graph, node)
        } else {
            XML::stringify(&doc.graph)
        }
    }
}
