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
use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};
use crate::{DataRef, NodeRef, SId};

/// Invalid/dirty name symbol.
pub const INVALID_NODE_NAME: SId = SId(Bytes::from_static(b"name"));

/// Invalid/dirty parent symbol.
pub const INVALID_NODE_PARENT: SId = SId(Bytes::from_static(b"parent"));

/// Invalid/dirty children symbol.
pub const INVALID_NODE_CHILDREN: SId = SId(Bytes::from_static(b"children"));

/// Invalid/dirty data symbol.
pub const INVALID_NODE_DATA: SId = SId(Bytes::from_static(b"data"));


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// Node.
pub struct Node {
    pub id: NodeRef,
    pub name: SId,
    pub parent: Option<NodeRef>,
    pub children: FxHashSet<NodeRef>,
    pub data: FxHashSet<DataRef>,

    #[serde(skip)]
    pub dirty: FxHashSet<SId>,
}
impl Node {
    /// Create a new node.
    pub fn new(name: SId, id: NodeRef) -> Self {
        Self {
            id,
            name,
            parent: None,
            children: Default::default(),
            data: Default::default(),
            dirty: Default::default(),
        }
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
    /// Has data?
    pub fn has_data(&self, data: &DataRef) -> bool {
        self.data.contains(data)
    }

    #[inline]
    /// Add data.
    pub fn add_data(&mut self, data: DataRef) -> bool {
        if self.data.insert(data) {
            self.invalidate_data();
            true
        } else {
            false
        }
    }

    #[inline]
    /// Remove data.
    pub fn remove_data(&mut self, data: &DataRef) -> bool {
        if self.data.remove(data) {
            self.invalidate_data();
            true
        } else {
            false
        }
    }
}
