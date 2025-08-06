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

use core::str;
use std::{ops::Deref, str::FromStr, sync::Arc, time::Duration};
use arcstr::{literal, ArcStr};
use bytes::Bytes;
use imbl::{vector, OrdMap};
use reqwest::{header::{HeaderMap, HeaderName, CONTENT_TYPE}, Client, Method};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use tokio::{runtime::Runtime, sync::{mpsc::{self, Sender}, Mutex}};
use crate::{model::{Graph, LibFunc, Param}, runtime::{instruction::{Instruction, Instructions}, instructions::Base, proc::ProcEnv, reset, wake, Error, Num, NumT, Type, Units, Val, ValRef, Variable, WakeRef}};

/// Http library name.
pub(self) const HTTP_LIB: ArcStr = literal!("Http");


/// Insert the Http library into a graph.
pub fn insert_http_lib(graph: &mut Graph) {
    let sender = start_http_runtime();
    graph.insert_libfunc(http_fetch(&sender));
}


/// Http.fetch(url: str, method: str, body: blob | str, headers: map, timeout: s, query: map, bearer: str) -> map;
fn http_fetch(sender: &Sender<HTTPRequest>) -> LibFunc {
    let sender = sender.clone();
    LibFunc {
        library: HTTP_LIB.clone(),
        name: "fetch".into(),
        is_async: true,
        docs: "# Send HTTP Request (fetch)\nExecute an async HTTP request, returning a map of the results.".into(),
        params: vector![
            Param { name: "url".into(), param_type: Type::Str, default: None },
            Param { name: "method".into(), param_type: Type::Str, default: Some(Arc::new(Base::Literal(Val::Str("get".into())))) },
            Param { name: "body".into(), param_type: Type::Union(vector![Type::Blob, Type::Str]), default: Some(Arc::new(Base::Literal(Val::Null))) },
            Param { name: "headers".into(), param_type: Type::Map, default: Some(Arc::new(Base::Literal(Val::Null))) },
            Param { name: "timeout".into(), param_type: Type::Num(NumT::Float), default: Some(Arc::new(Base::Literal(Val::Null))) },
            Param { name: "query".into(), param_type: Type::Map, default: Some(Arc::new(Base::Literal(Val::Null))) },
            Param { name: "bearer".into(), param_type: Type::Str, default: Some(Arc::new(Base::Literal(Val::Null))) },
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(move |_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(HttpIns::Send(sender.clone())));
            Ok(instructions)
        })
    }
}


pub(self) struct HTTPRequest {
    pub url: String,
    pub method: Method,
    pub headers: HeaderMap,
    pub bearer: Option<String>,
    pub body: Option<Bytes>,
    pub timeout: Option<Duration>,
    pub query: Option<FxHashMap<String, String>>,

    /// Put results from the request here.
    pub results: Arc<Mutex<OrdMap<ValRef<Val>, ValRef<Val>>>>,

    /// Call wake at the end so that the process resumes with the results.
    pub waker: WakeRef,
}
fn start_http_runtime() -> Sender<HTTPRequest> {
    let (send, recv) = mpsc::channel(100);
    std::thread::spawn(move || {
        let rt = Runtime::new().expect("failed to create HTTP tokio runtime");
        rt.block_on(http_receiver(recv));
    });
    send
}
async fn http_receiver(mut recv: mpsc::Receiver<HTTPRequest>) {
    let client = Client::new();
    while let Some(req) = recv.recv().await {
        // spawn a new task on this runtime so that this one isn't held up
        let client = client.clone();
        tokio::spawn(async move {
            let mut builder = client.request(req.method, req.url);
            builder = builder.headers(req.headers);
            
            if let Some(bearer) = req.bearer {
                builder = builder.bearer_auth(bearer);
            }
            if let Some(body) = req.body {
                builder = builder.body(body);
            }
            if let Some(timeout) = req.timeout {
                builder = builder.timeout(timeout);
            }
            if let Some(query) = req.query {
                builder = builder.query(&query);
            }

            if let Ok(request) = builder.build() {
                let mut results = req.results.lock().await;
                match client.execute(request).await {
                    Ok(response) => {
                        results.insert(ValRef::new(Val::Str("status".into())), ValRef::new(Val::Num(Num::Int(response.status().as_u16() as i64))));
                        results.insert(ValRef::new(Val::Str("ok".into())), ValRef::new(Val::Bool(response.status().is_success())));

                        let mut headers = OrdMap::default();
                        for (k, v) in response.headers() {
                            if let Ok(val) = v.to_str() {
                                headers.insert(ValRef::new(Val::Str(k.as_str().into())), ValRef::new(Val::Str(val.into())));
                            }
                        }
                        results.insert(ValRef::new(Val::Str("headers".into())), ValRef::new(Val::Map(headers)));
                    
                        if let Some(ctype) = response.headers().get(CONTENT_TYPE) {
                            if let Ok(val) = ctype.to_str() {
                                results.insert(ValRef::new(Val::Str("content_type".into())), ValRef::new(Val::Str(val.into())));
                            }
                        }

                        if let Ok(bytes) = response.bytes().await {
                            if let Ok(res) = str::from_utf8(&bytes) {
                                results.insert(ValRef::new(Val::Str("text".into())), ValRef::new(Val::Str(res.into())));
                            }
                            results.insert(ValRef::new(Val::Str("bytes".into())), ValRef::new(Val::Blob(bytes.to_vec())));
                        }
                    },
                    Err(error) => {
                        results.insert(ValRef::new(Val::Str("error".into())), ValRef::new(Val::Str(error.to_string().into())));
                    },
                }
            }

            wake(&req.waker); // wake the process that is waiting
        });
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// HTTP instructions.
pub(self) enum HttpIns {
    #[serde(skip)]
    Send(Sender<HTTPRequest>),

    #[serde(skip)]
    InternalSend(Sender<HTTPRequest>, WakeRef, Arc<Mutex<OrdMap<ValRef<Val>, ValRef<Val>>>>),

    #[serde(skip)]
    Extract(Arc<Mutex<OrdMap<ValRef<Val>, ValRef<Val>>>>),
}
#[typetag::serde(name = "HttpIns")]
impl Instruction for HttpIns {
    fn exec(&self, env: &mut ProcEnv, graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        match self {
            Self::Send(sender) => {
                let wake_ref = WakeRef::default();
                let result_map: Arc<Mutex<OrdMap<ValRef<Val>, ValRef<Val>>>> = Arc::new(Mutex::new(OrdMap::default()));

                let mut instructions = Instructions::default();
                instructions.push(Arc::new(Self::InternalSend(sender.clone(), wake_ref.clone(), result_map.clone())));
                instructions.push(Arc::new(Base::CtrlSleepRef(wake_ref)));
                instructions.push(Arc::new(Self::Extract(result_map)));
                return Ok(Some(instructions));
            },
            Self::InternalSend(sender, waker, result_map) => {
                // Reset the result map and the waker value (Ex. in a loop)
                reset(waker);
                {
                    let mut results = result_map.blocking_lock();
                    results.clear();
                }

                // create the HTTPRequest and use the sender to send it off to be executed in the background
                // Http.send(url: str, method: str, body: blob | str, headers: map, timeout: ms, query: map, bearer: str) -> map;
                let mut url = String::default();
                let mut method = Method::GET;
                let mut headers = HeaderMap::default();
                let mut bearer = None;
                let mut body = None;
                let mut timeout = None;
                let mut query = None;
                if let Some(bearer_var) = env.stack.pop() {
                    match bearer_var.val.read().deref() {
                        Val::Void |
                        Val::Null => {
                            // leave empty
                        },
                        Val::Str(token) => {
                            bearer = Some(token.to_string());
                        },
                        _ => {
                            return Err(Error::HttpArgs(format!("bearer argument must be a string")));
                        }
                    }
                }
                if let Some(query_var) = env.stack.pop() {
                    match query_var.val.read().deref() {
                        Val::Void |
                        Val::Null => {
                            // leave empty
                        },
                        Val::Map(query_map) => {
                            let mut qu = FxHashMap::default();
                            for (k, v) in query_map {
                                let key = k.read().print(&graph);
                                let value = v.read().print(&graph);
                                qu.insert(key, value);
                            }
                            query = Some(qu);
                        },
                        _ => {
                            return Err(Error::HttpArgs(format!("query argument must be a map")));
                        }
                    }
                }
                if let Some(timeout_var) = env.stack.pop() {
                    match timeout_var.val.read().deref() {
                        Val::Void |
                        Val::Null => {
                            // leave empty
                        },
                        Val::Num(num) => {
                            let seconds = num.float(Some(Units::Seconds));
                            timeout = Some(Duration::from_secs(seconds as u64));
                        },
                        _ => {
                            return Err(Error::HttpArgs(format!("timeout must be a number")));
                        }
                    }
                }
                if let Some(headers_var) = env.stack.pop() {
                    match headers_var.val.read().deref() {
                        Val::Void |
                        Val::Null => {
                            // leave empty
                        },
                        Val::Map(header_map) => {
                            for (k, v) in header_map {
                                let key = k.read().print(&graph);
                                if let Ok(name) = HeaderName::from_str(&key) {
                                    headers.insert(name, v.read().print(&graph).parse().unwrap());
                                }
                            }
                        },
                        _ => {
                            return Err(Error::HttpArgs(format!("headers must be a map")));
                        }
                    }
                }
                if let Some(body_var) = env.stack.pop() {
                    match body_var.val.read().deref() {
                        Val::Void |
                        Val::Null => {
                            // leave empty
                        },
                        Val::Str(str) => {
                            body = Some(Bytes::from(str.to_string()));
                        },
                        Val::Blob(blob) => {
                            body = Some(Bytes::from(blob.clone()));
                        },
                        _ => {
                            return Err(Error::HttpArgs(format!("body must be either a string or a blob")));
                        }
                    }
                }
                if let Some(method_var) = env.stack.pop() {
                    match method_var.val.read().deref() {
                        Val::Void |
                        Val::Null => {
                            // leave as GET
                        },
                        Val::Str(method_val) => {
                            match method_val.to_lowercase().as_str() {
                                "get" => method = Method::GET,
                                "post" => method = Method::POST,
                                "put" => method = Method::PUT,
                                "patch" => method = Method::PATCH,
                                "delete" => method = Method::DELETE,
                                "head" => method = Method::HEAD,
                                "options" => method = Method::OPTIONS,
                                "trace" => method = Method::TRACE,
                                "connect" => method = Method::CONNECT,
                                _ => {
                                    return Err(Error::HttpArgs(format!("method must be one of 'get', 'post', 'put', 'patch', 'delete', 'head', 'options', 'trace', or 'connect'")));
                                }
                            }
                        },
                        _ => {
                            return Err(Error::HttpArgs(format!("method must be one of 'get', 'post', 'put', 'patch', 'delete', 'head', 'options', 'trace', or 'connect'")));
                        }
                    }
                }
                if let Some(url_var) = env.stack.pop() {
                    match url_var.val.read().deref() {
                        Val::Str(url_val) => {
                            url = url_val.to_string();
                        },
                        _ => {
                            return Err(Error::HttpArgs(format!("URL must be a string")));
                        }
                    }
                }

                let request = HTTPRequest {
                    url,
                    method,
                    headers,
                    bearer,
                    body,
                    timeout,
                    query,
                    results: result_map.clone(),
                    waker: waker.clone(),
                };
                match sender.blocking_send(request) {
                    Ok(_) => {},
                    Err(error) => {
                        return Err(Error::HttpSendError(error.to_string()));
                    },
                }
            },
            Self::Extract(map) => {
                let map = map.blocking_lock();
                let map = map.deref().clone(); // structural sharing makes more efficient
                env.stack.push(Variable::val(Val::Map(map))); // map return value
            },
        }
        Ok(None)
    }
}
