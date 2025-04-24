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

use docx_rs::read_docx;
use crate::{json::JSON, lang::SError, Format, SDoc, SGraph};


/// Stof DOCX interface.
/// Import only for now. TODO: library for export, etc.
pub struct DOCX;
impl DOCX {
    /// Parse DOCX bytes into a Stof graph via JSON.
    pub fn parse(docx: Vec<u8>) -> Result<SGraph, String> {
        let res = read_docx(&docx);
        match res {
            Ok(doc) => {
                if let Ok(value) = serde_json::to_value(doc) {
                    let mut doc = SDoc::new(JSON::from_value(value));
                    let _ = doc.string_import("main", "stof", r#"
                    fn text(sep: str = ' '): str {
                        let text = box('');
                        try {
                            for (child in self.document.children) {
                                self.__docx_read_children__(child, text, sep);
                            }
                        } catch (message: str) {
                            throw('DocxParseError', message);
                        }
                        return text;
                    }
                    fn __docx_read_children__(node: obj, text: Box<str>, sep: str) {
                        for (child in node.data.children) {
                            if (child.type != 'text') {
                                text += self.__docx_read_children__(child, text, sep);
                            } else {
                                text += child.data.text + sep;
                            }
                        }
                    }
                    "#, "");
                    return Ok(doc.graph);
                }
                Err("cannot parse docx file".into())
            },
            Err(error) => {
                Err(error.to_string())
            }
        }
    }
}


impl Format for DOCX {
    /// Format getter.
    fn format(&self) -> String {
        "docx".to_string()
    }

    /// Content type.
    fn content_type(&self) -> String {
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string()
    }

    /// Header import.
    fn header_import(&self, pid: &str, doc: &mut crate::SDoc, _content_type: &str, bytes: &mut bytes::Bytes, as_name: &str) -> Result<(), SError> {
        let res = DOCX::parse(bytes.to_vec());
        if res.is_err() {
            return Err(SError::fmt(pid, &doc, "docx", &format!("{}", res.err().unwrap().to_string())));
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

    /// File import.
    fn file_import(&self, pid: &str, doc: &mut crate::SDoc, format: &str, full_path: &str, _extension: &str, as_name: &str) -> Result<(), SError> {
        let mut bytes = doc.fs_read_blob(pid, full_path)?;
        self.header_import(pid, doc, format, &mut bytes, as_name)
    }
}


#[cfg(test)]
mod tests {
    use crate::SDoc;

    #[test]
    fn stof_docx_test_suite() {
        SDoc::test_file("src/docx/tests.stof", true);
    }
}
