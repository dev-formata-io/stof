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


#[derive(Debug, Default)]
/// Process Env.
pub struct ProcEnv {
    pub pid: SId,
    pub self_stack: Vec<NodeRef>,
    pub call_stack: Vec<DataRef>,
    pub new_stack: Vec<NodeRef>,
    pub stack: Vec<Variable>,
    pub table: Box<SymbolTable>,
}


#[derive(Debug, Default)]
/// Process.
pub struct Process {
    pub env: ProcEnv,
    pub instructions: Instructions,
}
impl Process {
    #[inline(always)]
    /// Progress this process by one.
    /// If there's more, a MoreProc state will be returned.
    pub fn progress(&mut self, graph: &mut Graph) -> Result<State, Error> {
        self.instructions.exec(&mut self.env, graph)
    }
}
