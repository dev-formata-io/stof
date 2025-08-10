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

use bytes::Bytes;
use docx_rs::read_docx;
use crate::{model::{import::parse_json_object_value, Format, Graph, NodeRef}, runtime::Error};


#[derive(Debug)]
pub struct DocxFormat;
impl Format for DocxFormat {
    fn identifiers(&self) -> Vec<String> {
        vec!["docx".into()]
    }
    fn content_type(&self) -> String {
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string()
    }
    fn binary_import(&self, graph: &mut Graph, _format: &str, bytes: Bytes, node: Option<NodeRef>) -> Result<(), Error> {
        match read_docx(&bytes) {
            Ok(doc) => {
                match serde_json::to_value(doc) {
                    Ok(value) => {
                        let mut parse_node = graph.ensure_main_root();
                        if let Some(nd) = node {
                            parse_node = nd;
                        }
                        parse_json_object_value(graph, &parse_node, value);

                        // Import helper Stof functions
                        graph.string_import("stof", r#"

                            /**
                             * Get text out of this DocX document.
                             */
                            fn text(sep: str = ' ') -> str {
                                let text = '';
                                try for (const child in self.document.children) self.__docx_child__(&child, &text, &sep);
                                catch (message: str) throw('DocXTextError: ' + message);
                                text
                            }
                            fn __docx_child__(node: obj, text: str, sep: str) {
                                for (const child in node.data.children) {
                                    if (child.type != 'text') text.push(this(&child, &text, &sep));
                                    else text.push(child.data.text + sep);
                                }
                            }

                        "#, Some(parse_node))?;

                        Ok(())
                    },
                    Err(error) => {
                        Err(Error::DocXImport(error.to_string()))
                    }
                }
            },
            Err(error) => {
                Err(Error::DocXImport(error.to_string()))
            }
        }
    }
}
