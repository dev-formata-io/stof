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

use arcstr::ArcStr;
use serde::{Deserialize, Serialize};
use crate::{model::Graph, runtime::{expr::Expr, instruction::{Instruction, State}, proc::ProcEnv, Error, Type, Variable}};


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Instruction to declare a local variable (non-const version).
pub struct Declare {
    pub name: ArcStr,
    pub stype: Option<Type>,
    pub expr: Expr,
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

        let mut var = self.expr.exec(graph)?;
        if let Some(stype) = &self.stype {
            if &var.spec_type(&graph) != stype {
                if let Err(cast_error) = var.cast(stype, graph) {
                    return Err(Error::DeclareInvalidType(Box::new(cast_error)));
                }
            }
        }
        env.table.insert(&self.name, var);
        Ok(State::None)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Instruction to declare a local variable (non-const version).
pub struct ConstDeclare {
    pub name: ArcStr,
    pub stype: Option<Type>,
    pub expr: Expr,
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

        let mut var = self.expr.exec(graph)?;
        if let Some(stype) = &self.stype {
            if &var.spec_type(&graph) != stype {
                if let Err(cast_error) = var.cast(stype, graph) {
                    return Err(Error::DeclareInvalidType(Box::new(cast_error)));
                }
            }
        }
        env.table.insert(&self.name, Variable::Const(Box::new(var)));
        Ok(State::None)
    }
}


#[cfg(test)]
mod tests {
    use std::{ops::Deref, sync::Arc};
    use arcstr::ArcStr;
    use crate::{model::Graph, runtime::{expr::Expr, instruction::Instructions, instructions::Declare, proc::Process, Runtime, Val}};

    #[test]
    fn declare() {
        let mut graph = Graph::default();
        let mut runtime = Runtime::default();
        let mut instructions = Instructions::default();

        instructions.push(Arc::new(Declare {
            name: ArcStr::from("test"),
            stype: None,
            expr: Expr::Lit(Val::Str(ArcStr::from("hello, world"))),
        }));

        let pid = runtime.push_running_proc(Process::from(instructions));
        runtime.run_to_complete(&mut graph);
        let mut proc = runtime.done.remove(&pid).unwrap();

        //println!("{:?}", proc.env.table);
        assert!(proc.env.table.get("test").is_some());
        assert_eq!(proc.env.table.get("test").unwrap().gen_type().type_of().deref(), "str");

        assert!(proc.env.table.drop_var("test"));
        assert!(proc.env.table.get("test").is_none());
    }
}
