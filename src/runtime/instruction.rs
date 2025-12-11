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

use std::{any::Any, ops::Deref, sync::Arc};
use arcstr::ArcStr;
use colored::Colorize;
use imbl::{vector, Vector};
use serde::{Deserialize, Serialize};
use crate::{model::Graph, runtime::{instructions::{list::{NEW_LIST, PUSH_LIST}, Base, ConsumeStack, AWAIT}, proc::{ProcEnv, ProcRes}, Error, Val, Variable}};


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// Instructions.
pub struct Instructions {
    /// Uses structural sharing, then only copies the Arc when needed lazily.
    /// Store instructions in a Func, then clone into the proc without any copies.
    pub instructions: Vector<Arc<dyn Instruction>>,
    pub executed: Vector<Arc<dyn Instruction>>,
}
impl From<Arc<dyn Instruction>> for Instructions {
    fn from(value: Arc<dyn Instruction>) -> Self {
        Self {
            instructions: vector![value],
            ..Default::default()
        }
    }
}
impl From<Vector<Arc<dyn Instruction>>> for Instructions {
    fn from(value: Vector<Arc<dyn Instruction>>) -> Self {
        Self {
            instructions: value,
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
    }

    #[inline(always)]
    /// Are there more instructions to process?
    pub fn more(&self) -> bool {
        !self.instructions.is_empty()
    }

    /// Trace out the last N instructions that were executed.
    pub fn trace_n(&self, n: usize) -> String {
        let mut count = 0;
        let mut ins = Vector::default();
        for exec in self.executed.iter().rev() {
            ins.push_front(exec.clone());
            count += 1;
            if count >= n { break; }
        }

        let mut output = String::default();
        for i in 0..ins.len() {
            let ins = &ins[i];
            let inner = format!("{i}: {:?}", ins);
            if i == 0 {
                output.push_str(&format!("\t\t{}", inner.dimmed()));
            } else {
                output.push_str(&format!("\n\t\t{}", inner.dimmed()));
            }
        }
        output
    }

    /// Trace out the next N instructions that are going to be executed.
    pub fn peek_n(&self, n: usize) -> String {
        let mut count = 0;
        let mut ins = Vector::default();
        for exec in self.instructions.iter() {
            ins.push_back(exec.clone());
            count += 1;
            if count >= n { break; }
        }

        let mut output = String::default();
        for i in 0..ins.len() {
            let ins = &ins[i];
            let inner = format!("{i}: {:?}", ins);
            if i == 0 {
                output.push_str(&format!("\t\t{}", inner.dimmed()));
            } else {
                output.push_str(&format!("\n\t\t{}", inner.dimmed()));
            }
        }
        output
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

    /// Backup to a specific tag in these instructions, killing all instructions.
    pub fn kill_back_to(&mut self, tag: &ArcStr) {
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

    #[inline]
    /// Execute one instruction, in order.
    /// This will pop the first instruction, leaving the next ready to be consumed later.
    pub fn exec(&mut self, env: &mut ProcEnv, graph: &mut Graph, mut limit: i32) -> Result<ProcRes, Error> {
        if env.start_time.is_none() {
            env.start_time = Some(web_time::Instant::now());
        }
        let keep_count = limit > 0;
        'exec_loop: loop {
            if keep_count {
                if limit <= 0 {
                    if self.more() {
                        return Ok(ProcRes::More);
                    } else {
                        return Ok(ProcRes::Done);
                    }
                }
                limit -= 1;
            }

            // enforce max execution time
            if let Some(max) = &env.max_execution_time {
                if let Some(start) = &env.start_time {
                    if &start.elapsed() > max {
                        return Err(Error::ExecutionTimeout);
                    }
                }
            }

            // enforce max stack sizes
            if env.stack.len() > env.max_stack_size {
                return Err(Error::StackOverflow);
            }
            if env.call_stack.len() > env.max_call_stack_depth {
                return Err(Error::CallStackOverflow);
            }

            if let Some(ins) = self.instructions.pop_front() {
                self.executed.push_back(ins.clone());

                if let Some(base) = ins.as_dyn_any().downcast_ref::<Base>() {
                    match base {
                        Base::CtrlTrace(n) => {
                            return Ok(ProcRes::Trace(*n));
                        },
                        Base::CtrlPeek(n) => {
                            return Ok(ProcRes::Peek(*n));
                        },
                        Base::CtrlExit => {
                            if let Some(promise) = env.stack.pop() {
                                if let Some((pid, _)) = promise.try_promise() {
                                    return Ok(ProcRes::Exit(Some(pid)));
                                } else {
                                    env.stack.push(promise);
                                }
                            }
                            return Ok(ProcRes::Exit(None));
                        },
                        Base::CtrlAwait => {
                            if let Some(promise) = env.stack.pop() {
                                if let Some((pid, cast_type)) = promise.try_promise() {
                                    if !cast_type.empty() {
                                        // Special instruction to cast the awaited value when we return to this process
                                        self.instructions.push_front(Arc::new(Base::CtrlAwaitCast(cast_type)));
                                    }
                                    return Ok(ProcRes::Wait(pid.clone()));
                                } else if promise.val.read().list() || promise.val.read().set() {
                                    let mut gtg = true;
                                    let mut awaits = Vec::new();

                                    match promise.val.read().deref() {
                                        Val::List(vals) => {
                                            for val in vals {
                                                if let Some(_) = val.read().try_promise() {
                                                    awaits.push(Arc::new(Base::Literal(val.read().clone())));
                                                } else {
                                                    gtg = false;
                                                }
                                            }
                                        },
                                        Val::Set(set) => {
                                            for val in set {
                                                if let Some(_) = val.read().try_promise() {
                                                    awaits.push(Arc::new(Base::Literal(val.read().clone())));
                                                } else {
                                                    gtg = false;
                                                }
                                            }
                                        },
                                        _ => {}
                                    }

                                    if gtg {
                                        self.executed.pop_back();
                                        for promise in awaits.into_iter().rev() {
                                            self.instructions.push_front(PUSH_LIST.clone());
                                            self.instructions.push_front(AWAIT.clone());
                                            self.instructions.push_front(promise);
                                        }
                                        self.instructions.push_front(NEW_LIST.clone());
                                    } else {
                                        env.stack.push(promise); // not a collection of promises
                                    }
                                } else {
                                    env.stack.push(promise); // put it back because not a promise
                                }
                            }
                            // Awaits on anything else are a passthrough operation...
                        },
                        Base::CtrlAwaitCast(cast_type) => {
                            self.executed.pop_back(); // This one doesn't stick around, which makes it special
                            if let Some(var) = env.stack.pop() {
                                var.cast(cast_type, graph, Some(env.self_ptr()))?;
                                env.stack.push(var);
                            } else if cast_type.empty() {
                                // nothing to do in this case
                            } else {
                                return Err(Error::CastStackError);
                            }
                            continue 'exec_loop;
                        },
                        Base::CtrlSuspend => {
                            // Go to the next processes instructions
                            // Used to spawn new processes as well
                            return Ok(ProcRes::More);
                        },
                        Base::CtrlSleepFor(dur) => {
                            // Instruct this process to sleep for an amount of time
                            return Ok(ProcRes::SleepFor(dur.clone()));
                        },
                        Base::CtrlSleepRef(wref) => {
                            // Instruct this process to sleep until the wake reference has been set
                            return Ok(ProcRes::Sleep(wref.clone()));
                        },
                        Base::CtrlBackTo(tag) => {
                            self.back_to(tag);
                            continue 'exec_loop;
                        },
                        Base::CtrlForwardTo(tag) => {
                            self.forward_to(tag);
                            continue 'exec_loop;
                        },
                        Base::CtrlFnReturn => {
                            // Go forwards to the current callstack function ID return tag
                            if let Some(last) = env.return_stack.last() {
                                self.forward_to(last);
                                continue 'exec_loop;
                            }
                        },
                        Base::CtrlBreak => {
                            if let Some(loop_tag) = env.loop_stack.last() {
                                let break_tag: ArcStr = format!("{}_brk", &loop_tag).into();
                                self.forward_to(&break_tag);
                                continue 'exec_loop;
                            }
                        },
                        Base::CtrlContinue => {
                            if let Some(loop_tag) = env.loop_stack.last() {
                                let continue_tag: ArcStr = format!("{}_con", &loop_tag).into();
                                self.forward_to(&continue_tag);
                                continue 'exec_loop;
                            }
                        },
                        Base::CtrlLoopBackTo { top_tag, .. } => {
                            self.kill_back_to(top_tag);
                            // don't continue...
                        },
                        Base::CtrlJumpTable(table, default, end) => {
                            // Compares the value on the top of the stack and jumps forwards to the associated tag
                            if let Some(var) = env.stack.pop() {
                                if let Some(tag) = table.get(&var.get()) {
                                    self.forward_to(tag);
                                    continue 'exec_loop;
                                } else if let Some(tag) = default {
                                    self.forward_to(tag);
                                    continue 'exec_loop;
                                } else {
                                    self.forward_to(end);
                                    continue 'exec_loop;
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

                let res = ins.exec(env, graph);
                match res {
                    Ok(replacements) => {
                        if let Some(mut dynamic) = replacements {
                            if dynamic.more() {
                                self.executed.pop_back(); // replacing this instruction with these instructions
                                while dynamic.more() {
                                    self.instructions.push_front(dynamic.instructions.pop_back().unwrap());
                                }
                            }
                        }
                    },
                    Err(error) => {
                        if let Some(try_tag) = env.try_stack.pop() {
                            self.forward_to(&try_tag);
                            match error {
                                Error::Thrown(val) => {
                                    env.stack.push(Variable::val(val));
                                },
                                _ => {
                                    env.stack.push(Variable::val(Val::Str(error.to_string().into())));
                                }
                            }
                            continue 'exec_loop;
                        } else {
                            return Err(error);
                        }
                    },
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
    fn exec(&self, env: &mut ProcEnv, graph: &mut Graph) -> Result<Option<Instructions>, Error>;
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
