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
use lazy_static::lazy_static;
use crate::{model::{Graph, NodeRef, SId}, runtime::{instruction::Instruction, proc::Process, Error, Runtime, Val}};


lazy_static! {
    static ref PARSE_ID: SId = SId::from("parse");
}


/// Parse context.
pub struct ParseContext<'ctx> {
    pub graph: &'ctx mut Graph,
    pub runtime: Runtime,
}
impl<'ctx> ParseContext<'ctx> {
    /// Create a new parse context.
    pub fn new(graph: &'ctx mut Graph) -> Self {
        let mut runtime = Runtime::default();
        
        // Stage the process for eval in done
        let mut process = Process::default();
        process.env.pid = PARSE_ID.clone();
        runtime.done.insert(process.env.pid.clone(), process);

        Self {
            graph,
            runtime,
        }
    }

    /// Get the current parse process.
    pub fn parse_proc<'a>(&'a mut self) -> &'a mut Process {
        self.runtime.done.get_mut(&PARSE_ID).unwrap()
    }

    /// Get the current self pointer.
    pub fn self_ptr(&mut self) -> NodeRef {
        let proc = self.parse_proc();
        if proc.env.self_stack.len() > 0 {
            proc.env.self_ptr()
        } else {
            self.graph.ensure_main_root()
        }
    }

    /// Reset the process when things go badly.
    fn reset_proc(&mut self) {
        self.runtime.clear();

        let mut process = Process::default();
        process.env.pid = PARSE_ID.clone();
        self.runtime.done.insert(process.env.pid.clone(), process);
    }

    /// Use this to quickly evaluate one instruction in the parse process.
    /// Must have a process in done.
    pub fn eval(&mut self, instruction: Arc<dyn Instruction>) -> Result<Val, Error> {
        // get the process and clear it (preserving memory allocations)
        let mut proc = self.runtime.done.remove(&PARSE_ID).unwrap();
        //proc.env.self_stack.clear(); // use this stack as the parse self stack, so dont clear!
        proc.env.call_stack.clear();
        proc.env.new_stack.clear();
        proc.env.stack.clear();
        proc.env.table.clear();
        proc.instructions.clear();
        proc.result = None;
        proc.error = None;
        proc.waiting = None;

        // load the instruction and push to running
        proc.instructions.push(instruction);
        self.runtime.push_running_proc(proc, &mut self.graph); // makes sure there is a self stack

        // run to end and grab the result
        self.runtime.run_to_complete(&mut self.graph);

        if let Some(proc) = self.runtime.done.get_mut(&PARSE_ID) {
            if let Some(res) = proc.result.take() {
                Ok(res.get())
            } else {
                Ok(Val::Void)
            }
        } else if let Some(mut proc) = self.runtime.errored.remove(&PARSE_ID) {
            let res;
            if let Some(err) = proc.error.take() {
                res = Err(err);
            } else {
                res = Err(Error::NotImplemented);
            }

            // Move proc back to done for next time
            self.runtime.done.insert(proc.env.pid.clone(), proc);
            res
        } else {
            self.reset_proc();
            Err(Error::NotImplemented)
        }
    }
}
