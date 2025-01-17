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
pub use parser::*;

use bytes::Bytes;

pub mod env;
pub use env::*;

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
        if let Ok(doc) = bincode::deserialize::<SDoc>(vec.as_ref()) {
            Ok(doc)
        } else {
            if let Ok(graph) = bincode::deserialize::<SGraph>(vec.as_ref()) {
                Ok(SDoc::new(graph))
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
    fn header_import(&self, _pid: &str, doc: &mut crate::SDoc, _content_type: &str, bytes: &mut bytes::Bytes, _as_name: &str) -> Result<()> {
        let mut new_doc = BSTOF::parse(&bytes)?;

        // Before we merge the types, we have to re-link the decids with the collisions that happened
        let collisions = doc.graph.get_collisions(&new_doc.graph);
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
                    for (_name, ctypes) in &mut new_doc.types.types {
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

        doc.graph.default_absorb_merge(new_doc.graph)?;
        doc.perms.merge(&new_doc.perms);
        doc.types.merge(&new_doc.types);
        Ok(())
    }

    /// File import.
    fn file_import(&self, pid: &str, doc: &mut crate::SDoc, format: &str, full_path: &str, _extension: &str, as_name: &str) -> Result<()> {
        let mut bytes = doc.fs_read_blob(pid, full_path)?;
        self.header_import(pid, doc, format, &mut bytes, as_name)
    }

    /// Export to binary form.
    fn export_bytes(&self, _pid: &str, doc: &SDoc, _node: Option<&crate::SNodeRef>) -> Result<Bytes> {
        BSTOF::doc_to_bytes(doc)
    }
}

/// Stof string format interface.
pub struct STOF;
impl STOF {
    /// Parse a STOF string into a new document.
    pub fn parse_new(pid: &str, stof: &str, env: Option<&mut StofEnv>) -> Result<SDoc> {
        let mut doc = SDoc::default();
        Self::parse(pid, &mut doc, stof, env)?;
        Ok(doc)
    }

    /// Parse a STOF string into a Stof graph.
    pub fn parse(pid: &str, doc: &mut SDoc, stof: &str, env: Option<&mut StofEnv>) -> Result<()> {
        let res;
        if let Some(env_ref) = env {
            env_ref.before_parse(doc);
            res = parse_internal(stof, doc, env_ref);
            env_ref.after_parse(doc);
        } else {
            let mut internal_env = StofEnv::new(pid, doc);
            internal_env.before_parse(doc);
            res = parse_internal(stof, doc, &mut internal_env);
            internal_env.after_parse(doc);
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
    fn header_import(&self, pid: &str, doc: &mut crate::SDoc, _content_type: &str, bytes: &mut bytes::Bytes, as_name: &str) -> Result<()> {
        let str = std::str::from_utf8(bytes.as_ref())?;
        self.string_import(pid, doc, str, as_name)
    }

    /// String import.
    fn string_import(&self, pid: &str, doc: &mut crate::SDoc, src: &str, as_name: &str) -> Result<()> {
        if doc.graph.roots.len() < 1 {
            doc.graph.insert_root("root");
        }
        let mut location = doc.graph.main_root().unwrap();
        if as_name.len() > 0 && as_name != "root" {
            if as_name.starts_with("self") || as_name.starts_with("super") {
                location = doc.graph.ensure_nodes(as_name, '.', true, doc.self_ptr(pid));
            } else {
                location = doc.graph.ensure_nodes(as_name, '.', true, None);
            }
        }


        let process = doc.processes.get(pid).cloned();
        let mut env = StofEnv::new_at_node(pid, doc, &location).unwrap();
        STOF::parse(pid, doc, src, Some(&mut env))?;

        // Undo the clean that happens...
        if let Some(process) = process {
            doc.processes.processes.insert(pid.to_owned(), process);
        }

        Ok(())
    }

    /// File import.
    fn file_import(&self, pid: &str, doc: &mut crate::SDoc, _format: &str, full_path: &str, _extension: &str, as_name: &str) -> Result<()> {
        let src = doc.fs_read_string(pid, full_path)?;

        if doc.graph.roots.len() < 1 {
            doc.graph.insert_root("root");
        }
        let mut location = doc.graph.main_root().unwrap();
        if as_name.len() > 0 && as_name != "root" {
            if as_name.starts_with("self") || as_name.starts_with("super") {
                location = doc.graph.ensure_nodes(as_name, '.', true, doc.self_ptr(pid));
            } else {
                location = doc.graph.ensure_nodes(as_name, '.', true, None);
            }
        }


        let process = doc.processes.get(pid).cloned();
        let mut env = StofEnv::new_at_node(pid, doc, &location).unwrap();
        
        let mut relative_path = full_path.trim().split('/').collect::<Vec<&str>>();
        relative_path.pop(); // pop the file name
        env.relative_import_path = relative_path.join("/");
        
        STOF::parse(pid, doc, &src, Some(&mut env))?;

        // Undo the clean that happens...
        if let Some(process) = process {
            doc.processes.processes.insert(pid.to_owned(), process);
        }

        Ok(())
    }
}
