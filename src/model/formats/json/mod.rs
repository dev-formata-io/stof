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


mod import;
pub mod export;
use serde_json::Value;
use crate::{model::{json::{export::json_value_from_node, import::parse_json_object_value}, Format, Graph, NodeRef}, runtime::Error};


#[derive(Debug)]
pub struct JsonFormat;
impl Format for JsonFormat {
    fn identifiers(&self) -> Vec<String> {
        vec!["json".into()]
    }
    fn content_type(&self) -> String {
        "application/json".into()
    }
    fn string_import(&self, graph: &mut Graph, _format: &str, src: &str, node: Option<NodeRef>) -> Result<(), Error> {
        match serde_json::from_str::<Value>(src) {
            Ok(value) => {
                let mut parse_node = graph.ensure_main_root();
                if let Some(nd) = node {
                    parse_node = nd;
                }
                parse_json_object_value(graph, &parse_node, value);
                Ok(())
            },
            Err(error) => {
                Err(Error::JSONStringImport(error.to_string()))
            }
        }
    }
    fn string_export(&self, graph: &Graph, _format: &str, node: Option<NodeRef>) -> Result<String, Error> {
        let exp_node;
        if let Some(nd) = node {
            exp_node = nd;
        } else {
            exp_node = graph.main_root().expect("graph does not have a main 'root' node for default JSON export");
        }
        let value = json_value_from_node(graph, &exp_node);
        match serde_json::to_string(&value) {
            Ok(res) => {
                Ok(res)
            },
            Err(error) => {
                Err(Error::JSONStringExport(error.to_string()))
            }
        }
    }
}
