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
use crate::{model::Graph, runtime::{instruction::{Instruction, Instructions}, instructions::{Base, ConsumeStack, POP_LOOP, PUSH_SYMBOL_SCOPE, TRUTHY}, proc::ProcEnv, Error}};


#[derive(Debug, Clone, Serialize, Deserialize)]
/// While statement.
pub struct WhileIns {
    // optional custom tag for this loop
    pub tag: Option<ArcStr>,

    // the real stuff
    pub test: Arc<dyn Instruction>,
    pub ins: Vector<Arc<dyn Instruction>>,

    // added control flow stuff for flexibility
    pub declare: Option<Arc<dyn Instruction>>,
    pub inc: Option<Arc<dyn Instruction>>,
}
#[typetag::serde(name = "WhileIns")]
impl Instruction for WhileIns {
    fn exec(&self, env: &mut ProcEnv, _graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        // Create a tag for this loop (control statements)
        let mut tag: ArcStr = nanoid!(10).into();
        if let Some(ctag) = &self.tag {
            tag = ctag.clone();
        }

        // Create break and continue tags
        let continue_tag: ArcStr = format!("{}_con", &tag).into();
        let break_tag: ArcStr = format!("{}_brk", &tag).into();

        // Record scopes for poping at end
        let scope_count = env.table.scopes.len();

        let mut instructions = Instructions::default();
        instructions.push(Arc::new(Base::PushLoop(tag)));

        if let Some(declare) = &self.declare {
            instructions.push(declare.clone());
        }
        
        let top_tag: ArcStr = nanoid!(10).into();
        let end_tag: ArcStr = nanoid!(10).into();

        instructions.push(Arc::new(Base::Tag(top_tag.clone())));
        {
            // Test if the value is truthy, go to end_tag if not
            instructions.push(self.test.clone());
            instructions.push(TRUTHY.clone());
            instructions.push(Arc::new(Base::CtrlForwardToIfNotTruthy(end_tag.clone(), ConsumeStack::Consume)));
            
            // Do the thing
            instructions.push(PUSH_SYMBOL_SCOPE.clone());
            instructions.append(&self.ins);

            // Continue statements will go to here
            instructions.push(Arc::new(Base::Tag(continue_tag)));
            instructions.push(Arc::new(Base::PopSymbolScopeUntilDepth(scope_count + 1))); // take loop count into consideration

            // If we have an inc expr, do that now before we start the loop again
            if let Some(inc) = &self.inc {
                instructions.push(inc.clone());
            }

            // Go back to the top
            instructions.push(Arc::new(Base::CtrlBackTo(top_tag)));
        }

        // Break statements will go here, as well as our jump if not truthy
        instructions.push(Arc::new(Base::Tag(break_tag)));
        instructions.push(Arc::new(Base::Tag(end_tag)));
        instructions.push(POP_LOOP.clone());
        instructions.push(Arc::new(Base::PopSymbolScopeUntilDepth(scope_count)));
        Ok(Some(instructions))
    }
}


#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use arcstr::literal;
    use crate::{model::Graph, runtime::{instructions::{block::Block, ops::{Op, OpIns}, whiles::WhileIns, Base}, Num, Runtime, Type, Val}};


    #[test]
    fn for_range_loop() {
        let mut outer_block = Block::default();
        outer_block.ins.push_back(Arc::new(Base::Literal(Val::Num(Num::Int(0)))));
        outer_block.ins.push_back(Arc::new(Base::DeclareVar(literal!("total"), Type::Void)));

        let mut declare = Block::default();
        declare.ins.push_back(Arc::new(Base::Literal(Val::Num(Num::Int(0)))));
        declare.ins.push_back(Arc::new(Base::DeclareVar(literal!("i"), Type::Void)));

        let mut test_block = Block::default();
        test_block.ins.push_back(Arc::new(OpIns {
            lhs: Arc::new(Base::LoadVariable(literal!("i"), false, false)),
            op: Op::Less,
            rhs: Arc::new(Base::Literal(Val::Num(Num::Int(100)))),
        }));

        let mut inc_block = Block::default();
        inc_block.ins.push_back(Arc::new(OpIns {
            lhs: Arc::new(Base::LoadVariable(literal!("i"), false, false)),
            op: Op::Add,
            rhs: Arc::new(Base::Literal(Val::Num(Num::Int(1)))),
        }));
        inc_block.ins.push_back(Arc::new(Base::SetVariable(literal!("i"))));

        // let total: int = 0; for (int i in 0..100) { total += 1; } total
        let mut while_loop = WhileIns {
            tag: None,
            test: Arc::new(test_block),
            ins: Default::default(),
            declare: Some(Arc::new(declare)),
            inc: Some(Arc::new(inc_block)),
        };
        //while_loop.ins.push_back(Arc::new(Base::CtrlForwardTo(literal!("continue"))));
        while_loop.ins.push_back(Arc::new(OpIns {
            lhs: Arc::new(Base::LoadVariable(literal!("total"), false, false)),
            op: Op::Add,
            rhs: Arc::new(Base::Literal(Val::Num(Num::Int(1)))),
        }));
        while_loop.ins.push_back(Arc::new(Base::SetVariable(literal!("total"))));
        //while_loop.ins.push_back(Arc::new(Base::CtrlForwardTo(literal!("break"))));

        outer_block.ins.push_back(Arc::new(while_loop));
        outer_block.ins.push_back(Arc::new(Base::LoadVariable(literal!("total"), false, false))); // return

        // run
        let mut graph = Graph::default();
        let res = Runtime::eval(&mut graph, Arc::new(outer_block)).expect("expected pass");
        assert_eq!(res, 100.into());
    }
}
