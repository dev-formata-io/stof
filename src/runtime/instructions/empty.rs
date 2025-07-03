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
use arcstr::ArcStr;
use imbl::Vector;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use crate::{model::Graph, runtime::{instruction::{Instruction, Instructions}, instructions::{ops::{Op, OpIns}, whiles::WhileIns, Base, NOOP, POP_STACK}, proc::ProcEnv, Error, Val}};


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// Empty instructions are those that (after all exec) do not modify the current stack.
/// This inserts the instructions to make sure this is the case.
/// Ex. { .. } instruction blocks that have a return value but no declaration or other use (empty expr with a ;).
pub struct EmptyIns {
    pub ins: Vector<Arc<dyn Instruction>>,
}
#[typetag::serde(name = "EmptyIns")]
impl Instruction for EmptyIns {
    fn exec(&self, instructions: &mut Instructions, _env: &mut ProcEnv, _graph: &mut Graph) -> Result<(), Error> {
        let marker: ArcStr = nanoid!(5).into();

        // push a marker value onto the stack
        instructions.push(Arc::new(Base::Literal(Val::Str(marker.clone()))));

        // do instructions
        instructions.append(&self.ins);

        // while the current stack val != marker, drop the value
        let mut while_ins = WhileIns {
            continue_tag: marker.clone(), // doesn't matter
            break_tag: marker.clone(), // doesn't matter
            declare: None,
            inc: None,

            // while (current != marker)
            test: Arc::new(OpIns {
                lhs: NOOP.clone(), // current stack value
                op: Op::Neq,
                rhs: Arc::new(Base::Literal(Val::Str(marker))),
            }),
            ins: Default::default(),
        };
        while_ins.ins.push_back(POP_STACK.clone()); // pop the current value off the stack
        instructions.push(Arc::new(while_ins));

        // drop the marker value from the stack
        instructions.push(POP_STACK.clone());

        // now the stack is the same as when we started!
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use arcstr::literal;
    use crate::{model::Graph, runtime::{instructions::{empty::EmptyIns, Base}, Runtime, Val}};

    #[test]
    fn empty() {
        let mut empty = EmptyIns::default();
        empty.ins.push_back(Arc::new(Base::Literal(Val::Bool(true))));
        empty.ins.push_back(Arc::new(Base::Literal(Val::Str(literal!("yo, dude")))));

        let mut graph = Graph::default();
        let res = Runtime::eval(&mut graph, Arc::new(empty)).expect("expected pass");
        assert_eq!(res, Val::Void);
    }
}
