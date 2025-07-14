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
use arcstr::ArcStr;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use crate::{model::Graph, runtime::{instruction::{Instruction, Instructions}, instructions::{Base, ConsumeStack, DUPLICATE, IS_NULL, POP_STACK}, proc::ProcEnv, Error}};


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Nullcheck instruction.
pub struct NullcheckIns {
    pub ins: Arc<dyn Instruction>,
    pub ifnull: Arc<dyn Instruction>,
}
#[typetag::serde(name = "NullcheckIns")]
impl Instruction for NullcheckIns {
    fn exec(&self, _env: &mut ProcEnv, _graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        let mut instructions = Instructions::default();

        instructions.push(self.ins.clone());

        // Duplicate the value, check if its null, and go to end if not
        let end_tag: ArcStr = nanoid!(10).into();
        instructions.push(DUPLICATE.clone());
        instructions.push(IS_NULL.clone());
        instructions.push(Arc::new(Base::CtrlForwardToIfNotTruthy(end_tag.clone(), ConsumeStack::Consume)));

        // In here, we have a null value on the stack, so drop it and do the ifnull instructions
        instructions.push(POP_STACK.clone());
        instructions.push(self.ifnull.clone());
        
        instructions.push(Arc::new(Base::Tag(end_tag)));

        Ok(Some(instructions))
    }
}
