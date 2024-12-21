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

pub mod parser;
use bytes::Bytes;
pub use parser::*;

pub mod env;
pub use env::*;

use std::fs;
use anyhow::{anyhow, Result};
use crate::{Format, SDoc, SGraph};

#[cfg(test)]
mod tests;

/// Stof binary format interface.
/// BSTOF is a snappy compressed bincode serialized SDoc.
pub struct BSTOF;
impl BSTOF {
    /// Parse bytes into a new document.
    /// Bytes can either be a serialized SDoc or a serialized SGraph.
    pub fn parse(bytes: &Bytes) -> Result<SDoc> {
        let mut decoder = snap::raw::Decoder::new();
        let vec = decoder.decompress_vec(&bytes)?;
        if let Ok(mut graph) = bincode::deserialize::<SGraph>(vec.as_ref()) {
            graph.build_node_tries();
            Ok(SDoc::new(graph))
        } else {
            if let Ok(mut doc) = bincode::deserialize::<SDoc>(vec.as_ref()) {
                doc.graph.build_node_tries();
                Ok(doc)
            } else {
                Err(anyhow!(""))
            }
        }
    }

    /// To bytes.
    pub fn doc_to_bytes(doc: &SDoc) -> Result<Bytes> {
        if let Ok(bytes) = bincode::serialize(doc) {
            let mut encoder = snap::raw::Encoder::new();
            let bytes = encoder.compress_vec(&bytes)?;
            return Ok(bytes.into());
        }
        Err(anyhow!("Could not serialize document"))
    }

    /// To bytes.
    pub fn graph_to_bytes(graph: &SGraph) -> Result<Bytes> {
        if let Ok(bytes) = bincode::serialize(graph) {
            let mut encoder = snap::raw::Encoder::new();
            let bytes = encoder.compress_vec(&bytes)?;
            return Ok(bytes.into());
        }
        Err(anyhow!("Could not serialize graph"))
    }
}
impl Format for BSTOF {
    /// Format for BSTOF.
    fn format(&self) -> String {
        "bstof".to_string()
    }

    /// Content type for BSTOF.
    fn content_type(&self) -> String {
        "application/bstof".to_string()
    }

    /// Header import.
    fn header_import(&self, original: &mut SGraph, _content_type: &str, bytes: &mut bytes::Bytes, _as_name: &str) -> Result<()> {
        let mut new_doc = BSTOF::parse(&bytes)?;

        // Before we merge the types, we have to re-link the decids with the collisions that happened
        let collisions = original.get_collisions(&new_doc.graph);
        for (_, nodes) in &collisions.0 {
            let mut other_nodes = Vec::new();
            let mut self_nodes = Vec::new();
            for node in nodes {
                if node.exists(&new_doc.graph) { other_nodes.push(node.clone()); }
                else { self_nodes.push(node.clone()); }
            }
            if self_nodes.len() > 0 {
                let link_id = self_nodes.first().unwrap().id.clone();
                for other_node in other_nodes {
                    // Any decids on types in the new_doc have to be re-linked with a self_node...
                    for (_name, ctypes) in &mut new_doc.graph.types.types {
                        for ctype in ctypes {
                            if ctype.decid == other_node.id {
                                // This type was defined on a node that collides with a node already defined in doc
                                // In order for this type to still work, we need to re-point it as defined in a valid node
                                ctype.decid = link_id.clone();
                            }
                        }
                    }
                }
            }
        }

        original.perms.merge(&new_doc.graph.perms);
        original.types.merge(&new_doc.graph.types);
        original.default_absorb_merge(new_doc.graph)?;
        Ok(())
    }

    /// File import.
    fn file_import(&self, graph: &mut SGraph, format: &str, full_path: &str, _extension: &str, as_name: &str) -> Result<()> {
        let mut bytes = Bytes::from(fs::read(full_path)?);
        self.header_import(graph, format, &mut bytes, as_name)
    }

    /// Export to binary form.
    fn export_bytes(&self, graph: &SGraph, _node: Option<&crate::SNodeRef>) -> Result<Bytes> {
        BSTOF::graph_to_bytes(graph)
    }
}

/// Stof string format interface.
pub struct STOF;
impl STOF {
    /// Parse a STOF string into a new document.
    pub fn parse_new(stof: &str, env: Option<&mut StofEnv>) -> Result<SDoc> {
        let mut doc = SDoc::default();
        Self::parse(&mut doc.graph, stof, env)?;
        Ok(doc)
    }

    /// Parse a STOF string into a Stof graph.
    pub fn parse(graph: &mut SGraph, stof: &str, env: Option<&mut StofEnv>) -> Result<()> {
        let res;
        if let Some(env_ref) = env {
            env_ref.before_parse(graph);
            res = parse_internal(stof, graph, env_ref);
            env_ref.after_parse(graph);
        } else {
            let mut internal_env = StofEnv::new(graph);
            internal_env.before_parse(graph);
            res = parse_internal(stof, graph, &mut internal_env);
            internal_env.after_parse(graph);
        }
        res
    }
}
impl Format for STOF {
    /// Format for STOF.
    fn format(&self) -> String {
        "stof".to_string()
    }

    /// Content type for STOF.
    fn content_type(&self) -> String {
        "application/stof".to_string()
    }

    /// Header import.
    fn header_import(&self, graph: &mut SGraph, _content_type: &str, bytes: &mut bytes::Bytes, as_name: &str) -> Result<()> {
        let str = std::str::from_utf8(bytes.as_ref())?;
        self.string_import(graph, str, as_name)
    }

    /// String import.
    fn string_import(&self, graph: &mut SGraph, src: &str, as_name: &str) -> Result<()> {
        if graph.roots.len() < 1 {
            graph.insert_root("root");
        }
        let mut location = graph.main_root().unwrap();
        if as_name.len() > 0 && as_name != "root" {
            if as_name.starts_with("self") || as_name.starts_with("super") {
                location = graph.ensure_nodes(as_name, '.', true, graph.stack.self_ptr());
            } else {
                location = graph.ensure_nodes(as_name, '.', true, None);
            }
        }

        let stack = graph.stack.stack.clone();
        let self_stack = graph.stack.self_stack.clone();
        let table = graph.stack.table.clone();
        let bubble = graph.stack.bubble_control_flow;
        
        let mut env = StofEnv::new_at_node(graph, &location).unwrap();
        STOF::parse(graph, src, Some(&mut env))?;

        // Undo the clean that happens...
        graph.stack.bubble_control_flow = bubble;
        graph.stack.stack = stack;
        graph.stack.self_stack = self_stack;
        graph.stack.table = table;

        Ok(())
    }

    /// File import.
    fn file_import(&self, graph: &mut SGraph, _format: &str, full_path: &str, _extension: &str, as_name: &str) -> Result<()> {
        let src = fs::read_to_string(full_path)?;
        self.string_import(graph, &src, as_name)
    }
}
