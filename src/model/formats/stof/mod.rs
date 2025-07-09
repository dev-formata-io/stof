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
use arcstr::literal;
use crate::{model::{Format, Graph, NodeRef}, parser::{context::ParseContext, doc::document}, runtime::Error};


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
    fn file_import(&self, graph: &mut Graph, format: &str, path: &str, node: Option<NodeRef>) -> Result<(), Error> {
        // TODO remove this function and use a lib in Format
        match fs::read_to_string(path) {
            Ok(src) => {
                self.string_import(graph, format, &src, node)
            },
            Err(_error) => {
                Err(Error::Custom(literal!("Stof file read error: file is not a text file.")))
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::model::Graph;

    #[test]
    fn stof_suite() {
        let stof = r#"
        #[test]
        fn test_function() -> str {
            'hello, world'
        }

        #[test]
        #[errors]
        fn errors() {
            42
        }

        #[test]
        #[errors]
        fn errors_but_ok() -> int {
            4.2.3 + 43
        }

        #[test]
        fn test_abs_lib() {
            let v = -45;
            v = v.abs();
            if (v < 0) {
                v += 1.2.3; // throw an error lol
            }
        }
        "#;

        let mut graph = Graph::default();
        graph.parse(stof, None).unwrap();
        let res = graph.test(None, false);
        match res {
            Ok(res) => println!("{res}"),
            Err(err) => panic!("{err}")
        }
    }
}
