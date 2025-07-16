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
use serde::{Deserialize, Serialize};
use crate::{model::{DataRef, Func, Graph}, runtime::{instruction::{Instruction, Instructions}, instructions::Base, proc::ProcEnv, Error, Val}};


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Arrow function literal value.
/// Only inserts the function once into the graph, then references it over and over.
/// Replaces itself with a literal instruction for efficiency.
pub struct FuncLit {
    pub dref: DataRef,
    pub func: Func,
}
#[typetag::serde(name = "FuncLit")]
impl Instruction for FuncLit {
    fn exec(&self, env: &mut ProcEnv, graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        let mut instructions = Instructions::default();

        if self.dref.data_exists(&graph) {
            instructions.push(Arc::new(Base::Literal(Val::Fn(self.dref.clone()))));
        } else {
            let self_ptr = env.self_ptr();
            if let Some(dref) = graph.insert_stof_data(&self_ptr, &self.dref, Box::new(self.func.clone()), Some(self.dref.clone())) {
                instructions.push(Arc::new(Base::Literal(Val::Fn(dref))));
            }
        }

        Ok(Some(instructions))
    }
}
