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
use crate::{model::{num::abs::num_abs, Graph}, runtime::{instruction::{Instruction, Instructions}, proc::ProcEnv, Error}};

mod abs;


/// Add the number library to a graph.
pub fn insert_number_lib(graph: &mut Graph) {
    graph.insert_libfunc(num_abs());
}


/// Library name.
pub(self) const NUM_LIB: ArcStr = literal!("Num");


// Static instructions.
lazy_static! {
    pub(self) static ref ABS: Arc<dyn Instruction> = Arc::new(NumIns::Abs);
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Number Instruction.
pub enum NumIns {
    Abs,
}
#[typetag::serde(name = "NumIns")]
impl Instruction for NumIns {
    fn exec(&self, env: &mut ProcEnv, _graph: &mut Graph) -> Result<Option<Instructions> , Error> {
        match self {
            Self::Abs => {
                if let Some(var) = env.stack.pop() {
                    if let Some(num) = var.val.write().try_num() {
                        num.abs()?;
                    } else {
                        return Err(Error::StackError)
                    }
                    env.stack.push(var);
                } else {
                    return Err(Error::StackError)
                }
            }
        }
        Ok(None)
    }
}
