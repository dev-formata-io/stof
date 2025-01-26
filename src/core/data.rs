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


/// Data trait that allows for dynamically typed data in Stof.
#[typetag::serde]
pub trait Data: AsDynAny + std::fmt::Debug + DataClone {}

/// String data.
#[typetag::serde(name = "_String")]
impl Data for String {}

/// Empty data.
#[typetag::serde(name = "_None")]
impl Data for () {}

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
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SData {
    pub id: String,
    pub nodes: Vec<SNodeRef>,
    pub data: Box<dyn Data>,

    #[serde(skip)]
    pub dirty: HashSet<String>,
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
