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
    
    /// Parser import.
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
