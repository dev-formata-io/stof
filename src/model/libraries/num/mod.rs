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

use std::sync::Arc;
use arcstr::{literal, ArcStr};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use crate::{model::{Graph, num::{iter::{num_at, num_len}, maxmin::{num_max, num_min}, ops::{num_abs, num_acos, num_acosh, num_asin, num_asinh, num_atan, num_atan2, num_atanh, num_bin, num_cbrt, num_ceil, num_cos, num_cosh, num_exp, num_exp2, num_floor, num_fract, num_has_units, num_hex, num_inf, num_is_angle, num_is_length, num_is_mass, num_is_memory, num_is_temp, num_is_time, num_ln, num_log, num_nan, num_oct, num_pow, num_remove_units, num_round, num_signum, num_sin, num_sinh, num_sqrt, num_string, num_tan, num_tanh, num_to_units, num_trunc}}}, parser::types::parse_type_complete, runtime::{Error, NumT, Type, Val, Variable, instruction::{Instruction, Instructions}, proc::ProcEnv}};

mod ops;
mod maxmin;
mod iter;


/// Add the number library to a graph.
pub fn insert_number_lib(graph: &mut Graph) {
    graph.insert_libfunc(num_abs());
    graph.insert_libfunc(num_sqrt());
    graph.insert_libfunc(num_cbrt());
    graph.insert_libfunc(num_floor());
    graph.insert_libfunc(num_ceil());
    graph.insert_libfunc(num_trunc());
    graph.insert_libfunc(num_fract());
    graph.insert_libfunc(num_signum());
    graph.insert_libfunc(num_exp());
    graph.insert_libfunc(num_exp2());
    graph.insert_libfunc(num_ln());

    graph.insert_libfunc(num_nan());
    graph.insert_libfunc(num_inf());

    graph.insert_libfunc(num_sin());
    graph.insert_libfunc(num_cos());
    graph.insert_libfunc(num_tan());
    graph.insert_libfunc(num_asin());
    graph.insert_libfunc(num_acos());
    graph.insert_libfunc(num_atan());
    graph.insert_libfunc(num_sinh());
    graph.insert_libfunc(num_cosh());
    graph.insert_libfunc(num_tanh());
    graph.insert_libfunc(num_asinh());
    graph.insert_libfunc(num_acosh());
    graph.insert_libfunc(num_atanh());

    graph.insert_libfunc(num_hex());
    graph.insert_libfunc(num_bin());
    graph.insert_libfunc(num_oct());
    graph.insert_libfunc(num_string());
    
    graph.insert_libfunc(num_max());
    graph.insert_libfunc(num_min());

    graph.insert_libfunc(num_len());
    graph.insert_libfunc(num_at());

    graph.insert_libfunc(num_has_units());
    graph.insert_libfunc(num_to_units());
    graph.insert_libfunc(num_remove_units());
    graph.insert_libfunc(num_is_angle());
    graph.insert_libfunc(num_is_temp());
    graph.insert_libfunc(num_is_length());
    graph.insert_libfunc(num_is_mass());
    graph.insert_libfunc(num_is_memory());
    graph.insert_libfunc(num_is_time());

    graph.insert_libfunc(num_round());
    graph.insert_libfunc(num_pow());
    graph.insert_libfunc(num_log());
    graph.insert_libfunc(num_atan2());
}


/// Library name.
pub(self) const NUM_LIB: ArcStr = literal!("Num");


// Static instructions.
lazy_static! {
    pub(self) static ref ABS: Arc<dyn Instruction> = Arc::new(NumIns::Abs);
    pub(self) static ref SQRT: Arc<dyn Instruction> = Arc::new(NumIns::Sqrt);
    pub(self) static ref CBRT: Arc<dyn Instruction> = Arc::new(NumIns::Cbrt);
    pub(self) static ref FLOOR: Arc<dyn Instruction> = Arc::new(NumIns::Floor);
    pub(self) static ref CEIL: Arc<dyn Instruction> = Arc::new(NumIns::Ceil);
    pub(self) static ref TRUNC: Arc<dyn Instruction> = Arc::new(NumIns::Trunc);
    pub(self) static ref FRACT: Arc<dyn Instruction> = Arc::new(NumIns::Fract);
    pub(self) static ref SIGNUM: Arc<dyn Instruction> = Arc::new(NumIns::Signum);
    pub(self) static ref EXP: Arc<dyn Instruction> = Arc::new(NumIns::Exp);
    pub(self) static ref EXP2: Arc<dyn Instruction> = Arc::new(NumIns::Exp2);
    pub(self) static ref LN: Arc<dyn Instruction> = Arc::new(NumIns::Ln);

    pub(self) static ref NAN: Arc<dyn Instruction> = Arc::new(NumIns::NaN);
    pub(self) static ref INF: Arc<dyn Instruction> = Arc::new(NumIns::Inf);

    pub(self) static ref SIN: Arc<dyn Instruction> = Arc::new(NumIns::Sin);
    pub(self) static ref COS: Arc<dyn Instruction> = Arc::new(NumIns::Cos);
    pub(self) static ref TAN: Arc<dyn Instruction> = Arc::new(NumIns::Tan);
    pub(self) static ref ASIN: Arc<dyn Instruction> = Arc::new(NumIns::ASin);
    pub(self) static ref ACOS: Arc<dyn Instruction> = Arc::new(NumIns::ACos);
    pub(self) static ref ATAN: Arc<dyn Instruction> = Arc::new(NumIns::ATan);
    pub(self) static ref SINH: Arc<dyn Instruction> = Arc::new(NumIns::SinH);
    pub(self) static ref COSH: Arc<dyn Instruction> = Arc::new(NumIns::CosH);
    pub(self) static ref TANH: Arc<dyn Instruction> = Arc::new(NumIns::TanH);
    pub(self) static ref ASINH: Arc<dyn Instruction> = Arc::new(NumIns::ASinH);
    pub(self) static ref ACOSH: Arc<dyn Instruction> = Arc::new(NumIns::ACosH);
    pub(self) static ref ATANH: Arc<dyn Instruction> = Arc::new(NumIns::ATanH);

    pub(self) static ref HEX: Arc<dyn Instruction> = Arc::new(NumIns::Hex);
    pub(self) static ref BIN: Arc<dyn Instruction> = Arc::new(NumIns::Bin);
    pub(self) static ref OCT: Arc<dyn Instruction> = Arc::new(NumIns::Oct);
    pub(self) static ref STRING: Arc<dyn Instruction> = Arc::new(NumIns::String);

    pub(self) static ref AT: Arc<dyn Instruction> = Arc::new(NumIns::At);
    pub(self) static ref Round: Arc<dyn Instruction> = Arc::new(NumIns::Round);
    pub(self) static ref ROUND2: Arc<dyn Instruction> = Arc::new(NumIns::Round2);
    pub(self) static ref POW: Arc<dyn Instruction> = Arc::new(NumIns::Pow);
    pub(self) static ref LOG: Arc<dyn Instruction> = Arc::new(NumIns::Log);
    pub(self) static ref ATAN2: Arc<dyn Instruction> = Arc::new(NumIns::ATan2);

    pub(self) static ref HAS_UNITS: Arc<dyn Instruction> = Arc::new(NumIns::HasUnits);
    pub(self) static ref TO_UNITS: Arc<dyn Instruction> = Arc::new(NumIns::ToUnits);
    pub(self) static ref REMOVE_UNITS: Arc<dyn Instruction> = Arc::new(NumIns::RemoveUnits);
    pub(self) static ref IS_ANGLE: Arc<dyn Instruction> = Arc::new(NumIns::IsAngle);
    pub(self) static ref IS_LENGTH: Arc<dyn Instruction> = Arc::new(NumIns::IsLength);
    pub(self) static ref IS_MASS: Arc<dyn Instruction> = Arc::new(NumIns::IsMass);
    pub(self) static ref IS_TEMP: Arc<dyn Instruction> = Arc::new(NumIns::IsTemp);
    pub(self) static ref IS_TIME: Arc<dyn Instruction> = Arc::new(NumIns::IsTime);
    pub(self) static ref IS_MEMORY: Arc<dyn Instruction> = Arc::new(NumIns::IsMemory);
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Number Instruction.
pub enum NumIns {
    Abs,
    Sqrt,
    Cbrt,
    Floor,
    Ceil,
    Trunc,
    Fract,
    Signum,
    Exp,
    Exp2,
    Ln,

    NaN,
    Inf,

    Sin,
    Cos,
    Tan,
    ASin,
    ACos,
    ATan,
    SinH,
    CosH,
    TanH,
    ASinH,
    ACosH,
    ATanH,

    Hex,
    Bin,
    Oct,
    String,

    Max(usize),
    Min(usize),

    At,
    Round,
    Round2,
    Pow,
    Log,
    ATan2,

    HasUnits,
    RemoveUnits,
    ToUnits,
    IsAngle,
    IsTemp,
    IsLength,
    IsTime,
    IsMass,
    IsMemory,
}
#[typetag::serde(name = "NumIns")]
impl Instruction for NumIns {
    fn exec(&self, env: &mut ProcEnv, graph: &mut Graph) -> Result<Option<Instructions> , Error> {
        match self {
            Self::Abs => {
                if let Some(var) = env.stack.pop() {
                    if let Some(num) = var.val.write().try_num() {
                        num.abs()?;
                    } else {
                        return Err(Error::NumAbs)
                    }
                    env.stack.push(var);
                } else {
                    return Err(Error::NumAbs)
                }
            },
            Self::Sqrt => {
                if let Some(var) = env.stack.pop() {
                    let mut push = false;
                    if let Some(num) = var.val.write().try_num() {
                        num.sqrt()?;
                        push = true;
                    }
                    if push {
                        env.stack.push(var);
                        return Ok(None);
                    }
                }
                return Err(Error::NumSqrt);
            },
            Self::Cbrt => {
                if let Some(var) = env.stack.pop() {
                    let mut push = false;
                    if let Some(num) = var.val.write().try_num() {
                        num.cbrt()?;
                        push = true;
                    }
                    if push {
                        env.stack.push(var);
                        return Ok(None);
                    }
                }
                return Err(Error::NumCbrt);
            },
            Self::Floor => {
                if let Some(var) = env.stack.pop() {
                    let mut push = false;
                    if let Some(num) = var.val.write().try_num() {
                        num.floor()?;
                        push = true;
                    }
                    if push {
                        env.stack.push(var);
                        return Ok(None);
                    }
                }
                return Err(Error::NumFloor);
            },
            Self::Ceil => {
                if let Some(var) = env.stack.pop() {
                    let mut push = false;
                    if let Some(num) = var.val.write().try_num() {
                        num.ceil()?;
                        push = true;
                    }
                    if push {
                        env.stack.push(var);
                        return Ok(None);
                    }
                }
                return Err(Error::NumCeil);
            },
            Self::Trunc => {
                if let Some(var) = env.stack.pop() {
                    let mut push = false;
                    if let Some(num) = var.val.write().try_num() {
                        num.trunc()?;
                        push = true;
                    }
                    if push {
                        env.stack.push(var);
                        return Ok(None);
                    }
                }
                return Err(Error::NumTrunc);
            },
            Self::Fract => {
                if let Some(var) = env.stack.pop() {
                    let mut push = false;
                    if let Some(num) = var.val.write().try_num() {
                        num.fract()?;
                        push = true;
                    }
                    if push {
                        env.stack.push(var);
                        return Ok(None);
                    }
                }
                return Err(Error::NumFract);
            },
            Self::Signum => {
                if let Some(var) = env.stack.pop() {
                    let mut push = false;
                    if let Some(num) = var.val.write().try_num() {
                        num.signum()?;
                        push = true;
                    }
                    if push {
                        env.stack.push(var);
                        return Ok(None);
                    }
                }
                return Err(Error::NumSignum);
            },
            Self::Exp => {
                if let Some(var) = env.stack.pop() {
                    let mut push = false;
                    if let Some(num) = var.val.write().try_num() {
                        num.exp()?;
                        push = true;
                    }
                    if push {
                        env.stack.push(var);
                        return Ok(None);
                    }
                }
                return Err(Error::NumExp);
            },
            Self::Exp2 => {
                if let Some(var) = env.stack.pop() {
                    let mut push = false;
                    if let Some(num) = var.val.write().try_num() {
                        num.exp2()?;
                        push = true;
                    }
                    if push {
                        env.stack.push(var);
                        return Ok(None);
                    }
                }
                return Err(Error::NumExp2);
            },
            Self::Ln => {
                if let Some(var) = env.stack.pop() {
                    let mut push = false;
                    if let Some(num) = var.val.write().try_num() {
                        num.ln()?;
                        push = true;
                    }
                    if push {
                        env.stack.push(var);
                        return Ok(None);
                    }
                }
                return Err(Error::NumLn);
            },

            Self::Sin => {
                if let Some(var) = env.stack.pop() {
                    let mut push = false;
                    if let Some(num) = var.val.write().try_num() {
                        num.sin()?;
                        push = true;
                    }
                    if push {
                        env.stack.push(var);
                        return Ok(None);
                    }
                }
                return Err(Error::NumSin);
            },
            Self::Cos => {
                if let Some(var) = env.stack.pop() {
                    let mut push = false;
                    if let Some(num) = var.val.write().try_num() {
                        num.cos()?;
                        push = true;
                    }
                    if push {
                        env.stack.push(var);
                        return Ok(None);
                    }
                }
                return Err(Error::NumCos);
            },
            Self::Tan => {
                if let Some(var) = env.stack.pop() {
                    let mut push = false;
                    if let Some(num) = var.val.write().try_num() {
                        num.tan()?;
                        push = true;
                    }
                    if push {
                        env.stack.push(var);
                        return Ok(None);
                    }
                }
                return Err(Error::NumTan);
            },
            Self::ASin => {
                if let Some(var) = env.stack.pop() {
                    let mut push = false;
                    if let Some(num) = var.val.write().try_num() {
                        num.asin()?;
                        push = true;
                    }
                    if push {
                        env.stack.push(var);
                        return Ok(None);
                    }
                }
                return Err(Error::NumASin);
            },
            Self::ACos => {
                if let Some(var) = env.stack.pop() {
                    let mut push = false;
                    if let Some(num) = var.val.write().try_num() {
                        num.acos()?;
                        push = true;
                    }
                    if push {
                        env.stack.push(var);
                        return Ok(None);
                    }
                }
                return Err(Error::NumACos);
            },
            Self::ATan => {
                if let Some(var) = env.stack.pop() {
                    let mut push = false;
                    if let Some(num) = var.val.write().try_num() {
                        num.atan()?;
                        push = true;
                    }
                    if push {
                        env.stack.push(var);
                        return Ok(None);
                    }
                }
                return Err(Error::NumATan);
            },
            Self::SinH => {
                if let Some(var) = env.stack.pop() {
                    let mut push = false;
                    if let Some(num) = var.val.write().try_num() {
                        num.sinh()?;
                        push = true;
                    }
                    if push {
                        env.stack.push(var);
                        return Ok(None);
                    }
                }
                return Err(Error::NumSinH);
            },
            Self::CosH => {
                if let Some(var) = env.stack.pop() {
                    let mut push = false;
                    if let Some(num) = var.val.write().try_num() {
                        num.cosh()?;
                        push = true;
                    }
                    if push {
                        env.stack.push(var);
                        return Ok(None);
                    }
                }
                return Err(Error::NumCosH);
            },
            Self::TanH => {
                if let Some(var) = env.stack.pop() {
                    let mut push = false;
                    if let Some(num) = var.val.write().try_num() {
                        num.tanh()?;
                        push = true;
                    }
                    if push {
                        env.stack.push(var);
                        return Ok(None);
                    }
                }
                return Err(Error::NumTanH);
            },
            Self::ASinH => {
                if let Some(var) = env.stack.pop() {
                    let mut push = false;
                    if let Some(num) = var.val.write().try_num() {
                        num.asinh()?;
                        push = true;
                    }
                    if push {
                        env.stack.push(var);
                        return Ok(None);
                    }
                }
                return Err(Error::NumASinH);
            },
            Self::ACosH => {
                if let Some(var) = env.stack.pop() {
                    let mut push = false;
                    if let Some(num) = var.val.write().try_num() {
                        num.acosh()?;
                        push = true;
                    }
                    if push {
                        env.stack.push(var);
                        return Ok(None);
                    }
                }
                return Err(Error::NumACosH);
            },
            Self::ATanH => {
                if let Some(var) = env.stack.pop() {
                    let mut push = false;
                    if let Some(num) = var.val.write().try_num() {
                        num.atanh()?;
                        push = true;
                    }
                    if push {
                        env.stack.push(var);
                        return Ok(None);
                    }
                }
                return Err(Error::NumATanH);
            },

            Self::Hex => {
                if let Some(var) = env.stack.pop() {
                    if let Some(num) = var.val.write().try_num() {
                        let int = num.int();
                        env.stack.push(Variable::val(Val::Str(format!("{:X}", int).into())));
                        return Ok(None);
                    }
                }
                return Err(Error::NumHex);
            },
            Self::Bin => {
                if let Some(var) = env.stack.pop() {
                    if let Some(num) = var.val.write().try_num() {
                        let int = num.int();
                        env.stack.push(Variable::val(Val::Str(format!("{:b}", int).into())));
                        return Ok(None);
                    }
                }
                return Err(Error::NumBin);
            },
            Self::Oct => {
                if let Some(var) = env.stack.pop() {
                    if let Some(num) = var.val.write().try_num() {
                        let int = num.int();
                        env.stack.push(Variable::val(Val::Str(format!("{:o}", int).into())));
                        return Ok(None);
                    }
                }
                return Err(Error::NumOct);
            },
            Self::String => {
                if let Some(var) = env.stack.pop() {
                    if let Some(num) = var.val.write().try_num() {
                        env.stack.push(Variable::val(Val::Str(num.print().into())));
                        return Ok(None);
                    }
                }
                return Err(Error::NumStr);
            },

            Self::Max(stack_count) => {
                let mut res = None;
                for _ in 0..*stack_count {
                    if let Some(var) = env.stack.pop() {
                        let max_var = var.val.read().maximum(graph)?;
                        if let Some(current) = res {
                            let gt = max_var.gt(&current, &graph)?;
                            if gt.truthy() {
                                res = Some(max_var);
                            } else {
                                res = Some(current);
                            }
                        } else {
                            res = Some(max_var);
                        }
                    }
                }
                if let Some(res) = res {
                    env.stack.push(Variable::val(res));
                } else {
                    env.stack.push(Variable::val(Val::Null));
                }
            },
            Self::Min(stack_count) => {
                let mut res = None;
                for _ in 0..*stack_count {
                    if let Some(var) = env.stack.pop() {
                        let min_var = var.val.read().minimum(graph)?;
                        if let Some(current) = res {
                            let lt = min_var.lt(&current, &graph)?;
                            if lt.truthy() {
                                res = Some(min_var);
                            } else {
                                res = Some(current);
                            }
                        } else {
                            res = Some(min_var);
                        }
                    }
                }
                if let Some(res) = res {
                    env.stack.push(Variable::val(res));
                } else {
                    env.stack.push(Variable::val(Val::Null));
                }
            },
            Self::At => {
                if let Some(index_var) = env.stack.pop() {
                    if let Some(val_var) = env.stack.pop() {
                        let lt = index_var.lt(&val_var, &graph)?;
                        if lt.truthy() {
                            env.stack.push(index_var);
                        } else {
                            env.stack.push(val_var);
                        }
                        return Ok(None);
                    }
                }
                return Err(Error::NumAt);
            },
            Self::NaN => {
                if let Some(val) = env.stack.pop() {
                    if let Some(val) = val.val.write().try_num() {
                        env.stack.push(Variable::val(Val::Bool(val.nan())));
                        return Ok(None);
                    }
                }
                return Err(Error::NumNan);
            },
            Self::Inf => {
                if let Some(val) = env.stack.pop() {
                    if let Some(val) = val.val.write().try_num() {
                        env.stack.push(Variable::val(Val::Bool(val.inf())));
                        return Ok(None);
                    }
                }
                return Err(Error::NumNan);
            },

            Self::HasUnits => {
                if let Some(val) = env.stack.pop() {
                    if let Some(val) = val.val.write().try_num() {
                        env.stack.push(Variable::val(Val::Bool(val.has_units())));
                        return Ok(None);
                    }
                }
                return Err(Error::NumHasUnits);
            },
            Self::IsAngle => {
                if let Some(val) = env.stack.pop() {
                    if let Some(val) = val.val.write().try_num() {
                        env.stack.push(Variable::val(Val::Bool(val.is_angle())));
                        return Ok(None);
                    }
                }
                return Err(Error::NumIsAngle);
            },
            Self::IsTemp => {
                if let Some(val) = env.stack.pop() {
                    if let Some(val) = val.val.write().try_num() {
                        env.stack.push(Variable::val(Val::Bool(val.is_temp())));
                        return Ok(None);
                    }
                }
                return Err(Error::NumIsTemp);
            },
            Self::IsLength => {
                if let Some(val) = env.stack.pop() {
                    if let Some(val) = val.val.write().try_num() {
                        env.stack.push(Variable::val(Val::Bool(val.is_length())));
                        return Ok(None);
                    }
                }
                return Err(Error::NumIsLength);
            },
            Self::IsTime => {
                if let Some(val) = env.stack.pop() {
                    if let Some(val) = val.val.write().try_num() {
                        env.stack.push(Variable::val(Val::Bool(val.is_time())));
                        return Ok(None);
                    }
                }
                return Err(Error::NumIsTime);
            },
            Self::IsMass => {
                if let Some(val) = env.stack.pop() {
                    if let Some(val) = val.val.write().try_num() {
                        env.stack.push(Variable::val(Val::Bool(val.is_mass())));
                        return Ok(None);
                    }
                }
                return Err(Error::NumIsMass);
            },
            Self::IsMemory => {
                if let Some(val) = env.stack.pop() {
                    if let Some(val) = val.val.write().try_num() {
                        env.stack.push(Variable::val(Val::Bool(val.is_memory())));
                        return Ok(None);
                    }
                }
                return Err(Error::NumIsMemory);
            },
            Self::RemoveUnits => {
                if let Some(val) = env.stack.pop() {
                    let mut push = false;
                    if let Some(val) = val.val.write().try_num() {
                        val.remove_units();
                        push = true;
                    }
                    if push {
                        env.stack.push(val);
                        return Ok(None);
                    }
                }
                return Err(Error::NumHasUnits);
            },
            Self::ToUnits => {
                if let Some(units_val) = env.stack.pop() {
                    if let Some(mut val) = env.stack.pop() {
                        let mut to_type = None;
                        if let Some(units_num) = units_val.val.write().try_num() {
                            if let Some(units) = units_num.units() {
                                to_type = Some(Type::Num(NumT::Units(units)));
                            }
                        } else {
                            let val_str = units_val.val.read().to_string();
                            if let Ok(ty) = parse_type_complete(&val_str) {
                                to_type = Some(ty);
                            } else {
                                return Err(Error::NumToUnits);
                            }
                        }
                        if let Some(to) = to_type {
                            match &to {
                                Type::Num(_) => {},
                                _ => {
                                    // to_type is not a number type
                                    return Err(Error::NumToUnits);
                                }
                            }
                            val = val.stack_var(false);
                            if let Err(_error) = val.cast(&to, graph, Some(env.self_ptr())) {
                                return Err(Error::NumToUnits);
                            }
                        }
                        env.stack.push(val);
                        return Ok(None);
                    }
                }
                return Err(Error::NumToUnits);
            },

            Self::Round => {
                if let Some(var) = env.stack.pop() {
                    let mut push = false;
                    if let Some(num) = var.val.write().try_num() {
                        num.round()?;
                        push = true;
                    }
                    if push {
                        env.stack.push(var);
                        return Ok(None);
                    }
                }
                return Err(Error::NumRound);
            },
            Self::Round2 => {
                if let Some(places_var) = env.stack.pop() {
                    if let Some(val_var) = env.stack.pop() {
                        let mut push = false;
                        if let Some(digits) = places_var.val.write().try_num() {
                            if let Some(val) = val_var.val.write().try_num() {
                                val.round2(&digits)?;
                                push = true;
                            }
                        }
                        if push {
                            env.stack.push(val_var);
                            return Ok(None);
                        }
                    }
                }
                return Err(Error::NumRound2);
            },
            Self::Pow => {
                if let Some(to_var) = env.stack.pop() {
                    if let Some(val_var) = env.stack.pop() {
                        let mut push = false;
                        if let Some(to) = to_var.val.write().try_num() {
                            if let Some(val) = val_var.val.write().try_num() {
                                val.pow(&to)?;
                                push = true;
                            }
                        }
                        if push {
                            env.stack.push(val_var);
                            return Ok(None);
                        }
                    }
                }
                return Err(Error::NumPow);
            },
            Self::Log => {
                if let Some(base_var) = env.stack.pop() {
                    if let Some(val_var) = env.stack.pop() {
                        let mut push = false;
                        if let Some(base) = base_var.val.write().try_num() {
                            if let Some(val) = val_var.val.write().try_num() {
                                val.log(&base)?;
                                push = true;
                            }
                        }
                        if push {
                            env.stack.push(val_var);
                            return Ok(None);
                        }
                    }
                }
                return Err(Error::NumLog);
            },
            Self::ATan2 => {
                if let Some(base_var) = env.stack.pop() {
                    if let Some(val_var) = env.stack.pop() {
                        let mut push = false;
                        if let Some(base) = base_var.val.write().try_num() {
                            if let Some(val) = val_var.val.write().try_num() {
                                val.atan2(&base)?;
                                push = true;
                            }
                        }
                        if push {
                            env.stack.push(val_var);
                            return Ok(None);
                        }
                    }
                }
                return Err(Error::NumATan2);
            },
        }
        Ok(None)
    }
}
