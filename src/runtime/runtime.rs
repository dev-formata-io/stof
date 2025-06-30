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
use crate::{model::{Graph, SId}, runtime::{instruction::Instructions, proc::{ProcState, Process}}};


#[derive(Default, Debug)]
/// Runtime.
pub struct Runtime {
    pub running: FxHashMap<SId, Process>, // TODO: change to Vec
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
        self.running.insert(id.clone(), proc);
        id
    }

    /// Run to completion.
    pub fn run_to_complete(&mut self, graph: &mut Graph) {
        let mut to_done = Vec::new();
        let mut to_wait = Vec::new();
        let mut to_err = Vec::new();
        let mut to_run = Vec::new();
        while !self.running.is_empty() {
            for (pid, proc) in self.running.iter_mut() {
                match proc.progress(graph) {
                    Ok(state) => {
                        match state {
                            ProcState::More => {
                                // nada..
                            },
                            ProcState::Done => {
                                to_done.push(pid.clone());
                            },
                            ProcState::Wait(opid) => {
                                proc.waiting = Some(opid);
                                to_wait.push(pid.clone());
                            }
                        }
                    },
                    Err(error) => {
                        proc.error = Some(error);
                        to_err.push(pid.clone());
                    }
                }
            }

            if !to_done.is_empty() {
                for id in to_done.drain(..) {
                    if let Some(proc) = self.running.remove(&id) {
                        self.done.insert(id, proc);
                    }
                }

                for (id, waiting_proc) in &mut self.waiting {
                    if let Some(wait_id) = waiting_proc.waiting.clone() {
                        if let Some(done_proc) = self.done.remove(&wait_id) {
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
                        if let Some(proc) = self.waiting.remove(&id) {
                            self.running.insert(id, proc);
                        }
                    }
                }
            }

            if !to_wait.is_empty() {
                for id in to_wait.drain(..) {
                    if let Some(proc) = self.running.remove(&id) {
                        self.waiting.insert(id, proc);
                    }
                }
            }

            if !to_err.is_empty() {
                for id in to_err.drain(..) {
                    if let Some(proc) = self.running.remove(&id) {
                        self.errored.insert(id, proc);
                    }
                }
            }
        }
    }
}
