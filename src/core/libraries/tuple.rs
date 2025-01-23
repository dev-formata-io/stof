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

use std::ops::{Deref, DerefMut};
use crate::{lang::SError, Library, SDoc, SNum, SVal};


/// Tuple library.
#[derive(Default, Debug)]
pub struct TupleLibrary;
impl TupleLibrary {
    /// Call tuple operation.
    pub fn operate(&self, pid: &str, doc: &mut SDoc, name: &str, tup: &mut Vec<SVal>, parameters: &mut Vec<SVal>) -> Result<SVal, SError> {
        match name {
            // Get the length of this tuple.
            "len" => {
                Ok(SVal::Number(SNum::I64(tup.len() as i64)))
            },
            // Get a value from this tuple at a specific index.
            "at" => {
                if parameters.len() < 1 {
                    return Err(SError::tup(pid, &doc, "at", "invalid arguments - index not found"));
                }
                let index = parameters.pop().unwrap().unbox();
                match index {
                    SVal::Number(index) => {
                        let index = index.int() as usize;
                        if let Some(val) = tup.get(index) {
                            return Ok(val.clone());
                        }
                        Err(SError::tup(pid, &doc, "at", "index out of bounds"))
                    },
                    SVal::Boxed(val) => {
                        let val = val.lock().unwrap();
                        let val = val.deref();
                        match val {
                            SVal::Number(index) => {
                                let index = index.int() as usize;
                                if let Some(val) = tup.get(index) {
                                    return Ok(val.clone());
                                }
                                Err(SError::tup(pid, &doc, "at", "index out of bounds"))
                            },
                            _ => {
                                Err(SError::tup(pid, &doc, "at", "non-numerical index not supported"))
                            }
                        }
                    },
                    _ => {
                        Err(SError::tup(pid, &doc, "at", "non-numerical index not supported"))
                    }
                }
            },
            _ => {
                Err(SError::tup(pid, &doc, "NotFound", &format!("{} is not a function in the Tuple Library", name)))
            }
        }
    }
}
impl Library for TupleLibrary {
    /// Scope.
    fn scope(&self) -> String {
        "Tuple".to_string()
    }

    /// Call into the Tuple library.
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
                _ => {}
            }

            let mut params;
            if parameters.len() > 1 {
                params = parameters.drain(1..).collect();
            } else {
                params = Vec::new();
            }
            match &mut parameters[0] {
                SVal::Tuple(tup) => {
                    return self.operate(pid, doc, name, tup, &mut params);
                },
                SVal::Boxed(val) => {
                    let mut val = val.lock().unwrap();
                    let val = val.deref_mut();
                    match val {
                        SVal::Tuple(tup) => {
                            return self.operate(pid, doc, name, tup, &mut params);
                        },
                        _ => {
                            return Err(SError::tup(pid, &doc, "InvalidArgument", "tuple argument not found"));
                        }
                    }
                },
                _ => {
                    return Err(SError::tup(pid, &doc, "InvalidArgument", "tuple argument not found"));
                }
            }
        } else {
            return Err(SError::tup(pid, &doc, "InvalidArgument", "tuple argument not found"));
        }
    }
}
