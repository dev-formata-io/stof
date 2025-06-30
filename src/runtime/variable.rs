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
use crate::{model::{DataRef, Graph, NodeRef}, runtime::{Error, Type, Val}};


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Variable.
pub enum Variable {
    Val(Val),
    Ref(Arc<RwLock<Val>>),
    Const(Box<Self>),
}
impl Variable {
    /// Try to set this variable.
    pub fn set(&mut self, val: Val) -> Result<(), ()> {
        match self {
            Self::Val(v) => {
                *v = val;
                Ok(())
            },
            Self::Ref(r) => {
                *r.write().unwrap() = val;
                Ok(())
            },
            Self::Const(_) => {
                Err(())
            }
        }
    }

    /// Get the generic type for this variable.
    pub fn gen_type(&self) -> Type {
        match self {
            Self::Val(val) => val.gen_type(),
            Self::Ref(val) => val.read().unwrap().gen_type(),
            Self::Const(val) => val.gen_type()
        }
    }

    /// Get the specific type for this variable.
    pub fn spec_type(&self, graph: &Graph) -> Type {
        match self {
            Self::Val(val) => val.spec_type(graph),
            Self::Ref(val) => val.read().unwrap().spec_type(graph),
            Self::Const(val) => val.spec_type(graph),
        }
    }

    /// Cast this variable to a new type.
    /// Manipulates this var in place (no copy).
    pub fn cast(&mut self, target: &Type, graph: &mut Graph) -> Result<(), Error> {
        match self {
            Self::Val(val) => val.cast(target, graph),
            Self::Ref(val) => val.write().unwrap().cast(target, graph),
            Self::Const(val) => val.cast(target, graph),
        }
    }

    /// Try getting an object out of this variable.
    pub fn try_obj(&self) -> Option<NodeRef> {
        match self {
            Self::Val(val) => val.try_obj(),
            Self::Ref(val) => val.read().unwrap().try_obj(),
            Self::Const(val) => val.try_obj(),
        }
    }

    /// Is this variable equal to this data reference?
    pub fn is_data_ref(&self, data: &DataRef) -> bool {
        match self {
            Self::Val(val) => val.is_data_ref(data),
            Self::Ref(val) => val.read().unwrap().is_data_ref(data),
            Self::Const(val) => val.is_data_ref(data),
        }
    }
}
