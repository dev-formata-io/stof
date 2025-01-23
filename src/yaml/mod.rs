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

use serde_json::Value;
use crate::{json::JSON, lang::SError, Format, IntoNodeRef, SDoc, SGraph};


#[cfg(test)]
mod tests;


/// Stof YAML interface.
pub struct YAML;
impl YAML {
    /// Parse a YAML string into a new document.
    pub fn parse_new(yaml: &str) -> Result<SDoc, SError> {
        Ok(SDoc::new(Self::parse(yaml)?))
    }

    /// Parse a YAML string into a Stof graph.
    pub fn parse(yaml: &str) -> Result<SGraph, SError> {
        let res = serde_yaml::from_str::<Value>(yaml);
        match res {
            Ok(value) => {
                Ok(JSON::from_value(value))
            },
            Err(error) => {
                Err(SError::empty_fmt("yaml", &error.to_string()))
            }
        }
    }

    /// Export a graph as a YAML string.
    /// Exports the main root of the graph only.
    pub fn stringify(pid: &str, doc: &SDoc) -> Result<String, SError> {
        if let Ok(value) = JSON::to_value(pid, doc) {
            let res = serde_yaml::to_string(&value);
            return match res {
                Ok(value) => {
                    Ok(value)
                },
                Err(error) => {
                    Err(SError::fmt(pid, doc, "yaml", &error.to_string()))
                }
            };
        }
        Err(SError::fmt(pid, doc, "yaml", "could not stringify document"))
    }

    /// Export a node as a TOML string.
    pub fn stringify_node(pid: &str, doc: &SDoc, node: impl IntoNodeRef) -> Result<String, SError> {
        let res = serde_yaml::to_string(&JSON::to_node_value(&doc.graph, node));
        match res {
            Ok(value) => {
                Ok(value)
            },
            Err(error) => {
                Err(SError::fmt(pid, doc, "yaml", &error.to_string()))
            }
        }
    }
}

impl Format for YAML {
    /// Format getter.
    fn format(&self) -> String {
        "yaml".to_string()
    }

    /// Content type.
    fn content_type(&self) -> String {
        "application/yaml".to_string()
    }

    /// Header import.
    fn header_import(&self, pid: &str, doc: &mut crate::SDoc, _content_type: &str, bytes: &mut bytes::Bytes, as_name: &str) -> Result<(), SError> {
        let res = std::str::from_utf8(bytes.as_ref());
        match res {
            Ok(str) => {
                self.string_import(pid, doc, str, as_name)
            },
            Err(error) => {
                Err(SError::fmt(pid, &doc, "yaml", &error.to_string()))
            }
        }
    }

    /// String import.
    fn string_import(&self, pid: &str, doc: &mut crate::SDoc, src: &str, as_name: &str) -> Result<(), SError> {
        let mut graph = YAML::parse(src)?;
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
    fn file_import(&self, pid: &str, doc: &mut crate::SDoc, _format: &str, full_path: &str, _extension: &str, as_name: &str) -> Result<(), SError> {
        let src = doc.fs_read_string(pid, full_path)?;
        self.string_import(pid, doc, &src, as_name)
    }

    /// Export string.
    fn export_string(&self, pid: &str, doc: &crate::SDoc, node: Option<&crate::SNodeRef>) -> Result<String, SError> {
        if node.is_some() {
            YAML::stringify_node(pid, &doc, node)
        } else {
            YAML::stringify(pid, &doc)
        }
    }
}
