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

use std::{ops::Deref, sync::Arc};
use arcstr::ArcStr;
use imbl::Vector;
use serde::{Deserialize, Serialize};
use crate::{model::{Computed, ComputedCall, Graph, NodeRef}, runtime::{Error, Val, Variable, instruction::{Instruction, Instructions}, instructions::{AWAIT, Base, call::FuncCall, empty::ValueIns}, proc::ProcEnv}};


/// How many rounds of computed values an export may take (a computed value that is an object with computed values
/// of its own takes another round). Stops runaway recursion (Ex. a function that returns a new object each time).
const MAX_COMPUTED_ROUNDS: usize = 32;


/// Exports (`stringify` and `blobify`) for formats with computed values (see `Format::computed`).
/// Instructions that call the computed values in this process, then export; None when the format computes nothing.
pub(super) fn computed_export(env: &ProcEnv, graph: &Graph, format: &str, node: Option<NodeRef>, binary: bool) -> Option<Instructions> {
    let fmt = graph.get_format(format).or_else(|| graph.get_format_by_content_type(format))?;
    let known = Computed::default();
    let calls = fmt.computed(graph, format, node.clone(), &known);
    if calls.is_empty() { return None; }
    Some(round_instructions(env, ComputedExport { format: format.to_string(), node, binary, round: 1, keys: vec![], known }, calls))
}

/// Call each function (its value lands on the stack, awaited, null when it returns nothing), then export.
fn round_instructions(env: &ProcEnv, mut next: ComputedExport, calls: Vec<ComputedCall>) -> Instructions {
    let mut instructions = Instructions::default();
    let base = env.stack.len();
    next.keys = Vec::with_capacity(calls.len());
    for (i, call) in calls.into_iter().enumerate() {
        let oself: Option<Arc<dyn Instruction>> = if call.method { Some(Arc::new(Base::Literal(Val::Obj(call.obj.clone())))) } else { None };
        instructions.push(Arc::new(ValueIns { ins: Arc::new(FuncCall { func: Some(call.func), search: None, stack: false, as_ref: false, cnull: false, args: Vector::default(), oself }) }));
        instructions.push(AWAIT.clone());
        instructions.push(Arc::new(Base::EnsureValue(base + i)));
        next.keys.push((call.obj, call.name));
    }
    instructions.push(Arc::new(next));
    instructions
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Collects a round of computed values from the stack, then exports (or starts the next round).
/// Only created while running (never stored in a document), so the values aren't serialized.
pub struct ComputedExport {
    pub format: String,
    pub node: Option<NodeRef>,
    pub binary: bool,
    pub round: usize,
    #[serde(skip)]
    pub keys: Vec<(NodeRef, ArcStr)>,
    #[serde(skip)]
    pub known: Computed,
}
#[typetag::serde(name = "ComputedExport")]
impl Instruction for ComputedExport {
    fn exec(&self, env: &mut ProcEnv, graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        let mut known = self.known.clone();
        let mut values = Vec::with_capacity(self.keys.len());
        for _ in 0..self.keys.len() {
            match env.stack.pop() {
                Some(var) => values.push(var.val.read().deref().clone()),
                None => return Err(Error::StdStringify("computed values missing from the stack".into())),
            }
        }
        values.reverse();
        for (key, value) in self.keys.iter().cloned().zip(values) {
            known.values.insert(key, value);
        }

        let Some(fmt) = graph.get_format(&self.format).or_else(|| graph.get_format_by_content_type(&self.format)) else {
            return Err(Error::GraphFormatNotFound);
        };
        let calls = fmt.computed(graph, &self.format, self.node.clone(), &known);
        if !calls.is_empty() {
            if self.round >= MAX_COMPUTED_ROUNDS {
                return Err(Error::StdStringify(format!("computed values kept producing more computed values ({} rounds)", MAX_COMPUTED_ROUNDS)));
            }
            let next = ComputedExport { format: self.format.clone(), node: self.node.clone(), binary: self.binary, round: self.round + 1, keys: vec![], known };
            return Ok(Some(round_instructions(env, next, calls)));
        }

        if self.binary {
            let bytes = fmt.binary_export_computed(graph, &self.format, self.node.clone(), &known)?;
            env.stack.push(Variable::val(Val::Blob(bytes)));
        } else {
            let src = fmt.string_export_computed(graph, &self.format, self.node.clone(), &known)?;
            env.stack.push(Variable::val(Val::Str(src.into())));
        }
        Ok(None)
    }
}
