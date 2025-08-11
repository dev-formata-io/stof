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
use crate::{model::Graph, runtime::{instruction::{Instruction, Instructions}, instructions::{Base, END_TRY}, proc::ProcEnv, Error}};


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// Try Catch instructions.
pub struct TryCatchIns {
    pub try_ins: Vector<Arc<dyn Instruction>>,
    pub err_ins: Vector<Arc<dyn Instruction>>,
    pub catch_ins: Vector<Arc<dyn Instruction>>,
}
#[typetag::serde(name = "TryCatchIns")]
impl Instruction for TryCatchIns {
    fn exec(&self, env: &mut ProcEnv, _graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        let catch_tag: ArcStr = nanoid!(10).into();
        let end_tag: ArcStr = nanoid!(10).into();
        let size = env.stack.len();

        let mut instructions = Instructions::default();
        instructions.push(Arc::new(Base::CtrlTry(catch_tag.clone()))); // Go here when theres an error & inc try count
        instructions.append(&self.try_ins);

        instructions.push(Arc::new(Base::CtrlForwardTo(end_tag.clone())));
        instructions.push(Arc::new(Base::Tag(catch_tag))); // now theres an error on the stack!
        instructions.append(&self.err_ins);
        instructions.push(Arc::new(Base::PopUntilStackCount(size)));
        instructions.append(&self.catch_ins);

        instructions.push(Arc::new(Base::Tag(end_tag)));
        instructions.push(END_TRY.clone()); // de-increment the try count.
        Ok(Some(instructions))
    }
}


#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use crate::{model::Graph, runtime::{instructions::{trycatch::TryCatchIns, Base}, Num, Runtime, Val}};


    #[test]
    fn no_catch() {
        let mut try_catch = TryCatchIns::default();
        try_catch.try_ins.push_back(Arc::new(Base::Literal(Val::Num(Num::Float(42.2)))));
        try_catch.catch_ins.push_back(Arc::new(Base::Literal(Val::Num(Num::Float(1242.2)))));

        let mut graph = Graph::default();
        let res = Runtime::eval(&mut graph, Arc::new(try_catch)).unwrap();
        assert_eq!(res, 42.2.into());
    }
}
