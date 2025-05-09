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

use core::str;
use std::ops::Deref;
use bytes::Bytes;
use crate::{lang::SError, Format, IntoNodeRef, SData, SDoc, SField, SGraph, SVal};


/// Stof BYTES interface.
pub struct BYTES;
impl BYTES {
    /// Parse into a new document.
    pub fn parse_new(bytes: &Bytes) -> SDoc {
        SDoc::new(Self::parse(bytes))
    }

    /// Parse into a new graph.
    pub fn parse(bytes: &Bytes) -> SGraph {
        let mut graph = SGraph::default();
        let root = graph.insert_root("root");
        let field = SField::new("bytes", SVal::Blob(bytes.to_vec()));
        SData::insert_new(&mut graph, &root, Box::new(field));
        graph
    }

    /// To bytes.
    pub fn to_bytes(pid: &str, doc: &SDoc) -> Result<Bytes, SError> {
        if let Some(field) = SField::field(&doc.graph, "bytes", '.', doc.graph.main_root().as_ref()) {
            match &field.value {
                SVal::Blob(bytes) => return Ok(Bytes::from(bytes.clone())),
                SVal::Boxed(val) => {
                    let val = val.lock().unwrap();
                    let val = val.deref();
                    match val {
                        SVal::Blob(bytes) => return Ok(Bytes::from(bytes.clone())),
                        _ => {}
                    }
                },
                _ => {}
            }
        }
        Err(SError::fmt(pid, doc, "bytes", "did not find a 'bytes' field on the main root of this graph"))
    }

    /// Node to bytes.
    pub fn node_to_bytes(pid: &str, doc: &SDoc, node: impl IntoNodeRef) -> Result<Bytes, SError> {
        if let Some(field) = SField::field(&doc.graph, "bytes", '.', Some(&node.node_ref())) {
            match &field.value {
                SVal::Blob(bytes) => return Ok(Bytes::from(bytes.clone())),
                SVal::Boxed(val) => {
                    let val = val.lock().unwrap();
                    let val = val.deref();
                    match val {
                        SVal::Blob(bytes) => return Ok(Bytes::from(bytes.clone())),
                        _ => {}
                    }
                },
                _ => {}
            }
        }
        Err(SError::fmt(pid, doc, "bytes", "did not find a 'bytes' field on this node"))
    }
}

impl Format for BYTES {
    /// Format getter.
    fn format(&self) -> String {
        "bytes".to_string()
    }

    /// Content type.
    fn content_type(&self) -> String {
        "application/octet-stream".to_string()
    }

    /// Header import.
    fn header_import(&self, pid: &str, doc: &mut crate::SDoc, _content_type: &str, bytes: &mut bytes::Bytes, as_name: &str) -> Result<(), SError> {
        let mut graph = BYTES::parse(bytes);
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

    /// String import.
    fn string_import(&self, pid: &str, doc: &mut crate::SDoc, src: &str, as_name: &str) -> Result<(), SError> {
        let mut bytes = Bytes::from(src.to_string());
        self.header_import(pid, doc, "bytes", &mut bytes, as_name)
    }

    /// File import.
    fn file_import(&self, pid: &str, doc: &mut crate::SDoc, format: &str, full_path: &str, _extension: &str, as_name: &str) -> Result<(), SError> {
        let mut bytes = doc.fs_read_blob(pid, full_path)?;
        self.header_import(pid, doc, format, &mut bytes, as_name)
    }

    /// Export bytes.
    fn export_bytes(&self, pid: &str, doc: &SDoc, node: Option<&crate::SNodeRef>) -> Result<Bytes, SError> {
        if node.is_some() {
            BYTES::node_to_bytes(pid, doc, node)
        } else {
            BYTES::to_bytes(pid, &doc)
        }
    }

    /// Export string.
    /// Tries exporting these bytes as a utf-8 string.
    fn export_string(&self, pid: &str, doc: &SDoc, node: Option<&crate::SNodeRef>) -> Result<String, SError> {
        let bytes = self.export_bytes(pid, doc, node)?;
        let res = str::from_utf8(&bytes);
        match res {
            Ok(val) => {
                Ok(val.to_owned())
            },
            Err(error) => {
                Err(SError::fmt(pid, &doc, "bytes", &error.to_string()))
            }
        }
    }
}
