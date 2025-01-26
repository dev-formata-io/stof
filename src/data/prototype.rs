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
use serde::{Deserialize, Serialize};
use crate::{IntoNodeRef, Data, SData, SDataRef, SDoc, SGraph, SNodeRef};
use super::{lang::CustomType, SField, SVal};


/// Stof prototype.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SPrototype {
    /// This is the Node ID (Ref) of the prototype.
    pub prototype: String,
}

#[typetag::serde(name = "_SProto")]
impl Data for SPrototype {}

impl IntoNodeRef for SPrototype {
    fn node_ref(&self) -> SNodeRef {
        SNodeRef::from(&self.prototype)
    }
}
impl SPrototype {
    /// Create a new prototype.
    pub fn new(node: impl IntoNodeRef) -> Self {
        Self {
            prototype: node.node_ref().id,
        }
    }

    /// Get a nodes prototype (if any).
    pub fn get(graph: &SGraph, node: impl IntoNodeRef) -> Option<&Self> {
        if let Some(node) = node.node_ref().node(graph) {
            for proto in node.data::<Self>(graph) {
                return Some(proto);
            }
        }
        None
    }

    /// Get a nodes prototype ref (if any).
    pub fn get_ref(graph: &SGraph, node: impl IntoNodeRef) -> Option<SDataRef> {
        if let Some(node) = node.node_ref().node(graph) {
            for dref in &node.data {
                if let Some(_proto) = SData::get::<SPrototype>(graph, dref) {
                    return Some(dref.clone());
                }
            }
        }
        None
    }

    /// Get this prototypes "typepath" field.
    pub fn typepath(&self, graph: &SGraph) -> Option<String> {
        if let Some(typepath) = SField::field(graph, "typepath", '.', Some(&self.node_ref())) {
            return Some(typepath.to_string());
        }
        None
    }

    /// Typepath stack.
    pub fn typepath_stack(&self, graph: &SGraph) -> Vec<String> {
        let mut type_stack = Vec::new();
        let mut current = Some(self.node_ref());
        while let Some(typename) = SField::field(graph, "typepath", '.', current.as_ref()) {
            type_stack.push(typename.to_string());

            if let Some(node) = current.unwrap().node(graph) {
                if let Some(parent_ref) = &node.parent {
                    current = Some(parent_ref.clone());
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        type_stack
    }

    /// Get this prototypes "typename" field.
    pub fn typename(&self, graph: &SGraph) -> Option<String> {
        if let Some(typename) = SField::field(graph, "typename", '.', Some(&self.node_ref())) {
            return Some(typename.to_string());
        }
        None
    }

    /// Type stack.
    pub fn type_stack(&self, graph: &SGraph) -> Vec<String> {
        let mut type_stack = Vec::new();
        let mut current = Some(self.node_ref());
        while let Some(typename) = SField::field(graph, "typename", '.', current.as_ref()) {
            type_stack.push(typename.to_string());

            if let Some(node) = current.unwrap().node(graph) {
                if let Some(parent_ref) = &node.parent {
                    current = Some(parent_ref.clone());
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        type_stack
    }

    /// Custom type for this prototype.
    pub fn custom_type<'a>(&self, doc: &'a SDoc) -> Option<&'a CustomType> {
        if let Some(typename) = self.typename(&doc.graph) {
            if let Some(ctypes) = doc.types.types.get(&typename) {
                for ctype in ctypes {
                    if ctype.locid == self.prototype {
                        return Some(ctype);
                    }
                }
            }
        }
        None
    }

    /// Attributes for this prototype.
    pub fn attributes(&self, doc: &SDoc) -> BTreeMap<String, SVal> {
        if let Some(ctype) = self.custom_type(doc) {
            return ctype.attributes.clone();
        }
        BTreeMap::default()
    }
}
