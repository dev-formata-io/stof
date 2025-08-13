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
use toml::{Table, Value};
use crate::{model::{Field, Graph, NodeRef, NOEXPORT_FIELD_ATTR}, runtime::{Num, Val, ValRef}};


pub(super) fn toml_value_from_node(graph: &Graph, node: &NodeRef) -> Table {
    let mut map = Table::new();
    if let Some(node) = node.node(graph) {
        for (name, dref) in &node.data {
            if let Some(field) = graph.get_stof_data::<Field>(dref) {
                if !field.attributes.contains_key(NOEXPORT_FIELD_ATTR.as_str()) {
                    // could still be objects... just not child object fields (unles you create another reference...)
                    map.insert(name.to_string(), toml_value(graph, field.value.get()));
                }
            }
        }
        for child in &node.children {
            if let Some(child) = child.node(graph) {
                if child.is_field() && !child.attributes.contains_key(NOEXPORT_FIELD_ATTR.as_str()) {
                    map.insert(child.name.to_string(), toml_value(graph, Val::Obj(child.id.clone())));
                }
            }
        }
    }
    map
}

pub(super) fn toml_value(graph: &Graph, val: Val) -> Value {
    match val {
        Val::Void |
        Val::Null => Value::String("null".into()),
        Val::Promise(..) => Value::String("promise".into()),
        Val::Bool(v) => Value::Boolean(v),
        Val::Str(v) => Value::String(v.to_string()),
        Val::Num(v) => {
            match v {
                Num::Int(v) => Value::Integer(v),
                Num::Float(v) => Value::Float(v),
                Num::Units(v, _) => Value::Float(v),
            }
        },
        Val::Blob(blob) => {
            let mut array = Vec::new();
            for v in blob {
                array.push(Value::Integer(v as i64));
            }
            Value::Array(array)
        },
        Val::Fn(_dref) => Value::String("fn".into()),
        Val::Data(_dref) => Value::String("data".into()),
        Val::List(vals) => value_from_array(graph, vals),
        Val::Tup(vals) => value_from_array(graph, vals),
        Val::Ver(..) => Value::String(val.to_string()),
        Val::Set(vals) => value_from_array(graph, vals.into_iter().collect()),
        Val::Obj(nref) => {
            let map = toml_value_from_node(graph, &nref);
            Value::Table(map)
        },
        Val::Map(map) => {
            let mut table = Table::new();
            for (k, v) in map {
                let key = k.read().to_string();
                let value = toml_value(graph, v.read().clone());
                table.insert(key, value);
            }
            Value::Table(table)
        },
    }
}

fn value_from_array(graph: &Graph, vals: Vector<ValRef<Val>>) -> Value {
    let mut results: Vec<Value> = Vec::new();
    for val in vals {
        results.push(toml_value(graph, val.read().clone()));
    }
    Value::Array(results)
}
