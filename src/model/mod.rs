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


pub mod sid;
pub use sid::*;

pub mod spath;
pub use spath::*;

pub mod graph;
pub use graph::*;

pub mod sref;
pub use sref::*;

pub mod node;
pub use node::*;

pub mod data;
pub use data::*;

pub mod components;
pub use components::*;

pub mod formats;
pub use formats::*;

pub mod libraries;
pub use libraries::*;


/*****************************************************************************
 * Stof Data Trait for Graph Data.
 *****************************************************************************/

use std::any::Any;
use serde::{Deserialize, Serialize};


/// Data trait that allows for dynamically typed data in Stof.
#[typetag::serde]
pub trait StofData: AsDynAny + std::fmt::Debug + DataClone + Send + Sync {
    /// Returning true will serialize and deserialize as normal (do this for all data defined in this crate, that is 'always' included).
    /// Returning false will serialize this data into a container first, so that others can deserialize even if they don't know of this data type.
    fn core_data(&self) -> bool {
        false
    }

    /// Is this a conainer data?
    /// Used to determin deserialize behavior.
    fn is_container(&self) -> bool {
        false
    }

    /// Does this data directly reference a node?
    /// If so, and you want this data to be removed when the node is removed, say yes.
    #[allow(unused)]
    fn hard_node_ref(&self, node: &NodeRef) -> bool {
        false
    }

    /// Deep copy this data.
    #[allow(unused)]
    fn deep_copy(&self, graph: &mut Graph, context: Option<NodeRef>) -> Box::<dyn StofData> {
        self.clone_data()
    }
}


/// Blanket manual upcast to dyn Any for data.
pub trait AsDynAny {
    fn as_dyn_any(&self) -> &dyn Any;
    fn as_mut_dyn_any(&mut self) -> &mut dyn Any;
}
impl<T: StofData + Any> AsDynAny for T {
    fn as_dyn_any(&self) -> &dyn Any {
        self
    }
    fn as_mut_dyn_any(&mut self) -> &mut dyn Any {
        self
    }
}


/// Blanket Clone implementation for any struct that implements Clone + Data
pub trait DataClone {
    fn clone_data(&self) -> Box<dyn StofData>;
}
impl<T: StofData + Clone + 'static> DataClone for T {
    fn clone_data(&self) -> Box<dyn StofData> {
        Box::new(self.clone())
    }
}
impl Clone for Box<dyn StofData> {
    fn clone(&self) -> Box<dyn StofData> {
        self.clone_data()
    }
}


/// String data.
#[typetag::serde(name = "String")]
impl StofData for String {
    fn core_data(&self) -> bool {
        return true;
    }
}


/// Empty data.
#[typetag::serde(name = "None")]
impl StofData for () {
    fn core_data(&self) -> bool {
        return true;
    }
}


/// Container data.
/// This contains non-core data, encoded twice for unknown types at load.
/// Any core_data -> false data will get serialized into a container.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct StofDataContainer {
    pub contained: Vec<u8>,
}
#[typetag::serde(name = "Contained")]
impl StofData for StofDataContainer {
    fn core_data(&self) -> bool {
        return true;
    }
    fn is_container(&self) -> bool {
        return true;
    }
}
