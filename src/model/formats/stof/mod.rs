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
use rustc_hash::FxHashSet;

use crate::{model::{filesys::FS_LIB, Format, Graph, NodeRef}, parser::{context::ParseContext, doc::document}, runtime::Error};


#[derive(Debug, Default)]
/// Stof language format.
pub struct StofFormat;
impl Format for StofFormat {
    fn identifiers(&self) -> Vec<String> {
        vec!["stof".into()]
    }
    fn content_type(&self) -> String {
        "application/stof".into()
    }
    fn string_import(&self, graph: &mut Graph, _format: &str, src: &str, node: Option<NodeRef>) -> Result<(), Error> {
        let mut context = ParseContext::new(graph);
        if let Some(node) = node {
            context.push_self_node(node);
        }
        document(src, &mut context)?;
        Ok(())
    }
    fn parser_import(&self, _format: &str, path: &str, context: &mut ParseContext) -> Result<(), Error> {
        if let Some(_lib) = context.graph.libfunc(&FS_LIB, "read") {
            match fs::read(path) {
                Ok(content) => {
                    match std::str::from_utf8(&content) {
                        Ok(src) => {
                            document(src, context)?;
                            return Ok(());
                        },
                        Err(_error) => {
                            return Err(Error::FormatBinaryImportUtf8Error);
                        }
                    }
                },
                Err(error) => {
                    return Err(Error::FormatFileImportFsError(error.to_string()));
                }
            }
        }
        Err(Error::FormatFileImportNotAllowed)
    }
}


#[derive(Debug)]
/// .bstf format (serialized graph)
pub struct BstfFormat;
impl Format for BstfFormat {
    fn identifiers(&self) -> Vec<String> {
        vec!["bstf".into()]
    }
    fn content_type(&self) -> String {
        "application/bstf".into()
    }
    fn binary_export(&self, graph: &Graph, _format: &str, node: Option<NodeRef>) -> Result<bytes::Bytes, Error> {
        if let Some(node) = node {
            let mut context = FxHashSet::default();
            context.insert(node);
            let graph = graph.context_clone(context);
            match bincode::serialize(&graph) {
                Ok(bytes) => {
                    Ok(bytes.into())
                },
                Err(error) => {
                    Err(Error::BSTFExport(error.to_string()))
                }
            }
        } else {
            match bincode::serialize(graph) {
                Ok(bytes) => {
                    Ok(bytes.into())
                },
                Err(error) => {
                    Err(Error::BSTFExport(error.to_string()))
                }
            }
        }
    }
    fn binary_import(&self, graph: &mut Graph, _format: &str, bytes: bytes::Bytes, node: Option<NodeRef>) -> Result<(), Error> {
        match bincode::deserialize::<Graph>(bytes.as_ref()) {
            Ok(mut imported) => {
                // Insert types
                for (k, v) in &imported.typemap {
                    graph.typemap.insert(k.clone(), v.clone());
                }

                if let Some(node) = node {
                    // absorb the main root onto this graph node
                    if let Some(main) = imported.ensure_main_root().node(&imported) {
                        graph.absorb_external_node(&imported, main, &node, true);
                    }
                } else {
                    // insert all roots into the graph
                    for import_root in &imported.roots {
                        if let Some(import_root_name) = import_root.node_name(&imported) {
                            if let Some(existing_root) = graph.find_root_named(&import_root_name) {
                                if let Some(import_root_node) = import_root.node(&imported) {
                                    graph.absorb_external_node(&imported, import_root_node, &existing_root, true);
                                }
                            } else if let Some(import_root_node) = import_root.node(&imported) {
                                graph.insert_external_node(&imported, import_root_node, None, None, None);
                            }
                        }
                    }
                }
                Ok(())
            },
            Err(error) => {
                Err(Error::BSTFImport(error.to_string()))
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::model::Graph;

    #[test]
    fn stof_suite() {
        let mut graph = Graph::default();
        graph.parse_stof_file("stof", "src/model/formats/stof/tests/tests.stof", None, false).unwrap();
        let res = graph.test(None, true);
        match res {
            Ok(res) => println!("{res}"),
            Err(err) => panic!("{err}")
        }
    }
}
