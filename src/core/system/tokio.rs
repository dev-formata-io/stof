//
// Copyright 2025 Formata, Inc. All rights reserved.
//

use std::{collections::{BTreeMap, HashMap, HashSet}, sync::Arc, time::{Duration, SystemTime, UNIX_EPOCH}};
use tokio::{runtime::Handle, sync::Mutex, task::JoinHandle, time::sleep};
use lazy_static::lazy_static;
use nanoid::nanoid;
use crate::{lang::SError, Library, SDataRef, SDoc, SFunc, SNodeRef, SUnits, SVal};


lazy_static! {
    // Global thread pool
    static ref TOKIO_POOL: Arc<Mutex<TokioPool>> = Arc::new(Mutex::new(TokioPool::default()));

    // Global thread handles
    static ref TOKIO_HANDLES: Arc<Mutex<HashMap<String, JoinHandle<()>>>> = Arc::new(Mutex::new(HashMap::default()));

    // Active handles
    static ref ACTIVE_HANDLES: Arc<Mutex<HashSet<String>>> = Arc::new(Mutex::new(HashSet::default()));
}


/// Task pool.
#[derive(Default)]
pub struct TokioPool {
    /// Tasks.
    pub tasks: HashMap<String, Task>,
}
impl TokioPool {
    /// Spawn a new thread, splitting the doc (with optional context) and calling some functions.
    pub fn spawn(doc: &SDoc, calls: Vec<(SDataRef, Vec<SVal>)>, context: Option<HashSet<SNodeRef>>, task_id: Option<String>) -> String {
        let mut thread_id = format!("tokasc_{}", nanoid!(10));
        if let Some(id) = task_id {
            thread_id = id;
        }
        
        let mut split;
        if let Some(context) = context {
            split = doc.context_split(context);
        } else {
            split = doc.split();
        }

        let pid = split.processes.spawn();
        let tid = thread_id.clone();
        let current = Handle::current();
        let join_handle = current.spawn_blocking(move || {
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

            let current = Handle::current();
            current.block_on(async move {
                {
                    let mut pool = TOKIO_POOL.lock().await;
                    if let Some(thread) = pool.tasks.get_mut(&tid) {
                        thread.doc = Some(split);
                        thread.results = results;
                        thread.finished_ts = Some(SystemTime::now().duration_since(UNIX_EPOCH).unwrap());
                    }
                }
                {
                    let mut tmp_handles = ACTIVE_HANDLES.lock().await;
                    tmp_handles.remove(&tid);
                }
                {
                    let mut handles = TOKIO_HANDLES.lock().await;
                    handles.remove(&tid);
                }
            });
        });

        let tid = thread_id.clone();
        current.block_on(async move {
            {
                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                let mut pool = TOKIO_POOL.lock().await;
                
                // Remove tasks that have been finished for longer than 30 seconds
                pool.tasks.retain(|_tid, task| {
                    if let Some(ts) = &task.finished_ts {
                        let diff = now - *ts;
                        if diff.as_secs() > 30 {
                            return false;
                        }
                    }
                    true
                });

                pool.tasks.insert(tid.clone(), Task {
                    id: tid.clone(),
                    results: Default::default(),
                    doc: None,
                    finished_ts: None,
                });
            }
            {
                let mut tmp_handles = ACTIVE_HANDLES.lock().await;
                tmp_handles.insert(tid.clone());
            }
            {
                let mut handles = TOKIO_HANDLES.lock().await;
                handles.insert(tid, join_handle);
            }
        });

        thread_id
    }

    /// Join a thread back into this document, returning the function results.
    pub fn join(doc: &mut SDoc, thread_id: &str) -> Option<Vec<SVal>> {
        let current = Handle::current();
        current.block_on(async move {
            {
                let mut pool = TOKIO_POOL.lock().await;
                let mut done = false;
                if let Some(thread) = pool.tasks.get_mut(thread_id) {
                    done = thread.doc.is_some();
                }
                if done {
                    let thread = pool.tasks.remove(thread_id).unwrap();
                    let mut other = thread.doc.unwrap();
                    doc.join(&mut other);
                    return Some(thread.results);
                }
            }

            let mut handle = None;
            {
                let mut handles = TOKIO_HANDLES.lock().await;
                if let Some(h) = handles.remove(thread_id) {
                    handle = Some(h);
                }
            }
            if let Some(handle) = handle {
                {
                    let mut tmp_handles = ACTIVE_HANDLES.lock().await;
                    tmp_handles.insert(thread_id.to_string());
                }
                let _ = handle.await;
                let mut pool = TOKIO_POOL.lock().await;
                if let Some(thread) = pool.tasks.remove(thread_id) {
                    let mut other = thread.doc.unwrap();
                    doc.join(&mut other);
                    return Some(thread.results);
                }
            }

            None
        })
    }

    /// Join many thread ids.
    pub fn join_many(doc: &mut SDoc, ids: impl IntoIterator<Item = String>) -> BTreeMap<SVal, SVal> {
        let mut results = BTreeMap::new();
        for id in ids {
            if let Some(mut res) = Self::join(doc, &id) {
                if res.len() == 1 {
                    results.insert(id.into(), res.pop().unwrap());
                } else {
                    results.insert(id.into(), SVal::Array(res));
                }
            }
        }
        results
    }

    /// Is an active handle?
    pub fn is_handle(task_id: &str) -> bool {
        let current = Handle::current();
        current.block_on(async {
            {
                let tmp_handles = ACTIVE_HANDLES.lock().await;
                if tmp_handles.contains(task_id) {
                    return true;
                }
            }
            let handles = TOKIO_HANDLES.lock().await;
            handles.contains_key(task_id)
        })
    }

    /// Is a task running?
    pub fn is_running(task_id: &str) -> bool {
        let current = Handle::current();
        current.block_on(async {
            let pool = TOKIO_POOL.lock().await;
            if let Some(task) = pool.tasks.get(task_id) {
                return task.finished_ts.is_none();
            }
            false
        })
    }
}


/// Task.
pub struct Task {
    /// Task ID.
    #[allow(unused)]
    pub id: String,

    /// Resulting values.
    pub results: Vec<SVal>,

    /// Resulting Document.
    pub doc: Option<SDoc>,

    /// Finished timestamp.
    pub finished_ts: Option<Duration>,
}


/// Tokio "Async" library.
#[derive(Default, Debug)]
pub struct TokioLibrary;
impl Library for TokioLibrary {
    fn scope(&self) -> String {
        "Async".to_string()
    }

    fn call(&self, pid: &str, doc: &mut SDoc, name: &str, parameters: &mut Vec<SVal>) -> Result<SVal, SError> {
        match name {
            // Spawn a new task, returning a join handle ID.
            // Parameters are a fn pointer followed by arguments Array (for each fn, there must be an array).
            // Any objects in the params will be interpreted as the context.
            // Tokio.spawn(self, Other, (self.func, [42]));
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
                            return Err(SError::thread(pid, &doc, "spawn", "invalid task parameter type"));
                        }
                    }
                }

                if calls.len() < 1 {
                    return Err(SError::thread(pid, &doc, "spawn", "cannot spawn a task without a function to call"));
                }

                let ctx;
                if context.len() > 0 {
                    ctx = Some(context);
                } else {
                    ctx = None;
                }
                let thread_id = TokioPool::spawn(&doc, calls, ctx, None);
                Ok(SVal::String(thread_id))
            },
            
            // Join (or await) task IDs, returning call results and joining the documents together.
            "join" |
            "await" => {
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
                    match TokioPool::join(doc, &thread_id) {
                        Some(mut results) => {
                            if results.len() == 1 {
                                return Ok(results.pop().unwrap());
                            }
                            return Ok(SVal::Array(results));
                        },
                        None => {
                            return Ok(SVal::Null);
                        }
                    }
                } else if parameters.len() > 0 {
                    let ids = parameters.drain(..).map(|p| p.owned_to_string()).collect::<Vec<_>>();
                    return Ok(SVal::Map(TokioPool::join_many(doc, ids)));
                }
                Err(SError::thread(pid, &doc, "join", "expected a task ID to join"))
            },

            // Sleep for an amount of time.
            "sleep" => {
                if parameters.len() < 1 {
                    return Err(SError::thread(pid, &doc, "sleep", "must provide an amount of time to sleep"));
                }
                match &parameters[0] {
                    SVal::Number(num) => {
                        let val = num.float_with_units(SUnits::Milliseconds);
                        let current = Handle::current();
                        current.block_on(async {
                            sleep(Duration::from_millis(val.abs() as u64)).await;
                        });
                        return Ok(SVal::Void);
                    },
                    _ => {}
                }
                Err(SError::thread(pid, &doc, "sleep", "sleep time must be a number"))
            },

            // Is this a valid task handle?
            "isHandle" => {
                if parameters.len() == 1 {
                    let thread_id = parameters.pop().unwrap().owned_to_string();
                    let current = Handle::current();
                    let res = current.block_on(async {
                        {
                            let tmp_handles = ACTIVE_HANDLES.lock().await;
                            if tmp_handles.contains(&thread_id) {
                                return true;
                            }
                        }
                        let handles = TOKIO_HANDLES.lock().await;
                        handles.contains_key(&thread_id)
                    });
                    return Ok(SVal::Bool(res));
                }
                Err(SError::thread(pid, &doc, "isHandle", "expecting a string handle ID"))
            },

            // Task is running?
            "isRunning" => {
                if parameters.len() == 1 {
                    let thread_id = parameters.pop().unwrap().owned_to_string();
                    return Ok(SVal::Bool(TokioPool::is_running(&thread_id)));
                }
                Err(SError::thread(pid, &doc, "isRunning", "expecting a string handle ID"))
            },

            // Abort a tokio task.
            "abort" => {
                if parameters.len() < 1 { return Err(SError::thread(pid, &doc, "abort", "expecting a task ID to abort")); }
                let id = parameters[0].to_string();
                let current = Handle::current();
                current.spawn(async move {
                    {
                        let mut handles = TOKIO_HANDLES.lock().await;
                        if let Some(h) = handles.remove(&id) {
                            h.abort();
                        }
                    }
                    {
                        let mut tmp_handles = ACTIVE_HANDLES.lock().await;
                        tmp_handles.remove(&id);
                    }
                    {
                        let mut pool = TOKIO_POOL.lock().await;
                        pool.tasks.remove(&id);
                    }
                });
                Ok(SVal::Void)
            },
            _ => {
                Err(SError::thread(pid, &doc, "NotFound", &format!("{} is not a function in the Async Library", name)))
            }
        }
    }
}
