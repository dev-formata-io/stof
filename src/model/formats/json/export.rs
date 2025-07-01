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

use imbl::Vector;
use serde_json::{Map, Number, Value};
use crate::{model::{Field, Graph, NodeRef, NOEXPORT_FIELD_ATTR}, runtime::{Num, Val}};


/// Export a serde_json Value from a node in the graph.
pub(crate) fn json_value_from_node(graph: &Graph, node: &NodeRef) -> Value {
    let mut map = Map::new();
    if let Some(node) = node.node(graph) {
        for (name, dref) in &node.data {
            if let Some(field) = graph.get_stof_data::<Field>(dref) {
                if !field.attributes.contains_key(&NOEXPORT_FIELD_ATTR) {
                    // could still be objects... just not child object fields (unles you create another reference...)
                    map.insert(name.to_string(), json_value(graph, field.value.get()));
                }
            }
        }
        for child in &node.children {
            if let Some(child) = child.node(graph) {
                if child.is_field() && !child.attributes.contains_key(&NOEXPORT_FIELD_ATTR) {
                    map.insert(child.name.to_string(), json_value(graph, Val::Obj(child.id.clone())));
                }
            }
        }
    }
    Value::Object(map)
}

/// Get a JSON value from a Val.
fn json_value(graph: &Graph, val: Val) -> Value {
    match val {
        Val::Void |
        Val::Null => Value::Null,
        Val::Bool(v) => Value::Bool(v),
        Val::Str(v) => Value::String(v.to_string()),
        Val::Num(v) => Value::Number(Number::from(v)),
        Val::Blob(blob) => Value::from_iter(blob.into_iter()),
        Val::Fn(_dref) => Value::Null,
        Val::Data(_dref) => Value::Null, // TODO custom exports
        Val::List(vals) => value_from_array(graph, vals),
        Val::Tup(vals) => value_from_array(graph, vals),
        Val::Ver(..) => Value::String(val.to_string()),
        Val::Set(vals) => value_from_array(graph, vals.into_iter().collect()),
        Val::Obj(nref) => json_value_from_node(graph, &nref),
        Val::Map(map) => {
            let mut value = Map::new();
            for (k, v) in map {
                value.insert(k.to_string(), json_value(graph, v));
            }
            Value::Object(value)
        },
    }
}

/// Export value from an array of values.
fn value_from_array(graph: &Graph, vals: Vector<Val>) -> Value {
    let mut results: Vec<Value> = Vec::new();
    for val in vals {
        results.push(json_value(graph, val));
    }
    Value::Array(results)
}

impl From<Num> for Number {
    fn from(value: Num) -> Self {
        match value {
            Num::Float(v) => {
                Number::from_f64(v).unwrap()
            },
            Num::Int(v) => {
                Number::from(v)
            },
            Num::Units(v, _) => {
                Number::from_f64(v).unwrap()
            }
        }
    }
}
