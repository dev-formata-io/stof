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
use std::collections::HashMap;

pub use parser::*;

use bytes::Bytes;

pub mod env;
pub use env::*;

use crate::{lang::SError, Format, SDoc, SGraph, SNodeRef};

#[cfg(test)]
mod tests;


/// Stof binary format interface.
/// BSTOF is a snappy compressed bincode serialized SDoc.
pub struct BSTOF;
impl BSTOF {
    /// Parse bytes into a new document.
    /// Bytes can either be a serialized SDoc or a serialized SGraph.
    pub fn parse(bytes: &Bytes) -> Result<SDoc, SError> {
        let mut decoder = snap::raw::Decoder::new();
        let res = decoder.decompress_vec(&bytes);
        let vec;
        match res {
            Ok(res) => vec = res,
            Err(error) => return Err(SError::empty_fmt("bstof", &error.to_string())),
        }
        if let Ok(doc) = bincode::deserialize::<SDoc>(vec.as_ref()) {
            Ok(doc)
        } else {
            if let Ok(graph) = bincode::deserialize::<SGraph>(vec.as_ref()) {
                Ok(SDoc::new(graph))
            } else {
                Err(SError::empty_fmt("bstof", "failed to deserialize/parse bstof doc/graph"))
            }
        }
    }

    /// To bytes.
    pub fn doc_to_bytes(pid: &str, doc: &SDoc) -> Result<Bytes, SError> {
        if let Ok(bytes) = bincode::serialize(doc) {
            let mut encoder = snap::raw::Encoder::new();
            let res = encoder.compress_vec(&bytes);
            match res {
                Ok(bytes) => {
                    return Ok(bytes.into());
                },
                Err(error) => {
                    return Err(SError::fmt(pid, doc, "bstof", &format!("failed to compress document to bytes: {}", error.to_string())));
                }
            }
        }
        Err(SError::fmt(pid, doc, "bstof", "could not serialize document"))
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
    fn header_import(&self, pid: &str, doc: &mut crate::SDoc, _content_type: &str, bytes: &mut bytes::Bytes, as_name: &str) -> Result<(), SError> {
        let mut new_doc = BSTOF::parse(&bytes)?;

        if doc.graph.roots.len() < 1 {
            doc.graph.insert_root("root");
        }
        if new_doc.graph.roots.len() < 1 {
            new_doc.graph.insert_root("root");
        }

        // Map of IDs on new_doc (other graph) -> self IDs for mapping new types
        let mut decid_map = HashMap::new();

        // as name is used to re-arrange the main root of this new graph into a particular region of the existing graph
        if as_name.len() > 0 && as_name != "root" {
            let mut new_graph = SGraph::default();
            new_graph.insert_root("root");
            
            let location;
            if as_name.starts_with("self") || as_name.starts_with("super") {
                let path = format!("{}/{}", doc.self_ptr(pid).unwrap_or(doc.graph.main_root().unwrap()).path(&doc.graph), as_name.replace('.', "/"));
                location = new_graph.ensure_nodes(&path, '/', true, None);
            } else {
                location = new_graph.ensure_nodes(&as_name.replace('.', "/"), '/', true, None);
            }

            let mut absorbed = false;
            if let Some(node_ref) = new_doc.graph.main_root() {
                if let Some(node) = node_ref.node(&new_doc.graph) {
                    new_graph.absorb_external_node(&new_doc.graph, node, &location);
                    absorbed = true;
                    // need to map any old types defined on the old root to the new root
                    decid_map.insert(node_ref.id, location.id);
                }
            }
            if absorbed {
                // We've absorbed the main root under a new graph location, now insert all other roots...
                for root_index in 1..new_doc.graph.roots.len() {
                    if let Some(node) = new_doc.graph.roots[root_index].node(&new_doc.graph) {
                        new_graph.insert_external_node(&new_doc.graph, node, None, None); // keep as root and keep name
                    }
                }
                new_doc.graph = new_graph;
            }
        }

        // Before we merge the types, we have to re-link the decids with the collisions that will happen
        let collisions = doc.graph.get_collisions(&new_doc.graph);
        for index in 0..collisions.0.len() {
            if index < collisions.1.len() {
                decid_map.insert(collisions.1[index].id.clone(), collisions.0[index].id.clone());
            }
        }
        // Any decids on types in the new_doc have to be re-linked with a self_node...
        for (_name, ctypes) in &mut new_doc.types.types {
            for ctype in ctypes {
                if let Some(new_id) = decid_map.get(&ctype.decid) {
                    // This type was defined on a node that collides with a node already defined in doc
                    // In order for this type to still work, we need to re-point it as defined in a valid node
                    ctype.decid = new_id.clone();
                }
            }
        }

        doc.graph.default_absorb_merge(new_doc.graph)?;
        doc.perms.merge(&new_doc.perms);
        doc.types.merge(&new_doc.types);
        Ok(())
    }

    /// File import.
    fn file_import(&self, pid: &str, doc: &mut crate::SDoc, format: &str, full_path: &str, _extension: &str, as_name: &str) -> Result<(), SError> {
        let mut bytes = doc.fs_read_blob(pid, full_path)?;
        self.header_import(pid, doc, format, &mut bytes, as_name)
    }

    /// Export to binary form.
    fn export_bytes(&self, pid: &str, doc: &SDoc, node: Option<&SNodeRef>) -> Result<Bytes, SError> {
        if let Some(node) = node {
            let mut partial_doc = SDoc::default();
            
            // Create a new graph, inserting 'node' as the new 'root' object, interfaces and all.
            // Nodes keep their IDs and such, so references from these nodes to elsewhere in or under this node will stay valid.
            // Keep in mind, that references to other roots or nodes/interfaces above will be broken!
            if let Some(node) = node.node(&doc.graph) {
                let mut graph = SGraph::default();
                graph.insert_external_node(&doc.graph, node, None, Some("root".into()));
                partial_doc.graph = graph;
            }
            
            partial_doc.types = doc.types.declared_types_for(node, &doc.graph);
            partial_doc.perms.merge(&doc.perms);
            BSTOF::doc_to_bytes(pid, &partial_doc)
        } else {
            BSTOF::doc_to_bytes(pid, doc)
        }
    }
}

/// Stof string format interface.
pub struct STOF;
impl STOF {
    /// Parse a STOF string into a new document.
    pub fn parse_new(pid: &str, stof: &str, env: Option<&mut StofEnv>) -> Result<SDoc, SError> {
        let mut doc = SDoc::default();
        Self::parse(pid, &mut doc, stof, env)?;
        Ok(doc)
    }

    /// Parse a STOF string into a Stof graph.
    pub fn parse(pid: &str, doc: &mut SDoc, stof: &str, env: Option<&mut StofEnv>) -> Result<(), SError> {
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
    fn header_import(&self, pid: &str, doc: &mut crate::SDoc, _content_type: &str, bytes: &mut bytes::Bytes, as_name: &str) -> Result<(), SError> {
        let res = std::str::from_utf8(bytes.as_ref());
        match res {
            Ok(str) => {
                self.string_import(pid, doc, str, as_name)
            },
            Err(error) => {
                Err(SError::fmt(pid, &doc, "stof", &error.to_string()))
            }
        }
    }

    /// String import.
    fn string_import(&self, pid: &str, doc: &mut crate::SDoc, src: &str, as_name: &str) -> Result<(), SError> {
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
    fn file_import(&self, pid: &str, doc: &mut crate::SDoc, _format: &str, full_path: &str, _extension: &str, as_name: &str) -> Result<(), SError> {
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
