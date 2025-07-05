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
use rustc_hash::FxHashMap;
use serde_json::Value;
use crate::{model::{Field, Graph, NodeRef, SId, NOEXPORT_FIELD_ATTR}, runtime::{Val, ValRef, Variable}};


/// Parse a serde_json Object value into a graph.
pub(crate) fn parse_json_object_value(graph: &mut Graph, node: &NodeRef, value: Value) {
    match value {
        Value::Object(map) => {
            for (field, val) in map {
                let jf = parse_json_field_value(graph, node, val, &field);
                graph.insert_stof_data(node, &field, Box::new(jf), None);
            }
        },
        _ => {}
    }
}

/// Parse a serde_json field value into the graph.
pub(crate) fn parse_json_field_value(graph: &mut Graph, node: &NodeRef, value: Value, field: &str) -> Field {
    match value {
        Value::Null => {
            Field::new(Variable::new(graph, true, Val::Null, false), None)
        },
        Value::Number(v)  => {
            let val: Val;
            if v.is_i64() {
                val = v.as_i64().unwrap().into();
            } else if v.is_u64() {
                val = v.as_u64().unwrap().into();
            } else if v.is_f64() {
                val = v.as_f64().unwrap().into();
            } else {
                val = Val::Null
            }
            Field::new(Variable::new(graph, true, val, false), None)
        },
        Value::String(v)  => {
            Field::new(Variable::new(graph, true, Val::from(v.as_str()), false), None)
        },
        Value::Bool(v) => {
            Field::new(Variable::new(graph, true, Val::from(v), false), None)
        },
        Value::Array(vals) => {
            let mut jf_arr = Vector::default();
            parse_json_array_values(graph, node, vals, &mut jf_arr);
            Field::new(Variable::new(graph, true, Val::List(jf_arr), false), None)
        }
        Value::Object(_) => {
            let child_node = graph.insert_node(field, Some(node.clone()), true);
            parse_json_object_value(graph, &child_node, value);

            let mut attrs = FxHashMap::default();
            attrs.insert(NOEXPORT_FIELD_ATTR, Val::Null); // don't export object fields
            Field::new(Variable::new(graph, true, Val::Obj(child_node), false), Some(attrs))
        },
    }
}

/// Parse array values.
pub(crate) fn parse_json_array_values(graph: &mut Graph, node: &NodeRef, vals: Vec<Value>, res: &mut Vector<ValRef<Val>>) {
    for val in vals {
        match val {
            Value::Null => {
                res.push_back(ValRef::new(Val::Null));
            },
            Value::Number(v)  => {
                let val: Val;
                if v.is_i64() {
                    val = v.as_i64().unwrap().into();
                } else if v.is_u64() {
                    val = v.as_u64().unwrap().into();
                } else if v.is_f64() {
                    val = v.as_f64().unwrap().into();
                } else {
                    val = Val::Null
                }
                res.push_back(ValRef::new(val));
            },
            Value::String(v)  => {
                res.push_back(ValRef::new(Val::from(v.as_str())));
            },
            Value::Bool(v) => {
                res.push_back(ValRef::new(Val::from(v)));
            },
            Value::Array(vals) => {
                let mut jf_arr = Vector::default();
                parse_json_array_values(graph, node, vals, &mut jf_arr);
                res.push_back(ValRef::new(Val::List(jf_arr)));
            }
            Value::Object(_) => {
                let id = SId::default();
                let child_node = graph.insert_node_id(&id, &id, Some(node.clone()), false);
                parse_json_object_value(graph, &child_node, val);
                res.push_back(ValRef::new(Val::Obj(child_node)));
            },
        }
    }
}
