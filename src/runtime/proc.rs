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
use serde::{Deserialize, Serialize};
use crate::{model::{DataRef, Graph, NodeRef, SId}, runtime::{instruction::{Instruction, Instructions}, table::SymbolTable, Error, Val}};


#[derive(Debug)]
/// Process Result.
pub enum ProcRes {
    Done,
    More,
    Wait(SId),
}


#[derive(Clone, Debug, Default, Serialize, Deserialize)]
/// Process Env.
pub struct ProcEnv {
    pub pid: SId,
    pub self_stack: Vec<NodeRef>,
    pub call_stack: Vec<DataRef>,
    pub new_stack: Vec<NodeRef>,
    pub stack: Vec<Val>,
    pub table: Box<SymbolTable>,

    // Setting this will put the process into a waiting mode
    pub spawn: Option<Box<Process>>,
}
impl ProcEnv {
    // Get the current self ptr.
    pub fn self_ptr(&self) -> NodeRef {
        self.self_stack.last().unwrap().clone()
    }
}


#[derive(Clone, Debug, Default, Serialize, Deserialize)]
/// Process.
pub struct Process {
    pub env: ProcEnv,
    pub instructions: Instructions,
    pub result: Option<Val>,
    pub error: Option<Error>,
    pub waiting: Option<SId>,
}
impl From<Instructions> for Process {
    fn from(value: Instructions) -> Self {
        Self {
            instructions: value,
            ..Default::default()
        }
    }
}
impl From<Arc<dyn Instruction>> for Process {
    fn from(value: Arc<dyn Instruction>) -> Self {
        Self {
            instructions: Instructions::from(value),
            ..Default::default()
        }
    }
}
impl Process {
    #[inline(always)]
    /// Progress this process by one instruction.
    pub(super) fn progress(&mut self, graph: &mut Graph) -> Result<ProcRes, Error> {
        match self.instructions.exec(&mut self.env, graph) {
            Ok(res) => {
                Ok(res)
            },
            Err(error) => {
                Err(error)
            }
        }
    }
}
