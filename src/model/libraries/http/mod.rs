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
use lazy_static::lazy_static;
use reqwest::{header::{HeaderMap, HeaderName, CONTENT_TYPE}, Client, Method};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use crate::{model::{Graph, LibFunc, Param}, runtime::{instruction::{Instruction, Instructions}, instructions::Base, proc::ProcEnv, reset, wake, Error, Num, NumT, Type, Units, Val, ValRef, Variable, WakeRef}};


lazy_static! {
    static ref HTTP_CLIENT: Arc<Client> = Arc::new(Client::new());
}


/// Http library name.
pub(self) const HTTP_LIB: ArcStr = literal!("Http");


/// Insert the Http library into a graph.
pub fn insert_http_lib(graph: &mut Graph) {
    // Http.fetch(url: str, method: str, body: blob | str, headers: map, timeout: s, query: map, bearer: str) -> map;
    graph.insert_libfunc(LibFunc {
        library: HTTP_LIB.clone(),
        name: "fetch".into(),
        is_async: true,
        docs: r#"# async Http.fetch(url: str, method: str = "get", body: str | blob = null, headers: map = null, timeout: seconds = null, query: map = null, bearer: str = null) -> Promise<map>
Make an HTTP request, using the thread pool in the background so that other Stof processes can continue running.
```rust
const resp = await Http.fetch("https://restcountries.com/v3.1/region/europe");
assert(resp.get('text').len() > 100);
```"#.into(),
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
            let wake_ref = WakeRef::default();
            let result_map: Arc<Mutex<OrdMap<ValRef<Val>, ValRef<Val>>>> = Arc::new(Mutex::new(OrdMap::default()));
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(HttpIns::InternalSend(wake_ref.clone(), result_map.clone())));
            instructions.push(Arc::new(Base::CtrlSleepRef(wake_ref)));
            instructions.push(Arc::new(HttpIns::Extract(result_map)));
            Ok(instructions)
        })
    });

    // Http.parse(response: map, context: obj = self) -> obj
    graph.insert_libfunc(LibFunc {
        library: HTTP_LIB.clone(),
        name: "parse".into(),
        is_async: false,
        docs: r#"# Http.parse(response: map, context: obj = self) -> obj
Parse an HTTP response into the context object (also the return value), using the response "Content-Type" header as a Stof format (binary import). Default content type if not found in response headers is "stof". Will throw an error if the format isn't accepted by this graph, or if the body doesn't exist.
```rust
const resp = await Http.fetch("https://restcountries.com/v3.1/region/europe");
const body = new {};
try Http.parse(resp, body);
catch { /* didn't work out.. */ }
```"#.into(),
        params: vector![
            Param { name: "response".into(), param_type: Type::Map, default: None },
            Param { name: "context".into(), param_type: Type::Void, default: Some(Arc::new(Base::Literal(Val::Null))) },
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(move |_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(HttpIns::ParseResponse));
            Ok(instructions)
        })
    });

    // Http.success(response: map) -> bool
    graph.insert_libfunc(LibFunc {
        library: HTTP_LIB.clone(),
        name: "success".into(),
        is_async: false,
        docs: r#"# Http.success(response: map) -> bool
Was the request successful? Meaning, is the response 'status' between [200, 299]?
```rust
const resp = await Http.fetch("https://restcountries.com/v3.1/region/europe");
assert(Http.success(resp));
```"#.into(),
        params: vector![
            Param { name: "response".into(), param_type: Type::Map, default: None },
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(move |_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(HttpIns::ResponseSuccess));
            Ok(instructions)
        })
    });

    // Http.client_error(response: map) -> bool
    graph.insert_libfunc(LibFunc {
        library: HTTP_LIB.clone(),
        name: "client_error".into(),
        is_async: false,
        docs: r#"# Http.client_error(response: map) -> bool
Was the request a client error? Meaning, is the response 'status' between [400, 499]?
```rust
const resp = await Http.fetch("https://restcountries.com/v3.1/region/europe");
assert_not(Http.client_error(resp));
```"#.into(),
        params: vector![
            Param { name: "response".into(), param_type: Type::Map, default: None },
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(move |_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(HttpIns::ClientError));
            Ok(instructions)
        })
    });

    // Http.server_error(response: map) -> bool
    graph.insert_libfunc(LibFunc {
        library: HTTP_LIB.clone(),
        name: "server_error".into(),
        is_async: false,
        docs: r#"# Http.server_error(response: map) -> bool
Was the request a server error? Meaning, is the response 'status' between [500, 599]?
```rust
const resp = await Http.fetch("https://restcountries.com/v3.1/region/europe");
assert_not(Http.server_error(resp));
```"#.into(),
        params: vector![
            Param { name: "response".into(), param_type: Type::Map, default: None },
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(move |_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(HttpIns::ServerError));
            Ok(instructions)
        })
    });
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
impl HTTPRequest {
    /// Send this http request with the given process env (potentially blocking).
    /// Put results into the results map.
    pub fn send(self, env: &mut ProcEnv) {
        if let Some(handle) = &env.tokio_runtime {
            handle.spawn(async move {
                let client = &HTTP_CLIENT;
                let mut builder = client.request(self.method, self.url);
                builder = builder.headers(self.headers);
                
                if let Some(bearer) = self.bearer {
                    builder = builder.bearer_auth(bearer);
                }
                if let Some(body) = self.body {
                    builder = builder.body(body);
                }
                if let Some(timeout) = self.timeout {
                    builder = builder.timeout(timeout);
                }
                if let Some(query) = self.query {
                    builder = builder.query(&query);
                }

                if let Ok(request) = builder.build() {
                    let mut results = self.results.lock().await;
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
                                // TODO: v0.9 remove "text" and only use the lib body or parse functions
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

                wake(&self.waker); // wake the process that is waiting
            });
        } else {
            // TODO: blocking reqwests (http depends on tokio at the moment)
            wake(&self.waker);
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// HTTP instructions.
pub(self) enum HttpIns {
    #[serde(skip)]
    InternalSend(WakeRef, Arc<Mutex<OrdMap<ValRef<Val>, ValRef<Val>>>>),

    #[serde(skip)]
    Extract(Arc<Mutex<OrdMap<ValRef<Val>, ValRef<Val>>>>),

    /// Parse an HTTP response map into an object context, using the recieved content_type (or STOF/JSON by default..)
    ParseResponse,
    /// Was the response successful (200 <= status <= 299)?
    ResponseSuccess,
    /// Client error response (400 <= status <= 499)?
    ClientError,
    /// Server error response (500 <= status <= 599)?
    ServerError,
}
#[typetag::serde(name = "HttpIns")]
impl Instruction for HttpIns {
    fn exec(&self, env: &mut ProcEnv, graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        match self {
            Self::InternalSend(waker, result_map) => {
                // Reset the result map and the waker (shouldn't happen, but good redundancy)
                if reset(waker) {
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
                request.send(env);
            },
            Self::Extract(map) => {
                let map = map.blocking_lock();
                let map = map.deref().clone(); // structural sharing makes more efficient
                env.stack.push(Variable::val(Val::Map(map))); // map return value
            },
            
            // Http.parse(response: map, context: obj = self) -> obj
            Self::ParseResponse => {
                let mut context = env.self_ptr();
                if let Some(context_var) = env.stack.pop() {
                    if let Some(obj) = context_var.try_obj() {
                        context = obj;
                    }
                }
                if let Some(response) = env.stack.pop() {
                    match response.val.read().deref() {
                        Val::Map(response) => {
                            let mut content_type = "stof".to_string();
                            if let Some(ctype) = response.get(&ValRef::new(Val::Str("content_type".into()))) {
                                content_type = ctype.read().to_string();
                            }
                            if let Some(body) = response.get(&ValRef::new(Val::Str("bytes".into()))) {
                                match body.read().deref() {
                                    Val::Blob(bytes) => {
                                        let bytes = Bytes::from(bytes.clone()); // TODO: v0.9 Blob(vec) -> Blob(Bytes)
                                        graph.binary_import(&content_type, bytes, Some(context.clone()))?;
                                        env.stack.push(Variable::val(Val::Obj(context)));
                                    },
                                    _ => {
                                        return Err(Error::HttpSendError(format!("parse response map 'bytes' body must be a blob")));
                                    }
                                }
                            } else {
                                return Err(Error::HttpSendError(format!("parse response map must have a 'bytes' key-value blob body")));
                            }
                        },
                        _ => {
                            return Err(Error::HttpSendError(format!("parse response must be a map")))
                        }
                    }
                }
            },

            // Http.success(response: map) -> bool
            Self::ResponseSuccess => {
                if let Some(response_var) = env.stack.pop() {
                    match response_var.val.read().deref() {
                        Val::Map(response) => {
                            if let Some(status) = response.get(&ValRef::new(Val::Str("status".into()))) {
                                match status.read().deref() {
                                    Val::Num(num) => {
                                        let v = num.int();
                                        env.stack.push(Variable::val(Val::Bool(v >= 200 && v < 300)));
                                    },
                                    _ => {
                                        env.stack.push(Variable::val(Val::Bool(false)));
                                    }
                                }
                            } else {
                                env.stack.push(Variable::val(Val::Bool(false)));
                            }
                        },
                        _ => {}
                    }
                }
            },
            // Http.client_error(response: map) -> bool
            Self::ClientError => {
                if let Some(response_var) = env.stack.pop() {
                    match response_var.val.read().deref() {
                        Val::Map(response) => {
                            if let Some(status) = response.get(&ValRef::new(Val::Str("status".into()))) {
                                match status.read().deref() {
                                    Val::Num(num) => {
                                        let v = num.int();
                                        env.stack.push(Variable::val(Val::Bool(v >= 400 && v < 500)));
                                    },
                                    _ => {
                                        env.stack.push(Variable::val(Val::Bool(false)));
                                    }
                                }
                            } else {
                                env.stack.push(Variable::val(Val::Bool(false)));
                            }
                        },
                        _ => {}
                    }
                }
            },
            // Http.server_error(response: map) -> bool
            Self::ServerError => {
                if let Some(response_var) = env.stack.pop() {
                    match response_var.val.read().deref() {
                        Val::Map(response) => {
                            if let Some(status) = response.get(&ValRef::new(Val::Str("status".into()))) {
                                match status.read().deref() {
                                    Val::Num(num) => {
                                        let v = num.int();
                                        env.stack.push(Variable::val(Val::Bool(v >= 500 && v < 600)));
                                    },
                                    _ => {
                                        env.stack.push(Variable::val(Val::Bool(false)));
                                    }
                                }
                            } else {
                                env.stack.push(Variable::val(Val::Bool(false)));
                            }
                        },
                        _ => {}
                    }
                }
            },
        }
        Ok(None)
    }
}
