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
use std::{ops::{Deref, DerefMut}, str::FromStr, sync::Arc, time::Duration};
use arcstr::{literal, ArcStr};
use bytes::Bytes;
use imbl::{vector, OrdMap};
use lazy_static::lazy_static;
use reqwest::{header::{HeaderMap, HeaderName, CONTENT_TYPE}, Method};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use crate::{model::{Graph, LibFunc, Param}, runtime::{instruction::{Instruction, Instructions}, instructions::Base, proc::ProcEnv, wake, Error, Num, NumT, Type, Units, Val, ValRef, Variable, WakeRef}};

#[cfg(feature = "tokio")]
use reqwest::Client;

#[cfg(not(feature = "tokio"))]
use reqwest::blocking::Client;

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
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(HttpIns::SendRequest(wake_ref.clone())));
            instructions.push(Arc::new(Base::CtrlSleepRef(wake_ref)));
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

    // Http.text(response: map) -> str
    graph.insert_libfunc(LibFunc {
        library: HTTP_LIB.clone(),
        name: "text".into(),
        is_async: false,
        docs: r#"# Http.text(response: map) -> str
Extract a UTF-8 text body from this response map (Equivalent to Http.blob(response) as str).
```rust
const resp = await Http.fetch("https://restcountries.com/v3.1/region/europe");
const body = Http.text(resp);
```"#.into(),
        params: vector![
            Param { name: "response".into(), param_type: Type::Map, default: None },
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(move |_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(HttpIns::TextBody));
            Ok(instructions)
        })
    });

    // Http.size(response: map) -> bytes
    graph.insert_libfunc(LibFunc {
        library: HTTP_LIB.clone(),
        name: "size".into(),
        is_async: false,
        docs: r#"# Http.size(response: map) -> bytes
Extract the response body size in bytes.
```rust
const resp = await Http.fetch("https://restcountries.com/v3.1/region/europe");
const mib_body_size = Http.size(resp) as MiB;
```"#.into(),
        params: vector![
            Param { name: "response".into(), param_type: Type::Map, default: None },
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(move |_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(HttpIns::BodySize));
            Ok(instructions)
        })
    });

    // Http.blob(response: map) -> blob
    graph.insert_libfunc(LibFunc {
        library: HTTP_LIB.clone(),
        name: "blob".into(),
        is_async: false,
        docs: r#"# Http.blob(response: map) -> blob
Extract the body of this response as bytes.
```rust
const resp = await Http.fetch("https://restcountries.com/v3.1/region/europe");
const body = Http.blob(resp);
```"#.into(),
        params: vector![
            Param { name: "response".into(), param_type: Type::Map, default: None },
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(move |_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(HttpIns::BlobBody));
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

    /// Put results from the request into this value.
    pub results: ValRef<Val>,

    /// Call wake at the end so that the process resumes with the results.
    pub waker: WakeRef,
}
impl HTTPRequest {
    #[allow(unused)]
    /// Send this http request with the given process env (potentially blocking).
    /// Put results into the results map.
    pub fn send(self, env: &mut ProcEnv) {
        #[cfg(feature = "tokio")]
        {
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
                        match client.execute(request).await {
                            Ok(response) => {
                                let status = response.status();
                                let headers = response.headers().clone();
                                let response_body = response.bytes().await;

                                let mut results = self.results.write();
                                match results.deref_mut() {
                                    Val::Map(results) => {
                                        results.insert(ValRef::new(Val::Str("status".into())), ValRef::new(Val::Num(Num::Int(status.as_u16() as i64))));
                                        results.insert(ValRef::new(Val::Str("ok".into())), ValRef::new(Val::Bool(status.is_success())));

                                        let mut result_headers = OrdMap::default();
                                        for (k, v) in &headers {
                                            if let Ok(val) = v.to_str() {
                                                result_headers.insert(ValRef::new(Val::Str(k.as_str().into())), ValRef::new(Val::Str(val.into())));
                                            }
                                        }
                                        results.insert(ValRef::new(Val::Str("headers".into())), ValRef::new(Val::Map(result_headers)));
                                    
                                        if let Some(ctype) = headers.get(CONTENT_TYPE) {
                                            if let Ok(val) = ctype.to_str() {
                                                results.insert(ValRef::new(Val::Str("content_type".into())), ValRef::new(Val::Str(val.into())));
                                            }
                                        }

                                        if let Ok(bytes) = response_body {
                                            results.insert(ValRef::new(Val::Str("bytes".into())), ValRef::new(Val::Blob(bytes)));
                                        }
                                    },
                                    _ => {},
                                }
                            },
                            Err(error) => {
                                let mut results = self.results.write();
                                match results.deref_mut() {
                                    Val::Map(results) => {
                                        results.insert(ValRef::new(Val::Str("error".into())), ValRef::new(Val::Str(error.to_string().into())));
                                    },
                                    _ => {},
                                }
                            }
                        }
                    }
                    wake(&self.waker); // wake the process that is waiting
                });
            } else {
                // no runtime to work with, so fall back on a blocking client... boo
                let client = reqwest::blocking::Client::new();
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
                    match client.execute(request) {
                        Ok(response) => {
                            let status = response.status();
                            let headers = response.headers().clone();
                            let response_body = response.bytes();

                            let mut results = self.results.write();
                            match results.deref_mut() {
                                Val::Map(results) => {
                                    results.insert(ValRef::new(Val::Str("status".into())), ValRef::new(Val::Num(Num::Int(status.as_u16() as i64))));
                                    results.insert(ValRef::new(Val::Str("ok".into())), ValRef::new(Val::Bool(status.is_success())));

                                    let mut result_headers = OrdMap::default();
                                    for (k, v) in &headers {
                                        if let Ok(val) = v.to_str() {
                                            result_headers.insert(ValRef::new(Val::Str(k.as_str().into())), ValRef::new(Val::Str(val.into())));
                                        }
                                    }
                                    results.insert(ValRef::new(Val::Str("headers".into())), ValRef::new(Val::Map(result_headers)));
                                
                                    if let Some(ctype) = headers.get(CONTENT_TYPE) {
                                        if let Ok(val) = ctype.to_str() {
                                            results.insert(ValRef::new(Val::Str("content_type".into())), ValRef::new(Val::Str(val.into())));
                                        }
                                    }

                                    if let Ok(bytes) = response_body {
                                        results.insert(ValRef::new(Val::Str("bytes".into())), ValRef::new(Val::Blob(bytes)));
                                    }
                                },
                                _ => {},
                            }
                        },
                        Err(error) => {
                            let mut results = self.results.write();
                            match results.deref_mut() {
                                Val::Map(results) => {
                                    results.insert(ValRef::new(Val::Str("error".into())), ValRef::new(Val::Str(error.to_string().into())));
                                },
                                _ => {},
                            }
                        }
                    }
                }
                wake(&self.waker); // wake the process that is waiting
            }
        }

        #[cfg(not(feature = "tokio"))]
        {
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
                match client.execute(request) {
                    Ok(response) => {
                        let status = response.status();
                        let headers = response.headers().clone();
                        let response_body = response.bytes();

                        let mut results = self.results.write();
                        match results.deref_mut() {
                            Val::Map(results) => {
                                results.insert(ValRef::new(Val::Str("status".into())), ValRef::new(Val::Num(Num::Int(status.as_u16() as i64))));
                                results.insert(ValRef::new(Val::Str("ok".into())), ValRef::new(Val::Bool(status.is_success())));

                                let mut result_headers = OrdMap::default();
                                for (k, v) in &headers {
                                    if let Ok(val) = v.to_str() {
                                        result_headers.insert(ValRef::new(Val::Str(k.as_str().into())), ValRef::new(Val::Str(val.into())));
                                    }
                                }
                                results.insert(ValRef::new(Val::Str("headers".into())), ValRef::new(Val::Map(result_headers)));
                            
                                if let Some(ctype) = headers.get(CONTENT_TYPE) {
                                    if let Ok(val) = ctype.to_str() {
                                        results.insert(ValRef::new(Val::Str("content_type".into())), ValRef::new(Val::Str(val.into())));
                                    }
                                }

                                if let Ok(bytes) = response_body {
                                    results.insert(ValRef::new(Val::Str("bytes".into())), ValRef::new(Val::Blob(bytes)));
                                }
                            },
                            _ => {},
                        }
                    },
                    Err(error) => {
                        let mut results = self.results.write();
                        match results.deref_mut() {
                            Val::Map(results) => {
                                results.insert(ValRef::new(Val::Str("error".into())), ValRef::new(Val::Str(error.to_string().into())));
                            },
                            _ => {},
                        }
                    }
                }
            }
            wake(&self.waker); // wake the process that is waiting
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// HTTP instructions.
pub(self) enum HttpIns {
    /// Send an HTTP request, creating a map on the stack with results.
    SendRequest(WakeRef),
    /// Parse an HTTP response map into an object context, using the recieved content_type (or STOF/JSON by default..)
    ParseResponse,
    /// Extract a text response body.
    TextBody,
    /// Extract the size of the blob response body in bytes.
    BodySize,
    /// Extract a blob response body.
    BlobBody,
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
            // Http.fetch(..) -> map
            Self::SendRequest(waker) => {
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

                let map = ValRef::new(Val::Map(OrdMap::default()));
                let request = HTTPRequest {
                    url,
                    method,
                    headers,
                    bearer,
                    body,
                    timeout,
                    query,
                    results: map.clone(),
                    waker: waker.clone(),
                };
                env.stack.push(Variable::refval(map));
                request.send(env);
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
                                        graph.binary_import(&content_type, bytes.clone(), Some(context.clone()))?;
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

            // Http.text(response: map) -> str
            Self::TextBody => {
                if let Some(response) = env.stack.pop() {
                    match response.val.read().deref() {
                        Val::Map(response) => {
                            if let Some(body) = response.get(&ValRef::new(Val::Str("bytes".into()))) {
                                match body.read().deref() {
                                    Val::Blob(bytes) => {
                                        if let Ok(res) = str::from_utf8(&bytes) {
                                            env.stack.push(Variable::val(Val::Str(res.into())));
                                        } else {
                                            env.stack.push(Variable::val(Val::Null));
                                        }
                                    },
                                    _ => {
                                        return Err(Error::HttpSendError(format!("text body response map 'bytes' value must be a blob")));
                                    }
                                }
                            } else {
                                return Err(Error::HttpSendError(format!("text body must have a 'bytes' key-value blob body")));
                            }
                        },
                        _ => {
                            return Err(Error::HttpSendError(format!("text body response must be a map")))
                        }
                    }
                }
            },

            // Http.size(response: map) -> bytes
            Self::BodySize => {
                if let Some(response) = env.stack.pop() {
                    match response.val.read().deref() {
                        Val::Map(response) => {
                            if let Some(body) = response.get(&ValRef::new(Val::Str("bytes".into()))) {
                                match body.read().deref() {
                                    Val::Blob(bytes) => {
                                        env.stack.push(Variable::val(Val::Num(Num::Units(bytes.len() as f64, Units::Bytes))));
                                    },
                                    _ => {
                                        return Err(Error::HttpSendError(format!("blob body size response map 'bytes' value must be a blob")));
                                    }
                                }
                            } else {
                                return Err(Error::HttpSendError(format!("blob body size must have a 'bytes' key-value blob body")));
                            }
                        },
                        _ => {
                            return Err(Error::HttpSendError(format!("blob body size response must be a map")))
                        }
                    }
                }
            },

            // Http.blob(response: map) -> blob
            Self::BlobBody => {
                if let Some(response) = env.stack.pop() {
                    match response.val.read().deref() {
                        Val::Map(response) => {
                            if let Some(body) = response.get(&ValRef::new(Val::Str("bytes".into()))) {
                                match body.read().deref() {
                                    Val::Blob(bytes) => {
                                        env.stack.push(Variable::val(Val::Blob(bytes.clone())));
                                    },
                                    _ => {
                                        return Err(Error::HttpSendError(format!("blob body response map 'bytes' value must be a blob")));
                                    }
                                }
                            } else {
                                return Err(Error::HttpSendError(format!("blob body must have a 'bytes' key-value blob body")));
                            }
                        },
                        _ => {
                            return Err(Error::HttpSendError(format!("blob body response must be a map")))
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
