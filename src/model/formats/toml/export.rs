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
                    if let Some(value) = toml_value(graph, field.value.get()) {
                        map.insert(name.to_string(), value);
                    }
                }
            }
        }
        for child in &node.children {
            if let Some(child) = child.node(graph) {
                if child.is_field() && !child.attributes.contains_key(NOEXPORT_FIELD_ATTR.as_str()) {
                    if let Some(value) = toml_value(graph, Val::Obj(child.id.clone())) {
                        map.insert(child.name.to_string(), value);
                    }
                }
            }
        }
    }
    map
}

pub(super) fn toml_value(graph: &Graph, val: Val) -> Option<Value> {
    match val {
        Val::Void |
        Val::Null => None,
        Val::Promise(..) => None,
        Val::Bool(v) => Some(Value::Boolean(v)),
        Val::Str(v) => Some(Value::String(v.to_string())),
        Val::Prompt(v) => Some(Value::String(v.to_string())),
        Val::Num(v) => {
            match v {
                Num::Int(v) => Some(Value::Integer(v)),
                Num::Float(v) => Some(Value::Float(v)),
                Num::Units(v, _) => Some(Value::Float(v)),
            }
        },
        Val::Blob(blob) => {
            let mut array = Vec::new();
            for v in blob {
                array.push(Value::Integer(v as i64));
            }
            Some(Value::Array(array))
        },
        Val::Fn(_dref) => None,
        Val::Data(_dref) => None,
        Val::List(vals) => Some(value_from_array(graph, vals)),
        Val::Tup(vals) => Some(value_from_array(graph, vals)),
        Val::Ver(..) => Some(Value::String(val.to_string())),
        Val::Set(vals) => Some(value_from_array(graph, vals.into_iter().collect())),
        Val::Obj(nref) => {
            let map = toml_value_from_node(graph, &nref);
            Some(Value::Table(map))
        },
        Val::Map(map) => {
            let mut table = Table::new();
            for (k, v) in map {
                let key = k.read().to_string();
                if let Some(value) = toml_value(graph, v.read().clone()) {
                    table.insert(key, value);
                }
            }
            Some(Value::Table(table))
        },
    }
}

fn value_from_array(graph: &Graph, vals: Vector<ValRef<Val>>) -> Value {
    let mut results: Vec<Value> = Vec::new();
    for val in vals {
        if let Some(value) = toml_value(graph, val.read().clone()) {
            results.push(value);
        }
    }
    Value::Array(results)
}
