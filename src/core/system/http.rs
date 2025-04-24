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

use std::{collections::BTreeMap, io::Read, ops::Deref, time::Duration};
use bytes::Bytes;
use crate::{lang::SError, Library, SDoc, SNodeRef, SNum, SUnits, SVal};
use ureq::Agent;


#[derive(Debug)]
pub struct HTTPLibrary {
    pub agent: Agent,
}
impl Default for HTTPLibrary {
    fn default() -> Self {
        Self {
            agent: ureq::AgentBuilder::new()
                .timeout_read(Duration::from_secs(5))
                .timeout_write(Duration::from_secs(5))
                .build(),
        }
    }
}
impl Library for HTTPLibrary {
    /// Scope of this library.
    /// This is how this library is invoked from Stof.
    /// Ex. `HTTP.get('https://example.com')`
    fn scope(&self) -> String {
        "HTTP".to_string()
    }

    /// Call an HTTP method in this library.
    /// Returns (status code: int, headers: map, body: blob).
    ///
    /// Supported functions:
    /// - HTTP.ok - check to see if a status code is OK (in range 200-299)
    /// - HTTP.clientError - check to see if a status code is 400-499
    /// - HTTP.serverError - check to see if a status code is 500-599
    /// - HTTP.contentType - get content type from a header map
    ///
    /// - HTTP.send - must take an additional parameter at first (one of below methods).
    /// - HTTP.get
    /// - HTTP.head
    /// - HTTP.patch
    /// - HTTP.post
    /// - HTTP.put
    /// - HTTP.delete
    ///
    /// Parameters (in order) for each call:
    /// - url: str                       - The HTTP request path (REQUIRED)
    /// - headers: vec[(str, str)] | map - The request headers (OPTIONAL)
    /// - body: str | blob               - The request body (OPTIONAL)
    /// - timeout: float | units         - The overall timeout for the request (OPTIONAL) (default 5 seconds - use time units as needed)
    /// - response_obj: obj              - A response object to parse the response into via doc.header_import with the content type (OPTIONAL)
    ///
    /// Basic GET request: `HTTP.get('https://example.com')`
    ///
    /// POST request with a body: `HTTP.post('https://example.com', 'this is a string body to send')`
    ///
    /// POST request json body and a timeout: `HTTP.post('https://example.com', map(('content-type', 'application/json')), stringify(self, 'json'), 10s)`
    fn call(&self, pid: &str, doc: &mut SDoc, name: &str, parameters: &mut Vec<SVal>) -> Result<SVal, SError> {
        // Helper functions first
        let mut method = name.to_string();
        match name {
            "send" => {
                if parameters.len() > 0 {
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
                                    return Err(SError::custom(pid, &doc, "HTTPError", "cannot check a non-numerical status code"));
                                }
                            }
                        },
                        _ => {
                            return Err(SError::custom(pid, &doc, "HTTPError", "cannot check a non-numerical status code"));
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
                                    return Err(SError::custom(pid, &doc, "HTTPError", "cannot check a non-numerical status code"));
                                }
                            }
                        },
                        _ => {
                            return Err(SError::custom(pid, &doc, "HTTPError", "cannot check a non-numerical status code"));
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
                                    return Err(SError::custom(pid, &doc, "HTTPError", "cannot check a non-numerical status code"));
                                }
                            }
                        },
                        _ => {
                            return Err(SError::custom(pid, &doc, "HTTPError", "cannot check a non-numerical status code"));
                        }
                    }
                }
            },
            "contentType" => {
                if parameters.len() > 0 {
                    let headers = parameters.pop().unwrap();
                    match headers {
                        SVal::Map(map) => {
                            if let Some(ctype) = map.get(&"Content-Type".into()) {
                                return Ok(ctype.clone());
                            } else if let Some(ctype) = map.get(&"content-type".into()) {
                                return Ok(ctype.clone());
                            }
                            return Ok(SVal::Null); // no content type header found
                        },
                        SVal::Boxed(val) => {
                            let val = val.lock().unwrap();
                            let val = val.deref();
                            match val {
                                SVal::Map(map) => {
                                    if let Some(ctype) = map.get(&"Content-Type".into()) {
                                        return Ok(ctype.clone());
                                    } else if let Some(ctype) = map.get(&"content-type".into()) {
                                        return Ok(ctype.clone());
                                    }
                                    return Ok(SVal::Null); // no content type header found
                                },
                                _ => {}
                            }
                        },
                        _ => {}
                    }
                    return Err(SError::custom(pid, &doc, "HTTPError", "cannot check a non map set of headers"));
                }
            },
            _ => {}
        }
        
        let url;
        if parameters.len() > 0 {
            match &parameters[0] {
                SVal::String(val) => {
                    url = val.clone();
                },
                SVal::Boxed(val) => {
                    let val = val.lock().unwrap();
                    let val = val.deref();
                    match val {
                        SVal::String(val) => {
                            url = val.clone();
                        },
                        _ => {
                            return Err(SError::custom(pid, &doc, "HTTPError", "url must be a string"));
                        }
                    }
                },
                _ => {
                    return Err(SError::custom(pid, &doc, "HTTPError", "url must be a string"));
                }
            }
        } else {
            return Err(SError::custom(pid, &doc, "HTTPError", "must provide a URL as the first parameter when calling into the HTTP library"));
        }

        let mut request;
        match method.to_lowercase().as_str() {
            "get" => request = self.agent.get(&url),
            "head" => request = self.agent.head(&url),
            "patch" => request = self.agent.patch(&url),
            "post" => request = self.agent.post(&url),
            "put" => request = self.agent.put(&url),
            "delete" => request = self.agent.delete(&url),
            _ => {
                return Err(SError::custom(pid, &doc, "HTTPError", &format!("unrecognized HTTP library function: {}", method)));
            }
        }

        let mut headers = Vec::new();
        let mut str_body: Option<String> = None;
        let mut blob_body: Option<Vec<u8>> = None;
        let mut timeout = Duration::from_secs(5);
        let mut response_obj: Option<SNodeRef> = None;
        if parameters.len() > 1 {
            match &parameters[1] {
                SVal::Array(vals) => {
                    for val in vals {
                        match val {
                            SVal::Tuple(vals) => {
                                if vals.len() == 2 {
                                    headers.push((vals[0].to_string(), vals[1].to_string()));
                                }
                            },
                            _ => {}
                        }
                    }
                },
                SVal::Map(map) => {
                    for (k, v) in map {
                        headers.push((k.to_string(), v.to_string()));
                    }
                },
                SVal::String(body) => {
                    str_body = Some(body.clone());
                },
                SVal::Blob(body) => {
                    blob_body = Some(body.clone());
                },
                SVal::Number(num) => {
                    let seconds = num.float_with_units(SUnits::Seconds);
                    timeout = Duration::from_secs(seconds as u64);
                },
                SVal::Object(nref) => {
                    response_obj = Some(nref.clone());
                },
                SVal::Boxed(val) => {
                    let val = val.lock().unwrap();
                    let val = val.deref();
                    match val {
                        SVal::Array(vals) => {
                            for val in vals {
                                match val {
                                    SVal::Tuple(vals) => {
                                        if vals.len() == 2 {
                                            headers.push((vals[0].to_string(), vals[1].to_string()));
                                        }
                                    },
                                    _ => {}
                                }
                            }
                        },
                        SVal::Map(map) => {
                            for (k, v) in map {
                                headers.push((k.to_string(), v.to_string()));
                            }
                        },
                        SVal::String(body) => {
                            str_body = Some(body.clone());
                        },
                        SVal::Blob(body) => {
                            blob_body = Some(body.clone());
                        },
                        SVal::Number(num) => {
                            let seconds = num.float_with_units(SUnits::Seconds);
                            timeout = Duration::from_secs(seconds as u64);
                        },
                        SVal::Object(nref) => {
                            response_obj = Some(nref.clone());
                        },
                        _ => {
                            return Err(SError::custom(pid, &doc, "HTTPError", "second parameter for an HTTP request must be either headers (vec), a body (str | blob), a timeout (float | units), or response object (obj)"));
                        }
                    }
                },
                _ => {
                    return Err(SError::custom(pid, &doc, "HTTPError", "second parameter for an HTTP request must be either headers (vec), a body (str | blob), a timeout (float | units), or response object (obj)"));
                }
            }
        }
        if parameters.len() > 2 {
            match &parameters[2] {
                SVal::String(body) => {
                    str_body = Some(body.clone());
                },
                SVal::Blob(body) => {
                    blob_body = Some(body.clone());
                },
                SVal::Number(num) => {
                    let seconds = num.float_with_units(SUnits::Seconds);
                    timeout = Duration::from_secs(seconds as u64);
                },
                SVal::Object(nref) => {
                    response_obj = Some(nref.clone());
                },
                SVal::Boxed(val) => {
                    let val = val.lock().unwrap();
                    let val = val.deref();
                    match val {
                        SVal::String(body) => {
                            str_body = Some(body.clone());
                        },
                        SVal::Blob(body) => {
                            blob_body = Some(body.clone());
                        },
                        SVal::Number(num) => {
                            let seconds = num.float_with_units(SUnits::Seconds);
                            timeout = Duration::from_secs(seconds as u64);
                        },
                        SVal::Object(nref) => {
                            response_obj = Some(nref.clone());
                        },
                        _ => {
                            return Err(SError::custom(pid, &doc, "HTTPError", "third parameter for an HTTP request must be either a body (str | blob), a timeout (float | units), or a response object (obj)"));
                        }
                    }
                },
                _ => {
                    return Err(SError::custom(pid, &doc, "HTTPError", "third parameter for an HTTP request must be either a body (str | blob), a timeout (float | units), or a response object (obj)"));
                }
            }
        }
        if parameters.len() > 3 {
            match &parameters[3] {
                SVal::Number(num) => {
                    let seconds = num.float_with_units(SUnits::Seconds);
                    timeout = Duration::from_secs(seconds as u64);
                },
                SVal::Object(nref) => {
                    response_obj = Some(nref.clone());
                },
                SVal::Boxed(val) => {
                    let val = val.lock().unwrap();
                    let val = val.deref();
                    match val {
                        SVal::Number(num) => {
                            let seconds = num.float_with_units(SUnits::Seconds);
                            timeout = Duration::from_secs(seconds as u64);
                        },
                        SVal::Object(nref) => {
                            response_obj = Some(nref.clone());
                        },
                        _ => {
                            return Err(SError::custom(pid, &doc, "HTTPError", "fourth parameter for an HTTP request must be a timeout (float | units) or a response object (obj)"));
                        }
                    }
                },
                _ => {
                    return Err(SError::custom(pid, &doc, "HTTPError", "fourth parameter for an HTTP request must be a timeout (float | units) or a response object (obj)"));
                }
            }
        }
        if parameters.len() > 4 {
            match &parameters[4] {
                SVal::Object(nref) => {
                    response_obj = Some(nref.clone());
                },
                SVal::Boxed(val) => {
                    let val = val.lock().unwrap();
                    let val = val.deref();
                    match val {
                        SVal::Object(nref) => {
                            response_obj = Some(nref.clone());
                        },
                        _ => {
                            return Err(SError::custom(pid, &doc, "HTTPError", "fifth parameter for an HTTP request must be a response object (obj)"));
                        }
                    }
                },
                _ => {
                    return Err(SError::custom(pid, &doc, "HTTPError", "fifth parameter for an HTTP request must be a response object (obj)"));
                }
            }
        }

        // Set headers and timeout
        for header in headers {
            request = request.set(header.0.as_str(), header.1.as_str());
        }
        request = request.timeout(timeout);
        
        // Send with body or call without
        let response_res;
        if let Some(body) = str_body {
            response_res = request.send_string(&body);
        } else if let Some(body) = blob_body {
            response_res = request.send_bytes(&body);
        } else {
            response_res = request.call();
        }
        let response;
        match response_res {
            Ok(res) => response = res,
            Err(error) => return Err(SError::custom(pid, &doc, "HTTPError", &format!("error sending request: {}", error.to_string()))),
        }

        // Get the status of the response
        let status: u16 = response.status();

        // Get content type and headers from the response
        let content_type = response.content_type().to_owned();
        let mut response_headers = BTreeMap::new();
        for name in response.headers_names() {
            if let Some(value) = response.header(&name) {
                response_headers.insert(SVal::String(name), SVal::String(value.to_owned()));
            }
        }

        // Read response body into a blob
        let mut buf: Vec<u8> = vec![];
        let res = response.into_reader()
            .take(((10 * 1_024 * 1_024) + 1) as u64)
            .read_to_end(&mut buf);
        if res.is_err() {
            return Err(SError::custom(pid, &doc, "HTTPError", &format!("error reading response into buffer: {}", res.err().unwrap().to_string())));
        }
        if buf.len() > (10 * 1_024 * 1_024) {
            return Err(SError::custom(pid, &doc, "HTTPError", "response is too large to be read into a buffer"));
        }

        // Import the response into a response object if one was provided
        if let Some(response_obj) = response_obj {
            let mut bytes = Bytes::from(buf.clone());
            let as_name = response_obj.path(&doc.graph);
            doc.header_import(pid, &content_type, &content_type, &mut bytes, &as_name)?;
        }

        // Return the response status, headers, and body
        return Ok(SVal::Tuple(vec![SVal::Number(SNum::I64(status as i64)), SVal::Map(response_headers), SVal::Blob(buf)]));
    }
}
