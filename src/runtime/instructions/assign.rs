//
// Copyright 2025 Formata, Inc. All rights reserved.
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

use arcstr::ArcStr;
use serde::{Deserialize, Serialize};
use crate::{model::{Graph, SPath}, runtime::{instruction::{Instruction, Instructions}, instructions::assign_path, proc::ProcEnv, Error}};


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Set a field on an object value (Ex. `self.customer(id).type = x`).
/// Requires the object, then the value, on the stack.
/// Sets the field at `path` under the object, creating it if needed, like a normal field assignment.
pub struct SetFieldIns {
    pub path: ArcStr,
}
#[typetag::serde(name = "SetFieldIns")]
impl Instruction for SetFieldIns {
    fn exec(&self, env: &mut ProcEnv, graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        let Some(var) = env.stack.pop() else { return Err(Error::StackError) };
        let Some(target) = env.stack.pop() else { return Err(Error::StackError) };
        let Some(obj) = target.try_obj() else {
            return Err(Error::Custom(format!("cannot set '{}' on a non-object value ({})", self.path, target.spec_type(graph).rt_type_of(graph)).into()));
        };
        assign_path(graph, SPath::from(self.path.as_str()), Some(obj), var)?;
        Ok(None)
    }
}
