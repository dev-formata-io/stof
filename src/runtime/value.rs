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
use nanoid::nanoid;
use parking_lot::RwLock;
use rustc_hash::FxHashMap;
use std::{cmp::Ordering, hash::{Hash, Hasher}, ops::{Deref, DerefMut}, sync::Arc};
use arcstr::{literal, ArcStr};
use bytes::Bytes;
use imbl::{vector, OrdMap, OrdSet, Vector};
use serde::{Deserialize, Serialize};
use crate::{model::{export::json_value_from_node, Data, DataRef, Field, Func, Graph, Node, NodeRef, Prototype, SId}, parser::{number::number, semver::parse_semver_alone}, runtime::{Error, Num, NumT, Type, Units, DATA, OBJ}};


/// Value reference (value, by reference?).
#[derive(Serialize, Deserialize, Debug)]
pub struct ValRef<T: ?Sized>(pub Arc<RwLock<T>>, pub bool);
impl<T: ?Sized + Hash> Hash for ValRef<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.read().hash(state);
    }
}
impl<T: ?Sized> Deref for ValRef<T> {
    type Target = Arc<RwLock<T>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T: ?Sized> DerefMut for ValRef<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl ValRef<Val> {
    /// Create a new ValRef from a value.
    pub fn new(val: Val) -> Self {
        Self(Arc::new(RwLock::new(val)), false)
    }

    /// Duplicate this value, taking into account the reference setting & value type.
    /// If by_ref is set to true, then the value is forced to duplicate by reference.
    pub fn duplicate(&self, by_ref: bool) -> Self {
        let value_type = self.0.read().val_type();
        if value_type && !self.1 && !by_ref { // by value instead of by reference
            Self(Arc::new(RwLock::new(self.0.read().clone())), self.1)
        } else {
            let mut clone = self.clone();
            if by_ref { clone.1 = true; }
            clone
        }
    }
}

impl PartialEq for ValRef<Val> {
    fn eq(&self, other: &Self) -> bool {
        self.0.read().eq(&other.0.read())
    }
}
impl Eq for ValRef<Val> {}
impl PartialOrd for ValRef<Val> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.read().partial_cmp(&other.0.read())
    }
}
impl Ord for ValRef<Val> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.read().cmp(&other.0.read())
    }
}
impl Clone for ValRef<Val> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1)
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, Default, Hash)]
/// Value.
pub enum Val {
    #[default]
    Void,
    Null,

    // A reference to another process (pid)
    Promise(SId, Type),
    
    // Scalar types
    Bool(bool),
    Num(Num),
    Str(ArcStr),
    Blob(Vec<u8>),

    // Semantic Versioning as a value
    Ver(i32, i32, i32, Option<ArcStr>, Option<ArcStr>),

    // Reference types
    Obj(NodeRef),
    Fn(DataRef),
    Data(DataRef),

    // Compound types
    List(Vector<ValRef<Self>>),
    Tup(Vector<ValRef<Self>>),
    Map(OrdMap<ValRef<Self>, ValRef<Self>>),
    Set(OrdSet<ValRef<Self>>),
}

impl From<&char> for Val {
    fn from(value: &char) -> Self {
        Self::Str(value.to_string().into())
    }
}
impl From<&str> for Val {
    fn from(value: &str) -> Self {
        Self::Str(value.into())
    }
}
impl From<&SId> for Val {
    fn from(value: &SId) -> Self {
        Self::Str(value.as_ref().into())
    }
}
impl From<u8> for Val {
    fn from(value: u8) -> Self {
        Self::Num(Num::Int(value as i64))
    }
}
impl From<u16> for Val {
    fn from(value: u16) -> Self {
        Self::Num(Num::Int(value as i64))
    }
}
impl From<u32> for Val {
    fn from(value: u32) -> Self {
        Self::Num(Num::Int(value as i64))
    }
}
impl From<u64> for Val {
    fn from(value: u64) -> Self {
        Self::Num(Num::Int(value as i64))
    }
}
impl From<u128> for Val {
    fn from(value: u128) -> Self {
        Self::Num(Num::Int(value as i64))
    }
}
impl From<i8> for Val {
    fn from(value: i8) -> Self {
        Self::Num(Num::Int(value as i64))
    }
}
impl From<i16> for Val {
    fn from(value: i16) -> Self {
        Self::Num(Num::Int(value as i64))
    }
}
impl From<i32> for Val {
    fn from(value: i32) -> Self {
        Self::Num(Num::Int(value as i64))
    }
}
impl From<i64> for Val {
    fn from(value: i64) -> Self {
        Self::Num(Num::Int(value))
    }
}
impl From<i128> for Val {
    fn from(value: i128) -> Self {
        Self::Num(Num::Int(value as i64))
    }
}
impl From<f32> for Val {
    fn from(value: f32) -> Self {
        Self::Num(Num::Float(value as f64))
    }
}
impl From<f64> for Val {
    fn from(value: f64) -> Self {
        Self::Num(Num::Float(value))
    }
}
impl From<bool> for Val {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}
impl From<Vec<u8>> for Val {
    fn from(value: Vec<u8>) -> Self {
        Self::Blob(value)
    }
}
impl From<Bytes> for Val {
    fn from(value: Bytes) -> Self {
        Self::from(value.to_vec())
    }
}
impl<T: Into<Val>> From<(T, Units)> for Val {
    fn from(value: (T, Units)) -> Self {
        let mut val: Val = value.0.into();
        match &mut val {
            Val::Num(num) => {
                *num = num.cast(NumT::Units(value.1));
            },
            _ => {}
        }
        val
    }
}

impl PartialOrd for Val {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Val {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            Self::Void => Ordering::Less,
            Self::Null => Ordering::Greater,
            Self::Bool(v) => {
                match other {
                    Self::Bool(ov) => v.cmp(ov),
                    Self::Void => Ordering::Greater,
                    _ => Ordering::Less,
                }
            },
            Self::Num(v) => {
                match other {
                    Self::Num(ov) => {
                        if v.gt(ov) {
                            Ordering::Greater
                        } else if v.lt(ov) {
                            Ordering::Less
                        } else {
                            Ordering::Equal
                        }
                    },
                    Self::Bool(_) |
                    Self::Void => Ordering::Greater,
                    _ => Ordering::Less,
                }
            },
            Self::Str(val) => {
                match other {
                    Self::Str(oval) => val.cmp(oval),
                    Self::Void |
                    Self::Bool(_) |
                    Self::Num(_) => Ordering::Greater,
                    _ => Ordering::Less,
                }
            },
            Self::Obj(nref) => {
                match other {
                    Self::Obj(oref) => nref.cmp(oref),
                    Self::Void |
                    Self::Bool(_) |
                    Self::Num(_) |
                    Self::Str(_) => Ordering::Greater,
                    _ => Ordering::Less,
                }
            },
            Self::Fn(dref) => {
                match other {
                    Self::Fn(oref) => dref.cmp(oref),
                    Self::Void |
                    Self::Bool(_) |
                    Self::Num(_) |
                    Self::Str(_) |
                    Self::Obj(_) => Ordering::Greater,
                    _ => Ordering::Less,
                }
            },
            Self::List(vals) => {
                match other {
                    Self::List(ovals) => vals.cmp(ovals),
                    Self::Void |
                    Self::Bool(_) |
                    Self::Num(_) |
                    Self::Str(_) |
                    Self::Obj(_) |
                    Self::Fn(_) => Ordering::Greater,
                    _ => Ordering::Less,
                }
            },
            Self::Tup(vals) => {
                match other {
                    Self::Tup(ovals) => vals.cmp(&ovals),
                    Self::Void |
                    Self::Bool(_) |
                    Self::Num(_) |
                    Self::Str(_) |
                    Self::Obj(_) |
                    Self::Fn(_) |
                    Self::List(_) => Ordering::Greater,
                    _ => Ordering::Less,
                }
            },
            Self::Blob(vals) => {
                match other {
                    Self::Blob(ovals) => vals.cmp(ovals),
                    Self::Void |
                    Self::Bool(_) |
                    Self::Num(_) |
                    Self::Str(_) |
                    Self::Obj(_) |
                    Self::Fn(_) |
                    Self::List(_) |
                    Self::Tup(_) => Ordering::Greater,
                    _ => Ordering::Less,
                }
            },
            Self::Set(set) => {
                match other {
                    Self::Set(oset) => set.cmp(oset),
                    Self::Void |
                    Self::Bool(_) |
                    Self::Num(_) |
                    Self::Str(_) |
                    Self::Obj(_) |
                    Self::Fn(_) |
                    Self::List(_) |
                    Self::Tup(_) |
                    Self::Blob(_) => Ordering::Greater,
                    _ => Ordering::Less,
                }
            },
            Self::Map(map) => {
                match other {
                    Self::Map(omap) => map.cmp(omap),
                    Self::Void |
                    Self::Bool(_) |
                    Self::Num(_) |
                    Self::Str(_) |
                    Self::Obj(_) |
                    Self::Fn(_) |
                    Self::List(_) |
                    Self::Tup(_) |
                    Self::Blob(_) |
                    Self::Set(_) => Ordering::Greater,
                    _ => Ordering::Less,
                }
            },
            Self::Data(dref) => {
                match other {
                    Self::Data(oref) => dref.cmp(oref),
                    Self::Void |
                    Self::Bool(_) |
                    Self::Num(_) |
                    Self::Str(_) |
                    Self::Obj(_) |
                    Self::Fn(_) |
                    Self::List(_) |
                    Self::Tup(_) |
                    Self::Blob(_) |
                    Self::Set(_) |
                    Self::Map(_) => Ordering::Greater,
                    _ => Ordering::Less,
                }
            },
            Self::Ver(maj, min, pat, rel, bld) => {
                match other {
                    Self::Ver(omaj, omin, opat, orel, obld) => {
                        let mut cmp = maj.cmp(omaj);
                        if cmp == Ordering::Equal {
                            cmp = min.cmp(omin);
                            if cmp == Ordering::Equal {
                                cmp = pat.cmp(opat);
                                if cmp == Ordering::Equal {
                                    cmp = rel.cmp(orel);
                                    if cmp == Ordering::Equal {
                                        cmp = bld.cmp(obld);
                                    }
                                }
                            }
                        }
                        cmp
                    },
                    Self::Void |
                    Self::Bool(_) |
                    Self::Num(_) |
                    Self::Str(_) |
                    Self::Obj(_) |
                    Self::Fn(_) |
                    Self::List(_) |
                    Self::Tup(_) |
                    Self::Blob(_) |
                    Self::Set(_) |
                    Self::Map(_) |
                    Self::Data(_) => Ordering::Greater,
                    _ => Ordering::Less,
                }
            },
            Self::Promise(id, _) => {
                match other {
                    Self::Promise(pid, _) => id.cmp(pid),
                    Self::Void |
                    Self::Bool(_) |
                    Self::Num(_) |
                    Self::Str(_) |
                    Self::Obj(_) |
                    Self::Fn(_) |
                    Self::List(_) |
                    Self::Tup(_) |
                    Self::Blob(_) |
                    Self::Set(_) |
                    Self::Ver(..) |
                    Self::Map(_) => Ordering::Greater,
                    _ => Ordering::Less,
                }
            },
        }
    }
}
impl PartialEq for Val {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Void => {
                match other {
                    Self::Void => true,
                    _ => false,
                }
            },
            Self::Null => {
                match other {
                    Self::Null => true,
                    _ => false,
                }
            },
            Self::Bool(val) => {
                match other {
                    Self::Bool(oval) => val == oval,
                    _ => false,
                }
            },
            Self::Obj(nref) => {
                match other {
                    Self::Obj(oref) => nref == oref,
                    _ => false,
                }
            },
            Self::Blob(vals) => {
                match other {
                    Self::Blob(ovals) => vals == ovals,
                    _ => false
                }
            },
            Self::Data(dref) => {
                match other {
                    Self::Data(oref) => dref == oref,
                    _ => false,
                }
            },
            Self::Fn(dref) => {
                match other {
                    Self::Fn(oref) => dref == oref,
                    _ => false,
                }
            },
            Self::Num(val) => {
                match other {
                    Self::Num(oval) => val == oval,
                    _ => false,
                }
            },
            Self::Str(val) => {
                match other {
                    Self::Str(oval) => val == oval,
                    _ => false,
                }
            },
            Self::List(vals) => {
                match other {
                    Self::List(ovals) => vals == ovals,
                    _ => false,
                }
            },
            Self::Tup(vals) => {
                match other {
                    Self::Tup(ovals) => vals == ovals,
                    _ => false,
                }
            },
            Self::Set(set) => {
                match other {
                    Self::Set(oset) => set == oset,
                    _ => false,
                }
            },
            Self::Map(map) => {
                match other {
                    Self::Map(omap) => map == omap,
                    _ => false,
                }
            },
            Self::Ver(maj, min, pat, rel, bld) => {
                match other {
                    Self::Ver(omaj, omin, opat, orel, obld) => {
                        let mut cmp = maj.cmp(omaj);
                        if cmp == Ordering::Equal {
                            cmp = min.cmp(omin);
                            if cmp == Ordering::Equal {
                                cmp = pat.cmp(opat);
                                if cmp == Ordering::Equal {
                                    cmp = rel.cmp(orel);
                                    if cmp == Ordering::Equal {
                                        cmp = bld.cmp(obld);
                                    }
                                }
                            }
                        }
                        cmp.is_eq()
                    },
                    _ => false,
                }
            },
            Self::Promise(id, _) => {
                match other {
                    Self::Promise(pid, _) => id == pid,
                    _ => false,
                }
            },
        }
    }
}
impl Eq for Val {}

impl ToString for Val {
    fn to_string(&self) -> String {
        match self {
            Self::Null => "null".to_owned(),
            Self::Void => "void".to_owned(),
            Self::Str(val) => val.to_string(),
            Self::Bool(val) => val.to_string(),
            Self::Num(val) => val.to_string(),
            Self::List(val) => format!("{val:?}"),
            Self::Tup(val) => format!("tup({val:?})"),
            Self::Map(map) => format!("{map:?}"),
            Self::Set(set) => format!("{set:?}"),
            Self::Obj(nref) => nref.to_string(),
            Self::Fn(dref) => format!("fn({dref})"),
            Self::Data(dref) => format!("data({dref})"),
            Self::Promise(pid, _) => format!("promise({pid})"),
            Self::Blob(blob) => format!("blob({} bytes)", blob.len()),
            Self::Ver(maj, min, pat, rel, build) => {
                let mut major_str = format!("{maj}");
                if *maj < 0 { major_str = "*".to_string(); }
                let mut minor_str = format!("{min}");
                if *min < 0 { minor_str = "*".to_string(); }
                let mut patch_str = format!("{pat}");
                if *pat < 0 { patch_str = "*".to_string(); }

                let mut res = format!("{major_str}.{minor_str}.{patch_str}");
                if let Some(release) = rel {
                    res.push_str(&format!("-{release}"));
                }
                if let Some(build) = build {
                    res.push_str(&format!("+{build}"));
                }
                res
            }
        }
    }
}

impl Val {
    #[inline(always)]
    /// Is void value?
    pub fn void(&self) -> bool {
        match self {
            Self::Void => true,
            _ => false,
        }
    }

    #[inline(always)]
    /// Is null value?
    pub fn null(&self) -> bool {
        match self {
            Self::Null => true,
            _ => false,
        }
    }

    #[inline(always)]
    /// Is empty value (null or void)?
    pub fn empty(&self) -> bool {
        match self {
            Self::Null | Self::Void => true,
            _ => false,
        }
    }

    #[inline(always)]
    /// Is bool value?
    pub fn bool(&self) -> bool {
        match self {
            Self::Bool(_) => true,
            _ => false,
        }
    }

    #[inline(always)]
    /// Is number value?
    pub fn number(&self) -> bool {
        match self {
            Self::Num(_) => true,
            _ => false,
        }
    }

    #[inline]
    /// Is int value?
    pub fn is_int(&self) -> bool {
        match self {
            Self::Num(num) => {
                match num {
                    Num::Int(_) => true,
                    _ => false,
                }
            },
            _ => false,
        }
    }

    #[inline]
    /// Is float value?
    pub fn is_float(&self) -> bool {
        match self {
            Self::Num(num) => {
                match num {
                    Num::Float(_) | Num::Units(..) => true,
                    _ => false,
                }
            },
            _ => false,
        }
    }

    #[inline]
    /// Is units value (has units and they aren't undefined)?
    pub fn is_units(&self) -> bool {
        match self {
            Self::Num(num) => {
                match num {
                    Num::Units(_, u) => u.has_units() && !u.is_undefined(),
                    _ => false,
                }
            },
            _ => false,
        }
    }

    #[inline(always)]
    /// Is str value?
    pub fn str(&self) -> bool {
        match self {
            Self::Str(_) => true,
            _ => false,
        }
    }

    #[inline(always)]
    /// Is semver value?
    pub fn ver(&self) -> bool {
        match self {
            Self::Ver(..) => true,
            _ => false,
        }
    }

    #[inline(always)]
    /// Is obj value?
    pub fn obj(&self) -> bool {
        match self {
            Self::Obj(_) => true,
            _ => false,
        }
    }

    #[inline]
    /// Try extracting an obj value.
    pub fn try_obj(&self) -> Option<NodeRef> {
        match self {
            Self::Obj(nref) => Some(nref.clone()),
            _ => None,
        }
    }

    #[inline(always)]
    /// Is fn value?
    pub fn func(&self) -> bool {
        match self {
            Self::Fn(_) => true,
            _ => false,
        }
    }

    #[inline]
    /// Try extracting an func value.
    pub fn try_func(&self) -> Option<DataRef> {
        match self {
            Self::Fn(dref) => Some(dref.clone()),
            _ => None,
        }
    }

    #[inline(always)]
    /// Is data value?
    pub fn data(&self) -> bool {
        match self {
            Self::Data(_) => true,
            _ => false,
        }
    }

    #[inline]
    /// Try extracting an data value.
    pub fn try_data(&self) -> Option<DataRef> {
        match self {
            Self::Data(dref) => Some(dref.clone()),
            _ => None,
        }
    }

    #[inline]
    /// Try getting a data reference value.
    pub fn try_data_or_func(&self) -> Option<DataRef> {
        match self {
            Self::Data(dref) => Some(dref.clone()),
            Self::Fn(dref) => Some(dref.clone()),
            _ => None,
        }
    }

    #[inline(always)]
    /// Is blob value?
    pub fn blob(&self) -> bool {
        match self {
            Self::Blob(_) => true,
            _ => false,
        }
    }

    #[inline(always)]
    /// Is list value?
    pub fn list(&self) -> bool {
        match self {
            Self::List(_) => true,
            _ => false,
        }
    }

    #[inline(always)]
    /// Is tup value?
    pub fn tup(&self) -> bool {
        match self {
            Self::Tup(_) => true,
            _ => false,
        }
    }

    #[inline(always)]
    /// Is map value?
    pub fn map(&self) -> bool {
        match self {
            Self::Map(_) => true,
            _ => false,
        }
    }

    #[inline(always)]
    /// Is set value?
    pub fn set(&self) -> bool {
        match self {
            Self::Set(_) => true,
            _ => false,
        }
    }

    #[inline]
    /// Is this value equal to the data reference?
    /// Used for GC, etc.
    pub fn is_data_ref(&self, data: &DataRef) -> bool {
        match self {
            Self::Data(dref) => dref == data,
            Self::Fn(dref) => dref == data,
            _ => false,
        }
    }

    #[inline]
    /// Is this value a promise?
    pub fn promise(&self) -> bool {
        match self {
            Self::Promise(..) => true,
            _ => false,
        }
    }

    #[inline]
    /// Try extracting the promise value PID & type.
    pub fn try_promise(&self) -> Option<(SId, Type)> {
        match self {
            Self::Promise(pid, ty) => Some((pid.clone(), ty.clone())),
            _ => None,
        }
    }

    /// Value type?
    /// If true, this val should be cloned in variables instead of referenced.
    pub fn val_type(&self) -> bool {
        match self {
            Self::Bool(_) |
            Self::Str(_) |
            Self::Obj(_) |
            Self::Data(_) |
            Self::Fn(_) |
            Self::Null |
            Self::Void |
            Self::Num(_) |
            Self::Ver(..) |
            Self::Promise(..) => true,
            
            Self::Tup(_) |
            Self::Map(_) |
            Self::Set(_) |
            Self::Blob(_) |
            Self::List(_) => false,
        }
    }

    /// Try getting a number value.
    pub fn try_num(&mut self) -> Option<&mut Num> {
        match &mut *self {
            Self::Num(num) => {
                Some(num)
            },
            _ => None
        }
    }

    /// Deep copy this value.
    pub fn deep_copy(&self, graph: &mut Graph, context: Option<NodeRef>) -> Self {
        match self {
            Self::List(vals) => {
                let mut new_list = Vector::default();
                for val in vals {
                    new_list.push_back(ValRef::new(val.read().deep_copy(graph, context.clone())));
                }
                Self::List(new_list)
            },
            Self::Map(map) => {
                let mut new_map = OrdMap::default();
                for (k, v) in map {
                    new_map.insert(ValRef::new(k.read().deep_copy(graph, context.clone())), ValRef::new(v.read().deep_copy(graph, context.clone())));
                }
                Self::Map(new_map)
            },
            Self::Tup(tup) => {
                let mut new_tup = Vector::default();
                for val in tup {
                    new_tup.push_back(ValRef::new(val.read().deep_copy(graph, context.clone())));
                }
                Self::Tup(new_tup)
            },
            Self::Set(set) => {
                let mut new_set = OrdSet::default();
                for val in set {
                    new_set.insert(ValRef::new(val.read().deep_copy(graph, context.clone())));
                }
                Self::Set(new_set)
            },
            Self::Fn(dref) => {
                if let Some(context) = context {
                    let mut clone = None;
                    if let Some(data) = dref.data(&graph) {
                        let mut name = data.name.clone();
                        if data.nodes.contains(&context) {
                            // If inserting on the same node, make sure the names don't collide
                            name = SId::from(&format!("{}_{}", name.as_ref(), nanoid!(4)));
                        }

                        clone = Some(Data {
                            id: Default::default(),
                            name,
                            nodes: Default::default(),
                            data: data.data.clone(),
                            dirty: Default::default(),
                        });
                    }
                    if let Some(mut clone) = clone {
                        clone.data = clone.data.deep_copy(graph, Some(context.clone()));
                        if let Some(dref) = graph.insert_data(&context, clone) {
                            Self::Fn(dref)
                        } else {
                            Self::Null
                        }
                    } else {
                        Self::Null
                    }
                } else {
                    Self::Null
                }
            },
            Self::Data(dref) => {
                if let Some(context) = context {
                    let mut clone = None;
                    if let Some(data) = dref.data(&graph) {
                        let mut name = data.name.clone();
                        if data.nodes.contains(&context) {
                            // If inserting on the same node, make sure the names don't collide
                            name = SId::from(&format!("{}_{}", name.as_ref(), nanoid!(4)));
                        }

                        clone = Some(Data {
                            id: Default::default(),
                            name,
                            nodes: Default::default(),
                            data: data.data.clone(),
                            dirty: Default::default(),
                        });
                    }
                    if let Some(mut clone) = clone {
                        clone.data = clone.data.deep_copy(graph, Some(context.clone()));
                        if let Some(dref) = graph.insert_data(&context, clone) {
                            Self::Data(dref)
                        } else {
                            Self::Null
                        }
                    } else {
                        Self::Null
                    }
                } else {
                    Self::Null
                }
            },
            Self::Obj(nref) => {
                // deep copy an object
                let mut name = SId::default();
                let mut parent = context;
                let mut attributes = FxHashMap::default();
                let mut children = Vec::new();
                let mut data = Vec::new();
                
                if let Some(node) = nref.node(&graph) {
                    if parent.is_none() && node.parent.is_some() {
                        parent = node.parent.clone();
                    }
                    attributes = node.attributes.clone();
                    name = node.name.clone();
                    children = node.children.iter().cloned().collect();
                    for (_, v) in &node.data {
                        data.push(v.clone());
                    }
                }
                
                let node = Node {
                    id: Default::default(),
                    name,
                    parent: None,
                    children: Default::default(),
                    data: Default::default(),
                    attributes,
                    dirty: Default::default(),
                };
                let obj = graph.insert_stof_node(node, parent.clone());

                // Deep copy all children
                for child in children {
                    Self::Obj(child).deep_copy(graph, Some(obj.clone()));
                }

                // Deep copy all data and put back on obj
                for dref in data {
                    // Skip fields that are for child nodes
                    if let Some(parent) = &parent {
                        if let Some(field) = graph.get_stof_data::<Field>(&dref) {
                            if let Some(field_obj) = field.value.try_obj() {
                                if field_obj.child_of(graph, parent) {
                                    continue;
                                }
                            }
                        }
                    }

                    let mut clone = None;
                    if let Some(data) = dref.data(&graph) {
                        clone = Some(Data {
                            id: Default::default(),
                            name: data.name.clone(),
                            nodes: Default::default(),
                            data: data.data.clone(),
                            dirty: Default::default(),
                        });
                    }
                    if let Some(mut clone) = clone {
                        clone.data = clone.data.deep_copy(graph, Some(obj.clone()));
                        graph.insert_data(&obj, clone);
                    }
                }

                Self::Obj(obj)
            },
            _ => {
                self.clone()
            }
        }
    }


    /*****************************************************************************
     * Drop.
     *****************************************************************************/
    
    /// Drop data in this val (this is like a heap).
    /// Only remove if a primitive val allocated in the graph (node, field, func, etc..)
    pub fn drop_data(&self, graph: &mut Graph) {
        match self {
            Self::Obj(nref) => {
                graph.remove_node(nref, false);
            },
            Self::Data(dref) => {
                graph.remove_data(dref, None);
            },
            Self::Fn(dref) => {
                graph.remove_data(dref, None);
            },
            _ => {}
        }
    }


    /*****************************************************************************
     * Types.
     *****************************************************************************/
    
    /// Get the generic type for this value.
    pub fn gen_type(&self) -> Type {
        match self {
            Self::Void => Type::Void,
            Self::Null => Type::Null,
            Self::Num(num) => Type::Num(num.ntype()),
            Self::Str(_) => Type::Str,
            Self::Blob(_) => Type::Blob,
            Self::Data(_) => Type::Data(DATA),
            Self::Obj(_) => Type::Obj(SId::from(&OBJ)),
            Self::Fn(_) => Type::Fn,
            Self::Ver(..) => Type::Ver,
            Self::List(_) => Type::List,
            Self::Tup(vals) => {
                let mut types = vector![];
                for val in vals { types.push_back(val.read().gen_type()); }
                Type::Tup(types)
            },
            Self::Map(_) => Type::Map,
            Self::Set(_) => Type::Set,
            Self::Bool(_) => Type::Bool,
            Self::Promise(_, ty) => Type::Promise(Box::new(ty.clone())),
        }
    }

    /// Get the complex type for this value.
    /// This only applies to data and objects, otherwise it is the same as gen_type.
    pub fn spec_type(&self, graph: &Graph) -> Type {
        match self {
            Self::Data(dref) => {
                if !dref.core_data(graph) { // non-core data are custom complex data types defined outside of this crate
                    if let Some(tagname) = dref.tagname(graph) {
                        return Type::Data(tagname.into());
                    }
                }
                Type::Data(DATA)
            },
            Self::Obj(nref) => {
                let mut prototypes = Prototype::prototype_nodes(graph, nref, false);
                if prototypes.len() == 1 {
                    return Type::Obj(prototypes.pop().unwrap());
                } else if prototypes.len() > 1 {
                    let mut types = vector![];
                    for ty in prototypes { types.push_back(Type::Obj(ty)); }
                    return Type::Union(types);
                }
                Type::Obj(SId::from(&OBJ))
            },
            _ => self.gen_type()
        }
    }

    #[inline]
    /// Is this value of a type?
    pub fn is_type(&self, target: &Type, graph: &Graph) -> bool {
        &self.gen_type() == target || &self.spec_type(graph) == target
    }

    /// Is this value an instance of a prototype?
    pub fn instance_of(&self, other: &NodeRef, graph: &Graph) -> Result<bool, Error> {
        if let Some(obj) = self.try_obj() {
            let proto_nrefs = Prototype::prototype_nodes(graph, &obj, false);
            for nref in &proto_nrefs { if nref == other { return Ok(true); } }
            for nref in proto_nrefs {
                if let Ok(contains) = Self::Obj(nref).instance_of(other, graph) {
                    if contains {
                        return Ok(true);
                    }
                }
            }
            return Ok(false);
        }
        Err(Error::CastVal)
    }

    /// Cast this value to a different type.
    pub fn cast(&mut self, target: &Type, graph: &mut Graph, context: Option<NodeRef>) -> Result<(), Error> {
        // Cast object must come before type check here due to "obj" type
        if self.obj() {
            return self.cast_object(target, graph, context);
        }
        if self.is_type(target, &graph) {
            return Ok(());
        }
        match target {
            Type::Union(types) => {
                for ty in types {
                    match self.cast(ty, graph, context.clone()) {
                        Ok(_) => return Ok(()),
                        Err(_) => {}
                    }
                }
                return Err(Error::CastVal);
            },
            _ => {}
        }
        match self {
            Self::Blob(blob) => {
                match target {
                    Type::List => {
                        *self = Self::List(blob.iter().map(|byte| ValRef::new(Self::Num(Num::Int((*byte) as i64)))).collect());
                        Ok(())
                    },
                    Type::Str => {
                        match str::from_utf8(blob.as_slice()) {
                            Ok(val) => {
                                *self = Self::Str(val.into());
                                Ok(())
                            },
                            Err(_) => Err(Error::NotImplemented)
                        }
                    },
                    _ => Err(Error::NotImplemented)
                }
            },
            Self::List(values) => {
                match target {
                    Type::Set => {
                        let mut set = OrdSet::new();
                        while !values.is_empty() { set.insert(values.pop_front().unwrap()); }
                        *self = Self::Set(set);
                        Ok(())
                    },
                    Type::Blob => {
                        let mut blob: Vec<u8> = Vec::new();
                        while !values.is_empty() {
                            if let Some(val) = values.pop_front() {
                                match val.read().deref() {
                                    Self::Num(num) => {
                                        let res: Result<u8, _> = num.int().try_into();
                                        if res.is_err() { return Err(Error::CastVal); }
                                        blob.push(res.unwrap());
                                    },
                                    _ => return Err(Error::CastVal)
                                }
                            }
                        }
                        *self = Self::Blob(blob);
                        Ok(())
                    },
                    Type::Tup(types) => {
                        let tup = Self::Tup(values.clone());
                        if tup.gen_type() == Type::Tup(types.clone()) {
                            *self = tup;
                            return Ok(());
                        }
                        if values.len() == types.len() {
                            for i in 0..values.len() {
                                values[i].write().cast(&types[i], graph, context.clone())?;
                            }
                            *self = Self::Tup(values.clone());
                            return Ok(());
                        }
                        Err(Error::CastVal)
                    },
                    _ => Err(Error::NotImplemented)
                }
            },
            Self::Bool(val) => {
                match target {
                    Type::Num(num) => {
                        let v = *val as i64;
                        match num {
                            NumT::Int => *self = Self::Num(Num::Int(v)),
                            NumT::Float => *self = Self::Num(Num::Float(v as f64)),
                            NumT::Units(units) => *self = Self::Num(Num::Units(v as f64, *units)),
                        }
                        Ok(())
                    },
                    Type::Str => {
                        *self = Self::Str(ArcStr::from(val.to_string()));
                        Ok(())
                    },
                    _ => Err(Error::NotImplemented)
                }
            },
            Self::Fn(dref) => {
                match target {
                    Type::Data(inner) => {
                        if inner != &DATA && inner != &literal!("Fn") {
                            Err(Error::NotImplemented)
                        } else {
                            *self = Self::Data(dref.clone());
                            Ok(())
                        }
                    },
                    _ => Err(Error::NotImplemented)
                }
            },
            Self::Data(dref) => {
                match target {
                    Type::Data(tagname) => {
                        if tagname == &DATA {
                            Ok(())
                        } else if let Some(dtn) = dref.tagname(&graph) {
                            if tagname == &dtn {
                                Ok(())
                            } else {
                                Err(Error::CastVal)
                            }
                        } else {
                            Err(Error::CastVal)
                        }
                    },
                    Type::Fn => {
                        if dref.type_of::<Func>(graph) {
                            *self = Self::Fn(dref.clone());
                            Ok(())
                        } else {
                            Err(Error::CastVal)
                        }
                    },
                    _ => Err(Error::NotImplemented)
                }
            },
            Self::Null => Ok(()),
            Self::Num(num) => {
                match target {
                    Type::Bool => {
                        *self = Self::Bool(num.truthy());
                        Ok(())
                    },
                    Type::Str => {
                        *self = Self::Str(num.print().into());
                        Ok(())
                    },
                    Type::Num(numt) => {
                        *self = Self::Num(num.cast(*numt));
                        Ok(())
                    },
                    _ => Err(Error::NotImplemented)
                }
            },
            Self::Str(val) => {
                match target {
                    Type::List => {
                        let mut list = Vector::new();
                        for char in val.as_str().chars() {
                            list.push_back(ValRef::new(Self::Str(char.to_string().into())));
                        }
                        *self = Self::List(list);
                        Ok(())
                    },
                    Type::Set => {
                        let mut set = OrdSet::new();
                        for char in val.as_str().chars() {
                            set.insert(ValRef::new(Self::Str(char.to_string().into())));
                        }
                        *self = Self::Set(set);
                        Ok(())
                    },
                    Type::Blob => {
                        *self = Self::Blob(str::as_bytes(&val).to_vec());
                        Ok(())
                    },
                    Type::Bool => {
                        *self = Self::Bool(val.len() > 0);
                        Ok(())
                    },
                    Type::Ver => {
                        if let Some(ver) = parse_semver_alone(&val) {
                            *self = ver;
                            Ok(())
                        } else {
                            Err(Error::CastVal)
                        }
                    },
                    Type::Num(_) => {
                        match number(&val) {
                            Ok((_, mut res)) => {
                                res.cast(target, graph, context)?; // get the number into the right type, no matter the string
                                *self = res;
                                Ok(())
                            },
                            Err(_) => Err(Error::CastVal)
                        }
                    },
                    _ => Err(Error::NotImplemented)
                }
            },
            Self::Ver(..) => {
                match target {
                    Type::Str => {
                        *self = Self::Str(self.to_string().into());
                        Ok(())
                    },
                    Type::Ver => Ok(()),
                    _ => Err(Error::NotImplemented)
                }
            },
            Self::Tup(values) => {
                match target {
                    Type::Tup(types) => {
                        if values.len() == types.len() {
                            for i in 0..values.len() {
                                values[i].write().cast(&types[i], graph, context.clone())?;
                            }
                            return Ok(());
                        }
                        Err(Error::CastVal)
                    },
                    _ => Err(Error::NotImplemented)
                }
            },
            Self::Set(set) => {
                match target {
                    Type::List => {
                        let mut list = Vector::new();
                        for val in set.iter() {
                            list.push_back(val.clone());
                        }
                        *self = Self::List(list);
                        Ok(())
                    },
                    _ => Err(Error::NotImplemented)
                }
            },
            Self::Promise(_, ptype) => {
                match target {
                    Type::Promise(optype) => {
                        *ptype = *optype.clone();
                    },
                    _ => {
                        *ptype = target.clone();
                    }
                }
                Ok(())
            },
            _ => Err(Error::NotImplemented)
        }
    }
    fn cast_object(&self, target: &Type, graph: &mut Graph, context: Option<NodeRef>) -> Result<(), Error> {
        // Get the object we are casting
        let obj = self.try_obj().unwrap(); // will err if not done right

        // Get the target type in terms of prototypes
        let mut target = target.clone();
        target.obj_to_proto(graph, context.clone());

        match target {
            Type::Union(types) => {
                let mut objects = Vec::new();
                for cast_type in types {
                    match cast_type {
                        Type::Obj(proto_id) => {
                            if self.instance_of(&proto_id, graph)? {
                                return Ok(());
                            } else {
                                objects.push(Type::Obj(proto_id));
                            }
                        },
                        _ => {}
                    }
                }
                for object in objects {
                    let res = self.cast_object(&object, graph, context.clone());
                    if res.is_ok() {
                        return Ok(());
                    }
                }
                Err(Error::CastVal)
            },
            Type::Obj(proto_id) => {
                if proto_id == SId::from(&OBJ) {
                    return Ok(()); // no need to cast to general type as it always works
                }
                if !proto_id.node_exists(graph) {
                    return Err(Error::ObjectCastProtoDne);
                }
                if !self.instance_of(&proto_id, graph)? {
                    // Remove all current prototypes from the object and add the new one
                    let existing_prototypes = Prototype::prototype_refs(graph, &obj);
                    for dref in existing_prototypes { graph.remove_data(&dref, Some(obj.clone())); }
                    graph.insert_stof_data(&obj, "__proto__", Box::new(Prototype { node: proto_id.clone() }), None);
                
                    // Perform field initializations and checks for the type
                    let obj_fields = Field::fields(graph, &obj);
                    'proto_loop: for (fname, fref) in Field::fields(graph, &proto_id) {
                        let mut field_value = None;
                        let mut field_attributes = None;
                        if let Some(field) = graph.get_stof_data::<Field>(&fref) {
                            if field.attributes.contains_key("type_ignore") {
                                continue 'proto_loop;
                            }
                            field_value = Some(field.value.clone()); // shallow
                            field_attributes = Some(field.attributes.clone());
                        }
                        if let Some(field_value) = field_value {
                            if let Some(existing) = obj_fields.get(&fname) {
                                let mut existing_value = None;
                                if let Some(field) = graph.get_stof_data::<Field>(existing) {
                                    existing_value = Some(field.value.clone());
                                }
                                if let Some(existing_value) = existing_value {
                                    let field_type = field_value.spec_type(graph);
                                    if existing_value.spec_type(graph) != field_type {
                                        existing_value.cast(&field_type, graph, context.clone())?;
                                    } else if let Some(field_obj) = field_value.try_obj() {
                                        if let Some(existing_obj) = existing_value.try_obj() {
                                            let target = Type::Obj(field_obj);
                                            Self::Obj(existing_obj).cast(&target, graph, context.clone())?;
                                        }
                                    }
                                    if let Some(field) = graph.get_mut_stof_data::<Field>(existing) {
                                        field.value = existing_value;
                                        if let Some(attrs) = field_attributes {
                                            for (k, v) in attrs {
                                                if !field.attributes.contains_key(&k) {
                                                    field.attributes.insert(k, v);
                                                }
                                            }
                                        }
                                    }
                                }
                            } else {
                                let copied = field_value.deep_copy(graph, context.clone());
                                
                                // if copied is an object, cast it too & move it to the new object
                                if let Some(copy) = copied.try_obj() {
                                    graph.move_node(&copy, &obj);

                                    let target = Type::Obj(field_value.try_obj().unwrap());
                                    Self::Obj(copy).cast(&target, graph, context.clone())?;
                                }

                                let field = Field::new(copied, field_attributes);
                                graph.insert_stof_data(&obj, &fname, Box::new(field), None);
                            }
                        }
                    }
                }
                Ok(())
            },
            _ => Err(Error::CastVal)
        }
    }

    /// Library name for this value.
    pub fn lib_name(&self, graph: &Graph) -> ArcStr {
        if let Some(dref) = self.try_data() {
            if let Some(data) = dref.data(graph) {
                if !data.core_data() {
                    // TODO: check that the lib exists
                    return data.tagname().into();
                }
            }
        }
        self.spec_type(graph).gen_lib_name()
    }


    /*****************************************************************************
     * Boolean.
     *****************************************************************************/
    
    /// Is this value truthy?
    pub fn truthy(&self) -> bool {
        match self {
            Self::Void |
            Self::Null => false,
            Self::Str(v) => v.len() > 0,
            Self::Bool(v) => *v,
            Self::Num(v) => v.truthy(),
            _ => true,
        }
    }

    /// Merge this value with another.
    /// Collision of values.
    pub fn collide(&mut self, other: &Self) {
        match self {
            Self::Void |
            Self::Null => *self = other.clone(),
            Self::Tup(_) |
            Self::Obj(_) |
            Self::Num(_) |
            Self::Blob(_) |
            Self::Fn(_) |
            Self::Data(_) |
            Self::Str(_) |
            Self::Ver(..) |
            Self::Promise(..) |
            Self::Bool(_) => {
                if self != other {
                    match other {
                        Self::List(vals) => {
                            let mut v = vals.clone();
                            v.push_front(ValRef::new(self.clone()));
                            *self = Val::List(v);
                        },
                        Self::Set(set) => {
                            let mut v = set.clone();
                            v.insert(ValRef::new(self.clone()));
                            *self = Val::Set(v);
                        },
                        _ => {
                            *self = Val::List(vector![ValRef::new(self.clone()), ValRef::new(other.clone())]);
                        }
                    }
                }
            },
            Val::List(vals) => {
                match other {
                    Val::List(ovals) => {
                        vals.append(ovals.clone());
                    },
                    _ => {
                        vals.push_back(ValRef::new(other.clone()));
                    }
                }
            },
            Val::Set(set) => {
                match other {
                    Val::Set(oset) => {
                        *set = set.clone().union(oset.clone());
                    },
                    _ => {
                        set.insert(ValRef::new(other.clone()));
                    }
                }
            },
            Val::Map(map) => {
                match other {
                    Val::Map(omap) => {
                        for (k, v) in omap {
                            if let Some(existing_val) = map.get_mut(k) {
                                existing_val.write().collide(&v.read());
                            } else {
                                map.insert(k.clone(), v.clone());
                            }
                        }
                    },
                    _ => {
                        *self = Val::List(vector![ValRef::new(self.clone()), ValRef::new(other.clone())]);
                    }
                }
            },
        }
    }

    /// Runtime equality?
    pub fn equal(&self, other: &Self) -> Result<Self, Error> {
        // SemVer equality: SemVer == SemVer || SemVer == String (does not parse String to SemVer for performance reasons)
        match self {
            Self::Ver(major, minor, patch, release, build ) => {
                match other {
                    Self::Str(semver) => {
                        if let Some(semver) = parse_semver_alone(semver) {
                            match semver {
                                Self::Ver(omajor, ominor, opatch, orelease, obuild ) => {
                                    let mut cmp = major.cmp(&omajor);
                                    if *major < 0 || omajor < 0 || cmp == Ordering::Equal {
                                        cmp = minor.cmp(&ominor);
                                        if *minor < 0 || ominor < 0 || cmp == Ordering::Equal {
                                            cmp = patch.cmp(&opatch);
                                            if *patch < 0 || opatch < 0 || cmp == Ordering::Equal {
                                                cmp = release.cmp(&orelease);
                                                if cmp == Ordering::Equal {
                                                    cmp = build.cmp(&obuild);
                                                }
                                            }
                                        }
                                    }
                                    return Ok(cmp.is_eq().into());
                                },
                                _ => {}
                            }
                        }
                        return Ok(Val::Bool(false));
                    },
                    Self::Ver(omajor, ominor, opatch, orelease, obuild ) => {
                        let mut cmp = major.cmp(omajor);
                        if *major < 0 || *omajor < 0 || cmp == Ordering::Equal {
                            cmp = minor.cmp(ominor);
                            if *minor < 0 || *ominor < 0 || cmp == Ordering::Equal {
                                cmp = patch.cmp(opatch);
                                if *patch < 0 || *opatch < 0 || cmp == Ordering::Equal {
                                    cmp = release.cmp(orelease);
                                    if cmp == Ordering::Equal {
                                        cmp = build.cmp(obuild);
                                    }
                                }
                            }
                        }
                        return Ok(cmp.is_eq().into());
                    },
                    _ => {}
                }
            },
            Self::Fn(dref) =>{
                match other {
                    Self::Data(odref) => {
                        return Ok((dref == odref).into());
                    },
                    _ => {}
                }
            },
            Self::Data(dref) => {
                match other {
                    Self::Fn(odref) => {
                        return Ok((dref == odref).into());
                    },
                    _ => {}
                }
            },
            _ => {}
        }
        Ok((self == other).into())
    }

    /// Runtime not equals?
    pub fn not_equal(&self, other: &Self) -> Result<Self, Error> {
        let eq = self.equal(other)?;
        Ok((!eq.truthy()).into())
    }

    /// Runtime greater than another value?
    pub fn gt(&self, other: &Self, graph: &Graph) -> Result<Self, Error> {
        match self {
            Self::List(_) |
            Self::Tup(_) |
            Self::Data(_) |
            Self::Fn(_) |
            Self::Promise(..) |
            Self::Void |
            Self::Null => Ok(false.into()),
            Self::Blob(blob) => {
                match other {
                    Self::Blob(other_blob) => Ok(Self::Bool(blob.len() > other_blob.len())),
                    _ => Ok(Self::Bool(false))
                }
            },
            Self::Obj(nref) => {
                match other {
                    Self::Obj(onref) => {
                        // nref is a parent of onref and not the same node?
                        Ok((onref.child_of(graph, nref) && onref != nref).into())
                    },
                    _ => Ok(false.into())
                }
            },
            Self::Bool(v) => {
                Self::Num(Num::Int(*v as i64)).gt(other, graph)
            },
            Self::Num(val) => {
                match other {
                    Self::Num(oval) => {
                        Ok(Self::Bool(val.gt(oval)))
                    },
                    Self::Bool(ov) => {
                        Ok(Self::Bool(val.int() > *ov as i64))
                    },
                    _ => Ok(Self::Bool(false))
                }
            },
            Self::Str(val) => {
                match other {
                    Self::Str(oval) => Ok(Self::Bool(val > oval)),
                    _ => Ok(Self::Bool(false)),
                }
            },
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
            Self::Ver(major, minor, patch, release, build ) => {
                match other {
                    Self::Str(semver) => {
                        if let Some(semver) = parse_semver_alone(semver) {
                            match &semver {
                                Self::Ver(omajor, ominor, opatch, orelease, obuild ) => {
                                    let mut cmp = major.cmp(omajor);
                                    if *major < 0 || *omajor < 0 || cmp == Ordering::Equal {
                                        cmp = minor.cmp(ominor);
                                        if *minor < 0 || *ominor < 0 || cmp == Ordering::Equal {
                                            cmp = patch.cmp(opatch);
                                            if *patch < 0 || *opatch < 0 || cmp == Ordering::Equal {
                                                cmp = release.cmp(orelease);
                                                if cmp == Ordering::Equal {
                                                    cmp = build.cmp(obuild);
                                                }
                                            }
                                        }
                                    }
                                    return Ok(cmp.is_gt().into());
                                },
                                _ => {}
                            }
                        }
                        Ok(Self::Bool(false))
                    },
                    Self::Ver(omajor, ominor, opatch, orelease, obuild ) => {
                        let mut cmp = major.cmp(omajor);
                        if *major < 0 || *omajor < 0 || cmp == Ordering::Equal {
                            cmp = minor.cmp(ominor);
                            if *minor < 0 || *ominor < 0 || cmp == Ordering::Equal {
                                cmp = patch.cmp(opatch);
                                if *patch < 0 || *opatch < 0 || cmp == Ordering::Equal {
                                    cmp = release.cmp(orelease);
                                    if cmp == Ordering::Equal {
                                        cmp = build.cmp(obuild);
                                    }
                                }
                            }
                        }
                        Ok(cmp.is_gt().into())
                    },
                    _ => Ok(Self::Bool(false)),
                }
            },
        }
    }

    /// Runtime less than another value?
    pub fn lt(&self, other: &Self, graph: &Graph) -> Result<Self, Error> {
        match self {
            Self::List(_) |
            Self::Tup(_) |
            Self::Data(_) |
            Self::Fn(_) |
            Self::Promise(..) |
            Self::Void |
            Self::Null => Ok(true.into()),
            Self::Blob(blob) => {
                match other {
                    Self::Blob(other_blob) => Ok(Self::Bool(blob.len() < other_blob.len())),
                    _ => Ok(Self::Bool(false))
                }
            },
            Self::Obj(onref) => {
                match other {
                    Self::Obj(nref) => {
                        // nref is a parent of onref and not the same node?
                        Ok((onref.child_of(graph, nref) && onref != nref).into())
                    },
                    _ => Ok(false.into())
                }
            },
            Self::Bool(v) => {
                Self::Num(Num::Int(*v as i64)).lt(other, graph)
            },
            Self::Num(val) => {
                match other {
                    Self::Num(oval) => {
                        Ok(Self::Bool(val.lt(oval)))
                    },
                    Self::Bool(ov) => {
                        Ok(Self::Bool(val.int() < *ov as i64))
                    },
                    _ => Ok(Self::Bool(false))
                }
            },
            Self::Str(val) => {
                match other {
                    Self::Str(oval) => Ok(Self::Bool(val < oval)),
                    _ => Ok(Self::Bool(false)),
                }
            },
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
            Self::Ver(major, minor, patch, release, build ) => {
                match other {
                    Self::Str(semver) => {
                        if let Some(semver) = parse_semver_alone(semver) {
                            match &semver {
                                Self::Ver(omajor, ominor, opatch, orelease, obuild ) => {
                                    let mut cmp = major.cmp(omajor);
                                    if *major < 0 || *omajor < 0 || cmp == Ordering::Equal {
                                        cmp = minor.cmp(ominor);
                                        if *minor < 0 || *ominor < 0 || cmp == Ordering::Equal {
                                            cmp = patch.cmp(opatch);
                                            if *patch < 0 || *opatch < 0 || cmp == Ordering::Equal {
                                                cmp = release.cmp(orelease);
                                                if cmp == Ordering::Equal {
                                                    cmp = build.cmp(obuild);
                                                }
                                            }
                                        }
                                    }
                                    return Ok(cmp.is_lt().into());
                                },
                                _ => {}
                            }
                        }
                        Ok(Self::Bool(false))
                    },
                    Self::Ver(omajor, ominor, opatch, orelease, obuild ) => {
                        let mut cmp = major.cmp(omajor);
                        if *major < 0 || *omajor < 0 || cmp == Ordering::Equal {
                            cmp = minor.cmp(ominor);
                            if *minor < 0 || *ominor < 0 || cmp == Ordering::Equal {
                                cmp = patch.cmp(opatch);
                                if *patch < 0 || *opatch < 0 || cmp == Ordering::Equal {
                                    cmp = release.cmp(orelease);
                                    if cmp == Ordering::Equal {
                                        cmp = build.cmp(obuild);
                                    }
                                }
                            }
                        }
                        Ok(cmp.is_lt().into())
                    },
                    _ => Ok(Self::Bool(false)),
                }
            },
        }
    }

    /// Runtime greater or equal?
    pub fn gte(&self, other: &Self, graph: &Graph) -> Result<Self, Error> {
        let res = self.gt(other, graph)?;
        if res.truthy() { return Ok(res); }
        self.equal(other)
    }

    /// Runtime less than or equal?
    pub fn lte(&self, other: &Self, graph: &Graph) -> Result<Self, Error> {
        let res = self.lt(other, graph)?;
        if res.truthy() { return Ok(res); }
        self.equal(other)
    }


    /*****************************************************************************
     * Ops.
     *****************************************************************************/
    
    /// Add a value to this value.
    pub fn add(&mut self, other: Self, graph: &mut Graph) -> Result<(), Error> {
        if other.empty() { return Ok(()); }
        match &mut *self {
            Self::Null |
            Self::Void => {
                *self = other;
                Ok(())
            },
            Self::Blob(ref mut blob) => {
                match other {
                    Self::Blob(mut other) => {
                        blob.append(&mut other);
                        Ok(())
                    },
                    Self::Str(other) => {
                        *self = Self::Str(format!("{}{other}", self.print(&graph)).into());
                        Ok(())
                    },
                    _ => Err(Error::NotImplemented)
                }
            },
            Self::Bool(val) => {
                match other {
                    Self::Bool(other) => {
                        *self = Self::Bool(*val && other);
                        Ok(())
                    },
                    Self::Num(num) => {
                        let res;
                        if *val {
                            res = num.add(&Num::Int(1));
                        } else {
                            res = num;
                        }
                        *self = Self::Num(res);
                        Ok(())
                    },
                    Self::Str(other) => {
                        *self = Self::Str(format!("{}{}", val, other).into());
                        Ok(())
                    },
                    _ => Err(Error::NotImplemented)
                }
            },
            Self::Str(val) => {
                match other {
                    Self::Str(other) => {
                        *self = Self::Str(format!("{val}{other}").into());
                        Ok(())
                    },
                    _ => {
                        *self = Self::Str(format!("{val}{}", other.print(&graph)).into());
                        Ok(())
                    }
                }
            },
            Self::Num(val) => {
                match other {
                    Self::Num(other) => {
                        *val = val.add(&other);
                        Ok(())
                    },
                    Self::Str(other) => {
                        match number(&other) {
                            Ok((_, res)) => {
                                self.add(res, graph)?;
                                Ok(())
                            },
                            Err(_) => {
                                *self = Self::Str(format!("{}{other}", self.print(&graph)).into());
                                Ok(())
                            }
                        }
                    },
                    Self::Bool(other) => {
                        if other {
                            *val = val.add(&Num::Int(1));
                        }
                        Ok(())
                    },
                    _ => Err(Error::NotImplemented)
                }
            },
            Self::List(ref mut vals) => {
                match other {
                    Self::List(other) => {
                        vals.append(other);
                        Ok(())
                    },
                    Self::Tup(other) => {
                        vals.append(other);
                        Ok(())
                    },
                    Self::Set(set) => {
                        for val in set {
                            vals.push_back(val);
                        }
                        Ok(())
                    },
                    _ => {
                        vals.push_back(ValRef::new(other));
                        Ok(())
                    }
                }
            },
            Self::Map(ref mut map) => {
                match other {
                    Self::Map(other) => {
                        for (k, v) in other {
                            map.insert(k, v);
                        }
                        Ok(())
                    },
                    _ => Err(Error::NotImplemented)
                }
            },
            Self::Set(ref mut set) => {
                match other {
                    Self::Set(other) => {
                        for val in other { set.insert(val); }
                        Ok(())
                    },
                    Self::List(other) => {
                        for val in other { set.insert(val); }
                        Ok(())
                    },
                    _ => {
                        set.insert(ValRef::new(other));
                        Ok(())
                    }
                }
            },
            _ => Err(Error::NotImplemented)
        }
    }

    /// Subtract value from this value.
    pub fn sub(&mut self, other: Self, graph: &mut Graph) -> Result<(), Error> {
        if other.empty() { return Ok(()); }
        match &mut *self {
            Self::Null |
            Self::Void => {
                *self = other;
                Ok(())
            },
            Self::Bool(val) => {
                match other {
                    Self::Bool(other) => {
                        *val = *val ^ other;
                        Ok(())
                    },
                    Self::Num(num) => {
                        let res;
                        if *val {
                            res = Num::Int(1).sub(&num);
                        } else {
                            res = num.mul(&Num::Int(-1));
                        }
                        *self = Self::Num(res);
                        Ok(())
                    },
                    _ => Err(Error::NotImplemented)
                }
            },
            Self::Str(val) => {
                match other {
                    Self::Str(other) => {
                        *self = Self::Str(val.replace(other.as_str(), "").into());
                        Ok(())
                    },
                    _ => {
                        *self = Self::Str(val.replace(&other.print(&graph), "").into());
                        Ok(())
                    }
                }
            },
            Self::Num(val) => {
                match other {
                    Self::Num(other) => {
                        *self = Self::Num(val.sub(&other));
                        Ok(())
                    },
                    Self::Str(other) => {
                        match number(&other) {
                            Ok((_, res)) => {
                                self.sub(res, graph)?;
                                Ok(())
                            },
                            Err(_) => Err(Error::NotImplemented)
                        }
                    },
                    Self::Bool(other) => {
                        if other {
                            *val = val.sub(&Num::Int(1));
                        }
                        Ok(())
                    },
                    _ => Err(Error::NotImplemented)
                }
            },
            Self::Map(ref mut map) => {
                match other {
                    Self::Map(other) => {
                        for (k, _v) in other {
                            map.remove(&k);
                        }
                        Ok(())
                    },
                    _ => Err(Error::NotImplemented)
                }
            },
            Self::Set(ref mut set) => {
                match other {
                    Self::Set(other) => {
                        for val in other { set.remove(&val); }
                        Ok(())
                    },
                    Self::List(other) => {
                        for val in other { set.remove(&val); }
                        Ok(())
                    },
                    _ => {
                        set.remove(&ValRef::new(other));
                        Ok(())
                    }
                }
            },
            _ => Err(Error::NotImplemented)
        }
    }

    /// Multiply this value with another.
    pub fn mul(&mut self, other: Self, graph: &mut Graph) -> Result<(), Error> {
        if other.empty() { return Ok(()); }
        match self {
            Self::Null |
            Self::Void => {
                *self = other;
                Ok(())
            },
            Self::Bool(val) => {
                match other {
                    Self::Bool(other) => {
                        *val = *val || other;
                        Ok(())
                    },
                    _ => Err(Error::NotImplemented)
                }
            },
            Self::Str(val) => {
                match other {
                    Self::Str(other) => {
                        *val = format!("{val}{other}").into();
                        Ok(())
                    },
                    Self::Num(num) => {
                        let mut other = String::default();
                        for _ in 0..num.int() {
                            other.push_str(&val);
                        }
                        *val = other.into();
                        Ok(())
                    },
                    _ => {
                        *val = format!("{val}{}", other.print(&graph)).into();
                        Ok(())
                    }
                }
            },
            Self::Num(val) => {
                match other {
                    Self::Num(other) => {
                        *val = val.mul(&other);
                        Ok(())
                    },
                    Self::Str(other) => {
                        match number(&other) {
                            Ok((_, res)) => {
                                self.mul(res, graph)?;
                                Ok(())
                            },
                            Err(_) => Err(Error::NotImplemented)
                        }
                    },
                    Self::Bool(other) => {
                        if other {
                            *val = val.mul(&Num::Int(1));
                        } else {
                            *val = val.mul(&Num::Int(0)); // keep units, etc.
                        }
                        Ok(())
                    },
                    _ => Err(Error::NotImplemented)
                }
            },
            Self::List(vals) => {
                match other {
                    Self::List(other) => {
                        let mut s = OrdSet::new();
                        for val in vals { s.insert(val.clone()); }
                        let mut os = OrdSet::new();
                        for val in other { os.insert(val); }
                        
                        *self = Self::Set(s.intersection(os));
                        Ok(())
                    },
                    Self::Set(set) => {
                        let mut os = OrdSet::new();
                        for val in vals { os.insert(val.clone()); }
                        *self = Self::Set(os.intersection(set));
                        Ok(())
                    },
                    _ => Err(Error::NotImplemented)
                }
            },
            Self::Map(map) => {
                match other {
                    Self::Map(other) => {
                        *self = Self::Map(map.clone().intersection(other));
                        Ok(())
                    },
                    _ => Err(Error::NotImplemented)
                }
            },
            Self::Set(set) => {
                match other {
                    Self::Set(other) => {
                        *self = Self::Set(set.clone().intersection(other));
                        Ok(())
                    },
                    Self::List(other) => {
                        let mut os = OrdSet::new();
                        for val in other { os.insert(val); }
                        *self = Self::Set(set.clone().intersection(os));
                        Ok(())
                    },
                    _ => Err(Error::NotImplemented)
                }
            },
            _ => Err(Error::NotImplemented)
        }
    }

    /// Divide two values.
    pub fn div(&mut self, other: Self, graph: &mut Graph) -> Result<(), Error> {
        if other.empty() { return Ok(()); }
        match self {
            Self::Null |
            Self::Void => {
                *self = other;
                Ok(())
            },
            Self::Bool(val) => {
                match other {
                    Self::Bool(other) => {
                        *val = *val && other;
                        Ok(())
                    },
                    _ => Err(Error::NotImplemented)
                }
            },
            Self::Str(val) => {
                match other {
                    Self::Str(other) => {
                        let vec = val.split(other.as_str()).collect::<Vec<&str>>();
                        let mut new = Vector::new();
                        for v in vec {
                            new.push_back(ValRef::new(v.into()));
                        }
                        *self = Self::List(new);
                        Ok(())
                    },
                    _ => {
                        *val = format!("{val}{}", other.print(&graph)).into();
                        Ok(())
                    }
                }
            },
            Self::Num(val) => {
                match other {
                    Self::Num(other) => {
                        *val = val.div(&other);
                        Ok(())
                    },
                    Self::Str(other) => {
                        match number(&other) {
                            Ok((_, res)) => {
                                self.div(res, graph)?;
                                Ok(())
                            },
                            Err(_) => Err(Error::NotImplemented)
                        }
                    },
                    _ => Err(Error::NotImplemented)
                }
            },
            Self::List(vals) => {
                match other {
                    Self::List(other) => {
                        let mut s = OrdSet::new();
                        for val in vals { s.insert(val.clone()); }
                        let mut os = OrdSet::new();
                        for val in other { os.insert(val); }
                        
                        *self = Self::Set(s.union(os));
                        Ok(())
                    },
                    Self::Set(set) => {
                        let mut os = OrdSet::new();
                        for val in vals { os.insert(val.clone()); }
                        *self = Self::Set(os.union(set));
                        Ok(())
                    },
                    _ => Err(Error::NotImplemented)
                }
            },
            Self::Map(map) => {
                match other {
                    Self::Map(other) => {
                        *self = Self::Map(map.clone().union(other));
                        Ok(())
                    },
                    _ => Err(Error::NotImplemented)
                }
            },
            Self::Set(set) => {
                match other {
                    Self::Set(other) => {
                        *self = Self::Set(set.clone().union(other));
                        Ok(())
                    },
                    Self::List(other) => {
                        let mut os = OrdSet::new();
                        for val in other { os.insert(val); }
                        *self = Self::Set(set.clone().union(os));
                        Ok(())
                    },
                    _ => Err(Error::NotImplemented)
                }
            },
            _ => Err(Error::NotImplemented)
        }
    }

    /// Modulus two values.
    pub fn rem(&mut self, other: Self, graph: &mut Graph) -> Result<(), Error> {
        if other.empty() { return Ok(()); }
        match self {
            Self::Null |
            Self::Void => {
                *self = other;
                Ok(())
            },
            Self::Bool(val) => {
                match other {
                    Self::Bool(other) => {
                        *val = *val && other;
                        Ok(())
                    },
                    _ => Err(Error::NotImplemented)
                }
            },
            Self::Str(val) => {
                match other {
                    Self::Str(other) => {
                        let vec = val.split(other.as_str()).collect::<Vec<&str>>();
                        let mut new = Vector::new();
                        for v in vec {
                            new.push_back(ValRef::new(v.into()));
                        }
                        *self = Self::List(new);
                        Ok(())
                    },
                    _ => {
                        *val = format!("{val}{}", other.print(&graph)).into();
                        Ok(())
                    }
                }
            },
            Self::Num(val) => {
                match other {
                    Self::Num(other) => {
                        *val = val.rem(&other);
                        Ok(())
                    },
                    Self::Str(other) => {
                        match number(&other) {
                            Ok((_, res)) => {
                                self.rem(res, graph)?;
                                Ok(())
                            },
                            Err(_) => Err(Error::NotImplemented)
                        }
                    },
                    _ => Err(Error::NotImplemented)
                }
            },
            Self::List(vals) => {
                match other {
                    Self::List(other) => {
                        let mut s = OrdSet::new();
                        for val in vals { s.insert(val.clone()); }
                        let mut os = OrdSet::new();
                        for val in other { os.insert(val); }
                        
                        *self = Self::Set(s.symmetric_difference(os));
                        Ok(())
                    },
                    Self::Set(set) => {
                        let mut os = OrdSet::new();
                        for val in vals { os.insert(val.clone()); }
                        *self = Self::Set(os.symmetric_difference(set));
                        Ok(())
                    },
                    _ => Err(Error::NotImplemented)
                }
            },
            Self::Map(map) => {
                match other {
                    Self::Map(other) => {
                        *self = Self::Map(map.clone().symmetric_difference(other));
                        Ok(())
                    },
                    _ => Err(Error::NotImplemented)
                }
            },
            Self::Set(set) => {
                match other {
                    Self::Set(other) => {
                        *self = Self::Set(set.clone().symmetric_difference(other));
                        Ok(())
                    },
                    Self::List(other) => {
                        let mut os = OrdSet::new();
                        for val in other { os.insert(val); }
                        *self = Self::Set(set.clone().symmetric_difference(os));
                        Ok(())
                    },
                    _ => Err(Error::NotImplemented)
                }
            },
            _ => Err(Error::NotImplemented)
        }
    }

    /// Bitwise AND operation.
    pub fn bit_and(&mut self, other: Self) -> Result<(), Error> {
        match &mut *self {
            Self::Num(num) => {
                match other {
                    Self::Num(onum) => {
                        *num = num.bit_and(&onum);
                        return Ok(());
                    },
                    _ => {}
                }
            },
            _ => {}
        }
        Err(Error::NotImplemented)
    }

    /// Bitwise OR operation.
    pub fn bit_or(&mut self, other: Self) -> Result<(), Error> {
        match self {
            Self::Num(num) => {
                match other {
                    Self::Num(onum) => {
                        *num = num.bit_or(&onum);
                        return Ok(());
                    },
                    _ => {}
                }
            },
            _ => {}
        }
        Err(Error::NotImplemented)
    }

    /// Bitwise XOR operation.
    pub fn bit_xor(&mut self, other: Self) -> Result<(), Error> {
        match self {
            Self::Num(num) => {
                match other {
                    Self::Num(onum) => {
                        *num = num.bit_xor(&onum);
                        return Ok(());
                    },
                    _ => {}
                }
            },
            _ => {}
        }
        Err(Error::NotImplemented)
    }

    /// Bitwise SHIFT LEFT operation.
    pub fn bit_shl(&mut self, other: Self) -> Result<(), Error> {
        match self {
            Self::Num(num) => {
                match other {
                    Self::Num(onum) => {
                        *num = num.bit_shl(&onum);
                        return Ok(());
                    },
                    _ => {}
                }
            },
            _ => {}
        }
        Err(Error::NotImplemented)
    }

    /// Bitwise SHIFT RIGHT operation.
    pub fn bit_shr(&mut self, other: Self) -> Result<(), Error> {
        match self {
            Self::Num(num) => {
                match other {
                    Self::Num(onum) => {
                        *num = num.bit_shr(&onum);
                        return Ok(());
                    },
                    _ => {}
                }
            },
            _ => {}
        }
        Err(Error::NotImplemented)
    }


    /*****************************************************************************
     * Print & Debug.
     *****************************************************************************/
    
    /// Print this value (pretty).
    pub fn print(&self, graph: &Graph) -> String {
        match self {
            Self::Num(_) |
            Self::Void |
            Self::Null |
            Self::Str(_) |
            Self::Bool(_) |
            Self::Ver(..) |
            Self::Fn(_) |
            Self::Blob(_) => self.to_string(),
            Self::Promise(..) |
            Self::Data(_) => self.spec_type(graph).rt_type_of(graph).to_string(),
            Self::Map(map) => {
                let mut res = String::default();
                let mut first = true;
                for (key, val) in map {
                    let key_str;
                    if key.read().str() { key_str = format!("\"{}\"", key.read().print(graph)); }
                    else { key_str = key.read().print(graph); }
                    
                    let val_str;
                    if val.read().str() { val_str = format!("\"{}\"", val.read().print(graph)); }
                    else { val_str = val.read().print(graph); }

                    if first {
                        res.push_str(&format!("({} -> {})", key_str, val_str));
                        first = false;
                    } else {
                        res.push_str(&format!(", ({} -> {})", key_str, val_str));
                    }
                }
                format!("{{{}}}", res)
            },
            Self::Set(set) => {
                let mut res = String::default();
                let mut first = true;
                for val in set {
                    let val_str;
                    if val.read().str() { val_str = format!("\"{}\"", val.read().print(graph)); }
                    else { val_str = val.read().print(graph); }

                    if first {
                        res.push_str(&val_str);
                        first = false;
                    } else {
                        res.push_str(&format!(", {}", val_str));
                    }
                }
                format!("{{{}}}", res)
            },
            Self::List(vals) => {
                let mut res = String::default();
                let mut first = true;
                for val in vals {
                    let val_str;
                    if val.read().str() { val_str = format!("\"{}\"", val.read().print(graph)); }
                    else { val_str = val.read().print(graph); }

                    if first {
                        res.push_str(&val_str);
                        first = false;
                    } else {
                        res.push_str(&format!(", {}", val_str));
                    }
                }
                format!("[{}]", res)
            },
            Self::Tup(vals) => {
                let mut res = String::default();
                let mut first = true;
                for val in vals {
                    let val_str;
                    if val.read().str() { val_str = format!("\"{}\"", val.read().print(graph)); }
                    else { val_str = val.read().print(graph); }

                    if first {
                        res.push_str(&val_str);
                        first = false;
                    } else {
                        res.push_str(&format!(", {}", val_str));
                    }
                }
                format!("({})", res)
            },
            Self::Obj(nref) => {
                let value = json_value_from_node(graph, nref);
                if let Ok(str) = serde_json::to_string(&value) {
                    str
                } else {
                    self.to_string()
                }
            },
        }
    }

    /// Debug print.
    pub fn debug(&self, graph: &Graph) -> String {
        match self {
            Self::Obj(nref) => {
                if let Some(node) = nref.node(graph) {
                    return node.dump(graph, 0, true);
                }
            },
            _ => {}
        }
        self.to_string()
    }


    /*****************************************************************************
     * Max & Min.
     *****************************************************************************/
    
    /// Get the max value from this value.
    pub fn maximum(&self, graph: &Graph) -> Result<Self, Error> {
        match self {
            Self::List(vals) => {
                let mut res = Self::Void;
                for val in vals {
                    let gt = val.read().gt(&res, graph)?;
                    if gt.truthy() || res.empty() {
                        res = val.read().clone();
                    }
                }
                Ok(res)
            },
            Self::Tup(vals) => {
                let mut res = Self::Void;
                for val in vals {
                    let gt = val.read().gt(&res, graph)?;
                    if gt.truthy() || res.empty() {
                        res = val.read().clone();
                    }
                }
                Ok(res)
            },
            Self::Set(set) => {
                let mut res = Self::Void;
                for val in set {
                    let gt = val.read().gt(&res, graph)?;
                    if gt.truthy() || res.empty() {
                        res = val.read().clone();
                    }
                }
                Ok(res)
            },
            _ => {
                Ok(self.clone())
            }
        }
    }

    /// Get the min value from this value.
    pub fn minimum(&self, graph: &Graph) -> Result<Self, Error> {
        match self {
            Self::List(vals) => {
                let mut res = Self::Null;
                for val in vals {
                    let lt = val.read().lt(&res, graph)?;
                    if lt.truthy() || res.empty() {
                        res = val.read().clone();
                    }
                }
                Ok(res)
            },
            Self::Tup(vals) => {
                let mut res = Self::Null;
                for val in vals {
                    let lt = val.read().lt(&res, graph)?;
                    if lt.truthy() || res.empty() {
                        res = val.read().clone();
                    }
                }
                Ok(res)
            },
            Self::Set(set) => {
                let mut res = Self::Null;
                for val in set {
                    let lt = val.read().lt(&res, graph)?;
                    if lt.truthy() || res.empty() {
                        res = val.read().clone();
                    }
                }
                Ok(res)
            },
            _ => {
                Ok(self.clone())
            }
        }
    }
}
