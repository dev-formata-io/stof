//
// Copyright 2025 Formata, Inc. All rights reserved.
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

use crate::{model::{Format, Graph, NodeRef, Profile, import::parse_json_object_value}, runtime::Error};
mod encoded;
use encoded::URLEncode;


#[derive(Debug)]
pub struct UrlEncodedFormat;
impl Format for UrlEncodedFormat {
    fn identifiers(&self) -> Vec<String> {
        vec!["urlencoded".into(), "www-form".into()]
    }
    fn content_type(&self) -> String {
        "application/x-www-form-urlencoded".into()
    }
    fn string_import(&self, graph: &mut Graph, _format: &str, src: &str, node: Option<NodeRef>, _profile: &Profile) -> Result<(), Error> {
        if src.is_empty() { return Ok(()); }
        let value = URLEncode::decode(src);
        let mut parse_node = graph.ensure_main_root();
        if let Some(nd) = node {
            parse_node = nd;
        }
        parse_json_object_value(graph, &parse_node, value);
        Ok(())
    }
    fn string_export(&self, graph: &Graph, _format: &str, node: Option<NodeRef>) -> Result<String, Error> {
        let exp_node;
        if let Some(nd) = node {
            exp_node = nd;
        } else {
            exp_node = graph.main_root().expect("graph does not have a main 'root' node for default URLEncoded export");
        }
        Ok(URLEncode::encode(graph, &exp_node))
    }
}
