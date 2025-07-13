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

use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use crate::{model::{Func, Graph}, runtime::{instruction::{Instruction, Instructions}, proc::ProcEnv, Error, Val, Variable}};


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Arrow function literal value.
pub struct FuncLit {
    pub func: Func,
}
#[typetag::serde(name = "FuncLit")]
impl Instruction for FuncLit {
    fn exec(&self, env: &mut ProcEnv, graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        let self_ptr = env.self_ptr();
        let name = nanoid!(7);
        if let Some(dref) = graph.insert_stof_data(&self_ptr, &name, Box::new(self.func.clone()), None) {
            env.stack.push(Variable::val(Val::Fn(dref)));
        }
        Ok(None)
    }
}
