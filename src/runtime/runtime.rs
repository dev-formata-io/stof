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
use web_time::{SystemTime, UNIX_EPOCH};
use colored::Colorize;
use imbl::Vector;
use rustc_hash::{FxHashMap, FxHashSet};
use crate::{model::{DataRef, Func, Graph, SId}, runtime::{instruction::Instruction, instructions::{call::FuncCall, Base}, proc::{ProcRes, Process}, Error, Val, Waker}};

#[cfg(feature = "tokio")]
use parking_lot::RwLock;

#[cfg(feature = "tokio")]
use lazy_static::lazy_static;
#[cfg(feature = "tokio")]
lazy_static! {
    static ref TOKIO_RUNTIME: Arc<std::sync::Mutex<Option<tokio::runtime::Runtime>>> = Arc::new(std::sync::Mutex::new(None));
    static ref TOKIO_HANDLE_OVERRIDE: Arc<RwLock<Option<tokio::runtime::Handle>>> = Arc::new(RwLock::new(None));
}


/// Runtime.
pub struct Runtime {
    running: Vec<Process>, // TODO: split into high-priority and low-priority based on size to minimize mean running time?
    waiting: FxHashMap<SId, Process>,
    pub done: FxHashMap<SId, Process>,
    pub errored: FxHashMap<SId, Process>,

    sleeping: FxHashMap<SId, Process>,
    wakers: Vec<Waker>,

    pub done_callback: Option<Box<dyn FnMut(&Graph, &Process)->bool>>,
    pub err_callback: Option<Box<dyn FnMut(&Graph, &Process)->bool>>,

    #[cfg(feature = "tokio")]
    /// Optional tokio runtime handle (default exists, but still optional for flexibility & best practice in lib development).
    /// Processes can use this handle to spawn background tasks (ex. HTTP, Database Ops, etc.).
    pub tokio_runtime: Option<tokio::runtime::Handle>,
}
#[cfg(feature = "tokio")]
impl Default for Runtime {
    fn default() -> Self {
        let mut rt = Self {
            running: Default::default(),
            waiting: Default::default(),
            done: Default::default(),
            errored: Default::default(),
            sleeping: Default::default(),
            wakers: Default::default(),
            done_callback: Default::default(),
            err_callback: Default::default(),
            tokio_runtime: Default::default(),
        };

        // all Stof runtimes share the same background tokio runtime (thread pool)
        // check to see if there has been a tokio runtime handle given to us
        let tokio_runtime_handle = TOKIO_HANDLE_OVERRIDE.read();
        if let Some(handle) = &*tokio_runtime_handle {
            rt.tokio_runtime = Some(handle.clone());
        } else {
            // lazily create our own new tokio runtime if needed with defaults
            let mut tokio_runtime = TOKIO_RUNTIME.lock().unwrap();
            if let Some(tokio_runtime) = &*tokio_runtime {
                rt.tokio_runtime = Some(tokio_runtime.handle().clone());
            } else {
                let trt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
                rt.tokio_runtime = Some(trt.handle().clone());
                *tokio_runtime = Some(trt);
            }
        }
        rt
    }
}
#[cfg(not(feature = "tokio"))]
impl Default for Runtime {
    fn default() -> Self {
        Self {
            running: Default::default(),
            waiting: Default::default(),
            done: Default::default(),
            errored: Default::default(),
            sleeping: Default::default(),
            wakers: Default::default(),
            done_callback: Default::default(),
            err_callback: Default::default(),
        }
    }
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
        let mut to_exit = Vec::new();
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

                #[cfg(feature = "tokio")]
                {
                    // make sure each process has a handle to the correct runtime if needed
                    if self.tokio_runtime.is_some() {
                        proc.env.tokio_runtime = self.tokio_runtime.clone();
                    } else {
                        proc.env.tokio_runtime = None;
                    }
                }

                match proc.progress(graph, limit) {
                    Ok(state) => {
                        match state {
                            ProcRes::Exit(pid) => {
                                if let Some(pid) = pid {
                                    to_exit.push(pid);
                                } else {
                                    to_exit.push(proc.env.pid.clone());
                                }
                            },
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
                            ProcRes::Trace(n) => {
                                let trace = proc.trace(&graph, n);
                                println!("{trace}");
                            },
                            ProcRes::Peek(n) => {
                                let trace = proc.peek(&graph, n);
                                println!("{trace}");
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
                    self.move_running_to_done(&graph, &id);
                }
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
                        println!("{} {}{}{}{}{}\n{}", "await error".red().bold(), "(".dimmed(), waiting_proc.env.pid.as_ref().dimmed().purple(), " waiting on ".dimmed(), error_proc.env.pid.as_ref().dimmed().cyan(), ")".dimmed(), error_proc.trace(&graph, 20));
                        if let Some(error) = error_proc.error {
                            waiting_proc.instructions.instructions.push_front(Arc::new(Base::CtrlAwaitError(Error::AwaitError(Box::new(error)))));
                        }
                        to_run.push(id.clone());
                    } else if let Some(max) = &waiting_proc.env.max_execution_time {
                        if let Some(start) = &waiting_proc.env.start_time {
                            if &start.elapsed() > max {
                                // If the waiting process has outlived its ttl, then error
                                println!("{} {}{}{}{}{}", "await timeout error".red().bold(), "(".dimmed(), waiting_proc.env.pid.as_ref().dimmed().purple(), " waiting on ".dimmed(), wait_id.as_ref().dimmed().cyan(), ")".dimmed());
                                waiting_proc.instructions.instructions.push_front(Arc::new(Base::CtrlAwaitError(Error::ExecutionTimeout)));
                                to_run.push(id.clone());
                            }
                        }
                    }
                }
            }

            for (id, sleeping_proc) in &mut self.sleeping {
                if let Some(max) = &sleeping_proc.env.max_execution_time {
                    if let Some(start) = &sleeping_proc.env.start_time {
                        if &start.elapsed() > max {
                            // If the sleeping process has outlived its ttl, then error
                            println!("{} {}{}{}", "sleep timeout error".red().bold(), "(".dimmed(), sleeping_proc.env.pid.as_ref().dimmed().purple(), ")".dimmed());
                            sleeping_proc.instructions.instructions.push_front(Arc::new(Base::CtrlAwaitError(Error::ExecutionTimeout)));
                            to_run.push(id.clone());
                        }
                    }
                }
            }

            if !to_run.is_empty() {
                for id in to_run.drain(..) {
                    if let Some(mut proc) = self.waiting.remove(&id) {
                        proc.waiting = None;
                        self.running.push(proc);
                    } else if let Some(mut proc) = self.sleeping.remove(&id) {
                        proc.waiting = None;
                        self.running.push(proc);
                    }
                }
            }

            if !to_exit.is_empty() {
                for id in to_exit.drain(..) {
                    if let Some(proc) = self.waiting.remove(&id) {
                        if let Some(cb) = &mut self.done_callback {
                            if cb(graph, &proc) {
                                self.done.insert(id, proc);
                            } else {
                                self.errored.insert(id, proc);
                            }
                        } else {
                            self.done.insert(id, proc);
                        }
                    } else if let Some(proc) = self.sleeping.remove(&id) {
                        if let Some(cb) = &mut self.done_callback {
                            if cb(graph, &proc) {
                                self.done.insert(id, proc);
                            } else {
                                self.errored.insert(id, proc);
                            }
                        } else {
                            self.done.insert(id, proc);
                        }
                    } else {
                        self.move_running_to_done(graph, &id);
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
     * Singular & asynchronous.
     *****************************************************************************/
    
    /// Run a single step of this runtime.
    /// Returns true if there is another step to run.
    /// N.B: Do not alter this function - change run_to_complete and copy changes here.
    /// Make sure the limit is greater than 0 so that the step yields before complete if only one proc.
    pub fn run_single_step(&mut self, graph: &mut Graph) -> bool {
        let mut to_done = Vec::new();
        let mut to_wait = Vec::new();
        let mut to_err = Vec::new();
        let mut to_run = Vec::new();
        let mut to_spawn = Vec::new();
        let mut to_sleep = Vec::new();
        let mut to_exit = Vec::new();
        if !self.running.is_empty() || !self.sleeping.is_empty() {
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
            let mut limit: i32 = 100;
            if !self.sleeping.is_empty() || self.running.len() > 1 {
                let len = (self.sleeping.len() + self.running.len()) as i32;
                limit = i32::max(10, 500 / len);
            }

            for proc in self.running.iter_mut() {

                #[cfg(feature = "tokio")]
                {
                    // make sure each process has a handle to the correct runtime if needed
                    if self.tokio_runtime.is_some() {
                        proc.env.tokio_runtime = self.tokio_runtime.clone();
                    } else {
                        proc.env.tokio_runtime = None;
                    }
                }

                match proc.progress(graph, limit) {
                    Ok(state) => {
                        match state {
                            ProcRes::Exit(pid) => {
                                if let Some(pid) = pid {
                                    to_exit.push(pid);
                                } else {
                                    to_exit.push(proc.env.pid.clone());
                                }
                            },
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
                            ProcRes::Trace(n) => {
                                let trace = proc.trace(&graph, n);
                                println!("{trace}");
                            },
                            ProcRes::Peek(n) => {
                                let trace = proc.peek(&graph, n);
                                println!("{trace}");
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
                    self.move_running_to_done(&graph, &id);
                }
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
                        println!("{} {}{}{}{}{}\n{}", "await error".red().bold(), "(".dimmed(), waiting_proc.env.pid.as_ref().dimmed().purple(), " waiting on ".dimmed(), error_proc.env.pid.as_ref().dimmed().cyan(), ")".dimmed(), error_proc.trace(&graph, 20));
                        if let Some(error) = error_proc.error {
                            waiting_proc.instructions.instructions.push_front(Arc::new(Base::CtrlAwaitError(Error::AwaitError(Box::new(error)))));
                        }
                        to_run.push(id.clone());
                    } else if let Some(max) = &waiting_proc.env.max_execution_time {
                        if let Some(start) = &waiting_proc.env.start_time {
                            if &start.elapsed() > max {
                                // If the waiting process has outlived its ttl, then error
                                println!("{} {}{}{}{}{}", "await timeout error".red().bold(), "(".dimmed(), waiting_proc.env.pid.as_ref().dimmed().purple(), " waiting on ".dimmed(), wait_id.as_ref().dimmed().cyan(), ")".dimmed());
                                waiting_proc.instructions.instructions.push_front(Arc::new(Base::CtrlAwaitError(Error::ExecutionTimeout)));
                                to_run.push(id.clone());
                            }
                        }
                    }
                }
            }

            for (id, sleeping_proc) in &mut self.sleeping {
                if let Some(max) = &sleeping_proc.env.max_execution_time {
                    if let Some(start) = &sleeping_proc.env.start_time {
                        if &start.elapsed() > max {
                            // If the sleeping process has outlived its ttl, then error
                            println!("{} {}{}{}", "sleep timeout error".red().bold(), "(".dimmed(), sleeping_proc.env.pid.as_ref().dimmed().purple(), ")".dimmed());
                            sleeping_proc.instructions.instructions.push_front(Arc::new(Base::CtrlAwaitError(Error::ExecutionTimeout)));
                            to_run.push(id.clone());
                        }
                    }
                }
            }

            if !to_run.is_empty() {
                for id in to_run.drain(..) {
                    if let Some(mut proc) = self.waiting.remove(&id) {
                        proc.waiting = None;
                        self.running.push(proc);
                    } else if let Some(mut proc) = self.sleeping.remove(&id) {
                        proc.waiting = None;
                        self.running.push(proc);
                    }
                }
            }

            if !to_exit.is_empty() {
                for id in to_exit.drain(..) {
                    if let Some(proc) = self.waiting.remove(&id) {
                        if let Some(cb) = &mut self.done_callback {
                            if cb(graph, &proc) {
                                self.done.insert(id, proc);
                            } else {
                                self.errored.insert(id, proc);
                            }
                        } else {
                            self.done.insert(id, proc);
                        }
                    } else if let Some(proc) = self.sleeping.remove(&id) {
                        if let Some(cb) = &mut self.done_callback {
                            if cb(graph, &proc) {
                                self.done.insert(id, proc);
                            } else {
                                self.errored.insert(id, proc);
                            }
                        } else {
                            self.done.insert(id, proc);
                        }
                    } else {
                        self.move_running_to_done(graph, &id);
                    }
                }
            }
        }

        !self.running.is_empty() || !self.sleeping.is_empty()
    }

    #[cfg(any(feature = "js", feature = "tokio"))]
    /// Single step async.
    pub async fn async_single_step(&mut self, graph: &mut Graph, yield_to_outer: bool) -> bool {
        let res = self.run_single_step(graph);

        #[cfg(feature = "js")]
        {
            if yield_to_outer {
                let promise = js_sys::Promise::new(&mut |resolve, _reject| {
                    let window = web_sys::window().unwrap();
                    window.set_timeout_with_callback_and_timeout_and_arguments_0(
                        &resolve,
                        0  // 0ms timeout - yields to macrotask queue (instead of just microtasks)
                    ).unwrap();
                });
                wasm_bindgen_futures::JsFuture::from(promise).await.unwrap();
            }
        }
        res
    }

    #[cfg(any(feature = "js", feature = "tokio"))]
    /// Run every #[main] function within this graph.
    /// If throw is false, this will only return Ok.
    pub async fn async_run(graph: &mut Graph, context: Option<String>, throw: bool) -> Result<String, String> {
        Self::async_run_functions(graph, context, Func::main_functions(graph), throw).await
    }

    #[cfg(any(feature = "js", feature = "tokio"))]
    /// Run functions with the given attributes in this graph.
    pub async fn async_run_attribute_functions(graph: &mut Graph, context: Option<String>, attributes: &Option<FxHashSet<String>>, throw: bool) -> Result<String, String> {
        Self::async_run_functions(graph, context, Func::all_functions(graph, attributes), throw).await
    }

    #[cfg(any(feature = "js", feature = "tokio"))]
    /// Run all given functions.
    pub async fn async_run_functions(graph: &mut Graph, context: Option<String>, functions: FxHashSet<DataRef>, throw: bool) -> Result<String, String> {
        let mut rt = Self::default();
        for func_ref in functions {
            if let Some(context) = &context {
                for node in func_ref.data_nodes(&graph) {
                    if let Some(node_path) = node.node_path(&graph, true) {
                        let path = node_path.join(".");
                        if path.contains(context) {
                            let instruction = Arc::new(FuncCall {
                                as_ref: false,
                                cnull: false,
                                stack: false,
                                func: Some(func_ref),
                                search: None,
                                args: Default::default(),
                                oself: None,
                            }) as Arc<dyn Instruction>;
                            let proc = Process::from(instruction);
                            rt.push_running_proc(proc, graph);
                            break;
                        }
                    }
                }
            } else {
                let instruction = Arc::new(FuncCall {
                    as_ref: false,
                    cnull: false,
                    stack: false,
                    func: Some(func_ref),
                    search: None,
                    args: Default::default(),
                    oself: None,
                }) as Arc<dyn Instruction>;
                let proc = Process::from(instruction);
                rt.push_running_proc(proc, graph);
            }
        }

        const YIELD_INTERVAL_MS: u64 = 20;
        let mut yield_to_outer = false;
        let mut last_yield = web_time::Instant::now();
        while rt.async_single_step(graph, yield_to_outer).await {
            yield_to_outer = false;
            if last_yield.elapsed().as_millis() as u64 >= YIELD_INTERVAL_MS {
                yield_to_outer = true;
                last_yield = web_time::Instant::now();
            }
        }

        let mut output = String::from("");
        for (_, success) in &rt.done {
            if success.env.call_stack.len() < 1 && success.instructions.executed.len() > 0 {
                let func = success.instructions.executed[0].clone();
                if let Some(func) = func.as_dyn_any().downcast_ref::<Base>() {
                    match func {
                        Base::Literal(val) => {
                            if let Some(func_ref) = val.try_func() {
                                if let Some(name) = func_ref.data_name(graph) {
                                    let mut func_path = String::from("<unknown>");
                                    for node in func_ref.data_nodes(graph) {
                                        func_path = node.node_path(graph, true).unwrap().join(".");
                                    }
                                    // Only print something if there's a result
                                    if let Some(res) = &success.result {
                                        let suc_str = res.val.read().print(&graph);
                                        let msg = format!("{} {} {} {} {}\n", "main".purple(), func_path.italic().dimmed(), name.as_ref().italic().blue(), "...".dimmed(), suc_str.bold().bright_cyan());
                                        if output.len() < 1 { output.push('\n'); }
                                        output.push_str(&msg);
                                    }
                                }
                            }
                        },
                        _ => {}
                    }
                }
            }
        }
        for (_, errored) in &rt.errored {
            if errored.env.call_stack.len() > 0 {
                let func_ref = errored.env.call_stack.first().unwrap();
                if let Some(name) = func_ref.data_name(graph) {
                    let mut func_path = String::from("<unknown>");
                    for node in func_ref.data_nodes(graph) {
                        func_path = node.node_path(graph, true).unwrap().join(".");
                    }
                    let err_str = errored.trace(graph, 10);
                    let msg = format!("{} {} {} {} {} {}\n{}\n", "main".purple(), func_path.italic().dimmed(), name.as_ref().italic().blue(), "...".dimmed(), "failed".bold().red(), "@".dimmed(), err_str.bold().bright_cyan());
                    output.push_str(&msg);
                }
            }
        }

        if throw && rt.errored.len() > 0 {
            Err(output)
        } else {
            Ok(output)
        }
    }


    /*****************************************************************************
     * Run.
     *****************************************************************************/
    
    /// Run every #[main] function within this graph.
    /// If throw is false, this will only return Ok.
    pub fn run(graph: &mut Graph, context: Option<String>, throw: bool) -> Result<String, String> {
        Self::run_functions(graph, context, Func::main_functions(graph), throw)
    }

    /// Run functions with the given attributes in this graph.
    pub fn run_attribute_functions(graph: &mut Graph, context: Option<String>, attributes: &Option<FxHashSet<String>>, throw: bool) -> Result<String, String> {
        Self::run_functions(graph, context, Func::all_functions(graph, attributes), throw)
    }

    /// Run all given functions.
    pub fn run_functions(graph: &mut Graph, context: Option<String>, functions: FxHashSet<DataRef>, throw: bool) -> Result<String, String> {
        let mut rt = Self::default();
        for func_ref in functions {
            if let Some(context) = &context {
                for node in func_ref.data_nodes(&graph) {
                    if let Some(node_path) = node.node_path(&graph, true) {
                        let path = node_path.join(".");
                        if path.contains(context) {
                            let instruction = Arc::new(FuncCall {
                                as_ref: false,
                                cnull: false,
                                stack: false,
                                func: Some(func_ref),
                                search: None,
                                args: Default::default(),
                                oself: None,
                            }) as Arc<dyn Instruction>;
                            let proc = Process::from(instruction);
                            rt.push_running_proc(proc, graph);
                            break;
                        }
                    }
                }
            } else {
                let instruction = Arc::new(FuncCall {
                    as_ref: false,
                    cnull: false,
                    stack: false,
                    func: Some(func_ref),
                    search: None,
                    args: Default::default(),
                    oself: None,
                }) as Arc<dyn Instruction>;
                let proc = Process::from(instruction);
                rt.push_running_proc(proc, graph);
            }
        }

        rt.run_to_complete(graph);
        let mut output = String::from("");
        for (_, success) in &rt.done {
            if success.env.call_stack.len() < 1 && success.instructions.executed.len() > 0 {
                let func = success.instructions.executed[0].clone();
                if let Some(func) = func.as_dyn_any().downcast_ref::<Base>() {
                    match func {
                        Base::Literal(val) => {
                            if let Some(func_ref) = val.try_func() {
                                if let Some(name) = func_ref.data_name(graph) {
                                    let mut func_path = String::from("<unknown>");
                                    for node in func_ref.data_nodes(graph) {
                                        func_path = node.node_path(graph, true).unwrap().join(".");
                                    }
                                    // Only print something if there's a result
                                    if let Some(res) = &success.result {
                                        let suc_str = res.val.read().print(&graph);
                                        let msg = format!("{} {} {} {} {}\n", "main".purple(), func_path.italic().dimmed(), name.as_ref().italic().blue(), "...".dimmed(), suc_str.bold().bright_cyan());
                                        if output.len() < 1 { output.push('\n'); }
                                        output.push_str(&msg);
                                    }
                                }
                            }
                        },
                        _ => {}
                    }
                }
            }
        }
        for (_, errored) in &rt.errored {
            if errored.env.call_stack.len() > 0 {
                let func_ref = errored.env.call_stack.first().unwrap();
                if let Some(name) = func_ref.data_name(graph) {
                    let mut func_path = String::from("<unknown>");
                    for node in func_ref.data_nodes(graph) {
                        func_path = node.node_path(graph, true).unwrap().join(".");
                    }
                    let err_str = errored.trace(graph, 10);
                    let msg = format!("{} {} {} {} {} {}\n{}\n", "main".purple(), func_path.italic().dimmed(), name.as_ref().italic().blue(), "...".dimmed(), "failed".bold().red(), "@".dimmed(), err_str.bold().bright_cyan());
                    output.push_str(&msg);
                }
            }
        }

        if throw && rt.errored.len() > 0 {
            Err(output)
        } else {
            Ok(output)
        }
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
        for func_ref in Func::test_functions(&graph) {
            if let Some(context) = &context {
                for node in func_ref.data_nodes(&graph) {
                    if let Some(node_path) = node.node_path(&graph, true) {
                        let path = node_path.join(".");
                        if path.contains(context) {
                            let instruction = Arc::new(FuncCall {
                                as_ref: false,
                                cnull: false,
                                stack: false,
                                func: Some(func_ref),
                                search: None,
                                args: Default::default(),
                                oself: None,
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
                    as_ref: false,
                    cnull: false,
                    stack: false,
                    func: Some(func_ref),
                    search: None,
                    args: Default::default(),
                    oself: None,
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
                    if let Some(_err) = &failure.error {
                        err_str = failure.trace(graph, 10); // contains the error
                    }
                } else if failure.env.call_stack.len() < 1 && failure.instructions.executed.len() > 0 {
                    let func = failure.instructions.executed[0].clone();
                    if let Some(func) = func.as_dyn_any().downcast_ref::<Base>() {
                        match func {
                            Base::Literal(val) => {
                                if let Some(fref) = val.try_func() {
                                    func_ref = fref;
                                    err_str = format!("\texpected to error, but received a result of '{:?}'", failure.result);
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
                    output.push_str(&format!("\n{}: {}{}{} ...\n{}\n", "failed".bold().red(), func_path.italic().purple(), " @ ".dimmed(), name.as_ref().italic().blue(), err_str.bold().bright_cyan()));
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
            as_ref: false,
            cnull: false,
            stack: false,
            func: None,
            search: Some(search.into()),
            args: arguments,
            oself: None,
        });
        Self::eval(graph, instruction)
    }

    #[cfg(any(feature = "js", feature = "tokio"))]
    /// Call a singular function with this runtime.
    pub async fn async_call(graph: &mut Graph, search: &str, args: Vec<Val>) -> Result<Val, Error> {
        let mut arguments: Vector<Arc<dyn Instruction>> = Vector::default();
        for arg in args { arguments.push_back(Arc::new(Base::Literal(arg))); }
        let instruction = Arc::new(FuncCall {
            as_ref: false,
            cnull: false,
            stack: false,
            func: None,
            search: Some(search.into()),
            args: arguments,
            oself: None,
        });
        Self::async_eval(graph, instruction).await
    }
    
    /// Call a singular function with this runtime.
    pub fn call_func(graph: &mut Graph, func: &DataRef, args: Vec<Val>) -> Result<Val, Error> {
        if !func.type_of::<Func>(&graph) {
            return Err(Error::FuncDne(format!("Data Ptr not Func")));
        }
        let mut arguments: Vector<Arc<dyn Instruction>> = Vector::default();
        for arg in args { arguments.push_back(Arc::new(Base::Literal(arg))); }
        let instruction = Arc::new(FuncCall {
            as_ref: false,
            cnull: false,
            stack: false,
            func: Some(func.clone()),
            search: None,
            args: arguments,
            oself: None,
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

    #[cfg(any(feature = "js", feature = "tokio"))]
    /// Evaluate a single instruction.
    /// Creates a new runtime and process just for this (lightweight).
    /// Use this while parsing if needed.
    pub async fn async_eval(graph: &mut Graph, instruction: Arc<dyn Instruction>) -> Result<Val, Error> {
        let mut runtime = Self::default();
        let proc = Process::from(instruction);
        let pid = proc.env.pid.clone();
        
        runtime.push_running_proc(proc, graph);
        
        const YIELD_INTERVAL_MS: u64 = 20;
        let mut yield_to_outer = false;
        let mut last_yield = web_time::Instant::now();
        while runtime.async_single_step(graph, yield_to_outer).await {
            yield_to_outer = false;
            if last_yield.elapsed().as_millis() as u64 >= YIELD_INTERVAL_MS {
                yield_to_outer = true;
                last_yield = web_time::Instant::now();
            }
        }

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

    #[cfg(feature = "tokio")]
    /// Set tokio runtime handle for Stof.
    pub fn set_tokio_runtime(runtime: tokio::runtime::Handle) {
        let mut tokio_handle = TOKIO_HANDLE_OVERRIDE.write();
        *tokio_handle = Some(runtime);
    }
}
