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

use std::any::Any;
use nanoid::nanoid;
use rustc_hash::FxHashSet;
use serde::{ser::Error, Deserialize, Serialize};
use super::{IntoDataRef, IntoNodeRef, SDataRef, SGraph, SNodeRef};


/// Data value dirty flag.
pub const DATA_DIRTY_VAL: &str = "value";

/// Data nodes dirty flag.
pub const DATA_DIRTY_NODES: &str = "nodes";


/// Data trait that allows for dynamically typed data in Stof.
#[typetag::serde]
pub trait Data: AsDynAny + std::fmt::Debug + DataClone + Send + Sync {
    /// Returning true will serialize and deserialize as normal (do this for all data defined in this crate, that is 'always' included).
    /// Returning false will serialize this data into a container first, so that others can deserialize even if they don't know of this data type.
    fn core_data(&self) -> bool {
        return false;
    }

    /// Is this a conainer data?
    /// Used to determin deserialize behavior.
    fn is_container(&self) -> bool {
        return false;
    }
}

/// String data.
#[typetag::serde(name = "_String")]
impl Data for String {
    fn core_data(&self) -> bool {
        return true;
    }
}

/// Empty data.
#[typetag::serde(name = "_None")]
impl Data for () {
    fn core_data(&self) -> bool {
        return true;
    }
}

/// Container data.
/// This contains non-core data, encoded twice for unknown types at load.
/// Any core_data -> false data will get serialized into a container.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ContainerData {
    pub contained: Vec<u8>,
}
#[typetag::serde(name = "_Contained")]
impl Data for ContainerData {
    fn core_data(&self) -> bool {
        return true;
    }
    fn is_container(&self) -> bool {
        return true;
    }
}

/// Blanket Clone implementation for any struct that implements Clone + Data
pub trait DataClone {
    fn clone_data(&self) -> Box<dyn Data>;
}
impl<T: Data + Clone + 'static> DataClone for T {
    fn clone_data(&self) -> Box<dyn Data> {
        Box::new(self.clone())
    }
}
impl Clone for Box<dyn Data> {
    fn clone(&self) -> Box<dyn Data> {
        self.clone_data()
    }
}

/// Blanket manual upcast to dyn Any for Data.
pub trait AsDynAny {
    fn as_dyn_any(&self) -> &dyn Any;
    fn as_mut_dyn_any(&mut self) -> &mut dyn Any;
}
impl<T: Data + Any> AsDynAny for T {
    fn as_dyn_any(&self) -> &dyn Any {
        self
    }
    fn as_mut_dyn_any(&mut self) -> &mut dyn Any {
        self
    }
}


/// Stof Data.
// Serialize and deserialize_data_field implemented below.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SData {
    pub id: String,
    pub nodes: Vec<SNodeRef>,

    #[serde(deserialize_with = "deserialize_data_field")]
    #[serde(serialize_with = "serialize_data_field")]
    pub data: Box<dyn Data>,

    #[serde(skip)]
    pub dirty: FxHashSet<String>,
}
impl SData {
    /// Create a new SData with an ID.
    pub fn new_id(id: &str, data: Box<dyn Data>) -> Self {
        Self {
            id: id.to_owned(),
            nodes: Default::default(),
            dirty: Default::default(),
            data,
        }
    }

    /// Create a new SData without an ID.
    pub fn new(data: Box<dyn Data>) -> Self {
        Self {
            id: format!("dta{}", nanoid!()),
            nodes: Default::default(),
            dirty: Default::default(),
            data,
        }
    }

    /// New reference from 'node'.
    /// 'node' is now referencing this data.
    pub fn new_reference(&mut self, node: SNodeRef) {
        self.nodes.push(node);
        self.invalidate(DATA_DIRTY_NODES);
    }

    /// Reference removed from 'node'.
    /// 'node' is no longer referencing this data.
    pub fn ref_removed(&mut self, node: &SNodeRef) {
        self.nodes.retain(|x| x.id != node.id);
        self.invalidate(DATA_DIRTY_NODES);
    }

    /// Ref count.
    pub fn ref_count(&self) -> usize {
        self.nodes.len()
    }

    
    /*****************************************************************************
     * Invalidate/Validate.
     *****************************************************************************/
    
    /// Invalidate this data with a symbol.
    pub fn invalidate(&mut self, symbol: &str) {
        self.dirty.insert(symbol.to_owned());
    }

    /// Invalidate value.
    pub fn invalidate_val(&mut self) {
        self.invalidate(DATA_DIRTY_VAL);
    }

    /// Validate value.
    pub fn validate_val(&mut self) -> bool {
        self.validate(DATA_DIRTY_VAL)
    }

    /// Has dirty value?
    pub fn dirty_val(&self) -> bool {
        self.dirty(DATA_DIRTY_VAL)
    }

    /// Invalidate nodes.
    pub fn invalidate_nodes(&mut self) {
        self.invalidate(DATA_DIRTY_NODES);
    }

    /// Validate nodes.
    pub fn validate_nodes(&mut self) -> bool {
        self.validate(DATA_DIRTY_NODES)
    }

    /// Has dirty nodes?
    pub fn dirty_nodes(&self) -> bool {
        self.dirty(DATA_DIRTY_NODES)
    }

    /// Has the dirty symbol?
    pub fn dirty(&self, symbol: &str) -> bool {
        self.dirty.contains(symbol)
    }

    /// Validate all dirty symbols in the set at once.
    pub fn validate_clear(&mut self) -> bool {
        let res = self.dirty.len() > 0;
        self.dirty.clear();
        res
    }

    /// Validate.
    pub fn validate(&mut self, symbol: &str) -> bool {
        self.dirty.remove(symbol)
    }

    /// Has dirty tags?
    pub fn has_dirty(&self) -> bool {
        self.dirty.len() > 0
    }


    /*****************************************************************************
     * Insert Data into Stof.
     *****************************************************************************/
    
    /// Create a new data and insert it into a graph, using the specified data ID.
    pub fn insert_new_id(graph: &mut SGraph, node: impl IntoNodeRef, data: Box<dyn Data>, id: &str) -> Option<SDataRef> {
        let dta = Self::new_id(id, data);
        dta.insert(graph, node)
    }

    /// Create a new data and insert it into a graph.
    pub fn insert_new(graph: &mut SGraph, node: impl IntoNodeRef, data: Box<dyn Data>) -> Option<SDataRef> {
        let dta = Self::new(data);
        dta.insert(graph, node)
    }
    
    /// Insert this data into a graph.
    /// Will overwrite data with the same ID if already in the graph.
    pub fn insert(self, graph: &mut SGraph, node: impl IntoNodeRef) -> Option<SDataRef> {
        graph.put_data(node, self)
    }


    /*****************************************************************************
     * Attach existing to additional nodes.
     *****************************************************************************/
    
    /// Attach an existing data reference to a node.
    /// Returns true if the data was present and newly attached to the node.
    pub fn attach_existing(graph: &mut SGraph, node: impl IntoNodeRef, data: impl IntoDataRef) -> bool {
        let nref = node.node_ref();
        if nref.exists(graph) {
            return graph.put_data_ref(node, data);
        }
        false
    }

    /// Attach this data to an additional node in the graph.
    /// Returns true if this data was present and newly attached to the node.
    pub fn attach(&self, graph: &mut SGraph, node: impl IntoNodeRef) -> bool {
        let nref = node.node_ref();
        if nref.exists(graph) {
            return graph.put_data_ref(node, &self.id);
        }
        false
    }
    

    /*****************************************************************************
     * Type check for this data.
     *****************************************************************************/
    
    /// Is data a type of?
    /// Compares the unboxed data type to T.
    pub fn is_type_of<T: Any>(&self) -> bool {
        if let Some(_) = self.get_data::<T>() {
            return true;
        }
        false
    }

    /// Is data a type of (from the outside).
    pub fn type_of<T: Any>(graph: &SGraph, dref: impl IntoDataRef) -> bool {
        if let Some(sdata) = dref.data_ref().data(graph) {
            return sdata.is_type_of::<T>();
        }
        false
    }


    /*****************************************************************************
     * Set data.
     *****************************************************************************/

    /// Set data.
    pub fn set_data(&mut self, data: Box<dyn Data>) {
        self.data = data;
        self.invalidate_val();
    }

    /// Set data from the outside.
    pub fn set(graph: &mut SGraph, dref: impl IntoDataRef, data: Box<dyn Data>) -> bool {
        if let Some(sdata) = dref.data_ref().data_mut(graph) {
            sdata.set_data(data);
            return true;
        }
        false
    }


    /*****************************************************************************
     * Get data.
     *****************************************************************************/
    
    /// Get a reference to our data in the type we would like if able.
    pub fn get_data<T: Any>(&self) -> Option<&T> {
        let any = self.data.as_dyn_any();
        if let Some(data) = any.downcast_ref::<T>() {
            Some(data)
        } else {
            None
        }
    }

    /// Get a mutable reference to our data in the type we would like if able.
    pub fn get_data_mut<T: Any>(&mut self) -> Option<&mut T> {
        let any = self.data.as_mut_dyn_any();
        if let Some(data) = any.downcast_mut::<T>() {
            Some(data)
        } else {
            None
        }
    }

    /// Tagname.
    pub fn tagname(graph: &SGraph, into_ref: impl IntoDataRef) -> Option<String> {
        if let Some(data) = into_ref.data_ref().data(graph) {
            return Some(data.data.typetag_name().to_string());
        }
        None
    }

    /// Core data?
    pub fn core_data(graph: &SGraph, into_ref: impl IntoDataRef) -> Option<bool> {
        if let Some(data) = into_ref.data_ref().data(graph) {
            return Some(data.data.core_data());
        }
        None
    }

    /// Get data from the graph with a specific ID.
    pub fn get<T: Any>(graph: &SGraph, into_ref: impl IntoDataRef) -> Option<&T> {
        if let Some(data) = into_ref.data_ref().data(graph) {
            return data.get_data::<T>();
        }
        None
    }

    /// Get mutable data from the graph with a specific ID.
    pub fn get_mut<T: Any>(graph: &mut SGraph, into_ref: impl IntoDataRef) -> Option<&mut T> {
        if let Some(data) = into_ref.data_ref().data_mut(graph) {
            return data.get_data_mut::<T>();
        }
        None
    }
}


/// Custom serialize for data field.
fn serialize_data_field<S>(data: &Box<dyn Data>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
    if data.core_data() {
        data.serialize(serializer)
    } else {
        // Not core, so create a container and serialize it instead
        if let Ok(bytes) = bincode::serialize(data) {
            let container: Box<dyn Data> = Box::new(ContainerData { contained: bytes });
            container.as_ref().serialize(serializer)
        } else {
            Err(S::Error::custom("error serializing containerized (non-core) data to bytes"))
        }
    }
}


/// Custom deserialize for data field.
fn deserialize_data_field<'de, D>(deserializer: D) -> Result<Box<dyn Data>, D::Error>
    where
        D: serde::Deserializer<'de> {
    let mut data: Box<dyn Data> = Deserialize::deserialize(deserializer)?;

    // If data is a container, try deserializing the contained contents, replacing the container if possible
    if data.is_container() {
        let any = data.as_dyn_any();
        if let Some(container) = any.downcast_ref::<ContainerData>() {
            if let Ok(res) = bincode::deserialize::<Box<dyn Data>>(container.contained.as_ref()) {
                data = res;
            }
        }
    }

    Ok(data)
}
