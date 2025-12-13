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

use std::collections::VecDeque;
use serde_json::{Map, Number, Value};
use crate::{model::{Field, Graph, NodeRef, NOEXPORT_FIELD_ATTR}, runtime::Val};


/// URL encode struct.
pub(super) struct URLEncode;
impl URLEncode {
    /// Encode a node as a URL encoded string.
    pub fn encode(graph: &Graph, node: &NodeRef) -> String {
        let mut results = Vec::new();
        let mut path = Vec::new();
        encode_node(graph, node, &mut results, &mut path);
        results.join("&")
    }

    /// Decode a URL encoded string.
    pub fn decode(encoded: &str) -> Value {
        value_from_field_tuples(decode_into_tuples(encoded))
    }
}


/// Encode a graph node.
fn encode_node(graph: &Graph, node: &NodeRef, results: &mut Vec<String>, path: &mut Vec<String>) {
    if let Some(node) = node.node(graph) {
        for (name, dref) in &node.data {
            if let Some(field) = graph.get_stof_data::<Field>(dref) {
                if !field.attributes.contains_key(NOEXPORT_FIELD_ATTR.as_str()) {
                    // could still be objects... just not child object fields (unles you create another reference...)
                    encode_field_value(graph, field.value.val.read().clone(), &name, results, path);
                }
            }
        }
        for child in &node.children {
            if let Some(child) = child.node(graph) {
                if child.is_field() && !child.attributes.contains_key(NOEXPORT_FIELD_ATTR.as_str()) {
                    encode_field_value(graph, Val::Obj(child.id.clone()), child.name.as_ref(), results, path);
                }
            }
        }
    }
}


/// Encode field value.
fn encode_field_value(graph: &Graph, value: Val, field: &str, results: &mut Vec<String>, path: &mut Vec<String>) {
    match value {
        Val::List(vals) => {
            path.push(field.to_owned());
            for i in 0..vals.len() {
                path.push(i.to_string());
                let val = &vals[i];
                encode_array_value(graph, val.read().clone(), results, path);
                path.pop();
            }
            path.pop();
        },
        Val::Obj(nref) => {
            path.push(field.to_owned());
            encode_node(graph, &nref, results, path);
            path.pop();
        },
        _ => {
            let value = value.to_string();
            let encoded_value = urlencoding::encode(&value);
            
            let mut name = field.to_string();
            if path.len() > 0 {
                name = path[0].clone();
                for i in 1..path.len() {
                    name.push_str(&format!("[{}]", &path[i]));
                }
                name.push_str(&format!("[{}]", field));
            }

            let encoded_name = urlencoding::encode(&name).into_owned();
            results.push(format!("{}={}", encoded_name, encoded_value.into_owned()));
        }
    }
}


/// Encode array value.
fn encode_array_value(graph: &Graph, value: Val, results: &mut Vec<String>, path: &mut Vec<String>) {
    match value {
        Val::List(vals) => {
            for i in 0..vals.len() {
                path.push(i.to_string());
                let val = &vals[i];
                encode_array_value(graph, val.read().clone(), results, path);
                path.pop();
            }
        },
        Val::Obj(nref) => {
            encode_node(graph, &nref, results, path);
        },
        _ => {
            let value = value.to_string();
            let encoded_value = urlencoding::encode(&value);
            
            let mut name = String::default();
            if path.len() > 0 {
                name = path[0].clone();
                for i in 1..path.len() {
                    name.push_str(&format!("[{}]", &path[i]));
                }
            }

            let encoded_name = urlencoding::encode(&name).into_owned();
            results.push(format!("{}={}", encoded_name, encoded_value.into_owned()));
        }
    }
}


/// Decode into tuples.
fn decode_into_tuples(encoded: &str) -> Vec<(String, Value)> {
    let mut fields = Vec::new();
    let url_fields: Vec<&str> = encoded.split("&").collect();
    for url_field in url_fields {
        let vals: Vec<&str> = url_field.split("=").collect();
        if vals.len() == 2 {
            let name = vals[0];
            let value = vals[1];
            let decoded_name = urlencoding::decode(name).unwrap().into_owned();
            let decoded_value = urlencoding::decode(value).unwrap().into_owned();

            let decoded_int: Result<i64, _> = decoded_value.parse();
            match decoded_int {
                Ok(number) => {
                    let decoded_val = Value::Number(Number::from(number));
                    fields.push((decoded_name, decoded_val));
                },
                Err(_) => {
                    let decoded_float: Result<f64, _> = decoded_value.parse();
                    match decoded_float {
                        Ok(number) => {
                            let decoded_val = Value::Number(Number::from_f64(number).unwrap());
                            fields.push((decoded_name, decoded_val));
                        },
                        Err(_) => {
                            let decoded_boolean: Result<bool, _> = decoded_value.parse();
                            match decoded_boolean {
                                Ok(val) => {
                                    let decoded_val = Value::Bool(val);
                                    fields.push((decoded_name, decoded_val));
                                },
                                Err(_) => {
                                    let decoded_val = Value::String(decoded_value);
                                    fields.push((decoded_name, decoded_val));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    fields
}


/// Value from field tuples.
fn value_from_field_tuples(tuples: Vec<(String, Value)>) -> Value {
    // Turn each tuple into a field, then place it into the graph
    let mut to_insert = Vec::new();
    for (field_name, field_value) in tuples {
        let mut path = Vec::new();
        let first_split: Vec<&str> = field_name.split("][").collect();
        for splt in first_split {
            let second_split: Vec<&str> = splt.split("[").collect();
            for val in second_split {
                let val = val.to_string().replace("]", "");
                path.push(val);
            }
        }
        to_insert.push((field_name, path, field_value));
    }
    to_insert.sort_by(|a, b| {
        a.0.cmp(&b.0)
    });

    let mut json = Value::Object(Map::new());
    for field in to_insert {
        let mut obj_path: VecDeque<String> = field.1.into_iter().collect();
        insert_value(&mut json, &mut obj_path, field.2);
    }
    json
}


/// Ensure the object exists and insert the value for it
fn insert_value(value: &mut Value, path: &mut VecDeque<String>, to_insert: Value) {
    if path.len() > 1 {
        // We have more calls to make, we're creating an object or an array potentially
        let item = path.pop_front().unwrap();
        let next = path.front().unwrap();

        let mut array = next.len() < 1;
        if !array {
            let res: Result<i32, _> = next.parse();
            array = res.is_ok();
        }

        match value {
            Value::Object(map) => {
                if let Some(val) = map.get_mut(&item) {
                    insert_value(val, path, to_insert);
                } else {
                    if array {
                        map.insert(item.clone(), Value::Array(vec![]));
                        if let Some(val) = map.get_mut(&item) {
                            insert_value(val, path, to_insert);
                        }
                    } else {
                        map.insert(item.clone(), Value::Object(Map::new()));
                        if let Some(val) = map.get_mut(&item) {
                            insert_value(val, path, to_insert);
                        }
                    }
                }
            },
            Value::Array(vals) => {
                if let Ok(index) = item.parse::<usize>() {
                    if index >= vals.len() {
                        // Value isn't yet in the array
                        let mut val = Value::Object(Map::new());
                        if array { val = Value::Array(vec![]); }
                        insert_value(&mut val, path, to_insert);
                        vals.push(val);
                    } else {
                        // Value is already in the array
                        let val = &mut vals[index];
                        insert_value(val, path, to_insert);
                    }
                }
            },
            _ => {}
        }
    } else if path.len() == 1 {
        // Name of the thing to insert
        let item = path.pop_front().unwrap();

        // We are inserting "to_insert" onto this value
        match value {
            Value::Object(map) => {
                map.insert(item, to_insert);
            },
            Value::Array(vals) => {
                vals.push(to_insert);
            },
            _ => {}
        }
    }
}
