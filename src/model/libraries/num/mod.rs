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
use arcstr::{literal, ArcStr};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use crate::{model::{num::{abs::num_abs, iter::{num_at, num_len}, maxmin::{num_max, num_min}}, Graph}, runtime::{instruction::{Instruction, Instructions}, proc::ProcEnv, Error, Variable}};

mod abs;
mod maxmin;
mod iter;


/// Add the number library to a graph.
pub fn insert_number_lib(graph: &mut Graph) {
    graph.insert_libfunc(num_abs());
    
    graph.insert_libfunc(num_max());
    graph.insert_libfunc(num_min());

    graph.insert_libfunc(num_len());
    graph.insert_libfunc(num_at());
}


/// Library name.
pub(self) const NUM_LIB: ArcStr = literal!("Num");


// Static instructions.
lazy_static! {
    pub(self) static ref ABS: Arc<dyn Instruction> = Arc::new(NumIns::Abs);
    pub(self) static ref AT: Arc<dyn Instruction> = Arc::new(NumIns::At);
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Number Instruction.
pub enum NumIns {
    Abs,

    Max(usize),
    Min(usize),

    At,
}
#[typetag::serde(name = "NumIns")]
impl Instruction for NumIns {
    fn exec(&self, env: &mut ProcEnv, graph: &mut Graph) -> Result<Option<Instructions> , Error> {
        match self {
            Self::Abs => {
                if let Some(var) = env.stack.pop() {
                    if let Some(num) = var.val.write().try_num() {
                        num.abs()?;
                    } else {
                        return Err(Error::NumAbsStack)
                    }
                    env.stack.push(var);
                } else {
                    return Err(Error::NumAbsStack)
                }
            },
            Self::Max(stack_count) => {
                let mut res = None;
                for _ in 0..*stack_count {
                    if let Some(var) = env.stack.pop() {
                        let max_var = var.val.read().maximum(graph)?;
                        if let Some(current) = res {
                            let gt = max_var.gt(&current, &graph)?;
                            if gt.truthy() {
                                res = Some(max_var);
                            } else {
                                res = Some(current);
                            }
                        } else {
                            res = Some(max_var);
                        }
                    }
                }
                if let Some(res) = res {
                    env.stack.push(Variable::val(res));
                }
            },
            Self::Min(stack_count) => {
                let mut res = None;
                for _ in 0..*stack_count {
                    if let Some(var) = env.stack.pop() {
                        let min_var = var.val.read().minimum(graph)?;
                        if let Some(current) = res {
                            let lt = min_var.lt(&current, &graph)?;
                            if lt.truthy() {
                                res = Some(min_var);
                            } else {
                                res = Some(current);
                            }
                        } else {
                            res = Some(min_var);
                        }
                    }
                }
                if let Some(res) = res {
                    env.stack.push(Variable::val(res));
                }
            },
            Self::At => {
                if let Some(index_var) = env.stack.pop() {
                    if let Some(val_var) = env.stack.pop() {
                        let lt = index_var.lt(&val_var, &graph)?;
                        if lt.truthy() {
                            env.stack.push(index_var);
                        } else {
                            env.stack.push(val_var);
                        }
                        return Ok(None);
                    }
                }
                return Err(Error::NumAtStack);
            },
        }
        Ok(None)
    }
}
