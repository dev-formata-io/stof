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
use crate::{model::Graph, runtime::{instruction::Instructions, proc::ProcEnv, Error, Val}};


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Expressions.
pub enum Expr {
    Lit(Val),
    Var(ArcStr),
}
impl<T: Into<Val>> From<T> for Expr {
    fn from(value: T) -> Self {
        Self::Lit(value.into())
    }
}
impl Expr {
    /// Execute this expression to get another value.
    pub fn exec(&self, instructions: &mut Instructions, env: &mut ProcEnv, graph: &mut Graph) -> Result<Val, Error> {
        match self {
            Self::Lit(val) => {
                Ok(val.clone())
            },
            Self::Var(val) => {
                if let Some(var) = env.table.get(val) {
                    return Ok(var.get());
                }

                

                Err(Error::NotImplemented)
            },
        }
    }
}
