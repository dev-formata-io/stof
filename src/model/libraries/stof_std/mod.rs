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

use std::{mem::swap, ops::{Deref, DerefMut}, sync::Arc, time::Duration};
use arcstr::{literal, ArcStr};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use crate::{model::{stof_std::{assert::{assert, assert_eq, assert_neq, assert_not, throw}, containers::{std_copy, std_list, std_map, std_set, std_swap}, exit::stof_exit, print::{dbg, err, pln}, sleep::stof_sleep}, Graph}, runtime::{instruction::{Instruction, Instructions}, instructions::{list::{NEW_LIST, PUSH_LIST}, map::{NEW_MAP, PUSH_MAP}, set::{NEW_SET, PUSH_SET}, Base, EXIT}, proc::ProcEnv, Error, Type, Units, Val, Variable}};

mod print;
mod sleep;
mod assert;
mod exit;
mod containers;


/// Add the std library to a graph.
pub fn stof_std_lib(graph: &mut Graph) {
    graph.insert_libfunc(pln());
    graph.insert_libfunc(dbg());
    graph.insert_libfunc(err());
    graph.insert_libfunc(stof_sleep());
    graph.insert_libfunc(throw());
    graph.insert_libfunc(stof_exit());

    graph.insert_libfunc(assert());
    graph.insert_libfunc(assert_not());
    graph.insert_libfunc(assert_eq());
    graph.insert_libfunc(assert_neq());

    graph.insert_libfunc(std_list());
    graph.insert_libfunc(std_set());
    graph.insert_libfunc(std_map());

    graph.insert_libfunc(std_copy());
    graph.insert_libfunc(std_swap());
}


/// Library name.
pub(self) const STD_LIB: ArcStr = literal!("Std");


// Static instructions.
lazy_static! {
    pub(self) static ref SLEEP: Arc<dyn Instruction> = Arc::new(StdIns::Sleep);
    pub(self) static ref THROW: Arc<dyn Instruction> = Arc::new(StdIns::Throw);
    pub(self) static ref ASSERT: Arc<dyn Instruction> = Arc::new(StdIns::Assert);
    pub(self) static ref ASSERT_NOT: Arc<dyn Instruction> = Arc::new(StdIns::AssertNot);
    pub(self) static ref ASSERT_EQ: Arc<dyn Instruction> = Arc::new(StdIns::AssertEq);
    pub(self) static ref ASSERT_NEQ: Arc<dyn Instruction> = Arc::new(StdIns::AssertNeq);

    pub(self) static ref COPY: Arc<dyn Instruction> = Arc::new(StdIns::Copy);
    pub(self) static ref SWAP: Arc<dyn Instruction> = Arc::new(StdIns::Swap);
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Standard Lib Instruction.
pub enum StdIns {
    Pln(usize),
    Dbg(usize),
    Err(usize),

    Throw,
    Sleep,

    Exit(usize),

    Assert,
    AssertNot,
    AssertEq,
    AssertNeq,

    List(usize),
    Set(usize),
    Map(usize),

    Copy,
    Swap,
}
#[typetag::serde(name = "StdIns")]
impl Instruction for StdIns {
    fn exec(&self, env: &mut ProcEnv, graph: &mut Graph) -> Result<Option<Instructions> , Error> {
        match self {
            Self::Pln(arg_count) => {
                let mut values = Vec::new();
                for _ in 0..*arg_count {
                    if let Some(var) = env.stack.pop() {
                        values.push(var);
                    } else {
                        return Err(Error::StackError);
                    }
                }
                let mut output = Vec::new();
                let mut seen_str = false;
                for var in values.into_iter().rev() {
                    if !seen_str {
                        if var.gen_type() == Type::Str { seen_str = true; }
                    }
                    let out = var.val.read().print(&graph);
                    output.push(out);
                }
                let mut sep = "";
                if !seen_str { sep = ", " }
                println!("{}", output.join(sep));
            },
            Self::Dbg(arg_count) => {
                let mut values = Vec::new();
                for _ in 0..*arg_count {
                    if let Some(var) = env.stack.pop() {
                        values.push(var);
                    } else {
                        return Err(Error::StackError);
                    }
                }
                let mut output = Vec::new();
                let mut seen_str = false;
                for var in values.into_iter().rev() {
                    if !seen_str {
                        if var.gen_type() == Type::Str { seen_str = true; }
                    }
                    let out = var.val.read().debug(&graph);
                    output.push(out);
                }
                let mut sep = "";
                if !seen_str { sep = ", " }
                println!("{}", output.join(sep));
            },
            Self::Err(arg_count) => {
                let mut values = Vec::new();
                for _ in 0..*arg_count {
                    if let Some(var) = env.stack.pop() {
                        values.push(var);
                    } else {
                        return Err(Error::StackError);
                    }
                }
                let mut output = Vec::new();
                let mut seen_str = false;
                for var in values.into_iter().rev() {
                    if !seen_str {
                        if var.gen_type() == Type::Str { seen_str = true; }
                    }
                    let out = var.val.read().print(&graph);
                    output.push(out);
                }
                let mut sep = "";
                if !seen_str { sep = ", " }
                eprintln!("{}", output.join(sep));
            }
            
            Self::Sleep => {
                let duration;
                if let Some(val) = env.stack.pop() {
                    if let Some(num) = val.val.write().try_num() {
                        duration = num.float(Some(Units::Milliseconds));
                    } else {
                        return Err(Error::StackError);
                    }
                } else {
                    return Err(Error::StackError);
                }

                let mut instructions = Instructions::default();
                instructions.push(Arc::new(Base::CtrlSleepFor(Duration::from_millis(duration.abs() as u64))));
                return Ok(Some(instructions));
            },

            Self::Exit(arg_count) => {
                let mut instructions = Instructions::default();

                if *arg_count < 1 {
                    instructions.push(EXIT.clone());
                } else {
                    let mut promises = Vec::new();
                    for _ in 0..*arg_count {
                        if let Some(var) = env.stack.pop() {
                            if var.try_promise().is_some() {
                                promises.push(var);
                            }
                        }
                    }
                    for promise in promises.into_iter().rev() {
                        instructions.push(Arc::new(Base::Variable(promise)));
                        instructions.push(EXIT.clone());
                    }
                }

                return Ok(Some(instructions));
            },

            Self::Throw => {
                if let Some(val) = env.stack.pop() {
                    return Err(Error::Thrown(val.get()));
                }
            },

            Self::Assert => {
                if let Some(val) = env.stack.pop() {
                    if !val.val.read().truthy() {
                        let message = format!("'{}' is not truthy", val.val.read().print(&graph));
                        return Err(Error::AssertFailed(message));
                    }
                }
            },
            Self::AssertNot => {
                if let Some(val) = env.stack.pop() {
                    if val.val.read().truthy() {
                        let message = format!("'{}' is truthy", val.val.read().print(&graph));
                        return Err(Error::AssertNotFailed(message));
                    }
                }
            },
            Self::AssertEq => {
                if let Some(val) = env.stack.pop() {
                    if let Some(other) = env.stack.pop() {
                        if let Ok(res) = val.equal(&other) {
                            if !res.val.read().truthy() {
                                let message = format!("'{}' does not equal '{}'", other.val.read().print(&graph), val.val.read().print(&graph));
                                return Err(Error::AssertEqFailed(message));
                            }
                        }
                    }
                }
            },
            Self::AssertNeq => {
                if let Some(val) = env.stack.pop() {
                    if let Some(other) = env.stack.pop() {
                        if let Ok(res) = val.equal(&other) {
                            if res.val.read().truthy() {
                                let message = format!("'{}' equals '{}'", other.val.read().print(&graph), val.val.read().print(&graph));
                                return Err(Error::AssertNotEqFailed(message));
                            }
                        }
                    }
                }
            },

            Self::List(arg_count) => {
                let mut instructions = Instructions::default();
                instructions.push(NEW_LIST.clone());

                let mut args = Vec::new();
                for _ in 0..*arg_count {
                    args.push(env.stack.pop().unwrap());
                }
                for arg in args.into_iter().rev() {
                    instructions.push(Arc::new(Base::Variable(arg)));
                    instructions.push(PUSH_LIST.clone());
                }

                return Ok(Some(instructions));
            },
            Self::Set(arg_count) => {
                let mut instructions = Instructions::default();
                instructions.push(NEW_SET.clone());

                let mut args = Vec::new();
                for _ in 0..*arg_count {
                    args.push(env.stack.pop().unwrap());
                }
                for arg in args.into_iter().rev() {
                    instructions.push(Arc::new(Base::Variable(arg)));
                    instructions.push(PUSH_SET.clone());
                }

                return Ok(Some(instructions));
            },
            Self::Map(arg_count) => {
                let mut instructions = Instructions::default();
                instructions.push(NEW_MAP.clone());

                let mut args = Vec::new();
                for _ in 0..*arg_count {
                    args.push(env.stack.pop().unwrap());
                }
                for arg in args.into_iter().rev() {
                    match arg.val.read().deref() {
                        Val::Tup(vals) => {
                            if vals.len() == 2 {
                                instructions.push(Arc::new(Base::Variable(Variable::refval(vals[0].duplicate(false)))));
                                instructions.push(Arc::new(Base::Variable(Variable::refval(vals[1].duplicate(false)))));
                                instructions.push(PUSH_MAP.clone());
                            } else {
                                return Err(Error::MapConstructor("map init must have a key-value pair in the form of a list or tuple".into()));
                            }
                        },
                        Val::List(vals) => {
                            if vals.len() == 2 {
                                instructions.push(Arc::new(Base::Variable(Variable::refval(vals[0].duplicate(false)))));
                                instructions.push(Arc::new(Base::Variable(Variable::refval(vals[1].duplicate(false)))));
                                instructions.push(PUSH_MAP.clone());
                            } else {
                                return Err(Error::MapConstructor("map init must have a key-value pair in the form of a list or tuple".into()));
                            }
                        },
                        _ => {
                            return Err(Error::MapConstructor("unrecognized map init value (has to be a tuple or list with a key and value)".into()));
                        }
                    }
                }

                return Ok(Some(instructions));
            },

            Self::Copy => {
                if let Some(var) = env.stack.pop() {
                    env.stack.push(var.deep_copy());
                }
            },
            Self::Swap => {
                if let Some(first) = env.stack.pop() {
                    if let Some(second) = env.stack.pop() {
                        let mut first = first.val.write();
                        let mut second = second.val.write();

                        let first = first.deref_mut();
                        let second = second.deref_mut();
                        
                        swap(first, second);
                    }
                }
            },
        }
        Ok(None)
    }
}
