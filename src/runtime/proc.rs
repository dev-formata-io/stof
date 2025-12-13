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

use std::{sync::Arc, time::Duration};
use arcstr::ArcStr;
use colored::Colorize;
use crate::{model::{DataRef, Func, Graph, NodeRef, SId}, runtime::{instruction::{Instruction, Instructions}, table::SymbolTable, Error, Variable, WakeRef, Waker}};


#[derive(Debug)]
/// Process Result.
pub enum ProcRes {
    Done,
    More,
    Trace(usize),
    Peek(usize),
    Wait(SId),
    SleepFor(Duration),
    Sleep(WakeRef),
    Exit(Option<SId>),
}


#[derive(Clone, Debug)]
/// Process Env.
pub struct ProcEnv {
    pub pid: SId,
    pub start_time: Option<web_time::Instant>,
    pub max_execution_time: Option<web_time::Duration>,
    pub self_stack: Vec<NodeRef>,
    pub max_call_stack_depth: usize,
    pub call_stack: Vec<DataRef>,
    pub new_stack: Vec<NodeRef>,
    pub max_stack_size: usize,
    pub stack: Vec<Variable>,
    pub table: Box<SymbolTable>,
    pub loop_stack: Vec<ArcStr>,
    pub return_stack: Vec<ArcStr>,
    pub ret_valid_stack: Vec<usize>,
    pub try_stack: Vec<ArcStr>,

    // Setting this will put the process into a waiting mode
    pub spawn: Option<Box<Process>>,

    #[cfg(feature = "tokio")]
    pub tokio_runtime: Option<tokio::runtime::Handle>,
}
impl Default for ProcEnv {
    fn default() -> Self {
        Self {
            pid: Default::default(),
            start_time: None,
            max_execution_time: Some(Duration::from_secs(120)),
            self_stack: Default::default(),
            max_call_stack_depth: 10_000,
            call_stack: Default::default(),
            new_stack: Default::default(),
            max_stack_size: 100_000,
            stack: Default::default(),
            table: Default::default(),
            loop_stack: Default::default(),
            return_stack: Default::default(),
            ret_valid_stack: Default::default(),
            try_stack: Default::default(),
            spawn: None,

            #[cfg(feature = "tokio")]
            tokio_runtime: None,
        }
    }
}
impl ProcEnv {
    // Get the current self ptr.
    pub fn self_ptr(&self) -> NodeRef {
        self.self_stack.last().unwrap().clone()
    }

    // Trace.
    pub fn trace(&self, graph: &Graph) -> String {
        let mut output = format!("\t{} {}", "PID:".dimmed().italic(), self.pid.to_string().bright_green());

        if let Some(path) = self.self_ptr().node_path(graph, true) {
            output.push_str(&format!("\n\t{} {}", "Self:".dimmed().italic(), path.join(".").bright_cyan()));
        }

        let mut callstack = String::default();
        for index in 0..self.call_stack.len() {
            let func = &self.call_stack[index];
            
            let mut func_path = String::default();
            let nodes = func.data_nodes(graph);
            if nodes.len() > 0 {
                for node in nodes {
                    if let Some(path) = node.node_path(graph, true) {
                        func_path = path.join(".");
                        break;
                    }
                }
            }

            if let Some(this) = graph.get_stof_data::<Func>(func) {
                let mut params = String::default();
                let mut first = true;
                for param in &this.params {
                    if first {
                        first = false;
                        params.push_str(&format!("{}: {}", param.name.as_ref(), param.param_type.rt_type_of(graph)));
                    } else {
                        params.push_str(&format!(", {}: {}", param.name.as_ref(), param.param_type.rt_type_of(graph)));
                    }
                }
                let prefix = format!("{index}.");
                let signature = format!("{} {}.{}({params}) -> {};", prefix.dimmed(), func_path.cyan().dimmed(), func.data_name(graph).unwrap().as_ref().bright_purple(), this.return_type.rt_type_of(graph).as_str().blue());
                callstack.push_str(&format!("\n\t\t{}", signature));
            }
        }
        output.push_str(&format!("\n\t{} {callstack}", "Call-stack:".dimmed().italic()));
        
        output
    }
}


#[derive(Clone, Debug, Default)]
/// Process.
pub struct Process {
    pub env: ProcEnv,
    pub instructions: Instructions,
    pub result: Option<Variable>,
    pub error: Option<Error>,
    pub waiting: Option<SId>,
}
impl From<Instructions> for Process {
    fn from(value: Instructions) -> Self {
        Self {
            instructions: value,
            ..Default::default()
        }
    }
}
impl From<Arc<dyn Instruction>> for Process {
    fn from(value: Arc<dyn Instruction>) -> Self {
        Self {
            instructions: Instructions::from(value),
            ..Default::default()
        }
    }
}
impl Process {
    #[inline(always)]
    /// Progress this process.
    pub(super) fn progress(&mut self, graph: &mut Graph, limit: i32) -> Result<ProcRes, Error> {
        match self.instructions.exec(&mut self.env, graph, limit) {
            Ok(res) => {
                Ok(res)
            },
            Err(error) => {
                Err(error)
            }
        }
    }

    /// Trace.
    pub fn trace(&self, graph: &Graph, n: usize) -> String {
        let mut output = self.env.trace(graph);
        
        if let Some(waiting) = &self.waiting {
            output.push_str(&format!("\n\t{} {}", "Waiting:".dimmed().italic(), waiting.to_string().bright_green()));
        }

        if let Some(error) = &self.error {
            output.push_str(&format!("\n\t{} {}", "Error:".dimmed().italic(), error.to_string().red()));
        }

        if let Some(result) = &self.result {
            output.push_str(&format!("\n\t{} {}", "Result:".dimmed().italic(), result.val.read().print(graph).dimmed()));
        }

        output.push_str(&format!("\n\t{}{}", "Executed Instructions:\n".dimmed().italic(), self.instructions.trace_n(n)));

        output
    }

    /// Peek.
    pub fn peek(&self, graph: &Graph, n: usize) -> String {
        let mut output = self.env.trace(graph);
        
        if let Some(waiting) = &self.waiting {
            output.push_str(&format!("\n\t{} {}", "Waiting:".dimmed().italic(), waiting.to_string().bright_green()));
        }

        if let Some(error) = &self.error {
            output.push_str(&format!("\n\t{} {}", "Error:".dimmed().italic(), error.to_string().red()));
        }

        if let Some(result) = &self.result {
            output.push_str(&format!("\n\t{} {}", "Result:".dimmed().italic(), result.val.read().print(graph).dimmed()));
        }

        output.push_str(&format!("\n\t{}{}", "Next Instructions:\n".dimmed().italic(), self.instructions.peek_n(n)));

        output
    }

    #[inline]
    /// Create a waker for this process with a wake reference.
    pub(super) fn waker_ref(&self, wref: WakeRef) -> Waker {
        Waker { pid: self.env.pid.clone(), at: None, with: wref }
    }

    #[inline]
    /// Create a waker for this process with a wake time.
    pub(super) fn waker_time(&self, at: Duration) -> Waker {
        Waker { pid: self.env.pid.clone(), at: Some(at), with: Default::default() }
    }
}
