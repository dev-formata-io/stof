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
    pub static ref NEW_LIST: Arc<dyn Instruction> = Arc::new(ListIns::NewList);
    pub static ref PUSH_LIST: Arc<dyn Instruction> = Arc::new(ListIns::PushList);
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// List creation instructions.
pub enum ListIns {
    // Low-level for list construction
    NewList,
    PushList,

    // High-level
    AppendList(Arc<dyn Instruction>), // evaluate and add to the stack (push)
}
#[typetag::serde(name = "ListIns")]
impl Instruction for ListIns {
    fn exec(&self, instructions: &mut Instructions, env: &mut ProcEnv, _graph: &mut Graph) -> Result<(), Error> {
        match self {
            Self::NewList => {
                env.stack.push(Variable::val(Val::List(Default::default())));
            },
            Self::PushList => {
                if let Some(push_var) = env.stack.pop() {
                    if let Some(list_var) = env.stack.pop() {
                        {
                            let mut val = list_var.val.write();
                            let val = val.deref_mut();
                            match &mut *val {
                                Val::List(values) => {
                                    values.push_back(push_var.val);
                                },
                                _ => {}
                            }
                        }
                        env.stack.push(list_var);
                    }
                }
            },

            /*****************************************************************************
             * High-level.
             *****************************************************************************/
            Self::AppendList(ins) => {
                instructions.push(ins.clone());
                instructions.push(PUSH_LIST.clone());
            },
        }
        Ok(())
    }
}
