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
use crate::{model::Graph, runtime::{instruction::{Instruction, Instructions}, instructions::{END_TAG, POP_SYMBOL_SCOPE, PUSH_SYMBOL_SCOPE, START_TAG}, proc::ProcEnv, Error}};


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Block of instructions to be executed.
pub struct Block {
    /// Places default start and end tags for continue and break instructions.
    /// To create custom tags, just add them to the start and end of ins in addition.
    pub tagged: bool,
    pub ins: Vector<Arc<dyn Instruction>>,
    pub finally: Option<Vector<Arc<dyn Instruction>>>,
}
#[typetag::serde(name = "Block")]
impl Instruction for Block {
    fn exec(&self, instructions: &mut Instructions, _env: &mut ProcEnv, _graph: &mut Graph) -> Result<(), Error> {
        // start a new scope for this block
        instructions.push(PUSH_SYMBOL_SCOPE.clone());

        if self.tagged { instructions.push(START_TAG.clone()); }
        instructions.append(&self.ins);
        if self.tagged { instructions.push(END_TAG.clone()); }

        if let Some(finally) = &self.finally {
            instructions.append(finally);
        }

        // end the scope for this block
        instructions.push(POP_SYMBOL_SCOPE.clone());
        
        Ok(())
    }
}
