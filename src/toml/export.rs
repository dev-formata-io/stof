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

use std::ops::Deref;
use toml::{Table, Value};
use crate::{SField, SGraph, SNodeRef, SNum, SVal};


/// Export a serde_json Value from a node in the graph.
pub(crate) fn toml_value_from_node(graph: &SGraph, node_ref: &SNodeRef) -> Table {
    let mut map = Table::new();
    for field in SField::fields(graph, node_ref) {
        let mut do_export = true;
        if let Some(export) = field.attributes.get("export") {
            if !export.truthy() {
                do_export = false;
            }
        }
        if do_export {
            let value;
            match field.value {
                SVal::Ref(rf) => {
                    let val = rf.read().unwrap();
                    value = val.deref().clone();
                },
                _ => {
                    value = field.value;
                }
            }
            match value {
                SVal::String(val) => map.insert(field.name, Value::String(val)),
                SVal::Bool(val) => map.insert(field.name, Value::Boolean(val)),
                SVal::Number(val) => {
                    match val {
                        SNum::F64(v) => map.insert(field.name, Value::Float(v)),
                        SNum::I64(v) => map.insert(field.name, Value::Integer(v)),
                        SNum::Units(v, _) => map.insert(field.name, Value::Float(v)),
                    }
                },
                SVal::Blob(blob) => {
                    let mut array = Vec::new();
                    for v in blob {
                        array.push(Value::Integer(v as i64));
                    }
                    map.insert(field.name, Value::Array(array))
                },
                SVal::FnPtr(ptr) => map.insert(field.name, Value::String(format!("fn({})", ptr.id))),
                SVal::Array(vals) => map.insert(field.name, value_from_array(graph, vals)),
                SVal::Tuple(vals) => map.insert(field.name, value_from_array(graph, vals)),
                SVal::Object(nref) => map.insert(field.name, Value::Table(toml_value_from_node(graph, &nref))),
                _ => None
            };
        }
    }
    map
}


/// Export value from an array of values.
fn value_from_array(graph: &SGraph, vals: Vec<SVal>) -> Value {
    let mut results: Vec<Value> = Vec::new();
    for val in vals {
        let value;
        match val {
            SVal::Ref(rf) => {
                let val = rf.read().unwrap();
                value = val.deref().clone();
            },
            _ => {
                value = val;
            }
        }
        match value {
            SVal::String(val) => results.push(Value::String(val)),
            SVal::Bool(val) => results.push(Value::Boolean(val)),
            SVal::Number(val) => {
                match val {
                    SNum::F64(v) => results.push(Value::Float(v)),
                    SNum::I64(v) => results.push(Value::Integer(v)),
                    SNum::Units(v, _) => results.push(Value::Float(v)),
                }
            },
            SVal::Blob(blob) => {
                let mut array = Vec::new();
                for v in blob {
                    array.push(Value::Integer(v as i64));
                }
                results.push(Value::Array(array))
            },
            SVal::FnPtr(ptr) => results.push(Value::String(format!("fn({})", ptr.id))),
            SVal::Array(vals) => results.push(value_from_array(graph, vals)),
            SVal::Tuple(vals) => results.push(value_from_array(graph, vals)),
            SVal::Object(nref) => results.push(Value::Table(toml_value_from_node(graph, &nref))),
            _ => {}
        };
    }
    Value::Array(results)
}
