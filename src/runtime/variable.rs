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

use std::sync::{Arc, RwLock};
use serde::{Deserialize, Serialize};
use crate::{model::{DataRef, Graph, NodeRef}, runtime::{Type, Val}};


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Variable.
/// Used in symbol tables and for fields.
pub struct Variable {
    val: Arc<RwLock<Val>>,
    mutable: bool,
}
impl Variable {
    /// Create a new variable.
    pub fn new(mutable: bool, val: Val) -> Self {
        Self {
            mutable,
            val: Arc::new(RwLock::new(val)),
        }
    }

    /// Try to set this variable.
    /// Will error if not able to set.
    pub fn set(&mut self, val: Val) -> Result<(), ()> {
        if self.mutable {
            *self.val.write().unwrap() = val;
            Ok(())
        } else {
            Err(())
        }
    }

    #[inline]
    /// Get a value from this variable.
    /// Vals are pretty cheap to clone.
    pub fn get(&self) -> Val {
        self.val.read().unwrap().clone()
    }

    #[inline]
    /// Try extracting an object reference from this var.
    pub fn try_obj(&self) -> Option<NodeRef> {
        self.val.read().unwrap().try_obj()
    }

    #[inline]
    /// Is this variable a dangling object reference?
    pub fn dangling_obj(&self, graph: &Graph) -> bool {
        if let Some(obj) = self.try_obj() {
            !obj.node_exists(graph)
        } else {
            false
        }
    }

    #[inline]
    /// Is this variable a data reference?
    pub fn is_data_ref(&self, data: &DataRef) -> bool {
        self.val.read().unwrap().is_data_ref(data)
    }

    #[inline]
    /// Variables generic type.
    pub fn gen_type(&self) -> Type {
        self.val.read().unwrap().gen_type()
    }

    #[inline]
    /// Specific type.
    pub fn spec_type(&self, graph: &Graph) -> Type {
        self.val.read().unwrap().spec_type(graph)
    }
}
