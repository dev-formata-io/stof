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

use std::{ops::Deref, sync::{Arc, RwLock}};
use serde::{Deserialize, Serialize};
use crate::{model::{DataRef, Graph, NodeRef, SId}, runtime::{Error, Type, Val}};


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Variable.
/// Used in symbol tables and for fields.
pub struct Variable {
    pub val: Arc<RwLock<Val>>,
    pub mutable: bool,
    pub vtype: Option<Type>,
}
impl Variable {
    /// Create a new variable.
    pub fn new(graph: &Graph, mutable: bool, val: Val, typed: bool) -> Self {
        let mut var = Self {
            mutable,
            val: Arc::new(RwLock::new(val)),
            vtype: None,
        };
        if typed {
            var.vtype = Some(var.spec_type(graph));
        }
        var
    }

    /// Create a new val variable.
    /// Shorthand for some stack situations.
    pub fn val(val: Val) -> Self {
        Self {
            mutable: true,
            val: Arc::new(RwLock::new(val)),
            vtype: None,
        }
    }

    /// Try to set this variable.
    /// Will error if not able to set.
    pub fn set(&mut self, var: &Variable, graph: &mut Graph) -> Result<(), Error> {
        if self.mutable {
            if var.value_type() {
                // Set by value
                let mut val = var.val.read().unwrap().clone();
                if let Some(vtype) = &self.vtype {
                    if vtype != &val.spec_type(graph) {
                        if let Err(error) = val.cast(vtype, graph) {
                            return Err(error);
                        }
                    }
                }
                *self.val.write().unwrap() = val;
            } else {
                // Set by reference
                self.val = var.val.clone();
            }
            Ok(())
        } else {
            Err(Error::AssignConst)
        }
    }

    /// Stack var from this var (LoadVariable).
    /// This is the variable that gets loaded onto the stack.
    /// Not always a direct clone because of value types.
    pub fn stack_var(&self) -> Self {
        let mut clone = self.clone();
        if self.value_type() {
            let val = self.val.read().unwrap().clone();
            clone.val = Arc::new(RwLock::new(val));
        }
        clone
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
    /// Try extracting a function reference from this var.
    pub fn try_func(&self) -> Option<DataRef> {
        self.val.read().unwrap().try_func()
    }

    #[inline]
    /// Try extracting a promise from this var.
    pub fn try_promise(&self) -> Option<(SId, Type)> {
        self.val.read().unwrap().try_promise()
    }

    #[inline]
    /// Is this var a value type?
    pub fn value_type(&self) -> bool {
        self.val.read().unwrap().val_type()
    }

    #[inline]
    /// Cast this variable to a new type.
    pub fn cast(&self, target: &Type, graph: &mut Graph) -> Result<(), Error> {
        self.val.write().unwrap().cast(target, graph)
    }

    #[inline]
    /// Is this variable truthy?
    pub fn truthy(&self) -> bool {
        self.val.read().unwrap().truthy()
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

    #[inline]
    /// Drop this variable (data held within).
    pub fn drop_data(self, graph: &mut Graph) {
        self.val.read().unwrap().drop_data(graph);
    }

    /*****************************************************************************
     * Ops.
     *****************************************************************************/
    
    /// Greater than?
    pub fn gt(&self, rhs: &Self, graph: &Graph) -> Result<Self, Error> {
        match self.val.read().unwrap().gt(rhs.val.read().unwrap().deref(), graph) {
            Ok(val) => {
                Ok(Self::val(val))
            },
            Err(e) => Err(e)
        }
    }

    /// Less than?
    pub fn lt(&self, rhs: &Self, graph: &Graph) -> Result<Self, Error> {
        match self.val.read().unwrap().lt(rhs.val.read().unwrap().deref(), graph) {
            Ok(val) => {
                Ok(Self::val(val))
            },
            Err(e) => Err(e)
        }
    }

    /// Greater than or equal?
    pub fn gte(&self, rhs: &Self, graph: &Graph) -> Result<Self, Error> {
        match self.val.read().unwrap().gte(rhs.val.read().unwrap().deref(), graph) {
            Ok(val) => {
                Ok(Self::val(val))
            },
            Err(e) => Err(e)
        }
    }

    /// Less than or equal?
    pub fn lte(&self, rhs: &Self, graph: &Graph) -> Result<Self, Error> {
        match self.val.read().unwrap().lte(rhs.val.read().unwrap().deref(), graph) {
            Ok(val) => {
                Ok(Self::val(val))
            },
            Err(e) => Err(e)
        }
    }

    /// Equal?
    pub fn equal(&self, rhs: &Self) -> Result<Self, Error> {
        match self.val.read().unwrap().equal(rhs.val.read().unwrap().deref()) {
            Ok(val) => {
                Ok(Self::val(val))
            },
            Err(e) => Err(e)
        }
    }

    /// Not equal?
    pub fn not_equal(&self, rhs: &Self) -> Result<Self, Error> {
        match self.val.read().unwrap().not_equal(rhs.val.read().unwrap().deref()) {
            Ok(val) => {
                Ok(Self::val(val))
            },
            Err(e) => Err(e)
        }
    }

    #[inline]
    /// Add.
    pub fn add(&self, rhs: Self, graph: &mut Graph) -> Result<(), Error> {
        self.val.write().unwrap().add(rhs.val.read().unwrap().clone(), graph)?;
        Ok(())
    }

    #[inline]
    /// Subtract.
    pub fn sub(&self, rhs: Self, graph: &mut Graph) -> Result<(), Error> {
        self.val.write().unwrap().sub(rhs.val.read().unwrap().clone(), graph)?;
        Ok(())
    }

    #[inline]
    /// Multiply.
    pub fn mul(&self, rhs: Self, graph: &mut Graph) -> Result<(), Error> {
        self.val.write().unwrap().mul(rhs.val.read().unwrap().clone(), graph)?;
        Ok(())
    }

    #[inline]
    /// Divide.
    pub fn div(&self, rhs: Self, graph: &mut Graph) -> Result<(), Error> {
        self.val.write().unwrap().div(rhs.val.read().unwrap().clone(), graph)?;
        Ok(())
    }

    #[inline]
    /// Mod.
    pub fn rem(&self, rhs: Self, graph: &mut Graph) -> Result<(), Error> {
        self.val.write().unwrap().rem(rhs.val.read().unwrap().clone(), graph)?;
        Ok(())
    }

    #[inline]
    /// Bit And.
    pub fn bit_and(&self, rhs: Self) -> Result<(), Error> {
        self.val.write().unwrap().bit_and(rhs.val.read().unwrap().clone())?;
        Ok(())
    }

    #[inline]
    /// Bit Or.
    pub fn bit_or(&self, rhs: Self) -> Result<(), Error> {
        self.val.write().unwrap().bit_or(rhs.val.read().unwrap().clone())?;
        Ok(())
    }

    #[inline]
    /// Bit XOr.
    pub fn bit_xor(&self, rhs: Self) -> Result<(), Error> {
        self.val.write().unwrap().bit_xor(rhs.val.read().unwrap().clone())?;
        Ok(())
    }

    #[inline]
    /// Bit Shift Left.
    pub fn bit_shl(&self, rhs: Self) -> Result<(), Error> {
        self.val.write().unwrap().bit_shl(rhs.val.read().unwrap().clone())?;
        Ok(())
    }

    #[inline]
    /// Bit Shift Right.
    pub fn bit_shr(&self, rhs: Self) -> Result<(), Error> {
        self.val.write().unwrap().bit_shr(rhs.val.read().unwrap().clone())?;
        Ok(())
    }
}
