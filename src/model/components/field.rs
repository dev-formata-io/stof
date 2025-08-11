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

use std::collections::BTreeMap;
use arcstr::{literal, ArcStr};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use crate::{model::{DataRef, Graph, NodeRef, SPath, StofData, SELF_KEYWORD, SUPER_KEYWORD}, runtime::{Val, Variable}};

/// Marks a field as no export.
/// Used in export formats.
pub const NOEXPORT_FIELD_ATTR: ArcStr = literal!("no-export");

/// Can the field be viewed outside of its scope?
pub const PRIVATE_FIELD_ATTR: ArcStr = literal!("private");

/// Can this field be set or just read?
pub const READ_ONLY_FIELD_ATTR: ArcStr = literal!("readonly");


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Field.
/// Name specified by the object.
pub struct Field {
    pub value: Variable,
    pub attributes: FxHashMap<String, Val>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDoc {
    pub field: DataRef,
    pub docs: String,
}

#[typetag::serde(name = "_Field")]
impl StofData for Field {
    fn core_data(&self) -> bool {
        return true;
    }

    /// Does this data directly reference a node?
    /// If so, and you want this data to be removed when the node is removed, say yes.
    fn hard_node_ref(&self, node: &NodeRef) -> bool {
        if let Some(nref) = self.value.try_obj() {
            &nref == node
        } else {
            false
        }
    }

    /// Deep copy.
    fn deep_copy(&self, graph: &mut Graph, context: Option<NodeRef>) -> Box::<dyn StofData> {
        // For fields that are objects, there could be a lot of extra allocated memory for this op...
        let copied = self.value.deep_copy(graph, context);
        Box::new(Field {
            value: copied,
            attributes: self.attributes.clone(),
        })
    }
}
#[typetag::serde(name = "_FieldDoc")]
impl StofData for FieldDoc {
    fn core_data(&self) -> bool {
        return true;
    }
}
impl FieldDoc {
    /// Field docs on a node.
    pub fn docs(graph: &Graph, node: &NodeRef) -> FxHashMap<DataRef, String> {
        let mut docs = FxHashMap::default();
        if let Some(node) = node.node(graph) {
            for (_, dref) in &node.data {
                if let Some(doc) = graph.get_stof_data::<Self>(dref) {
                    docs.insert(doc.field.clone(), doc.docs.clone());
                }
            }
        }
        docs
    }
}

impl Field {
    /// Create a new field.
    pub fn new(value: Variable, attrs: Option<FxHashMap<String, Val>>) -> Self {
        let mut attributes = FxHashMap::default();
        if let Some(attr) = attrs {
            attributes = attr;
        }
        Self {
            value,
            attributes
        }
    }

    /// Can set this field?
    pub fn can_set(&self) -> bool {
        !self.attributes.contains_key(READ_ONLY_FIELD_ATTR.as_str())
    }

    /// Can read this field?
    pub fn is_private(&self) -> bool {
        self.attributes.contains_key(PRIVATE_FIELD_ATTR.as_str())
    }

    /// Get a field from a dot separated name path string.
    /// Ex. "root.hello" -> root object with a field named "hello". If hello is an object, a field might get created for it.
    pub fn field_from_path(graph: &mut Graph, path: &str, start: Option<NodeRef>) -> Option<DataRef> {
        let mut spath = SPath::from(path);
        if spath.path.is_empty() { return None; }
        
        let field_name = spath.path.pop().unwrap();
        if let Some(node) = SPath::node(&graph, spath, start) {
            return Self::field(graph, &node, field_name.as_ref());
        }
        None
    }
    
    #[inline]
    /// Field lookup, but does not create a field for a child node if needed.
    /// This is used for complex node relationships in path finding...
    pub fn direct_field(graph: &Graph, node: &NodeRef, field_name: &str) -> Option<DataRef> {
        if let Some(node) = node.node(graph) {
            if let Some(dref) = node.data.get(field_name) {
                if let Some(field) = graph.get_stof_data::<Self>(dref) {
                    if !field.value.dangling_obj(graph) {
                        return Some(dref.clone());
                    }
                }
            }
        }
        None
    }

    /// Field lookup an a graph from a singular node and name.
    /// Lazily creates a field for a child node if needed.
    pub fn field(graph: &mut Graph, node: &NodeRef, field_name: &str) -> Option<DataRef> {
        let mut created = None;
        let mut self_parent = None;
        let mut self_name = None;
        if let Some(node) = node.node(&graph) {
            if let Some(dref) = node.data.get(field_name) {
                if let Some(field) = graph.get_stof_data::<Self>(dref) {
                    if !field.value.dangling_obj(graph) {
                        return Some(dref.clone());
                    }
                }
            }
            for child in &node.children {
                if let Some(child) = child.node(&graph) {
                    if child.name.as_ref() == field_name && child.is_field() {
                        let mut attrs = child.attributes.clone();
                        attrs.insert(NOEXPORT_FIELD_ATTR.to_string(), Val::Null); // don't export these lazily created fields
                        let var = Variable::new(graph, true, Val::Obj(child.id.clone()), false);
                        created = Some(Self::new(var, Some(attrs)));
                        break;
                    }
                }
            }

            // Look for a parent with the field name (that is also a field)
            let mut gp = None;
            if let Some(parent) = &node.parent {
                if let Some(parent) = parent.node(&graph) {
                    if (parent.name.as_ref() == field_name || field_name == SUPER_KEYWORD.as_ref()) && parent.is_field() {
                        if let Some(grand) = &parent.parent {
                            if grand.node_exists(&graph) {
                                gp = Some((grand.clone(), parent.name.clone()));
                            }
                        }
                    }
                }
            }
            if let Some((gp, field_name)) = gp {
                return Self::field(graph, &gp, field_name.as_ref());
            }

            // Is this node the thing?
            if (node.name.as_ref() == field_name || field_name == SELF_KEYWORD.as_ref()) && node.is_field() {
                if let Some(parent) = &node.parent {
                    if parent.node_exists(graph) {
                        self_parent = Some(parent.clone());
                        self_name = Some(node.name.clone());
                    }
                }
            }
        }
        if let Some(parent) = self_parent {
            if let Some(name) = self_name {
                return Self::field(graph, &parent, name.as_ref());
            }
        }
        if let Some(field) = created {
            if let Some(dref) = graph.insert_stof_data(node, field_name, Box::new(field), None) {
                return Some(dref);
            }
        }
        None
    }

    /// Get all fields on a node.
    /// Will create object fields as needed.
    pub fn fields(graph: &mut Graph, node: &NodeRef) -> BTreeMap<String, DataRef> {
        let mut fields = BTreeMap::default();
        let mut to_create = Vec::new();

        if let Some(node) = node.node(&graph) {
            for (name, dref) in &node.data {
                if let Some(field) = graph.get_stof_data::<Self>(dref) {
                    if !field.value.dangling_obj(graph) {
                        fields.insert(name.clone(), dref.clone());
                    }
                }
            }

            for child in &node.children {
                if let Some(child) = child.node(&graph) {
                    if child.is_field() && !fields.contains_key(child.name.as_ref()) {
                        let mut attrs = child.attributes.clone();
                        attrs.insert(NOEXPORT_FIELD_ATTR.to_string(), Val::Null); // don't export these lazily created fields
                        let var = Variable::new(graph, true, Val::Obj(child.id.clone()), false);
                        to_create.push((child.name.clone(), Self::new(var, Some(attrs))));
                    }
                }
            }
        }

        for (name, field) in to_create {
            if let Some(dref) = graph.insert_stof_data(node, name.clone(), Box::new(field), None) {
                fields.insert(name.to_string(), dref);
            }
        }
        
        fields
    }
}
