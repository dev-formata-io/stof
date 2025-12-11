//
// Copyright 2025 Formata, Inc. All rights reserved.
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
use toml::{Table, Value};
use crate::{model::{Field, Graph, NodeRef, SId, NOEXPORT_FIELD_ATTR}, runtime::{Val, ValRef, Variable}};


pub(super) fn parse_toml_object_value(graph: &mut Graph, node: &NodeRef, table: Table) {
    for (field, val) in table {
        let tf = parse_toml_field_value(graph, node, val, &field);
        graph.insert_stof_data(node, &field, Box::new(tf), None);
    }
}

pub(super) fn parse_toml_field_value(graph: &mut Graph, node: &NodeRef, value: Value, field: &str) -> Field {
    match value {
        Value::Integer(val) => {
            Field::new(Variable::new(graph, true, Val::from(val), false), None)
        },
        Value::Float(val) => {
            Field::new(Variable::new(graph, true, Val::from(val), false), None)
        },
        Value::String(val) => {
            Field::new(Variable::new(graph, true, Val::Str(val.into()), false), None)
        },
        Value::Boolean(val) => {
            Field::new(Variable::new(graph, true, Val::from(val), false), None)
        },
        Value::Datetime(val) => {
            Field::new(Variable::new(graph, true, Val::Str(val.to_string().into()), false), None)
        },
        Value::Array(vals) => {
            let mut tf_arr = Vector::default();
            parse_toml_array_values(graph, node, vals, &mut tf_arr);
            Field::new(Variable::new(graph, true, Val::List(tf_arr), false), None)
        },
        Value::Table(value) => {
            let child_node = graph.insert_node(field, Some(node.clone()), true);
            parse_toml_object_value(graph, &child_node, value);

            let mut attrs = FxHashMap::default();
            attrs.insert(NOEXPORT_FIELD_ATTR.to_string(), Val::Null); // don't export object fields
            Field::new(Variable::new(graph, true, Val::Obj(child_node), false), Some(attrs))
        },
    }
}

pub(super) fn parse_toml_array_values(graph: &mut Graph, node: &NodeRef, vals: Vec<Value>, res: &mut Vector<ValRef<Val>>) {
    for val in vals {
        match val {
            Value::Integer(val) => {
                res.push_back(ValRef::new(Val::from(val)));
            },
            Value::Float(val) => {
                res.push_back(ValRef::new(Val::from(val)));
            },
            Value::String(val) => {
                res.push_back(ValRef::new(Val::Str(val.into())));
            },
            Value::Boolean(val) => {
                res.push_back(ValRef::new(Val::Bool(val)));
            },
            Value::Datetime(val) => {
                res.push_back(ValRef::new(Val::Str(val.to_string().into())));  
            },
            Value::Array(vals) => {
                let mut tf_arr = Vector::default();
                parse_toml_array_values(graph, node, vals, &mut tf_arr);
                res.push_back(ValRef::new(Val::List(tf_arr)));
            },
            Value::Table(table) => {
                let id = SId::default();
                let child_node = graph.insert_node_id(&id, &id, Some(node.clone()), false);
                parse_toml_object_value(graph, &child_node, table);
                res.push_back(ValRef::new(Val::Obj(child_node)));
            },
        }
    }
}
