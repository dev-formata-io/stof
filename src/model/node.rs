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

use std::collections::{BTreeMap, BTreeSet};
use bytes::Bytes;
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use crate::{model::{DataRef, NodeRef, SId}, runtime::Val};


/// Invalid/dirty new symbol.
pub const INVALID_NODE_NEW: SId = SId(Bytes::from_static(b"new"));

/// Invalid/dirty name symbol.
pub const INVALID_NODE_NAME: SId = SId(Bytes::from_static(b"name"));

/// Invalid/dirty parent symbol.
pub const INVALID_NODE_PARENT: SId = SId(Bytes::from_static(b"parent"));

/// Invalid/dirty children symbol.
pub const INVALID_NODE_CHILDREN: SId = SId(Bytes::from_static(b"children"));

/// Invalid/dirty data symbol.
pub const INVALID_NODE_DATA: SId = SId(Bytes::from_static(b"data"));

/// Invalid/dirty attributes symbol.
pub const INVALID_NODE_ATTRS: SId = SId(Bytes::from_static(b"attributes"));

/// Field node attribute.
/// Used for lazy field creation of nodes.
const FIELD_NODE_ATTR: SId = SId(Bytes::from_static(b"field"));


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// Node.
pub struct Node {
    pub id: NodeRef,
    pub name: SId,
    pub parent: Option<NodeRef>,
    pub children: BTreeSet<NodeRef>,  // keeps order
    pub data: BTreeMap<SId, DataRef>, // keeps order
    pub attributes: FxHashMap<SId, Val>,

    #[serde(skip)]
    pub dirty: FxHashSet<SId>,
}
impl Node {
    /// Create a new node.
    pub fn new(name: SId, id: NodeRef, field: bool) -> Self {
        let mut attributes = FxHashMap::default();
        if field {
            // marks this node as also a field
            attributes.insert(FIELD_NODE_ATTR, Val::Null);
        }
        Self {
            id,
            name,
            parent: None,
            children: Default::default(),
            data: Default::default(),
            dirty: Default::default(),
            attributes,
        }
    }

    #[inline(always)]
    /// Is this node also a field?
    pub fn is_field(&self) -> bool {
        self.attributes.contains_key(&FIELD_NODE_ATTR)
    }

    #[inline]
    /// Make this node a field.
    /// Returns whether this object was not previously a field (or if changed as thats easier to think about).
    pub fn make_field(&mut self) -> bool {
        let res = self.attributes.insert(FIELD_NODE_ATTR, Val::Null).is_none();
        if res {
            self.invalidate_attrs();
        }
        res
    }

    /// Make this node not a field.
    /// Does not remove any fields if some have been created for this node.
    /// Avoid switching nodes to and from fields... this is for the graph (external insert, etc.).
    /// Returns whether this object was previously a field or not (or if changed).
    #[inline]
    pub fn not_field(&mut self) -> bool {
        let res = self.attributes.remove(&FIELD_NODE_ATTR).is_some();
        if res {
            self.invalidate_attrs();
        }
        res
    }

    #[inline(always)]
    /// Invalidate with a symbol.
    pub fn invalidate(&mut self, symbol: SId) -> bool {
        self.dirty.insert(symbol)
    }

    #[inline(always)]
    /// Invalidate name.
    pub fn invalidate_name(&mut self) -> bool {
        self.invalidate(INVALID_NODE_NAME)
    }

    #[inline(always)]
    /// Invalidate attributes.
    pub fn invalidate_attrs(&mut self) -> bool {
        self.invalidate(INVALID_NODE_ATTRS)
    }

    #[inline(always)]
    /// Invalidate parent.
    pub fn invalidate_parent(&mut self) -> bool {
        self.invalidate(INVALID_NODE_PARENT)
    }

    #[inline(always)]
    /// Invlidate children.
    pub fn invalidate_children(&mut self) -> bool {
        self.invalidate(INVALID_NODE_CHILDREN)
    }

    #[inline(always)]
    /// Invalidate data.
    pub fn invalidate_data(&mut self) -> bool {
        self.invalidate(INVALID_NODE_DATA)
    }

    #[inline(always)]
    /// Validate with a symbol.
    pub fn validate(&mut self, symbol: &SId) -> bool {
        self.dirty.remove(symbol)
    }

    #[inline(always)]
    /// Validate name.
    pub fn validate_name(&mut self) -> bool {
        self.validate(&INVALID_NODE_NAME)
    }

    #[inline(always)]
    /// Validate attributes.
    pub fn validate_attrs(&mut self) -> bool {
        self.validate(&INVALID_NODE_ATTRS)
    }

    #[inline(always)]
    /// Validate parent.
    pub fn validate_parent(&mut self) -> bool {
        self.validate(&INVALID_NODE_PARENT)
    }

    #[inline(always)]
    /// Validate children.
    pub fn validate_children(&mut self) -> bool {
        self.validate(&INVALID_NODE_CHILDREN)
    }

    #[inline(always)]
    /// Validate data.
    pub fn validate_data(&mut self) -> bool {
        self.validate(&INVALID_NODE_DATA)
    }

    #[inline]
    /// Validate all dirty symbols at once.
    pub fn validate_clear(&mut self) -> bool {
        let res = self.dirty.len() > 0;
        self.dirty.clear();
        res
    }

    #[inline(always)]
    /// Is this node dirty
    pub fn dirty(&self, symbol: &SId) -> bool {
        self.dirty.contains(symbol)
    }

    #[inline(always)]
    /// Any dirty symbols?
    pub fn any_dirty(&self) -> bool {
        self.dirty.len() > 0
    }

    #[inline]
    /// Insert an attribute.
    pub fn insert_attribute(&mut self, id: SId, val: Val) -> bool {
        let res = self.attributes.insert(id, val).is_none();
        if res {
            self.invalidate_attrs();
        }
        res
    }

    #[inline]
    /// Remove an attribute.
    pub fn remove_attribute(&mut self, id: &SId) -> bool {
        let res = self.attributes.remove(id).is_some();
        if res {
            self.invalidate_attrs();
        }
        res
    }

    #[inline]
    /// Set name.
    pub fn set_name(&mut self, name: SId) -> bool {
        if name != self.name {
            self.name = name;
            self.invalidate_name();
            true
        } else {
            false
        }
    }

    #[inline(always)]
    /// Has a child?
    pub fn has_child(&self, child: &NodeRef) -> bool {
        self.children.contains(child)
    }

    #[inline]
    /// Add a child.
    pub fn add_child(&mut self, child: NodeRef) -> bool {
        if self.children.insert(child) {
            self.invalidate_children();
            true
        } else {
            false
        }
    }

    #[inline]
    /// Remove a child.
    pub fn remove_child(&mut self, child: &NodeRef) -> bool {
        if self.children.remove(child) {
            self.invalidate_children();
            true
        } else {
            false
        }
    }

    #[inline(always)]
    /// Has data by name?
    pub fn has_data_named(&self, name: &SId) -> bool {
        self.data.contains_key(name)
    }

    #[inline]
    /// Has data?
    pub fn has_data(&self, data: &DataRef) -> bool {
        for (_, id) in &self.data {
            if id == data { return true; }
        }
        false
    }

    #[inline]
    /// Add data.
    /// If this name already exists on this node, the old ref is returned.
    pub fn add_data(&mut self, name: SId, data: DataRef) -> Option<DataRef> {
        let old = self.data.insert(name, data);
        self.invalidate_data();
        old
    }

    /// Remove data.
    pub fn remove_data(&mut self, data: &DataRef) -> bool {
        let mut remove_name = None;
        for (name, id) in &self.data {
            if id == data {
                remove_name = Some(name.clone());
                break;
            }
        }
        if let Some(name) = remove_name {
            self.data.remove(&name).is_some()
        } else {
            false
        }
    }

    #[inline(always)]
    /// Remove data by name.
    pub fn remove_data_named(&mut self, name: &SId) -> Option<DataRef> {
        self.data.remove(name)
    }

    #[inline(always)]
    /// Get named data.
    pub fn get_data(&self, name: &SId) -> Option<&DataRef> {
        self.data.get(name)
    }
}
