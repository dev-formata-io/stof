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

use std::sync::Arc;
use imbl::vector;
use crate::{model::{num::{ABS, ACOS, ACOSH, ASIN, ASINH, ATAN, ATAN2, ATANH, BIN, CBRT, CEIL, COS, COSH, EXP, EXP2, FLOOR, FRACT, HAS_UNITS, HEX, INF, IS_ANGLE, IS_LENGTH, IS_MASS, IS_TEMP, IS_TIME, LN, LOG, NAN, NUM_LIB, OCT, POW, REMOVE_UNITS, ROUND2, SIGNUM, SIN, SINH, SQRT, STRING, TAN, TANH, TRUNC}, LibFunc, Param}, runtime::{instruction::Instructions, instructions::Base, Num, Type, Val}};


/// Absolute value library function.
pub fn num_abs() -> LibFunc {
    LibFunc {
        library: NUM_LIB.clone(),
        name: "abs".into(),
        is_async: false,
        docs: "# Absolute Value\nReturn the absolute value of a number.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(ABS.clone());
            Ok(instructions)
        })
    }
}

/// Sqrt.
pub fn num_sqrt() -> LibFunc {
    LibFunc {
        library: NUM_LIB.clone(),
        name: "sqrt".into(),
        is_async: false,
        docs: "# Square Root\nReturn the square root of a number.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(SQRT.clone());
            Ok(instructions)
        })
    }
}

/// Cbrt.
pub fn num_cbrt() -> LibFunc {
    LibFunc {
        library: NUM_LIB.clone(),
        name: "cbrt".into(),
        is_async: false,
        docs: "# Cube Root\nReturn the cube root of a number.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(CBRT.clone());
            Ok(instructions)
        })
    }
}

/// Floor.
pub fn num_floor() -> LibFunc {
    LibFunc {
        library: NUM_LIB.clone(),
        name: "floor".into(),
        is_async: false,
        docs: "# Floor\nReturn the largest integer less than or equal to self.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(FLOOR.clone());
            Ok(instructions)
        })
    }
}

/// Ceil.
pub fn num_ceil() -> LibFunc {
    LibFunc {
        library: NUM_LIB.clone(),
        name: "ceil".into(),
        is_async: false,
        docs: "# Ceil\nReturn the smallest integer greater than or equal to self.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(CEIL.clone());
            Ok(instructions)
        })
    }
}

/// Trunc.
pub fn num_trunc() -> LibFunc {
    LibFunc {
        library: NUM_LIB.clone(),
        name: "trunc".into(),
        is_async: false,
        docs: "# Trunc\nReturn the integer part of self.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(TRUNC.clone());
            Ok(instructions)
        })
    }
}

/// Fract.
pub fn num_fract() -> LibFunc {
    LibFunc {
        library: NUM_LIB.clone(),
        name: "fract".into(),
        is_async: false,
        docs: "# Fract\nReturn the fractional part of self.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(FRACT.clone());
            Ok(instructions)
        })
    }
}

/// Signum.
pub fn num_signum() -> LibFunc {
    LibFunc {
        library: NUM_LIB.clone(),
        name: "signum".into(),
        is_async: false,
        docs: "# Sign Number\nReturn the sign number of self (-1 or 1).".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(SIGNUM.clone());
            Ok(instructions)
        })
    }
}

/// Exp.
pub fn num_exp() -> LibFunc {
    LibFunc {
        library: NUM_LIB.clone(),
        name: "exp".into(),
        is_async: false,
        docs: "# Exponential Function\ne^(self).".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(EXP.clone());
            Ok(instructions)
        })
    }
}

/// Exp2.
pub fn num_exp2() -> LibFunc {
    LibFunc {
        library: NUM_LIB.clone(),
        name: "exp2".into(),
        is_async: false,
        docs: "# Exponential 2\n2^(self).".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(EXP2.clone());
            Ok(instructions)
        })
    }
}

/// Ln.
pub fn num_ln() -> LibFunc {
    LibFunc {
        library: NUM_LIB.clone(),
        name: "ln".into(),
        is_async: false,
        docs: "# Natural Log\nln(self).".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(LN.clone());
            Ok(instructions)
        })
    }
}

/// NaN?
pub fn num_nan() -> LibFunc {
    LibFunc {
        library: NUM_LIB.clone(),
        name: "nan".into(),
        is_async: false,
        docs: "# Not a Number?\nReturns true if this is value is NaN.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(NAN.clone());
            Ok(instructions)
        })
    }
}

/// Inf?
pub fn num_inf() -> LibFunc {
    LibFunc {
        library: NUM_LIB.clone(),
        name: "inf".into(),
        is_async: false,
        docs: "# Infinity?\nReturns true if this number is infinity.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(INF.clone());
            Ok(instructions)
        })
    }
}

/// Sin.
pub fn num_sin() -> LibFunc {
    LibFunc {
        library: NUM_LIB.clone(),
        name: "sin".into(),
        is_async: false,
        docs: "# Sine\nSine function.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(SIN.clone());
            Ok(instructions)
        })
    }
}

/// Cos.
pub fn num_cos() -> LibFunc {
    LibFunc {
        library: NUM_LIB.clone(),
        name: "cos".into(),
        is_async: false,
        docs: "# Cosine\nCosine function.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(COS.clone());
            Ok(instructions)
        })
    }
}

/// Tan.
pub fn num_tan() -> LibFunc {
    LibFunc {
        library: NUM_LIB.clone(),
        name: "tan".into(),
        is_async: false,
        docs: "# Tangent\nTangent function.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(TAN.clone());
            Ok(instructions)
        })
    }
}

/// ASin.
pub fn num_asin() -> LibFunc {
    LibFunc {
        library: NUM_LIB.clone(),
        name: "asin".into(),
        is_async: false,
        docs: "# Arc Sine\nASine function (return Radians).".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(ASIN.clone());
            Ok(instructions)
        })
    }
}

/// ACos.
pub fn num_acos() -> LibFunc {
    LibFunc {
        library: NUM_LIB.clone(),
        name: "acos".into(),
        is_async: false,
        docs: "# Arc Cosine\nACosine function (returns Radians).".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(ACOS.clone());
            Ok(instructions)
        })
    }
}

/// ATan.
pub fn num_atan() -> LibFunc {
    LibFunc {
        library: NUM_LIB.clone(),
        name: "atan".into(),
        is_async: false,
        docs: "# Arc Tangent\nATangent function (return Radians).".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(ATAN.clone());
            Ok(instructions)
        })
    }
}

/// SinH.
pub fn num_sinh() -> LibFunc {
    LibFunc {
        library: NUM_LIB.clone(),
        name: "sinh".into(),
        is_async: false,
        docs: "# Hyperbolic Sine\nSinH function.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(SINH.clone());
            Ok(instructions)
        })
    }
}

/// CosH.
pub fn num_cosh() -> LibFunc {
    LibFunc {
        library: NUM_LIB.clone(),
        name: "cosh".into(),
        is_async: false,
        docs: "# Hyperbolic Cosine\nCosH function.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(COSH.clone());
            Ok(instructions)
        })
    }
}

/// TanH.
pub fn num_tanh() -> LibFunc {
    LibFunc {
        library: NUM_LIB.clone(),
        name: "tanh".into(),
        is_async: false,
        docs: "# Hyperbolic Tangent\nTanH function.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(TANH.clone());
            Ok(instructions)
        })
    }
}

/// ASinH.
pub fn num_asinh() -> LibFunc {
    LibFunc {
        library: NUM_LIB.clone(),
        name: "asinh".into(),
        is_async: false,
        docs: "# Inverse Hyperbolic Sine\nASinH function.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(ASINH.clone());
            Ok(instructions)
        })
    }
}

/// ACosH.
pub fn num_acosh() -> LibFunc {
    LibFunc {
        library: NUM_LIB.clone(),
        name: "acosh".into(),
        is_async: false,
        docs: "# Inverse Hyperbolic Cosine\nACosH function.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(ACOSH.clone());
            Ok(instructions)
        })
    }
}

/// ATanH.
pub fn num_atanh() -> LibFunc {
    LibFunc {
        library: NUM_LIB.clone(),
        name: "atanh".into(),
        is_async: false,
        docs: "# Inverse Hyperbolic Tangent\nATanH function.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(ATANH.clone());
            Ok(instructions)
        })
    }
}

/// Hex string.
pub fn num_hex() -> LibFunc {
    LibFunc {
        library: NUM_LIB.clone(),
        name: "hex".into(),
        is_async: false,
        docs: "# Hex String\nReturn this numbers hex string (integer).".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(HEX.clone());
            Ok(instructions)
        })
    }
}

/// Binary string.
pub fn num_bin() -> LibFunc {
    LibFunc {
        library: NUM_LIB.clone(),
        name: "bin".into(),
        is_async: false,
        docs: "# Binary String\nReturn this numbers binary string (integer).".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(BIN.clone());
            Ok(instructions)
        })
    }
}

/// Oct string.
pub fn num_oct() -> LibFunc {
    LibFunc {
        library: NUM_LIB.clone(),
        name: "oct".into(),
        is_async: false,
        docs: "# Oct String\nReturn this numbers octal string (integer).".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(OCT.clone());
            Ok(instructions)
        })
    }
}

/// To string.
pub fn num_string() -> LibFunc {
    LibFunc {
        library: NUM_LIB.clone(),
        name: "to_string".into(),
        is_async: false,
        docs: "# To String\nReturn this number as a string (like print).".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(STRING.clone());
            Ok(instructions)
        })
    }
}

/// Has units?
pub fn num_has_units() -> LibFunc {
    LibFunc {
        library: NUM_LIB.clone(),
        name: "has_units".into(),
        is_async: false,
        docs: "# Has Units?\nReturn true if this number has units.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(HAS_UNITS.clone());
            Ok(instructions)
        })
    }
}

/// Remove units.
pub fn num_remove_units() -> LibFunc {
    LibFunc {
        library: NUM_LIB.clone(),
        name: "remove_units".into(),
        is_async: false,
        docs: "# Remove Units\nRemove units if this number has any.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(REMOVE_UNITS.clone());
            Ok(instructions)
        })
    }
}

/// Is angle?
pub fn num_is_angle() -> LibFunc {
    LibFunc {
        library: NUM_LIB.clone(),
        name: "is_angle".into(),
        is_async: false,
        docs: "# Is Angle?\nReturn true if this number has angle units.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(IS_ANGLE.clone());
            Ok(instructions)
        })
    }
}

/// Is temperature?
pub fn num_is_temp() -> LibFunc {
    LibFunc {
        library: NUM_LIB.clone(),
        name: "is_temp".into(),
        is_async: false,
        docs: "# Is Temperature?\nReturn true if this number has temperature units.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(IS_TEMP.clone());
            Ok(instructions)
        })
    }
}

/// Is length?
pub fn num_is_length() -> LibFunc {
    LibFunc {
        library: NUM_LIB.clone(),
        name: "is_length".into(),
        is_async: false,
        docs: "# Is Length?\nReturn true if this number has units of length.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(IS_LENGTH.clone());
            Ok(instructions)
        })
    }
}

/// Is time?
pub fn num_is_time() -> LibFunc {
    LibFunc {
        library: NUM_LIB.clone(),
        name: "is_time".into(),
        is_async: false,
        docs: "# Is Time?\nReturn true if this number has units of time.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(IS_TIME.clone());
            Ok(instructions)
        })
    }
}

/// Is mass?
pub fn num_is_mass() -> LibFunc {
    LibFunc {
        library: NUM_LIB.clone(),
        name: "is_mass".into(),
        is_async: false,
        docs: "# Is Mass?\nReturn true if this number has units of mass.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(IS_MASS.clone());
            Ok(instructions)
        })
    }
}

/// Round.
pub fn num_round() -> LibFunc {
    LibFunc {
        library: NUM_LIB.clone(),
        name: "round".into(),
        is_async: false,
        docs: "# Round\nRound this number, optionally specifying the number of places to be rounded to.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None },
            Param { name: "places".into(), param_type: Type::Void, default: Some(Arc::new(Base::Literal(Val::Num(Num::Int(0))))) }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(ROUND2.clone());
            Ok(instructions)
        })
    }
}

/// Pow.
pub fn num_pow() -> LibFunc {
    LibFunc {
        library: NUM_LIB.clone(),
        name: "pow".into(),
        is_async: false,
        docs: "# Power\nRaise this number to a power (default is to square).".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None },
            Param { name: "to".into(), param_type: Type::Void, default: Some(Arc::new(Base::Literal(Val::Num(Num::Int(2))))) }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(POW.clone());
            Ok(instructions)
        })
    }
}

/// Log.
pub fn num_log() -> LibFunc {
    LibFunc {
        library: NUM_LIB.clone(),
        name: "log".into(),
        is_async: false,
        docs: "# Log\nLog function with a base (default of 10).".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None },
            Param { name: "base".into(), param_type: Type::Void, default: Some(Arc::new(Base::Literal(Val::Num(Num::Int(10))))) }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(LOG.clone());
            Ok(instructions)
        })
    }
}

/// ATan2.
pub fn num_atan2() -> LibFunc {
    LibFunc {
        library: NUM_LIB.clone(),
        name: "atan2".into(),
        is_async: false,
        docs: "# ATan2 Function\nATan2 function.".into(),
        params: vector![
            Param { name: "y".into(), param_type: Type::Void, default: None },
            Param { name: "x".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(ATAN2.clone());
            Ok(instructions)
        })
    }
}
