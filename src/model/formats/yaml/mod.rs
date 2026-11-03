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

use serde_json::Value;
use crate::{model::{Format, Graph, NodeRef, Profile, export::json_value_from_node, import::parse_json_object_value}, runtime::Error};


#[derive(Debug)]
pub struct YamlFormat;
impl Format for YamlFormat {
    fn identifiers(&self) -> Vec<String> {
        vec!["yaml".into(), "yml".into()]
    }
    fn content_type(&self) -> String {
        "application/yaml".into()
    }
    fn string_import(&self, graph: &mut Graph, _format: &str, src: &str, node: Option<NodeRef>, _profile: &Profile) -> Result<(), Error> {
        if src.is_empty() { return Ok(()); }
        // through serde_yaml's own value: YAML has .inf, -.inf, and .nan, which a JSON value can't hold
        match serde_yaml::from_str::<serde_yaml::Value>(src) {
            Ok(value) => {
                let value = json_from_yaml(value);
                let mut parse_node = graph.ensure_main_root();
                if let Some(nd) = node {
                    parse_node = nd;
                }
                parse_json_object_value(graph, &parse_node, value);
                Ok(())
            },
            Err(error) => {
                Err(Error::YAMLStringImport(error.to_string()))
            }
        }
    }
    fn string_export(&self, graph: &Graph, _format: &str, node: Option<NodeRef>) -> Result<String, Error> {
        let exp_node;
        if let Some(nd) = node {
            exp_node = nd;
        } else {
            exp_node = graph.main_root().expect("graph does not have a main 'root' node for default YAML export");
        }
        let value = yaml_from_json(json_value_from_node(graph, &exp_node));
        match serde_yaml::to_string(&value) {
            Ok(yaml) => {
                Ok(yaml)
            },
            Err(error) => {
                Err(Error::YAMLStringExport(error.to_string()))
            }
        }
    }
}


/// A YAML value as a JSON value for import. YAML's infinity and NaN become the strings JSON import reads as those
/// numbers ("Infinity", "-Infinity", "NaN"); tags are dropped (the tagged value is kept).
fn json_from_yaml(value: serde_yaml::Value) -> Value {
    use serde_yaml::Value as Yaml;
    match value {
        Yaml::Null => Value::Null,
        Yaml::Bool(b) => Value::Bool(b),
        Yaml::Number(number) => {
            if let Some(int) = number.as_i64() { return Value::from(int); }
            if let Some(int) = number.as_u64() { return Value::from(int); }
            let float = number.as_f64().unwrap_or(f64::NAN);
            match serde_json::Number::from_f64(float) {
                Some(number) => Value::Number(number),
                None => Value::String(crate::runtime::float_text(float)),
            }
        },
        Yaml::String(text) => Value::String(text),
        Yaml::Sequence(items) => Value::Array(items.into_iter().map(json_from_yaml).collect()),
        Yaml::Mapping(map) => {
            let mut object = serde_json::Map::new();
            for (key, value) in map {
                let key = match key {
                    Yaml::String(key) => key,
                    other => serde_yaml::to_string(&other).map(|k| k.trim_end().to_string()).unwrap_or_default(),
                };
                object.insert(key, json_from_yaml(value));
            }
            Value::Object(object)
        },
        Yaml::Tagged(tagged) => json_from_yaml(tagged.value),
    }
}

/// A JSON value from export as a YAML value: the strings JSON export uses for infinity and NaN are YAML's own
/// .inf, -.inf, and .nan (numbers other YAML readers understand).
fn yaml_from_json(value: Value) -> serde_yaml::Value {
    use serde_yaml::Value as Yaml;
    match value {
        Value::Null => Yaml::Null,
        Value::Bool(b) => Yaml::Bool(b),
        Value::Number(number) => {
            if let Some(int) = number.as_i64() { Yaml::Number(int.into()) }
            else if let Some(int) = number.as_u64() { Yaml::Number(int.into()) }
            else { Yaml::Number(number.as_f64().unwrap_or(f64::NAN).into()) }
        },
        Value::String(text) => match text.as_str() {
            "Infinity" => Yaml::Number(f64::INFINITY.into()),
            "-Infinity" => Yaml::Number(f64::NEG_INFINITY.into()),
            "NaN" => Yaml::Number(f64::NAN.into()),
            _ => Yaml::String(text),
        },
        Value::Array(items) => Yaml::Sequence(items.into_iter().map(yaml_from_json).collect()),
        Value::Object(object) => {
            let mut map = serde_yaml::Mapping::new();
            for (key, value) in object { map.insert(Yaml::String(key), yaml_from_json(value)); }
            Yaml::Mapping(map)
        },
    }
}
