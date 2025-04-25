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

pub mod library;
use std::{fmt::Debug, ops::Deref, path::Path};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use bytes::Bytes;
use lopdf::{xobject::PdfImage, Document};
use serde::{Deserialize, Serialize};
use crate::{lang::SError, Data, Format, IntoNodeRef, SData, SDoc, SField, SGraph, SNodeRef, SVal};


/// PDF.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SPDF {
    #[serde(deserialize_with = "deserialize_doc_field")]
    #[serde(serialize_with = "serialize_doc_field")]
    pub doc: Document,
}

/// Impl Data for PDF.
/// Not included by default, so not a "core_data" type.
/// Default library name will be the Serde Tagname (if exists).
#[typetag::serde(name = "PDF")]
impl Data for SPDF {}

/// Implement PDF.
impl SPDF {
    /// Create a PDF from a file.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        if let Ok(doc) = Document::load(path) {
            return Ok(Self {
                doc,
            });
        }
        Err("could not load the PDF from path".into())
    }

    /// Create a PDF from bytes.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, String> {
        if let Ok(doc) = Document::load_mem(&bytes) {
            return Ok(Self {
                doc,
            });
        }
        Err("could not load PDF from bytes".into())
    }


    /*****************************************************************************
     * PDF Helpers.
     *****************************************************************************/
    
    /// Extract single page text.
    pub fn extract_single_page_text(&self, page: u32) -> Option<String> {
        if let Ok(text) = self.doc.extract_text(&[page]) {
            return Some(text);
        }
        None
    }
    
    /// Extract all text from this PDF document per page.
    pub fn extract_page_text(&self) -> Vec<String> {
        let pages = self.doc.get_pages();
        let mut texts = Vec::new();
        for (i, _) in pages.iter().enumerate() {
            let text = self.doc.extract_text(&[(i + 1) as u32]);
            texts.push(text.unwrap_or_default());
        }
        texts
    }

    /// Extract all text from all pages.
    pub fn extract_text(&self) -> String {
        let mut text = String::default();
        let mut first = true;
        for page in self.extract_page_text() {
            if page.len() > 0 {
                if !first {
                    text.push('\n');
                } else {
                    first = false;
                }
                text.push_str(&page);
            }
        }
        text
    }

    /// Extract single page images.
    pub fn extract_single_page_images(&self, page: u32) -> Option<Vec<PdfImage>> {
        for (i, (_, id)) in self.doc.get_pages().into_iter().enumerate() {
            if (i + 1) as u32 == page {
                if let Ok(page_images) = self.doc.get_page_images(id) {
                    return Some(page_images);
                }
                return None;
            }
        }
        None
    }
    
    /// Extract all images from all pages.
    pub fn extract_images(&self) -> Vec<PdfImage> {
        let pages = self.doc.get_pages();
        let mut images = Vec::new();
        for (_number, id) in pages.into_iter() {
            if let Ok(mut page_images) = self.doc.get_page_images(id) {
                images.append(&mut page_images);
            }
        }
        images
    }

    /*****************************************************************************
     * Format Helpers.
     *****************************************************************************/
    
    /// Parse into a new graph.
    pub fn parse(bytes: &Bytes) -> Result<SGraph, String> {
        let mut graph = SGraph::default();
        let root = graph.insert_root("root");

        let pdf = SPDF::from_bytes(bytes.to_vec())?;
        let dref = SData::insert_new(&mut graph, &root, Box::new(pdf)).unwrap();

        let field = SField::new("pdf", SVal::Data(dref));
        SData::insert_new(&mut graph, &root, Box::new(field));
        
        Ok(graph)
    }

    /// To bytes.
    pub fn to_bytes(pid: &str, doc: &SDoc) -> Result<Bytes, SError> {
        if let Some(field) = SField::field(&doc.graph, "pdf", '.', doc.graph.main_root().as_ref()) {
            match &field.value {
                SVal::Data(dref) => {
                    if let Some(pdf) = SData::get::<SPDF>(&doc.graph, dref) {
                        let mut data: Vec<u8> = Vec::new();
                        let mut mutable = pdf.doc.clone();
                        let _ = mutable.save_to(&mut data);
                        return Ok(Bytes::from(data));
                    }
                },
                SVal::Blob(bytes) => return Ok(Bytes::from(bytes.clone())),
                SVal::Boxed(val) => {
                    let val = val.lock().unwrap();
                    let val = val.deref();
                    match val {
                        SVal::Blob(bytes) => return Ok(Bytes::from(bytes.clone())),
                        SVal::Data(dref) => {
                            if let Some(pdf) = SData::get::<SPDF>(&doc.graph, dref) {
                                let mut data: Vec<u8> = Vec::new();
                                let mut mutable = pdf.doc.clone();
                                let _ = mutable.save_to(&mut data);
                                return Ok(Bytes::from(data));
                            }
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        }
        Err(SError::fmt(pid, doc, "pdf", "did not find pdf on the main root of this graph"))
    }

    /// Node to bytes.
    pub fn node_to_bytes(pid: &str, doc: &SDoc, node: impl IntoNodeRef) -> Result<Bytes, SError> {
        if let Some(field) = SField::field(&doc.graph, "pdf", '.', Some(&node.node_ref())) {
            match &field.value {
                SVal::Data(dref) => {
                    if let Some(pdf) = SData::get::<SPDF>(&doc.graph, dref) {
                        let mut data: Vec<u8> = Vec::new();
                        let mut mutable = pdf.doc.clone();
                        let _ = mutable.save_to(&mut data);
                        return Ok(Bytes::from(data));
                    }
                },
                SVal::Blob(bytes) => return Ok(Bytes::from(bytes.clone())),
                SVal::Boxed(val) => {
                    let val = val.lock().unwrap();
                    let val = val.deref();
                    match val {
                        SVal::Blob(bytes) => return Ok(Bytes::from(bytes.clone())),
                        SVal::Data(dref) => {
                            if let Some(pdf) = SData::get::<SPDF>(&doc.graph, dref) {
                                let mut data: Vec<u8> = Vec::new();
                                let mut mutable = pdf.doc.clone();
                                let _ = mutable.save_to(&mut data);
                                return Ok(Bytes::from(data));
                            }
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        }
        Err(SError::fmt(pid, doc, "pdf", "did not find pdf on the requested node"))
    }

    /// Header import.
    pub fn header_import(pid: &str, doc: &mut SDoc, bytes: &mut Bytes, as_name: &str) -> Result<(), SError> {
        let res = SPDF::parse(bytes);
        if res.is_err() {
            return Err(SError::fmt(pid, &doc, "pdf", "could not parse bytes into a graph (header import)"));
        }
        let mut graph = res.unwrap();
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

    /// String import (base64 pdf string).
    pub fn string_import(pid: &str, doc: &mut SDoc, src: &str, as_name: &str) -> Result<(), SError> {
        if let Ok(bytes) = STANDARD.decode(src) {
            let mut bytes = Bytes::from(bytes);
            Self::header_import(pid, doc, &mut bytes, as_name)
        } else {
            Err(SError::fmt(pid, &doc, "pdf", "failed to decode base64 pdf string"))
        }
    }

    /// Export bytes.
    pub fn export_bytes(pid: &str, doc: &SDoc, node: Option<&SNodeRef>) -> Result<Bytes, SError> {
        if node.is_some() {
            Self::node_to_bytes(pid, doc, node)
        } else {
            Self::to_bytes(pid, &doc)
        }
    }

    /// Export PDF as base64 string.
    pub fn export_string(pid: &str, doc: &SDoc, node: Option<&SNodeRef>) -> Result<String, SError> {
        let bytes = Self::export_bytes(pid, doc, node)?;
        let res = STANDARD.encode(&bytes);
        Ok(res)
    }
}


/// Custom serialize for doc field.
fn serialize_doc_field<S>(doc: &Document, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
    let mut data: Vec<u8> = Vec::new();
    let mut mutable = doc.clone();
    let _ = mutable.save_to(&mut data);
    data.serialize(serializer)
}


/// Custom deserialize for data field.
fn deserialize_doc_field<'de, D>(deserializer: D) -> Result<Document, D::Error>
    where
        D: serde::Deserializer<'de> {
    let data: Vec<u8> = Deserialize::deserialize(deserializer)?;
    if let Ok(doc) = Document::load_mem(&data) {
        Ok(doc)
    } else {
        Err(serde::de::Error::custom("could not deserialize Stof PDF document"))
    }
}


/// PDF Format.
pub struct PDF;
impl Format for PDF {
    /// Format getter.
    fn format(&self) -> String {
        "pdf".to_string()
    }

    /// Content type.
    fn content_type(&self) -> String {
        "application/pdf".to_string()
    }

    /// Header import.
    fn header_import(&self, pid: &str, doc: &mut SDoc, _content_type: &str, bytes: &mut Bytes, as_name: &str) -> Result<(), SError> {
        SPDF::header_import(pid, doc, bytes, as_name)
    }

    /// String import.
    fn string_import(&self, pid: &str, doc: &mut SDoc, src: &str, as_name: &str) -> Result<(), SError> {
        SPDF::string_import(pid, doc, src, as_name)
    }

    /// File import.
    fn file_import(&self, pid: &str, doc: &mut SDoc, format: &str, full_path: &str, _extension: &str, as_name: &str) -> Result<(), SError> {
        let mut bytes = doc.fs_read_blob(pid, full_path)?;
        self.header_import(pid, doc, format, &mut bytes, as_name)
    }

    /// Export bytes.
    fn export_bytes(&self, pid: &str, doc: &SDoc, node: Option<&SNodeRef>) -> Result<Bytes, SError> {
        SPDF::export_bytes(pid, doc, node)
    }

    /// Export string.
    fn export_string(&self, pid: &str, doc: &SDoc, node: Option<&SNodeRef>) -> Result<String, SError> {
        SPDF::export_string(pid, doc, node)
    }
}


#[cfg(test)]
mod tests {
    use crate::SDoc;

    #[test]
    fn stof_pdf_test_suite() {
        SDoc::test_file("src/pdf/tests.stof", true);
    }
}
