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
use crate::{model::Graph, runtime::{instruction::{Instruction, Instructions}, proc::ProcEnv, Error}};


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Nullcheck instruction (`value ?? ifnull`).
pub struct NullcheckIns {
    pub ins: Arc<dyn Instruction>,
    pub ifnull: Arc<dyn Instruction>,
    /// The check, made once (not stored: made again when loaded).
    #[serde(skip)]
    pub check: Option<Arc<dyn Instruction>>,
}
impl NullcheckIns {
    pub fn new(ins: Arc<dyn Instruction>, ifnull: Arc<dyn Instruction>) -> Self {
        let check: Arc<dyn Instruction> = Arc::new(NullcheckElse { ifnull: ifnull.clone() });
        Self { ins, ifnull, check: Some(check) }
    }
}
#[typetag::serde(name = "NullcheckIns")]
impl Instruction for NullcheckIns {
    fn exec(&self, _env: &mut ProcEnv, _graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        // the value, then a check of it (no jump tags: this runs on hot paths)
        let mut instructions = Instructions::default();
        instructions.push(self.ins.clone());
        match &self.check {
            Some(check) => instructions.push(check.clone()),
            None => instructions.push(Arc::new(NullcheckElse { ifnull: self.ifnull.clone() })),
        }
        Ok(Some(instructions))
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// After a nullcheck's value: when it's null, replace it with the ifnull value.
/// Only created while running.
pub struct NullcheckElse {
    pub ifnull: Arc<dyn Instruction>,
}
#[typetag::serde(name = "NullcheckElse")]
impl Instruction for NullcheckElse {
    fn exec(&self, env: &mut ProcEnv, _graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        let Some(var) = env.stack.last() else { return Err(Error::IsNull); };
        if !var.null() { return Ok(None); }
        env.stack.pop();
        let mut instructions = Instructions::default();
        instructions.push(self.ifnull.clone());
        Ok(Some(instructions))
    }
}
