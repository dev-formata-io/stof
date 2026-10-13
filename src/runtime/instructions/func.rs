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
use rustc_hash::FxHashMap;
use crate::{model::{DataRef, Func, Graph}, runtime::{instruction::{Instruction, Instructions}, instructions::Base, proc::ProcEnv, Error, Val, Variable}};


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Arrow function literal value.
/// Only inserts the function once into the graph, then references it over and over.
/// Replaces itself with a literal instruction for efficiency.
pub struct FuncLit {
    pub dref: DataRef,
    pub func: Func,
}
#[typetag::serde(name = "FuncLit")]
impl Instruction for FuncLit {
    fn exec(&self, env: &mut ProcEnv, graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        let mut instructions = Instructions::default();

        // the variables visible here, for when the arrow is called from somewhere they aren't (latest evaluation wins)
        let captured = env.table.capture(self.func.capture_names.as_ref().map(|names| names.as_slice()));
        let captured = if captured.is_empty() { None } else { Some(Arc::new(captured)) };

        if self.dref.data_exists(&graph) {
            let unchanged = captured.is_none() && graph.get_stof_data::<Func>(&self.dref).is_some_and(|func| func.captured.is_none());
            if !unchanged {
                if let Some(func) = graph.get_mut_stof_data::<Func>(&self.dref) {
                    func.captured = captured;
                }
            }
            instructions.push(Arc::new(Base::Literal(Val::Fn(self.dref.clone()))));
        } else {
            let mut self_ptr = env.self_ptr();
            if let Some(ns) = env.new_stack.last() {
                self_ptr = ns.clone(); // if we are currently creating an object, make it the scope!
            }
            let mut func = self.func.clone();
            func.captured = captured;
            if let Some(dref) = graph.insert_stof_data(&self_ptr, &self.dref, Box::new(func), Some(self.dref.clone())) {
                instructions.push(Arc::new(Base::Literal(Val::Fn(dref))));
            }
        }

        Ok(Some(instructions))
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// An arrow function call's scope, with the variables the arrow captured where it was evaluated (looked up after
/// every visible scope). Made when the arrow is called, so never stored in documents.
pub struct ArrowScope {
    #[serde(skip)]
    pub captured: Option<Arc<FxHashMap<String, Variable>>>,
}
#[typetag::serde(name = "ArrowScope")]
impl Instruction for ArrowScope {
    fn exec(&self, env: &mut ProcEnv, _graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        env.table.push_captured(self.captured.clone());
        Ok(None)
    }
}
