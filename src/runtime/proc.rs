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

use crate::{model::{DataRef, Graph, NodeRef, SId}, runtime::{instruction::{Instructions, State}, table::SymbolTable, Error, Variable}};


#[derive(Debug)]
/// Process State.
pub enum ProcState {
    Done,
    More,
    Wait(SId),
}


#[derive(Clone, Debug, Default)]
/// Process Env.
pub struct ProcEnv {
    pub pid: SId,
    pub self_stack: Vec<NodeRef>,
    pub call_stack: Vec<DataRef>,
    pub new_stack: Vec<NodeRef>,
    pub stack: Vec<Variable>,
    pub table: Box<SymbolTable>,
}


#[derive(Clone, Debug, Default)]
/// Process.
pub struct Process {
    pub env: ProcEnv,
    pub instruction_stack: Vec<Instructions>,
    pub result: Option<Variable>,
    pub error: Option<Error>,
    pub waiting: Option<SId>,
}
impl From<Instructions> for Process {
    fn from(value: Instructions) -> Self {
        Self {
            instruction_stack: vec![value],
            ..Default::default()
        }
    }
}
impl Process {
    /// Progress this process by one.
    /// If there's more, a MoreProc state will be returned.
    pub fn progress(&mut self, graph: &mut Graph) -> Result<ProcState, Error> {
        if self.instruction_stack.is_empty() {
            Ok(ProcState::Done)
        } else {
            match self.instruction_stack.last_mut().unwrap().exec(&mut self.env, graph) {
                Ok(state) => {
                    match state {
                        State::None => {
                            while !self.instruction_stack.is_empty() && !self.instruction_stack.last().unwrap().more() {
                                self.instruction_stack.pop();
                            }
                            if self.instruction_stack.is_empty() {
                                Ok(ProcState::Done)
                            } else {
                                Ok(ProcState::More)
                            }
                        },
                        State::Return(pushed) => {
                            self.instruction_stack.pop();
                            while !self.instruction_stack.is_empty() && !self.instruction_stack.last().unwrap().more() {
                                self.instruction_stack.pop();
                            }
                            if self.instruction_stack.is_empty() {
                                if pushed {
                                    if let Some(var) = self.env.stack.pop() {
                                        self.result = Some(var);
                                        Ok(ProcState::Done)
                                    } else {
                                        Ok(ProcState::Done)
                                    }
                                } else {
                                    Ok(ProcState::Done)
                                }
                            } else {
                                // In this case, it's like Pop.
                                // Any function calls are expected to have the stack pop in thier instructions.
                                Ok(ProcState::More)
                            }
                        },
                        State::Push(instructions) => {
                            self.instruction_stack.push(instructions);
                            Ok(ProcState::More)
                        },
                        State::Pop => {
                            self.instruction_stack.pop();
                            while !self.instruction_stack.is_empty() && !self.instruction_stack.last().unwrap().more() {
                                self.instruction_stack.pop();
                            }
                            if self.instruction_stack.is_empty() {
                                Ok(ProcState::Done)
                            } else {
                                Ok(ProcState::More)
                            }
                        },
                        State::StartOver => {
                            self.instruction_stack.last_mut().unwrap().start_over();
                            Ok(ProcState::More)
                        },
                    }
                },
                Err(error) => {
                    Err(error)
                }
            }
        }
    }
}
