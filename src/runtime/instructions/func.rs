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
use crate::{model::{DataRef, Func, Graph}, runtime::{instruction::{Instruction, Instructions}, instructions::Base, proc::ProcEnv, Captured, Error, FnRef, Val}};


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Arrow function literal value.
/// Only inserts the function once into the graph, then references it over and over.
/// Replaces itself with a literal instruction for efficiency.
pub struct FuncLit {
    /// Random per parse: left out of seal digests (the rest of the arrow is in them).
    #[serde(serialize_with = "crate::model::seal::serialize_arrow_id")]
    pub dref: DataRef,
    pub func: Func,
}
#[typetag::serde(name = "FuncLit")]
impl Instruction for FuncLit {
    fn exec(&self, env: &mut ProcEnv, graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        let mut instructions = Instructions::default();

        // the variables its body uses, as they are here: they go with this evaluation's value (each evaluation is its
        // own value with its own captures; nothing about them is written to the document)
        let captured = env.table.capture(self.func.capture_names.as_ref().map(|names| names.as_slice()));
        let captured = if captured.is_empty() { None } else { Some(Arc::new(captured)) };
        let value = |dref: DataRef| Arc::new(Base::Literal(Val::Fn(FnRef::new(dref, captured.clone()))));

        // code confined to part of the document (see model::seal::Confine) makes arrows on its own part: one it
        // evaluates somewhere it can't change (Ex. in a helper it reads) is a new arrow on the part instead
        let confined_root = if graph.confined.is_empty() { None } else {
            graph.confined.iter().rev().find(|reach| reach.reads.is_some()).map(|reach| reach.root.clone())
        };
        if let Some(root) = &confined_root {
            let outside = if self.dref.data_exists(&graph) {
                !self.dref.data(graph).is_some_and(|data| data.nodes.iter().all(|node| graph.confined_changes(node)))
            } else {
                let target = env.new_stack.last().cloned().unwrap_or_else(|| env.self_ptr());
                !graph.confined_changes(&target)
            };
            if outside {
                // one copy per arrow and part (made once; each evaluation's value carries its own captures)
                let id = crate::model::SId::from(format!("{}@{}", self.dref.as_ref(), root.as_ref()).as_str());
                if id.data_exists(&graph) {
                    instructions.push(value(id));
                } else {
                    let func = self.func.clone();
                    if let Some(dref) = graph.without_seals(|graph| graph.insert_stof_data(root, id.clone(), Box::new(func), Some(id.clone()))) {
                        instructions.push(value(dref));
                    }
                }
                return Ok(Some(instructions));
            }
        }

        if self.dref.data_exists(&graph) {
            instructions.push(value(self.dref.clone()));
        } else {
            let mut self_ptr = env.self_ptr();
            if let Some(ns) = env.new_stack.last() {
                self_ptr = ns.clone(); // if we are currently creating an object, make it the scope!
            }
            let func = self.func.clone();
            // an arrow that sealed code evaluates lives on the sealed object, under its own new name
            if let Some(dref) = graph.without_seals(|graph| graph.insert_stof_data(&self_ptr, &self.dref, Box::new(func), Some(self.dref.clone()))) {
                instructions.push(value(dref));
            }
        }

        Ok(Some(instructions))
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// An arrow function call's scope, with the variables the arrow captured where it was evaluated (looked up right
/// after the arrow's own variables, before the caller's). Made when the arrow is called, so never stored in documents.
pub struct ArrowScope {
    #[serde(skip)]
    pub captured: Option<Captured>,
}
#[typetag::serde(name = "ArrowScope")]
impl Instruction for ArrowScope {
    fn exec(&self, env: &mut ProcEnv, _graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        env.table.push_captured(self.captured.clone());
        Ok(None)
    }
}
