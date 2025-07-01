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

use rustc_hash::FxHashMap;
use crate::{model::{Graph, SId}, runtime::{instruction::Instructions, proc::{ProcRes, Process}}};


#[derive(Default, Debug)]
/// Runtime.
pub struct Runtime {
    pub running: Vec<Process>,
    pub done: FxHashMap<SId, Process>,
    pub waiting: FxHashMap<SId, Process>,
    pub errored: FxHashMap<SId, Process>,
}
impl From<Instructions> for Runtime {
    fn from(value: Instructions) -> Self {
        let mut rt = Self::default();
        rt.push_running_proc(Process::from(value));
        rt
    }
}
impl Runtime {
    #[inline]
    /// Push a process to this runtime.
    pub fn push_running_proc(&mut self, proc: Process) -> SId {
        let id = proc.env.pid.clone();
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
                            ProcRes::More => {
                                if let Some(spawn) = proc.env.spawn.take() {
                                    proc.waiting = Some(spawn.env.pid.clone());
                                    to_wait.push(proc.env.pid.clone());
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
                    self.push_running_proc(*proc);
                }
            }
        }
    }
}
