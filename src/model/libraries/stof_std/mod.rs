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

use std::{sync::Arc, time::Duration};
use arcstr::{literal, ArcStr};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use crate::{model::{stof_std::{assert::{assert, assert_eq, assert_neq, assert_not}, print::{dbg, err, pln}, sleep::stof_sleep}, Graph}, runtime::{instruction::{Instruction, Instructions}, instructions::Base, proc::ProcEnv, Error, Type, Units}};

mod print;
mod sleep;
mod assert;


/// Add the std library to a graph.
pub fn stof_std_lib(graph: &mut Graph) {
    graph.insert_libfunc(pln());
    graph.insert_libfunc(dbg());
    graph.insert_libfunc(err());
    graph.insert_libfunc(stof_sleep());

    graph.insert_libfunc(assert());
    graph.insert_libfunc(assert_not());
    graph.insert_libfunc(assert_eq());
    graph.insert_libfunc(assert_neq());
}


/// Library name.
pub(self) const STD_LIB: ArcStr = literal!("Std");


// Static instructions.
lazy_static! {
    pub(self) static ref SLEEP: Arc<dyn Instruction> = Arc::new(StdIns::Sleep);
    pub(self) static ref ASSERT: Arc<dyn Instruction> = Arc::new(StdIns::Assert);
    pub(self) static ref ASSERT_NOT: Arc<dyn Instruction> = Arc::new(StdIns::AssertNot);
    pub(self) static ref ASSERT_EQ: Arc<dyn Instruction> = Arc::new(StdIns::AssertEq);
    pub(self) static ref ASSERT_NEQ: Arc<dyn Instruction> = Arc::new(StdIns::AssertNeq);
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Standard Lib Instruction.
pub enum StdIns {
    Pln(usize),
    Dbg(usize),
    Err(usize),

    Sleep,

    Assert,
    AssertNot,
    AssertEq,
    AssertNeq,
}
#[typetag::serde(name = "StdIns")]
impl Instruction for StdIns {
    fn exec(&self, env: &mut ProcEnv, graph: &mut Graph) -> Result<Option<Instructions> , Error> {
        match self {
            Self::Pln(arg_count) => {
                let mut values = Vec::new();
                for _ in 0..*arg_count {
                    if let Some(var) = env.stack.pop() {
                        values.push(var);
                    } else {
                        return Err(Error::StackError);
                    }
                }
                let mut output = Vec::new();
                let mut seen_str = false;
                for var in values.into_iter().rev() {
                    if !seen_str {
                        if var.gen_type() == Type::Str { seen_str = true; }
                    }
                    let out = var.val.read().print(&graph);
                    output.push(out);
                }
                let mut sep = "";
                if !seen_str { sep = ", " }
                println!("{}", output.join(sep));
            },
            Self::Dbg(arg_count) => {
                let mut values = Vec::new();
                for _ in 0..*arg_count {
                    if let Some(var) = env.stack.pop() {
                        values.push(var);
                    } else {
                        return Err(Error::StackError);
                    }
                }
                let mut output = Vec::new();
                let mut seen_str = false;
                for var in values.into_iter().rev() {
                    if !seen_str {
                        if var.gen_type() == Type::Str { seen_str = true; }
                    }
                    let out = var.val.read().debug(&graph);
                    output.push(out);
                }
                let mut sep = "";
                if !seen_str { sep = ", " }
                println!("{}", output.join(sep));
            },
            Self::Err(arg_count) => {
                let mut values = Vec::new();
                for _ in 0..*arg_count {
                    if let Some(var) = env.stack.pop() {
                        values.push(var);
                    } else {
                        return Err(Error::StackError);
                    }
                }
                let mut output = Vec::new();
                let mut seen_str = false;
                for var in values.into_iter().rev() {
                    if !seen_str {
                        if var.gen_type() == Type::Str { seen_str = true; }
                    }
                    let out = var.val.read().print(&graph);
                    output.push(out);
                }
                let mut sep = "";
                if !seen_str { sep = ", " }
                eprintln!("{}", output.join(sep));
            }
            
            Self::Sleep => {
                let duration;
                if let Some(val) = env.stack.pop() {
                    if let Some(num) = val.val.write().try_num() {
                        duration = num.float(Some(Units::Milliseconds));
                    } else {
                        return Err(Error::StackError);
                    }
                } else {
                    return Err(Error::StackError);
                }

                let mut instructions = Instructions::default();
                instructions.push(Arc::new(Base::CtrlSleepFor(Duration::from_millis(duration.abs() as u64))));
                return Ok(Some(instructions));
            },

            Self::Assert => {
                if let Some(val) = env.stack.pop() {
                    if !val.val.read().truthy() {
                        let message = format!("{} is not truthy", val.val.read().print(&graph));
                        return Err(Error::AssertFailed(message));
                    }
                }
            },
            Self::AssertNot => {
                if let Some(val) = env.stack.pop() {
                    if val.val.read().truthy() {
                        let message = format!("{} is truthy", val.val.read().print(&graph));
                        return Err(Error::AssertNotFailed(message));
                    }
                }
            },
            Self::AssertEq => {
                if let Some(val) = env.stack.pop() {
                    if let Some(other) = env.stack.pop() {
                        if let Ok(res) = val.equal(&other) {
                            if !res.val.read().truthy() {
                                let message = format!("{} does not equal {}", val.val.read().print(&graph), other.val.read().print(&graph));
                                return Err(Error::AssertEqFailed(message));
                            }
                        }
                    }
                }
            },
            Self::AssertNeq => {
                if let Some(val) = env.stack.pop() {
                    if let Some(other) = env.stack.pop() {
                        if let Ok(res) = val.equal(&other) {
                            if res.val.read().truthy() {
                                let message = format!("{} equals {}", val.val.read().print(&graph), other.val.read().print(&graph));
                                return Err(Error::AssertNotEqFailed(message));
                            }
                        }
                    }
                }
            },
        }
        Ok(None)
    }
}
