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
    pub static ref NEW_MAP: Arc<dyn Instruction> = Arc::new(MapIns::NewMap);
    pub static ref PUSH_MAP: Arc<dyn Instruction> = Arc::new(MapIns::PushMap);
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Map creation instructions.
pub enum MapIns {
    // Low-level for construction
    NewMap,
    PushMap,

    // High-level
    AppendMap((Arc<dyn Instruction>, Arc<dyn Instruction>)), // evaluate and add to the stack (push)
}
#[typetag::serde(name = "MapIns")]
impl Instruction for MapIns {
    fn exec(&self, instructions: &mut Instructions, env: &mut ProcEnv, _graph: &mut Graph) -> Result<(), Error> {
        match self {
            Self::NewMap => {
                env.stack.push(Variable::val(Val::Map(Default::default())));
            },
            Self::PushMap => {
                if let Some(value_var) = env.stack.pop() {
                    if let Some(key_var) = env.stack.pop() {
                        if let Some(map_var) = env.stack.pop() {
                            {
                                let mut val = map_var.val.write();
                                let val = val.deref_mut();
                                match &mut *val {
                                    Val::Map(map) => {
                                        map.insert(key_var.val, value_var.val);
                                    },
                                    _ => {}
                                }
                            }
                            env.stack.push(map_var);
                        }
                    }
                }
            },

            /*****************************************************************************
             * High-level.
             *****************************************************************************/
            Self::AppendMap((key, value)) => {
                instructions.push(key.clone());
                instructions.push(value.clone());
                instructions.push(PUSH_MAP.clone());
            },
        }
        Ok(())
    }
}
