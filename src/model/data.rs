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

use std::any::Any;
use bytes::Bytes;
use rustc_hash::FxHashSet;
use serde::{ser::Error, Deserialize, Serialize};
use crate::model::{StofDataContainer, DataRef, NodeRef, SId, StofData};


/// Invalid/dirty name.
pub const INVALID_DATA_NAME: SId = SId(Bytes::from_static(b"name"));

/// Invalid/dirty nodes.
pub const INVALID_DATA_NODES: SId = SId(Bytes::from_static(b"nodes"));

/// Invalid/dirty value.
pub const INVALID_DATA_VALUE: SId = SId(Bytes::from_static(b"value"));


#[derive(Debug, Clone, Deserialize, Serialize)]
/// Data.
pub struct Data {
    pub id: DataRef,
    pub name: SId,
    pub nodes: FxHashSet<NodeRef>,

    #[serde(deserialize_with = "deserialize_data_field")]
    #[serde(serialize_with = "serialize_data_field")]
    pub data: Box<dyn StofData>,

    #[serde(skip)]
    pub dirty: FxHashSet<SId>,
}
impl From<Box<dyn StofData>> for Data {
    fn from(value: Box<dyn StofData>) -> Self {
        let id = SId::default();
        Self::new(id.clone(), id, value)
    }
}
impl Data {
    /// Create new graph data.
    pub fn new(id: DataRef, name: SId, data: Box<dyn StofData>) -> Self {
        Self {
            id,
            name,
            nodes: Default::default(),
            data,
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
        self.invalidate(INVALID_DATA_NAME)
    }

    #[inline(always)]
    /// Invalidate nodes.
    pub fn invalidate_nodes(&mut self) -> bool {
        self.invalidate(INVALID_DATA_NODES)
    }

    #[inline(always)]
    /// Invalidate value.
    pub fn invalidate_value(&mut self) -> bool {
        self.invalidate(INVALID_DATA_VALUE)
    }

    #[inline(always)]
    /// Validate with a symbol.
    pub fn validate(&mut self, symbol: &SId) -> bool {
        self.dirty.remove(symbol)
    }

    #[inline(always)]
    /// Validate name.
    pub fn validate_name(&mut self) -> bool {
        self.validate(&INVALID_DATA_NAME)
    }

    #[inline(always)]
    /// Validate nodes.
    pub fn validate_nodes(&mut self) -> bool {
        self.validate(&INVALID_DATA_NODES)
    }

    #[inline(always)]
    /// Validate value.
    pub fn validate_value(&mut self) -> bool {
        self.validate(&INVALID_DATA_VALUE)
    }

    #[inline]
    /// Validate all dirty symbols at once.
    pub fn validate_clear(&mut self) -> bool {
        let res = self.dirty.len() > 0;
        self.dirty.clear();
        res
    }

    #[inline(always)]
    /// Is this data dirty
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

    #[inline]
    /// Added to a node.
    pub(crate) fn node_added(&mut self, node: NodeRef) -> bool {
        if self.nodes.insert(node) {
            self.invalidate_nodes();
            true
        } else {
            false
        }
    }

    #[inline]
    /// Removed from a node.
    pub(crate) fn node_removed(&mut self, node: &NodeRef) -> bool {
        if self.nodes.remove(node) {
            self.invalidate_nodes();
            true
        } else {
            false
        }
    }

    #[inline(always)]
    /// Ref count.
    pub fn ref_count(&self) -> usize {
        self.nodes.len()
    }

    #[inline]
    /// Is the data value of a certain type?
    pub fn is_type<T: Any>(&self) -> bool {
        if let Some(_) = self.get::<T>() {
            true
        } else {
            false
        }
    }

    #[inline]
    /// Set data value.
    pub fn set(&mut self, data: Box<dyn StofData>) {
        self.data = data;
        self.invalidate_value();
    }

    /// Get data value.
    pub fn get<T: Any>(&self) -> Option<&T> {
        let any = self.data.as_dyn_any();
        if let Some(data) = any.downcast_ref::<T>() {
            Some(data)
        } else {
            None
        }
    }

    /// Get mutable data value.
    pub fn get_mut<T: Any>(&mut self) -> Option<&mut T> {
        let any = self.data.as_mut_dyn_any();
        if let Some(data) = any.downcast_mut::<T>() {
            Some(data)
        } else {
            None
        }
    }

    #[inline(always)]
    /// Tagname.
    pub fn tagname(&self) -> String {
        self.data.typetag_name().to_string()
    }

    #[inline(always)]
    /// Core data?
    pub fn core_data(&self) -> bool {
        self.data.core_data()
    }
}


/// Custom serialize for data field.
fn serialize_data_field<S>(data: &Box<dyn StofData>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
    if data.core_data() {
        data.serialize(serializer)
    } else {
        // Not core, so create a container and serialize it instead
        if let Ok(bytes) = bincode::serialize(data) {
            let container: Box<dyn StofData> = Box::new(StofDataContainer { contained: bytes });
            container.as_ref().serialize(serializer)
        } else {
            Err(S::Error::custom("error serializing containerized (non-core) data to bytes"))
        }
    }
}


/// Custom deserialize for data field.
fn deserialize_data_field<'de, D>(deserializer: D) -> Result<Box<dyn StofData>, D::Error>
    where
        D: serde::Deserializer<'de> {
    let mut data: Box<dyn StofData> = Deserialize::deserialize(deserializer)?;

    // If data is a container, try deserializing the contained contents, replacing the container if possible
    if data.is_container() {
        let any = data.as_dyn_any();
        if let Some(container) = any.downcast_ref::<StofDataContainer>() {
            if let Ok(res) = bincode::deserialize::<Box<dyn StofData>>(container.contained.as_ref()) {
                data = res;
            }
        }
    }

    Ok(data)
}
