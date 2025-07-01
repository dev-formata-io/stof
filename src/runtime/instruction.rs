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

use std::{any::Any, mem::swap, sync::Arc};
use imbl::Vector;
use serde::{Deserialize, Serialize};
use crate::{model::Graph, runtime::{proc::ProcEnv, Error}};


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// Instructions.
pub struct Instructions {
    /// Uses structural sharing, then only copies the Arc when needed lazily.
    /// Store instructions in a Func, then clone into the proc without any copies.
    pub instructions: Vector<Arc<dyn Instruction>>,
    executed: Vector<Arc<dyn Instruction>>,
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

    #[inline]
    /// Execute one instruction, in order.
    /// This will pop the first instruction, leaving the next ready to be consumed later.
    pub fn exec(&mut self, env: &mut ProcEnv, graph: &mut Graph) -> Result<(), Error> {
        loop {
            if let Some(ins) = self.instructions.pop_front() {
                // Go to the next processes instructions
                if ins.suspend_op() {
                    break;
                }

                self.executed.push_back(ins.clone());

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

    /// Start over (used with loops, etc.)
    pub fn start_over(&mut self) -> bool {
        let res = !self.executed.is_empty();
        if res {
            while !self.instructions.is_empty() {
                self.executed.push_back(self.instructions.pop_front().unwrap());
            }
            swap(&mut self.executed, &mut self.instructions);
        }
        res
    }

    #[inline(always)]
    /// Append instructions.
    pub fn append(&mut self, instructions: Vector<Arc<dyn Instruction>>) {
        self.instructions.append(instructions);
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

    /// Is a suspend operation?
    /// This kind of operation will not get executed, nor will it be placed in the instruction stack.
    /// It will prompt the rotating of processes though... so make sure to include them!
    fn suspend_op(&self) -> bool {
        false
    }
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
