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

use std::{ops::DerefMut, sync::Arc};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use crate::{model::Graph, runtime::{instruction::{Instruction, Instructions}, proc::ProcEnv, Error, Val, Variable}};


lazy_static! {
    pub static ref NEW_SET: Arc<dyn Instruction> = Arc::new(SetIns::NewSet);
    pub static ref PUSH_SET: Arc<dyn Instruction> = Arc::new(SetIns::PushSet);
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Set creation instructions.
pub enum SetIns {
    // Low-level for construction
    NewSet,
    PushSet,

    // High-level
    AppendSet(Arc<dyn Instruction>), // evaluate and add to the stack (push)
}
#[typetag::serde(name = "SetIns")]
impl Instruction for SetIns {
    fn exec(&self, env: &mut ProcEnv, _graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        match self {
            Self::NewSet => {
                env.stack.push(Variable::val(Val::Set(Default::default())));
            },
            Self::PushSet => {
                if let Some(push_var) = env.stack.pop() {
                    if let Some(set_var) = env.stack.pop() {
                        {
                            let mut val = set_var.val.write();
                            let val = val.deref_mut();
                            match &mut *val {
                                Val::Set(values) => {
                                    values.insert(push_var.val);
                                },
                                _ => {}
                            }
                        }
                        env.stack.push(set_var);
                    }
                }
            },

            /*****************************************************************************
             * High-level.
             *****************************************************************************/
            Self::AppendSet(ins) => {
                let mut instructions = Instructions::default();
                instructions.push(ins.clone());
                instructions.push(PUSH_SET.clone());
                return Ok(Some(instructions));
            },
        }
        Ok(None)
    }
}
