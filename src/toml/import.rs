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
use toml::{Table, Value};
use crate::{Data, SField, SGraph, SNodeRef, SNum, SVal};


/// Parse a serde_json Object value into the graph.
pub(crate) fn parse_object_value(graph: &mut SGraph, node: &SNodeRef, table: Table) {
    for (field, val) in table {
        let mut tf = parse_field_value(graph, node, val, &field);
        tf.attach(node, graph);
    }
}

/// Parse a serde_json field value into the graph.
pub(crate) fn parse_field_value(graph: &mut SGraph, node: &SNodeRef, value: Value, field: &str) -> SField {
    match value {
        Value::Integer(val) => {
            SField::new(field, SVal::Number(SNum::I64(val)))
        },
        Value::Float(val) => {
            SField::new(field, SVal::Number(SNum::F64(val)))
        },
        Value::String(val) => {
            SField::new(field, SVal::String(val))
        },
        Value::Boolean(val) => {
            SField::new(field, SVal::Bool(val))
        },
        Value::Datetime(val) => {
            SField::new(field, SVal::String(val.to_string()))
        },
        Value::Array(vals) => {
            let mut jf_arr = Vec::new();
            parse_array_values(graph, node, vals, &mut jf_arr, field);
            SField::new(field, SVal::Array(jf_arr))
        }
        Value::Table(map) => {
            let child_node = graph.insert_node(field, Some(node));
            parse_object_value(graph, &child_node, Table::from(map));
            SField::new(field, child_node)
        },
    }
}

/// Parse array values.
pub(crate) fn parse_array_values(graph: &mut SGraph, node: &SNodeRef, vals: Vec<Value>, res: &mut Vec<SVal>, field: &str) {
    for val in vals {
        match val {
            Value::Integer(val) => {
                res.push(SVal::Number(SNum::I64(val)));
            },
            Value::Float(val) => {
                res.push(SVal::Number(SNum::F64(val)));
            },
            Value::String(val) => {
                res.push(SVal::String(val));
            },
            Value::Boolean(val) => {
                res.push(SVal::Bool(val));
            },
            Value::Datetime(val) => {
                res.push(SVal::String(val.to_string()));
            },
            Value::Array(vals) => {
                let mut jf_arr = Vec::new();
                parse_array_values(graph, node, vals, &mut jf_arr, field);
                res.push(SVal::Array(jf_arr));
            }
            Value::Table(map) => {
                let name = format!("_a_obj{}", nanoid!(7));
                let child_node = graph.insert_node(&name, Some(node));
                parse_object_value(graph, &child_node, Table::from(map));
                res.push(SVal::Object(child_node));
            },
        }
    }
}
