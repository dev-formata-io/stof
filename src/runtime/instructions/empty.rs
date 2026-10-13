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
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use crate::{model::Graph, runtime::{instruction::{Instruction, Instructions}, instructions::{block::Block, call::FuncCall, Base}, proc::ProcEnv, Error}};


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


#[derive(Debug, Clone, Serialize, Deserialize)]
/// A value is needed from these instructions (Ex. a declaration's value): if they leave nothing on the stack,
/// like a call to a function that returns nothing, the value is null instead of a missing stack value.
pub struct ValueIns {
    pub ins: Arc<dyn Instruction>,
}
#[typetag::serde(name = "ValueIns")]
impl Instruction for ValueIns {
    fn exec(&self, env: &mut ProcEnv, _graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        let mut instructions = Instructions::default();
        instructions.push(self.ins.clone());
        let size = env.stack.len();
        instructions.push(match ENSURE_VALUE.get(size) {
            Some(ins) => ins.clone(), // shared for the usual stack sizes, so a call's value costs no allocation
            None => Arc::new(Base::EnsureValue(size)),
        });
        Ok(Some(instructions))
    }
}

lazy_static! {
    static ref ENSURE_VALUE: Vec<Arc<dyn Instruction>> = (0..64).map(|size| Arc::new(Base::EnsureValue(size)) as Arc<dyn Instruction>).collect();
}

/// The value of an expression that might leave nothing on the stack: a call (the function may return nothing)
/// or an await (the process may finish without a value). Anything else is returned as it is, at no extra cost.
pub fn value_of(ins: Arc<dyn Instruction>) -> Arc<dyn Instruction> {
    let any = ins.as_dyn_any();
    let last_may_be_empty = |ins: &Arc<dyn Instruction>| {
        let any = ins.as_dyn_any();
        any.downcast_ref::<FuncCall>().is_some() || matches!(any.downcast_ref::<Base>(), Some(Base::CtrlAwait))
    };
    let may_be_empty = any.downcast_ref::<FuncCall>().is_some()
        || any.downcast_ref::<Block>().is_some_and(|block| block.ins.last().is_some_and(last_may_be_empty));
    if may_be_empty { Arc::new(ValueIns { ins }) } else { ins }
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
