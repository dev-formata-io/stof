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
use serde::{Deserialize, Serialize};
use crate::{model::Graph, runtime::{expr::Expr, instruction::{Instruction, State}, proc::ProcEnv, Error, Type, Variable}};


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Instruction to declare a local variable (non-const version).
pub struct Declare {
    pub name: ArcStr,
    pub stype: Option<Type>,
    pub expr: Expr,
    pub next: Option<Arc<dyn Instruction>>,
}

#[typetag::serde(name = "Dec")]
impl Instruction for Declare {
    fn exec(&self, env: &mut ProcEnv, graph: &mut Graph) -> Result<State, Error> {
        if !env.table.can_declare(&self.name) {
            return Err(Error::DeclareExisting);
        }
        if self.name.contains('.') {
            return Err(Error::DeclareInvalidName);
        }

        let var = self.expr.exec(graph)?;
        if let Some(stype) = &self.stype {
            if &var.spec_type(&graph) != stype {
                return Err(Error::DeclareInvalidType);
            }
        }
        env.table.insert(&self.name, var);

        if let Some(next) = &self.next {
            Ok(State::More(next.clone()))
        } else {
            Ok(State::None)
        }
    }

    fn push(&mut self, instruction: Arc<dyn Instruction>) -> bool {
        self.next = Some(instruction);
        true
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Instruction to declare a local variable (non-const version).
pub struct ConstDeclare {
    pub name: ArcStr,
    pub stype: Option<Type>,
    pub expr: Expr,
    pub next: Option<Arc<dyn Instruction>>,
}

#[typetag::serde(name = "ConstDec")]
impl Instruction for ConstDeclare {
    fn exec(&self, env: &mut ProcEnv, graph: &mut Graph) -> Result<State, Error> {
        if !env.table.can_declare(&self.name) {
            return Err(Error::DeclareExisting);
        }
        if self.name.contains('.') {
            return Err(Error::DeclareInvalidName);
        }

        let var = self.expr.exec(graph)?;
        if let Some(stype) = &self.stype {
            if &var.spec_type(&graph) != stype {
                return Err(Error::DeclareInvalidType);
            }
        }
        env.table.insert(&self.name, Variable::Const(Box::new(var)));

        if let Some(next) = &self.next {
            Ok(State::More(next.clone()))
        } else {
            Ok(State::None)
        }
    }

    fn push(&mut self, instruction: Arc<dyn Instruction>) -> bool {
        self.next = Some(instruction);
        true
    }
}
