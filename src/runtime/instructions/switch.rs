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
use arcstr::ArcStr;
use imbl::Vector;
use nanoid::nanoid;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use crate::{model::Graph, runtime::{instruction::{Instruction, Instructions}, instructions::Base, proc::ProcEnv, Error, Val}};


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Switch statement.
pub struct SwitchIns {
    pub map: FxHashMap<Val, Arc<dyn Instruction>>,
    pub def: Option<Vector<Arc<dyn Instruction>>>,
}
#[typetag::serde(name = "SwitchIns")]
impl Instruction for SwitchIns {
    fn exec(&self, _env: &mut ProcEnv, _graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        let mut table = FxHashMap::default();
        let mut default = None;

        let end_tag: ArcStr = nanoid!(12).into();
        let mut table_instructions = Instructions::default();
        for (v, ins) in &self.map {
            let tag: ArcStr = nanoid!(12).into();
            table_instructions.push(Arc::new(Base::Tag(tag.clone())));
            table_instructions.push(ins.clone());
            table_instructions.push(Arc::new(Base::CtrlForwardTo(end_tag.clone())));
            table.insert(v.clone(), tag);
        }
        if let Some(def) = &self.def {
            let tag: ArcStr = nanoid!(12).into();
            table_instructions.push(Arc::new(Base::Tag(tag.clone())));
            table_instructions.append(def);
            default = Some(tag);
        }
        table_instructions.push(Arc::new(Base::Tag(end_tag.clone())));

        if !table.is_empty() || default.is_some() {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(Base::CtrlJumpTable(table, default, end_tag)));
            instructions.append(&table_instructions.instructions);
            return Ok(Some(instructions));
        }
        Ok(None)
    }
}


#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use arcstr::literal;
    use rustc_hash::FxHashMap;
    use crate::{model::Graph, runtime::{instruction::Instruction, instructions::{block::Block, switch::SwitchIns, Base}, Runtime, Val}};

    #[test]
    fn switch_pass() {
        let mut ins = Block::default();
        ins.ins.push_back(Arc::new(Base::Literal(Val::Str(literal!("cj")))));

        let mut map: FxHashMap<Val, Arc<dyn Instruction>> = FxHashMap::default();
        let def = None;
        map.insert(Val::Str(literal!("amelia")), Arc::new(Base::Literal(Val::Str(literal!("fail")))));
        map.insert(Val::Str(literal!("cj")), Arc::new(Base::Literal(Val::Str(literal!("pass")))));

        ins.ins.push_back(Arc::new(SwitchIns { map, def }));

        let mut graph = Graph::default();
        let res = Runtime::eval(&mut graph, Arc::new(ins)).expect("expected pass");
        assert_eq!(res, "pass".into());
    }
}
