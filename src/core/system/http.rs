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
use std::{collections::{BTreeMap, HashMap}, ops::Deref, str::FromStr, time::Duration};
use crate::{lang::SError, Library, SDoc, SUnits, SVal};
use bytes::Bytes;
use reqwest::{blocking, header::{HeaderMap, HeaderName, CONTENT_TYPE}, Client, Method};

#[cfg(feature = "async")]
use tokio::runtime::Handle;


pub struct HttpLibrary {
    pub blocking_client: Option<blocking::Client>,

    /// Async client calls "block_on", so make sure you're running Stof accordingly
    pub async_client: Option<Client>,
}
impl Default for HttpLibrary {
    fn default() -> Self {
        #[cfg(not(feature = "async"))]
        return Self {
            async_client: None,
            blocking_client: Some(blocking::Client::new()),
        };

        #[cfg(feature = "async")]
        return Self {
            async_client: Some(Client::new()),
            blocking_client: None,
        };
    }
}
impl HttpLibrary {
    /// Make an HTTP request.
    /// Does not alter the graph/doc in any way.
    ///
    /// Response is a map that will translate into a Stof value:
    ///     Error:
    ///         "error" - the error encountered by the request
    ///     Ok:
    ///         "status"  - the status code (integer) for the response
    ///         "ok"      - boolean value indicating whether it was a success or not
    ///         "headers" - response header map for the request
    ///         "mime"    - string "Content-Type" response header
    ///         "text"    - string response body (if able to parse into a string)
    ///         "bytes"   - blob response body
    pub fn request(&self, method: Method, url: String, headers: HeaderMap, body: Option<Bytes>, query: Option<HashMap<String, String>>, timeout: Option<Duration>, bearer: Option<String>, basic: Option<(String, Option<String>)>) -> BTreeMap<SVal, SVal> {
        // Just because the feature might be enabled, doesn't mean we are able to use it...
        #[cfg(feature = "async")]
        let is_async = Handle::try_current().is_ok();
        #[cfg(not(feature = "async"))]
        let is_async = false;
        
        if is_async {
            #[cfg(feature = "async")]
            if let Some(client) = &self.async_client {
                let mut builder = client.request(method, url);
                builder = builder.headers(headers);
                if let Some(basic) = basic {
                    builder = builder.basic_auth(basic.0, basic.1);
                }
                if let Some(bearer) = bearer {
                    builder = builder.bearer_auth(bearer);
                }
                if let Some(body) = body {
                    builder = builder.body(body);
                }
                if let Some(timeout) = timeout {
                    builder = builder.timeout(timeout);
                }
                if let Some(query) = query {
                    builder = builder.query(&query);
                }
                if let Ok(request) = builder.build() {
                    let current = Handle::current();
                    return current.block_on(async move {
                        let mut result = BTreeMap::new();
                        let response = client.execute(request).await;
                        match response {
                            Ok(response) => {
                                result.insert("status".into(), SVal::from(response.status().as_u16() as i32));
                                result.insert("ok".into(), response.status().is_success().into());

                                let mut headers: BTreeMap<SVal, SVal> = BTreeMap::new();
                                for (k, v) in response.headers() {
                                    if let Ok(val) = v.to_str() {
                                        headers.insert(k.as_str().into(), val.into());
                                    }
                                }
                                result.insert("headers".into(), SVal::Map(headers));


                                if let Some(ctype) = response.headers().get(CONTENT_TYPE) {
                                    if let Ok(val) = ctype.to_str() {
                                        // Content Type as MIME type
                                        result.insert("mime".into(), val.into());
                                    }
                                }

                                if let Ok(bytes) = response.bytes().await {
                                    // try parsing utf8 text out of the response
                                    if let Ok(res) = str::from_utf8(&bytes) {
                                        result.insert("text".into(), res.into());
                                    }
                                    result.insert("bytes".into(), SVal::Blob(bytes.to_vec()));
                                }
                            },
                            Err(error) => {
                                result.insert("error".into(), error.to_string().into());
                            },
                        }
                        result
                    });
                }
            }
        } else {
            let mut builder;
            if let Some(client) = &self.blocking_client {
                builder = client.request(method, url);
            } else {
                let new_blocking = blocking::Client::new();
                builder = new_blocking.request(method, url);
            }

            builder = builder.headers(headers);
            if let Some(basic) = basic {
                builder = builder.basic_auth(basic.0, basic.1);
            }
            if let Some(bearer) = bearer {
                builder = builder.bearer_auth(bearer);
            }
            if let Some(body) = body {
                builder = builder.body(body);
            }
            if let Some(timeout) = timeout {
                builder = builder.timeout(timeout);
            }
            if let Some(query) = query {
                builder = builder.query(&query);
            }

            let mut result = BTreeMap::new();
            let response = builder.send();
            match response {
                Ok(response) => {
                    result.insert("status".into(), SVal::from(response.status().as_u16() as i32));
                    result.insert("ok".into(), response.status().is_success().into());

                    let mut headers: BTreeMap<SVal, SVal> = BTreeMap::new();
                    for (k, v) in response.headers() {
                        if let Ok(val) = v.to_str() {
                            headers.insert(k.as_str().into(), val.into());
                        }
                    }
                    result.insert("headers".into(), SVal::Map(headers));


                    if let Some(ctype) = response.headers().get(CONTENT_TYPE) {
                        if let Ok(val) = ctype.to_str() {
                            // Content Type as MIME type
                            result.insert("mime".into(), val.into());
                        }
                    }

                    if let Ok(bytes) = response.bytes() {
                        // try parsing utf8 text out of the response
                        if let Ok(res) = str::from_utf8(&bytes) {
                            result.insert("text".into(), res.into());
                        }
                        result.insert("bytes".into(), SVal::Blob(bytes.to_vec()));
                    }
                },
                Err(error) => {
                    result.insert("error".into(), error.to_string().into());
                },
            }
            return result;
        }
        let mut result = BTreeMap::new();
        result.insert("error".into(), "http client not found".into());
        result
    }
}
impl Library for HttpLibrary {
    fn scope(&self) -> String {
        "Http".to_owned()
    }

    /// Call an HTTP method.
    ///
    /// Examples:
    /// Http.send('post', 'https://www.example.com', map(('header', 'value')), 10s, parse_obj);
    /// let res: map = Http.get('https://example.com');
    /// Http.get('https://example.com', ('bearer-auth', '0a345lbikdhj'), ('header', 'value'), ('q', 'query', 'value'));
    fn call(&self, pid: &str, doc: &mut SDoc, name: &str, parameters: &mut Vec<SVal>) -> Result<SVal, SError> {
        let mut method = name.to_string();
        match name {
            "call" |
            "execute" |
            "send" => {
                if parameters.len() > 0 && parameters[0].is_string() {
                    let name = parameters.remove(0);
                    method = name.owned_to_string();
                }
            },
            "ok" => {
                if parameters.len() > 0 {
                    let code = parameters.pop().unwrap();
                    match code {
                        SVal::Number(num) => {
                            let code = num.int();
                            return Ok(SVal::Bool(code >= 200 && code < 300));
                        },
                        SVal::Boxed(val) => {
                            let val = val.lock().unwrap();
                            let val = val.deref();
                            match val {
                                SVal::Number(num) => {
                                    let code = num.int();
                                    return Ok(SVal::Bool(code >= 200 && code < 300));
                                },
                                _ => {
                                    return Err(SError::custom(pid, &doc, "HttpError", "cannot check a non-numerical status code"));
                                }
                            }
                        },
                        _ => {
                            return Err(SError::custom(pid, &doc, "HttpError", "cannot check a non-numerical status code"));
                        }
                    }
                }
            },
            "clientError" => {
                if parameters.len() > 0 {
                    let code = parameters.pop().unwrap();
                    match code {
                        SVal::Number(num) => {
                            let code = num.int();
                            return Ok(SVal::Bool(code >= 400 && code < 500));
                        },
                        SVal::Boxed(val) => {
                            let val = val.lock().unwrap();
                            let val = val.deref();
                            match val {
                                SVal::Number(num) => {
                                    let code = num.int();
                                    return Ok(SVal::Bool(code >= 400 && code < 500));
                                },
                                _ => {
                                    return Err(SError::custom(pid, &doc, "HttpError", "cannot check a non-numerical status code"));
                                }
                            }
                        },
                        _ => {
                            return Err(SError::custom(pid, &doc, "HttpError", "cannot check a non-numerical status code"));
                        }
                    }
                }
            },
            "serverError" => {
                if parameters.len() > 0 {
                    let code = parameters.pop().unwrap();
                    match code {
                        SVal::Number(num) => {
                            let code = num.int();
                            return Ok(SVal::Bool(code >= 500 && code < 600));
                        },
                        SVal::Boxed(val) => {
                            let val = val.lock().unwrap();
                            let val = val.deref();
                            match val {
                                SVal::Number(num) => {
                                    let code = num.int();
                                    return Ok(SVal::Bool(code >= 500 && code < 600));
                                },
                                _ => {
                                    return Err(SError::custom(pid, &doc, "HttpError", "cannot check a non-numerical status code"));
                                }
                            }
                        },
                        _ => {
                            return Err(SError::custom(pid, &doc, "HttpError", "cannot check a non-numerical status code"));
                        }
                    }
                }
            },
            _ => {}
        }

        let mut http_method = Method::GET;
        match method.to_lowercase().as_str() {
            "head" => http_method = Method::HEAD,
            "patch" => http_method = Method::PATCH,
            "post" => http_method = Method::POST,
            "put" => http_method = Method::PUT,
            "delete" => http_method = Method::DELETE,
            "connect" => http_method = Method::CONNECT,
            "trace" => http_method = Method::TRACE,
            "options" => http_method = Method::OPTIONS,
            _ => {}
        }

        let mut url = None;
        let mut headers = HeaderMap::new();
        let mut inserted_header_map = false;
        let mut query: Option<HashMap<String, String>> = None;
        let mut body = None;
        let mut timeout = None;
        let mut bearer = None;
        let mut basic = None;
        let mut parse_obj = None;
        for param in parameters.drain(..) {
            match param {
                SVal::String(val) => {
                    if url.is_none() {
                        url = Some(val);
                    } else {
                        body = Some(Bytes::from(val));
                    }
                },
                SVal::Map(map) => {
                    if !inserted_header_map {
                        inserted_header_map = true;
                        for (k, v) in map {
                            let key = k.owned_to_string();
                            if let Ok(name) = HeaderName::from_str(&key) {
                                headers.insert(name, v.owned_to_string().parse().unwrap());
                            }
                        }
                    } else {
                        if let Some(query) = &mut query {
                            for (k, v) in map {
                                query.insert(k.owned_to_string(), v.owned_to_string());
                            }
                        } else {
                            let mut query_map = HashMap::new();
                            for (k, v) in map {
                                query_map.insert(k.owned_to_string(), v.owned_to_string());
                            }
                            query = Some(query_map);
                        }
                    }
                },
                SVal::Blob(bytes) => body = Some(Bytes::from(bytes)),
                SVal::Number(num) => {
                    let seconds = num.float_with_units(SUnits::Seconds);
                    timeout = Some(Duration::from_secs(seconds as u64));
                },
                SVal::Object(nref) => {
                    parse_obj = Some(nref);
                },
                SVal::Tuple(mut vals) => {
                    if vals.len() == 2 {
                        // assumed to be a header value
                        let value = vals.pop().unwrap().owned_to_string();
                        let key = vals.pop().unwrap().owned_to_string();
                        if key.to_lowercase() == "bearer-auth" {
                            bearer = Some(value);
                        } else if key.to_lowercase() == "basic-auth" {
                            basic = Some((value, None));
                        } else if let Ok(name) = HeaderName::from_str(&key) {
                            headers.insert(name, value.parse().unwrap());
                        }
                    } else if vals.len() == 3 {
                        let value = vals.pop().unwrap().owned_to_string();
                        let key = vals.pop().unwrap().owned_to_string();
                        let tt = vals.pop().unwrap().owned_to_string().to_lowercase();
                        if tt == "header" { // header
                            if let Ok(name) = HeaderName::from_str(&key) {
                                headers.insert(name, value.parse().unwrap());
                            }
                        } else if tt == "basic-auth" {
                            basic = Some((key, Some(value)));
                        } else { // query
                            if let Some(query) = &mut query {
                                query.insert(key, value);
                            } else {
                                let mut query_map = HashMap::new();
                                query_map.insert(key, value);
                                query = Some(query_map);
                            }
                        }
                    }
                },
                SVal::Boxed(val) => {
                    let val = val.lock().unwrap();
                    let val = val.deref();
                    match val {
                        SVal::String(val) => {
                            if url.is_none() {
                                url = Some(val.clone());
                            } else {
                                body = Some(Bytes::from(val.clone()));
                            }
                        },
                        SVal::Map(map) => {
                            if !inserted_header_map {
                                inserted_header_map = true;
                                for (k, v) in map {
                                    let key = k.to_string();
                                    if let Ok(name) = HeaderName::from_str(&key) {
                                        headers.insert(name, v.to_string().parse().unwrap());
                                    }
                                }
                            } else {
                                if let Some(query) = &mut query {
                                    for (k, v) in map {
                                        query.insert(k.to_string(), v.to_string());
                                    }
                                } else {
                                    let mut query_map = HashMap::new();
                                    for (k, v) in map {
                                        query_map.insert(k.to_string(), v.to_string());
                                    }
                                    query = Some(query_map);
                                }
                            }
                        },
                        SVal::Blob(bytes) => body = Some(Bytes::from(bytes.clone())),
                        SVal::Number(num) => {
                            let seconds = num.float_with_units(SUnits::Seconds);
                            timeout = Some(Duration::from_secs(seconds as u64));
                        },
                        SVal::Object(nref) => {
                            parse_obj = Some(nref.clone());
                        },
                        SVal::Tuple(vals) => {
                            if vals.len() == 2 {
                                // assumed to be a header value
                                let value = vals[1].to_string();
                                let key = vals[0].to_string();
                                if key.to_lowercase() == "bearer-auth" {
                                    bearer = Some(value);
                                } else if key.to_lowercase() == "basic-auth" {
                                    basic = Some((value, None));
                                } else if let Ok(name) = HeaderName::from_str(&key) {
                                    headers.insert(name, value.parse().unwrap());
                                }
                            } else if vals.len() == 3 {
                                let value = vals[2].to_string();
                                let key = vals[1].to_string();
                                let tt = vals[0].to_string().to_lowercase();
                                if tt == "header" { // header
                                    if let Ok(name) = HeaderName::from_str(&key) {
                                        headers.insert(name, value.parse().unwrap());
                                    }
                                } else if tt == "basic-auth" {
                                    basic = Some((key, Some(value)));
                                } else { // query
                                    if let Some(query) = &mut query {
                                        query.insert(key, value);
                                    } else {
                                        let mut query_map = HashMap::new();
                                        query_map.insert(key, value);
                                        query = Some(query_map);
                                    }
                                }
                            }
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        }

        if let Some(url) = url {
            let result = self.request(http_method, url, headers, body, query, timeout, bearer, basic);
            if let Some(res_obj) = parse_obj {
                if let Some(mime) = result.get(&SVal::String("mime".into())) {
                    let content_type = mime.to_string();
                    if let Some(bytes) = result.get(&SVal::String("bytes".into())) {
                        match bytes {
                            SVal::Blob(bytes) => {
                                let mut bytes = Bytes::from(bytes.clone());
                                let as_name = res_obj.path(&doc.graph);
                                doc.header_import(pid, &content_type, &content_type, &mut bytes, &as_name)?;
                            },
                            _ => {}
                        }
                    }
                }
            }
            return Ok(SVal::Map(result));
        }
        Err(SError::custom(pid, &doc, "HttpError", "URL not found - cannot make request"))
    }
}
