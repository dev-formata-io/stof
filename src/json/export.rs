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
            map.insert(field.name.clone(), json_value(graph, field.value.clone()));
        }
    }
    Value::Object(map)
}


/// Export value from an array of values.
fn value_from_array(graph: &SGraph, vals: Vec<SVal>) -> Value {
    let mut results: Vec<Value> = Vec::new();
    for val in vals {
        results.push(json_value(graph, val));
    }
    Value::Array(results)
}


/// Get a JSON value from a Stof Value.
fn json_value(graph: &SGraph, val: SVal) -> Value {
    match val {
        SVal::Boxed(val) => json_value(graph, val.lock().unwrap().clone()),
        SVal::Void => Value::Null,
        SVal::Null => Value::Null,
        SVal::String(val) => Value::String(val),
        SVal::Bool(val) => Value::Bool(val),
        SVal::Number(val) => Value::Number(Number::from(val)),
        SVal::Blob(blob) => Value::from_iter(blob.into_iter()),
        SVal::FnPtr(ptr) => Value::String(format!("fn({})", ptr.id)),
        SVal::Object(nref) => json_value_from_node(graph, &nref),
        SVal::Set(set) => value_from_array(graph, set.into_iter().collect()),
        SVal::Array(vals) => value_from_array(graph, vals),
        SVal::Tuple(vals) => value_from_array(graph, vals),
        SVal::Map(stof_map) => {
            let mut json_map = Map::new();
            for (k, v) in stof_map {
                let key = k.to_string();
                let value = json_value(graph, v);
                json_map.insert(key, value);
            }
            Value::Object(json_map)
        },
    }
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
