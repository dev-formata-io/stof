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

use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::{model::Graph, runtime::{instruction::{Instruction, Instructions}, instructions::Base, proc::ProcEnv, Error}};


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Empty instructions are those that (after all exec) do not modify the current stack.
/// This inserts the instructions to make sure this is the case.
/// Ex. { .. } instruction blocks that have a return value but no declaration or other use (empty expr with a ;).
pub struct EmptyIns {
    pub ins: Arc<dyn Instruction>,
}
#[typetag::serde(name = "EmptyIns")]
impl Instruction for EmptyIns {
    fn exec(&self, env: &mut ProcEnv, _graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        let mut instructions = Instructions::default();
        let size = env.stack.len();
        instructions.push(self.ins.clone());
        instructions.push(Arc::new(Base::PopUntilStackCount(size)));
        Ok(Some(instructions))
    }
}


#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use arcstr::literal;
    use crate::{model::Graph, runtime::{instructions::{block::Block, empty::EmptyIns, Base}, Runtime, Val}};

    #[test]
    fn empty() {
        let mut empty = Block::default();
        empty.ins.push_back(Arc::new(Base::Literal(Val::Bool(true))));
        empty.ins.push_back(Arc::new(Base::Literal(Val::Str(literal!("yo, dude")))));

        let mut graph = Graph::default();
        let res = Runtime::eval(&mut graph, Arc::new(EmptyIns { ins: Arc::new(empty) })).expect("expected pass");
        assert_eq!(res, Val::Void);
    }
}
