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
use serde_json::{Map, Number, Value};
use crate::{SField, SGraph, SNodeRef, SNum, SVal};


/// Export a serde_json Value from a node in the graph.
pub(crate) fn json_value_from_node(graph: &SGraph, node_ref: &SNodeRef) -> Value {
    let mut map = Map::new();
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
                SVal::Ref(_) => map.insert(field.name, Value::Null), // handled above
                SVal::Void => map.insert(field.name, Value::Null),
                SVal::Null => map.insert(field.name, Value::Null),
                SVal::String(val) => map.insert(field.name, Value::String(val)),
                SVal::Bool(val) => map.insert(field.name, Value::Bool(val)),
                SVal::Number(val) => map.insert(field.name, Value::Number(Number::from(val))),
                SVal::Blob(blob) => map.insert(field.name, Value::from_iter(blob.into_iter())),
                SVal::FnPtr(ptr) => map.insert(field.name, Value::String(format!("fn({})", ptr.id))),
                SVal::Array(vals) => map.insert(field.name, value_from_array(graph, vals)),
                SVal::Tuple(vals) => map.insert(field.name, value_from_array(graph, vals)),
                SVal::Object(nref) => map.insert(field.name, json_value_from_node(graph, &nref)),
            };
        }
    }
    Value::Object(map)
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
            SVal::Ref(_) => results.push(Value::Null), // handled above
            SVal::Void => results.push(Value::Null),
            SVal::Null => results.push(Value::Null),
            SVal::String(val) => results.push(Value::String(val)),
            SVal::Bool(val) => results.push(Value::Bool(val)),
            SVal::Number(val) => results.push(Value::Number(Number::from(val))),
            SVal::Blob(blob) => results.push(Value::from_iter(blob.into_iter())),
            SVal::FnPtr(ptr) => results.push(Value::String(format!("fn({})", ptr.id))),
            SVal::Object(nref) => results.push(json_value_from_node(graph, &nref)),
            SVal::Array(vals) => results.push(value_from_array(graph, vals)),
            SVal::Tuple(vals) => results.push(value_from_array(graph, vals)),
        };
    }
    Value::Array(results)
}


impl From<SNum> for Number {
    fn from(value: SNum) -> Self {
        match value {
            SNum::F64(v) => {
                Number::from_f64(v).unwrap()
            },
            SNum::I64(v) => {
                Number::from(v)
            },
            SNum::Units(v, _) => {
                Number::from_f64(v).unwrap()
            }
        }
    }
}
