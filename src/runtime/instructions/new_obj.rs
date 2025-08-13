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
use crate::{model::{Graph, SId}, runtime::{instruction::{Instruction, Instructions}, proc::ProcEnv, Error, Type, Val, Variable}};


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// New object instruction.
/// Creates an object and adds a ref on the stack.
pub struct NewObjIns {
    /// Set to true if a parent is on the stack.
    pub parent: bool,
    /// Cast the object to a type?
    pub cast_type: Option<Type>,
}
#[typetag::serde(name = "NewObjIns")]
impl Instruction for NewObjIns {
    fn exec(&self, env: &mut ProcEnv, graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        if !self.parent { // cannot have an "on" when creating a new root object
            if let Some(cast_type) = &self.cast_type {
                match cast_type {
                    Type::Obj(typename) => {
                        if typename.as_ref() == "root" {
                            // Special syntax for creating a root object instead of a sub-object
                            // Name will be re-assigned when using SetVariable Ex. MyRoot = new root {};
                            let name = SId::default();
                            let nref = graph.insert_root(name);
                            env.stack.push(Variable::val(Val::Obj(nref)));
                            return Ok(None);
                        }
                    },
                    _ => {}
                }
            }
        }

        let mut parent = Some(env.self_ptr());
        if self.parent {
            if let Some(prnt) = env.stack.pop() {
                if let Some(prnt) = prnt.try_obj() {
                    parent = Some(prnt);
                }
            }
        }

        let id = SId::default();
        let name = id.clone();
        let nref = graph.insert_node_id(name, id, parent, false);

        let mut val = Val::Obj(nref);
        if let Some(cast_type) = &self.cast_type {
            val.cast(cast_type, graph, Some(env.self_ptr()))?;
        }
        env.stack.push(Variable::val(val));
        Ok(None)
    }
}
