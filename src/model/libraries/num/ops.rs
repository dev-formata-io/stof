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
        docs: r#"# Num.abs(val: int | float) -> int | float
Return the absolute value of the given number.
```rust
const v = -2;
assert_eq(v.abs(), 2);
```"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
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
        docs: r#"# Num.sqrt(val: int | float) -> float
Return the square root of a number.
```rust
const v = 4;
assert_eq(v.sqrt(), 2);
```"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
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
        docs: r#"# Num.cbrt(val: int | float) -> float
Return the cube root of a number.
```rust
const v = 8;
assert_eq(v.cbrt(), 2);
```"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
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
        docs: r#"# Num.floor(val: int | float) -> int | float
Return the largest integer less than or equal to the given value.
```rust
const v = 2.4;
assert_eq(v.floor(), 2);
```"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
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
        docs: r#"# Num.ceil(val: int | float) -> int | float
Return the smallest integer greater than or equal to the given value.
```rust
const v = 2.4;
assert_eq(v.ceil(), 3);
```"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
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
        docs: r#"# Num.trunc(val: int | float) -> int | float
Return the integer part of the given value.
```rust
const v = 2.4;
assert_eq(v.trunc(), 2);
```"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
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
        docs: r#"# Num.fract(val: int | float) -> int | float
Return the fractional part of this number.
```rust
const v = 2.4;
assert_eq(v.trunc(), 0.4);
```"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
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
        docs: r#"# Num.signum(val: int | float) -> int | float
Return a number representing the sign of this value (-1 or 1).
```rust
assert_eq((42).signum(), 1);
assert_eq((-42).signum(), -1);
```"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
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
        docs: r#"# Num.exp(val: int | float) -> float
Exponential function (e^(val)).
```rust
assert_eq((1).exp().round(3), 2.718);
```"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
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
        docs: r#"# Num.exp2(val: int | float) -> float
Exponential 2 function (2^(val)).
```rust
assert_eq((2).exp2(), 4);
```"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
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
        docs: r#"# Num.ln(val: int | float) -> float
Natural log.
```rust
assert_eq((1).ln(), 0);
```"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
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
        docs: r#"# Num.nan(val: int | float) -> bool
Return true if this value is NaN.
```rust
assert_not((14).nan());
```"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
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
        docs: r#"# Num.inf(val: int | float) -> bool
Return true if this value is infinity.
```rust
assert_not((14).inf());
```"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
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
        docs: r#"# Num.sin(val: int | float) -> float
Sine function.
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
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
        docs: r#"# Num.cos(val: int | float) -> float
Cosine function.
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
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
        docs: r#"# Num.tan(val: int | float) -> float
Tangent function.
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
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
        docs: r#"# Num.asin(val: int | float) -> rad
Arc Sine function (returns a float with radian units).
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
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
        docs: r#"# Num.acos(val: int | float) -> rad
Arc Cosine function (returns a float with radian units).
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
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
        docs: r#"# Num.atan(val: int | float) -> rad
Arc Tangent function (returns a float with radian units).
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
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
        docs: r#"# Num.sinh(val: int | float) -> float
Hyperbolic Sine function.
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
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
        docs: r#"# Num.cosh(val: int | float) -> float
Hyperbolic Cosine function.
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
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
        docs: r#"# Num.tanh(val: int | float) -> float
Hyperbolic Tangent function.
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
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
        docs: r#"# Num.asinh(val: int | float) -> float
Inverse hyperbolic Sine function.
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
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
        docs: r#"# Num.acosh(val: int | float) -> float
Inverse hyperbolic Cosine function.
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
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
        docs: r#"# Num.atanh(val: int | float) -> float
Inverse hyperbolic Tangent function.
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
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
        docs: r#"# Num.hex(val: int) -> str
Returns this number represented as a hexidecimal string.
```rust
assert_eq((10).hex(), "A");
```
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
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
        docs: r#"# Num.bin(val: int) -> str
Returns this number represented as a binary string.
```rust
assert_eq((10).bin(), "1010");
```
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
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
        docs: r#"# Num.oct(val: int) -> str
Returns this number represented as an octal string.
```rust
assert_eq((10).oct(), "12");
```
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
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
        docs: r#"# Num.to_string(val: int | float) -> str
Returns this number represented as a string (like print).
```rust
assert_eq((10).to_string(), "10");
assert_eq(str(10), "10"); // prefer Std.str(..)
```
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
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
        docs: r#"# Num.has_units(val: int | float) -> bool
Returns true if the given number has units.
```rust
const val = 10kg;
assert(val.has_units());
```
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
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
        docs: r#"# Num.remove_units(val: int | float) -> int | float
Removes the units (if any) on this number.
```rust
const val = 10kg;
assert_eq(typeof val.remove_units(), "float");
```
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
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
        docs: r#"# Num.is_angle(val: int | float) -> bool
Returns true if the given number has angular units (degrees or radians).
```rust
const val = 10deg;
assert(val.is_angle());
```
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
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
        docs: r#"# Num.is_temp(val: int | float) -> bool
Returns true if the given number has temperature units.
```rust
const val = 10F;
assert(val.is_temp());
```
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
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
        docs: r#"# Num.is_length(val: int | float) -> bool
Returns true if the given number has length units.
```rust
const val = 10m;
assert(val.is_length());
```
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
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
        docs: r#"# Num.is_time(val: int | float) -> bool
Returns true if the given number has units of time.
```rust
const val = 10s;
assert(val.is_time());
```
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
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
        docs: r#"# Num.is_mass(val: int | float) -> bool
Returns true if the given number has units of mass.
```rust
const val = 10kg;
assert(val.is_mass());
```
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
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
        docs: r#"# Num.round(val: int | float, places: int = 0) -> int | float
Round the given number to the given number of places. If value is an integer, do nothing.
```rust
const val = 10.348;
assert_eq(val.round(2), 10.35);
assert_eq(val.round(), 10);
```
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None },
            Param { name: "places".into(), param_type: Type::Void, default: Some(Arc::new(Base::Literal(Val::Num(Num::Int(0))))) }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
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
        docs: r#"# Num.pow(val: int | float, to: int | float = 2) -> float
Returns the given value raised to the given power.
```rust
const val = 10;
assert_eq(val.pow(to = 2), 100);
assert_eq(val.pow(), 100);
```
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None },
            Param { name: "to".into(), param_type: Type::Void, default: Some(Arc::new(Base::Literal(Val::Num(Num::Int(2))))) }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
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
        docs: r#"# Num.log(val: int | float, base: int | float = 10) -> float
Log function with a given base value.
```rust
assert_eq((2).log().round(3), 0.301);
```
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None },
            Param { name: "base".into(), param_type: Type::Void, default: Some(Arc::new(Base::Literal(Val::Num(Num::Int(10))))) }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
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
        docs: r#"# Num.atan2(y: int | float, x: int | float) -> rad
Computes the four quadrant arctangent of self (y) and other (x) in radians.
```rust
assert_eq((Num.atan2(1, 2) as deg).round(), 27deg);
```
"#.into(),
        params: vector![
            Param { name: "y".into(), param_type: Type::Void, default: None },
            Param { name: "x".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(ATAN2.clone());
            Ok(instructions)
        })
    }
}
