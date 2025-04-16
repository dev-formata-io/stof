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
            map.insert(field.name.clone(), toml_value(graph, field.value.clone()));
        }
    }
    map
}


/// Export value from an array of values.
fn value_from_array(graph: &SGraph, vals: Vec<SVal>) -> Value {
    let mut results: Vec<Value> = Vec::new();
    for val in vals {
        results.push(toml_value(graph, val));
    }
    Value::Array(results)
}


/// Get a toml value from a stof value.
fn toml_value(graph: &SGraph, val: SVal) -> Value {
    match val {
        SVal::Boxed(val) => toml_value(graph, val.lock().unwrap().clone()),
        SVal::String(val) => Value::String(val),
        SVal::SemVer { major: _, minor: _, patch: _, release: _, build: _ } => Value::String(val.to_string()),
        SVal::Bool(val) => Value::Boolean(val),
        SVal::Number(val) => {
            match val {
                SNum::F64(v) => Value::Float(v),
                SNum::I64(v) => Value::Integer(v),
                SNum::Units(v, _) => Value::Float(v),
            }
        },
        SVal::Blob(blob) => {
            let mut array = Vec::new();
            for v in blob {
                array.push(Value::Integer(v as i64));
            }
            Value::Array(array)
        },
        SVal::FnPtr(ptr) => Value::String(format!("fn({})", ptr.id)),
        SVal::Data(ptr) => Value::String(format!("data({})", ptr.id)),
        SVal::Set(set) => value_from_array(graph, set.into_iter().collect()),
        SVal::Array(vals) => value_from_array(graph, vals),
        SVal::Tuple(vals) => value_from_array(graph, vals),
        SVal::Object(nref) => Value::Table(toml_value_from_node(graph, &nref)),
        SVal::Map(stof_map) => {
            let mut table = Table::new();
            for (k, v) in stof_map {
                let key = k.to_string();
                let value = toml_value(graph, v);
                table.insert(key, value);
            }
            Value::Table(table)
        },
        _ => {
            Value::String("unknown Stof value".to_string())
        }
    }
}
