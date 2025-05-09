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
use std::ops::{Deref, DerefMut};
use base64::{engine::general_purpose::{STANDARD, URL_SAFE}, Engine as _};
use crate::{lang::SError, Library, SDoc, SNum, SVal};


/// Blob library.
#[derive(Default, Debug)]
pub struct BlobLibrary;
impl BlobLibrary {
    /// Call blob operation.
    pub fn operate(&self, pid: &str, doc: &mut SDoc, name: &str, blob: &mut Vec<u8>, parameters: &mut Vec<SVal>) -> Result<SVal, SError> {
        match name {
            "len" |
            "size" => {
                Ok(SVal::Number(SNum::I64(blob.len() as i64)))
            },
            "at" => {
                if parameters.len() < 1 {
                    return Err(SError::blob(pid, &doc, "at", "index argument not found"));
                }
                match &parameters[0] {
                    SVal::Number(num) => {
                        let index = num.int() as usize;
                        if index < blob.len() {
                            return Ok(SVal::Number(SNum::I64(blob[index] as i64)));
                        }
                    },
                    SVal::Boxed(val) => {
                        let val = val.lock().unwrap();
                        let val = val.deref();
                        match val {
                            SVal::Number(num) => {
                                let index = num.int() as usize;
                                if index < blob.len() {
                                    return Ok(SVal::Number(SNum::I64(blob[index] as i64)));
                                }
                            },
                            _ => {}
                        }
                    },
                    _ => {}
                }
                Err(SError::blob(pid, &doc, "at", "index argument not found"))
            },
            "utf8" => {
                if let Ok(res) = str::from_utf8(blob.as_slice()) {
                    Ok(SVal::String(res.to_owned()))
                } else {
                    Err(SError::blob(pid, &doc, "utf8", "failed to decode blob into a utf-8 string"))
                }
            },
            "base64" => {
                let res = STANDARD.encode(blob);
                Ok(SVal::String(res))
            },
            "urlSafeBase64" => {
                let res = URL_SAFE.encode(blob);
                Ok(SVal::String(res))
            },
            _ => {
                Err(SError::blob(pid, &doc, "NotFound", &format!("{} is not a function in the Blob Library", name)))
            }
        }
    }
}
impl Library for BlobLibrary {
    /// Scope.
    fn scope(&self) -> String {
        "Blob".to_string()
    }
    
    /// Call into the Blob library.
    fn call(&self, pid: &str, doc: &mut SDoc, name: &str, parameters: &mut Vec<SVal>) -> Result<SVal, SError> {
        if parameters.len() > 0 {
            match name {
                "toString" => {
                    return Ok(SVal::String(parameters[0].print(doc)));
                },
                "or" => {
                    for param in parameters.drain(..) {
                        if !param.is_empty() {
                            return Ok(param);
                        }
                    }
                    return Ok(SVal::Null);
                },
                "fromUtf8" => {
                    let value = parameters.pop().unwrap().owned_to_string();
                    return Ok(SVal::Blob(str::as_bytes(&value).to_vec()));
                },
                "fromBase64" => {
                    let value = parameters.pop().unwrap().owned_to_string();
                    if let Ok(bytes) = STANDARD.decode(value) {
                        return Ok(SVal::Blob(bytes));
                    } else {
                        return Err(SError::blob(pid, &doc, "fromBase64", "failed to decode base64 standard string"));
                    }
                },
                "fromUrlSafeBase64" => {
                    let value = parameters.pop().unwrap().owned_to_string();
                    if let Ok(bytes) = URL_SAFE.decode(value) {
                        return Ok(SVal::Blob(bytes));
                    } else {
                        return Err(SError::blob(pid, &doc, "fromUrlSafeBase64", "failed to decode base64 url-safe string"));
                    }
                },
                _ => {}
            }

            let mut params;
            if parameters.len() > 1 {
                params = parameters.drain(1..).collect();
            } else {
                params = Vec::new();
            }
            match &mut parameters[0] {
                SVal::Blob(val) => {
                    return self.operate(pid, doc, name, val, &mut params);
                },
                SVal::Boxed(val) => {
                    let mut val = val.lock().unwrap();
                    let val = val.deref_mut();
                    match val {
                        SVal::Blob(val) => {
                            return self.operate(pid, doc, name, val, &mut params);
                        },
                        _ => {
                            return Err(SError::blob(pid, &doc, "InvalidArgument", "blob argument not found"));
                        }
                    }
                },
                _ => {
                    return Err(SError::blob(pid, &doc, "InvalidArgument", "blob argument not found"));
                }
            }
        } else {
            return Err(SError::blob(pid, &doc, "InvalidArgument", "blob argument not found"));
        }
    }
}
