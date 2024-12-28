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

pub mod import;
use export::json_value_from_node;
use import::parse_object_value;
pub mod export;

use anyhow::{anyhow, Result};
use serde_json::{Map, Value};
use crate::{Format, IntoNodeRef, SDoc, SGraph};

#[cfg(test)]
mod tests;


/// Stof NDJSON interface.
pub struct NDJSON;
impl NDJSON {
    /// Parse a NDJSON string into a new document.
    pub fn parse_new(json: &str) -> Result<SDoc> {
        Ok(SDoc::new(Self::parse(json)?))
    }

    /// Parse a NDJSON string into a singular graph.
    pub fn parse(ndjson: &str) -> Result<SGraph> {
        let mut graphs = Vec::new();
        for json in ndjson.split('\n').collect::<Vec<&str>>() {
            if json.len() > 0 && json.contains('{') && json.contains('}') {
                graphs.push(JSON::parse(json)?);
            }
        }
        let mut result = SGraph::default();
        for graph in graphs {
            result.default_absorb_merge(graph)?;
        }
        Ok(result)
    }
}


/// Stof JSON interface.
pub struct JSON;
impl JSON {
    /// Parse a JSON string into a new document.
    pub fn parse_new(json: &str) -> Result<SDoc> {
        Ok(SDoc::new(Self::parse(json)?))
    }

    /// Parse a JSON string into a Stof graph.
    pub fn parse(json: &str) -> Result<SGraph> {
        if let Ok(value) = serde_json::from_str::<Value>(json) {
            Ok(Self::from_value(value))
        } else {
            Err(anyhow!("Stof JSON Error: Unable to parse JSON string into graph"))
        }
    }

    /// Create a Stof graph from a serde_json Value.
    pub fn from_value(mut value: Value) -> SGraph {
        if !value.is_object() {
            let mut map = Map::new();
            map.insert("field".to_string(), value);
            value = Value::Object(map);
        }
        let mut graph = SGraph::default();
        let root = graph.insert_root("root");
        parse_object_value(&mut graph, &root, value);
        graph
    }

    /// Export a graph as a JSON string.
    /// Exports the main root of the graph only.
    pub fn stringify(graph: &SGraph) -> Result<String> {
        if let Some(main) = graph.main_root() {
            let value = json_value_from_node(graph, &main);
            if let Ok(json) = serde_json::to_string(&value) {
                return Ok(json);
            }
            return Err(anyhow!("Stof JSON Error: Could not parse serde_json::Value into json string"));
        }
        Err(anyhow!("Stof JSON Error: Did not find a main root to stringify"))
    }

    /// Export a node as a JSON string.
    pub fn stringify_node(graph: &SGraph, node: impl IntoNodeRef) -> Result<String> {
        let value = json_value_from_node(graph, &node.node_ref());
        if let Ok(json) = serde_json::to_string(&value) {
            return Ok(json);
        }
        Err(anyhow!("Stof JSON Error: Could not parse serde_json::Value into json string"))
    }

    /// To serde_json Value.
    pub fn to_value(graph: &SGraph) -> Result<Value> {
        if let Some(main) = graph.main_root() {
            return Ok(json_value_from_node(graph, &main));
        }
        Err(anyhow!("Stof JSON Error: Did not find a main root to parse into a Value"))
    }

    /// To serde_json Value from a node.
    pub fn to_node_value(graph: &SGraph, node: impl IntoNodeRef) -> Value {
        json_value_from_node(graph, &node.node_ref())
    }
}


impl Format for JSON {
    /// Format getter for JSON.
    fn format(&self) -> String {
        "json".to_string()
    }

    /// Content type for JSON.
    fn content_type(&self) -> String {
        "application/json".to_string()
    }

    /// Header import.
    fn header_import(&self, pid: &str, doc: &mut crate::SDoc, _content_type: &str, bytes: &mut bytes::Bytes, as_name: &str) -> Result<()> {
        let str = std::str::from_utf8(bytes.as_ref())?;
        self.string_import(pid, doc, str, as_name)
    }

    /// String import.
    fn string_import(&self, pid: &str, doc: &mut crate::SDoc, src: &str, as_name: &str) -> Result<()> {
        let mut graph = JSON::parse(src)?;
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
    fn file_import(&self, pid: &str, doc: &mut crate::SDoc, _format: &str, full_path: &str, _extension: &str, as_name: &str) -> Result<()> {
        let src = doc.fs_read_string(pid, full_path)?;
        self.string_import(pid, doc, &src, as_name)
    }

    /// Export string.
    fn export_string(&self, _pid: &str, doc: &crate::SDoc, node: Option<&crate::SNodeRef>) -> Result<String> {
        if node.is_some() {
            JSON::stringify_node(&doc.graph, node)
        } else {
            JSON::stringify(&doc.graph)
        }
    }
}


impl Format for NDJSON {
    /// Format getter for NDJSON.
    fn format(&self) -> String {
        "ndjson".to_string()
    }

    /// Content type for NDJSON.
    fn content_type(&self) -> String {
        "application/x-ndjson".to_string()
    }

    /// Header import.
    fn header_import(&self, pid: &str, doc: &mut crate::SDoc, _content_type: &str, bytes: &mut bytes::Bytes, as_name: &str) -> Result<()> {
        let str = std::str::from_utf8(bytes.as_ref())?;
        self.string_import(pid, doc, str, as_name)
    }

    /// String import.
    fn string_import(&self, pid: &str, doc: &mut crate::SDoc, src: &str, as_name: &str) -> Result<()> {
        let mut graph = NDJSON::parse(src)?;
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
    fn file_import(&self, pid: &str, doc: &mut crate::SDoc, _format: &str, full_path: &str, _extension: &str, as_name: &str) -> Result<()> {
        let src = doc.fs_read_string(pid, full_path)?;
        self.string_import(pid, doc, &src, as_name)
    }

    /// Export string.
    /// NDJSON is just JSON here...
    fn export_string(&self, _pid: &str, doc: &crate::SDoc, node: Option<&crate::SNodeRef>) -> Result<String> {
        if node.is_some() {
            JSON::stringify_node(&doc.graph, node)
        } else {
            JSON::stringify(&doc.graph)
        }
    }
}


#[cfg(test)]
mod inner_tests {
    use serde_json::json;
    use super::JSON;

    #[test]
    fn from_json_string() {
        let res = JSON::parse(r#"
        {
            "a": "A",
            "b": "B",
            "bool": true,
            "num": 42,
            "arr": ["hi", { "arr_obj": 53 }, true, 23, ["nested", { "nested": true }]],
            "obj": {
                "child_val": 44
            }
        }"#);
        assert!(res.is_ok());
        let graph = res.unwrap();
        assert_eq!(graph.root_count(), 1);
        assert!(graph.root_by_name("root").is_some());

        let root = graph.main_root().unwrap();

        let b = root.dot_field(&graph, "b");
        assert!(b.is_some()); let b = b.unwrap();
        assert_eq!(b.name, "b");
        assert_eq!(b.value, "B".into());

        assert_eq!(r#"{"a":"A","arr":["hi",{"arr_obj":53},true,23,["nested",{"nested":true}]],"b":"B","bool":true,"num":42,"obj":{"child_val":44}}"#, JSON::stringify(&graph).unwrap());
    }

    #[test]
    fn from_json_value() {
        let graph = JSON::from_value(json!({
            "a": "A",
            "b": "B",
            "bool": true,
            "num": 42,
            "arr": ["hi", { "arr_obj": 53 }, true, 23, ["nested", { "nested": true }]],
            "obj": {
                "child_val": 44
            }
        }));
        assert_eq!(graph.root_count(), 1);
        assert!(graph.root_by_name("root").is_some());

        let root = graph.main_root().unwrap();

        let b = root.dot_field(&graph, "b");
        assert!(b.is_some()); let b = b.unwrap();
        assert_eq!(b.name, "b");
        assert_eq!(b.value, "B".into());

        assert_eq!(r#"{"a":"A","arr":["hi",{"arr_obj":53},true,23,["nested",{"nested":true}]],"b":"B","bool":true,"num":42,"obj":{"child_val":44}}"#, JSON::stringify(&graph).unwrap());
    }
}
