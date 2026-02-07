//
// Copyright 2025 Formata, Inc. All rights reserved.
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

use std::hash::Hash;
use serde::{Deserialize, Serialize};
use crate::runtime::{Error, NumT, Units};


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
impl ToString for Num {
    fn to_string(&self) -> String {
        self.print()
    }
}
impl Num {
    /// Gen type for this number.
    pub fn gen_type(&self) -> NumT {
        match self {
            Self::Float(_) => NumT::Float,
            Self::Int(_) => NumT::Int,
            Self::Units(_, _) => NumT::Float,
        }
    }

    /// Type for this number.
    pub fn ntype(&self) -> NumT {
        match self {
            Self::Float(_) => NumT::Float,
            Self::Int(_) => NumT::Int,
            Self::Units(_, units) => NumT::Units(*units),
        }
    }

    /// Print this number.
    pub fn print(&self) -> String {
        match self {
            Self::Int(val) => format!("{val}"),
            Self::Float(val) => format!("{val}"),
            Self::Units(val, units) => format!("{val}{}", units.to_string())
        }
    }

    /// Debug this number.
    pub fn debug(&self) -> String {
        match self {
            Self::Int(val) => format!("Int({val})"),
            Self::Float(val) => format!("Float({val})"),
            Self::Units(val, units) => format!("Units({val}, {:?})", units)
        }
    }

    /// Truthy?
    pub fn truthy(&self) -> bool {
        match self {
            Self::Int(v) => *v != 0,
            Self::Float(v) => v.round() != 0.,
            Self::Units(v, _) => v.round() != 0.
        }
    }

    /// This number has units?
    pub fn has_units(&self) -> bool {
        match self {
            Self::Units(_, u) => u.has_units(),
            _ => false,
        }
    }

    /// Remove units.
    pub fn remove_units(&mut self) {
        match &mut *self {
            Self::Units(v, _) => {
                *self = Self::Float(*v);
            },
            _ => {}
        }
    }

    /// Is angle?
    pub fn is_angle(&self) -> bool {
        match self {
            Self::Units(_, units) => {
                units.is_angle()
            },
            _ => false,
        }
    }

    /// Is temp?
    pub fn is_temp(&self) -> bool {
        match self {
            Self::Units(_, units) => {
                units.is_temperature()
            },
            _ => false,
        }
    }

    /// Is length?
    pub fn is_length(&self) -> bool {
        match self {
            Self::Units(_, units) => {
                units.is_length()
            },
            _ => false,
        }
    }

    /// Is time?
    pub fn is_time(&self) -> bool {
        match self {
            Self::Units(_, units) => {
                units.is_time()
            },
            _ => false,
        }
    }

    /// Is mass?
    pub fn is_mass(&self) -> bool {
        match self {
            Self::Units(_, units) => {
                units.is_mass()
            },
            _ => false,
        }
    }

    /// Is memory?
    pub fn is_memory(&self) -> bool {
        match self {
            Self::Units(_, units) => {
                units.is_memory()
            },
            _ => false,
        }
    }

    /// Get units.
    pub fn units(&self) -> Option<Units> {
        match self {
            Self::Units(_, u) => {
                if u.has_units() { Some(*u) }
                else { None }
            },
            _ => None
        }
    }

    /// Get integer portion of this number.
    pub fn int(&self) -> i64 {
        match self {
            Self::Int(v) => *v,
            Self::Float(v) => *v as i64,
            Self::Units(v, _) => *v as i64
        }
    }

    /// Get float, optionally converted to some units.
    pub fn float(&self, units: Option<Units>) -> f64 {
        match self {
            Self::Int(val) => {
                if let Some(units) = units {
                    if let Ok(val) = Units::convert(*val as f64, units, units) {
                        return val;
                    }
                }
                *val as f64
            },
            Self::Float(val) => {
                if let Some(units) = units {
                    if let Ok(val) = Units::convert(*val, units, units) {
                        return val;
                    }
                }
                *val
            },
            Self::Units(val, sunits) => {
                if let Some(units) = units {
                    if let Ok(val) = Units::convert(*val, *sunits, units) {
                        return val;
                    }
                }
                *val
            },
        }
    }

    /// Cast a number into another number type.
    pub fn cast(&self, target: NumT) -> Self {
        match self {
            Self::Int(val) => {
                match target {
                    NumT::Int => Self::Int(*val as i64),
                    NumT::Float => Self::Float(*val as f64),
                    NumT::Units(ounits) => {
                        if let Ok(v) = Units::convert(*val as f64, ounits, ounits) {
                            Self::Units(v, ounits)
                        } else {
                            Self::Units(*val as f64, ounits)
                        }
                    }
                }
            },
            Self::Float(val) => {
                match target {
                    NumT::Int => Self::Int(*val as i64),
                    NumT::Float => Self::Float(*val as f64),
                    NumT::Units(ounits) => {
                        if let Ok(v) = Units::convert(*val, ounits, ounits) {
                            Self::Units(v, ounits)
                        } else {
                            Self::Units(*val, ounits)
                        }
                    }
                }
            },
            Self::Units(val, units) => {
                match target {
                    NumT::Int => Self::Int(*val as i64),
                    NumT::Float => Self::Float(*val as f64),
                    NumT::Units(ounits) => {
                        // Try casting directly to ounits
                        if let Ok(value) = Units::convert(*val, *units, ounits) {
                            return Self::Units(value, ounits);
                        }

                        // Try finding a base unit next...
                        let base = units.common(ounits);
                        if let Ok(value) = Units::convert(*val, *units, base) {
                            return Self::Units(value, base);
                        }

                        // No units anymore...
                        Self::Float(*val)
                    },
                }
            }
        }
    }

    /// Greater than another number?
    pub fn gt(&self, other: &Self) -> bool {
        match self {
            Self::Int(val) => {
                match other {
                    Self::Int(oval) => *val > *oval,
                    Self::Float(oval) => *val as f64 > *oval,
                    Self::Units(oval, ounits) => {
                        let mut base = *ounits;
                        if base.is_angle() { base = Units::PositiveRadians; }
                        if let Ok(a) = Units::convert(*val as f64, base, base) {
                            if let Ok(b) = Units::convert(*oval, *ounits, base) {
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
            Self::Float(val) => {
                match other {
                    Self::Int(oval) => *val > *oval as f64,
                    Self::Float(oval) => *val > *oval,
                    Self::Units(oval, ounits) => {
                        let mut base = *ounits;
                        if base.is_angle() { base = Units::PositiveRadians; }
                        if let Ok(a) = Units::convert(*val, base, base) {
                            if let Ok(b) = Units::convert(*oval, *ounits, base) {
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
                    Self::Int(oval) => {
                        let mut base = *units;
                        if base.is_angle() { base = Units::PositiveRadians; }
                        if let Ok(a) = Units::convert(*val, base, base) {
                            if let Ok(b) = Units::convert(*oval as f64, base, base) {
                                if base.is_angle() {
                                    return (a*1000000.).round() > (b*1000000.).round();
                                }
                                return a > b;
                            }
                        }
                        *val > *oval as f64
                    },
                    Self::Float(oval) => {
                        let mut base = *units;
                        if base.is_angle() { base = Units::PositiveRadians; }
                        if let Ok(a) = Units::convert(*val, base, base) {
                            if let Ok(b) = Units::convert(*oval, base, base) {
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
                        if base.is_angle() { base = Units::PositiveRadians; }
                        if let Ok(a) = Units::convert(*val, *units, base) {
                            if let Ok(b) = Units::convert(*oval, *ounits, base) {
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
            Self::Int(val) => {
                match other {
                    Self::Int(oval) => *val < *oval,
                    Self::Float(oval) => (*val as f64) < *oval,
                    Self::Units(oval, ounits) => {
                        let mut base = *ounits;
                        if base.is_angle() { base = Units::PositiveRadians; }
                        if let Ok(a) = Units::convert(*val as f64, base, base) {
                            if let Ok(b) = Units::convert(*oval, *ounits, base) {
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
            Self::Float(val) => {
                match other {
                    Self::Int(oval) => *val < *oval as f64,
                    Self::Float(oval) => *val < *oval,
                    Self::Units(oval, ounits) => {
                        let mut base = *ounits;
                        if base.is_angle() { base = Units::PositiveRadians; }
                        if let Ok(a) = Units::convert(*val, base, base) {
                            if let Ok(b) = Units::convert(*oval, *ounits, base) {
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
                    Self::Int(oval) => {
                        let mut base = *units;
                        if base.is_angle() { base = Units::PositiveRadians; }
                        if let Ok(a) = Units::convert(*val, base, base) {
                            if let Ok(b) = Units::convert(*oval as f64, base, base) {
                                if base.is_angle() {
                                    return (a*1000000.).round() < (b*1000000.).round();
                                }
                                return a < b;
                            }
                        }
                        *val < *oval as f64
                    },
                    Self::Float(oval) => {
                        let mut base = *units;
                        if base.is_angle() { base = Units::PositiveRadians; }
                        if let Ok(a) = Units::convert(*val, base, base) {
                            if let Ok(b) = Units::convert(*oval, base, base) {
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
                        if base.is_angle() { base = Units::PositiveRadians; }
                        if let Ok(a) = Units::convert(*val, *units, base) {
                            if let Ok(b) = Units::convert(*oval, *ounits, base) {
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
            Self::Int(val) => {
                match other {
                    Self::Int(bval) => {
                        Self::Int(*val + *bval)
                    },
                    Self::Float(bval) => {
                        Self::Float(*val as f64 + *bval)
                    },
                    Self::Units(bval, ounits) => {
                        let mut res = *val as f64 + *bval;
                        let base = *ounits;
                        if let Ok(a) = Units::convert(*val as f64, base, base) {
                            if let Ok(c) = Units::convert(a + *bval, base, base) {
                                res = c;
                            }
                        }
                        Self::Units(res, base)
                    }
                }
            },
            Self::Float(val) => {
                match other {
                    Self::Int(bval) => {
                        Self::Float(*val + *bval as f64)
                    },
                    Self::Float(bval) => {
                        Self::Float(*val + *bval)
                    },
                    Self::Units(bval, ounits) => {
                        let mut res = *val + *bval;
                        let base = *ounits;
                        if let Ok(a) = Units::convert(*val, base, base) {
                            if let Ok(c) = Units::convert(a + *bval, base, base) {
                                res = c;
                            }
                        }
                        Self::Units(res, base)
                    }
                }
            },
            Self::Units(val, units) => {
                match other {
                    Self::Int(bval) => {
                        let mut res = *val + *bval as f64;
                        let base = *units;
                        if let Ok(a) = Units::convert(*val, base, base) {
                            if let Ok(c) = Units::convert(a + *bval as f64, base, base) {
                                res = c;
                            }
                        }
                        Self::Units(res, base)
                    },
                    Self::Float(bval) => {
                        let mut res = *val + *bval;
                        let base = *units;
                        if let Ok(a) = Units::convert(*val, base, base) {
                            if let Ok(c) = Units::convert(a + *bval, base, base) {
                                res = c;
                            }
                        }
                        Self::Units(res, base)
                    },
                    Self::Units(bval, ounits) => {
                        let mut res = *val + *bval;
                        let base = units.common(*ounits);
                        if let Ok(a) = Units::convert(*val, *units, base) {
                            if let Ok(b) = Units::convert(*bval, *ounits, base) {
                                if let Ok(c) = Units::convert(a + b, base, base) {
                                    res = c;
                                } else {
                                    res = a + b;
                                }
                                if base.is_undefined() {
                                    return Self::Float(res);
                                }
                                return Self::Units(res, base);
                            }
                        }
                        // No units anymore...
                        Self::Float(res)
                    }
                }
            }
        }
    }

    /// Subtract two number.
    pub fn sub(&self, other: &Self) -> Self {
        match self {
            Self::Int(val) => {
                match other {
                    Self::Int(bval) => {
                        Self::Int(*val - *bval)
                    },
                    Self::Float(bval) => {
                        Self::Float(*val as f64 - *bval)
                    },
                    Self::Units(bval, ounits) => {
                        let mut res = *val as f64 - *bval;
                        let base = *ounits;
                        if let Ok(a) = Units::convert(*val as f64, base, base) {
                            if let Ok(c) = Units::convert(a - *bval, base, base) {
                                res = c;
                            }
                        }
                        Self::Units(res, base)
                    }
                }
            },
            Self::Float(val) => {
                match other {
                    Self::Int(bval) => {
                        Self::Float(*val - *bval as f64)
                    },
                    Self::Float(bval) => {
                        Self::Float(*val - *bval)
                    },
                    Self::Units(bval, ounits) => {
                        let mut res = *val - *bval;
                        let base = *ounits;
                        if let Ok(a) = Units::convert(*val, base, base) {
                            if let Ok(c) = Units::convert(a - *bval, base, base) {
                                res = c;
                            }
                        }
                        Self::Units(res, base)
                    }
                }
            },
            Self::Units(val, units) => {
                match other {
                    Self::Int(bval) => {
                        let mut res = *val - *bval as f64;
                        let base = *units;
                        if let Ok(a) = Units::convert(*val, base, base) {
                            if let Ok(c) = Units::convert(a - *bval as f64, base, base) {
                                res = c;
                            }
                        }
                        Self::Units(res, base)
                    },
                    Self::Float(bval) => {
                        let mut res = *val - *bval;
                        let base = *units;
                        if let Ok(a) = Units::convert(*val, base, base) {
                            if let Ok(c) = Units::convert(a - *bval, base, base) {
                                res = c;
                            }
                        }
                        Self::Units(res, base)
                    },
                    Self::Units(bval, ounits) => {
                        let mut res = *val - *bval;
                        let base = units.common(*ounits);
                        if let Ok(a) = Units::convert(*val, *units, base) {
                            if let Ok(b) = Units::convert(*bval, *ounits, base) {
                                if let Ok(c) = Units::convert(a - b, base, base) {
                                    res = c;
                                } else {
                                    res = a - b;
                                }
                                if base.is_undefined() {
                                    return Self::Float(res);
                                }
                                return Self::Units(res, base);
                            }
                        }
                        // No units anymore...
                        Self::Float(res)
                    }
                }
            }
        }
    }

    /// Multiply two numbers.
    pub fn mul(&self, other: &Self) -> Self {
        match self {
            Self::Int(val) => {
                match other {
                    Self::Int(bval) => {
                        Self::Int(*val * *bval)
                    },
                    Self::Float(bval) => {
                        Self::Float(*val as f64 * *bval)
                    },
                    Self::Units(bval, ounits) => {
                        let mut res = *val as f64 * *bval;
                        let base = *ounits;
                        if let Ok(a) = Units::convert(*val as f64, base, base) {
                            if let Ok(c) = Units::convert(a * *bval, base, base) {
                                res = c;
                            }
                        }
                        Self::Units(res, base)
                    }
                }
            },
            Self::Float(val) => {
                match other {
                    Self::Int(bval) => {
                        Self::Float(*val * *bval as f64)
                    },
                    Self::Float(bval) => {
                        Self::Float(*val * *bval)
                    },
                    Self::Units(bval, ounits) => {
                        let mut res = *val * *bval;
                        let base = *ounits;
                        if let Ok(a) = Units::convert(*val, base, base) {
                            if let Ok(c) = Units::convert(a * *bval, base, base) {
                                res = c;
                            }
                        }
                        Self::Units(res, base)
                    }
                }
            },
            Self::Units(val, units) => {
                match other {
                    Self::Int(bval) => {
                        let mut res = *val * *bval as f64;
                        let base = *units;
                        if let Ok(a) = Units::convert(*val, base, base) {
                            if let Ok(c) = Units::convert(a * *bval as f64, base, base) {
                                res = c;
                            }
                        }
                        Self::Units(res, base)
                    },
                    Self::Float(bval) => {
                        let mut res = *val * *bval;
                        let base = *units;
                        if let Ok(a) = Units::convert(*val, base, base) {
                            if let Ok(c) = Units::convert(a * *bval, base, base) {
                                res = c;
                            }
                        }
                        Self::Units(res, base)
                    },
                    Self::Units(bval, ounits) => {
                        let mut res = *val * *bval;
                        let base = units.common(*ounits);
                        if let Ok(a) = Units::convert(*val, *units, base) {
                            if let Ok(b) = Units::convert(*bval, *ounits, base) {
                                if let Ok(c) = Units::convert(a * b, base, base) {
                                    res = c;
                                } else {
                                    res = a * b;
                                }
                                if base.is_undefined() {
                                    return Self::Float(res);
                                }
                                return Self::Units(res, base);
                            }
                        }
                        // No units anymore...
                        Self::Float(res)
                    }
                }
            }
        }
    }

    /// Divide two numbers.
    pub fn div(&self, other: &Self) -> Self {
        match self {
            Self::Int(val) => {
                match other {
                    Self::Int(bval) => {
                        Self::Int(*val / *bval)
                    },
                    Self::Float(bval) => {
                        Self::Float(*val as f64 / *bval)
                    },
                    Self::Units(bval, ounits) => {
                        let mut res = *val as f64 / *bval;
                        let base = *ounits;
                        if let Ok(a) = Units::convert(*val as f64, base, base) {
                            if let Ok(c) = Units::convert(a / *bval, base, base) {
                                res = c;
                            }
                        }
                        Self::Units(res, base)
                    }
                }
            },
            Self::Float(val) => {
                match other {
                    Self::Int(bval) => {
                        Self::Float(*val / *bval as f64)
                    },
                    Self::Float(bval) => {
                        Self::Float(*val / *bval)
                    },
                    Self::Units(bval, ounits) => {
                        let mut res = *val / *bval;
                        let base = *ounits;
                        if let Ok(a) = Units::convert(*val, base, base) {
                            if let Ok(c) = Units::convert(a / *bval, base, base) {
                                res = c;
                            }
                        }
                        Self::Units(res, base)
                    }
                }
            },
            Self::Units(val, units) => {
                match other {
                    Self::Int(bval) => {
                        let mut res = *val / *bval as f64;
                        let base = *units;
                        if let Ok(a) = Units::convert(*val, base, base) {
                            if let Ok(c) = Units::convert(a / *bval as f64, base, base) {
                                res = c;
                            }
                        }
                        Self::Units(res, base)
                    },
                    Self::Float(bval) => {
                        let mut res = *val / *bval;
                        let base = *units;
                        if let Ok(a) = Units::convert(*val, base, base) {
                            if let Ok(c) = Units::convert(a / *bval, base, base) {
                                res = c;
                            }
                        }
                        Self::Units(res, base)
                    },
                    Self::Units(bval, ounits) => {
                        let mut res = *val / *bval;
                        let base = units.common(*ounits);
                        if let Ok(a) = Units::convert(*val, *units, base) {
                            if let Ok(b) = Units::convert(*bval, *ounits, base) {
                                if let Ok(c) = Units::convert(a / b, base, base) {
                                    res = c;
                                } else {
                                    res = a / b;
                                }
                                if base.is_undefined() {
                                    return Self::Float(res);
                                }
                                return Self::Units(res, base);
                            }
                        }
                        // No units anymore...
                        Self::Float(res)
                    }
                }
            }
        }
    }

    /// Rem (mod) between two numbers.
    pub fn rem(&self, other: &Self) -> Self {
        match self {
            Self::Int(val) => {
                match other {
                    Self::Int(bval) => {
                        Self::Int(*val % *bval)
                    },
                    Self::Float(bval) => {
                        Self::Float(*val as f64 % *bval)
                    },
                    Self::Units(bval, ounits) => {
                        let mut res = *val as f64 % *bval;
                        let base = *ounits;
                        if let Ok(a) = Units::convert(*val as f64, base, base) {
                            if let Ok(c) = Units::convert(a % *bval, base, base) {
                                res = c;
                            }
                        }
                        Self::Units(res, base)
                    }
                }
            },
            Self::Float(val) => {
                match other {
                    Self::Int(bval) => {
                        Self::Float(*val % *bval as f64)
                    },
                    Self::Float(bval) => {
                        Self::Float(*val % *bval)
                    },
                    Self::Units(bval, ounits) => {
                        let mut res = *val % *bval;
                        let base = *ounits;
                        if let Ok(a) = Units::convert(*val, base, base) {
                            if let Ok(c) = Units::convert(a % *bval, base, base) {
                                res = c;
                            }
                        }
                        Self::Units(res, base)
                    }
                }
            },
            Self::Units(val, units) => {
                match other {
                    Self::Int(bval) => {
                        let mut res = *val % *bval as f64;
                        let base = *units;
                        if let Ok(a) = Units::convert(*val, base, base) {
                            if let Ok(c) = Units::convert(a % *bval as f64, base, base) {
                                res = c;
                            }
                        }
                        Self::Units(res, base)
                    },
                    Self::Float(bval) => {
                        let mut res = *val % *bval;
                        let base = *units;
                        if let Ok(a) = Units::convert(*val, base, base) {
                            if let Ok(c) = Units::convert(a % *bval, base, base) {
                                res = c;
                            }
                        }
                        Self::Units(res, base)
                    },
                    Self::Units(bval, ounits) => {
                        let mut res = *val % *bval;
                        let base = units.common(*ounits);
                        if let Ok(a) = Units::convert(*val, *units, base) {
                            if let Ok(b) = Units::convert(*bval, *ounits, base) {
                                if let Ok(c) = Units::convert(a % b, base, base) {
                                    res = c;
                                } else {
                                    res = a % b;
                                }
                                if base.is_undefined() {
                                    return Self::Float(res);
                                }
                                return Self::Units(res, base);
                            }
                        }
                        // No units anymore...
                        Self::Float(res)
                    }
                }
            }
        }
    }


    /*****************************************************************************
     * Bitwise operations.
     *****************************************************************************/
    
    /// Bitwise and operation.
    pub fn bit_and(&self, other: &Self) -> Self {
        match self {
            Self::Int(val) => {
                match other {
                    Self::Int(oval) => Self::Int(*val & *oval),
                    Self::Float(oval) => Self::Int(*val & *oval as i64),
                    Self::Units(oval, _) => Self::Int(*val & *oval as i64),
                }
            },
            Self::Float(val) => {
                match other {
                    Self::Int(oval) => Self::Int((*val as i64) & *oval),
                    Self::Float(oval) => Self::Int((*val as i64) & *oval as i64),
                    Self::Units(oval, _) => Self::Int((*val as i64) & *oval as i64),
                }
            },
            Self::Units(val, units) => {
                match other {
                    Self::Int(oval) => Self::Units(((*val as i64) & *oval) as f64, *units),
                    Self::Float(oval) => Self::Units(((*val as i64) & *oval as i64) as f64, *units),
                    Self::Units(oval, _) => Self::Units(((*val as i64) & *oval as i64) as f64, *units),
                }
            },
        }
    }

    /// Bitwise or operation.
    pub fn bit_or(&self, other: &Self) -> Self {
        match self {
            Self::Int(val) => {
                match other {
                    Self::Int(oval) => Self::Int(*val | *oval),
                    Self::Float(oval) => Self::Int(*val | *oval as i64),
                    Self::Units(oval, _) => Self::Int(*val | *oval as i64),
                }
            },
            Self::Float(val) => {
                match other {
                    Self::Int(oval) => Self::Int((*val as i64) | *oval),
                    Self::Float(oval) => Self::Int((*val as i64) | *oval as i64),
                    Self::Units(oval, _) => Self::Int((*val as i64) | *oval as i64),
                }
            },
            Self::Units(val, units) => {
                match other {
                    Self::Int(oval) => Self::Units(((*val as i64) | *oval) as f64, *units),
                    Self::Float(oval) => Self::Units(((*val as i64) | *oval as i64) as f64, *units),
                    Self::Units(oval, _) => Self::Units(((*val as i64) | *oval as i64) as f64, *units),
                }
            },
        }
    }

    /// Bitwise xor operation.
    pub fn bit_xor(&self, other: &Self) -> Self {
        match self {
            Self::Int(val) => {
                match other {
                    Self::Int(oval) => Self::Int(*val ^ *oval),
                    Self::Float(oval) => Self::Int(*val ^ *oval as i64),
                    Self::Units(oval, _) => Self::Int(*val ^ *oval as i64),
                }
            },
            Self::Float(val) => {
                match other {
                    Self::Int(oval) => Self::Int((*val as i64) ^ *oval),
                    Self::Float(oval) => Self::Int((*val as i64) ^ *oval as i64),
                    Self::Units(oval, _) => Self::Int((*val as i64) ^ *oval as i64),
                }
            },
            Self::Units(val, units) => {
                match other {
                    Self::Int(oval) => Self::Units(((*val as i64) ^ *oval) as f64, *units),
                    Self::Float(oval) => Self::Units(((*val as i64) ^ *oval as i64) as f64, *units),
                    Self::Units(oval, _) => Self::Units(((*val as i64) ^ *oval as i64) as f64, *units),
                }
            },
        }
    }

    /// Bitwise shift left operation.
    pub fn bit_shl(&self, other: &Self) -> Self {
        match self {
            Self::Int(val) => {
                match other {
                    Self::Int(oval) => Self::Int(*val << *oval),
                    Self::Float(oval) => Self::Int(*val << *oval as i64),
                    Self::Units(oval, _) => Self::Int(*val << *oval as i64),
                }
            },
            Self::Float(val) => {
                match other {
                    Self::Int(oval) => Self::Int((*val as i64) << *oval),
                    Self::Float(oval) => Self::Int((*val as i64) << *oval as i64),
                    Self::Units(oval, _) => Self::Int((*val as i64) << *oval as i64),
                }
            },
            Self::Units(val, units) => {
                match other {
                    Self::Int(oval) => Self::Units(((*val as i64) << *oval) as f64, *units),
                    Self::Float(oval) => Self::Units(((*val as i64) << *oval as i64) as f64, *units),
                    Self::Units(oval, _) => Self::Units(((*val as i64) << *oval as i64) as f64, *units),
                }
            },
        }
    }

    /// Bitwise shift right operation.
    pub fn bit_shr(&self, other: &Self) -> Self {
        match self {
            Self::Int(val) => {
                match other {
                    Self::Int(oval) => Self::Int(*val >> *oval),
                    Self::Float(oval) => Self::Int(*val >> *oval as i64),
                    Self::Units(oval, _) => Self::Int(*val >> *oval as i64),
                }
            },
            Self::Float(val) => {
                match other {
                    Self::Int(oval) => Self::Int((*val as i64) >> *oval),
                    Self::Float(oval) => Self::Int((*val as i64) >> *oval as i64),
                    Self::Units(oval, _) => Self::Int((*val as i64) >> *oval as i64),
                }
            },
            Self::Units(val, units) => {
                match other {
                    Self::Int(oval) => Self::Units(((*val as i64) >> *oval) as f64, *units),
                    Self::Float(oval) => Self::Units(((*val as i64) >> *oval as i64) as f64, *units),
                    Self::Units(oval, _) => Self::Units(((*val as i64) >> *oval as i64) as f64, *units),
                }
            },
        }
    }


    /*****************************************************************************
     * Library ops.
     *****************************************************************************/
    
    /// Abs.
    pub fn abs(&mut self) -> Result<(), Error> {
        match &mut *self {
            Self::Float(v) => {
                *v = v.abs();
            },
            Self::Int(v) => {
                *v = v.abs();
            },
            Self::Units(v, _) => {
                *v = v.abs();
            }
        }
        Ok(())
    }

    /// Sqrt.
    pub fn sqrt(&mut self) -> Result<(), Error> {
        match &mut *self {
            Self::Float(v) => {
                *v = v.sqrt();
            },
            Self::Int(v) => {
                // just turn it into a float
                *self = Self::Float((*v as f64).sqrt());
            },
            Self::Units(v, _) => {
                *v = v.sqrt();
            }
        }
        Ok(())
    }

    /// Cbrt.
    pub fn cbrt(&mut self) -> Result<(), Error> {
        match &mut *self {
            Self::Float(v) => {
                *v = v.cbrt();
            },
            Self::Int(v) => {
                // just turn it into a float
                *self = Self::Float((*v as f64).cbrt());
            },
            Self::Units(v, _) => {
                *v = v.cbrt();
            }
        }
        Ok(())
    }

    /// Floor.
    pub fn floor(&mut self) -> Result<(), Error> {
        match &mut *self {
            Self::Float(v) => {
                *v = v.floor();
            },
            Self::Int(_v) => {
                // nothing to do here
            },
            Self::Units(v, _) => {
                *v = v.floor();
            }
        }
        Ok(())
    }

    /// Ceil.
    pub fn ceil(&mut self) -> Result<(), Error> {
        match &mut *self {
            Self::Float(v) => {
                *v = v.ceil();
            },
            Self::Int(_v) => {
                // nothing to do here
            },
            Self::Units(v, _) => {
                *v = v.ceil();
            }
        }
        Ok(())
    }

    /// Trunc.
    pub fn trunc(&mut self) -> Result<(), Error> {
        match &mut *self {
            Self::Float(v) => {
                *v = v.trunc();
            },
            Self::Int(_v) => {
                // nothing to do here
            },
            Self::Units(v, _) => {
                *v = v.trunc();
            }
        }
        Ok(())
    }

    /// Fract.
    pub fn fract(&mut self) -> Result<(), Error> {
        match &mut *self {
            Self::Float(v) => {
                *v = v.fract();
            },
            Self::Int(v) => {
                *v = 0; // no fractional part of an integer
            },
            Self::Units(v, _) => {
                *v = v.fract();
            }
        }
        Ok(())
    }

    /// Signum.
    pub fn signum(&mut self) -> Result<(), Error> {
        match &mut *self {
            Self::Float(v) => {
                *v = v.signum();
            },
            Self::Int(v) => {
                *v = v.signum();
            },
            Self::Units(v, _) => {
                *v = v.signum();
            }
        }
        Ok(())
    }

    /// Exp.
    pub fn exp(&mut self) -> Result<(), Error> {
        match &mut *self {
            Self::Float(v) => {
                *v = v.exp();
            },
            Self::Int(v) => {
                *self = Self::Float((*v as f64).exp());
            },
            Self::Units(v, _) => {
                *v = v.exp();
            }
        }
        Ok(())
    }

    /// Exp 2.
    pub fn exp2(&mut self) -> Result<(), Error> {
        match &mut *self {
            Self::Float(v) => {
                *v = v.exp2();
            },
            Self::Int(v) => {
                *self = Self::Float((*v as f64).exp2());
            },
            Self::Units(v, _) => {
                *v = v.exp2();
            }
        }
        Ok(())
    }

    /// NaN?
    pub fn nan(&self) -> bool {
        match self {
            Self::Float(v) => {
                v.is_nan()
            },
            Self::Int(_v) => {
                false
            },
            Self::Units(v, _) => {
                v.is_nan()
            }
        }
    }

    /// Infinite?
    pub fn inf(&self) -> bool {
        match self {
            Self::Float(v) => {
                v.is_infinite()
            },
            Self::Int(_v) => {
                false
            },
            Self::Units(v, _) => {
                v.is_infinite()
            }
        }
    }

    /// Ln.
    pub fn ln(&mut self) -> Result<(), Error> {
        match &mut *self {
            Self::Float(v) => {
                *v = v.ln();
            },
            Self::Int(v) => {
                *self = Self::Float((*v as f64).ln());
            },
            Self::Units(v, _) => {
                *v = v.ln();
            }
        }
        Ok(())
    }

    /// Sin.
    pub fn sin(&mut self) -> Result<(), Error> {
        match &mut *self {
            Self::Float(v) => {
                *v = v.sin();
            },
            Self::Int(v) => {
                *self = Self::Float((*v as f64).sin());
            },
            Self::Units(v, units) => {
                if units.is_degrees() {
                    *v = Units::to_radians(*v, Units::Degrees).sin();
                } else {
                    *v = v.sin();
                }
                *self = Self::Float(*v);
            }
        }
        Ok(())
    }

    /// Cos.
    pub fn cos(&mut self) -> Result<(), Error> {
        match &mut *self {
            Self::Float(v) => {
                *v = v.cos();
            },
            Self::Int(v) => {
                *self = Self::Float((*v as f64).cos());
            },
            Self::Units(v, units) => {
                if units.is_degrees() {
                    *v = Units::to_radians(*v, Units::Degrees).cos();
                } else {
                    *v = v.cos();
                }
                *self = Self::Float(*v);
            }
        }
        Ok(())
    }

    /// Tan.
    pub fn tan(&mut self) -> Result<(), Error> {
        match &mut *self {
            Self::Float(v) => {
                *v = v.tan();
            },
            Self::Int(v) => {
                *self = Self::Float((*v as f64).tan());
            },
            Self::Units(v, units) => {
                if units.is_degrees() {
                    *v = Units::to_radians(*v, Units::Degrees).tan();
                } else {
                    *v = v.tan();
                }
                *self = Self::Float(*v);
            }
        }
        Ok(())
    }

    /// ASin.
    pub fn asin(&mut self) -> Result<(), Error> {
        let rad;
        match &mut *self {
            Self::Float(v) => {
                rad = v.asin();
            },
            Self::Int(v) => {
                rad = (*v as f64).asin();
            },
            Self::Units(v, _) => {
                rad = v.asin();
            }
        }
        *self = Self::Units(rad, Units::Radians);
        Ok(())
    }

    /// ACos.
    pub fn acos(&mut self) -> Result<(), Error> {
        let rad;
        match &mut *self {
            Self::Float(v) => {
                rad = v.acos();
            },
            Self::Int(v) => {
                rad = (*v as f64).acos();
            },
            Self::Units(v, _) => {
                rad = v.acos();
            }
        }
        *self = Self::Units(rad, Units::Radians);
        Ok(())
    }

    /// ATan.
    pub fn atan(&mut self) -> Result<(), Error> {
        let rad;
        match &mut *self {
            Self::Float(v) => {
                rad = v.atan();
            },
            Self::Int(v) => {
                rad = (*v as f64).atan();
            },
            Self::Units(v, _) => {
                rad = v.atan();
            }
        }
        *self = Self::Units(rad, Units::Radians);
        Ok(())
    }

    /// sinh.
    pub fn sinh(&mut self) -> Result<(), Error> {
        match &mut *self {
            Self::Float(v) => {
                *v = v.sinh();
            },
            Self::Int(v) => {
                *self = Self::Float((*v as f64).sinh());
            },
            Self::Units(v, _) => {
                *v = v.sinh();
            }
        }
        Ok(())
    }

    /// Cosh.
    pub fn cosh(&mut self) -> Result<(), Error> {
        match &mut *self {
            Self::Float(v) => {
                *v = v.cosh();
            },
            Self::Int(v) => {
                *self = Self::Float((*v as f64).cosh());
            },
            Self::Units(v, _) => {
                *v = v.cosh();
            }
        }
        Ok(())
    }

    /// Tanh.
    pub fn tanh(&mut self) -> Result<(), Error> {
        match &mut *self {
            Self::Float(v) => {
                *v = v.tanh();
            },
            Self::Int(v) => {
                *self = Self::Float((*v as f64).tanh());
            },
            Self::Units(v, _) => {
                *v = v.tanh();
            }
        }
        Ok(())
    }

    /// ASinH.
    pub fn asinh(&mut self) -> Result<(), Error> {
        match &mut *self {
            Self::Float(v) => {
                *v = v.asinh();
            },
            Self::Int(v) => {
                *self = Self::Float((*v as f64).asinh());
            },
            Self::Units(v, _) => {
                *v = v.asinh();
            }
        }
        Ok(())
    }

    /// ACosH.
    pub fn acosh(&mut self) -> Result<(), Error> {
        match &mut *self {
            Self::Float(v) => {
                *v = v.acosh();
            },
            Self::Int(v) => {
                *self = Self::Float((*v as f64).acosh());
            },
            Self::Units(v, _) => {
                *v = v.acosh();
            }
        }
        Ok(())
    }

    /// ATanH.
    pub fn atanh(&mut self) -> Result<(), Error> {
        match &mut *self {
            Self::Float(v) => {
                *v = v.atanh();
            },
            Self::Int(v) => {
                *self = Self::Float((*v as f64).atanh());
            },
            Self::Units(v, _) => {
                *v = v.atanh();
            }
        }
        Ok(())
    }
    
    /// Round.
    pub fn round(&mut self) -> Result<(), Error> {
        match &mut *self {
            Self::Float(v) => {
                *v = v.round();
            },
            Self::Int(_v) => {
                // rounded
            },
            Self::Units(v, _) => {
                *v = v.round();
            }
        }
        Ok(())
    }

    /// Round 2.
    pub fn round2(&mut self, digits: &Self) -> Result<(), Error> {
        match &mut *self {
            Self::Float(v) => {
                let digits = digits.int();
                if digits > 0 {
                    let mut scale = 1;
                    for _ in 0..digits {
                        scale *= 10;
                    }
                    *v = (*v * scale as f64).round()/(scale as f64);
                } else {
                    *v = v.round();
                }
            },
            Self::Int(_v) => {
                // rounded
            },
            Self::Units(v, _) => {
                let digits = digits.int();
                if digits > 0 {
                    let mut scale = 1;
                    for _ in 0..digits {
                        scale *= 10;
                    }
                    *v = (*v * scale as f64).round()/(scale as f64);
                } else {
                    *v = v.round();
                }
            }
        }
        Ok(())
    }

    /// Pow.
    pub fn pow(&mut self, to: &Self) -> Result<(), Error> {
        match &mut *self {
            Self::Float(v) => {
                *v = v.powf(to.float(None));
            },
            Self::Int(v) => {
                *self = Self::Float((*v as f64).powf(to.float(None)));
            },
            Self::Units(v, _) => {
                *v = v.powf(to.float(None));
            }
        }
        Ok(())
    }

    /// Log.
    pub fn log(&mut self, base: &Self) -> Result<(), Error> {
        match &mut *self {
            Self::Float(v) => {
                *v = v.log(base.float(None));
            },
            Self::Int(v) => {
                *self = Self::Float((*v as f64).log(base.float(None)));
            },
            Self::Units(v, _) => {
                *v = v.log(base.float(None));
            }
        }
        Ok(())
    }

    /// ATan2.
    pub fn atan2(&mut self, base: &Self) -> Result<(), Error> {
        let rad;
        match &mut *self {
            Self::Float(v) => {
                rad = v.atan2(base.float(None));
            },
            Self::Int(v) => {
                rad = (*v as f64).atan2(base.float(None));
            },
            Self::Units(v, _) => {
                rad = v.atan2(base.float(None));
            }
        }
        *self = Self::Units(rad, Units::Radians);
        Ok(())
    }
}
