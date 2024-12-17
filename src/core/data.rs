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

use std::collections::HashSet;
use bincode::{Error, ErrorKind};
use nanoid::nanoid;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use super::{IntoDataRef, SGraph, SNodeRef};


/// Data value dirty flag.
pub const DATA_DIRTY_VAL: &str = "value";

/// Data nodes dirty flag.
pub const DATA_DIRTY_NODES: &str = "nodes";


/// Stof Data.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SData {
    pub id: String,
    pub nodes: Vec<SNodeRef>,
    #[serde(skip)]
    pub dirty: HashSet<String>,

    /// Serialized value of this data using bincode.
    /// Keeps the complexity out of Stof!
    pub value: Vec<u8>,
}
impl SData {
    /// Create a new SData with an ID.
    pub fn new_id(id: &str, value: impl Serialize) -> Self {
        Self {
            id: id.to_owned(),
            value: bincode::serialize(&value).expect("Unable to serialize Stof data value"),
            ..Default::default()
        }
    }

    /// Create a new SData without an ID.
    pub fn new(value: impl Serialize) -> Self {
        Self {
            id: format!("dta{}", nanoid!()),
            value: bincode::serialize(&value).expect("Unable to serialize Stof data value"),
            ..Default::default()
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

    /// Set the value of this data.
    pub fn set_value(&mut self, value: impl Serialize) {
        self.value = bincode::serialize(&value).expect("Unable to serialize and set Stof data value");
        self.invalidate_val();
    }

    /// Get the value of this data.
    pub fn get_value<T>(&self) -> Result<T, Error> where T: DeserializeOwned {
        bincode::deserialize(&self.value)
    }

    /// Get data from the graph with a specific ID, deserializing it into a specific type.
    pub fn data<T>(graph: &SGraph, into_ref: impl IntoDataRef) -> Result<T, Error> where T: DeserializeOwned {
        if let Some(data) = into_ref.data_ref().data(graph) {
            return data.get_value();
        }
        Err(Box::new(ErrorKind::Custom("Data Error: Unable to find data reference".into())))
    }
}


/// Generic Data.
/// Implement this trait to get helpers for attaching, setting, and removing data from an Stof Graph.
/// Also indexes this type of data for creating selections based on "kind".
pub trait Data: Serialize + DeserializeOwned + Clone + IntoDataRef {
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
}
