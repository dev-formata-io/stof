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
use imbl::Vector;
use serde::{Deserialize, Serialize};
use crate::{model::Graph, runtime::{instruction::{Instruction, Instructions}, instructions::{POP_SYMBOL_SCOPE, PUSH_SYMBOL_SCOPE}, proc::ProcEnv, Error}};


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// Block of instructions to be executed.
pub struct Block {
    pub scoped: bool,
    pub ins: Vector<Arc<dyn Instruction>>,
}
#[typetag::serde(name = "Block")]
impl Instruction for Block {
    fn exec(&self, instructions: &mut Instructions, _env: &mut ProcEnv, _graph: &mut Graph) -> Result<(), Error> {
        if self.scoped { instructions.push(PUSH_SYMBOL_SCOPE.clone()); }
        instructions.append(&self.ins);
        if self.scoped { instructions.push(POP_SYMBOL_SCOPE.clone()); }
        Ok(())
    }
}
