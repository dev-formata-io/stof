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
use imbl::{OrdMap, OrdSet, Vector};
use serde::{Deserialize, Serialize};
use crate::{model::{DataRef, NodeRef}, runtime::Units};


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
impl PartialOrd for Val {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Val {
    fn cmp(&self, other: &Self) -> Ordering {
        /*match self {
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

                    }
                }
            },
        }*/
        Ordering::Equal
    }
}
impl PartialEq for Val {
    fn eq(&self, other: &Self) -> bool {
        true
    }
}
impl Eq for Val {}


#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
/// Number.
pub enum Num {
    Int(i64),
    Float(f64),
    Units(f64, Units),
}
impl Default for Num {
    fn default() -> Self {
        Self::Int(0)
    }
}
#[derive(Hash)]
struct NumHash(u8, u64, u64);
impl Hash for Num {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Int(val) => val.hash(state),
            Self::Float(val) => {
                let mut sign = 1;
                if val.signum() < 0. { sign = 2; }
                NumHash(sign, val.trunc() as u64, (val.fract() * 1000000.) as u64).hash(state)
            },
            Self::Units(val, _) => {
                let mut sign = 1;
                if val.signum() < 0. { sign = 2; }
                NumHash(sign, val.trunc() as u64, (val.fract() * 1000000.) as u64).hash(state)
            },
        }
    }
}
impl PartialEq for Num {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Int(val) => {
                match other {
                    Self::Int(oval) => {
                        *val == *oval
                    },
                    Self::Float(oval) => {
                        *val as f64 == *oval
                    },
                    Self::Units(oval, ounits) => {
                        let mut base = *ounits;
                        if base.is_angle() {
                            // Make sure for eq we are always converting to positive radians!
                            base = Units::PositiveRadians;
                        }
                        if let Ok(a) = Units::convert(*val as f64, base, base) {
                            if let Ok(b) = Units::convert(*oval, *ounits, base) {
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
            Self::Float(val) => {
                match other {
                    Self::Int(oval) => {
                        *val == *oval as f64
                    },
                    Self::Float(oval) => {
                        *val == *oval
                    },
                    Self::Units(oval, ounits) => {
                        let mut base = *ounits;
                        if base.is_angle() {
                            // Make sure for eq we are always converting to positive radians!
                            base = Units::PositiveRadians;
                        }
                        if let Ok(a) = Units::convert(*val, base, base) {
                            if let Ok(b) = Units::convert(*oval, *ounits, base) {
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
                    Self::Int(oval) => {
                        let mut base = *units;
                        if base.is_angle() {
                            // Make sure for eq we are always converting to positive radians!
                            base = Units::PositiveRadians;
                        }
                        if let Ok(a) = Units::convert(*val, base, base) {
                            if let Ok(b) = Units::convert(*oval as f64, base, base) {
                                if base.is_angle() {
                                    // Lower precision for angles 6 places
                                    return (a*1000000.).round() == (b*1000000.).round();
                                }
                                return a == b;
                            }
                        }
                        *val == *oval as f64
                    },
                    Self::Float(oval) => {
                        let mut base = *units;
                        if base.is_angle() {
                            // Make sure for eq we are always converting to positive radians!
                            base = Units::PositiveRadians;
                        }
                        if let Ok(a) = Units::convert(*val, base, base) {
                            if let Ok(b) = Units::convert(*oval, base, base) {
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
                            base = Units::PositiveRadians;
                        }
                        if let Ok(a) = Units::convert(*val, *units, base) {
                            if let Ok(b) = Units::convert(*oval, *ounits, base) {
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
impl Eq for Num {}
impl From<i32> for Num {
    fn from(value: i32) -> Self {
        Self::Int(value as i64)
    }
}
impl From<i64> for Num {
    fn from(value: i64) -> Self {
        Self::Int(value)
    }
}
impl From<i16> for Num {
    fn from(value: i16) -> Self {
        Self::Int(value as i64)
    }
}
impl From<i8> for Num {
    fn from(value: i8) -> Self {
        Self::Int(value as i64)
    }
}
impl From<i128> for Num {
    fn from(value: i128) -> Self {
        Self::Int(value as i64)
    }
}
impl From<f32> for Num {
    fn from(value: f32) -> Self {
        Self::Float(value as f64)
    }
}
impl From<f64> for Num {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}
impl From<(i32, Units)> for Num {
    fn from(value: (i32, Units)) -> Self {
        Self::Units(value.0 as f64, value.1)
    }
}
impl From<(i64, Units)> for Num {
    fn from(value: (i64, Units)) -> Self {
        Self::Units(value.0 as f64, value.1)
    }
}
impl From<(f32, Units)> for Num {
    fn from(value: (f32, Units)) -> Self {
        Self::Units(value.0 as f64, value.1)
    }
}
impl From<(f64, Units)> for Num {
    fn from(value: (f64, Units)) -> Self {
        Self::Units(value.0, value.1)
    }
}

