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
use imbl::Vector;
use rustc_hash::FxHashMap;
use crate::{model::{DataRef, Func, Graph, SId}, runtime::{instruction::Instruction, instructions::{call::FuncCall, Base}, proc::{ProcRes, Process}, Error, Val}};


#[derive(Default, Debug)]
/// Runtime.
pub struct Runtime {
    running: Vec<Process>,
    waiting: FxHashMap<SId, Process>,
    pub done: FxHashMap<SId, Process>,
    pub errored: FxHashMap<SId, Process>,
}
impl Runtime {
    #[inline]
    /// Push a process to this runtime.
    pub fn push_running_proc(&mut self, mut proc: Process, graph: &mut Graph) -> SId {
        let id = proc.env.pid.clone();
        
        // make sure the process has a self
        if proc.env.self_stack.is_empty() {
            proc.env.self_stack.push(graph.ensure_main_root());
        }
        
        self.running.push(proc);
        id
    }

    #[inline]
    /// Remove a process from running and return it.
    fn remove_running(&mut self, id: &SId) -> Process {
        let mut i: usize = 0;
        for proc in &self.running {
            if &proc.env.pid == id {
                break;
            }
            i += 1;
        }
        self.running.swap_remove(i)
    }

    #[inline(always)]
    /// Move from running to done.
    fn move_running_to_done(&mut self, id: &SId) {
        let proc = self.remove_running(id);
        self.done.insert(id.clone(), proc);
    }

    #[inline(always)]
    /// Move from running to waiting.
    fn move_running_to_waiting(&mut self, id: &SId) {
        let proc = self.remove_running(id);
        self.waiting.insert(id.clone(), proc);
    }

    #[inline(always)]
    /// Move from running to errored.
    fn move_running_to_error(&mut self, id: &SId) {
        let proc = self.remove_running(id);
        self.errored.insert(id.clone(), proc);
    }

    /// Run to completion.
    pub fn run_to_complete(&mut self, graph: &mut Graph) {
        let mut to_done = Vec::new();
        let mut to_wait = Vec::new();
        let mut to_err = Vec::new();
        let mut to_run = Vec::new();
        let mut to_spawn = Vec::new();
        while !self.running.is_empty() {
            for proc in self.running.iter_mut() {
                match proc.progress(graph) {
                    Ok(state) => {
                        match state {
                            ProcRes::Wait(pid) => {
                                proc.waiting = Some(pid);
                                to_wait.push(proc.env.pid.clone());
                            },
                            ProcRes::More => {
                                if let Some(spawn) = proc.env.spawn.take() {
                                    // this is only set via the Spawn instruction, which creates a new PID each time
                                    // therefore, don't have to worry about collisions here
                                    to_spawn.push(spawn);
                                }
                            },
                            ProcRes::Done => {
                                if let Some(var) = proc.env.stack.pop() {
                                    proc.result = Some(var);
                                }
                                to_done.push(proc.env.pid.clone());
                            },
                        }
                    },
                    Err(error) => {
                        proc.error = Some(error);
                        to_err.push(proc.env.pid.clone());
                    }
                }
            }

            if !to_done.is_empty() {
                for id in to_done.drain(..) {
                    self.move_running_to_done(&id);
                }

                for (id, waiting_proc) in &mut self.waiting {
                    if let Some(wait_id) = &waiting_proc.waiting {
                        if let Some(done_proc) = self.done.remove(wait_id) {
                            // If the completed process has a result, push that to the waiting processes stack
                            if let Some(res) = done_proc.result {
                                waiting_proc.env.stack.push(res);
                            }
                            to_run.push(id.clone());
                        }
                    }
                }
                if !to_run.is_empty() {
                    for id in to_run.drain(..) {
                        if let Some(mut proc) = self.waiting.remove(&id) {
                            proc.waiting = None;
                            self.running.push(proc);
                        }
                    }
                }
            }
            if !to_wait.is_empty() {
                for id in to_wait.drain(..) {
                    self.move_running_to_waiting(&id);
                }
            }
            if !to_err.is_empty() {
                for id in to_err.drain(..) {
                    self.move_running_to_error(&id);
                }
            }
            if !to_spawn.is_empty() {
                for proc in to_spawn.drain(..) {
                    self.push_running_proc(*proc, graph);
                }
            }
        }
    }

    /// Clear this runtime completely.
    pub fn clear(&mut self) {
        self.running.clear();
        self.waiting.clear();
        self.done.clear();
        self.errored.clear();
    }

    /*****************************************************************************
     * Static functions.
     *****************************************************************************/
    
    /// Call a singular function with this runtime.
    pub fn call(graph: &mut Graph, search: &str, args: Vec<Val>) -> Result<Val, Error> {
        let mut arguments: Vector<Arc<dyn Instruction>> = Vector::default();
        for arg in args { arguments.push_back(Arc::new(Base::Literal(arg))); }
        let instruction = Arc::new(FuncCall {
            stack: false,
            func: None,
            search: Some(search.into()),
            args: arguments,
        });
        Self::eval(graph, instruction)
    }
    
    /// Call a singular function with this runtime.
    pub fn call_func(graph: &mut Graph, func: &DataRef, args: Vec<Val>) -> Result<Val, Error> {
        if !func.type_of::<Func>(&graph) {
            return Err(Error::FuncDne);
        }
        let mut arguments: Vector<Arc<dyn Instruction>> = Vector::default();
        for arg in args { arguments.push_back(Arc::new(Base::Literal(arg))); }
        let instruction = Arc::new(FuncCall {
            stack: false,
            func: Some(func.clone()),
            search: None,
            args: arguments,
        });
        Self::eval(graph, instruction)
    }
    
    /// Evaluate a single instruction.
    /// Creates a new runtime and process just for this (lightweight).
    /// Use this while parsing if needed.
    pub fn eval(graph: &mut Graph, instruction: Arc<dyn Instruction>) -> Result<Val, Error> {
        let mut runtime = Self::default();
        let proc = Process::from(instruction);
        let pid = proc.env.pid.clone();
        
        runtime.push_running_proc(proc, graph);
        runtime.run_to_complete(graph);

        if let Some(proc) = runtime.done.remove(&pid) {
            if let Some(res) = proc.result {
                Ok(res.get())
            } else {
                Ok(Val::Void)
            }
        } else if let Some(proc) = runtime.errored.remove(&pid) {
            if let Some(err) = proc.error {
                Err(err)
            } else {
                Err(Error::NotImplemented)
            }
        } else {
            Err(Error::NotImplemented)
        }
    }
}
