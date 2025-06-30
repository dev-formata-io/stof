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

use bytes::Bytes;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use crate::{model::{DataRef, Graph, NodeRef, SId, SPath, StofData}, runtime::{Val, Variable}};

/// Marks a field as no export.
/// Used in export formats.
pub const NOEXPORT_FIELD_ATTR: SId = SId(Bytes::from_static(b"no-export"));


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Field.
/// Name specified by the object.
pub struct Field {
    pub value: Variable,
    pub attributes: FxHashMap<SId, Variable>,
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
}

impl Field {
    /// Create a new field.
    pub fn new(value: Variable, attrs: Option<FxHashMap<SId, Variable>>) -> Self {
        let mut attributes = FxHashMap::default();
        if let Some(attr) = attrs {
            attributes = attr;
        }
        Self {
            value,
            attributes
        }
    }

    /// Get a field from a dot separated name path string.
    /// Ex. "root.hello" -> root object with a field named "hello". If hello is an object, a field might get created for it.
    pub fn field_from_path(graph: &mut Graph, path: &str, start: Option<NodeRef>) -> Option<DataRef> {
        let mut spath = SPath::from(path);
        if spath.path.is_empty() { return None; }
        
        let field_name = spath.path.pop().unwrap();
        if let Some(id_path) = spath.to_id_path(&graph, start) {
            if let Some(node) = id_path.node(&graph) {
                return Self::field(graph, &node, &field_name);
            }
        }
        None
    }
    
    #[inline]
    /// Field lookup, but does not create a field for a child node if needed.
    /// This is used for complex node relationships in path finding...
    pub fn direct_field(graph: &Graph, node: &NodeRef, field_name: &SId) -> Option<DataRef> {
        if let Some(node) = node.node(graph) {
            if let Some(dref) = node.data.get(field_name) {
                if dref.type_of::<Self>(&graph) {
                    return Some(dref.clone());
                }
            }
        }
        None
    }

    /// Field lookup an a graph from a singular node and name.
    /// Lazily creates a field for a child node if needed.
    pub fn field(graph: &mut Graph, node: &NodeRef, field_name: &SId) -> Option<DataRef> {
        let mut created = None;
        if let Some(node) = node.node(&graph) {
            if let Some(dref) = node.data.get(field_name) {
                if dref.type_of::<Self>(&graph) {
                    return Some(dref.clone());
                }
            }
            for child in &node.children {
                if let Some(child) = child.node(&graph) {
                    if &child.name == field_name && child.is_field() {
                        let mut attrs = child.attributes.clone();
                        attrs.insert(NOEXPORT_FIELD_ATTR, Variable::Val(Val::Null)); // don't export these lazily created fields
                        created = Some(Self::new(Variable::Val(Val::Obj(child.id.clone())), Some(attrs)));
                        break;
                    }
                }
            }
        }
        if let Some(field) = created {
            if let Some(dref) = graph.insert_stof_data(node, field_name, Box::new(field), None) {
                return Some(dref);
            }
        }
        None
    }
}
