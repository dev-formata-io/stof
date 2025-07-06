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

use std::ops::Deref;
use arcstr::ArcStr;
use serde::{Deserialize, Serialize};
use crate::{model::{DataRef, Graph, NodeRef, SId}, runtime::{Error, Type, Val, ValRef}};


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Variable.
/// Used in symbol tables and for fields.
pub struct Variable {
    pub val: ValRef<Val>,
    pub mutable: bool,
    pub vtype: Option<Type>,
}
impl Variable {
    /// Create a new variable.
    pub fn new(graph: &Graph, mutable: bool, val: Val, typed: bool) -> Self {
        let mut var = Self {
            mutable,
            val: ValRef::new(val),
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
            val: ValRef::new(val),
            vtype: None,
        }
    }

    /// Try to set this variable.
    /// Will error if not able to set.
    pub fn set(&mut self, var: &Variable, graph: &mut Graph) -> Result<(), Error> {
        if self.mutable {
            if var.value_type() {
                // Set by value - clone the internal value and set
                let mut val = var.val.read().clone();
                if let Some(vtype) = &self.vtype {
                    if vtype != &val.spec_type(graph) {
                        if let Err(error) = val.cast(vtype, graph) {
                            return Err(error);
                        }
                    }
                }
                *self.val.write() = val;
            } else {
                // Set by reference - set the val as a clone (modifying here will modify there)
                if let Some(vtype) = &self.vtype {
                    if vtype != &var.spec_type(graph) {
                        if let Err(error) = var.cast(vtype, graph) {
                            return Err(error);
                        }
                    }
                }
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
            let val = self.val.read().clone();
            clone.val = ValRef::new(val);
        }
        clone
    }

    #[inline]
    /// Get a value from this variable.
    /// Vals are pretty cheap to clone.
    pub fn get(&self) -> Val {
        self.val.read().clone()
    }

    #[inline]
    /// Try extracting an object reference from this var.
    pub fn try_obj(&self) -> Option<NodeRef> {
        self.val.read().try_obj()
    }

    #[inline]
    /// Try extracting a function reference from this var.
    pub fn try_func(&self) -> Option<DataRef> {
        self.val.read().try_func()
    }

    #[inline]
    /// Try extracting a promise from this var.
    pub fn try_promise(&self) -> Option<(SId, Type)> {
        self.val.read().try_promise()
    }

    #[inline]
    /// Is this var a value type?
    pub fn value_type(&self) -> bool {
        self.val.read().val_type()
    }

    #[inline]
    /// Cast this variable to a new type.
    pub fn cast(&self, target: &Type, graph: &mut Graph) -> Result<(), Error> {
        self.val.write().cast(target, graph)
    }

    #[inline]
    /// Is this variable truthy?
    pub fn truthy(&self) -> bool {
        self.val.read().truthy()
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
        self.val.read().is_data_ref(data)
    }

    #[inline]
    /// Variables generic type.
    pub fn gen_type(&self) -> Type {
        self.val.read().gen_type()
    }

    #[inline]
    /// Specific type.
    pub fn spec_type(&self, graph: &Graph) -> Type {
        self.val.read().spec_type(graph)
    }

    #[inline]
    /// Get this values library name.
    pub fn lib_name(&self, graph: &Graph) -> ArcStr {
        self.val.read().lib_name(graph)
    }

    #[inline]
    /// Drop this variable (data held within).
    pub fn drop_data(self, graph: &mut Graph) {
        self.val.read().drop_data(graph);
    }

    /*****************************************************************************
     * Ops.
     *****************************************************************************/
    
    /// Greater than?
    pub fn gt(&self, rhs: &Self, graph: &Graph) -> Result<Self, Error> {
        match self.val.read().gt(rhs.val.read().deref(), graph) {
            Ok(val) => {
                Ok(Self::val(val))
            },
            Err(e) => Err(e)
        }
    }

    /// Less than?
    pub fn lt(&self, rhs: &Self, graph: &Graph) -> Result<Self, Error> {
        match self.val.read().lt(rhs.val.read().deref(), graph) {
            Ok(val) => {
                Ok(Self::val(val))
            },
            Err(e) => Err(e)
        }
    }

    /// Greater than or equal?
    pub fn gte(&self, rhs: &Self, graph: &Graph) -> Result<Self, Error> {
        match self.val.read().gte(rhs.val.read().deref(), graph) {
            Ok(val) => {
                Ok(Self::val(val))
            },
            Err(e) => Err(e)
        }
    }

    /// Less than or equal?
    pub fn lte(&self, rhs: &Self, graph: &Graph) -> Result<Self, Error> {
        match self.val.read().lte(rhs.val.read().deref(), graph) {
            Ok(val) => {
                Ok(Self::val(val))
            },
            Err(e) => Err(e)
        }
    }

    /// Equal?
    pub fn equal(&self, rhs: &Self) -> Result<Self, Error> {
        match self.val.read().equal(rhs.val.read().deref()) {
            Ok(val) => {
                Ok(Self::val(val))
            },
            Err(e) => Err(e)
        }
    }

    /// Not equal?
    pub fn not_equal(&self, rhs: &Self) -> Result<Self, Error> {
        match self.val.read().not_equal(rhs.val.read().deref()) {
            Ok(val) => {
                Ok(Self::val(val))
            },
            Err(e) => Err(e)
        }
    }

    #[inline]
    /// Add.
    pub fn add(&self, rhs: Self, graph: &mut Graph) -> Result<(), Error> {
        self.val.write().add(rhs.val.read().clone(), graph)?;
        Ok(())
    }

    #[inline]
    /// Subtract.
    pub fn sub(&self, rhs: Self, graph: &mut Graph) -> Result<(), Error> {
        self.val.write().sub(rhs.val.read().clone(), graph)?;
        Ok(())
    }

    #[inline]
    /// Multiply.
    pub fn mul(&self, rhs: Self, graph: &mut Graph) -> Result<(), Error> {
        self.val.write().mul(rhs.val.read().clone(), graph)?;
        Ok(())
    }

    #[inline]
    /// Divide.
    pub fn div(&self, rhs: Self, graph: &mut Graph) -> Result<(), Error> {
        self.val.write().div(rhs.val.read().clone(), graph)?;
        Ok(())
    }

    #[inline]
    /// Mod.
    pub fn rem(&self, rhs: Self, graph: &mut Graph) -> Result<(), Error> {
        self.val.write().rem(rhs.val.read().clone(), graph)?;
        Ok(())
    }

    #[inline]
    /// Bit And.
    pub fn bit_and(&self, rhs: Self) -> Result<(), Error> {
        self.val.write().bit_and(rhs.val.read().clone())?;
        Ok(())
    }

    #[inline]
    /// Bit Or.
    pub fn bit_or(&self, rhs: Self) -> Result<(), Error> {
        self.val.write().bit_or(rhs.val.read().clone())?;
        Ok(())
    }

    #[inline]
    /// Bit XOr.
    pub fn bit_xor(&self, rhs: Self) -> Result<(), Error> {
        self.val.write().bit_xor(rhs.val.read().clone())?;
        Ok(())
    }

    #[inline]
    /// Bit Shift Left.
    pub fn bit_shl(&self, rhs: Self) -> Result<(), Error> {
        self.val.write().bit_shl(rhs.val.read().clone())?;
        Ok(())
    }

    #[inline]
    /// Bit Shift Right.
    pub fn bit_shr(&self, rhs: Self) -> Result<(), Error> {
        self.val.write().bit_shr(rhs.val.read().clone())?;
        Ok(())
    }
}
