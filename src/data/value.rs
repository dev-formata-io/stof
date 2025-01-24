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
use std::{cmp::Ordering, collections::{BTreeMap, BTreeSet, HashSet}, hash::{Hash, Hasher}, ops::{Deref, DerefMut}, sync::{Arc, Mutex}};
use serde::{Deserialize, Serialize};
use crate::{Data, SDataRef, SDoc, SGraph, SNodeRef};
use super::{lang::SError, SField, SNumType, SPrototype, SType, SUnits};

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


/// Reference value mutex (hashable).
#[derive(Serialize, Deserialize, Debug)]
pub struct SMutex<T: ?Sized>(Mutex<T>);
impl<T: ?Sized + Hash> Hash for SMutex<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.lock().unwrap().hash(state);
    }
}
impl<T: ?Sized> Deref for SMutex<T> {
    type Target = Mutex<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T: ?Sized> DerefMut for SMutex<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl SMutex<SVal> {
    pub fn new(val: SVal) -> Self {
        Self(Mutex::new(val))
    }
}


/// Stof Value.
#[derive(Debug, Clone, Serialize, Deserialize, Default, Hash)]
pub enum SVal {
    Void,
    #[default]
    Null,
    Bool(bool),
    Number(SNum),
    String(String),
    Object(SNodeRef),
    FnPtr(SDataRef),
    Array(Vec<Self>),
    Tuple(Vec<Self>),
    Blob(Vec<u8>),
    Map(BTreeMap<Self, Self>),
    Set(BTreeSet<Self>),
    Boxed(Arc<SMutex<Self>>),
}
impl PartialOrd for SVal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for SVal {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            Self::Boxed(val) => {
                val.lock().unwrap().cmp(other)
            },
            Self::Void |
            Self::Null => Ordering::Less,
            Self::Bool(val) => {
                match other {
                    Self::Bool(oval) => {
                        val.cmp(oval)
                    },
                    Self::Boxed(oval) => {
                        self.cmp(oval.lock().unwrap().deref())
                    },
                    Self::Void |
                    Self::Null => Ordering::Greater,
                    _ => Ordering::Less,
                }
            },
            Self::Number(num) => {
                match other {
                    Self::Number(onum) => {
                        num.float().total_cmp(&onum.float())
                    },
                    Self::Boxed(oval) => {
                        self.cmp(oval.lock().unwrap().deref())
                    },
                    Self::Bool(_) |
                    Self::Void |
                    Self::Null => Ordering::Greater,
                    _ => Ordering::Less,
                }
            },
            Self::String(val) => {
                match other {
                    Self::String(oval) => val.cmp(oval),
                    Self::Boxed(oval) => {
                        self.cmp(oval.lock().unwrap().deref())
                    },
                    Self::Void |
                    Self::Null |
                    Self::Bool(_) |
                    Self::Number(_) => Ordering::Greater,
                    _ => Ordering::Less,
                }
            },
            Self::Object(nref) => {
                match other {
                    Self::Object(oref) => nref.id.cmp(&oref.id),
                    Self::Boxed(oval) => {
                        self.cmp(oval.lock().unwrap().deref())
                    },
                    Self::Void |
                    Self::Null |
                    Self::Bool(_) |
                    Self::Number(_) |
                    Self::String(_) => Ordering::Greater,
                    _ => Ordering::Less,
                }
            },
            Self::FnPtr(dref) => {
                match other {
                    Self::FnPtr(odref) => dref.id.cmp(&odref.id),
                    Self::Boxed(oval) => {
                        self.cmp(oval.lock().unwrap().deref())
                    },
                    Self::Void |
                    Self::Null |
                    Self::Bool(_) |
                    Self::Number(_) |
                    Self::String(_) |
                    Self::Object(_) => Ordering::Greater,
                    _ => Ordering::Less,
                }
            },
            Self::Array(vals) => {
                match other {
                    Self::Array(ovals) => vals.cmp(&ovals),
                    Self::Boxed(oval) => {
                        self.cmp(oval.lock().unwrap().deref())
                    },
                    Self::Void |
                    Self::Null |
                    Self::Bool(_) |
                    Self::Number(_) |
                    Self::String(_) |
                    Self::Object(_) |
                    Self::FnPtr(_) => Ordering::Greater,
                    _ => Ordering::Less,
                }
            },
            Self::Tuple(vals) => {
                match other {
                    Self::Tuple(ovals) => vals.cmp(&ovals),
                    Self::Boxed(oval) => {
                        self.cmp(oval.lock().unwrap().deref())
                    },
                    Self::Void |
                    Self::Null |
                    Self::Bool(_) |
                    Self::Number(_) |
                    Self::String(_) |
                    Self::Object(_) |
                    Self::FnPtr(_) |
                    Self::Array(_) => Ordering::Greater,
                    _ => Ordering::Less,
                }
            },
            Self::Blob(vals) => {
                match other {
                    Self::Blob(ovals) => vals.cmp(&ovals),
                    Self::Boxed(oval) => {
                        self.cmp(oval.lock().unwrap().deref())
                    },
                    Self::Void |
                    Self::Null |
                    Self::Bool(_) |
                    Self::Number(_) |
                    Self::String(_) |
                    Self::Object(_) |
                    Self::FnPtr(_) |
                    Self::Array(_) |
                    Self::Tuple(_) => Ordering::Greater,
                    _ => Ordering::Less,
                }
            },
            Self::Set(set) => {
                match other {
                    Self::Set(oset) => set.cmp(&oset),
                    Self::Boxed(oval) => {
                        self.cmp(oval.lock().unwrap().deref())
                    },
                    Self::Void |
                    Self::Null |
                    Self::Bool(_) |
                    Self::Number(_) |
                    Self::String(_) |
                    Self::Object(_) |
                    Self::FnPtr(_) |
                    Self::Array(_) |
                    Self::Tuple(_) |
                    Self::Blob(_) => Ordering::Greater,
                    _ => Ordering::Less,
                }
            },
            Self::Map(map) => {
                match other {
                    Self::Map(omap) => map.cmp(&omap),
                    Self::Boxed(oval) => {
                        self.cmp(oval.lock().unwrap().deref())
                    },
                    Self::Void |
                    Self::Null |
                    Self::Bool(_) |
                    Self::Number(_) |
                    Self::String(_) |
                    Self::Object(_) |
                    Self::FnPtr(_) |
                    Self::Array(_) |
                    Self::Tuple(_) |
                    Self::Set(_) |
                    Self::Blob(_) => Ordering::Greater,
                }
            },
        }
    }
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
            Self::Void => {
                match other {
                    Self::Void => true,
                    Self::Boxed(oval) => {
                        self.eq(oval.lock().unwrap().deref())
                    },
                    _ => false,
                }
            },
            Self::Null => {
                match other {
                    Self::Null => true,
                    Self::Boxed(oval) => {
                        self.eq(oval.lock().unwrap().deref())
                    },
                    _ => false,
                }
            },
            Self::Bool(val) => {
                match other {
                    Self::Bool(oval) => *val == *oval,
                    Self::Boxed(oval) => {
                        self.eq(oval.lock().unwrap().deref())
                    },
                    _ => false
                }
            },
            Self::Object(nref) => {
                match other {
                    Self::Object(oref) => nref.id == oref.id,
                    Self::Boxed(oval) => {
                        self.eq(oval.lock().unwrap().deref())
                    },
                    _ => false,
                }
            },
            Self::Blob(vals) => {
                match other {
                    Self::Blob(ovals) => vals == ovals,
                    Self::Boxed(oval) => {
                        self.eq(oval.lock().unwrap().deref())
                    },
                    _ => false,
                }
            },
            Self::FnPtr(dref) => {
                match other {
                    Self::FnPtr(odref) => odref.id == dref.id,
                    Self::Boxed(oval) => {
                        self.eq(oval.lock().unwrap().deref())
                    },
                    _ => false,
                }
            },
            Self::Number(val) => {
                match other {
                    Self::Number(oval) => val.eq(oval),
                    Self::Boxed(oval) => {
                        self.eq(oval.lock().unwrap().deref())
                    },
                    _ => false,
                }
            },
            Self::String(val) => {
                match other {
                    Self::String(oval) => oval == val,
                    Self::Boxed(oval) => {
                        self.eq(oval.lock().unwrap().deref())
                    },
                    _ => false,
                }
            },
            Self::Array(vals) => {
                match other {
                    Self::Array(ovals) => vals == ovals,
                    Self::Boxed(oval) => {
                        self.eq(oval.lock().unwrap().deref())
                    },
                    _ => false,
                }
            },
            Self::Tuple(vals) => {
                match other {
                    Self::Tuple(ovals) => vals == ovals,
                    Self::Boxed(oval) => {
                        self.eq(oval.lock().unwrap().deref())
                    },
                    _ => false,
                }
            },
            Self::Set(set) => {
                match other {
                    Self::Set(oset) => set == oset,
                    Self::Boxed(oval) => {
                        self.eq(oval.lock().unwrap().deref())
                    },
                    _ => false,
                }
            },
            Self::Map(map) => {
                match other {
                    Self::Map(omap) => {
                        if map.len() == omap.len() {
                            for (k, v) in map {
                                if let Some(ov) = omap.get(k) {
                                    if v != ov {
                                        return false;
                                    }
                                } else {
                                    return false;
                                }
                            }
                            return true;
                        }
                        false
                    },
                    Self::Boxed(oval) => {
                        self.eq(oval.lock().unwrap().deref())
                    },
                    _ => false,
                }
            },
            Self::Boxed(val) => {
                val.lock().unwrap().eq(other)
            },
        }
    }
}
impl Eq for SVal {}
impl SVal {
    /// Schema equals another value?
    /// True if the values have the same type.
    pub fn schema_eq(&self, other: &Self, graph: &SGraph) -> bool {
        self.stype(graph) == other.stype(graph)
    }

    /// Is boxed?
    pub fn is_boxed(&self) -> bool {
        match self {
            Self::Boxed(_) => true,
            _ => false,
        }
    }

    /// Box this value.
    pub fn to_box(self) -> Self {
        if !self.is_boxed() {
            Self::Boxed(Arc::new(SMutex(Mutex::new(self))))
        } else {
            self
        }
    }

    /// Box this value by reference.
    pub fn to_box_ref(&mut self) {
        if !self.is_boxed() {
            *self = Self::Boxed(Arc::new(SMutex(Mutex::new(self.clone()))));
        }
    }

    /// Unbox this value if needed.
    pub fn unbox(mut self) -> Self {
        self.unbox_ref();
        self
    }

    /// Unbox ref.
    pub fn unbox_ref(&mut self) {
        match self {
            Self::Boxed(val) => {
                let tmp = val.lock().unwrap().clone();
                *self = tmp.unbox();
            },
            Self::Array(vals) => {
                for val in vals {
                    val.unbox_ref();
                }
            },
            Self::Tuple(vals) => {
                for val in vals {
                    val.unbox_ref();
                }
            },
            Self::Set(set) => {
                let mut new_set = BTreeSet::new();
                for val in set.iter() {
                    new_set.insert(val.clone().unbox());
                }
                *self = SVal::Set(new_set);
            },
            Self::Map(map) => {
                for (_k, v) in map {
                    v.unbox_ref(); // it's your own fault if you have boxed keys
                }
            },
            _ => {}
        }
    }

    /// Is void?
    pub fn is_void(&self) -> bool {
        match self {
            Self::Void => true,
            Self::Boxed(val) => {
                val.lock().unwrap().is_void()
            },
            _ => false,
        }
    }

    /// Is empty?
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Null |
            Self::Void => true,
            Self::Boxed(val) => {
                val.lock().unwrap().is_empty()
            },
            _ => false,
        }
    }

    /// Is null?
    pub fn is_null(&self) -> bool {
        match self {
            Self::Null => true,
            Self::Boxed(val) => {
                val.lock().unwrap().is_null()
            },
            _ => false,
        }
    }

    /// Is object?
    pub fn is_object(&self) -> bool {
        match self {
            Self::Object(_) => true,
            Self::Boxed(val) => {
                val.lock().unwrap().is_object()
            },
            _ => false,
        }
    }

    /// Is array?
    pub fn is_array(&self) -> bool {
        match self {
            Self::Array(_) => true,
            Self::Boxed(val) => {
                val.lock().unwrap().is_array()
            },
            _ => false,
        }
    }

    /// Is tuple?
    pub fn is_tuple(&self) -> bool {
        match self {
            Self::Tuple(_) => true,
            Self::Boxed(val) => {
                val.lock().unwrap().is_tuple()
            },
            _ => false,
        }
    }

    /// Is number?
    pub fn is_number(&self) -> bool {
        match self {
            Self::Number(_) => true,
            Self::Boxed(val) => {
                val.lock().unwrap().is_number()
            },
            _ => false,
        }
    }

    /// Is map?
    pub fn is_map(&self) -> bool {
        match self {
            Self::Map(_) => true,
            Self::Boxed(val) => {
                val.lock().unwrap().is_map()
            },
            _ => false,
        }
    }

    /// Is set?
    pub fn is_set(&self) -> bool {
        match self {
            Self::Set(_) => true,
            Self::Boxed(val) => {
                val.lock().unwrap().is_set()
            },
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
    pub fn stype(&self, graph: &SGraph) -> SType {
        match self {
            Self::Void => SType::Void,
            Self::Bool(_) => SType::Bool,
            Self::Number(val) => SType::Number(val.stype()),
            Self::String(_) => SType::String,
            Self::Array(_) => SType::Array,
            Self::Map(_) => SType::Map,
            Self::Set(_) => SType::Set,
            Self::Null => SType::Null,
            Self::Tuple(vals) => {
                let mut types: Vec<SType> = Vec::new();
                for val in vals { types.push(val.stype(graph)); }
                SType::Tuple(types)
            },
            Self::FnPtr(_) => SType::FnPtr,
            Self::Object(nref) => {
                if let Some(prototype) = SPrototype::get(graph, nref) {
                    // Use the full typepath here, so taht we arrive at the correct type when casting, etc...
                    if let Some(typepath) = prototype.typepath(graph) {
                        return SType::Object(typepath);
                    }
                }
                SType::Object("obj".to_string())
            },
            Self::Blob(_) => SType::Blob,
            Self::Boxed(val) => {
                let boxed_type = val.lock().unwrap().stype(graph);
                SType::Boxed(Box::new(boxed_type))
            },
        }
    }

    /// To string.
    pub fn to_string(&self) -> String {
        match self {
            Self::String(val) => { val.clone() },
            Self::Bool(val) => { val.to_string() },
            Self::Number(val) => { val.to_string() },
            Self::Array(vals) => { format!("{:?}", vals) },
            Self::Map(map) => { format!("{:?}", map) },
            Self::Set(set) => { format!("{:?}", set) },
            Self::Object(nref) => { format!("{:?}", nref) },
            Self::FnPtr(dref) => { format!("fn({:?})", dref) },
            Self::Null => { "null".to_string() },
            Self::Void => { "void".to_string() },
            Self::Tuple(tup) => { format!("tup({:?})", tup) },
            Self::Blob(blob) => { format!("blob({}bytes)", blob.len()) },
            Self::Boxed(val) => {
                val.lock().unwrap().to_string()
            },
        }
    }

    /// To string owned.
    pub fn owned_to_string(self) -> String {
        match self {
            Self::String(val) => { val },
            Self::Bool(val) => { val.to_string() },
            Self::Number(val) => { val.to_string() },
            Self::Array(vals) => { format!("{:?}", vals) },
            Self::Map(map) => { format!("{:?}", map) },
            Self::Set(set) => { format!("{:?}", set) },
            Self::Object(nref) => { format!("{:?}", nref) },
            Self::FnPtr(dref) => { format!("fn({:?})", dref) },
            Self::Null => { "null".to_string() },
            Self::Void => { "void".to_string() },
            Self::Tuple(tup) => { format!("tup({:?})", tup) },
            Self::Blob(blob) => { format!("blob({}bytes)", blob.len()) },
            Self::Boxed(val) => {
                val.lock().unwrap().to_string()
            },
        }
    }

    /// Truthy value for this val.
    pub fn truthy(&self) -> bool {
        match self {
            Self::Array(_) => true,
            Self::Map(_) => true,
            Self::Set(_) => true,
            Self::Bool(val) => *val,
            Self::FnPtr(_) => true,
            Self::Object(_) => true,
            Self::Null => false,
            Self::Number(val) => val.bool(),
            Self::String(val) => val.len() > 0,
            Self::Tuple(_) => true,
            Self::Void => false,
            Self::Blob(_) => true,
            Self::Boxed(val) => {
                val.lock().unwrap().truthy()
            },
        }
    }

    /// Typestack.
    pub fn type_stack(&self, graph: &SGraph) -> Vec<String> {
        match self {
            Self::Object(nref) => {
                if let Some(prototype) = SPrototype::get(graph, nref) {
                    return prototype.type_stack(graph);
                }
                Vec::default()
            },
            Self::Boxed(val) => {
                val.lock().unwrap().type_stack(graph)
            },
            _ => vec![]
        }
    }

    /// Instance of?
    pub fn instance_of(&self, graph: &SGraph, typename: &str) -> bool {
        for htype in self.type_stack(graph).iter().rev() {
            if htype == typename {
                return true;
            }
        }
        false
    }

    /// Typepath stack.
    pub fn typepath_stack(&self, graph: &SGraph) -> Vec<String> {
        match self {
            Self::Object(nref) => {
                if let Some(prototype) = SPrototype::get(graph, nref) {
                    return prototype.typepath_stack(graph);
                }
                Vec::default()
            },
            Self::Boxed(val) => {
                val.lock().unwrap().typepath_stack(graph)
            },
            _ => vec![]
        }
    }

    /// Instance of a typepath?
    pub fn instance_of_typepath(&self, graph: &SGraph, typepath: &str) -> bool {
        for htype in self.typepath_stack(graph).iter().rev() {
            if htype == typepath {
                return true;
            }
        }
        false
    }

    /// Typename.
    pub fn type_name(&self, graph: &SGraph) -> String {
        match self {
            Self::Object(nref) => {
                if let Some(prototype) = SPrototype::get(graph, nref) {
                    if let Some(name) = prototype.typename(graph) {
                        return name;
                    }
                }
                return "obj".to_string();
            },
            Self::Boxed(val) => {
                return val.lock().unwrap().type_name(graph);
            },
            _ => {}
        }
        let stype = self.stype(graph);
        stype.type_of()
    }

    /// Union this value with another, manipulating this value as the result.
    pub fn union(&mut self, other: &Self) {
        if other.is_boxed() {
            match other {
                SVal::Boxed(oval) => {
                    let lock = oval.lock().unwrap();
                    let other = lock.deref();
                    return self.union(other);
                },
                _ => {}
            }
        }
        match self {
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
            },
            SVal::Set(set) => {
                match other {
                    SVal::Set(oset) => {
                        *set = set.union(oset).cloned().collect();
                    },
                    _ => {
                        set.insert(other.clone());
                    }
                }
            },
            SVal::Map(map) => {
                match other {
                    SVal::Map(omap) => {
                        for (k, v) in omap {
                            if let Some(existing_val) = map.get_mut(k) {
                                existing_val.union(v);
                            } else {
                                map.insert(k.clone(), v.clone());
                            }
                        }
                    },
                    _ => {
                        *self = SVal::Array(vec![self.clone(), other.clone()]);
                    }
                }
            },
            Self::Boxed(val) => {
                val.lock().unwrap().union(other)
            },
        }
    }

    /// Cast a value to another type of value.
    pub fn cast(&self, target: SType, pid: &str, doc: &mut SDoc) -> Result<Self, SError> {
        match &target {
            SType::Boxed(target) => {
                // Make the resulting value boxed
                let target_type = target.deref().clone();
                let self_type = self.stype(&doc.graph);
                if target_type != self_type {
                    let value = self.cast(target.deref().clone(), pid, doc)?;
                    return Ok(Self::Boxed(Arc::new(SMutex(Mutex::new(value)))));
                } else {
                    return Ok(Self::Boxed(Arc::new(SMutex(Mutex::new(self.clone())))));
                }
            },
            _ => {}
        }
        match self {
            Self::Boxed(val) => {
                // Don't preserve the box, user needs to cast explicitly to a Boxed type to keep the box
                val.lock().unwrap().cast(target, pid, doc)
            },
            Self::Blob(blob) => {
                match target {
                    SType::Array => {
                        Ok(Self::Array(blob.iter().map(|byte| Self::Number(SNum::I64((*byte) as i64))).collect()))
                    },
                    SType::String => {
                        let res = str::from_utf8(blob.as_slice());
                        match res {
                            Ok(val) => {
                                Ok(Self::String(val.to_owned()))
                            },
                            Err(error) => {
                                Err(SError::val(pid, &doc, "cast", &error.to_string()))
                            }
                        }
                    },
                    _ => Err(SError::val(pid, &doc, "cast", &format!("cannot cast blob to {:?}", target)))
                }
            },
            Self::Array(vals) => {
                match target {
                    SType::Array => Ok(Self::Array(vals.clone())),
                    SType::Set => {
                        let mut set = BTreeSet::new();
                        for val in vals {
                            set.insert(val.clone());
                        }
                        return Ok(Self::Set(set));
                    },
                    SType::Blob => {
                        let mut blob: Vec<u8> = Vec::new();
                        for val in vals {
                            match val {
                                Self::Number(num) => {
                                    let res: Result<u8, _> = num.int().try_into();
                                    if res.is_err() {
                                        return Err(SError::val(pid, &doc, "cast", "cannot fit number into u8 while converting array to binary blob"));
                                    }
                                    blob.push(res.unwrap());
                                },
                                _ => {
                                    return Err(SError::val(pid, &doc, "cast", "cannot cast values that are not numerical into a u8 for blob cast"));
                                }
                            }
                        }
                        return Ok(Self::Blob(blob));
                    },
                    SType::Tuple(types) => {
                        let tup = Self::Tuple(vals.clone());
                        if tup.stype(&doc.graph) == SType::Tuple(types.clone()) {
                            return Ok(tup);
                        }
                        // Try to convert every individual value
                        if vals.len() == types.len() {
                            let mut new_vals: Vec<Self> = Vec::new();
                            for i in 0..types.len() {
                                let ty = types[i].clone();
                                let val = &vals[i];
                                let val_type = val.stype(&doc.graph);
                                
                                if val_type == ty {
                                    new_vals.push(val.clone());
                                } else {
                                    new_vals.push(val.cast(ty, pid, doc)?);
                                }
                            }
                            return Ok(Self::Tuple(new_vals));
                        }
                        Err(SError::val(pid, &doc, "cast", &format!("cannot cast array to type: {:?}", SType::Tuple(types.clone()))))
                    },
                    target => Err(SError::val(pid, &doc, "cast", &format!("cannot cast array to type: {:?}", target)))
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
                        return Err(SError::val(pid, &doc, "cast", &format!("cannot cast a bool to type: {:?}", ty)));
                    }
                }
            },
            Self::FnPtr(_) => {
                Err(SError::val(pid, &doc, "cast", "cannot cast a fn pointer to anything"))
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
                    stype => Err(SError::val(pid, &doc, "cast", &format!("cannot cast a number to type: {:?}", stype)))
                }
            },
            Self::String(val) => {
                match target {
                    SType::Array => {
                        Ok(Self::Array(vec![Self::String(val.clone())]))
                    },
                    SType::Set => {
                        let mut set = BTreeSet::new();
                        set.insert(Self::String(val.clone()));
                        Ok(Self::Set(set))
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
                                Err(SError::val(pid, &doc, "cast", &format!("value '{}' is not an int", val)))
                            },
                            SNumType::F64 => {
                                if let Ok(res) = val.replace('+', "").parse::<f64>() {
                                    return Ok(Self::Number(SNum::F64(res)));
                                }
                                Err(SError::val(pid, &doc, "cast", &format!("value '{}' is not a float", val)))
                            },
                            SNumType::Units(units) => {
                                if let Ok(res) = val.replace('+', "").parse::<f64>() {
                                    return Ok(Self::Number(SNum::Units(res, units)));
                                }
                                Err(SError::val(pid, &doc, "cast", &format!("value '{}' is not a float (to units)", val)))
                            },
                        }
                    },
                    stype => Err(SError::val(pid, &doc, "cast", &format!("cannot cast a string to type: {:?}", stype)))
                }
            },
            Self::Tuple(vals) => {
                match target {
                    SType::Array => {
                        return Ok(Self::Array(vals.clone()));
                    },
                    SType::Set => {
                        let set = vals.iter().cloned().collect();
                        return Ok(Self::Set(set));
                    },
                    SType::Tuple(types) => {
                        if types.len() == vals.len() {
                            let mut new_tup = Vec::new();
                            for i in 0..types.len() {
                                let val = &vals[i];
                                let ty = types[i].clone();
                                if val.stype(&doc.graph) != ty {
                                    new_tup.push(val.cast(ty, pid, doc)?);
                                } else {
                                    new_tup.push(val.clone());
                                }
                            }
                            return Ok(SVal::Tuple(new_tup));
                        }
                        return Err(SError::val(pid, &doc, "cast", "cannot cast tuple of one length into a tuple of another length"));
                    },
                    _ => {}
                }
                Err(SError::val(pid, &doc, "cast", &format!("cannot cast a tuple to type: {:?}", target)))
            },
            Self::Set(set) => {
                match target {
                    SType::Set => {
                        return Ok(Self::Set(set.clone()));
                    },
                    SType::Array => {
                        let vals = set.iter().cloned().collect();
                        return Ok(Self::Array(vals));
                    },
                    _ => {}
                }
                Err(SError::val(pid, &doc, "cast", &format!("cannot cast a set to type: {:?}", target)))
            },
            Self::Map(map) => {
                match target {
                    SType::Map => {
                        return Ok(SVal::Map(map.clone()));
                    },
                    SType::Set => {
                        let set = map.keys().cloned().collect();
                        return Ok(SVal::Set(set));
                    },
                    _ => {}
                }
                Err(SError::val(pid, &doc, "cast", &format!("cannot cast a map to type: {:?}", target)))
            },
            Self::Void => {
                Err(SError::val(pid, &doc, "cast", "cannot cast a void type"))
            },
            Self::Object(nref) => {
                match target {
                    SType::Object(typepath) => {
                        if typepath == "obj" || typepath == "root" { // Any object can cast to an obj, shouldn't hit "root" case though
                            return Ok(self.clone());
                        }

                        let current_scope;
                        if let Some(scope) = doc.self_ptr(pid) {
                            current_scope = scope;
                        } else if let Some(main) = doc.graph.main_root() {
                            current_scope = main;
                        } else {
                            current_scope = doc.graph.insert_root("root");
                        }

                        let mut type_path: Vec<&str> = typepath.split('.').collect();
                        let custom_type_name = type_path.pop().unwrap();

                        // Find a scope to use other than our own?
                        let mut type_scope = current_scope.clone();
                        if type_path.len() > 0 {
                            let path = type_path.join("/");
                            if path.starts_with("self") || path.starts_with("super") {
                                if let Some(nref) = doc.graph.node_ref(&path, Some(&type_scope)) {
                                    type_scope = nref;
                                } else {
                                    return Err(SError::val(pid, &doc, "cast", &format!("cannot find referenced type scope for casting an object to {}", typepath)));
                                }
                            } else {
                                if let Some(nref) = doc.graph.node_ref(&path, None) {
                                    type_scope = nref;
                                } else {
                                    return Err(SError::val(pid, &doc, "cast", &format!("cannot find referenced type scope for casting an object to {}", typepath)));
                                }
                            }
                        }

                        // Try assigning the prototype of this object since its not a value type
                        let mut success = false;
                        let mut typefields = Vec::new();
                        if let Some(custom_type) = doc.types.find(&doc.graph, custom_type_name, &type_scope) {
                            if custom_type.is_private() && !current_scope.is_child_of(&doc.graph, &type_scope) {
                                // Custom type is private and the current scope is not equal or a child of the type's scope
                                return Err(SError::val(pid, &doc, "cast", &format!("cannot cast expr to private object type: {}", typepath)));
                            }

                            // Check the current type of the object, to see if we already are an instance of this custom type
                            if self.instance_of_typepath(&doc.graph, &custom_type.typepath(&doc.graph)) {
                                return Ok(self.clone());
                            }

                            // Have to move typefields out of the borrow...
                            typefields = custom_type.fields.clone();

                            if let Some(mut prototype) = SPrototype::get(&doc.graph, nref) {
                                prototype.prototype = custom_type.locid.clone();
                                prototype.set(&mut doc.graph);
                            } else {
                                let mut prototype = custom_type.prototype();
                                prototype.attach(nref, &mut doc.graph);
                            }
                            success = true;
                        }
                        if success {
                            // Check for fields on this object in the correct type, otherwise create with the defaults from the custom type
                            for typefield in typefields {
                                if let Some(mut field) = SField::field(&doc.graph, &typefield.name, '.', Some(nref)) {
                                    let mut set = false;
                                    let existing_type = field.value.stype(&doc.graph);
                                    if existing_type != typefield.ptype {
                                        field.value = field.value.cast(typefield.ptype, pid, doc)?.unbox();
                                        set = true;
                                    }
                                    for (name, value) in typefield.attributes {
                                        if !field.attributes.contains_key(&name) {
                                            set = true;
                                            field.attributes.insert(name, value);
                                        }
                                    }
                                    if set {
                                        field.set(&mut doc.graph);
                                    }
                                } else if let Some(default) = &typefield.default {
                                    let default_value = default.exec(pid, doc)?;
                                    let mut field = SField::new(&typefield.name, default_value.unbox());
                                    field.attributes = typefield.attributes;
                                    field.attach(nref, &mut doc.graph);
                                } else {
                                    return Err(SError::val(pid, &doc, "cast", &format!("Could not find or create the field '{}' while casting object into '{}'", typefield.name, typepath)));
                                }
                            }
                            return Ok(self.clone());
                        }
                        Err(SError::val(pid, &doc, "cast", &format!("cannot cast expr to object type: {}", typepath)))
                    },
                    _ => Err(SError::val(pid, &doc, "cast", &format!("Cannot cast Object into {:?}", target)))
                }
            },
        }
    }

    /// Print this value.
    pub fn print(&self, doc: &mut SDoc) -> String {
        match self {
            Self::Boxed(val) => {
                val.lock().unwrap().print(doc)
            },
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
            Self::Map(map) => {
                format!("{:?}", map)
            },
            Self::Set(set) => {
                format!("{:?}", set)
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
                return JSON::stringify_node("main", &doc, nref).expect("Unable to export node during print to JSON");

                #[cfg(feature = "toml")]
                return TOML::stringify_node("main", &doc, nref).expect("Unable to export node during print to TOML");

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
            Self::Boxed(val) => {
                val.lock().unwrap().debug(doc)
            },
            Self::Void => {
                "void".to_string()
            },
            Self::Null => {
                "null".to_string()
            },
            Self::Map(map) => {
                format!("{:?}", map)
            },
            Self::Set(set) => {
                format!("{:?}", set)
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
    pub fn equal(&self, other: &Self) -> Result<Self, SError> {
        Ok((self == other).into())
    }

    /// Not equals.
    pub fn neq(&self, other: &Self) -> Result<Self, SError> {
        Ok((self != other).into())
    }

    /// Greater than other?
    pub fn gt(&self, other: &Self) -> Result<Self, SError> {
        if other.is_boxed() {
            match other {
                Self::Boxed(oval) => {
                    return self.gt(oval.lock().unwrap().deref());
                },
                _ => {}
            }
        }
        match self {
            Self::Boxed(val) => {
                val.lock().unwrap().gt(other)
            },
            Self::Array(_) => Ok(Self::Bool(false)),
            Self::Tuple(_) => Ok(Self::Bool(false)),
            Self::Bool(_) => Ok(Self::Bool(false)),
            Self::FnPtr(_) => Ok(Self::Bool(false)),
            Self::Void |
            Self::Null => Ok(Self::Bool(false)),
            Self::Blob(blob) => {
                match other {
                    Self::Blob(other_blob) => Ok(Self::Bool(blob.len() > other_blob.len())),
                    _ => Ok(Self::Bool(false))
                }
            },
            Self::Number(val) => {
                match other {
                    Self::Number(oval) => {
                        Ok(Self::Bool(val.gt(oval)))
                    },
                    _ => Ok(Self::Bool(false))
                }
            },
            Self::String(val) => {
                match other {
                    Self::String(oval) => Ok(Self::Bool(val > oval)),
                    _ => Ok(Self::Bool(false)),
                }
            },
            Self::Object(_) => Ok(Self::Bool(false)),
            Self::Map(map) => {
                match other {
                    Self::Map(omap) => Ok(Self::Bool(map.len() > omap.len())),
                    _ => Ok(Self::Bool(false)),
                }
            },
            Self::Set(set) => {
                match other {
                    Self::Set(oset) => Ok(Self::Bool(set.len() > oset.len())),
                    _ => Ok(Self::Bool(false)),
                }
            },
        }
    }

    /// Less than other?
    pub fn lt(&self, other: &Self) -> Result<Self, SError> {
        if other.is_boxed() {
            match other {
                Self::Boxed(oval) => {
                    return self.lt(oval.lock().unwrap().deref());
                },
                _ => {}
            }
        }
        match self {
            Self::Boxed(val) => val.lock().unwrap().lt(other),
            Self::Array(_) => Ok(Self::Bool(false)),
            Self::Tuple(_) => Ok(Self::Bool(false)),
            Self::Bool(_) => Ok(Self::Bool(false)),
            Self::FnPtr(_) => Ok(Self::Bool(false)),
            Self::Void |
            Self::Null => Ok(Self::Bool(false)),
            Self::Blob(blob) => {
                match other {
                    Self::Blob(other_blob) => Ok(Self::Bool(blob.len() < other_blob.len())),
                    _ => Ok(Self::Bool(false))
                }
            },
            Self::Number(val) => {
                match other {
                    Self::Number(oval) => {
                        Ok(Self::Bool(val.lt(oval)))
                    },
                    _ => Ok(Self::Bool(false))
                }
            },
            Self::String(val) => {
                match other {
                    Self::String(oval) => Ok(Self::Bool(val < oval)),
                    _ => Ok(Self::Bool(false)),
                }
            },
            Self::Object(_) => Ok(Self::Bool(false)),
            Self::Map(map) => {
                match other {
                    Self::Map(omap) => Ok(Self::Bool(map.len() < omap.len())),
                    _ => Ok(Self::Bool(false)),
                }
            },
            Self::Set(set) => {
                match other {
                    Self::Set(oset) => Ok(Self::Bool(set.len() < oset.len())),
                    _ => Ok(Self::Bool(false)),
                }
            },
        }
    }

    /// Greater than or equal?
    pub fn gte(&self, other: &Self) -> Result<Self, SError> {
        let mut res = self.gt(other)?;
        match res {
            Self::Bool(val) => {
                if val {
                    return Ok(Self::Bool(true));
                }
            },
            _ => {}
        }
        res = self.equal(other)?;
        match res {
            Self::Bool(_) => Ok(res),
            _ => Ok(Self::Bool(false))
        }
    }

    /// Less than or equal?
    pub fn lte(&self, other: &Self) -> Result<Self, SError> {
        let mut res = self.lt(other)?;
        match res {
            Self::Bool(val) => {
                if val {
                    return Ok(Self::Bool(true));
                }
            },
            _ => {}
        }
        res = self.equal(other)?;
        match res {
            Self::Bool(_) => Ok(res),
            _ => Ok(Self::Bool(false))
        }
    }

    /// Add.
    pub fn add(self, pid: &str, other: Self, doc: &mut SDoc) -> Result<Self, SError> {
        if other.is_boxed() {
            match other {
                Self::Boxed(oval) => {
                    return self.add(pid, oval.lock().unwrap().clone(), doc);
                },
                _ => {}
            }
        }
        match self {
            Self::Boxed(val) => {
                {
                    let mut lock = val.lock().unwrap();
                    let val = lock.deref_mut();
                    *val = val.clone().add(pid, other, doc)?;
                }
                Ok(Self::Boxed(val))
            },
            Self::Object(_) => Err(SError::val(pid, &doc, "add", "cannot add objects together")),
            Self:: Null |
            Self::Void => {
                Ok(other)
            },
            Self::Blob(mut blob) => {
                match other {
                    Self::Blob(mut other_blob) => {
                        blob.append(&mut other_blob);
                        Ok(Self::Blob(blob))
                    },
                    _ => Err(SError::val(pid, &doc, "add", "cannot add a non-blob to a blob"))
                }
            },
            Self::Bool(aval) => {
                match other {
                    Self::Object(_) => Err(SError::val(pid, &doc, "add", "cannot add objects together")),
                    Self::Null |
                    Self::Void => {
                        Ok(Self::Bool(aval))
                    },
                    Self::Bool(bval) => {
                        Ok(Self::Bool(aval && bval))
                    },
                    Self::Number(bval) => {
                        Ok(Self::String(format!("{}{}", aval, bval.print())))
                    },
                    Self::String(bval) => {
                        Ok(Self::String(format!("{}{}", aval, bval)))
                    },
                    Self::Array(_) => {
                        Err(SError::val(pid, &doc, "add", "cannot add a bool and an array"))
                    },
                    Self::Tuple(_) => {
                        Err(SError::val(pid, &doc, "add", "cannot add a bool and a tuple"))
                    },
                    Self::FnPtr(_) => {
                        Err(SError::val(pid, &doc, "add", "cannot add a bool and a function ptr"))
                    },
                    Self::Blob(_) => {
                        Err(SError::val(pid, &doc, "add", "cannot add a bool and a blob"))
                    },
                    Self::Map(_) => {
                        Err(SError::val(pid, &doc, "add", "cannot add a bool and a map"))
                    },
                    Self::Set(_) => {
                        Err(SError::val(pid, &doc, "add", "cannot add a bool and a set"))
                    },
                    Self::Boxed(_) => unreachable!("Boxed other is already taken care of")
                }
            },
            Self::String(aval) => {
                match other {
                    Self::Object(_) => Err(SError::val(pid, &doc, "add", "cannot add a string and an object")),
                    Self::Null |
                    Self::Void => {
                        Ok(Self::String(aval))
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
                        Err(SError::val(pid, &doc, "add", "cannot add a string and a blob"))
                    },
                    Self::Map(_) => {
                        Err(SError::val(pid, &doc, "add", "cannot add a string and a map"))
                    },
                    Self::Set(_) => {
                        Err(SError::val(pid, &doc, "add", "cannot add a string and a set"))
                    },
                    Self::Boxed(_) => unreachable!("Boxed other is already taken care of")
                }
            },
            Self::Number(aval) => {
                match other {
                    Self::Object(_) => Err(SError::val(pid, &doc, "add", "cannot add a number and an object")),
                    Self::Null |
                    Self::Void => {
                        Ok(Self::Number(aval))
                    },
                    Self::Number(bval) => {
                        Ok(Self::Number(aval.add(&bval)))
                    },
                    Self::String(bval) => {
                        if let Ok(bval) = bval.parse::<f64>() {
                            Ok(Self::Number(aval.add(&SNum::F64(bval))))
                        } else {
                            Err(SError::val(pid, &doc, "add", "this string cannot be parsed into a number for addition"))
                        }
                    },
                    Self::Bool(bval) => {
                        Ok(Self::String(format!("{}{}", aval.print(), bval)))
                    },
                    Self::Array(_) => {
                        Err(SError::val(pid, &doc, "add", "cannot add a number and an array"))
                    },
                    Self::Tuple(_) => {
                        Err(SError::val(pid, &doc, "add", "cannot add a number and a tuple"))
                    },
                    Self::FnPtr(_) => {
                        Err(SError::val(pid, &doc, "add", "cannot add a number and a function ptr"))
                    },
                    Self::Blob(_) => {
                        Err(SError::val(pid, &doc, "add", "cannot add a number and a blob"))
                    },
                    Self::Map(_) => {
                        Err(SError::val(pid, &doc, "add", "cannot add a number and a map"))
                    },
                    Self::Set(_) => {
                        Err(SError::val(pid, &doc, "add", "cannot add a number and a set"))
                    },
                    Self::Boxed(_) => unreachable!("Boxed other is already taken care of")
                }
            },
            Self::Array(mut vals) => {
                match other {
                    Self::Object(_) => Err(SError::val(pid, &doc, "add", "cannot add an array and an object")),
                    Self::Null |
                    Self::Void => {
                        Ok(Self::Array(vals))
                    },
                    Self::Bool(val) => {
                        vals.push(Self::Bool(val));
                        Ok(Self::Array(vals))
                    },
                    Self::Number(val) => {
                        vals.push(Self::Number(val));
                        Ok(Self::Array(vals))
                    },
                    Self::String(val) => {
                        vals.push(Self::String(val));
                        Ok(Self::Array(vals))
                    },
                    Self::Array(mut bvals) => {
                        vals.append(&mut bvals);
                        Ok(Self::Array(vals))
                    },
                    Self::Tuple(mut bvals) => {
                        vals.append(&mut bvals);
                        Ok(Self::Array(vals))
                    },
                    Self::FnPtr(dref) => {
                        vals.push(Self::FnPtr(dref));
                        Ok(Self::Array(vals))
                    },
                    Self::Blob(_) => {
                        // PID here doesn't matter, because they only get used when casting with objects...
                        let arr_blob = Self::Array(vals).cast(SType::Blob, "main", doc)?;
                        arr_blob.add(pid, other, doc)
                    },
                    Self::Set(set) => {
                        vals.append(&mut set.into_iter().collect());
                        Ok(Self::Array(vals))
                    },
                    Self::Map(_) => {
                        Err(SError::val(pid, &doc, "add", "cannot add an array and a map"))
                    },
                    Self::Boxed(_) => unreachable!("Boxed other is already taken care of")
                }
            },
            Self::Tuple(_) => {
                Err(SError::val(pid, &doc, "add", "cannot add a tuple with anything"))
            },
            Self::FnPtr(_) => {
                Err(SError::val(pid, &doc, "add", "cannot add a function pointer with anything"))
            },
            Self::Map(mut map) => {
                match other {
                    Self::Map(omap) => {
                        for (k, v) in omap {
                            if let Some(existing_val) = map.get_mut(&k) {
                                existing_val.union(&v);
                            } else {
                                map.insert(k.clone(), v.clone());
                            }
                        }
                        Ok(SVal::Map(map))
                    },
                    _ => {
                        Err(SError::val(pid, &doc, "add", "cannot add this value to a map"))
                    }
                }
            },
            Self::Set(mut set) => {
                match other {
                    Self::Set(oset) => {
                        for v in oset { set.insert(v); }
                        Ok(Self::Set(set))
                    },
                    Self::Array(ovals) => {
                        for v in ovals { set.insert(v); }
                        Ok(Self::Set(set))
                    },
                    _ => {
                        set.insert(other);
                        Ok(Self::Set(set))
                    }
                }
            },
        }
    }

    /// Subtract.
    pub fn sub(self, pid: &str, other: Self, doc: &mut SDoc) -> Result<Self, SError> {
        if other.is_boxed() {
            match other {
                Self::Boxed(oval) => {
                    return self.sub(pid, oval.lock().unwrap().clone(), doc);
                },
                _ => {}
            }
        }
        match self {
            Self::Boxed(val) => {
                {
                    let mut lock = val.lock().unwrap();
                    let val = lock.deref_mut();
                    *val = val.clone().sub(pid, other, doc)?;
                }
                Ok(Self::Boxed(val))
            },
            Self::Object(_) => Err(SError::val(pid, &doc, "subtract", "cannot subtract from an object")),
            Self::Null |
            Self::Void => {
                Err(SError::val(pid, &doc, "subtract", "cannot subtract from a null or void value"))
            },
            Self::Blob(_) => {
                Err(SError::val(pid, &doc, "subtract", "cannot subtract from a binary blob"))
            },
            Self::Bool(aval) => {
                match other {
                    Self::Object(_) => Err(SError::val(pid, &doc, "subtract", "cannot subtract an object from a bool")),
                    Self::Null |
                    Self::Void => {
                        Ok(Self::Bool(aval))
                    },
                    Self::Bool(bval) => {
                        Ok(Self::Bool(aval ^ bval))
                    },
                    Self::Number(_) => {
                        Err(SError::val(pid, &doc, "subtract", "cannot subtract a number from a bool"))
                    },
                    Self::String(_) => {
                        Err(SError::val(pid, &doc, "subtract", "cannot subtract a string from a bool"))
                    },
                    Self::Array(_) => {
                        Err(SError::val(pid, &doc, "subtract", "cannot subtract an array from a bool"))
                    },
                    Self::Tuple(_) => {
                        Err(SError::val(pid, &doc, "subtract", "cannot subtract a tuple from a bool"))
                    },
                    Self::FnPtr(_) => {
                        Err(SError::val(pid, &doc, "subtract", "cannot subtract a fn pointer from a bool"))
                    },
                    Self::Blob(_) => {
                        Err(SError::val(pid, &doc, "subtract", "cannot subtract a blob from a bool"))
                    },
                    Self::Map(_) => {
                        Err(SError::val(pid, &doc, "subtract", "cannot subtract a map from a bool"))
                    },
                    Self::Set(_) => {
                        Err(SError::val(pid, &doc, "subtract", "cannot subtract a set from a bool"))
                    },
                    Self::Boxed(_) => unreachable!("Boxed other is already taken care of"),
                }
            },
            Self::String(aval) => {
                match other {
                    Self::Object(_) => Err(SError::val(pid, &doc, "subtract", "cannot subtract an object from a string")),
                    Self::Null |
                    Self::Void => {
                        Ok(Self::String(aval))
                    },
                    Self::String(bval) => {
                        Ok(Self::String(aval.replace(&bval, "")))
                    },
                    Self::Number(bval) => {
                        Ok(Self::String(aval.replace(&bval.print(), "")))
                    },
                    Self::Bool(bval) => {
                        Ok(Self::String(aval.replace(&bval.to_string(), "")))
                    },
                    Self::Array(_) => {
                        Err(SError::val(pid, &doc, "subtract", "cannot subtract an array from a string"))
                    },
                    Self::Tuple(_) => {
                        Err(SError::val(pid, &doc, "subtract", "cannot subtract a tuple from a string"))
                    },
                    Self::FnPtr(_) => {
                        Err(SError::val(pid, &doc, "subtract", "cannot subtract a fn pointer from a string"))
                    },
                    Self::Blob(_) => {
                        Err(SError::val(pid, &doc, "subtract", "cannot subtract a blob from a string"))
                    },
                    Self::Map(_) => {
                        Err(SError::val(pid, &doc, "subtract", "cannot subtract a map from a string"))
                    },
                    Self::Set(_) => {
                        Err(SError::val(pid, &doc, "subtract", "cannot subtract a set from a string"))
                    },
                    Self::Boxed(_) => unreachable!("Boxed other is already taken care of"),
                }
            },
            Self::Number(aval) => {
                match other {
                    Self::Object(_) => Err(SError::val(pid, &doc, "subtract", "cannot subtract an object from a number")),
                    Self::Null |
                    Self::Void => {
                        Ok(Self::Number(aval))
                    },
                    Self::Number(bval) => {
                        Ok(Self::Number(aval.sub(&bval)))
                    },
                    Self::String(bval) => {
                        if let Ok(bval) = bval.parse::<f64>() {
                            Ok(Self::Number(aval.sub(&SNum::F64(bval))))
                        } else {
                            Err(SError::val(pid, &doc, "subtract", "this string cannot be parsed into a number for subtraction"))
                        }
                    },
                    Self::Bool(bval) => {
                        let mut num = 0;
                        if bval { num = 1; }
                        Ok(Self::Number(aval.sub(&SNum::I64(num))))
                    },
                    Self::Array(_) => {
                        Err(SError::val(pid, &doc, "subtract", "cannot subtract an array from a number"))
                    },
                    Self::Tuple(_) => {
                        Err(SError::val(pid, &doc, "subtract", "cannot subtract a tuple from a number"))
                    },
                    Self::FnPtr(_) => {
                        Err(SError::val(pid, &doc, "subtract", "cannot subtract a fn pointer from a number"))
                    },
                    Self::Blob(_) => {
                        Err(SError::val(pid, &doc, "subtract", "cannot subtract a blob from a number"))
                    },
                    Self::Map(_) => {
                        Err(SError::val(pid, &doc, "subtract", "cannot subtract a map from a number"))
                    },
                    Self::Set(_) => {
                        Err(SError::val(pid, &doc, "subtract", "cannot subtract a set from a number"))
                    },
                    Self::Boxed(_) => unreachable!("Boxed other is already taken care of"),
                }
            },
            Self::Array(_) => {
                Err(SError::val(pid, &doc, "subtract", "cannot subtract anything from an array"))
            },
            Self::Tuple(_) => {
                Err(SError::val(pid, &doc, "subtract", "cannot subtract anything from a tuple"))
            },
            Self::FnPtr(_) => {
                Err(SError::val(pid, &doc, "subtract", "cannot subtract anything from a function pointer"))
            },
            Self::Map(mut map) => {
                match other {
                    SVal::Array(ovals) => {
                        for oval in ovals {
                            map.remove(&oval);
                        }
                    },
                    SVal::Map(omap) => {
                        for key in omap.keys() {
                            map.remove(key);
                        }
                    },
                    SVal::Set(oset) => {
                        for key in oset {
                            map.remove(&key);
                        }
                    },
                    _ => {
                        map.remove(&other);
                    }
                }
                Ok(Self::Map(map))
            },
            Self::Set(mut set) => {
                match other {
                    SVal::Array(ovals) => {
                        for v in &ovals { set.remove(v); }
                    },
                    SVal::Set(oset) => {
                        for v in &oset { set.remove(v); }
                    },
                    _ => {
                        set.remove(&other);
                    }
                }
                Ok(Self::Set(set))
            },
        }
    }

    /// Multiply another value with this value.
    pub fn mul(self, pid: &str, other: Self, doc: &mut SDoc) -> Result<Self, SError> {
        if other.is_boxed() {
            match other {
                Self::Boxed(oval) => {
                    return self.mul(pid, oval.lock().unwrap().clone(), doc);
                },
                _ => {}
            }
        }
        match self {
            Self::Boxed(val) => {
                {
                    let mut lock = val.lock().unwrap();
                    let val = lock.deref_mut();
                    *val = val.clone().mul(pid, other, doc)?;
                }
                Ok(Self::Boxed(val))
            },
            Self::Object(_) => Err(SError::val(pid, &doc, "multiply", "cannot multiply objects")),
            Self::Null |
            Self::Void => {
                Ok(other)
            },
            Self::Blob(_) => {
                Err(SError::val(pid, &doc, "multiply", "cannot multiply a blob"))
            },
            Self::Bool(aval) => {
                match other {
                    Self::Object(_) => Err(SError::val(pid, &doc, "multiply", "cannot multiply a bool and object")),
                    Self::Null |
                    Self::Void => {
                        Ok(Self::Bool(aval))
                    },
                    Self::Bool(bval) => {
                        Ok(Self::Bool(aval || bval))
                    },
                    Self::Number(_) => {
                        Err(SError::val(pid, &doc, "multiply", "cannot multiply a bool and a number"))
                    },
                    Self::String(_) => {
                        Err(SError::val(pid, &doc, "multiply", "cannot multiply a bool and a string"))
                    },
                    Self::Array(_) => {
                        Err(SError::val(pid, &doc, "multiply", "cannot multiply a bool and an array"))
                    },
                    Self::Tuple(_) => {
                        Err(SError::val(pid, &doc, "multiply", "cannot multiply a bool and a tuple"))
                    },
                    Self::FnPtr(_) => {
                        Err(SError::val(pid, &doc, "multiply", "cannot multiply a bool and a function pointer"))
                    },
                    Self::Blob(_) => {
                        Err(SError::val(pid, &doc, "multiply", "cannot multiply a bool and a blob"))
                    },
                    Self::Map(_) => {
                        Err(SError::val(pid, &doc, "multiply", "cannot multiply a bool and a map"))
                    },
                    Self::Set(_) => {
                        Err(SError::val(pid, &doc, "multiply", "cannot multiply a bool and a set"))
                    },
                    Self::Boxed(_) => unreachable!("Boxed other is already taken care of"),
                }
            },
            Self::String(aval) => {
                match other {
                    Self::Object(_) => Err(SError::val(pid, &doc, "multiply", "cannot multiply a string and an object")),
                    Self::Null |
                    Self::Void => {
                        Ok(Self::String(aval))
                    },
                    Self::String(bval) => {
                        Ok(Self::String(format!("{}{}", aval, bval)))
                    },
                    Self::Number(bval) => {
                        let mut other = String::default();
                        for _ in 0..bval.int() {
                            other.push_str(&aval);
                        }
                        Ok(Self::String(other))
                    },
                    Self::Bool(bval) => {
                        Ok(Self::String(format!("{}{}", aval, bval)))
                    },
                    Self::Array(_) => {
                        Err(SError::val(pid, &doc, "multiply", "cannot multiply a string and an array"))
                    },
                    Self::Tuple(_) => {
                        Err(SError::val(pid, &doc, "multiply", "cannot multiply a string and a tuple"))
                    },
                    Self::FnPtr(_) => {
                        Err(SError::val(pid, &doc, "multiply", "cannot multiply a string and a function pointer"))
                    },
                    Self::Blob(_) => {
                        Err(SError::val(pid, &doc, "multiply", "cannot multiply a string and a blob"))
                    },
                    Self::Map(_) => {
                        Err(SError::val(pid, &doc, "multiply", "cannot multiply a string and a map"))
                    },
                    Self::Set(_) => {
                        Err(SError::val(pid, &doc, "multiply", "cannot multiply a string and a set"))
                    },
                    Self::Boxed(_) => unreachable!("Boxed other is already taken care of"),
                }
            },
            Self::Number(aval) => {
                match other {
                    Self::Object(_) => Err(SError::val(pid, &doc, "multiply", "cannot multiply a number and an object")),
                    Self::Null |
                    Self::Void => {
                        Ok(Self::Number(aval))
                    },
                    Self::Number(bval) => {
                        Ok(Self::Number(aval.mul(&bval)))
                    },
                    Self::String(bval) => {
                        if let Ok(bval) = bval.parse::<f64>() {
                            Ok(Self::Number(aval.mul(&SNum::F64(bval))))
                        } else {
                            Err(SError::val(pid, &doc, "multiply", "this string cannot be parsed into a number for multiplication"))
                        }
                    },
                    Self::Bool(_) => {
                        Err(SError::val(pid, &doc, "multiply", "cannot multiply a number and a bool"))
                    },
                    Self::Array(_) => {
                        Err(SError::val(pid, &doc, "multiply", "cannot multiply a number and an array"))
                    },
                    Self::Tuple(_) => {
                        Err(SError::val(pid, &doc, "multiply", "cannot multiply a number and a tuple"))
                    },
                    Self::FnPtr(_) => {
                        Err(SError::val(pid, &doc, "multiply", "cannot multiply a number and a function pointer"))
                    },
                    Self::Blob(_) => {
                        Err(SError::val(pid, &doc, "multiply", "cannot multiply a number and a blob"))
                    },
                    Self::Map(_) => {
                        Err(SError::val(pid, &doc, "multiply", "cannot multiply a number and a map"))
                    },
                    Self::Set(_) => {
                        Err(SError::val(pid, &doc, "multiply", "cannot multiply a number and a set"))
                    },
                    Self::Boxed(_) => unreachable!("Boxed other is already taken care of"),
                }
            },
            Self::Array(_) => {
                Err(SError::val(pid, &doc, "multiply", "cannot multiply an array"))
            },
            Self::Tuple(_) => {
                Err(SError::val(pid, &doc, "multiply", "cannot multiply a tuple"))
            },
            Self::FnPtr(_) => {
                Err(SError::val(pid, &doc, "multiply", "cannot multiply a function"))
            },
            Self::Set(set) => {
                match other {
                    Self::Set(oset) => {
                        // Perform an intersection with this other set
                        Ok(Self::Set(set.intersection(&oset).into_iter().cloned().collect()))
                    },
                    Self::Array(ovals) => {
                        let oset = ovals.into_iter().collect::<BTreeSet<SVal>>();
                        Ok(Self::Set(set.intersection(&oset).into_iter().cloned().collect()))
                    },
                    _ => {
                        Err(SError::val(pid, &doc, "multiply", "cannot intersect a set with a non-set and non-array value"))
                    }
                }
            },
            Self::Map(mut map) => {
                match other {
                    Self::Map(omap) => {
                        // Perform in intersection with this other map
                        // Keep only the keys that this other map has
                        let mut to_remove = Vec::new();
                        for (k, _v) in &map {
                            if !omap.contains_key(k) {
                                to_remove.push(k.clone());
                            }
                        }
                        for key in to_remove {
                            map.remove(&key);
                        }
                        return Ok(Self::Map(map));
                    },
                    Self::Array(vals) => {
                        // Perform an intersection with this array
                        // Keep only the keys that are in this array
                        let hashset = vals.into_iter().collect::<HashSet<SVal>>();
                        let mut to_remove = Vec::new();
                        for (k, _v) in &map {
                            if !hashset.contains(k) {
                                to_remove.push(k.clone());
                            }
                        }
                        for key in to_remove {
                            map.remove(&key);
                        }
                        return Ok(Self::Map(map));
                    },
                    Self::Set(vals) => {
                        // Perform an intersection with this set
                        // Keep only the keys that are in this set
                        let mut to_remove = Vec::new();
                        for (k, _v) in &map {
                            if !vals.contains(k) {
                                to_remove.push(k.clone());
                            }
                        }
                        for key in to_remove {
                            map.remove(&key);
                        }
                        return Ok(Self::Map(map));
                    },
                    _ => {}
                }
                Err(SError::val(pid, &doc, "multiply", "cannot intersect a map with a non-map, non-array, and non-set value"))
            }
        }
    }

    /// Divide another value with this value.
    pub fn div(self, pid: &str, other: Self, doc: &mut SDoc) -> Result<Self, SError> {
        if other.is_boxed() {
            match other {
                Self::Boxed(oval) => {
                    return self.div(pid, oval.lock().unwrap().clone(), doc);
                },
                _ => {}
            }
        }
        match self {
            Self::Boxed(val) => {
                {
                    let mut lock = val.lock().unwrap();
                    let val = lock.deref_mut();
                    *val = val.clone().div(pid, other, doc)?;
                }
                Ok(Self::Boxed(val))
            },
            Self::Object(_) => Err(SError::val(pid, &doc, "divide", "cannot divide an object")),
            Self::Null |
            Self::Void => {
                Ok(other)
            },
            Self::Blob(_) => {
                Err(SError::val(pid, &doc, "divide", "cannot divide a blob"))
            },
            Self::Bool(aval) => {
                match other {
                    Self::Object(_) => Err(SError::val(pid, &doc, "divide", "cannot divide a bool with an object")),
                    Self::Null |
                    Self::Void => {
                        Ok(Self::Bool(aval))
                    },
                    Self::Bool(bval) => {
                        Ok(Self::Bool(aval && bval))
                    },
                    Self::Number(_) => {
                        Err(SError::val(pid, &doc, "divide", "cannot divide a bool with a number"))
                    },
                    Self::String(_) => {
                        Err(SError::val(pid, &doc, "divide", "cannot divide a bool with a string"))
                    },
                    Self::Array(_) => {
                        Err(SError::val(pid, &doc, "divide", "cannot divide a bool with an array"))
                    },
                    Self::Tuple(_) => {
                        Err(SError::val(pid, &doc, "divide", "cannot divide a bool with a tuple"))
                    },
                    Self::FnPtr(_) => {
                        Err(SError::val(pid, &doc, "divide", "cannot divide a bool with a function pointer"))
                    },
                    Self::Blob(_) => {
                        Err(SError::val(pid, &doc, "divide", "cannot divide a bool with a blob"))
                    },
                    Self::Map(_) => {
                        Err(SError::val(pid, &doc, "divide", "cannot divide a bool with a map"))
                    },
                    Self::Set(_) => {
                        Err(SError::val(pid, &doc, "divide", "cannot divide a bool with a set"))
                    },
                    Self::Boxed(_) => unreachable!("Boxed other is already taken care of"),
                }
            },
            Self::String(aval) => {
                match other {
                    Self::Object(_) => Err(SError::val(pid, &doc, "divide", "cannot divide a string with an object")),
                    Self::Null |
                    Self::Void => {
                        Ok(Self::String(aval))
                    },
                    Self::String(bval) => {
                        let vec = aval.split(&bval).collect::<Vec<&str>>();
                        let mut new: Vec<Self> = Vec::new();
                        for v in vec {
                            new.push(v.into());
                        }
                        Ok(Self::Array(new))
                    },
                    Self::Number(_) => {
                        Err(SError::val(pid, &doc, "divide", "cannot divide a string with a number"))
                    },
                    Self::Bool(_) => {
                        Err(SError::val(pid, &doc, "divide", "cannot divide a string with a bool"))
                    },
                    Self::Array(_) => {
                        Err(SError::val(pid, &doc, "divide", "cannot divide a string with an array"))
                    },
                    Self::Tuple(_) => {
                        Err(SError::val(pid, &doc, "divide", "cannot divide a string with a tuple"))
                    },
                    Self::FnPtr(_) => {
                        Err(SError::val(pid, &doc, "divide", "cannot divide a string with a function pointer"))
                    },
                    Self::Blob(_) => {
                        Err(SError::val(pid, &doc, "divide", "cannot divide a string with a blob"))
                    },
                    Self::Map(_) => {
                        Err(SError::val(pid, &doc, "divide", "cannot divide a string with a map"))
                    },
                    Self::Set(_) => {
                        Err(SError::val(pid, &doc, "divide", "cannot divide a string with a set"))
                    },
                    Self::Boxed(_) => unreachable!("Boxed other is already taken care of"),
                }
            },
            Self::Number(aval) => {
                match other {
                    Self::Object(_) => Err(SError::val(pid, &doc, "divide", "cannot divide a number with an object")),
                    Self::Null |
                    Self::Void => {
                        Ok(Self::Number(aval))
                    },
                    Self::Number(bval) => {
                        Ok(Self::Number(aval.div(&bval)))
                    },
                    Self::String(bval) => {
                        if let Ok(bval) = bval.parse::<f64>() {
                            Ok(Self::Number(aval.div(&SNum::F64(bval))))
                        } else {
                            Err(SError::val(pid, &doc, "divide", "this string cannot be parsed into a number for division"))
                        }
                    },
                    Self::Bool(_) => {
                        Err(SError::val(pid, &doc, "divide", "cannot divide a number with a bool"))
                    },
                    Self::Array(_) => {
                        Err(SError::val(pid, &doc, "divide", "cannot divide a number with an array"))
                    },
                    Self::Tuple(_) => {
                        Err(SError::val(pid, &doc, "divide", "cannot divide a number with a tuple"))
                    },
                    Self::FnPtr(_) => {
                        Err(SError::val(pid, &doc, "divide", "cannot divide a number with a function pointer"))
                    },
                    Self::Blob(_) => {
                        Err(SError::val(pid, &doc, "divide", "cannot divide a number with a blob"))
                    },
                    Self::Map(_) => {
                        Err(SError::val(pid, &doc, "divide", "cannot divide a number with a map"))
                    },
                    Self::Set(_) => {
                        Err(SError::val(pid, &doc, "divide", "cannot divide a number with a set"))
                    },
                    Self::Boxed(_) => unreachable!("Boxed other is already taken care of"),
                }
            },
            Self::Array(_) => {
                Err(SError::val(pid, &doc, "divide", "cannot divide an array"))
            },
            Self::Tuple(_) => {
                Err(SError::val(pid, &doc, "divide", "cannot divide a tuple"))
            },
            Self::FnPtr(_) => {
                Err(SError::val(pid, &doc, "divide", "cannot divide a function pointer"))
            },
            Self::Map(_) => {
                Err(SError::val(pid, &doc, "divide", "cannot divide a map"))
            },
            Self::Set(_) => {
                Err(SError::val(pid, &doc, "divide", "cannot divide a set"))
            },
        }
    }

    /// Modulus/remainder (mod) another value with this value.
    pub fn rem(self, pid: &str, other: Self, doc: &mut SDoc) -> Result<Self, SError> {
        if other.is_boxed() {
            match other {
                Self::Boxed(oval) => {
                    return self.rem(pid, oval.lock().unwrap().clone(), doc);
                },
                _ => {}
            }
        }
        match self {
            Self::Boxed(val) => {
                {
                    let mut lock = val.lock().unwrap();
                    let val = lock.deref_mut();
                    *val = val.clone().rem(pid, other, doc)?;
                }
                Ok(Self::Boxed(val))
            },
            Self::Object(_) => Err(SError::val(pid, &doc, "modulo", "cannot divide an object")),
            Self::Null |
            Self::Void => {
                Ok(other)
            },
            Self::Blob(_) => {
                Err(SError::val(pid, &doc, "modulo", "cannot divide a blob"))
            },
            Self::Bool(aval) => {
                match other {
                    Self::Object(_) => Err(SError::val(pid, &doc, "modulo", "cannot divide a bool with an object")),
                    Self::Null |
                    Self::Void => {
                        Ok(Self::Bool(aval))
                    },
                    Self::Bool(bval) => {
                        Ok(Self::Bool(aval && bval))
                    },
                    Self::Number(_) => {
                        Err(SError::val(pid, &doc, "modulo", "cannot divide a bool with a number"))
                    },
                    Self::String(_) => {
                        Err(SError::val(pid, &doc, "modulo", "cannot divide a bool with a string"))
                    },
                    Self::Array(_) => {
                        Err(SError::val(pid, &doc, "modulo", "cannot divide a bool with an array"))
                    },
                    Self::Tuple(_) => {
                        Err(SError::val(pid, &doc, "modulo", "cannot divide a bool with a tuple"))
                    },
                    Self::FnPtr(_) => {
                        Err(SError::val(pid, &doc, "modulo", "cannot divide a bool with a function pointer"))
                    },
                    Self::Blob(_) => {
                        Err(SError::val(pid, &doc, "modulo", "cannot divide a bool with a blob"))
                    },
                    Self::Map(_) => {
                        Err(SError::val(pid, &doc, "modulo", "cannot divide a bool with a map"))
                    },
                    Self::Set(_) => {
                        Err(SError::val(pid, &doc, "modulo", "cannot divide a bool with a set"))
                    },
                    Self::Boxed(_) => unreachable!("Boxed other is already taken care of"),
                }
            },
            Self::String(aval) => {
                match other {
                    Self::Object(_) => Err(SError::val(pid, &doc, "modulo", "cannot divide a string with an object")),
                    Self::Null |
                    Self::Void => {
                        Ok(Self::String(aval))
                    },
                    Self::String(bval) => {
                        let vec = aval.split(&bval).collect::<Vec<&str>>();
                        let mut new: Vec<Self> = Vec::new();
                        for v in vec {
                            new.push(v.into());
                        }
                        Ok(Self::Array(new))
                    },
                    Self::Number(_) => {
                        Err(SError::val(pid, &doc, "modulo", "cannot divide a string with a number"))
                    },
                    Self::Bool(_) => {
                        Err(SError::val(pid, &doc, "modulo", "cannot divide a string with a bool"))
                    },
                    Self::Array(_) => {
                        Err(SError::val(pid, &doc, "modulo", "cannot divide a string with an array"))
                    },
                    Self::Tuple(_) => {
                        Err(SError::val(pid, &doc, "modulo", "cannot divide a string with a tuple"))
                    },
                    Self::FnPtr(_) => {
                        Err(SError::val(pid, &doc, "modulo", "cannot divide a string with a function pointer"))
                    },
                    Self::Blob(_) => {
                        Err(SError::val(pid, &doc, "modulo", "cannot divide a string with a blob"))
                    },
                    Self::Map(_) => {
                        Err(SError::val(pid, &doc, "modulo", "cannot divide a string with a map"))
                    },
                    Self::Set(_) => {
                        Err(SError::val(pid, &doc, "modulo", "cannot divide a string with a set"))
                    },
                    Self::Boxed(_) => unreachable!("Boxed other is already taken care of"),
                }
            },
            Self::Number(aval) => {
                match other {
                    Self::Object(_) => Err(SError::val(pid, &doc, "modulo", "cannot divide a number with an object")),
                    Self::Null |
                    Self::Void => {
                        Ok(Self::Number(aval))
                    },
                    Self::Number(bval) => {
                        Ok(Self::Number(aval.rem(&bval)))
                    },
                    Self::String(bval) => {
                        if let Ok(bval) = bval.parse::<f64>() {
                            Ok(Self::Number(aval.rem(&SNum::F64(bval))))
                        } else {
                            Err(SError::val(pid, &doc, "modulo", "this string cannot be parsed into a number for division"))
                        }
                    },
                    Self::Bool(_) => {
                        Err(SError::val(pid, &doc, "modulo", "cannot divide a number with a bool"))
                    },
                    Self::Array(_) => {
                        Err(SError::val(pid, &doc, "modulo", "cannot divide a number with an array"))
                    },
                    Self::Tuple(_) => {
                        Err(SError::val(pid, &doc, "modulo", "cannot divide a number with a tuple"))
                    },
                    Self::FnPtr(_) => {
                        Err(SError::val(pid, &doc, "modulo", "cannot divide a number with a function pointer"))
                    },
                    Self::Blob(_) => {
                        Err(SError::val(pid, &doc, "modulo", "cannot divide a number with a blob"))
                    },
                    Self::Map(_) => {
                        Err(SError::val(pid, &doc, "modulo", "cannot divide a number with a map"))
                    },
                    Self::Set(_) => {
                        Err(SError::val(pid, &doc, "modulo", "cannot divide a number with a set"))
                    },
                    Self::Boxed(_) => unreachable!("Boxed other is already taken care of"),
                }
            },
            Self::Array(_) => {
                Err(SError::val(pid, &doc, "modulo", "cannot divide an array"))
            },
            Self::Tuple(_) => {
                Err(SError::val(pid, &doc, "modulo", "cannot divide a tuple"))
            },
            Self::FnPtr(_) => {
                Err(SError::val(pid, &doc, "modulo", "cannot divide a function pointer"))
            },
            Self::Set(set) => {
                match other {
                    Self::Set(oset) => {
                        // Perform an symmetric difference with this other set
                        Ok(Self::Set(set.symmetric_difference(&oset).into_iter().cloned().collect()))
                    },
                    Self::Array(ovals) => {
                        let oset = ovals.into_iter().collect::<BTreeSet<SVal>>();
                        Ok(Self::Set(set.symmetric_difference(&oset).into_iter().cloned().collect()))
                    },
                    _ => {
                        Err(SError::val(pid, &doc, "modulo", "cannot perform a symmetric difference on a set with anything other than another set or array value"))
                    }
                }
            },
            Self::Map(mut map) => {
                match other {
                    Self::Map(omap) => {
                        // Perform symmetric diff with this other map
                        // Keep only the keys that don't intersect
                        let mut intersections = HashSet::new();
                        let mut to_remove = Vec::new();
                        for (k, _v) in &map {
                            if omap.contains_key(k) {
                                to_remove.push(k.clone());
                                intersections.insert(k.clone());
                            }
                        }
                        for key in to_remove {
                            map.remove(&key);
                        }
                        for (k, v) in omap {
                            if !intersections.contains(&k) {
                                map.insert(k, v);
                            }
                        }
                        return Ok(Self::Map(map));
                    },
                    _ => {}
                }
                Err(SError::val(pid, &doc, "modulo", "cannot perform a symmetric difference with a map and a non-map value"))
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
#[derive(Hash)]
struct NumHash(u8, u64, u64);
impl Hash for SNum {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::I64(val) => val.hash(state),
            Self::F64(val) => {
                let mut sign = 1;
                if val.signum() < 0. { sign = 2; }
                NumHash(sign, val.trunc() as u64, (val.fract() * 1000000.) as u64).hash(state)
            },
            Self::Units(val, _units) => {
                let mut sign = 1;
                if val.signum() < 0. { sign = 2; }
                NumHash(sign, val.trunc() as u64, (val.fract() * 1000000.) as u64).hash(state)
            },
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
