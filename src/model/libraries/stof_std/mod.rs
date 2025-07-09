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

use arcstr::{literal, ArcStr};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use crate::{model::{stof_std::print::{dbg, err, pln}, Graph}, runtime::{instruction::{Instruction, Instructions}, proc::ProcEnv, Error, Type}};

mod print;


/// Add the std library to a graph.
pub fn stof_std_lib(graph: &mut Graph) {
    graph.insert_libfunc(pln());
    graph.insert_libfunc(dbg());
    graph.insert_libfunc(err());
}


/// Library name.
pub(self) const STD_LIB: ArcStr = literal!("Std");


// Static instructions.
lazy_static! {
    
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Standard Lib Instruction.
pub enum StdIns {
    Pln(usize),
    Dbg(usize),
    Err(usize),
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
        }
        Ok(None)
    }
}
