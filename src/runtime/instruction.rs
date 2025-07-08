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

use std::{any::Any, sync::Arc};
use arcstr::ArcStr;
use imbl::{vector, Vector};
use serde::{Deserialize, Serialize};
use crate::{model::Graph, runtime::{instructions::{Base, ConsumeStack}, proc::{ProcEnv, ProcRes}, Error, Val, Variable}};


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// Instructions.
pub struct Instructions {
    /// Uses structural sharing, then only copies the Arc when needed lazily.
    /// Store instructions in a Func, then clone into the proc without any copies.
    pub instructions: Vector<Arc<dyn Instruction>>,
    pub executed: Vector<Arc<dyn Instruction>>,
    try_catch_count: u8,
}
impl From<Arc<dyn Instruction>> for Instructions {
    fn from(value: Arc<dyn Instruction>) -> Self {
        Self {
            instructions: vector![value],
            ..Default::default()
        }
    }
}
impl Instructions {
    #[inline(always)]
    /// Create a new Instructions.
    pub fn new(instructions: Vector<Arc<dyn Instruction>>) -> Self {
        Self { instructions, ..Default::default() }
    }

    #[inline]
    /// Clear these instructions.
    pub fn clear(&mut self) {
        self.instructions.clear();
        self.executed.clear();
        self.try_catch_count = 0;
    }

    #[inline(always)]
    /// Are there more instructions to process?
    pub fn more(&self) -> bool {
        !self.instructions.is_empty()
    }

    /// Backup to a specific tag in these instructions.
    pub fn back_to(&mut self, tag: &ArcStr) {
        'unwind: while let Some(ins) = self.executed.pop_back() {
            if let Some(base) = ins.as_dyn_any().downcast_ref::<Base>() {
                match base {
                    Base::Tag(tagged) => {
                        if tagged == tag {
                            self.executed.push_back(ins);
                            break 'unwind;
                        }
                    },
                    _ => {}
                }
            }
            self.instructions.push_front(ins);
        }
    }

    /// Backup to a specific tag in these instructions.
    pub fn forward_to(&mut self, tag: &ArcStr) {
        'fast_forward: while let Some(ins) = self.instructions.pop_front() {
            self.executed.push_back(ins.clone());
            if let Some(base) = ins.as_dyn_any().downcast_ref::<Base>() {
                match base {
                    Base::Tag(tagged) => {
                        if tagged == tag {
                            break 'fast_forward;
                        }
                    },
                    _ => {}
                }
            }
        }
    }

    /// Backup to the last try to see where we need to go, then forward to there.
    fn unwind_try(&mut self) -> bool {
        let mut try_tag = None;
        'unwind: while let Some(ins) = self.executed.pop_back() {
            if let Some(base) = ins.as_dyn_any().downcast_ref::<Base>() {
                match base {
                    Base::CtrlTry(tag) => {
                        try_tag = Some(tag.clone());
                        self.executed.push_back(ins);
                        break 'unwind;
                    },
                    _ => {}
                }
            }
            self.instructions.push_front(ins);
        }
        if let Some(tag) = try_tag {
            self.forward_to(&tag);
            true
        } else {
            false
        }
    }

    #[inline]
    /// Execute one instruction, in order.
    /// This will pop the first instruction, leaving the next ready to be consumed later.
    pub fn exec(&mut self, env: &mut ProcEnv, graph: &mut Graph, mut limit: i32) -> Result<ProcRes, Error> {
        let keep_count = limit > 0;
        'exec_loop: loop {
            if keep_count {
                if limit <= 0 { return Ok(ProcRes::More); }
                limit -= 1;
            }

            if let Some(ins) = self.instructions.pop_front() {
                self.executed.push_back(ins.clone());

                if let Some(base) = ins.as_dyn_any().downcast_ref::<Base>() {
                    match base {
                        Base::CtrlTry(_) => {
                            self.try_catch_count += 1;
                            continue 'exec_loop;
                        },
                        Base::CtrlTryEnd => {
                            if self.try_catch_count > 0 {
                                self.try_catch_count -= 1;
                            }
                            continue 'exec_loop;
                        },
                        Base::CtrlAwait => {
                            if let Some(promise) = env.stack.pop() {
                                if let Some((pid, _)) = promise.try_promise() {
                                    return Ok(ProcRes::Wait(pid.clone()));
                                } else {
                                    // TODO: expand array of promises by adding them each to the stack with additional awaits for each

                                    env.stack.push(promise); // put it back because not a promise
                                }
                            }
                            // Awaits on anything else are a passthrough operation...
                        },
                        Base::CtrlSuspend => {
                            // Go to the next processes instructions
                            // Used to spawn new processes as well
                            return Ok(ProcRes::More);
                        },
                        Base::CtrlBackTo(tag) => {
                            self.back_to(tag);
                            continue 'exec_loop;
                        },
                        Base::CtrlForwardTo(tag) => {
                            self.forward_to(tag);
                            continue 'exec_loop;
                        },
                        Base::CtrlJumpTable(table, default) => {
                            // Compares the value on the top of the stack and jumps forwards to the associated tag
                            // Throws a JumpTable error if not found in the table (and no default)
                            if let Some(var) = env.stack.pop() {
                                if let Some(tag) = table.get(&var.get()) {
                                    self.forward_to(tag);
                                    continue 'exec_loop;
                                } else if let Some(tag) = default {
                                    self.forward_to(tag);
                                    continue 'exec_loop;
                                } else {
                                    return Err(Error::JumpTable);
                                }
                            } else {
                                return Err(Error::StackError);
                            }
                        },
                        Base::CtrlForwardToIfTruthy(tag, consume) => {
                            if let Some(val) = env.stack.pop() {
                                if val.truthy() {
                                    match consume {
                                        ConsumeStack::Dont |
                                        ConsumeStack::IfTrue => {
                                            env.stack.push(val);
                                        },
                                        _ => {}
                                    }
                                    self.forward_to(tag);
                                    continue 'exec_loop;
                                } else {
                                    match consume {
                                        ConsumeStack::Dont |
                                        ConsumeStack::IfFalse => {
                                            env.stack.push(val);
                                        },
                                        _ => {}
                                    }
                                }
                            }
                        },
                        Base::CtrlForwardToIfNotTruthy(tag, consume) => {
                            if let Some(val) = env.stack.pop() {
                                if !val.truthy() {
                                    match consume {
                                        ConsumeStack::Dont |
                                        ConsumeStack::IfTrue => {
                                            env.stack.push(val);
                                        },
                                        _ => {}
                                    }
                                    self.forward_to(tag);
                                    continue 'exec_loop;
                                } else {
                                    match consume {
                                        ConsumeStack::Dont |
                                        ConsumeStack::IfFalse => {
                                            env.stack.push(val);
                                        },
                                        _ => {}
                                    }
                                }
                            }
                        },
                        _ => {}
                    }
                }

                // Some fresh instructions for ya
                let mut dynamic = Self::default();
                let res = ins.exec(&mut dynamic, env, graph);
                match res {
                    Ok(_) => {},
                    Err(error) => {
                        if self.try_catch_count > 0 && self.unwind_try() {
                            env.stack.push(Variable::val(Val::Str(error.to_string().into()))); // TODO better errors?
                            continue 'exec_loop;
                        } else {
                            return Err(error);
                        }
                    },
                }

                if dynamic.more() {
                    self.executed.pop_back(); // replacing this instruction with these instructions
                    while dynamic.more() {
                        self.instructions.push_front(dynamic.instructions.pop_back().unwrap());
                    }
                }
            } else {
                break;
            }
        }
        if self.more() {
            Ok(ProcRes::More)
        } else {
            Ok(ProcRes::Done)
        }
    }

    #[inline(always)]
    /// Append instructions.
    pub fn append(&mut self, instructions: &Vector<Arc<dyn Instruction>>) {
        self.instructions.append(instructions.clone());
    }

    #[inline(always)]
    /// Push an instruction.
    pub fn push(&mut self, instruction: Arc<dyn Instruction>) {
        self.instructions.push_back(instruction);
    }

    #[inline(always)]
    /// Pop an instruction.
    pub fn pop(&mut self) {
        self.instructions.pop_back();
    }
}


#[typetag::serde]
/// Instruction trait for an operation within the runtime.
pub trait Instruction: InsDynAny + std::fmt::Debug + InsClone + Send + Sync {
    /// Execute this instruction given the process it's running on and the graph.
    fn exec(&self, instructions: &mut Instructions, env: &mut ProcEnv, graph: &mut Graph) -> Result<(), Error>;
}


/// Blanket manual upcast to dyn Any for instructions.
pub trait InsDynAny {
    fn as_dyn_any(&self) -> &dyn Any;
    fn as_mut_dyn_any(&mut self) -> &mut dyn Any;
}
impl<T: Instruction + Any> InsDynAny for T {
    fn as_dyn_any(&self) -> &dyn Any {
        self
    }
    fn as_mut_dyn_any(&mut self) -> &mut dyn Any {
        self
    }
}


/// Blanket Clone implementation for any struct that implements Clone + Instruction
pub trait InsClone {
    fn clone_ins(&self) -> Box<dyn Instruction>;
}
impl<T: Instruction + Clone + 'static> InsClone for T {
    fn clone_ins(&self) -> Box<dyn Instruction> {
        Box::new(self.clone())
    }
}
impl Clone for Box<dyn Instruction> {
    fn clone(&self) -> Box<dyn Instruction> {
        self.clone_ins()
    }
}
