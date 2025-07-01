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
use crate::{model::Graph, runtime::{instructions::Base, proc::ProcEnv, Error}};


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// Instructions.
pub struct Instructions {
    /// Uses structural sharing, then only copies the Arc when needed lazily.
    /// Store instructions in a Func, then clone into the proc without any copies.
    pub instructions: Vector<Arc<dyn Instruction>>,
    executed: Vector<Arc<dyn Instruction>>,
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

    #[inline]
    /// Execute one instruction, in order.
    /// This will pop the first instruction, leaving the next ready to be consumed later.
    pub fn exec(&mut self, env: &mut ProcEnv, graph: &mut Graph) -> Result<(), Error> {
        'exec_loop: loop {
            if let Some(ins) = self.instructions.pop_front() {
                self.executed.push_back(ins.clone());

                if let Some(base) = ins.as_dyn_any().downcast_ref::<Base>() {
                    match base {
                        Base::CtrlSuspend => {
                            // Go to the next processes instructions
                            break 'exec_loop;
                        },
                        Base::CtrlBackTo(tag) => {
                            self.back_to(tag);
                            continue 'exec_loop;
                        },
                        Base::CtrlForwardTo(tag) => {
                            self.forward_to(tag);
                            continue 'exec_loop;
                        },
                        _ => {}
                    }
                }

                // Some fresh instructions for ya
                let mut dynamic = Self::default();
                let res = ins.exec(&mut dynamic, env, graph);
                if res.is_ok() && dynamic.more() {
                    while dynamic.more() {
                        self.instructions.push_front(dynamic.instructions.pop_back().unwrap());
                    }
                }
            } else {
                break;
            }
        }
        Ok(())
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
