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

use nanoid::nanoid;
use serde_json::Value;
use crate::{Data, IntoSVal, SField, SGraph, SNodeRef, SVal};


/// Parse a serde_json Object value into the graph.
pub(crate) fn parse_object_value(graph: &mut SGraph, node: &SNodeRef, value: Value) {
    match value {
        Value::Object(map) => {
            for (field, val) in map {
                let mut jf = parse_field_value(graph, node, val, &field);
                jf.attach(node, graph);
            }
        },
        _ => {}
    }
}

/// Parse a serde_json field value into the graph.
pub(crate) fn parse_field_value(graph: &mut SGraph, node: &SNodeRef, value: Value, field: &str) -> SField {
    match value {
        Value::Null |
        Value::Number(_) |
        Value::String(_) |
        Value::Bool(_) => {
            SField::new(field, SVal::from(&value))
        },
        Value::Array(vals) => {
            let mut jf_arr = Vec::new();
            parse_array_values(graph, node, vals, &mut jf_arr, field);
            SField::new(field, SVal::Array(jf_arr))
        }
        Value::Object(_) => {
            let child_node = graph.insert_node(field, Some(node));
            parse_object_value(graph, &child_node, value);
            SField::new(field, child_node)
        },
    }
}

/// Parse array values.
pub(crate) fn parse_array_values(graph: &mut SGraph, node: &SNodeRef, vals: Vec<Value>, res: &mut Vec<SVal>, field: &str) {
    for val in vals {
        match val {
            Value::Null |
            Value::String(_) |
            Value::Number(_) |
            Value::Bool(_) => {
                res.push(SVal::from(&val));
            },
            Value::Object(_) => {
                let name = format!("_a_obj{}", nanoid!(7));
                let child_node = graph.insert_node(&name, Some(node));
                parse_object_value(graph, &child_node, val);
                res.push(SVal::Object(child_node));
            },
            Value::Array(vals) => {
                let mut second_vals = Vec::new();
                parse_array_values(graph, node, vals, &mut second_vals, field);
                res.push(SVal::Array(second_vals));
            },
        }
    }
}


impl IntoSVal for &Value {
    /// Value without a graph.
    fn stof_value(&self) -> SVal {
        match self {
            Value::Array(vals) => {
                let mut arr = Vec::new();
                for val in vals {
                    arr.push(SVal::from(val));
                }
                SVal::Array(arr)
            },
            Value::Bool(val) => {
                SVal::Bool(*val)
            },
            Value::String(val) => {
                SVal::String(val.clone())
            },
            Value::Number(val) => {
                if val.is_i64() {
                    return val.as_i64().unwrap().into();
                } else if val.is_u64() {
                    return (val.as_u64().unwrap() as i64).into();
                } else if val.is_f64() {
                    return val.as_f64().unwrap().into();
                }
                SVal::Null
            },
            _ => {
                SVal::Null
            }
        }
    }
}
