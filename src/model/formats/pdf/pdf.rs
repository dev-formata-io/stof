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

use std::path::Path;
use lopdf::{xobject::PdfImage, Document};
use serde::{Deserialize, Serialize};
use crate::model::StofData;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pdf {
    #[serde(deserialize_with = "deserialize_doc_field")]
    #[serde(serialize_with = "serialize_doc_field")]
    pub doc: Document,
}

#[typetag::serde(name = "Pdf")]
impl StofData for Pdf {}

impl Pdf {
    #[allow(unused)]
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

    #[allow(unused)]
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

    #[allow(unused)]
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
