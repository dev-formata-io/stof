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

use core::str;
use std::{hash::Hash, sync::{Arc, RwLock}};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use crate::{SDataRef, SDoc, SGraph, SNodeRef};
use super::{SField, SNumType, SType, SUnits};

#[cfg(feature = "json")]
use crate::json::JSON;

#[cfg(feature = "toml")]
use crate::toml::TOML;


/// Into Stof value trait.
pub trait IntoSVal {
    fn stof_value(&self) -> SVal;
}
impl IntoSVal for i8 {
    fn stof_value(&self) -> SVal {
        SVal::Number(SNum::I64(*self as i64))
    }
}
impl IntoSVal for i16 {
    fn stof_value(&self) -> SVal {
        SVal::Number(SNum::I64(*self as i64))
    }
}
impl IntoSVal for i32 {
    fn stof_value(&self) -> SVal {
        SVal::Number(SNum::I64(*self as i64))
    }
}
impl IntoSVal for i64 {
    fn stof_value(&self) -> SVal {
        SVal::Number(SNum::I64(*self))
    }
}
impl IntoSVal for i128 {
    fn stof_value(&self) -> SVal {
        SVal::Number(SNum::I64(*self as i64))
    }
}
impl IntoSVal for f32 {
    fn stof_value(&self) -> SVal {
        SVal::Number(SNum::F64(*self as f64))
    }
}
impl IntoSVal for f64 {
    fn stof_value(&self) -> SVal {
        SVal::Number(SNum::F64(*self))
    }
}
impl IntoSVal for bool {
    fn stof_value(&self) -> SVal {
        SVal::Bool(*self)
    }
}
impl IntoSVal for &char {
    fn stof_value(&self) -> SVal {
        SVal::String(self.to_string())
    }
}
impl IntoSVal for char {
    fn stof_value(&self) -> SVal {
        SVal::String(self.to_string())
    }
}
impl IntoSVal for &str {
    fn stof_value(&self) -> SVal {
        SVal::String(self.to_string())
    }
}
impl IntoSVal for String {
    fn stof_value(&self) -> SVal {
        SVal::String(self.clone())
    }
}
impl IntoSVal for &String {
    fn stof_value(&self) -> SVal {
        SVal::String(self.to_string())
    }
}
impl IntoSVal for SNodeRef {
    fn stof_value(&self) -> SVal {
        SVal::Object(self.clone())
    }
}
impl IntoSVal for &SNodeRef {
    fn stof_value(&self) -> SVal {
        SVal::Object(SNodeRef::from(&self.id))
    }
}
impl IntoSVal for Option<SNodeRef> {
    fn stof_value(&self) -> SVal {
        if let Some(nref) = &self {
            SVal::Object(SNodeRef::from(nref.clone()))
        } else {
            SVal::Null
        }
    }
}
impl IntoSVal for &Option<SNodeRef> {
    fn stof_value(&self) -> SVal {
        if let Some(nref) = &self {
            SVal::Object(SNodeRef::from(nref.clone()))
        } else {
            SVal::Null
        }
    }
}
impl IntoSVal for Option<&SNodeRef> {
    fn stof_value(&self) -> SVal {
        if let Some(nref) = self {
            SVal::Object(SNodeRef::from(&nref.id))
        } else {
            SVal::Null
        }
    }
}
impl IntoSVal for &Option<&SNodeRef> {
    fn stof_value(&self) -> SVal {
        if let Some(nref) = &self {
            SVal::Object(SNodeRef::from(&nref.id))
        } else {
            SVal::Null
        }
    }
}
impl<T> IntoSVal for Vec<T> where T: IntoSVal {
    fn stof_value(&self) -> SVal {
        let mut arr = Vec::new();
        for val in self {
            arr.push(val.stof_value());
        }
        SVal::Array(arr)
    }
}
impl IntoSVal for Vec<u8> {
    fn stof_value(&self) -> SVal {
        SVal::Blob(self.clone())
    }
}
impl IntoSVal for &[u8] {
    fn stof_value(&self) -> SVal {
        self.to_vec().stof_value()
    }
}


/// Stof Value.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum SVal {
    Void,
    #[default]
    Null,
    Bool(bool),
    Number(SNum),
    String(String),
    Object(SNodeRef),
    FnPtr(SDataRef),
    Array(Vec<SVal>),
    Tuple(Vec<SVal>),
    Blob(Vec<u8>),
    Ref(Arc<RwLock<SVal>>),
}
impl From<&SVal> for SVal {
    fn from(value: &SVal) -> Self {
        value.clone()
    }
}
impl<T> From<T> for SVal where T: IntoSVal {
    fn from(value: T) -> Self {
        value.stof_value()
    }
}
impl PartialEq for SVal {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Ref(rf) => rf.read().unwrap().eq(other),
            Self::Void => {
                match other {
                    Self::Ref(rf) => self.eq(&rf.read().unwrap()),
                    Self::Void => true,
                    _ => false,
                }
            },
            Self::Null => {
                match other {
                    Self::Ref(rf) => self.eq(&rf.read().unwrap()),
                    Self::Null => true,
                    _ => false,
                }
            },
            Self::Bool(val) => {
                match other {
                    Self::Ref(rf) => self.eq(&rf.read().unwrap()),
                    Self::Bool(oval) => *val == *oval,
                    _ => false
                }
            },
            Self::Object(nref) => {
                match other {
                    Self::Ref(rf) => self.eq(&rf.read().unwrap()),
                    Self::Object(oref) => nref.id == oref.id,
                    _ => false,
                }
            },
            Self::Blob(vals) => {
                match other {
                    Self::Ref(rf) => self.eq(&rf.read().unwrap()),
                    Self::Blob(ovals) => vals == ovals,
                    _ => false,
                }
            },
            Self::FnPtr(dref) => {
                match other {
                    Self::Ref(rf) => self.eq(&rf.read().unwrap()),
                    Self::FnPtr(odref) => odref.id == dref.id,
                    _ => false,
                }
            },
            Self::Number(val) => {
                match other {
                    Self::Ref(rf) => self.eq(&rf.read().unwrap()),
                    Self::Number(oval) => val.eq(oval),
                    _ => false,
                }
            },
            Self::String(val) => {
                match other {
                    Self::Ref(rf) => self.eq(&rf.read().unwrap()),
                    Self::String(oval) => oval == val,
                    _ => false,
                }
            },
            Self::Array(vals) => {
                match other {
                    Self::Ref(rf) => self.eq(&rf.read().unwrap()),
                    Self::Array(ovals) => vals == ovals,
                    _ => false,
                }
            },
            Self::Tuple(vals) => {
                match other {
                    Self::Ref(rf) => self.eq(&rf.read().unwrap()),
                    Self::Tuple(ovals) => vals == ovals,
                    _ => false,
                }
            },
        }
    }
}
impl Eq for SVal {}
impl SVal {
    /// Schema equals another value?
    /// True if the values have the same type.
    pub fn schema_eq(&self, other: &Self) -> bool {
        self.stype() == other.stype()
    }

    /// Is void?
    pub fn is_void(&self) -> bool {
        match self {
            Self::Void => true,
            _ => false,
        }
    }

    /// Is empty?
    pub fn is_empty(&self) -> bool {
        self.stype().is_empty()
    }

    /// Is null?
    pub fn is_null(&self) -> bool {
        match self {
            Self::Null => true,
            _ => false,
        }
    }

    /// Is object?
    pub fn is_object(&self) -> bool {
        match self {
            Self::Object(_) => true,
            _ => false,
        }
    }

    /// Is array?
    pub fn is_array(&self) -> bool {
        match self {
            Self::Array(_) => true,
            _ => false,
        }
    }

    /// Is tuple?
    pub fn is_tuple(&self) -> bool {
        match self {
            Self::Tuple(_) => true,
            _ => false,
        }
    }

    /// Is number?
    pub fn is_number(&self) -> bool {
        match self {
            Self::Number(_) => true,
            _ => false,
        }
    }

    /// Is a ref?
    pub fn is_ref(&self) -> bool {
        match self {
            Self::Ref(_) => true,
            _ => false,
        }
    }

    /// Create a tuple.
    pub fn tuple<T>(vals: Vec<T>) -> Self where T: IntoSVal {
        let mut new: Vec<Self> = Vec::new();
        for v in vals { new.push(v.stof_value()); }
        Self::Tuple(new)
    }

    /// Type for this value.
    pub fn stype(&self) -> SType {
        match self {
            Self::Ref(rf) => rf.read().unwrap().stype(),
            Self::Void => SType::Void,
            Self::Bool(_) => SType::Bool,
            Self::Number(val) => SType::Number(val.stype()),
            Self::String(_) => SType::String,
            Self::Array(_) => SType::Array,
            Self::Null => SType::Null,
            Self::Tuple(vals) => {
                let mut types: Vec<SType> = Vec::new();
                for val in vals { types.push(val.stype()); }
                SType::Tuple(types)
            },
            Self::FnPtr(_) => SType::FnPtr,
            Self::Object(_) => SType::Object,
            Self::Blob(_) => SType::Blob,
        }
    }

    /// To string.
    pub fn to_string(&self) -> String {
        match self {
            Self::String(val) => { val.clone() },
            Self::Bool(val) => { val.to_string() },
            Self::Number(val) => { val.to_string() },
            Self::Array(vals) => { format!("{:?}", vals) },
            Self::Object(nref) => { format!("{:?}", nref) },
            Self::FnPtr(dref) => { format!("fn({:?})", dref) },
            Self::Null => { "null".to_string() },
            Self::Void => { "void".to_string() },
            Self::Ref(_) => { "ref".to_string() },
            Self::Tuple(tup) => { format!("tup({:?})", tup) },
            Self::Blob(blob) => { format!("blob({}bytes)", blob.len()) },
        }
    }

    /// Truthy value for this val.
    pub fn truthy(&self) -> bool {
        match self {
            Self::Ref(rf) => rf.read().unwrap().truthy(),
            Self::Array(_) => true,
            Self::Bool(val) => *val,
            Self::FnPtr(_) => true,
            Self::Object(_) => true,
            Self::Null => false,
            Self::Number(val) => val.bool(),
            Self::String(val) => val.len() > 0,
            Self::Tuple(_) => true,
            Self::Void => false,
            Self::Blob(_) => true,
        }
    }

    /// Typename.
    pub fn type_name(&self, graph: &SGraph) -> String {
        match self {
            Self::Object(nref) => {
                if let Some(prototype) = SField::field(graph, "__prototype__", '.', Some(nref)) {
                    if let Some(node) = graph.node_ref(&prototype.string(), None) {
                        if let Some(typename) = SField::field(graph, "typename", '.', Some(&node)) {
                            return typename.string();
                        }
                    }
                }
                "obj".into()
            },
            _ => {
                let stype = self.stype();
                stype.type_of()
            }
        }
    }

    /// Union this value with another, manipulating this value as the result.
    pub fn union(&mut self, other: &Self) {
        match self {
            SVal::Ref(rf) => {
                let mut val = rf.write().unwrap();
                val.union(other);
            },
            SVal::Void |
            SVal::Null => {
                *self = other.clone();
            },
            SVal::Tuple(_) |
            SVal::Object(_) |
            SVal::Number(_) |
            SVal::Blob(_) |
            SVal::FnPtr(_) |
            SVal::String(_) |
            SVal::Bool(_) => {
                if self != other {
                    match other {
                        SVal::Array(ovals) => {
                            let mut vals = ovals.clone();
                            vals.insert(0, self.clone());
                            *self = SVal::Array(vals);
                        },
                        _ => {
                            *self = SVal::Array(vec![self.clone(), other.clone()]);
                        }
                    }
                }
            },
            SVal::Array(vals) => {
                match other {
                    SVal::Array(ovals) => {
                        vals.append(&mut ovals.clone());
                    },
                    _ => {
                        vals.push(other.clone());
                    }
                }
            }
        }
    }

    /// Cast a value to another type of value.
    pub fn cast(&self, target: SType, doc: &mut SDoc) -> Result<Self> {
        match self {
            Self::Ref(rf) => rf.read().unwrap().cast(target, doc),
            Self::Blob(blob) => {
                match target {
                    SType::Array => {
                        Ok(Self::Array(blob.iter().map(|byte| Self::Number(SNum::I64((*byte) as i64))).collect()))
                    },
                    SType::String => {
                        Ok(Self::String(str::from_utf8(blob.as_slice())?.to_string()))
                    },
                    _ => Err(anyhow!("Cannot cast blob to anything but an array."))
                }
            },
            Self::Array(vals) => {
                match target {
                    SType::Array => Ok(Self::Array(vals.clone())),
                    SType::Blob => {
                        let mut blob: Vec<u8> = Vec::new();
                        for val in vals {
                            match val {
                                Self::Number(num) => {
                                    let res: Result<u8, _> = num.int().try_into();
                                    if res.is_err() {
                                        return Err(anyhow!("Cannot fit number into u8 while converting array to binary blob"));
                                    }
                                    blob.push(res.unwrap());
                                },
                                _ => {
                                    return Err(anyhow!("Cannot cast anything but numbers in an array to a binary blob"));
                                }
                            }
                        }
                        return Ok(Self::Blob(blob));
                    },
                    SType::Tuple(types) => {
                        let tup = Self::Tuple(vals.clone());
                        if tup.stype() == SType::Tuple(types.clone()) {
                            return Ok(tup);
                        }
                        // Try to convert every individual value
                        if vals.len() == types.len() {
                            let mut new_vals: Vec<Self> = Vec::new();
                            for i in 0..types.len() {
                                let ty = types[i].clone();
                                let val = &vals[i];
                                let val_type = val.stype();
                                
                                if val_type == ty {
                                    new_vals.push(val.clone());
                                } else {
                                    new_vals.push(val.cast(ty, doc)?);
                                }
                            }
                            return Ok(Self::Tuple(new_vals));
                        }
                        Err(anyhow!("Cannot cast array to type: {:?}", SType::Tuple(types.clone())))
                    },
                    target => Err(anyhow!("Cannot cast array to type: {:?}", target))
                }
            },
            Self::Bool(val) => {
                match target {
                    SType::Number(nt) => {
                        let mut v = 0;
                        if *val { v = 1; }
                        match nt {
                            SNumType::F64 => {
                                return Ok(Self::Number(SNum::F64(v as f64)));
                            },
                            SNumType::I64 => {
                                return Ok(Self::Number(SNum::I64(v as i64)));
                            },
                            SNumType::Units(units) => {
                                return Ok(Self::Number(SNum::Units(v as f64, units)));
                            }
                        }
                    },
                    SType::Array => {
                        return Ok(Self::Array(vec![Self::Bool(*val)]));
                    },
                    SType::String => {
                        return Ok(Self::String(format!("{}", val)));
                    },
                    SType::Bool => {
                        return Ok(Self::Bool(*val));
                    },
                    ty => {
                        return Err(anyhow!("Cannot cast a bool to type: {:?}", ty));
                    }
                }
            },
            Self::FnPtr(_) => {
                Err(anyhow!("Cannot cast fn pointer to anything"))
            },
            Self::Null => {
                // Null can be any type!
                Ok(self.clone())
            },
            Self::Number(val) => {
                match target {
                    SType::Array => {
                        Ok(Self::Array(vec![Self::Number(val.clone())]))
                    },
                    SType::Bool => {
                        Ok(Self::Bool(val.bool()))
                    },
                    SType::String => {
                        Ok(Self::String(val.print()))
                    },
                    SType::Number(ntype) => {
                        Ok(Self::Number(val.cast(ntype)))
                    },
                    stype => Err(anyhow!("Cannot cast number to: {:?}", stype))
                }
            },
            Self::String(val) => {
                match target {
                    SType::Array => {
                        Ok(Self::Array(vec![Self::String(val.clone())]))
                    },
                    SType::Blob => {
                        Ok(Self::Blob(str::as_bytes(&val).to_vec()))
                    },
                    SType::Bool => {
                        Ok(Self::Bool(val.len() > 0))
                    },
                    SType::String => {
                        Ok(Self::String(val.clone()))
                    },
                    SType::Number(ntype) => {
                        match ntype {
                            SNumType::I64 => {
                                if let Ok(res) = val.replace('+', "").parse::<i64>() {
                                    return Ok(Self::Number(SNum::I64(res)));
                                }
                                Err(anyhow!("Value '{}' is not i64", val))
                            },
                            SNumType::F64 => {
                                if let Ok(res) = val.replace('+', "").parse::<f64>() {
                                    return Ok(Self::Number(SNum::F64(res)));
                                }
                                Err(anyhow!("Value '{}' is not f64", val))
                            },
                            SNumType::Units(units) => {
                                if let Ok(res) = val.replace('+', "").parse::<f64>() {
                                    return Ok(Self::Number(SNum::Units(res, units)));
                                }
                                Err(anyhow!("Value '{}' is not f64", val))
                            },
                        }
                    },
                    stype => Err(anyhow!("Cannot cast string to: {:?}", stype))
                }
            },
            Self::Tuple(vals) => {
                match target {
                    SType::Array => {
                        return Ok(Self::Array(vals.clone()));
                    },
                    SType::Tuple(types) => {
                        if types.len() == vals.len() {
                            let mut new_tup = Vec::new();
                            for i in 0..types.len() {
                                let val = &vals[i];
                                let ty = types[i].clone();
                                if val.stype() != ty {
                                    new_tup.push(val.cast(ty, doc)?);
                                } else {
                                    new_tup.push(val.clone());
                                }
                            }
                            return Ok(SVal::Tuple(new_tup));
                        }
                        return Err(anyhow!("Cannot cast tuple of one length into a tuple of another length"))
                    },
                    _ => {}
                }
                Err(anyhow!("Cannot cast tuple to anything"))
            },
            Self::Void => {
                Err(anyhow!("Cannot cast void to anything"))
            },
            Self::Object(_) => {
                match target {
                    SType::Object => Ok(self.clone()),
                    _ => Err(anyhow!("Cannot cast Object into {:?}", target))
                }
            },
        }
    }

    /// Print this value.
    pub fn print(&self, doc: &mut SDoc) -> String {
        match self {
            Self::Ref(rf) => rf.read().unwrap().print(doc),
            Self::Void => {
                "void".to_string()
            },
            Self::Bool(val) => {
                format!("{}", val)
            },
            Self::Number(val) => {
                val.print()
            },
            Self::String(val) => {
                val.clone()
            },
            Self::Null => {
                "null".to_string()
            },
            Self::Array(vals) => {
                let mut arr = Vec::new();
                for val in vals {
                    arr.push(val.print(doc));
                }
                format!("{:?}", arr)
            },
            Self::Tuple(vals) => {
                let mut arr = Vec::new();
                for val in vals {
                    arr.push(val.print(doc));
                }
                format!("({:?})", arr)
            },
            Self::FnPtr(dref) => {
                format!("fn({})", dref.id)
            },
            #[allow(unused)]
            Self::Object(nref) => {
                #[cfg(feature = "json")]
                return JSON::stringify_node(&doc.graph, nref).expect("Unable to export node during print to JSON");

                #[cfg(feature = "toml")]
                return TOML::stringify_node(&doc.graph, nref).expect("Unable to export node during print to TOML");

                #[cfg(not(feature = "json"))]
                return self.debug(doc);
            },
            Self::Blob(blob) => {
                format!("blob({}bytes)", blob.len())
            },
        }
    }

    /// Debug this value to console.
    pub fn debug(&self, doc: &mut SDoc) -> String {
        match self {
            Self::Ref(rf) => {
                format!("Ref({})", rf.read().unwrap().debug(doc))
            },
            Self::Void => {
                "void".to_string()
            },
            Self::Null => {
                "null".to_string()
            },
            Self::Bool(val) => {
                format!("{:?}", val)
            },
            Self::Number(val) => {
                val.debug()
            },
            Self::String(val) => {
                format!("{:?}", val)
            },
            Self::Array(vals) => {
                let mut arr = Vec::new();
                for val in vals {
                    arr.push(val.print(doc));
                }
                format!("Array({:?})", arr)
            },
            Self::Tuple(vals) => {
                let mut arr = Vec::new();
                for val in vals {
                    arr.push(val.print(doc));
                }
                format!("Tuple({:?})", arr)
            },
            Self::FnPtr(dref) => {
                format!("Fn({})", dref.id)
            },
            Self::Object(nref) => {
                if let Some(node) = nref.node(&doc.graph) {
                    return node.dump(&doc.graph, 0, true);
                }
                format!("Object({})", nref.id)
            },
            Self::Blob(blob) => {
                format!("blob({}bytes)", blob.len())
            },
        }
    }

    /// Equality.
    pub fn equal(&self, other: &Self, _doc: &mut SDoc) -> Result<Self> {
        Ok((self == other).into())
    }

    /// Not equals.
    pub fn neq(&self, other: &Self, _doc: &mut SDoc) -> Result<Self> {
        Ok((self != other).into())
    }

    /// Greater than other?
    pub fn gt(&self, other: &Self, doc: &mut SDoc) -> Result<Self> {
        match self {
            Self::Ref(rf) => rf.read().unwrap().gt(other, doc),
            Self::Array(_) => Ok(Self::Bool(false)),
            Self::Tuple(_) => Ok(Self::Bool(false)),
            Self::Bool(_) => Ok(Self::Bool(false)),
            Self::FnPtr(_) => Ok(Self::Bool(false)),
            Self::Void |
            Self::Null => Ok(Self::Bool(false)),
            Self::Blob(blob) => {
                match other {
                    Self::Ref(rf) => self.gt(&rf.read().unwrap(), doc),
                    Self::Blob(other_blob) => Ok(Self::Bool(blob.len() > other_blob.len())),
                    _ => Ok(Self::Bool(false))
                }
            },
            Self::Number(val) => {
                match other {
                    Self::Ref(rf) => self.gt(&rf.read().unwrap(), doc),
                    Self::Number(oval) => {
                        Ok(Self::Bool(val.gt(oval)))
                    },
                    _ => Ok(Self::Bool(false))
                }
            },
            Self::String(val) => {
                match other {
                    Self::Ref(rf) => self.gt(&rf.read().unwrap(), doc),
                    Self::String(oval) => Ok(Self::Bool(val > oval)),
                    _ => Ok(Self::Bool(false)),
                }
            },
            Self::Object(_) => Ok(Self::Bool(false)),
        }
    }

    /// Less than other?
    pub fn lt(&self, other: &Self, doc: &mut SDoc) -> Result<Self> {
        match self {
            Self::Ref(rf) => rf.read().unwrap().lt(other, doc),
            Self::Array(_) => Ok(Self::Bool(false)),
            Self::Tuple(_) => Ok(Self::Bool(false)),
            Self::Bool(_) => Ok(Self::Bool(false)),
            Self::FnPtr(_) => Ok(Self::Bool(false)),
            Self::Void |
            Self::Null => Ok(Self::Bool(false)),
            Self::Blob(blob) => {
                match other {
                    Self::Ref(rf) => self.lt(&rf.read().unwrap(), doc),
                    Self::Blob(other_blob) => Ok(Self::Bool(blob.len() < other_blob.len())),
                    _ => Ok(Self::Bool(false))
                }
            },
            Self::Number(val) => {
                match other {
                    Self::Ref(rf) => self.lt(&rf.read().unwrap(), doc),
                    Self::Number(oval) => {
                        Ok(Self::Bool(val.lt(oval)))
                    },
                    _ => Ok(Self::Bool(false))
                }
            },
            Self::String(val) => {
                match other {
                    Self::Ref(rf) => self.lt(&rf.read().unwrap(), doc),
                    Self::String(oval) => Ok(Self::Bool(val < oval)),
                    _ => Ok(Self::Bool(false)),
                }
            },
            Self::Object(_) => Ok(Self::Bool(false)),
        }
    }

    /// Greater than or equal?
    pub fn gte(&self, other: &Self, doc: &mut SDoc) -> Result<Self> {
        let mut res = self.gt(other, doc)?;
        match res {
            Self::Bool(val) => {
                if val {
                    return Ok(Self::Bool(true));
                }
            },
            _ => {}
        }
        res = self.equal(other, doc)?;
        match res {
            Self::Bool(_) => Ok(res),
            _ => Ok(Self::Bool(false))
        }
    }

    /// Less than or equal?
    pub fn lte(&self, other: &Self, doc: &mut SDoc) -> Result<Self> {
        let mut res = self.lt(other, doc)?;
        match res {
            Self::Bool(val) => {
                if val {
                    return Ok(Self::Bool(true));
                }
            },
            _ => {}
        }
        res = self.equal(other, doc)?;
        match res {
            Self::Bool(_) => Ok(res),
            _ => Ok(Self::Bool(false))
        }
    }

    /// Logical and.
    pub fn and(&self, other: &Self, doc: &mut SDoc) -> Result<Self> {
        match self {
            Self::Ref(rf) => rf.read().unwrap().and(other, doc),
            // Object is truthy!
            Self::Object(_) => Self::Bool(true).and(other, doc),
            Self::Null |
            Self::Void => {
                // null && any => false
                Ok(Self::Bool(false))
            },
            Self::Blob(_) => Ok(Self::Bool(other.truthy())),
            Self::Bool(aval) => {
                match other {
                    Self::Object(_) => Ok(Self::Bool(*aval)),
                    Self::Ref(rf) => self.and(&rf.read().unwrap(), doc),
                    Self::Null |
                    Self::Void => {
                        // any && null => false
                        Ok(Self::Bool(false))
                    },
                    Self::Blob(_) => Ok(Self::Bool(*aval)),
                    Self::Bool(bval) => {
                        Ok(Self::Bool(*aval && *bval))
                    },
                    Self::Number(bval) => {
                        Ok(Self::Bool(*aval && bval.bool()))
                    },
                    Self::String(val) => {
                        Ok(Self::Bool(*aval && val.len() > 0))
                    },
                    Self::Array(_) => {
                        Err(anyhow!("Cannot and bool with array"))
                    },
                    Self::Tuple(_) => {
                        Err(anyhow!("Cannot and bool with a tuple"))
                    },
                    Self::FnPtr(_) => {
                        Err(anyhow!("Cannot and bool with fn pointer"))
                    }
                }
            },
            Self::String(aval) => {
                match other {
                    Self::Object(_) => Ok(Self::Bool(aval.len() > 0)),
                    Self::Ref(rf) => self.and(&rf.read().unwrap(), doc),
                    Self::Null |
                    Self::Void => {
                        // Any && null => false
                        Ok(Self::Bool(false))
                    },
                    Self::Blob(_) => Ok(Self::Bool(aval.len() > 0)),
                    Self::String(bval) => {
                        Ok(Self::Bool(aval.len() > 0 && bval.len() > 0))
                    },
                    Self::Number(bval) => {
                        Ok(Self::Bool(aval.len() > 0 && bval.bool()))
                    },
                    Self::Bool(bval) => {
                        Ok(Self::Bool(aval.len() > 0 && *bval))
                    },
                    Self::Array(_) => {
                        Err(anyhow!("Cannot and string with an array"))
                    },
                    Self::Tuple(_) => {
                        Err(anyhow!("Cannot and string with tuple"))
                    },
                    Self::FnPtr(_) => {
                        Err(anyhow!("Cannot and string with function pointer"))
                    }
                }
            },
            Self::Number(aval) => {
                match other {
                    Self::Object(_) => Ok(Self::Bool(aval.bool())),
                    Self::Ref(rf) => self.and(&rf.read().unwrap(), doc),
                    Self::Null |
                    Self::Void => {
                        // any && void => false
                        Ok(Self::Bool(false))
                    },
                    Self::Blob(_) => Ok(Self::Bool(aval.bool())),
                    Self::Number(bval) => {
                        Ok(Self::Bool(aval.bool() && bval.bool()))
                    },
                    Self::String(bval) => {
                        Ok(Self::Bool(aval.bool() && bval.len() > 0))
                    },
                    Self::Bool(bval) => {
                        Ok(Self::Bool(aval.bool() && *bval))
                    },
                    Self::Array(_) => {
                        Err(anyhow!("Cannot and number with an array"))
                    },
                    Self::Tuple(_) => {
                        Err(anyhow!("Cannot and number with a tuple"))
                    },
                    Self::FnPtr(_) => {
                        Err(anyhow!("Cannot and number with function pointer"))
                    }
                }
            },
            Self::Array(_) => {
                Err(anyhow!("Cannot and array with anything"))
            },
            Self::Tuple(_) => {
                Err(anyhow!("Cannot and tuple with anything"))
            },
            Self::FnPtr(_) => {
                Err(anyhow!("Cannot and function pointer with anything"))
            }
        }
    }

    /// Logical or.
    pub fn or(&self, other: &Self, doc: &mut SDoc) -> Result<Self> {
        match self {
            Self::Object(_) => Ok(Self::Bool(true)),
            Self::Ref(rf) => rf.read().unwrap().or(other, doc),
            Self::Null |
            Self::Void => {
                Ok(Self::Bool(!other.is_empty()))
            },
            Self::Blob(_) => Ok(Self::Bool(true)),
            Self::Bool(aval) => {
                match other {
                    Self::Object(_) => Ok(Self::Bool(true)),
                    Self::Ref(rf) => self.or(&rf.read().unwrap(), doc),
                    Self::Null |
                    Self::Void => {
                        Ok(Self::Bool(*aval))
                    },
                    Self::Blob(_) => Ok(Self::Bool(true)),
                    Self::Bool(bval) => {
                        Ok(Self::Bool(*aval || *bval))
                    },
                    Self::Number(bval) => {
                        Ok(Self::Bool(*aval || bval.bool()))
                    },
                    Self::String(bval) => {
                        Ok(Self::Bool(*aval || bval.len() > 0))
                    },
                    Self::Array(_) => {
                        Err(anyhow!("Cannot or bool with an array"))
                    },
                    Self::Tuple(_) => {
                        Err(anyhow!("Cannot or bool with a tuple"))
                    },
                    Self::FnPtr(_) => {
                        Err(anyhow!("Cannot or bool with function pointer"))
                    }
                }
            }
            Self::String(aval) => {
                match other {
                    Self::Object(_) => Ok(Self::Bool(true)),
                    Self::Ref(rf) => self.or(&rf.read().unwrap(), doc),
                    Self::Null |
                    Self::Void => {
                        Ok(Self::Bool(aval.len() > 0))
                    },
                    Self::Blob(_) => Ok(Self::Bool(true)),
                    Self::String(bval) => {
                        Ok(Self::Bool(aval.len() > 0 || bval.len() > 0))
                    },
                    Self::Number(bval) => {
                        Ok(Self::Bool(aval.len() > 0 || bval.bool()))
                    },
                    Self::Bool(bval) => {
                        Ok(Self::Bool(aval.len() > 0 || *bval))
                    },
                    Self::Array(_) => {
                        Err(anyhow!("Cannot or string with an array"))
                    },
                    Self::Tuple(_) => {
                        Err(anyhow!("Cannot or string with tuple"))
                    },
                    Self::FnPtr(_) => {
                        Err(anyhow!("Cannot or string with function pointer"))
                    }
                }
            },
            Self::Number(aval) => {
                match other {
                    Self::Object(_) => Ok(Self::Bool(true)),
                    Self::Ref(rf) => self.or(&rf.read().unwrap(), doc),
                    Self::Null |
                    Self::Void => {
                        Ok(Self::Bool(aval.bool()))
                    },
                    Self::Number(bval) => {
                        Ok(Self::Bool(aval.bool() || bval.bool()))
                    },
                    Self::Blob(_) => Ok(Self::Bool(true)),
                    Self::String(bval) => {
                        Ok(Self::Bool(aval.bool() || bval.len() > 0))
                    },
                    Self::Bool(bval) => {
                        Ok(Self::Bool(aval.bool() || *bval))
                    },
                    Self::Array(_) => {
                        Err(anyhow!("Cannot or number with array"))
                    },
                    Self::Tuple(_) => {
                        Err(anyhow!("Cannot or number with tuple"))
                    },
                    Self::FnPtr(_) => {
                        Err(anyhow!("Cannot or number with function pointer"))
                    }
                }
            },
            Self::Array(_) => {
                Err(anyhow!("Cannot or array with anything"))
            },
            Self::Tuple(_) => {
                Err(anyhow!("Cannot or tuple with anything"))
            },
            Self::FnPtr(_) => {
                Err(anyhow!("Cannot or function pointer with anything"))
            }
        }
    }

    /// Add.
    pub fn add(&self, other: &Self, doc: &mut SDoc) -> Result<Self> {
        match self {
            Self::Object(_) => Err(anyhow!("Cannot add objects")),
            Self::Ref(rf) => rf.read().unwrap().add(other, doc),
            Self:: Null |
            Self::Void => {
                Ok(other.clone())
            },
            Self::Blob(blob) => {
                match other {
                    Self::Blob(other_blob) => {
                        let mut res = blob.clone();
                        let mut other = other_blob.clone();
                        res.append(&mut other);
                        Ok(Self::Blob(res))
                    },
                    _ => Err(anyhow!("Cannot add something other than a binary blob to a blob"))
                }
            },
            Self::Bool(aval) => {
                match other {
                    Self::Object(_) => Err(anyhow!("Cannot add objects")),
                    Self::Ref(rf) => self.add(&rf.read().unwrap(), doc),
                    Self::Null |
                    Self::Void => {
                        Ok(self.clone())
                    },
                    Self::Bool(bval) => {
                        Ok(Self::Bool(*aval && *bval))
                    },
                    Self::Number(bval) => {
                        Ok(Self::String(format!("{}{}", aval, bval.print())))
                    },
                    Self::String(bval) => {
                        Ok(Self::String(format!("{}{}", aval, bval)))
                    },
                    Self::Array(_) => {
                        Err(anyhow!("Cannot 'bool + array'"))
                    },
                    Self::Tuple(_) => {
                        Err(anyhow!("Cannot 'bool + tuple'"))
                    },
                    Self::FnPtr(_) => {
                        Err(anyhow!("Cannot 'bool + function ptr'"))
                    },
                    Self::Blob(_) => {
                        Err(anyhow!("Cannot 'bool + blob'"))
                    },
                }
            },
            Self::String(aval) => {
                match other {
                    Self::Object(_) => Err(anyhow!("Cannot add objects")),
                    Self::Ref(rf) => self.add(&rf.read().unwrap(), doc),
                    Self::Null |
                    Self::Void => {
                        Ok(self.clone())
                    },
                    Self::String(bval) => {
                        Ok(Self::String(format!("{}{}", aval, bval)))
                    },
                    Self::Number(bval) => {
                        Ok(Self::String(format!("{}{}", aval, bval.print())))
                    },
                    Self::Bool(bval) => {
                        Ok(Self::String(format!("{}{}", aval, bval)))
                    },
                    Self::Array(_) => {
                        Ok(Self::String(format!("{}{}", aval, other.print(doc))))
                    },
                    Self::Tuple(_) => {
                        Ok(Self::String(format!("{}{}", aval, other.print(doc))))
                    },
                    Self::FnPtr(_) => {
                        Ok(Self::String(format!("{}{}", aval, other.print(doc))))
                    },
                    Self::Blob(_) => {
                        // TODO - cast string to blob?
                        Err(anyhow!("Cannot 'string + blob'"))
                    },
                }
            },
            Self::Number(aval) => {
                match other {
                    Self::Object(_) => Err(anyhow!("Cannot add objects")),
                    Self::Ref(rf) => self.add(&rf.read().unwrap(), doc),
                    Self::Null |
                    Self::Void => {
                        Ok(self.clone())
                    },
                    Self::Number(bval) => {
                        Ok(Self::Number(aval.add(bval)))
                    },
                    Self::String(bval) => {
                        if let Ok(bval) = bval.parse::<f64>() {
                            Ok(Self::Number(aval.add(&SNum::F64(bval))))
                        } else {
                            Err(anyhow!("Cannot add string that is not a number to a number"))
                        }
                    },
                    Self::Bool(bval) => {
                        Ok(Self::String(format!("{}{}", aval.print(), bval)))
                    },
                    Self::Array(_) => {
                        Err(anyhow!("Cannot 'number + array'"))
                    },
                    Self::Tuple(_) => {
                        Err(anyhow!("Cannot 'number + tuple'"))
                    },
                    Self::FnPtr(_) => {
                        Err(anyhow!("Cannot 'number + fn ptr'"))
                    },
                    Self::Blob(_) => {
                        Err(anyhow!("Cannot 'number + blob'"))
                    },
                }
            },
            Self::Array(vals) => {
                match other {
                    Self::Object(_) => Err(anyhow!("Cannot add objects")),
                    Self::Ref(rf) => {
                        let mut new = vals.clone();
                        new.push(Self::Ref(rf.clone()));
                        Ok(Self::Array(new))
                    },
                    Self::Null |
                    Self::Void => {
                        Ok(Self::Array(vals.clone()))
                    },
                    Self::Bool(val) => {
                        let mut new = vals.clone();
                        new.push(Self::Bool(*val));
                        Ok(Self::Array(new))
                    },
                    Self::Number(val) => {
                        let mut new = vals.clone();
                        new.push(Self::Number(val.clone()));
                        Ok(Self::Array(new))
                    },
                    Self::String(val) => {
                        let mut new = vals.clone();
                        new.push(Self::String(val.clone()));
                        Ok(Self::Array(new))
                    },
                    Self::Array(bvals) => {
                        let mut other = bvals.clone();
                        let mut new = vals.clone();
                        new.append(&mut other);
                        Ok(Self::Array(new))
                    },
                    Self::Tuple(bvals) => {
                        let mut other = bvals.clone();
                        let mut new = vals.clone();
                        new.append(&mut other);
                        Ok(Self::Array(new))
                    },
                    Self::FnPtr(dref) => {
                        let mut new = vals.clone();
                        new.push(Self::FnPtr(dref.clone()));
                        Ok(Self::Array(new))
                    },
                    Self::Blob(_) => {
                        let arr_blob = Self::Array(vals.clone()).cast(SType::Blob, doc)?;
                        arr_blob.add(other, doc)
                    },
                }
            },
            Self::Tuple(_) => {
                Err(anyhow!("Cannot mutate a tuple."))
            },
            Self::FnPtr(_) => {
                Err(anyhow!("Cannot add anything to a function"))
            }
        }
    }

    /// Subtract.
    pub fn sub(&self, other: &Self, doc: &mut SDoc) -> Result<Self> {
        match self {
            Self::Object(_) => Err(anyhow!("Cannot subtract objects")),
            Self::Ref(rf) => rf.read().unwrap().sub(other, doc),
            Self::Null |
            Self::Void => {
                Err(anyhow!("Cannot subtract anything from null or void"))
            },
            Self::Blob(_) => {
                Err(anyhow!("Cannot subtract from a binary blob"))
            },
            Self::Bool(aval) => {
                match other {
                    Self::Object(_) => Err(anyhow!("Cannot subtract objects")),
                    Self::Ref(rf) => self.sub(&rf.read().unwrap(), doc),
                    Self::Null |
                    Self::Void => {
                        Ok(self.clone())
                    },
                    Self::Bool(bval) => {
                        Ok(Self::Bool(*aval ^ *bval))
                    },
                    Self::Number(_) => {
                        Err(anyhow!("Cannot subtract a number from a bool"))
                    },
                    Self::String(_) => {
                        Err(anyhow!("Cannot subtract a string from a bool"))
                    },
                    Self::Array(_) => {
                        Err(anyhow!("Cannot subtract an array from a bool"))
                    },
                    Self::Tuple(_) => {
                        Err(anyhow!("Cannot subtract a tuple from a bool"))
                    },
                    Self::FnPtr(_) => {
                        Err(anyhow!("Cannot subtract a fn pointer from a bool"))
                    },
                    Self::Blob(_) => {
                        Err(anyhow!("Cannot subtract a blob"))
                    },
                }
            },
            Self::String(aval) => {
                match other {
                    Self::Object(_) => Err(anyhow!("Cannot subtract objects")),
                    Self::Ref(rf) => self.sub(&rf.read().unwrap(), doc),
                    Self::Null |
                    Self::Void => {
                        Ok(self.clone())
                    },
                    Self::String(bval) => {
                        Ok(Self::String(aval.replace(bval, "")))
                    },
                    Self::Number(bval) => {
                        Ok(Self::String(aval.replace(&bval.print(), "")))
                    },
                    Self::Bool(bval) => {
                        Ok(Self::String(aval.replace(&bval.to_string(), "")))
                    },
                    Self::Array(_) => {
                        Err(anyhow!("Cannot subtract an array from a string"))
                    },
                    Self::Tuple(_) => {
                        Err(anyhow!("Cannot subtract a tuple from a string"))
                    },
                    Self::FnPtr(_) => {
                        Err(anyhow!("Cannot subtract a fn pointer from a string"))
                    },
                    Self::Blob(_) => {
                        Err(anyhow!("Cannot subtract a blob"))
                    },
                }
            },
            Self::Number(aval) => {
                match other {
                    Self::Object(_) => Err(anyhow!("Cannot subtract objects")),
                    Self::Ref(rf) => self.sub(&rf.read().unwrap(), doc),
                    Self::Null |
                    Self::Void => {
                        Ok(self.clone())
                    },
                    Self::Number(bval) => {
                        Ok(Self::Number(aval.sub(bval)))
                    },
                    Self::String(bval) => {
                        if let Ok(bval) = bval.parse::<f64>() {
                            Ok(Self::Number(aval.sub(&SNum::F64(bval))))
                        } else {
                            Err(anyhow!("Cannot subtract string that is not a number to a number"))
                        }
                    },
                    Self::Bool(bval) => {
                        let mut num = 0;
                        if *bval { num = 1; }
                        Ok(Self::Number(aval.sub(&SNum::I64(num))))
                    },
                    Self::Array(_) => {
                        Err(anyhow!("Cannot subtract an array from a number"))
                    },
                    Self::Tuple(_) => {
                        Err(anyhow!("Cannot subtract a tuple from a string"))
                    },
                    Self::FnPtr(_) => {
                        Err(anyhow!("Cannot subtract a fn pointer from a number"))
                    },
                    Self::Blob(_) => {
                        Err(anyhow!("Cannot subtract a blob"))
                    },
                }
            },
            Self::Array(_) => {
                Err(anyhow!("Cannot subtract anything from an array"))
            },
            Self::Tuple(_) => {
                Err(anyhow!("Cannot mutate a tuple"))
            },
            Self::FnPtr(_) => {
                Err(anyhow!("Cannot subtract anything from a fn pointer"))
            }
        }
    }

    /// Multiply another value with this value.
    pub fn mul(&self, other: &Self, doc: &mut SDoc) -> Result<Self> {
        match self {
            Self::Object(_) => Err(anyhow!("Cannot multiply objects")),
            Self::Ref(rf) => rf.read().unwrap().mul(other, doc),
            Self::Null |
            Self::Void => {
                Ok(other.clone())
            },
            Self::Blob(_) => {
                Err(anyhow!("Cannot multiply a blob"))
            },
            Self::Bool(aval) => {
                match other {
                    Self::Object(_) => Err(anyhow!("Cannot multiply objects")),
                    Self::Ref(rf) => self.mul(&rf.read().unwrap(), doc),
                    Self::Null |
                    Self::Void => {
                        Ok(self.clone())
                    },
                    Self::Bool(bval) => {
                        Ok(Self::Bool(*aval || *bval))
                    },
                    Self::Number(_) => {
                        Err(anyhow!("Cannot multiply a bool and a number"))
                    },
                    Self::String(_) => {
                        Err(anyhow!("Cannot multiply a bool and a string"))
                    },
                    Self::Array(_) => {
                        Err(anyhow!("Cannot multiply a bool and an array"))
                    },
                    Self::Tuple(_) => {
                        Err(anyhow!("Cannot multiply a bool and a tuple"))
                    },
                    Self::FnPtr(_) => {
                        Err(anyhow!("Cannot multiply a bool and a fn pointer"))
                    },
                    Self::Blob(_) => {
                        Err(anyhow!("Cannot multiply a blob"))
                    },
                }
            },
            Self::String(aval) => {
                match other {
                    Self::Object(_) => Err(anyhow!("Cannot multiply objects")),
                    Self::Ref(rf) => self.mul(&rf.read().unwrap(), doc),
                    Self::Null |
                    Self::Void => {
                        Ok(self.clone())
                    },
                    Self::String(bval) => {
                        Ok(Self::String(format!("{}{}", aval, bval)))
                    },
                    Self::Number(bval) => {
                        let mut other = String::default();
                        for _ in 0..bval.int() {
                            other.push_str(&aval.clone());
                        }
                        Ok(Self::String(other))
                    },
                    Self::Bool(bval) => {
                        Ok(Self::String(format!("{}{}", aval, bval)))
                    },
                    Self::Array(_) => {
                        Err(anyhow!("Cannot multiply a string and an array"))
                    },
                    Self::Tuple(_) => {
                        Err(anyhow!("Cannot multiply a string and a tuple"))
                    },
                    Self::FnPtr(_) => {
                        Err(anyhow!("Cannot multiply a string and a fn pointer"))
                    },
                    Self::Blob(_) => {
                        Err(anyhow!("Cannot multiply a blob"))
                    },
                }
            },
            Self::Number(aval) => {
                match other {
                    Self::Object(_) => Err(anyhow!("Cannot multiply objects")),
                    Self::Ref(rf) => self.mul(&rf.read().unwrap(), doc),
                    Self::Null |
                    Self::Void => {
                        Ok(self.clone())
                    },
                    Self::Number(bval) => {
                        Ok(Self::Number(aval.mul(bval)))
                    },
                    Self::String(bval) => {
                        if let Ok(bval) = bval.parse::<f64>() {
                            Ok(Self::Number(aval.mul(&SNum::F64(bval))))
                        } else {
                            Err(anyhow!("Cannot multiply string that is not a number to a number"))
                        }
                    },
                    Self::Bool(_) => {
                        Err(anyhow!("Cannot multiply a number and a bool"))
                    },
                    Self::Array(_) => {
                        Err(anyhow!("Cannot multiply a number with an array"))
                    },
                    Self::Tuple(_) => {
                        Err(anyhow!("Cannot multiply a number with a tuple"))
                    },
                    Self::FnPtr(_) => {
                        Err(anyhow!("Cannot multiply a number with a fn pointer"))
                    },
                    Self::Blob(_) => {
                        Err(anyhow!("Cannot multiply a blob"))
                    },
                }
            },
            Self::Array(_) => {
                Err(anyhow!("Cannot multiply an array with anything"))
            },
            Self::Tuple(_) => {
                Err(anyhow!("Cannot multiply a tuple with anything"))
            },
            Self::FnPtr(_) => {
                Err(anyhow!("Cannot multiply a fn pointer with anything"))
            }
        }
    }

    /// Divide another value with this value.
    pub fn div(&self, other: &Self, doc: &mut SDoc) -> Result<Self> {
        match self {
            Self::Object(_) => Err(anyhow!("Cannot divide objects")),
            Self::Ref(rf) => rf.read().unwrap().div(other, doc),
            Self::Null |
            Self::Void => {
                Ok(other.clone())
            },
            Self::Blob(_) => {
                Err(anyhow!("Cannot divide a blob"))
            },
            Self::Bool(aval) => {
                match other {
                    Self::Object(_) => Err(anyhow!("Cannot divide objects")),
                    Self::Ref(rf) => self.div(&rf.read().unwrap(), doc),
                    Self::Null |
                    Self::Void => {
                        Ok(self.clone())
                    },
                    Self::Bool(bval) => {
                        Ok(Self::Bool(*aval && *bval))
                    },
                    Self::Number(_) => {
                        Err(anyhow!("Cannot divide a bool and a number"))
                    },
                    Self::String(_) => {
                        Err(anyhow!("Cannot divide a bool and a string"))
                    },
                    Self::Array(_) => {
                        Err(anyhow!("Cannot divide a bool and an array"))
                    },
                    Self::Tuple(_) => {
                        Err(anyhow!("Cannot divide a bool and a tuple"))
                    },
                    Self::FnPtr(_) => {
                        Err(anyhow!("Cannot divide a bool and a fn pointer"))
                    },
                    Self::Blob(_) => {
                        Err(anyhow!("Cannot divide a blob"))
                    },
                }
            },
            Self::String(aval) => {
                match other {
                    Self::Object(_) => Err(anyhow!("Cannot divide objects")),
                    Self::Ref(rf) => self.div(&rf.read().unwrap(), doc),
                    Self::Null |
                    Self::Void => {
                        Ok(self.clone())
                    },
                    Self::String(bval) => {
                        let vec = aval.split(bval).collect::<Vec<&str>>();
                        let mut new: Vec<Self> = Vec::new();
                        for v in vec {
                            new.push(v.into());
                        }
                        Ok(Self::Array(new))
                    },
                    Self::Number(_) => {
                        Err(anyhow!("Cannot divide a string by a number"))
                    },
                    Self::Bool(_) => {
                        Err(anyhow!("Cannot divide a string by a bool"))
                    },
                    Self::Array(_) => {
                        Err(anyhow!("Cannot divide a string and an array"))
                    },
                    Self::Tuple(_) => {
                        Err(anyhow!("Cannot divide a string and a tuple"))
                    },
                    Self::FnPtr(_) => {
                        Err(anyhow!("Cannot divide a string and a fn pointer"))
                    },
                    Self::Blob(_) => {
                        Err(anyhow!("Cannot divide a blob"))
                    },
                }
            },
            Self::Number(aval) => {
                match other {
                    Self::Object(_) => Err(anyhow!("Cannot divide objects")),
                    Self::Ref(rf) => self.div(&rf.read().unwrap(), doc),
                    Self::Null |
                    Self::Void => {
                        Ok(self.clone())
                    },
                    Self::Number(bval) => {
                        Ok(Self::Number(aval.div(bval)))
                    },
                    Self::String(bval) => {
                        if let Ok(bval) = bval.parse::<f64>() {
                            Ok(Self::Number(aval.div(&SNum::F64(bval))))
                        } else {
                            Err(anyhow!("Cannot divide string that is not a number to a number"))
                        }
                    },
                    Self::Bool(_) => {
                        Err(anyhow!("Cannot divide a number and a bool"))
                    },
                    Self::Array(_) => {
                        Err(anyhow!("Cannot divide a number with an array"))
                    },
                    Self::Tuple(_) => {
                        Err(anyhow!("Cannot divide a number with a tuple"))
                    },
                    Self::FnPtr(_) => {
                        Err(anyhow!("Cannot divide a number with a fn pointer"))
                    },
                    Self::Blob(_) => {
                        Err(anyhow!("Cannot divide a blob"))
                    },
                }
            },
            Self::Array(_) => {
                Err(anyhow!("Cannot divide an array with anything"))
            },
            Self::Tuple(_) => {
                Err(anyhow!("Cannot divide a tuple with anything"))
            },
            Self::FnPtr(_) => {
                Err(anyhow!("Cannot divide a fn pointer with anything"))
            }
        }
    }

    /// Modulus/remainder (mod) another value with this value.
    pub fn rem(&self, other: &Self, doc: &mut SDoc) -> Result<Self> {
        match self {
            Self::Object(_) => Err(anyhow!("Cannot divide objects")),
            Self::Ref(rf) => rf.read().unwrap().rem(other, doc),
            Self::Null |
            Self::Void => {
                Ok(other.clone())
            },
            Self::Blob(_) => {
                Err(anyhow!("Cannot divide a blob"))
            },
            Self::Bool(aval) => {
                match other {
                    Self::Object(_) => Err(anyhow!("Cannot divide objects")),
                    Self::Ref(rf) => self.rem(&rf.read().unwrap(), doc),
                    Self::Null |
                    Self::Void => {
                        Ok(self.clone())
                    },
                    Self::Bool(bval) => {
                        Ok(Self::Bool(*aval && *bval))
                    },
                    Self::Number(_) => {
                        Err(anyhow!("Cannot divide a bool and a number"))
                    },
                    Self::String(_) => {
                        Err(anyhow!("Cannot divide a bool and a string"))
                    },
                    Self::Array(_) => {
                        Err(anyhow!("Cannot divide a bool and an array"))
                    },
                    Self::Tuple(_) => {
                        Err(anyhow!("Cannot divide a bool and a tuple"))
                    },
                    Self::FnPtr(_) => {
                        Err(anyhow!("Cannot divide a bool and a fn pointer"))
                    },
                    Self::Blob(_) => {
                        Err(anyhow!("Cannot divide a blob"))
                    },
                }
            },
            Self::String(aval) => {
                match other {
                    Self::Object(_) => Err(anyhow!("Cannot divide objects")),
                    Self::Ref(rf) => self.rem(&rf.read().unwrap(), doc),
                    Self::Null |
                    Self::Void => {
                        Ok(self.clone())
                    },
                    Self::String(bval) => {
                        let vec = aval.split(bval).collect::<Vec<&str>>();
                        let mut new: Vec<Self> = Vec::new();
                        for v in vec {
                            new.push(v.into());
                        }
                        Ok(Self::Array(new))
                    },
                    Self::Number(_) => {
                        Err(anyhow!("Cannot divide a string by a number"))
                    },
                    Self::Bool(_) => {
                        Err(anyhow!("Cannot divide a string by a bool"))
                    },
                    Self::Array(_) => {
                        Err(anyhow!("Cannot divide a string and an array"))
                    },
                    Self::Tuple(_) => {
                        Err(anyhow!("Cannot divide a string and a tuple"))
                    },
                    Self::FnPtr(_) => {
                        Err(anyhow!("Cannot divide a string and a fn pointer"))
                    },
                    Self::Blob(_) => {
                        Err(anyhow!("Cannot divide a blob"))
                    },
                }
            },
            Self::Number(aval) => {
                match other {
                    Self::Object(_) => Err(anyhow!("Cannot divide objects")),
                    Self::Ref(rf) => self.rem(&rf.read().unwrap(), doc),
                    Self::Null |
                    Self::Void => {
                        Ok(self.clone())
                    },
                    Self::Number(bval) => {
                        Ok(Self::Number(aval.rem(bval)))
                    },
                    Self::String(bval) => {
                        if let Ok(bval) = bval.parse::<f64>() {
                            Ok(Self::Number(aval.rem(&SNum::F64(bval))))
                        } else {
                            Err(anyhow!("Cannot divide string that is not a number to a number"))
                        }
                    },
                    Self::Bool(_) => {
                        Err(anyhow!("Cannot divide a number and a bool"))
                    },
                    Self::Array(_) => {
                        Err(anyhow!("Cannot divide a number with an array"))
                    },
                    Self::Tuple(_) => {
                        Err(anyhow!("Cannot divide a number with a tuple"))
                    },
                    Self::FnPtr(_) => {
                        Err(anyhow!("Cannot divide a number with a fn pointer"))
                    },
                    Self::Blob(_) => {
                        Err(anyhow!("Cannot divide a blob"))
                    },
                }
            },
            Self::Array(_) => {
                Err(anyhow!("Cannot divide an array with anything"))
            },
            Self::Tuple(_) => {
                Err(anyhow!("Cannot divide a tuple with anything"))
            },
            Self::FnPtr(_) => {
                Err(anyhow!("Cannot divide a fn pointer with anything"))
            }
        }
    }
}


/// Stof Number.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SNum {
    I64(i64),           // int
    F64(f64),           // float
    Units(f64, SUnits), // units
}
impl Default for SNum {
    fn default() -> Self {
        Self::I64(0)
    }
}
impl Hash for SNum {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::I64(val) => val.hash(state),
            Self::F64(val) => ((*val * 1000000.) as i64).hash(state),
            Self::Units(val, _units) => ((*val * 1000000.) as i64).hash(state),
        }
    }
}
impl PartialEq for SNum {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::I64(val) => {
                match other {
                    Self::I64(oval) => {
                        *val == *oval
                    },
                    Self::F64(oval) => {
                        *val as f64 == *oval
                    },
                    Self::Units(oval, ounits) => {
                        let mut base = *ounits;
                        if base.is_angle() {
                            // Make sure for eq we are always converting to positive radians!
                            base = SUnits::PositiveRadians;
                        }
                        if let Ok(a) = SUnits::convert(*val as f64, base, base) {
                            if let Ok(b) = SUnits::convert(*oval, *ounits, base) {
                                if base.is_angle() {
                                    // Lower precision for angles 6 places
                                    return (a*1000000.).round() == (b*1000000.).round();
                                }
                                return a == b;
                            }
                        }
                        *val as f64 == *oval
                    }
                }
            },
            Self::F64(val) => {
                match other {
                    Self::I64(oval) => {
                        *val == *oval as f64
                    },
                    Self::F64(oval) => {
                        *val == *oval
                    },
                    Self::Units(oval, ounits) => {
                        let mut base = *ounits;
                        if base.is_angle() {
                            // Make sure for eq we are always converting to positive radians!
                            base = SUnits::PositiveRadians;
                        }
                        if let Ok(a) = SUnits::convert(*val, base, base) {
                            if let Ok(b) = SUnits::convert(*oval, *ounits, base) {
                                if base.is_angle() {
                                    // Lower precision for angles 6 places
                                    return (a*1000000.).round() == (b*1000000.).round();
                                }
                                return a == b;
                            }
                        }
                        *val == *oval
                    }
                }
            },
            Self::Units(val, units) => {
                match other {
                    Self::I64(oval) => {
                        let mut base = *units;
                        if base.is_angle() {
                            // Make sure for eq we are always converting to positive radians!
                            base = SUnits::PositiveRadians;
                        }
                        if let Ok(a) = SUnits::convert(*val, base, base) {
                            if let Ok(b) = SUnits::convert(*oval as f64, base, base) {
                                if base.is_angle() {
                                    // Lower precision for angles 6 places
                                    return (a*1000000.).round() == (b*1000000.).round();
                                }
                                return a == b;
                            }
                        }
                        *val == *oval as f64
                    },
                    Self::F64(oval) => {
                        let mut base = *units;
                        if base.is_angle() {
                            // Make sure for eq we are always converting to positive radians!
                            base = SUnits::PositiveRadians;
                        }
                        if let Ok(a) = SUnits::convert(*val, base, base) {
                            if let Ok(b) = SUnits::convert(*oval, base, base) {
                                if base.is_angle() {
                                    // Lower precision for angles 6 places
                                    return (a*1000000.).round() == (b*1000000.).round();
                                }
                                return a == b;
                            }
                        }
                        *val == *oval
                    },
                    Self::Units(oval, ounits) => {
                        let mut base = units.common(*ounits);
                        if base.is_angle() {
                            // Make sure for eq we are always converting to positive radians!
                            base = SUnits::PositiveRadians;
                        }
                        if let Ok(a) = SUnits::convert(*val, *units, base) {
                            if let Ok(b) = SUnits::convert(*oval, *ounits, base) {
                                if base.is_angle() {
                                    // Lower precision for angles 6 places
                                    return (a*1000000.).round() == (b*1000000.).round();
                                }
                                return a == b;
                            }
                        }
                        *val == *oval
                    }
                }
            }
        }
    }
}
impl Eq for SNum {}
impl From<i32> for SNum {
    fn from(value: i32) -> Self {
        Self::I64(value as i64)
    }
}
impl From<i64> for SNum {
    fn from(value: i64) -> Self {
        Self::I64(value)
    }
}
impl From<i16> for SNum {
    fn from(value: i16) -> Self {
        Self::I64(value as i64)
    }
}
impl From<i8> for SNum {
    fn from(value: i8) -> Self {
        Self::I64(value as i64)
    }
}
impl From<i128> for SNum {
    fn from(value: i128) -> Self {
        Self::I64(value as i64)
    }
}
impl From<f32> for SNum {
    fn from(value: f32) -> Self {
        Self::F64(value as f64)
    }
}
impl From<f64> for SNum {
    fn from(value: f64) -> Self {
        Self::F64(value)
    }
}
impl From<(i32, SUnits)> for SNum {
    fn from(value: (i32, SUnits)) -> Self {
        Self::Units(value.0 as f64, value.1)
    }
}
impl From<(i64, SUnits)> for SNum {
    fn from(value: (i64, SUnits)) -> Self {
        Self::Units(value.0 as f64, value.1)
    }
}
impl From<(f32, SUnits)> for SNum {
    fn from(value: (f32, SUnits)) -> Self {
        Self::Units(value.0 as f64, value.1)
    }
}
impl From<(f64, SUnits)> for SNum {
    fn from(value: (f64, SUnits)) -> Self {
        Self::Units(value.0, value.1)
    }
}
impl SNum {
    /// Type for this number.
    pub fn stype(&self) -> SNumType {
        match self {
            Self::F64(_) => SNumType::F64,
            Self::I64(_) => SNumType::I64,
            Self::Units(_, units) => SNumType::Units(*units),
        }
    }

    /// Print this number.
    pub fn print(&self) -> String {
        match self {
            Self::I64(val) => {
                format!("{}", val)
            },
            Self::F64(val) => {
                format!("{}", val)
            },
            Self::Units(val, units) => {
                format!("{}{}", val, units.to_string())
            }
        }
    }

    /// Debug print.
    pub fn debug(&self) -> String {
        match self {
            Self::I64(val) => {
                format!("{:?}", val)
            },
            Self::F64(val) => {
                format!("{:?}", val)
            },
            Self::Units(val, units) => {
                format!("{:?}{:?}", val, units)
            }
        }
    }

    /// To string.
    pub fn to_string(&self) -> String {
        match self {
            Self::I64(i) => { format!("{}", i) },
            Self::F64(i) => { format!("{}", i) },
            Self::Units(i, units) => {
                if units.is_undefined() || !units.has_units() {
                    return format!("{}", i);
                }
                format!("{}{}", i, units.to_string())
            },
        }
    }

    /// Boolean value for this number.
    pub fn bool(&self) -> bool {
        match self {
            Self::I64(val) => *val != 0,
            Self::F64(val) => *val as i64 != 0,
            Self::Units(val, _) => *val as i64 != 0,
        }
    }

    /// Has units?
    pub fn has_units(&self) -> bool {
        match self {
            Self::Units(_, units) => units.has_units(),
            _ => false,
        }
    }

    /// Get units.
    pub fn get_units(&self) -> Option<SUnits> {
        match self {
            Self::Units(_, units) => {
                if units.has_units() { Some(*units) }
                else { None }
            },
            _ => None
        }
    }

    /// Integer representation of this number.
    pub fn int(&self) -> i64 {
        match self {
            Self::I64(val) => *val,
            Self::F64(val) => *val as i64,
            Self::Units(val, _) => *val as i64,
        }
    }

    /// Float representation of this number.
    pub fn float(&self) -> f64 {
        match self {
            Self::I64(val) => *val as f64,
            Self::F64(val) => *val,
            Self::Units(val, _) => *val,
        }
    }

    /// Float as units representation of this number.
    /// Will convert this number into the units provided if possible.
    pub fn float_with_units(&self, units: SUnits) -> f64 {
        match self {
            Self::I64(val) => {
                if let Ok(val) = SUnits::convert(*val as f64, units, units) {
                    val
                } else {
                    *val as f64
                }
            },
            Self::F64(val) => {
                if let Ok(val) = SUnits::convert(*val, units, units) {
                    val
                } else {
                    *val
                }
            },
            Self::Units(val, sunits) => {
                if let Ok(val) = SUnits::convert(*val, *sunits, units) {
                    val
                } else {
                    *val
                }
            },
        }
    }

    /// Cast a number into another number type.
    pub fn cast(&self, target: SNumType) -> Self {
        match self {
            Self::I64(val) => {
                match target {
                    SNumType::I64 => Self::I64(*val as i64),
                    SNumType::F64 => Self::F64(*val as f64),
                    SNumType::Units(ounits) => {
                        if let Ok(v) = SUnits::convert(*val as f64, ounits, ounits) {
                            Self::Units(v, ounits)
                        } else {
                            Self::Units(*val as f64, ounits)
                        }
                    }
                }
            },
            Self::F64(val) => {
                match target {
                    SNumType::I64 => Self::I64(*val as i64),
                    SNumType::F64 => Self::F64(*val as f64),
                    SNumType::Units(ounits) => {
                        if let Ok(v) = SUnits::convert(*val, ounits, ounits) {
                            Self::Units(v, ounits)
                        } else {
                            Self::Units(*val, ounits)
                        }
                    }
                }
            },
            Self::Units(val, units) => {
                match target {
                    SNumType::I64 => Self::I64(*val as i64),
                    SNumType::F64 => Self::F64(*val as f64),
                    SNumType::Units(ounits) => {
                        // Try casting directly to ounits
                        if let Ok(value) = SUnits::convert(*val, *units, ounits) {
                            return Self::Units(value, ounits);
                        }

                        // Try finding a base unit next...
                        let base = units.common(ounits);
                        if let Ok(value) = SUnits::convert(*val, *units, base) {
                            return Self::Units(value, base);
                        }

                        // No units anymore...
                        Self::F64(*val)
                    },
                }
            }
        }
    }

    /// Greater than another number?
    pub fn gt(&self, other: &Self) -> bool {
        match self {
            Self::I64(val) => {
                match other {
                    Self::I64(oval) => *val > *oval,
                    Self::F64(oval) => *val as f64 > *oval,
                    Self::Units(oval, ounits) => {
                        let mut base = *ounits;
                        if base.is_angle() { base = SUnits::PositiveRadians; }
                        if let Ok(a) = SUnits::convert(*val as f64, base, base) {
                            if let Ok(b) = SUnits::convert(*oval, *ounits, base) {
                                if base.is_angle() {
                                    return (a*1000000.).round() > (b*1000000.).round();
                                }
                                return a > b;
                            }
                        }
                        *val as f64 > *oval
                    },
                }
            },
            Self::F64(val) => {
                match other {
                    Self::I64(oval) => *val > *oval as f64,
                    Self::F64(oval) => *val > *oval,
                    Self::Units(oval, ounits) => {
                        let mut base = *ounits;
                        if base.is_angle() { base = SUnits::PositiveRadians; }
                        if let Ok(a) = SUnits::convert(*val, base, base) {
                            if let Ok(b) = SUnits::convert(*oval, *ounits, base) {
                                if base.is_angle() {
                                    return (a*1000000.).round() > (b*1000000.).round();
                                }
                                return a > b;
                            }
                        }
                        *val > *oval
                    },
                }
            },
            Self::Units(val, units) => {
                match other {
                    Self::I64(oval) => {
                        let mut base = *units;
                        if base.is_angle() { base = SUnits::PositiveRadians; }
                        if let Ok(a) = SUnits::convert(*val, base, base) {
                            if let Ok(b) = SUnits::convert(*oval as f64, base, base) {
                                if base.is_angle() {
                                    return (a*1000000.).round() > (b*1000000.).round();
                                }
                                return a > b;
                            }
                        }
                        *val > *oval as f64
                    },
                    Self::F64(oval) => {
                        let mut base = *units;
                        if base.is_angle() { base = SUnits::PositiveRadians; }
                        if let Ok(a) = SUnits::convert(*val, base, base) {
                            if let Ok(b) = SUnits::convert(*oval, base, base) {
                                if base.is_angle() {
                                    return (a*1000000.).round() > (b*1000000.).round();
                                }
                                return a > b;
                            }
                        }
                        *val > *oval
                    },
                    Self::Units(oval, ounits) => {
                        let mut base = units.common(*ounits);
                        if base.is_angle() { base = SUnits::PositiveRadians; }
                        if let Ok(a) = SUnits::convert(*val, *units, base) {
                            if let Ok(b) = SUnits::convert(*oval, *ounits, base) {
                                if base.is_angle() {
                                    return (a*1000000.).round() > (b*1000000.).round();
                                }
                                return a > b;
                            }
                        }
                        *val > *oval
                    },
                }
            },
        }
    }

    /// Less than another number?
    pub fn lt(&self, other: &Self) -> bool {
        match self {
            Self::I64(val) => {
                match other {
                    Self::I64(oval) => *val < *oval,
                    Self::F64(oval) => (*val as f64) < *oval,
                    Self::Units(oval, ounits) => {
                        let mut base = *ounits;
                        if base.is_angle() { base = SUnits::PositiveRadians; }
                        if let Ok(a) = SUnits::convert(*val as f64, base, base) {
                            if let Ok(b) = SUnits::convert(*oval, *ounits, base) {
                                if base.is_angle() {
                                    return (a*1000000.).round() < (b*1000000.).round();
                                }
                                return a < b;
                            }
                        }
                        (*val as f64) < *oval
                    },
                }
            },
            Self::F64(val) => {
                match other {
                    Self::I64(oval) => *val < *oval as f64,
                    Self::F64(oval) => *val < *oval,
                    Self::Units(oval, ounits) => {
                        let mut base = *ounits;
                        if base.is_angle() { base = SUnits::PositiveRadians; }
                        if let Ok(a) = SUnits::convert(*val, base, base) {
                            if let Ok(b) = SUnits::convert(*oval, *ounits, base) {
                                if base.is_angle() {
                                    return (a*1000000.).round() < (b*1000000.).round();
                                }
                                return a < b;
                            }
                        }
                        *val < *oval
                    },
                }
            },
            Self::Units(val, units) => {
                match other {
                    Self::I64(oval) => {
                        let mut base = *units;
                        if base.is_angle() { base = SUnits::PositiveRadians; }
                        if let Ok(a) = SUnits::convert(*val, base, base) {
                            if let Ok(b) = SUnits::convert(*oval as f64, base, base) {
                                if base.is_angle() {
                                    return (a*1000000.).round() < (b*1000000.).round();
                                }
                                return a < b;
                            }
                        }
                        *val < *oval as f64
                    },
                    Self::F64(oval) => {
                        let mut base = *units;
                        if base.is_angle() { base = SUnits::PositiveRadians; }
                        if let Ok(a) = SUnits::convert(*val, base, base) {
                            if let Ok(b) = SUnits::convert(*oval, base, base) {
                                if base.is_angle() {
                                    return (a*1000000.).round() < (b*1000000.).round();
                                }
                                return a < b;
                            }
                        }
                        *val < *oval
                    },
                    Self::Units(oval, ounits) => {
                        let mut base = units.common(*ounits);
                        if base.is_angle() { base = SUnits::PositiveRadians; }
                        if let Ok(a) = SUnits::convert(*val, *units, base) {
                            if let Ok(b) = SUnits::convert(*oval, *ounits, base) {
                                if base.is_angle() {
                                    return (a*1000000.).round() < (b*1000000.).round();
                                }
                                return a < b;
                            }
                        }
                        *val < *oval
                    },
                }
            },
        }
    }

    /// Add two numbers together.
    pub fn add(&self, other: &Self) -> Self {
        match self {
            Self::I64(val) => {
                match other {
                    Self::I64(bval) => {
                        Self::I64(*val + *bval)
                    },
                    Self::F64(bval) => {
                        Self::F64(*val as f64 + *bval)
                    },
                    Self::Units(bval, ounits) => {
                        let mut res = *val as f64 + *bval;
                        let base = *ounits;
                        if let Ok(a) = SUnits::convert(*val as f64, base, base) {
                            if let Ok(c) = SUnits::convert(a + *bval, base, base) {
                                res = c;
                            }
                        }
                        Self::Units(res, base)
                    }
                }
            },
            Self::F64(val) => {
                match other {
                    Self::I64(bval) => {
                        Self::F64(*val + *bval as f64)
                    },
                    Self::F64(bval) => {
                        Self::F64(*val + *bval)
                    },
                    Self::Units(bval, ounits) => {
                        let mut res = *val + *bval;
                        let base = *ounits;
                        if let Ok(a) = SUnits::convert(*val, base, base) {
                            if let Ok(c) = SUnits::convert(a + *bval, base, base) {
                                res = c;
                            }
                        }
                        Self::Units(res, base)
                    }
                }
            },
            Self::Units(val, units) => {
                match other {
                    Self::I64(bval) => {
                        let mut res = *val + *bval as f64;
                        let base = *units;
                        if let Ok(a) = SUnits::convert(*val, base, base) {
                            if let Ok(c) = SUnits::convert(a + *bval as f64, base, base) {
                                res = c;
                            }
                        }
                        Self::Units(res, base)
                    },
                    Self::F64(bval) => {
                        let mut res = *val + *bval;
                        let base = *units;
                        if let Ok(a) = SUnits::convert(*val, base, base) {
                            if let Ok(c) = SUnits::convert(a + *bval, base, base) {
                                res = c;
                            }
                        }
                        Self::Units(res, base)
                    },
                    Self::Units(bval, ounits) => {
                        let mut res = *val + *bval;
                        let base = units.common(*ounits);
                        if let Ok(a) = SUnits::convert(*val, *units, base) {
                            if let Ok(b) = SUnits::convert(*bval, *ounits, base) {
                                if let Ok(c) = SUnits::convert(a + b, base, base) {
                                    res = c;
                                } else {
                                    res = a + b;
                                }
                                if base.is_undefined() {
                                    return Self::F64(res);
                                }
                                return Self::Units(res, base);
                            }
                        }
                        // No units anymore...
                        Self::F64(res)
                    }
                }
            }
        }
    }

    /// Subtract two number.
    pub fn sub(&self, other: &Self) -> Self {
        match self {
            Self::I64(val) => {
                match other {
                    Self::I64(bval) => {
                        Self::I64(*val - *bval)
                    },
                    Self::F64(bval) => {
                        Self::F64(*val as f64 - *bval)
                    },
                    Self::Units(bval, ounits) => {
                        let mut res = *val as f64 - *bval;
                        let base = *ounits;
                        if let Ok(a) = SUnits::convert(*val as f64, base, base) {
                            if let Ok(c) = SUnits::convert(a - *bval, base, base) {
                                res = c;
                            }
                        }
                        Self::Units(res, base)
                    }
                }
            },
            Self::F64(val) => {
                match other {
                    Self::I64(bval) => {
                        Self::F64(*val - *bval as f64)
                    },
                    Self::F64(bval) => {
                        Self::F64(*val - *bval)
                    },
                    Self::Units(bval, ounits) => {
                        let mut res = *val - *bval;
                        let base = *ounits;
                        if let Ok(a) = SUnits::convert(*val, base, base) {
                            if let Ok(c) = SUnits::convert(a - *bval, base, base) {
                                res = c;
                            }
                        }
                        Self::Units(res, base)
                    }
                }
            },
            Self::Units(val, units) => {
                match other {
                    Self::I64(bval) => {
                        let mut res = *val - *bval as f64;
                        let base = *units;
                        if let Ok(a) = SUnits::convert(*val, base, base) {
                            if let Ok(c) = SUnits::convert(a - *bval as f64, base, base) {
                                res = c;
                            }
                        }
                        Self::Units(res, base)
                    },
                    Self::F64(bval) => {
                        let mut res = *val - *bval;
                        let base = *units;
                        if let Ok(a) = SUnits::convert(*val, base, base) {
                            if let Ok(c) = SUnits::convert(a - *bval, base, base) {
                                res = c;
                            }
                        }
                        Self::Units(res, base)
                    },
                    Self::Units(bval, ounits) => {
                        let mut res = *val - *bval;
                        let base = units.common(*ounits);
                        if let Ok(a) = SUnits::convert(*val, *units, base) {
                            if let Ok(b) = SUnits::convert(*bval, *ounits, base) {
                                if let Ok(c) = SUnits::convert(a - b, base, base) {
                                    res = c;
                                } else {
                                    res = a - b;
                                }
                                if base.is_undefined() {
                                    return Self::F64(res);
                                }
                                return Self::Units(res, base);
                            }
                        }
                        // No units anymore...
                        Self::F64(res)
                    }
                }
            }
        }
    }

    /// Multiply two numbers.
    pub fn mul(&self, other: &Self) -> Self {
        match self {
            Self::I64(val) => {
                match other {
                    Self::I64(bval) => {
                        Self::I64(*val * *bval)
                    },
                    Self::F64(bval) => {
                        Self::F64(*val as f64 * *bval)
                    },
                    Self::Units(bval, ounits) => {
                        let mut res = *val as f64 * *bval;
                        let base = *ounits;
                        if let Ok(a) = SUnits::convert(*val as f64, base, base) {
                            if let Ok(c) = SUnits::convert(a * *bval, base, base) {
                                res = c;
                            }
                        }
                        Self::Units(res, base)
                    }
                }
            },
            Self::F64(val) => {
                match other {
                    Self::I64(bval) => {
                        Self::F64(*val * *bval as f64)
                    },
                    Self::F64(bval) => {
                        Self::F64(*val * *bval)
                    },
                    Self::Units(bval, ounits) => {
                        let mut res = *val * *bval;
                        let base = *ounits;
                        if let Ok(a) = SUnits::convert(*val, base, base) {
                            if let Ok(c) = SUnits::convert(a * *bval, base, base) {
                                res = c;
                            }
                        }
                        Self::Units(res, base)
                    }
                }
            },
            Self::Units(val, units) => {
                match other {
                    Self::I64(bval) => {
                        let mut res = *val * *bval as f64;
                        let base = *units;
                        if let Ok(a) = SUnits::convert(*val, base, base) {
                            if let Ok(c) = SUnits::convert(a * *bval as f64, base, base) {
                                res = c;
                            }
                        }
                        Self::Units(res, base)
                    },
                    Self::F64(bval) => {
                        let mut res = *val * *bval;
                        let base = *units;
                        if let Ok(a) = SUnits::convert(*val, base, base) {
                            if let Ok(c) = SUnits::convert(a * *bval, base, base) {
                                res = c;
                            }
                        }
                        Self::Units(res, base)
                    },
                    Self::Units(bval, ounits) => {
                        let mut res = *val * *bval;
                        let base = units.common(*ounits);
                        if let Ok(a) = SUnits::convert(*val, *units, base) {
                            if let Ok(b) = SUnits::convert(*bval, *ounits, base) {
                                if let Ok(c) = SUnits::convert(a * b, base, base) {
                                    res = c;
                                } else {
                                    res = a * b;
                                }
                                if base.is_undefined() {
                                    return Self::F64(res);
                                }
                                return Self::Units(res, base);
                            }
                        }
                        // No units anymore...
                        Self::F64(res)
                    }
                }
            }
        }
    }

    /// Divide two numbers.
    pub fn div(&self, other: &Self) -> Self {
        match self {
            Self::I64(val) => {
                match other {
                    Self::I64(bval) => {
                        Self::I64(*val / *bval)
                    },
                    Self::F64(bval) => {
                        Self::F64(*val as f64 / *bval)
                    },
                    Self::Units(bval, ounits) => {
                        let mut res = *val as f64 / *bval;
                        let base = *ounits;
                        if let Ok(a) = SUnits::convert(*val as f64, base, base) {
                            if let Ok(c) = SUnits::convert(a / *bval, base, base) {
                                res = c;
                            }
                        }
                        Self::Units(res, base)
                    }
                }
            },
            Self::F64(val) => {
                match other {
                    Self::I64(bval) => {
                        Self::F64(*val / *bval as f64)
                    },
                    Self::F64(bval) => {
                        Self::F64(*val / *bval)
                    },
                    Self::Units(bval, ounits) => {
                        let mut res = *val / *bval;
                        let base = *ounits;
                        if let Ok(a) = SUnits::convert(*val, base, base) {
                            if let Ok(c) = SUnits::convert(a / *bval, base, base) {
                                res = c;
                            }
                        }
                        Self::Units(res, base)
                    }
                }
            },
            Self::Units(val, units) => {
                match other {
                    Self::I64(bval) => {
                        let mut res = *val / *bval as f64;
                        let base = *units;
                        if let Ok(a) = SUnits::convert(*val, base, base) {
                            if let Ok(c) = SUnits::convert(a / *bval as f64, base, base) {
                                res = c;
                            }
                        }
                        Self::Units(res, base)
                    },
                    Self::F64(bval) => {
                        let mut res = *val / *bval;
                        let base = *units;
                        if let Ok(a) = SUnits::convert(*val, base, base) {
                            if let Ok(c) = SUnits::convert(a / *bval, base, base) {
                                res = c;
                            }
                        }
                        Self::Units(res, base)
                    },
                    Self::Units(bval, ounits) => {
                        let mut res = *val / *bval;
                        let base = units.common(*ounits);
                        if let Ok(a) = SUnits::convert(*val, *units, base) {
                            if let Ok(b) = SUnits::convert(*bval, *ounits, base) {
                                if let Ok(c) = SUnits::convert(a / b, base, base) {
                                    res = c;
                                } else {
                                    res = a / b;
                                }
                                if base.is_undefined() {
                                    return Self::F64(res);
                                }
                                return Self::Units(res, base);
                            }
                        }
                        // No units anymore...
                        Self::F64(res)
                    }
                }
            }
        }
    }

    /// Rem (mod) between two numbers.
    pub fn rem(&self, other: &Self) -> Self {
        match self {
            Self::I64(val) => {
                match other {
                    Self::I64(bval) => {
                        Self::I64(*val % *bval)
                    },
                    Self::F64(bval) => {
                        Self::F64(*val as f64 % *bval)
                    },
                    Self::Units(bval, ounits) => {
                        let mut res = *val as f64 % *bval;
                        let base = *ounits;
                        if let Ok(a) = SUnits::convert(*val as f64, base, base) {
                            if let Ok(c) = SUnits::convert(a % *bval, base, base) {
                                res = c;
                            }
                        }
                        Self::Units(res, base)
                    }
                }
            },
            Self::F64(val) => {
                match other {
                    Self::I64(bval) => {
                        Self::F64(*val % *bval as f64)
                    },
                    Self::F64(bval) => {
                        Self::F64(*val % *bval)
                    },
                    Self::Units(bval, ounits) => {
                        let mut res = *val % *bval;
                        let base = *ounits;
                        if let Ok(a) = SUnits::convert(*val, base, base) {
                            if let Ok(c) = SUnits::convert(a % *bval, base, base) {
                                res = c;
                            }
                        }
                        Self::Units(res, base)
                    }
                }
            },
            Self::Units(val, units) => {
                match other {
                    Self::I64(bval) => {
                        let mut res = *val % *bval as f64;
                        let base = *units;
                        if let Ok(a) = SUnits::convert(*val, base, base) {
                            if let Ok(c) = SUnits::convert(a % *bval as f64, base, base) {
                                res = c;
                            }
                        }
                        Self::Units(res, base)
                    },
                    Self::F64(bval) => {
                        let mut res = *val % *bval;
                        let base = *units;
                        if let Ok(a) = SUnits::convert(*val, base, base) {
                            if let Ok(c) = SUnits::convert(a % *bval, base, base) {
                                res = c;
                            }
                        }
                        Self::Units(res, base)
                    },
                    Self::Units(bval, ounits) => {
                        let mut res = *val % *bval;
                        let base = units.common(*ounits);
                        if let Ok(a) = SUnits::convert(*val, *units, base) {
                            if let Ok(b) = SUnits::convert(*bval, *ounits, base) {
                                if let Ok(c) = SUnits::convert(a % b, base, base) {
                                    res = c;
                                } else {
                                    res = a % b;
                                }
                                if base.is_undefined() {
                                    return Self::F64(res);
                                }
                                return Self::Units(res, base);
                            }
                        }
                        // No units anymore...
                        Self::F64(res)
                    }
                }
            }
        }
    }
}
