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

pub mod import;
use import::parse_object_value;

pub mod export;
use export::toml_value_from_node;
use toml::{Table, Value};

use crate::{Format, IntoNodeRef, SDoc, SGraph};
use anyhow::{anyhow, Result};


/// Stof TOML interface.
pub struct TOML;
impl TOML {
    /// Parse a TOML string into a new document.
    pub fn parse_new(toml: &str) -> Result<SDoc> {
        Ok(SDoc::new(Self::parse(toml)?))
    }

    /// Parse a TOML string into a Stof graph.
    pub fn parse(toml: &str) -> Result<SGraph> {
        if let Ok(value) = toml.parse::<Table>() {
            let mut graph = SGraph::default();
            let root = graph.insert_root("root");
            parse_object_value(&mut graph, &root, Table::from(value));
            Ok(graph)
        } else {
            Err(anyhow!("Stof TOML Error: Unable to parse TOML string into table"))
        }
    }

    /// Export a graph as a TOML string.
    /// Exports the main root of the graph only.
    pub fn stringify(graph: &SGraph) -> Result<String> {
        if let Some(main) = graph.main_root() {
            let value = toml_value_from_node(graph, &main);
            if let Ok(toml) = toml::to_string(&Value::Table(value)) {
                return Ok(toml);
            }
            return Err(anyhow!("Stof TOML Error: Could not parse toml::Value into toml string"));
        }
        Err(anyhow!("Stof TOML Error: Did not find a main root to stringify"))
    }

    /// Export a node as a TOML string.
    pub fn stringify_node(graph: &SGraph, node: impl IntoNodeRef) -> Result<String> {
        let value = toml_value_from_node(graph, &node.node_ref());
        if let Ok(toml) = toml::to_string(&Value::Table(value)) {
            return Ok(toml);
        }
        Err(anyhow!("Stof TOML Error: Could not parse toml::Value into toml string"))
    }
}

impl Format for TOML {
    /// Format getter for TOML.
    fn format(&self) -> String {
        "toml".to_string()
    }

    /// Content type for TOML.
    fn content_type(&self) -> String {
        "text/toml".to_string()
    }

    /// Header import.
    fn header_import(&self, pid: &str, doc: &mut crate::SDoc, _content_type: &str, bytes: &mut bytes::Bytes, as_name: &str) -> Result<()> {
        let str = std::str::from_utf8(bytes.as_ref())?;
        self.string_import(pid, doc, str, as_name)
    }

    /// String import.
    fn string_import(&self, pid: &str, doc: &mut crate::SDoc, src: &str, as_name: &str) -> Result<()> {
        let mut graph = TOML::parse(src)?;
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
            TOML::stringify_node(&doc.graph, node)
        } else {
            TOML::stringify(&doc.graph)
        }
    }
}
