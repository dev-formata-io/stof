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

use std::{cmp::Ordering, hash::Hash};
use arcstr::ArcStr;
use bytes::Bytes;
use imbl::{vector, OrdMap, OrdSet, Vector};
use serde::{Deserialize, Serialize};
use crate::{model::{DataRef, Graph, NodeRef, SId}, runtime::{Error, Num, Type, DATA, OBJ}};


#[derive(Debug, Clone, Serialize, Deserialize, Default, Hash)]
/// Value.
pub enum Val {
    #[default]
    Void,
    Null,
    
    Bool(bool),
    Num(Num),
    Str(ArcStr),

    // Semantic Versioning as a value
    Ver(u32, u32, u32, Option<ArcStr>, Option<ArcStr>),

    Obj(NodeRef),
    Fn(DataRef),
    Data(DataRef),
    Blob(Vec<u8>),

    List(Vector<Self>),
    Tup(Vector<Self>),
    Map(OrdMap<Self, Self>),
    Set(OrdSet<Self>),
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
        }
    }
}
impl Eq for Val {}
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

    #[inline(always)]
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

    #[inline(always)]
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

    #[inline(always)]
    /// Try extracting an data value.
    pub fn try_data(&self) -> Option<DataRef> {
        match self {
            Self::Data(dref) => Some(dref.clone()),
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

    /// Get the generic type for this value.
    pub fn gen_type(&self) -> Type {
        match self {
            Self::Void => Type::Void,
            Self::Null => Type::Null,
            Self::Num(num) => Type::Num(num.ntype()),
            Self::Str(_) => Type::Str,
            Self::Blob(_) => Type::Blob,
            Self::Data(_) => Type::Data(DATA),
            Self::Obj(_) => Type::Obj(OBJ),
            Self::Fn(_) => Type::Fn,
            Self::Ver(..) => Type::Ver,
            Self::List(_) => Type::List,
            Self::Tup(vals) => {
                let mut types = vector![];
                for val in vals { types.push_back(val.gen_type()); }
                Type::Tup(types)
            },
            Self::Map(_) => Type::Map,
            Self::Set(_) => Type::Set,
            Self::Bool(_) => Type::Bool,
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
                // TODO - prototype -> full typepath
                Type::Obj(OBJ)
            },
            _ => self.gen_type()
        }
    }

    /// Cast this value to a different type.
    pub fn cast(&mut self, target: &Type, graph: &mut Graph) -> Result<(), Error> {
        Err(Error::custom("not implemented"))
    }
}
