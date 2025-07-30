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

use std::fs;
use bytes::Bytes;
use crate::{model::{filesys::FS_LIB, Graph, NodeRef}, parser::context::ParseContext, runtime::Error};


/// Format.
pub trait Format: std::fmt::Debug + Send + Sync {
    /// Identifiers for this format.
    /// These will be the ways this format is referenced on the graph.
    fn identifiers(&self) -> Vec<String>;

    /// Content type for this format.
    fn content_type(&self) -> String;

    /// String import.
    #[allow(unused)]
    fn string_import(&self, graph: &mut Graph, format: &str, src: &str, node: Option<NodeRef>) -> Result<(), Error> {
        Err(Error::FormatStringImportNotImplemented(format.into()))
    }

    /// File import.
    #[allow(unused)]
    fn file_import(&self, graph: &mut Graph, format: &str, path: &str, node: Option<NodeRef>) -> Result<(), Error> {
        if let Some(_lib) = graph.libfunc(&FS_LIB, "read") {
            // Only allow reads if the FS library function is available
            match fs::read(path) {
                Ok(content) => {
                    return self.binary_import(graph, format, Bytes::from(content), node);
                },
                Err(error) => {
                    return Err(Error::FormatFileImportFsError(error.to_string()));
                }
            }
        }
        Err(Error::FormatFileImportNotAllowed)
    }

    /// Binary import.
    /// By default attempts to get bytes as UTF-8 string and uses string import.
    #[allow(unused)]
    fn binary_import(&self, graph: &mut Graph, format: &str, bytes: Bytes, node: Option<NodeRef>) -> Result<(), Error> {
        match std::str::from_utf8(bytes.as_ref()) {
            Ok(src) => {
                self.string_import(graph, format, src, node)
            },
            Err(_error) => {
                Err(Error::FormatBinaryImportUtf8Error)
            }
        }
    }

    /// String export.
    #[allow(unused)]
    fn string_export(&self, graph: &Graph, format: &str, node: Option<NodeRef>) -> Result<String, Error> {
        Err(Error::FormatStringExportNotImplemented(format.into()))
    }

    /// Binary export.
    #[allow(unused)]
    fn binary_export(&self, graph: &Graph, format: &str, node: Option<NodeRef>) -> Result<Bytes, Error> {
        match self.string_export(graph, format, node) {
            Ok(src) => {
                Ok(Bytes::from(src))
            },
            Err(error) => {
                Err(error)
            }
        }
    }

    #[allow(unused)]
    /// Parser import.
    fn parser_import(&self, format: &str, path: &str, context: &mut ParseContext) -> Result<(), Error> {
        let node = context.self_ptr();
        self.file_import(&mut context.graph, format, path, Some(node))
    }
}
