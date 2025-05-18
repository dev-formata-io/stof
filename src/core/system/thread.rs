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

use std::{collections::{BTreeMap, HashMap, HashSet}, sync::{Arc, Mutex}, thread::JoinHandle, time::Duration};
use lazy_static::lazy_static;
use nanoid::nanoid;
use crate::{lang::SError, Library, SDataRef, SDoc, SFunc, SNodeRef, SUnits, SVal};


lazy_static! {
    // Global thread pool
    static ref THREAD_POOL: Arc<Mutex<ThreadPool>> = Arc::new(Mutex::new(ThreadPool::default()));

    // Global thread handles
    static ref THREAD_HANDLES: Arc<Mutex<HashMap<String, JoinHandle<()>>>> = Arc::new(Mutex::new(HashMap::default()));
}


/// Thread pool.
#[derive(Default)]
pub struct ThreadPool {
    /// Threads.
    pub threads: HashMap<String, Thread>,
}
impl ThreadPool {
    /// Spawn a new thread, splitting the doc (with optional context) and calling some functions.
    pub fn spawn(doc: &SDoc, calls: Vec<(SDataRef, Vec<SVal>)>, context: Option<HashSet<SNodeRef>>) -> String {
        let thread_id = format!("thread_{}", nanoid!(25));
        
        let mut split;
        if let Some(context) = context {
            split = doc.context_split(context);
        } else {
            split = doc.split();
        }

        let pid = split.processes.spawn();
        let tid = thread_id.clone();
        let join_handle = std::thread::spawn(move || {
            let mut results = Vec::new();
            for (func, params) in calls {
                match SFunc::call(&func, &pid, &mut split, params, true, false) {
                    Ok(res) => {
                        results.push(res);
                    },
                    Err(error) => {
                        results.push(SVal::String(error.to_string(&split.graph)));
                    }
                }
            }
            split.processes.kill(&pid);
            let mut pool = THREAD_POOL.lock().unwrap();
            if let Some(thread) = pool.threads.get_mut(&tid) {
                thread.doc = Some(split);
                thread.results = results;
            }
        });

        let mut pool = THREAD_POOL.lock().unwrap();
        pool.threads.insert(thread_id.clone(), Thread {
            id: thread_id.clone(),
            results: Default::default(),
            doc: None,
        });

        let mut handles = THREAD_HANDLES.lock().unwrap();
        handles.insert(thread_id.clone(), join_handle);

        thread_id
    }

    /// Join a thread back into this document, returning the function results.
    pub fn join(doc: &mut SDoc, thread_id: &str) -> Option<Vec<SVal>> {
        {
            let mut pool = THREAD_POOL.lock().unwrap();
            let mut done = false;
            if let Some(thread) = pool.threads.get_mut(thread_id) {
                done = thread.doc.is_some();
            }
            if done {
                let thread = pool.threads.remove(thread_id).unwrap();
                let mut other = thread.doc.unwrap();
                doc.join(&mut other);
                return Some(thread.results);
            }
        }

        let mut handle = None;
        {
            let mut handles = THREAD_HANDLES.lock().unwrap();
            if let Some(h) = handles.remove(thread_id) {
                handle = Some(h);
            }
        }
        if let Some(handle) = handle {
            let _ = handle.join();
            let mut pool = THREAD_POOL.lock().unwrap();
            if let Some(thread) = pool.threads.remove(thread_id) {
                let mut other = thread.doc.unwrap();
                doc.join(&mut other);
                return Some(thread.results);
            }
        }
        None
    }

    /// Join many thread ids.
    pub fn join_many(doc: &mut SDoc, ids: impl IntoIterator<Item = String>) -> BTreeMap<SVal, SVal> {
        let mut results = BTreeMap::new();
        for id in ids {
            if let Some(res) = Self::join(doc, &id) {
                results.insert(id.into(), SVal::Array(res));
            }
        }
        results
    }
}


/// Thread.
pub struct Thread {
    /// Thread ID.
    pub id: String,

    /// Resulting values.
    pub results: Vec<SVal>,

    /// Resulting Document.
    pub doc: Option<SDoc>,
}


/// Thread library.
#[derive(Default, Debug)]
pub struct ThreadLibrary;
impl Library for ThreadLibrary {
    fn scope(&self) -> String {
        "Thread".to_string()
    }

    fn call(&self, pid: &str, doc: &mut SDoc, name: &str, parameters: &mut Vec<SVal>) -> Result<SVal, SError> {
        match name {
            // Spawn a new thread, returning a join handle ID.
            // Parameters are a fn pointer followed by arguments Array (for each fn, there must be an array).
            // Any objects in the params will be interpreted as the context.
            // Thread.spawn(self, Other, (self.func, [42]));
            "spawn" => {
                let mut context = HashSet::new();
                let mut calls = Vec::new();
                for param in parameters.drain(..) {
                    match param {
                        SVal::Object(nref) => {
                            context.insert(nref);
                        },
                        SVal::Tuple(vals) => {
                            let mut func = None;
                            let mut args = Vec::new();
                            for val in vals {
                                match val {
                                    SVal::FnPtr(dref) => {
                                        func = Some(dref);
                                    },
                                    SVal::Array(vals) => {
                                        args = vals;
                                    },
                                    _ => {
                                        return Err(SError::thread(pid, &doc, "spawn", "invalid call parameter type"));
                                    }
                                }
                            }
                            if let Some(func) = func {
                                calls.push((func, args));
                            }
                        },
                        _ => {
                            return Err(SError::thread(pid, &doc, "spawn", "invalid thread parameter type"));
                        }
                    }
                }

                if calls.len() < 1 {
                    return Err(SError::thread(pid, &doc, "spawn", "cannot spawn a thread without a function to call"));
                }

                let ctx;
                if context.len() > 0 {
                    ctx = Some(context);
                } else {
                    ctx = None;
                }
                let thread_id = ThreadPool::spawn(&doc, calls, ctx);
                Ok(SVal::String(thread_id))
            },
            
            // Join thread IDs, returning call results and joining the documents together.
            "join" => {
                // call join with an array of handles
                if parameters.len() == 1 && parameters[0].is_array() {
                    match parameters.pop().unwrap() {
                        SVal::Array(vals) => {
                            *parameters = vals;
                        },
                        _ => {}
                    }
                }
                if parameters.len() == 1 {
                    let thread_id = parameters.pop().unwrap().owned_to_string();
                    match ThreadPool::join(doc, &thread_id) {
                        Some(results) => {
                            return Ok(SVal::Array(results));
                        },
                        None => {
                            return Ok(SVal::Null);
                        }
                    }
                } else if parameters.len() > 0 {
                    let ids = parameters.drain(..).map(|p| p.owned_to_string()).collect::<Vec<_>>();
                    return Ok(SVal::Map(ThreadPool::join_many(doc, ids)));
                }
                Err(SError::thread(pid, &doc, "join", "expected a thread ID to join"))
            },

            // Sleep for an amount of time.
            "sleep" => {
                if parameters.len() < 1 {
                    return Err(SError::thread(pid, &doc, "sleep", "must provide an amount of time to sleep"));
                }
                match &parameters[0] {
                    SVal::Number(num) => {
                        let val = num.float_with_units(SUnits::Milliseconds);
                        std::thread::sleep(Duration::from_millis(val.abs() as u64));
                        return Ok(SVal::Void);
                    },
                    _ => {}
                }
                Err(SError::thread(pid, &doc, "sleep", "sleep time must be a number"))
            },
            _ => {
                Err(SError::thread(pid, &doc, "NotFound", &format!("{} is not a function in the Thread Library", name)))
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::SDoc;

    #[test]
    fn stof_threads() {
        let mut doc = SDoc::src(r#"
            #[test]
            fn main() {
                let handles = [
                    Thread.spawn(self.context, (self.context.simple, [0, 4ms])),
                    Thread.spawn(self.context, (self.context.simple, [1, 200ms])),
                    Thread.spawn(self.context, (self.context.simple, [2, 100ms])),
                    Thread.spawn(self.context, (self.context.simple, [3, 40ms])),
                ];
                Thread.join(handles);
                assertEq(self.context.results, [0, 3, 2, 1]);
            }

            context: {
                // box value is shared between threads and not cloned by value...
                results: box([])

                fn simple(a: int, c: ms) {
                    Thread.sleep(c);
                    self.results.push(a);
                }
            }
        "#, "stof").unwrap();

        let res = doc.run_tests(true, None).unwrap();
        println!("{res}");
    }
}
