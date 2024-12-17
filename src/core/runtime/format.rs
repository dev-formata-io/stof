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

use std::{collections::{BTreeMap, HashSet}, sync::Arc};
use anyhow::{anyhow, Result};
use bytes::Bytes;
use crate::{SDoc, SNodeRef};


/// Stof Formats.
#[derive(Default, Clone)]
pub struct SFormats {
    pub formats: BTreeMap<String, Arc<dyn Format>>,
}
impl SFormats {
    /// Insert a format.
    pub fn insert(&mut self, format: Arc<dyn Format>) {
        self.formats.insert(format.format(), format);
    }

    /// Get a format.
    pub fn get(&self, format: &str) -> Option<Arc<dyn Format>> {
        self.formats.get(format).cloned()
    }

    /// Available formats.
    pub fn available(&self) -> HashSet<String> {
        let mut formats = HashSet::new();
        for (format, _) in &self.formats {
            formats.insert(format.clone());
        }
        formats
    }

    /// Content type for a format.
    pub fn content_type(&self, format: &str) -> Option<String> {
        if let Some(format) = self.get(format) {
            return Some(format.content_type());
        }
        None
    }

    
    /*****************************************************************************
     * Import.
     *****************************************************************************/
    
    /// Header import (content type with bytes).
    /// Use an explicit "format" if you know it.
    /// Otherwise, supply a "content_type" for a more flexible format search.
    pub fn header_import(&self, format: &str, doc: &mut SDoc, content_type: &str, bytes: &mut Bytes, as_name: &str) -> Result<()> {
        // Check for an explicit format first!
        // If not found, search for the best match via content type.
        if let Some(format) = self.get(format) {
            return format.header_import(doc, content_type, bytes, as_name);
        } else {
            // Search for a format with the content_type if any!
            let mut fallbacks = Vec::new();
            for (fmt, imp) in &self.formats {
                let ctt = imp.content_type();
                if ctt == content_type {
                    // Do this import - content type is an exact match
                    return imp.header_import(doc, content_type, bytes, as_name);
                } else if content_type.contains(&ctt) || content_type.contains(fmt) {
                    fallbacks.push(imp);
                }
            }
            // If fallbacks, just use the first one that works
            for fallback in fallbacks {
                if let Ok(_) = fallback.header_import(doc, content_type, bytes, as_name) {
                    return Ok(());
                }
            }
            // Finally, fallback onto the 'bytes' format!
            if format != "bytes" {
                return self.header_import("bytes", doc, content_type, bytes, as_name);
            }
        }
        Err(anyhow!("Did not find a format to import with"))
    }

    /// String import.
    pub fn string_import(&self, format: &str, doc: &mut SDoc, src: &str, as_name: &str) -> Result<()> {
        if let Some(format) = self.get(format) {
            return format.string_import(doc, src, as_name);
        }
        Err(anyhow!("Did not find a format to import with"))
    }

    /// File import.
    /// Stof Syntax: 'import <format> "<path>.<extension>" as <as_name>;'
    /// If <format> isn't supplied, "format" will be "extension".
    /// If <as_name> isn't supplied, the data should be imported into the current doc scope (or main root).
    pub fn file_import(&self, format: &str, doc: &mut SDoc, full_path: &str, extension: &str, as_name: &str) -> Result<()> {
        if let Some(fmt) = self.get(format) {
            return fmt.file_import(doc, format, full_path, extension, as_name);
        }
        Err(anyhow!("Did not find a format to import with"))
    }


    /*****************************************************************************
     * Export.
     *****************************************************************************/
    
    /// Export document into a string (human readable string).
    pub fn export_string(&self, format: &str, doc: &SDoc, node: Option<&SNodeRef>) -> Result<String> {
        if let Some(format) = self.get(format) {
            return format.export_string(doc, node);
        }
        Err(anyhow!("Did not find a format to export with"))
    }

    /// Export document into a string (minified string).
    pub fn export_min_string(&self, format: &str, doc: &SDoc, node: Option<&SNodeRef>) -> Result<String> {
        if let Some(format) = self.get(format) {
            return format.export_min_string(doc, node);
        }
        Err(anyhow!("Did not find a format to export with"))
    }

    /// Export document into bytes.
    pub fn export_bytes(&self, format: &str, doc: &SDoc, node: Option<&SNodeRef>) -> Result<Bytes> {
        if let Some(format) = self.get(format) {
            return format.export_bytes(doc, node);
        }
        Err(anyhow!("Did not find a format to export with"))
    }
}


/// Stof Format.
/// Essentially an importer and exporter for Stof to use.
/// Create your own format, then add to your document to be able to import those file types.
pub trait Format: Send + Sync {
    /// Format identifier.
    /// Used to directly reference this Format.
    #[allow(unused)]
    fn format(&self) -> String {
        "text".to_string()
    }

    /// Content type.
    /// Used when sending documents over the wire, etc...
    #[allow(unused)]
    fn content_type(&self) -> String {
        "text/plain".to_owned()
    }


    /*****************************************************************************
     * Import Interface.
     *****************************************************************************/
    
    /// Content type import.
    #[allow(unused)]
    fn header_import(&self, doc: &mut SDoc, content_type: &str, bytes: &mut Bytes, as_name: &str) -> Result<()> {
        Err(anyhow!("Not implemented"))
    }

    /// String import.
    #[allow(unused)]
    fn string_import(&self, doc: &mut SDoc, src: &str, as_name: &str) -> Result<()> {
        Err(anyhow!("Not implemented"))
    }

    /// File import.
    /// Stof Syntax: 'import <format> "<path>.<extension>" as <as_name>;'
    /// If <format> isn't supplied, "format" will be "extension".
    /// If <as_name> isn't supplied, the data should be imported into the current doc scope (or main root).
    #[allow(unused)]
    fn file_import(&self, doc: &mut SDoc, format: &str, full_path: &str, extension: &str, as_name: &str) -> Result<()> {
        Err(anyhow!("Not implemented"))
    }


    /*****************************************************************************
     * Export Interface.
     *****************************************************************************/

    /// Export document into a string (human readable string).
    #[allow(unused)]
    fn export_string(&self, doc: &SDoc, node: Option<&SNodeRef>) -> Result<String> {
        Err(anyhow!("Not implemented"))
    }

    /// Export document into a string (minified string).
    #[allow(unused)]
    fn export_min_string(&self, doc: &SDoc, node: Option<&SNodeRef>) -> Result<String> {
        self.export_string(doc, node)
    }

    /// Export document into bytes.
    #[allow(unused)]
    fn export_bytes(&self, doc: &SDoc, node: Option<&SNodeRef>) -> Result<Bytes> {
        if let Ok(res) = self.export_min_string(doc, node) {
            return Ok(Bytes::from(res));
        }
        Err(anyhow!("Not implemented"))
    }
}
