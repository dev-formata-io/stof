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

use serde::{Deserialize, Serialize};
use crate::{model::{DataRef, Func, Graph}, runtime::{instruction::{Instruction, Instructions}, proc::ProcEnv, Error, Val}};


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Call a function instruction.
/// An expression will add this as the next instruction after a lookup to an internal function.
pub struct FuncCall {
    pub func: DataRef,
    pub args: Vec<Val>,
    pub add_self: bool,
}
impl FuncCall {
    pub(self) fn add_self(&self, env: &mut ProcEnv, graph: &Graph) {
        if self.add_self {
            for nref in self.func.data_nodes(graph) {
                if nref.node_exists(graph) {
                    env.self_stack.push(nref);
                }
            }
            env.self_stack.push(graph.main_root().unwrap());
        }
    }
}


#[typetag::serde(name = "FuncCall")]
impl Instruction for FuncCall {
    fn exec(&self, instructions: &mut Instructions, env: &mut ProcEnv, graph: &mut Graph) -> Result<(), Error> {
        let params;
        let func_instructions;
        let rtype;
        if let Some(func) = graph.get_stof_data::<Func>(&self.func) {
            params = func.params.clone();
            func_instructions = func.instructions.clone();
            rtype = func.return_type.clone();
        } else {
            return Err(Error::FuncDne);
        }

        graph.ensure_main_root();
        self.add_self(env, graph);
        env.call_stack.push(self.func.clone());
        let mut arguments = self.args.clone();
            
        // Validate the number of parameters required to call the function
        if params.len() < arguments.len() {
            let mut index = self.args.len();
            while index < params.len() {
                let param = &params[index];
                if let Some(default) = &param.default_expr {
                    let value = default.exec(instructions, env, graph);
                    match value {
                        Ok(val) => arguments.push(val),
                        Err(error) => {
                            return Err(Error::FuncDefaultArg(Box::new(error)));
                        }
                    }
                } else {
                    break;
                }
                index += 1;
            }
        }
        if params.len() != arguments.len() {
            return Err(Error::FuncArgs);
        }

        Err(Error::FuncDne)
    }
}
