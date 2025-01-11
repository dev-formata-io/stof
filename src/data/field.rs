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

use std::collections::{BTreeMap, HashMap, HashSet};
use serde::{Deserialize, Serialize};
use crate::{Data, IntoDataRef, SData, SDataRef, SGraph, SNodeRef};
use super::{SNum, SUnits, SVal};


/// Stof field kind.
pub const FKIND: &str = "fld";


/// Stof field.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SField {
    /// ID of this field.
    /// This will also be the SDataRef ID.
    pub id: String,

    /// Name of this field.
    pub name: String,

    /// Value of this field.
    pub value: SVal,

    /// Attributes.
    pub attributes: BTreeMap<String, SVal>,
}
impl IntoDataRef for SField {
    fn data_ref(&self) -> SDataRef {
        SDataRef::from(&self.id)
    }
}
impl Data for SField {
    fn kind(&self) -> String {
        FKIND.to_string()
    }
    fn set_ref(&mut self, to_ref: impl IntoDataRef) {
        self.id = to_ref.data_ref().id;
    }
}
impl PartialOrd for SField {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(&other.name)
    }
}
impl PartialEq for SField {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.value == other.value
    }
}
impl Eq for SField {}
impl SField {
    /// Create a new field.
    pub fn new(name: &str, value: impl Into<SVal>) -> Self {
        Self {
            name: name.to_owned(),
            value: value.into(),
            ..Default::default()
        }
    }

    /// Schema equals?
    /// Requires that the name and value types be the same.
    pub fn schema_eq(&self, other: &Self, graph: &SGraph) -> bool {
        self.name == other.name && self.value.schema_eq(&other.value, graph)
    }

    /// Is null field?
    pub fn is_null(&self) -> bool {
        match &self.value {
            SVal::Null => { true }
            _ => { false }
        }
    }

    /// Is boolean field?
    pub fn is_bool(&self) -> bool {
        match &self.value {
            SVal::Bool(_) => { true }
            _ => { false }
        }
    }

    /// Get boolean value for this field.
    pub fn bool(&self) -> bool {
        match &self.value {
            SVal::Bool(val) => { *val }
            _ => { false }
        }
    }

    /// Is a number field?
    pub fn is_number(&self) -> bool {
        match &self.value {
            SVal::Number(_) => { true }
            _ => { false }
        }
    }

    /// Get the number value from this field.
    pub fn number(&self) -> SNum {
        match &self.value {
            SVal::Number(val) => { val.clone() }
            _ => { 0.into() }
        }
    }

    /// Integer representation of this number.
    pub fn integer(&self) -> i64 {
        match &self.value {
            SVal::Number(val) => { val.int() }
            _ => { 0.into() }
        }
    }

    /// Get number float value from this field.
    pub fn float(&self) -> f64 {
        match &self.value {
            SVal::Number(val) => { val.float() }
            _ => 0.
        }
    }

    /// Float with units.
    pub fn float_with_units(&self, units: SUnits) -> f64 {
        match &self.value {
            SVal::Number(val) => val.float_with_units(units),
            _ => 0.
        }
    }

    /// Is a string?
    pub fn is_string(&self) -> bool {
        match &self.value {
            SVal::String(_) => { true }
            _ => { false }
        }
    }

    /// Get string value from this field.
    pub fn string(&self) -> String {
        match &self.value {
            SVal::String(val) => { val.clone() }
            _ => { Default::default() }
        }
    }

    /// To string.
    pub fn to_string(&self) -> String {
        self.value.to_string()
    }

    /// Is an object?
    pub fn is_object(&self) -> bool {
        self.value.is_object()
    }

    /// Get object value from this field.
    pub fn object(&self) -> Option<SNodeRef> {
        match &self.value {
            SVal::Object(val) => { Some(val.clone()) }
            _ => { None }
        }
    }

    /// Is an array?
    pub fn is_array(&self) -> bool {
        match &self.value {
            SVal::Array(_) => { true }
            _ => { false }
        }
    }

    /// Get array value from this field.
    pub fn array(&self) -> Vec<SVal> {
        match &self.value {
            SVal::Array(vals) => { vals.clone() }
            _ => { vec![] }
        }
    }

    /// Get all fields on a node.
    pub fn fields(graph: &SGraph, node: &SNodeRef) -> Vec<Self> {
        let mut res = Vec::new();
        if let Some(node) = node.node(graph) {
            for dref in node.prefix_selection(FKIND) {
                if let Ok(field) = SData::data::<SField>(graph, dref) {
                    res.push(field);
                }
            }
        }
        res
    }

    /// Get all fields on a node as a hashmap.
    pub fn fields_map(graph: &SGraph, node: &SNodeRef) -> HashMap<String, Self> {
        let mut res = HashMap::new();
        if let Some(node) = node.node(graph) {
            for dref in node.prefix_selection(FKIND) {
                if let Ok(field) = SData::data::<SField>(graph, dref) {
                    res.insert(field.name.clone(), field);
                }
            }
        }
        res
    }

    /// Get an adjacent field to this field.
    pub fn adjacent(&self, graph: &SGraph, path: &str, sep: char) -> Option<Self> {
        if let Some(data) = self.data_ref().data(graph) {
            for node_ref in &data.nodes {
                let field = Self::field(graph, path, sep, Some(node_ref));
                if field.is_some() {
                    return field;
                }
            }
        }
        None
    }

    /// Get the first field that matches a path given.
    pub fn first_match(graph: &SGraph, paths: Vec<&str>, sep: char, start: Option<&SNodeRef>) -> Option<Self> {
        for path in paths {
            let field = Self::field(graph, path, sep, start);
            if field.is_some() {
                return field;
            }
        }
        None
    }

    /// Get a field from a path with the given separator.
    /// Last name in the path is the field name.
    /// If path is only the field, will search on start if any or search each root in the graph.
    pub fn field(graph: &SGraph, path: &str, sep: char, start: Option<&SNodeRef>) -> Option<Self> {
        let mut items: Vec<&str> = path.split(sep).collect();

        let field_name = items.pop().unwrap();
        if items.len() > 0 {
            if let Some(node) = graph.node_from(&items.join("/"), start) {
                for dref in node.prefix_selection(FKIND) {
                    if let Ok(field) = SData::data::<SField>(graph, dref) {
                        if field.name == field_name {
                            return Some(field);
                        }
                    }
                }
            }
        } else {
            if let Some(start) = start {
                if let Some(node) = start.node(graph) {
                    for dref in node.prefix_selection(FKIND) {
                        if let Ok(field) = SData::data::<SField>(graph, dref) {
                            if field.name == field_name {
                                return Some(field);
                            }
                        }
                    }
                }
            } else {
                for root_ref in &graph.roots {
                    if let Some(node) = root_ref.node(graph) {
                        for dref in node.prefix_selection(FKIND) {
                            if let Ok(field) = SData::data::<SField>(graph, dref) {
                                if field.name == field_name {
                                    return Some(field);
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }


    /*****************************************************************************
     * Field Creation Helpers.
     *****************************************************************************/
    
    /// Helper function for inserting an object field into a graph.
    /// A new node will be inserted into the graph with the name "name".
    /// A field will be created on the parent that is an object with name "name".
    #[inline]
    pub fn new_object(graph: &mut SGraph, name: &str, parent: &SNodeRef) -> SNodeRef {
        let nref = graph.insert_node(name, Some(parent));
        let mut field = Self::new(name.into(), SVal::Object(nref.clone()));
        field.attach(parent, graph);
        nref
    }

    /// Insert an array field.
    #[inline]
    pub fn new_array(graph: &mut SGraph, name: &str, vals: Vec<SVal>, node: &SNodeRef) -> Self {
        let mut field = Self::new(name.into(), SVal::Array(vals));
        field.attach(node, graph);
        field
    }

    /// Insert a new string field.
    #[inline]
    pub fn new_string(graph: &mut SGraph, name: &str, value: &str, node: &SNodeRef) -> Self {
        let mut field = Self::new(name.into(), SVal::String(value.into()));
        field.attach(node, graph);
        field
    }

    /// Insert a new boolean field.
    #[inline]
    pub fn new_bool(graph: &mut SGraph, name: &str, value: bool, node: &SNodeRef) -> Self {
        let mut field = Self::new(name.into(), SVal::Bool(value));
        field.attach(node, graph);
        field
    }

    /// New integer field.
    #[inline]
    pub fn new_int(graph: &mut SGraph, name: &str, value: i64, node: &SNodeRef) -> Self {
        let mut field = Self::new(name.into(), SVal::Number(SNum::I64(value)));
        field.attach(node, graph);
        field
    }

    /// New float field.
    #[inline]
    pub fn new_float(graph: &mut SGraph, name: &str, value: f64, node: &SNodeRef) -> Self {
        let mut field = Self::new(name.into(), SVal::Number(SNum::F64(value)));
        field.attach(node, graph);
        field
    }

    /// New float units field.
    #[inline]
    pub fn new_units(graph: &mut SGraph, name: &str, value: f64, units: SUnits, node: &SNodeRef) -> Self {
        let mut field = Self::new(name.into(), SVal::Number(SNum::Units(value, units)));
        field.attach(node, graph);
        field
    }


    /*****************************************************************************
     * Boolean operations between fields.
     *****************************************************************************/

    /// Union two sets of fields together by name, manipulating the first set of fields
    pub(crate) fn union_fields(fields: &mut Vec<Self>, other_fields: &Vec<Self>) {
        let mut other_handled = HashSet::new();
        for field in &mut *fields {
            for other in other_fields {
                if field.name == other.name {
                    if field == other {
                        // Do nothing... other is already taken care of
                    } else {
                        field.union(other);
                    }
                    other_handled.insert(other.id.clone());
                }
            }
        }
        for other in other_fields {
            if !other_handled.contains(&other.id) {
                fields.push(other.clone()); // in other_fields, but wasn't in fields
            }
        }
    }

    /// Union two fields together, manipulating self with the new unioned value.
    pub(crate) fn union(&mut self, other: &Self) {
        for (attr, val) in &other.attributes {
            if !self.attributes.contains_key(attr.as_str()) {
                self.attributes.insert(attr.clone(), val.clone());
            }
        }
        self.value.union(&other.value);
    }
}
