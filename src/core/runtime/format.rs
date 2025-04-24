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
use bytes::Bytes;
use crate::{lang::SError, SDoc, SNodeRef};


/// Stof Formats.
#[derive(Default, Clone)]
pub struct SFormats {
    pub formats: BTreeMap<String, Arc<dyn Format>>,
}
impl SFormats {
    /// Insert a format.
    pub fn insert(&mut self, format: Arc<dyn Format>) {
        for additional in format.additional_formats() {
            self.formats.insert(additional, format.clone());
        }
        self.formats.insert(format.format(), format);
    }

    /// Get a format.
    pub fn get(&self, format: &str) -> Option<Arc<dyn Format>> {
        self.formats.get(format).cloned()
    }

    /// Remove a format.
    pub fn remove(&mut self, format: &str) -> bool {
        self.formats.remove(format).is_some()
    }

    /// Remove all versions of a format.
    pub fn remove_all(&mut self, format: &str) -> bool {
        let mut to_remove = HashSet::new();
        to_remove.insert(format.to_string());
        if let Some(format) = self.formats.get(format) {
            to_remove.insert(format.format());
            for additional in format.additional_formats() {
                to_remove.insert(additional);
            }
        }
        let mut removed = false;
        for fmt in to_remove {
            removed = self.remove(&fmt) || removed;
        }
        removed
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
    pub fn header_import(&self, format: &str, pid: &str, doc: &mut SDoc, content_type: &str, bytes: &mut Bytes, as_name: &str) -> Result<(), SError> {
        // Check for an explicit format first!
        // If not found, search for the best match via content type.
        if let Some(format) = self.get(format) {
            return format.header_import(pid, doc, content_type, bytes, as_name);
        } else {
            // Search for a format with the content_type if any!
            let mut fallbacks = Vec::new();
            for (fmt, imp) in &self.formats {
                let ctt = imp.content_type();
                if ctt == content_type {
                    // Do this import - content type is an exact match
                    return imp.header_import(pid, doc, content_type, bytes, as_name);
                } else if content_type.contains(&ctt) || content_type.contains(fmt) {
                    fallbacks.push(imp);
                }
            }
            // If fallbacks, just use the first one that works
            for fallback in fallbacks {
                if let Ok(_) = fallback.header_import(pid, doc, content_type, bytes, as_name) {
                    return Ok(());
                }
            }
            // Finally, fallback onto the 'bytes' format!
            if format != "bytes" {
                return self.header_import("bytes", pid, doc, content_type, bytes, as_name);
            }
        }
        Err(SError::fmt(pid, &doc, format, "header import (bytes) - format not found"))
    }

    /// String import.
    pub fn string_import(&self, format: &str, pid: &str, doc: &mut SDoc, src: &str, as_name: &str) -> Result<(), SError> {
        if let Some(format) = self.get(format) {
            return format.string_import(pid, doc, src, as_name);
        }
        Err(SError::fmt(pid, &doc, format, "import string - format not found"))
    }

    /// File import.
    /// Stof Syntax: 'import <format> "<path>.<extension>" as <as_name>;'
    /// If <format> isn't supplied, "format" will be "extension".
    /// If <as_name> isn't supplied, the data should be imported into the current doc scope (or main root).
    pub fn file_import(&self, format: &str, pid: &str, doc: &mut SDoc, full_path: &str, extension: &str, as_name: &str) -> Result<(), SError> {
        if let Some(fmt) = self.get(format) {
            return fmt.file_import(pid, doc, format, full_path, extension, as_name);
        }
        Err(SError::fmt(pid, &doc, format, "import file - format not found"))
    }


    /*****************************************************************************
     * Export.
     *****************************************************************************/
    
    /// Export document into a string (human readable string).
    pub fn export_string(&self, format: &str, pid: &str, doc: &SDoc, node: Option<&SNodeRef>) -> Result<String, SError> {
        if let Some(format) = self.get(format) {
            return format.export_string(pid, doc, node);
        }
        Err(SError::fmt(pid, &doc, format, "export string - format not found"))
    }

    /// Export document into a string (minified string).
    pub fn export_min_string(&self, format: &str, pid: &str, doc: &SDoc, node: Option<&SNodeRef>) -> Result<String, SError> {
        if let Some(format) = self.get(format) {
            return format.export_min_string(pid, doc, node);
        }
        Err(SError::fmt(pid, &doc, format, "export min string - format not found"))
    }

    /// Export document into bytes.
    pub fn export_bytes(&self, format: &str, pid: &str, doc: &SDoc, node: Option<&SNodeRef>) -> Result<Bytes, SError> {
        if let Some(format) = self.get(format) {
            return format.export_bytes(pid, doc, node);
        }
        Err(SError::fmt(pid, &doc, format, "export bytes - format not found"))
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

    /// Additional extensions (formats) that this format is listed under.
    #[allow(unused)]
    fn additional_formats(&self) -> Vec<String> {
        vec![]
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
    fn header_import(&self, pid: &str, doc: &mut SDoc, content_type: &str, bytes: &mut Bytes, as_name: &str) -> Result<(), SError> {
        Err(SError::fmt(pid, &doc, &self.format(), "header import not implemented"))
    }

    /// String import.
    #[allow(unused)]
    fn string_import(&self, pid: &str, doc: &mut SDoc, src: &str, as_name: &str) -> Result<(), SError> {
        Err(SError::fmt(pid, &doc, &self.format(), "string import not implemented"))
    }

    /// File import.
    /// Stof Syntax: 'import <format> "<path>.<extension>" as <as_name>;'
    /// If <format> isn't supplied, "format" will be "extension".
    /// If <as_name> isn't supplied, the data should be imported into the current doc scope (or main root).
    #[allow(unused)]
    fn file_import(&self, pid: &str, doc: &mut SDoc, format: &str, full_path: &str, extension: &str, as_name: &str) -> Result<(), SError> {
        Err(SError::fmt(pid, &doc, &self.format(), "file import not implemented"))
    }


    /*****************************************************************************
     * Export Interface.
     *****************************************************************************/

    /// Export document into a string (human readable string).
    #[allow(unused)]
    fn export_string(&self, pid: &str, doc: &SDoc, node: Option<&SNodeRef>) -> Result<String, SError> {
        Err(SError::fmt(pid, &doc, &self.format(), "export string not implemented"))
    }

    /// Export document into a string (minified string).
    #[allow(unused)]
    fn export_min_string(&self, pid: &str, doc: &SDoc, node: Option<&SNodeRef>) -> Result<String, SError> {
        self.export_string(pid, doc, node)
    }

    /// Export document into bytes.
    #[allow(unused)]
    fn export_bytes(&self, pid: &str, doc: &SDoc, node: Option<&SNodeRef>) -> Result<Bytes, SError> {
        if let Ok(res) = self.export_min_string(pid, doc, node) {
            return Ok(Bytes::from(res));
        }
        Err(SError::fmt(pid, &doc, &self.format(), "export bytes not implemented"))
    }
}
