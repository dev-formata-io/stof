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
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use crate::{model::Graph, runtime::{instruction::{Instruction, Instructions}, instructions::{Base, ConsumeStack}, proc::ProcEnv, Error}};


#[derive(Debug, Clone, Serialize, Deserialize)]
/// If statement.
pub struct IfIns {
    if_test: Option<Arc<dyn Instruction>>,
    if_ins: Arc<dyn Instruction>,
    el_ins: Arc<dyn Instruction>, // place another if here for "else if"
}
#[typetag::serde(name = "IfIns")]
impl Instruction for IfIns {
    fn exec(&self, instructions: &mut Instructions, _env: &mut ProcEnv, _graph: &mut Graph) -> Result<(), Error> {
        if let Some(test) = &self.if_test {
            instructions.push(test.clone());
        }
        
        let if_tag: ArcStr = nanoid!(10).into();
        let else_tag: ArcStr = nanoid!(10).into();
        instructions.push(Arc::new(Base::CtrlForwardToIfNotTruthy(else_tag.clone(), ConsumeStack::Consume)));
        instructions.push(self.if_ins.clone());
        instructions.push(Arc::new(Base::CtrlForwardTo(if_tag.clone())));
        instructions.push(Arc::new(Base::Tag(else_tag)));
        instructions.push(self.el_ins.clone());
        instructions.push(Arc::new(Base::Tag(if_tag)));
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use arcstr::literal;
    use crate::{model::Graph, runtime::{instructions::{ifs::IfIns, Base}, Runtime, Val}};

    #[test]
    fn if_true() {
        let ins = IfIns {
            if_test: Some(Arc::new(Base::Literal(Val::Bool(true)))),
            if_ins: Arc::new(Base::Literal(Val::Str(literal!("a")))),
            el_ins: Arc::new(Base::Literal(Val::Str(literal!("b")))),
        };
        let mut graph = Graph::default();
        let res = Runtime::eval(&mut graph, Arc::new(ins)).expect("expected pass");
        assert_eq!(res, "a".into());
    }

    #[test]
    fn if_false() {
        let ins = IfIns {
            if_test: Some(Arc::new(Base::Literal(Val::Bool(false)))),
            if_ins: Arc::new(Base::Literal(Val::Str(literal!("a")))),
            el_ins: Arc::new(Base::Literal(Val::Str(literal!("b")))),
        };
        let mut graph = Graph::default();
        let res = Runtime::eval(&mut graph, Arc::new(ins)).expect("expected pass");
        assert_eq!(res, "b".into());
    }
}
