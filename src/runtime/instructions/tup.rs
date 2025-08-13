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

use std::{ops::{Deref, DerefMut}, sync::Arc};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use crate::{model::Graph, runtime::{instruction::{Instruction, Instructions}, proc::ProcEnv, Error, Num, Val, Variable}};


lazy_static! {
    pub static ref NEW_TUP: Arc<dyn Instruction> = Arc::new(TupIns::NewTup);
    pub static ref PUSH_TUP: Arc<dyn Instruction> = Arc::new(TupIns::PushTup);

    pub static ref LEN_TUP: Arc<dyn Instruction> = Arc::new(TupIns::Len);
    pub static ref AT_TUP: Arc<dyn Instruction> = Arc::new(TupIns::At);
    pub static ref AT_REF_TUP: Arc<dyn Instruction> = Arc::new(TupIns::AtRef);
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Tuple creation instructions.
pub enum TupIns {
    // Low-level for construction
    NewTup,
    PushTup,

    // High-level
    AppendTup(Arc<dyn Instruction>), // evaluate and add to the stack (push)

    // Library
    Len,
    At,
    AtRef,
}
#[typetag::serde(name = "TupIns")]
impl Instruction for TupIns {
    fn exec(&self, env: &mut ProcEnv, _graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        match self {
            Self::NewTup => {
                env.stack.push(Variable::val(Val::Tup(Default::default())));
                Ok(None)
            },
            Self::PushTup => {
                if let Some(push_var) = env.stack.pop() {
                    if let Some(tup_var) = env.stack.pop() {
                        {
                            let mut val = tup_var.val.write();
                            let val = val.deref_mut();
                            match &mut *val {
                                Val::Tup(values) => {
                                    values.push_back(push_var.val);
                                },
                                _ => {}
                            }
                        }
                        env.stack.push(tup_var);
                    }
                }
                Ok(None)
            },

            /*****************************************************************************
             * High-level.
             *****************************************************************************/
            Self::AppendTup(ins) => {
                let mut instructions = Instructions::default();
                instructions.push(ins.clone());
                instructions.push(PUSH_TUP.clone());
                Ok(Some(instructions))
            },

            /*****************************************************************************
             * Library.
             *****************************************************************************/
            Self::Len => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Tup(tup) => {
                            env.stack.push(Variable::val(Val::Num(Num::Int(tup.len() as i64))));
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::TupLen)
            },
            Self::At => {
                if let Some(index_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        match index_var.val.read().deref() {
                            Val::Num(num) => {
                                match var.val.read().deref() {
                                    Val::Tup(tup) => {
                                        if let Some(val) = tup.get(num.int() as usize) {
                                            env.stack.push(Variable::refval(val.duplicate(false)));
                                        } else {
                                            env.stack.push(Variable::val(Val::Null));
                                        }
                                        return Ok(None);
                                    },
                                    _ => {}
                                }
                            },
                            _ => {},
                        }
                    }
                }
                Err(Error::TupAt)
            },
            Self::AtRef => {
                if let Some(index_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        match index_var.val.read().deref() {
                            Val::Num(num) => {
                                match var.val.read().deref() {
                                    Val::Tup(tup) => {
                                        if let Some(val) = tup.get(num.int() as usize) {
                                            env.stack.push(Variable::refval(val.duplicate(true)));
                                        } else {
                                            env.stack.push(Variable::val(Val::Null));
                                        }
                                        return Ok(None);
                                    },
                                    _ => {}
                                }
                            },
                            _ => {},
                        }
                    }
                }
                Err(Error::TupAt)
            },
        }
    }
}
