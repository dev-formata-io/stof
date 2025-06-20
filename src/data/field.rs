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

use std::{collections::{BTreeMap, HashSet}, ops::Deref};
use serde::{Deserialize, Serialize};
use crate::{Data, SData, SDataRef, SGraph, SNodeRef};
use super::{lang::{ErrorType, SError}, SNum, SUnits, SVal};


/// Stof field.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SField {
    pub name: String,
    pub value: SVal,
    pub attributes: BTreeMap<String, SVal>,
}

#[typetag::serde(name = "_SField")]
impl Data for SField {
    fn core_data(&self) -> bool {
        return true;
    }
}


/// Stof field doc.
/// Optionally added in parallel to the field to document it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SFieldDoc {
    pub field: SDataRef,
    pub docs: String,
}
impl SFieldDoc {
    /// Create a new fielddoc.
    pub fn new(field: SDataRef, docs: String) -> Self {
        Self {
            field,
            docs
        }
    }

    /// Get references to all field docs on a node.
    pub fn field_docs<'a>(graph: &'a SGraph, node: &SNodeRef) -> Vec<&'a Self> {
        if let Some(node) = node.node(graph) {
            return node.data::<Self>(graph);
        }
        vec![]
    }
}

#[typetag::serde(name = "_SFieldDoc")]
impl Data for SFieldDoc {
    fn core_data(&self) -> bool {
        return true;
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

    /// To string.
    pub fn to_string(&self) -> String {
        self.value.to_string()
    }

    /// Is an object?
    pub fn is_object(&self) -> bool {
        self.value.is_object()
    }

    /// Get references to all fields on a node.
    pub fn fields<'a>(graph: &'a SGraph, node: &SNodeRef) -> Vec<&'a Self> {
        if let Some(node) = node.node(graph) {
            return node.data::<Self>(graph);
        }
        vec![]
    }

    /// Get field data refs to all fields on a node.
    pub fn field_refs(graph: &SGraph, node: &SNodeRef) -> Vec<SDataRef> {
        if let Some(node) = node.node(graph) {
            return node.data_refs::<Self>(graph);
        }
        vec![]
    }

    /// Get a field from a path with the given separator.
    /// Last name in the path is the field name.
    /// If path is only the field, will search on start if any or search each root in the graph.
    pub fn field<'a>(graph: &'a SGraph, path: &str, sep: char, start: Option<&SNodeRef>) -> Option<&'a Self> {
        let mut items: Vec<&str> = path.split(sep).collect();

        let field_name = items.pop().unwrap();
        if items.len() > 0 {
            if let Some(node) = graph.node_from(&items.join("/"), start) {
                for field in node.data::<Self>(graph) {
                    if field.name == field_name {
                        return Some(field);
                    }
                }
            }
        } else {
            if let Some(start) = start {
                if let Some(node) = start.node(graph) {
                    for field in node.data::<Self>(graph) {
                        if field.name == field_name {
                            return Some(field);
                        }
                    }
                }
            } else {
                for root_ref in &graph.roots {
                    if let Some(node) = root_ref.node(graph) {
                        for field in node.data::<Self>(graph) {
                            if field.name == field_name {
                                return Some(field);
                            }
                        }
                    }
                }
            }
        }
        None
    }


    /// Get a field data reference from a path with the given separator.
    /// Last name in the path is the field name.
    /// If path is only the field, will search on start if any or search each root in the graph.
    pub fn field_ref(graph: &SGraph, path: &str, sep: char, start: Option<&SNodeRef>) -> Option<SDataRef> {
        let mut items: Vec<&str> = path.split(sep).collect();

        let field_name = items.pop().unwrap();
        if items.len() > 0 {
            if let Some(node) = graph.node_from(&items.join("/"), start) {
                for dref in &node.data {
                    if let Some(field) = SData::get::<Self>(graph, dref) {
                        if field.name == field_name {
                            return Some(dref.clone());
                        }
                    }
                }
            }
        } else {
            if let Some(start) = start {
                if let Some(node) = start.node(graph) {
                    for dref in &node.data {
                        if let Some(field) = SData::get::<Self>(graph, dref) {
                            if field.name == field_name {
                                return Some(dref.clone());
                            }
                        }
                    }
                }
            } else {
                for root_ref in &graph.roots {
                    if let Some(node) = root_ref.node(graph) {
                        for dref in &node.data {
                            if let Some(field) = SData::get::<Self>(graph, dref) {
                                if field.name == field_name {
                                    return Some(dref.clone());
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
        let field = Self::new(name.into(), SVal::Object(nref.clone()));
        SData::insert_new(graph, parent, Box::new(field));
        nref
    }

    /// Insert an array field.
    #[inline]
    pub fn new_array(graph: &mut SGraph, name: &str, vals: Vec<SVal>, node: &SNodeRef) -> Option<SDataRef> {
        let field = Self::new(name.into(), SVal::Array(vals));
        SData::insert_new(graph, node, Box::new(field))
    }

    /// Insert a new string field.
    #[inline]
    pub fn new_string(graph: &mut SGraph, name: &str, value: &str, node: &SNodeRef) -> Option<SDataRef> {
        let field = Self::new(name.into(), SVal::String(value.into()));
        SData::insert_new(graph, node, Box::new(field))
    }

    /// Insert a new boolean field.
    #[inline]
    pub fn new_bool(graph: &mut SGraph, name: &str, value: bool, node: &SNodeRef) -> Option<SDataRef> {
        let field = Self::new(name.into(), SVal::Bool(value));
        SData::insert_new(graph, node, Box::new(field))
    }

    /// New integer field.
    #[inline]
    pub fn new_int(graph: &mut SGraph, name: &str, value: i64, node: &SNodeRef) -> Option<SDataRef> {
        let field = Self::new(name.into(), SVal::Number(SNum::I64(value)));
        SData::insert_new(graph, node, Box::new(field))
    }

    /// New float field.
    #[inline]
    pub fn new_float(graph: &mut SGraph, name: &str, value: f64, node: &SNodeRef) -> Option<SDataRef> {
        let field = Self::new(name.into(), SVal::Number(SNum::F64(value)));
        SData::insert_new(graph, node, Box::new(field))
    }

    /// New float units field.
    #[inline]
    pub fn new_units(graph: &mut SGraph, name: &str, value: f64, units: SUnits, node: &SNodeRef) -> Option<SDataRef> {
        let field = Self::new(name.into(), SVal::Number(SNum::Units(value, units)));
        SData::insert_new(graph, node, Box::new(field))
    }


    /*****************************************************************************
     * Boolean operations between fields.
     *****************************************************************************/

    /// Union two sets of fields together by name, manipulating the first set of fields
    pub(crate) fn merge_fields(graph: &mut SGraph, node: &SNodeRef, other: &SGraph, other_fields: &Vec<SDataRef>) -> Result<(), SError> {
        let mut other_handled = HashSet::new();
        let fields = Self::field_refs(graph, node);
        for field_ref in fields {
            if let Some(field) = SData::get_mut::<SField>(graph, field_ref) {
                for other_ref in other_fields {
                    if let Some(other) = SData::get::<SField>(other, other_ref) {
                        if field.name == other.name {
                            if field == other || (field.is_object() && other.is_object()) {
                                // do nothing...
                            } else {
                                field.merge(other)?;
                            }
                            other_handled.insert(other_ref.clone());
                        }
                    }
                }
            }
        }
        'other_fields: for other_ref in other_fields {
            if !other_handled.contains(other_ref) {
                if let Some(other) = SData::get::<SField>(other, other_ref) {
                    let cloned = other.clone();
                    match &cloned.value {
                        SVal::Object(nref) => {
                            if !nref.exists(graph) {
                                continue 'other_fields;
                            }
                        },
                        SVal::Data(dref) => {
                            if !dref.exists(graph) {
                                continue 'other_fields;
                            }
                        },
                        SVal::FnPtr(dref) => {
                            if !dref.exists(graph) {
                                continue 'other_fields;
                            }
                        },
                        SVal::Boxed(val) => {
                            let val = val.lock().unwrap();
                            let val = val.deref();
                            match val {
                                SVal::Object(nref) => {
                                    if !nref.exists(graph) {
                                        continue 'other_fields;
                                    }
                                },
                                SVal::Data(dref) => {
                                    if !dref.exists(graph) {
                                        continue 'other_fields;
                                    }
                                },
                                SVal::FnPtr(dref) => {
                                    if !dref.exists(graph) {
                                        continue 'other_fields;
                                    }
                                },
                                _ => {}
                            }
                        },
                        _ => {}
                    }
                    SData::insert_new_id(graph, node, Box::new(cloned), &other_ref.id);
                }
            }
        }
        Ok(())
    }

    /// Union two fields together, manipulating self with the new unioned value.
    /// no attribute | #[merge] | #[merge('default')] - default merge is performed
    /// #[merge('none')] - ignore any merging - value is unchanged
    /// #[merge('override')] - other value is accepted INSTEAD of current value
    /// #[merge('error')] - throw an error if the value needs to be merged
    /// TODO #[merge(fn)] - custom merge handler function with the two values
    pub(crate) fn merge(&mut self, other: &Self) -> Result<(), SError> {
        for (attr, val) in &other.attributes {
            if !self.attributes.contains_key(attr.as_str()) {
                self.attributes.insert(attr.clone(), val.clone());
            }
        }

        if let Some(attr_val) = self.attributes.get("merge") {
            if attr_val.is_empty() {
                self.value.merge(&other.value);
            } else {
                match attr_val {
                    SVal::String(merge_type) => {
                        match merge_type.as_str() {
                            "none" => {
                                // don't do any merging!
                            },
                            "override" => {
                                // take the other value instead of our current value
                                self.value = other.value.clone();
                            },
                            "error" => {
                                return Err(SError {
                                    pid: "main".to_string(),
                                    error_type: ErrorType::Custom("MergeError".into()),
                                    message: format!("user instructed merge error between two fields, both named {}'", &self.name),
                                    call_stack: Default::default(),
                                });
                            },
                            _ => { // default merge
                                self.value.merge(&other.value);
                            }
                        }
                    },
                    //SVal::FnPtr(func_ref) => {
                        // TODO
                    //},
                    _ => {
                        self.value.merge(&other.value);
                    }
                }
            }
        } else {
            self.value.merge(&other.value);
        }
        Ok(())
    }
}
