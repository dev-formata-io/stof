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

use std::{sync::Arc, time::{SystemTime, UNIX_EPOCH}};
use colored::Colorize;
use imbl::Vector;
use rustc_hash::FxHashMap;
use crate::{model::{DataRef, Func, Graph, SId}, runtime::{instruction::Instruction, instructions::{call::FuncCall, Base}, proc::{ProcRes, Process}, Error, Val, Waker}};


#[derive(Default)]
/// Runtime.
pub struct Runtime {
    running: Vec<Process>, // TODO: split into high-priority and low-priority based on size to minimize mean running time
    waiting: FxHashMap<SId, Process>,
    pub done: FxHashMap<SId, Process>,
    pub errored: FxHashMap<SId, Process>,

    sleeping: FxHashMap<SId, Process>,
    wakers: Vec<Waker>,

    pub done_callback: Option<Box<dyn FnMut(&Graph, &Process)->bool>>,
    pub err_callback: Option<Box<dyn FnMut(&Graph, &Process)->bool>>,
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
    fn move_running_to_done(&mut self, graph: &Graph, id: &SId) {
        let proc = self.remove_running(id);
        if let Some(cb) = &mut self.done_callback {
            if cb(graph, &proc) {
                self.done.insert(id.clone(), proc);
            } else {
                self.errored.insert(id.clone(), proc);
            }
        } else {
            self.done.insert(id.clone(), proc);
        }
    }

    #[inline(always)]
    /// Move from running to waiting.
    fn move_running_to_waiting(&mut self, id: &SId) {
        let proc = self.remove_running(id);
        self.waiting.insert(id.clone(), proc);
    }

    #[inline(always)]
    /// Move from running to errored.
    fn move_running_to_error(&mut self, graph: &Graph, id: &SId) {
        let proc = self.remove_running(id);
        if let Some(cb) = &mut self.err_callback {
            if cb(graph, &proc) {
                self.errored.insert(id.clone(), proc);
            } else {
                self.done.insert(id.clone(), proc);
            }
        } else {
            self.errored.insert(id.clone(), proc);
        }
    }

    #[inline(always)]
    /// Move from running to sleeping.
    fn move_running_to_sleeping(&mut self, id: &SId) {
        let proc = self.remove_running(id);
        self.sleeping.insert(id.clone(), proc);
    }

    /// Run to completion.
    pub fn run_to_complete(&mut self, graph: &mut Graph) {
        let mut to_done = Vec::new();
        let mut to_wait = Vec::new();
        let mut to_err = Vec::new();
        let mut to_run = Vec::new();
        let mut to_spawn = Vec::new();
        let mut to_sleep = Vec::new();
        while !self.running.is_empty() || !self.sleeping.is_empty() {
            // Check to see if any sleeping processes need to be woken up first
            if !self.sleeping.is_empty() {
                let mut to_wake = Vec::new();
                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                self.wakers.retain(|waker| {
                    let woken = waker.woken(&now);
                    if woken { to_wake.push(waker.pid.clone()); }
                    !woken
                });
                for id in to_wake {
                    if let Some(proc) = self.sleeping.remove(&id) {
                        self.running.push(proc);
                    }
                }
            }

            // any limit < 1 will progress the process as much as possible per process
            let mut limit: i32 = 0;
            if !self.sleeping.is_empty() || self.running.len() > 1 {
                let len = (self.sleeping.len() + self.running.len()) as i32;
                limit = i32::max(10, 500 / len);
            }

            for proc in self.running.iter_mut() {
                match proc.progress(graph, limit) {
                    Ok(state) => {
                        match state {
                            ProcRes::Wait(pid) => {
                                proc.waiting = Some(pid);
                                to_wait.push(proc.env.pid.clone());
                            },
                            ProcRes::Sleep(wref) => {
                                to_sleep.push((proc.env.pid.clone(), proc.waker_ref(wref)));
                            },
                            ProcRes::SleepFor(dur) => {
                                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                                to_sleep.push((proc.env.pid.clone(), proc.waker_time(now + dur)));
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

            for id in to_done.drain(..) {
                self.move_running_to_done(&graph, &id);
            }

            if !to_wait.is_empty() {
                for id in to_wait.drain(..) {
                    self.move_running_to_waiting(&id);
                }
            }

            if !to_err.is_empty() {
                for id in to_err.drain(..) {
                    self.move_running_to_error(&graph, &id);
                }
            }

            if !to_spawn.is_empty() {
                for proc in to_spawn.drain(..) {
                    self.push_running_proc(*proc, graph);
                }
            }

            if !to_sleep.is_empty() {
                for (id, waker) in to_sleep.drain(..) {
                    self.move_running_to_sleeping(&id);
                    self.wakers.push(waker);
                }
            }

            for (id, waiting_proc) in &mut self.waiting {
                if let Some(wait_id) = &waiting_proc.waiting {
                    if let Some(done_proc) = self.done.remove(wait_id) {
                        // If the completed process has a result, push that to the waiting processes stack
                        if let Some(res) = done_proc.result {
                            waiting_proc.env.stack.push(res);
                        }
                        to_run.push(id.clone());
                    } else if let Some(error_proc) = self.errored.remove(wait_id) {
                        // Propagate the error back to the awaiting process, so that it can optionally handle it itself
                        if let Some(error) = error_proc.error {
                            waiting_proc.instructions.instructions.push_front(Arc::new(Base::CtrlAwaitError(Error::AwaitError(Box::new(error)))));
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
    }

    /// Clear this runtime completely.
    pub fn clear(&mut self) {
        self.running.clear();
        self.waiting.clear();
        self.done.clear();
        self.errored.clear();
    }


    /*****************************************************************************
     * Test.
     *****************************************************************************/
    
    /// Test every #[test] function within this graph.
    /// Will insert callbacks into this runtime for printing results.
    /// If throw is false, this will only return Ok.
    pub fn test(graph: &mut Graph, context: Option<String>, throw: bool) -> Result<String, String> {
        // Create a fresh runtime
        let mut rt = Self::default();

        // Load all processes for all test functions
        let mut count = 0;
        for (_, func_ref) in Func::test_functions(&graph) {
            if let Some(context) = &context {
                for node in func_ref.data_nodes(&graph) {
                    if let Some(node_path) = node.node_path(&graph, true) {
                        let path = node_path.join(".");
                        if path.contains(context) {
                            let instruction = Arc::new(FuncCall {
                                stack: false,
                                func: Some(func_ref),
                                search: None,
                                args: Default::default(),
                            }) as Arc<dyn Instruction>;
                            let proc = Process::from(instruction);
                            count += 1;
                            rt.push_running_proc(proc, graph);
                            break;
                        }
                    }
                }
            } else {
                let instruction = Arc::new(FuncCall {
                    stack: false,
                    func: Some(func_ref),
                    search: None,
                    args: Default::default(),
                }) as Arc<dyn Instruction>;
                let proc = Process::from(instruction);
                count += 1;
                rt.push_running_proc(proc, graph);
            }
        }

        // Create and set callbacks for printing successes and failures
        rt.done_callback = Some(Box::new(|graph, success| {
            // if this is top-level and executed something, print out a success message
            if success.env.call_stack.len() < 1 && success.instructions.executed.len() > 0 {
                let func = success.instructions.executed[0].clone();
                if let Some(func) = func.as_dyn_any().downcast_ref::<Base>() {
                    match func {
                        Base::Literal(val) => {
                            if let Some(func_ref) = val.try_func() {
                                if let Some(name) = func_ref.data_name(graph) {
                                    if let Some(func) = graph.get_stof_data::<Func>(&func_ref) {
                                        if func.attributes.contains_key("errors") {
                                            if !func.attributes.contains_key("silent") {
                                                let mut func_path = String::from("<unknown>");
                                                for node in func_ref.data_nodes(graph) {
                                                    func_path = node.node_path(graph, true).unwrap().join(".");
                                                }
                                                println!("{} {} {} {} {}", "test".purple(), func_path.italic().dimmed(), name.as_ref().italic().blue(), "...".dimmed(), "failed".bold().red());
                                            }
                                            return false; // push to error instead of done
                                        } else if !func.attributes.contains_key("silent") {
                                            let mut func_path = String::from("<unknown>");
                                            for node in func_ref.data_nodes(graph) {
                                                func_path = node.node_path(graph, true).unwrap().join(".");
                                            }
                                            println!("{} {} {} {} {}", "test".purple(), func_path.italic().dimmed(), name.as_ref().italic().blue(), "...".dimmed(), "ok".bold().green());
                                        }
                                    }
                                }
                            }
                        },
                        _ => {}
                    }
                }
            }
            true
        }));
        rt.err_callback = Some(Box::new(|graph, errored| {
            // if this is top-level and executed something, print out an error message
            if errored.env.call_stack.len() > 0 {
                let func_ref = errored.env.call_stack.first().unwrap();
                if let Some(name) = func_ref.data_name(graph) {
                    if let Some(func) = graph.get_stof_data::<Func>(&func_ref) {
                        if func.attributes.contains_key("errors") {
                            if !func.attributes.contains_key("silent") {
                                let mut func_path = String::from("<unknown>");
                                for node in func_ref.data_nodes(graph) {
                                    func_path = node.node_path(graph, true).unwrap().join(".");
                                }
                                println!("{} {} {} {} {}", "test".purple(), func_path.italic().dimmed(), name.as_ref().italic().blue(), "...".dimmed(), "ok".bold().green());
                            }
                            return false; // push to done instead of to errored
                        } else if !func.attributes.contains_key("silent") {
                            let mut func_path = String::from("<unknown>");
                            for node in func_ref.data_nodes(graph) {
                                func_path = node.node_path(graph, true).unwrap().join(".");
                            }
                            println!("{} {} {} {} {}", "test".purple(), func_path.italic().dimmed(), name.as_ref().italic().blue(), "...".dimmed(), "failed".bold().red());
                        }
                    }
                }
            }
            true
        }));

        // Run to completion
        println!("{} {} {} {}", "running".bold(), count, "tests".bold(), "...".dimmed());
        let start = SystemTime::now();
        rt.run_to_complete(graph);
        let duration = start.elapsed().unwrap();

        // Gather results and output
        let mut output = "\n".to_string();
        let mut result = "ok".bold().green();
        if rt.errored.len() > 0 {
            result = "failed".bold().red();
            output.push_str(&format!("{} failures:\n", rt.errored.len()));
            for (_, failure) in &rt.errored {
                let func_ref;
                let mut err_str = String::default();
                if failure.env.call_stack.len() > 0 {
                    func_ref = failure.env.call_stack.first().unwrap().clone();
                    if let Some(err) = &failure.error {
                        err_str = err.to_string();
                    }
                } else if failure.env.call_stack.len() < 1 && failure.instructions.executed.len() > 0 {
                    let func = failure.instructions.executed[0].clone();
                    if let Some(func) = func.as_dyn_any().downcast_ref::<Base>() {
                        match func {
                            Base::Literal(val) => {
                                if let Some(fref) = val.try_func() {
                                    func_ref = fref;
                                    err_str = format!("expected to error, but received a result of '{:?}'", failure.result);
                                } else {
                                    continue;
                                }
                            },
                            _ => {
                                continue;
                            },
                        }
                    } else {
                        continue;
                    }
                } else {
                    continue;
                }

                if let Some(name) = func_ref.data_name(graph) {
                    let mut func_path = String::from("<unknown>");
                    for node in func_ref.data_nodes(graph) {
                        func_path = node.node_path(graph, true).unwrap().join(".");
                    }
                    output.push_str(&format!("\n{}: {}{}{} ...\n\t{}\n", "failed".bold().red(), func_path.italic().purple(), " @ ".dimmed(), name.as_ref().italic().blue(), err_str.bold().bright_cyan()));
                }
            }
            output.push('\n');
        }
        let passed = count - rt.errored.len();
        let dur = (duration.as_secs_f32() * 100.0).round() / 100.0;
        output.push_str(&format!("\ntest result: {}. {} passed; {} failed; finished in {}s\n", result, passed, rt.errored.len(), dur));

        if throw && rt.errored.len() > 0 {
            Err(output)
        } else {
            Ok(output)
        }
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
