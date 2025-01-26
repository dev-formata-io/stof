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

use std::{any::Any, collections::HashSet};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use super::{IntoDataRef, IntoNodeRef, SDataRef, SGraph, SNodeRef};


/// Data value dirty flag.
pub const DATA_DIRTY_VAL: &str = "value";

/// Data nodes dirty flag.
pub const DATA_DIRTY_NODES: &str = "nodes";


#[typetag::serde]
pub trait Payload: AsDynAny + std::fmt::Debug + PayloadClone {}

/// Blanket Clone implementation for any struct that implements Clone + Payload
pub trait PayloadClone {
    fn clone_payload(&self) -> Box<dyn Payload>;
}
impl<T: Payload + Clone + 'static> PayloadClone for T {
    fn clone_payload(&self) -> Box<dyn Payload> {
        Box::new(self.clone())
    }
}
impl Clone for Box<dyn Payload> {
    fn clone(&self) -> Box<dyn Payload> {
        self.clone_payload()
    }
}

/// Blanket manual upcast to dyn Any for payloads to be worked with as dynamic types.
pub trait AsDynAny {
    fn as_dyn_any(&self) -> &dyn Any;
    fn as_mut_dyn_any(&mut self) -> &mut dyn Any;
}
impl<T: Payload + Any> AsDynAny for T {
    fn as_dyn_any(&self) -> &dyn Any {
        self
    }
    fn as_mut_dyn_any(&mut self) -> &mut dyn Any {
        self
    }
}

#[typetag::serde(name = "empty")]
impl Payload for () {}

/// Stof Data.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SData {
    pub id: String,
    pub nodes: Vec<SNodeRef>,
    #[serde(skip)]
    pub dirty: HashSet<String>,
    pub data: Box<dyn Payload>,
}
impl SData {
    /// Create a new SData with an ID.
    pub fn new_id(id: &str, data: Box<dyn Payload>) -> Self {
        Self {
            id: id.to_owned(),
            nodes: Default::default(),
            dirty: Default::default(),
            data,
        }
    }

    /// Create a new SData without an ID.
    pub fn new(data: Box<dyn Payload>) -> Self {
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

    /// Has the dirty symbol?
    pub fn dirty(&self, symbol: &str) -> bool {
        self.dirty.contains(symbol)
    }

    /// Validate.
    pub fn validate(&mut self, symbol: &str) -> bool {
        self.dirty.remove(symbol)
    }

    /// Has dirty tags?
    pub fn has_dirty(&self) -> bool {
        self.dirty.len() > 0
    }

    /// Create a new data and insert it into a graph, using the specified data ID.
    pub fn insert_new_id(graph: &mut SGraph, node: impl IntoNodeRef, data: Box<dyn Payload>, id: &str) -> Option<SDataRef> {
        let dta = Self::new_id(id, data);
        dta.insert(graph, node)
    }

    /// Create a new data and insert it into a graph.
    pub fn insert_new(graph: &mut SGraph, node: impl IntoNodeRef, data: Box<dyn Payload>) -> Option<SDataRef> {
        let dta = Self::new(data);
        dta.insert(graph, node)
    }
    
    /// Insert this data into a graph.
    /// Will overwrite data with the same ID if already in the graph.
    pub fn insert(self, graph: &mut SGraph, node: impl IntoNodeRef) -> Option<SDataRef> {
        graph.put_data(node, self)
    }

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

    /// Set data.
    pub fn set_data(&mut self, data: Box<dyn Payload>) {
        self.data = data;
        self.invalidate_val();
    }

    /// Set data from the outside.
    pub fn set(graph: &mut SGraph, dref: impl IntoDataRef, data: Box<dyn Payload>) -> bool {
        if let Some(sdata) = dref.data_ref().data_mut(graph) {
            sdata.set_data(data);
            return true;
        }
        false
    }

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


// Generic Data.
// Implement this trait to get helpers for attaching, setting, and removing data from an Stof Graph.
// Also indexes this type of data for creating selections based on "kind".
/*pub trait Data: Serialize + DeserializeOwned + Clone + IntoDataRef {
    /// Set data ref.
    fn set_ref(&mut self, to_ref: impl IntoDataRef);

    /// Kind of this data.
    /// This will be the ID prefix of this data when attached in the graph.
    /// Allows for quick finding of different types of data.
    fn kind(&self) -> String {
        return "dta".to_string();
    }

    /// Exists in the graph?
    fn exists(&self, graph: &SGraph) -> bool {
        self.data_ref().exists(graph)
    }

    /// Attach this data to a node.
    /// Doesn't have to already be in the graph.
    fn attach(&mut self, node: &SNodeRef, graph: &mut SGraph) {
        if self.exists(graph) {
            graph.put_data_ref(node, &self.data_ref());
        } else {
            let mut id = self.data_ref().id;
            if id.len() < 1 {
                id = format!("{}_{}", self.kind(), nanoid!());
                self.set_ref(&id);
            }

            let data = SData::new_id(&id, &self);
            graph.put_data(node, data);
        }
    }

    /// Set the value of this data in the graph.
    /// Invalidates the data in the graph.
    fn set(&mut self, graph: &mut SGraph) {
        if let Some(data) = self.data_ref().data_mut(graph) {
            data.set_value(&self);
        }
    }

    /// Remove this data from Stof.
    /// If a node is not given, the data will be removed completely.
    /// If a node is specified, the data will be reomved only from that node.
    /// If the data only exists on that node, the data is removed completely.
    ///
    /// To undo this, just call 'attach' again.
    fn remove(&self, graph: &mut SGraph, node: Option<&SNodeRef>) {
        graph.remove_data(&self.data_ref(), node.clone());
    }
}*/
