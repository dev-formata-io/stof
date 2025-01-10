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

use std::ops::DerefMut;
use anyhow::{anyhow, Result};
use crate::{SDoc, Library, SNum, SUnits, SVal};


/// Number library.
#[derive(Default, Debug)]
pub struct NumberLibrary;
impl Library for NumberLibrary {
    /// Scope.
    fn scope(&self) -> String {
        "Number".to_string()
    }
    
    /// Call into the Number library.
    fn call(&self, pid: &str, doc: &mut SDoc, name: &str, parameters: &mut Vec<SVal>) -> Result<SVal> {
        if parameters.len() > 0 {
            match name {
                "toString" => {
                    return Ok(SVal::String(parameters[0].print(doc)));
                },
                "or" => {
                    for param in parameters.drain(..) {
                        if !param.is_empty() {
                            return Ok(param);
                        }
                    }
                    return Ok(SVal::Null);
                },
                _ => {}
            }

            let mut params;
            if parameters.len() > 1 {
                params = parameters.drain(1..).collect();
            } else {
                params = Vec::new();
            }
            match &mut parameters[0] {
                SVal::Number(num) => {
                    return self.operate(pid, doc, name, num, &mut params);
                },
                SVal::Boxed(val) => {
                    let mut val = val.lock().unwrap();
                    let val = val.deref_mut();
                    match val {
                        SVal::Number(num) => {
                            return self.operate(pid, doc, name, num, &mut params);
                        },
                        _ => {
                            return Err(anyhow!("Number library requires the first parameter to be a number"));
                        }
                    }
                },
                _ => {
                    return Err(anyhow!("Number library requires the first parameter to be a number"));
                }
            }
        } else {
            return Err(anyhow!("Number library requires a number parameter to work with"));
        }
    }
}
impl NumberLibrary {
    /// Call number operation.
    pub fn operate(&self, _pid: &str, _doc: &mut SDoc, name: &str, nval: &mut SNum, parameters: &mut Vec<SVal>) -> Result<SVal> {
        match name {
            "len" => return Ok(SVal::Number(nval.int().into())),
            "units" => return Self::units(nval),
            "removeUnits" => return Self::remove_units(nval),
            "hasUnits" | "isUnits" => return Self::has_units(nval),
            "isAngle" => return Self::is_angle(nval),
            "isDegrees" => return Self::is_degrees(nval),
            "isPositiveDegrees" => return Self::is_pdegrees(nval),
            "isRadians" => return Self::is_radians(nval),
            "isPositiveRadians" => return Self::is_pradians(nval),
            "isTemperature" | "isTemp" => return Self::is_temp(nval),
            "isLength" => return Self::is_length(nval),
            "isTime" => return Self::is_time(nval),
            "isMass" => return Self::is_mass(nval),
            "sqrt" => return Self::sqrt(nval),
            "cbrt" => return Self::cbrt(nval),
            "abs" => return Self::abs(nval),
            "floor" => return Self::floor(nval),
            "ceil" => return Self::ceil(nval),
            "trunc" => return Self::trunc(nval),
            "fract" => return Self::fract(nval),
            "signum" => return Self::signum(nval),
            "exp" => return Self::exp(nval),
            "exp2" => return Self::exp2(nval),
            "ln" => return Self::ln(nval),
            "sin" => return Self::sin(nval),
            "cos" => return Self::cos(nval),
            "tan" => return Self::tan(nval),
            "asin" => return Self::asin(nval),
            "acos" => return Self::acos(nval),
            "atan" => return Self::atan(nval),
            "sinh" => return Self::sinh(nval),
            "cosh" => return Self::cosh(nval),
            "tanh" => return Self::tanh(nval),
            "asinh" => return Self::asinh(nval),
            "acosh" => return Self::acosh(nval),
            "atanh" => return Self::atanh(nval),
            "pow" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("Number.pow(num, num) requires 2 number parameters"));
                }
                let power = parameters.pop().unwrap().unbox();
                match &power {
                    SVal::Number(second) => {
                        Self::pow(nval, second)
                    },
                    _ => {
                        Err(anyhow!("Number.pow(num, num) must have a number as the second parameter"))
                    }
                }
            },
            "log" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("Number.log(num, num) requires 2 number parameters"));
                }
                let base = parameters.pop().unwrap().unbox();
                match &base {
                    SVal::Number(second) => {
                        Self::log(nval, second)
                    },
                    _ => {
                        Err(anyhow!("Number.log(num, num) must have a number as the second parameter"))
                    }
                }
            },
            "atan2" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("Number.atan2(num, num) requires 2 number parameters"));
                }
                let second = parameters.pop().unwrap().unbox();
                match &second {
                    SVal::Number(second) => {
                        Self::atan2(nval, second)
                    },
                    _ => {
                        Err(anyhow!("Number.atan2(num, num) must have a number as the second parameter"))
                    }
                }
            },
            "round" => {
                if parameters.len() < 1 {
                    return Self::round(nval);
                }
                let second = parameters.pop().unwrap().unbox();
                match &second {
                    SVal::Number(second) => {
                        Self::round2(nval, second)
                    },
                    _ => {
                        Err(anyhow!("Number.round(num, num) must have a number as the second parameter"))
                    }
                }
            },
            "at" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("Number.at(num, num) requires 2 number parameters"));
                }
                let second = parameters.pop().unwrap().unbox();
                match &second {
                    SVal::Number(second) => {
                        let first = nval.int();
                        let second = second.int();
                        if second < first {
                            Ok(SVal::Number(second.into()))
                        } else {
                            Ok(SVal::Number(first.into()))
                        }
                    },
                    _ => {
                        Err(anyhow!("Number.at(num, num) must have a number as the second parameter"))
                    }
                }
            },
            _ => {
                Err(anyhow!("Did not find the requested Number library function '{}'", name))
            }
        }
    }

    /// Units.
    /// Returns the string version of the units on this number, or null if no units are defined.
    pub fn units(number: &SNum) -> Result<SVal> {
        match number {
            SNum::Units(_, units) => Ok(SVal::String(units.to_string())),
            _ => Ok(SVal::Null)
        }
    }

    /// Remove units.
    pub fn remove_units(number: &SNum) -> Result<SVal> {
        match number {
            SNum::Units(val, _) => Ok(SVal::Number(SNum::F64(*val))),
            _ => Ok(SVal::Number(number.clone()))
        }
    }

    /// Has units?
    pub fn has_units(number: &SNum) -> Result<SVal> {
        match number {
            SNum::Units(_, units) => Ok(SVal::Bool(units.has_units())),
            _ => Ok(SVal::Bool(false))
        }
    }

    /// Is angle units?
    pub fn is_angle(number: &SNum) -> Result<SVal> {
        match number {
            SNum::Units(_, units) => Ok(SVal::Bool(units.is_angle())),
            _ => Ok(SVal::Bool(false)),
        }
    }

    /// Is degrees units?
    pub fn is_degrees(number: &SNum) -> Result<SVal> {
        match number {
            SNum::Units(_, units) => Ok(SVal::Bool(units.is_degrees())),
            _ => Ok(SVal::Bool(false)),
        }
    }

    /// Is radians units?
    pub fn is_radians(number: &SNum) -> Result<SVal> {
        match number {
            SNum::Units(_, units) => Ok(SVal::Bool(units.is_radians())),
            _ => Ok(SVal::Bool(false)),
        }
    }

    /// Is positive degrees units?
    pub fn is_pdegrees(number: &SNum) -> Result<SVal> {
        match number {
            SNum::Units(_, units) => Ok(SVal::Bool(units.is_degrees() && units.is_positive_angle())),
            _ => Ok(SVal::Bool(false)),
        }
    }

    /// Is positive radians units?
    pub fn is_pradians(number: &SNum) -> Result<SVal> {
        match number {
            SNum::Units(_, units) => Ok(SVal::Bool(units.is_radians() && units.is_positive_angle())),
            _ => Ok(SVal::Bool(false)),
        }
    }

    /// Is mass units?
    pub fn is_mass(number: &SNum) -> Result<SVal> {
        match number {
            SNum::Units(_, units) => Ok(SVal::Bool(units.is_mass())),
            _ => Ok(SVal::Bool(false))
        }
    }

    /// Is time units?
    pub fn is_time(number: &SNum) -> Result<SVal> {
        match number {
            SNum::Units(_, units) => Ok(SVal::Bool(units.is_time())),
            _ => Ok(SVal::Bool(false))
        }
    }

    /// Is temperature units?
    pub fn is_temp(number: &SNum) -> Result<SVal> {
        match number {
            SNum::Units(_, units) => Ok(SVal::Bool(units.is_temperature())),
            _ => Ok(SVal::Bool(false))
        }
    }

    /// Is length units?
    pub fn is_length(number: &SNum) -> Result<SVal> {
        match number {
            SNum::Units(_, units) => Ok(SVal::Bool(units.is_length())),
            _ => Ok(SVal::Bool(false))
        }
    }

    /// Raise a number to a power.
    pub fn pow(number: &SNum, power: &SNum) -> Result<SVal> {
        if let Some(units) = number.get_units() {
            return Ok(SVal::Number(SNum::Units(number.float().powf(power.float()), units)));
        }
        Ok(SVal::Number(SNum::F64(number.float().powf(power.float()))))
    }

    /// Sqrt.
    pub fn sqrt(number: &SNum) -> Result<SVal> {
        if let Some(units) = number.get_units() {
            return Ok(SVal::Number(SNum::Units(number.float().sqrt(), units)));
        }
        Ok(SVal::Number(SNum::F64(number.float().sqrt())))
    }

    /// Cbrt (cube root).
    pub fn cbrt(number: &SNum) -> Result<SVal> {
        if let Some(units) = number.get_units() {
            return Ok(SVal::Number(SNum::Units(number.float().cbrt(), units)));
        }
        Ok(SVal::Number(SNum::F64(number.float().cbrt())))
    }

    /// Abs.
    pub fn abs(number: &SNum) -> Result<SVal> {
        if let Some(units) = number.get_units() {
            return Ok(SVal::Number(SNum::Units(number.float().abs(), units)));
        }
        Ok(SVal::Number(SNum::F64(number.float().abs())))
    }

    /// Floor.
    pub fn floor(number: &SNum) -> Result<SVal> {
        if let Some(units) = number.get_units() {
            return Ok(SVal::Number(SNum::Units(number.float().floor(), units)));
        }
        Ok(SVal::Number(SNum::F64(number.float().floor())))
    }

    /// Ceil.
    pub fn ceil(number: &SNum) -> Result<SVal> {
        if let Some(units) = number.get_units() {
            return Ok(SVal::Number(SNum::Units(number.float().ceil(), units)));
        }
        Ok(SVal::Number(SNum::F64(number.float().ceil())))
    }

    /// Round.
    pub fn round(number: &SNum) -> Result<SVal> {
        if let Some(units) = number.get_units() {
            return Ok(SVal::Number(SNum::Units(number.float().round(), units)));
        }
        Ok(SVal::Number(SNum::F64(number.float().round())))
    }

    /// Round2.
    /// Round to places after the decimal point.
    pub fn round2(number: &SNum, digits: &SNum) -> Result<SVal> {
        let digits = digits.int();
        let mut float = number.float();

        if digits > 0 {
            let mut scale = 1;
            for _ in 0..digits {
                scale *= 10;
            }
            float = (float * scale as f64).round()/(scale as f64);
        }

        if let Some(units) = number.get_units() {
            return Ok(SVal::Number(SNum::Units(float, units)));
        }
        Ok(SVal::Number(SNum::F64(float)))
    }

    /// Trunc.
    pub fn trunc(number: &SNum) -> Result<SVal> {
        if let Some(units) = number.get_units() {
            return Ok(SVal::Number(SNum::Units(number.float().trunc(), units)));
        }
        Ok(SVal::Number(SNum::F64(number.float().trunc())))
    }

    /// Fract.
    /// Returns the fractional part of self.
    pub fn fract(number: &SNum) -> Result<SVal> {
        if let Some(units) = number.get_units() {
            return Ok(SVal::Number(SNum::Units(number.float().fract(), units)));
        }
        Ok(SVal::Number(SNum::F64(number.float().fract())))
    }

    /// Signum.
    /// The number that represents the sign of self.
    pub fn signum(number: &SNum) -> Result<SVal> {
        Ok(SVal::Number(SNum::F64(number.float().signum())))
    }

    /// Exp.
    /// e^(self).
    pub fn exp(number: &SNum) -> Result<SVal> {
        if let Some(units) = number.get_units() {
            return Ok(SVal::Number(SNum::Units(number.float().exp(), units)));
        }
        Ok(SVal::Number(SNum::F64(number.float().exp())))
    }

    /// Exp2.
    /// 2^(self)
    pub fn exp2(number: &SNum) -> Result<SVal> {
        if let Some(units) = number.get_units() {
            return Ok(SVal::Number(SNum::Units(number.float().exp2(), units)));
        }
        Ok(SVal::Number(SNum::F64(number.float().exp2())))
    }

    /// Ln.
    /// Natural log of self (ln(self)).
    pub fn ln(number: &SNum) -> Result<SVal> {
        if let Some(units) = number.get_units() {
            return Ok(SVal::Number(SNum::Units(number.float().ln(), units)));
        }
        Ok(SVal::Number(SNum::F64(number.float().ln())))
    }

    /// Log with a base.
    pub fn log(number: &SNum, base: &SNum) -> Result<SVal> {
        if let Some(units) = number.get_units() {
            return Ok(SVal::Number(SNum::Units(number.float().log(base.float()), units)));
        }
        Ok(SVal::Number(SNum::F64(number.float().log(base.float()))))
    }

    /// Sin in radians.
    pub fn sin(number: &SNum) -> Result<SVal> {
        if let Some(units) = number.get_units() {
            let mut val = number.float(); // assumes number is in rads
            if units.is_degrees() {
                val = SUnits::to_radians(val, SUnits::Degrees);
            }
            // Sine function removes units (angle -> length)
            return Ok(SVal::Number(SNum::F64(val.sin())));
        }
        Ok(SVal::Number(SNum::F64(number.float().sin())))
    }

    /// Cos in radians.
    pub fn cos(number: &SNum) -> Result<SVal> {
        if let Some(units) = number.get_units() {
            let mut val = number.float();
            if units.is_degrees() {
                val = SUnits::to_radians(val, SUnits::Degrees);
            }
            return Ok(SVal::Number(SNum::F64(val.cos())));
        }
        Ok(SVal::Number(SNum::F64(number.float().cos())))
    }

    /// Tan in radians.
    pub fn tan(number: &SNum) -> Result<SVal> {
        if let Some(units) = number.get_units() {
            let mut val = number.float();
            if units.is_degrees() {
                val = SUnits::to_radians(val, SUnits::Degrees);
            }
            return Ok(SVal::Number(SNum::F64(val.tan())));
        }
        Ok(SVal::Number(SNum::F64(number.float().tan())))
    }

    /// Arcsin in radians.
    pub fn asin(number: &SNum) -> Result<SVal> {
        let val = number.float().asin();
        if val.is_nan() || val.is_infinite() {
            return Ok(SVal::Null);
        }
        Ok(SVal::Number(SNum::Units(val, SUnits::Radians)))
    }

    /// Arccos in radians.
    pub fn acos(number: &SNum) -> Result<SVal> {
        let val = number.float().acos();
        if val.is_nan() || val.is_infinite() {
            return Ok(SVal::Null);
        }
        Ok(SVal::Number(SNum::Units(val, SUnits::Radians)))
    }

    /// Arctan in radians.
    pub fn atan(number: &SNum) -> Result<SVal> {
        let val = number.float().atan();
        if val.is_nan() || val.is_infinite() {
            return Ok(SVal::Null);
        }
        Ok(SVal::Number(SNum::Units(val, SUnits::Radians)))
    }

    /// Atan2.
    pub fn atan2(number: &SNum, other: &SNum) -> Result<SVal> {
        let val = number.float().atan2(other.float());
        if val.is_nan() || val.is_infinite() {
            return Ok(SVal::Null);
        }
        Ok(SVal::Number(SNum::Units(val, SUnits::Radians)))
    }

    /// Sinh.
    pub fn sinh(number: &SNum) -> Result<SVal> {
        if let Some(units) = number.get_units() {
            return Ok(SVal::Number(SNum::Units(number.float().sinh(), units)));
        }
        Ok(SVal::Number(SNum::F64(number.float().sinh())))
    }

    /// Cosh.
    pub fn cosh(number: &SNum) -> Result<SVal> {
        if let Some(units) = number.get_units() {
            return Ok(SVal::Number(SNum::Units(number.float().cosh(), units)));
        }
        Ok(SVal::Number(SNum::F64(number.float().cosh())))
    }

    /// Tanh.
    pub fn tanh(number: &SNum) -> Result<SVal> {
        if let Some(units) = number.get_units() {
            return Ok(SVal::Number(SNum::Units(number.float().tanh(), units)));
        }
        Ok(SVal::Number(SNum::F64(number.float().tanh())))
    }

    /// Asinh.
    pub fn asinh(number: &SNum) -> Result<SVal> {
        if let Some(units) = number.get_units() {
            return Ok(SVal::Number(SNum::Units(number.float().asinh(), units)));
        }
        Ok(SVal::Number(SNum::F64(number.float().asinh())))
    }

    /// Acosh.
    pub fn acosh(number: &SNum) -> Result<SVal> {
        if let Some(units) = number.get_units() {
            return Ok(SVal::Number(SNum::Units(number.float().acosh(), units)));
        }
        Ok(SVal::Number(SNum::F64(number.float().acosh())))
    }

    /// Atanh.
    pub fn atanh(number: &SNum) -> Result<SVal> {
        if let Some(units) = number.get_units() {
            return Ok(SVal::Number(SNum::Units(number.float().atanh(), units)));
        }
        Ok(SVal::Number(SNum::F64(number.float().atanh())))
    }
}
