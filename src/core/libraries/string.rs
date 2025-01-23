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


/// String library.
#[derive(Default, Debug)]
pub struct StringLibrary;
impl StringLibrary {
    /// Call string operation.
    pub fn operate(&self, pid: &str, doc: &mut SDoc, name: &str, val: &mut String, parameters: &mut Vec<SVal>) -> Result<SVal, SError> {
        match name {
            // Get the length of this string.
            "len" => {
                Ok(SVal::Number(SNum::I64(val.len() as i64)))
            },
            // Get a char from this string at a specific index.
            "at" => {
                if parameters.len() < 1 {
                    return Err(SError::string(pid, &doc, "at", "invalid arguments - index not found"));
                }
                let index = parameters.pop().unwrap().unbox();
                match index {
                    SVal::Number(index) => {
                        let index = index.int() as usize;
                        if index >= val.len() {
                            return Ok(SVal::Null);
                        }
                        let char = val.as_bytes()[index] as char;
                        Ok(SVal::from(char))
                    },
                    SVal::Boxed(bval) => {
                        let bval = bval.lock().unwrap();
                        let bval = bval.deref();
                        match bval {
                            SVal::Number(index) => {
                                let index = index.int() as usize;
                                if index >= val.len() {
                                    return Ok(SVal::Null);
                                }
                                let char = val.as_bytes()[index] as char;
                                Ok(SVal::from(char))
                            },
                            _ => {
                                Err(SError::string(pid, &doc, "at", "invalid arguments - index must be numerical"))
                            }
                        }
                    },
                    _ => {
                        Err(SError::string(pid, &doc, "at", "invalid arguments - index must be numerical"))
                    }
                }
            },
            // Get the first char in this string.
            "first" => {
                if val.len() < 1 {
                    return Ok(SVal::Null);
                }
                let char = val.as_bytes()[0] as char;
                Ok(SVal::from(char))
            },
            // Get the last char in this string.
            "last" => {
                if val.len() < 1 {
                    return Ok(SVal::Null);
                }
                let char = val.as_bytes()[val.len() - 1] as char;
                Ok(SVal::from(char))
            },
            // Test whether the string starts with a value.
            "startsWith" => {
                if parameters.len() == 1 {
                    let second = parameters[0].to_string();
                    return Ok(SVal::Bool(val.starts_with(&second)));
                }
                Err(SError::string(pid, &doc, "startsWith", "invalid arguments - string value to test startsWith not found"))
            },
            // Test whether the string ends with a value.
            "endsWith" => {
                if parameters.len() == 1 {
                    let second = parameters[0].to_string();
                    return Ok(SVal::Bool(val.ends_with(&second)));
                }
                Err(SError::string(pid, &doc, "endsWith", "invalid arguments - string value to test endsWith not found"))
            },
            // Push values to this string.
            "push" => {
                for param in parameters.drain(..) {
                    val.push_str(&param.to_string());
                }
                Ok(SVal::Void)
            },
            // Does this string contain a substring?
            "contains" => {
                if parameters.len() < 1 {
                    return Err(SError::string(pid, &doc, "contains", "invalid arguments - contains string not found"));
                }
                Ok(SVal::Bool(val.contains(&parameters[0].to_string())))
            },
            // Index of a substring in this string. -1 if not found, index of first char otherwise.
            "indexOf" => {
                if parameters.len() < 1 {
                    return Err(SError::string(pid, &doc, "indexOf", "invalid arguments - substring to search for not found"));
                }
                if let Some(index) = val.find(&parameters[0].to_string()) {
                    return Ok(SVal::Number(SNum::I64(index as i64)));
                }
                Ok(SVal::Number(SNum::I64(-1 as i64)))
            },
            // Replace all instances of a substring in this string.
            "replace" => {
                if parameters.len() < 2 {
                    return Err(SError::string(pid, &doc, "replace", "invalid arguments - string replace takes a from string value and a to string value"));
                }
                let from = parameters[0].to_string();
                let to = parameters[1].to_string();
                Ok(SVal::String(val.replace(&from, &to)))
            },
            // Split this string at every occurrence of a substring.
            "split" => {
                if parameters.len() < 1 {
                    return Err(SError::string(pid, &doc, "split", "invalid arguments - substring to split with not found"));
                }
                let vals = val.split(&parameters[0].to_string()).collect::<Vec<&str>>();
                let mut array = Vec::new();
                for v in vals { array.push(SVal::from(v)); }
                Ok(SVal::Array(array))
            },
            // Transform this string to all uppercase.
            "toUpper" => {
                Ok(SVal::String(val.to_uppercase()))
            },
            // Transform this string to all lowercase.
            "toLower" => {
                Ok(SVal::String(val.to_lowercase()))
            },
            // Trim the whitespace off of the front and end of this string.
            "trim" => {
                Ok(SVal::String(val.trim().to_string()))
            },
            // Trim the whitespace off of the front of this string.
            "trimStart" => {
                Ok(SVal::String(val.trim_start().to_string()))
            },
            // Trim the whitespace off of the end of this string.
            "trimEnd" => {
                Ok(SVal::String(val.trim_end().to_string()))
            },
            // Get a substring of this string.
            "substring" => {
                if parameters.len() < 1 {
                    return Err(SError::string(pid, &doc, "substring", "invalid arguments - start (and optional end) not found"));
                }
                let start;
                match &parameters[0] {
                    SVal::Number(num) => {
                        start = num.int() as usize;
                    },
                    _ => {
                        return Err(SError::string(pid, &doc, "substring", "non-numerical ranges are not supported"));
                    }
                }
                if parameters.len() > 1 {
                    let end;
                    match &parameters[1] {
                        SVal::Number(num) => {
                            end = num.int() as usize;
                        },
                        _ => {
                            return Err(SError::string(pid, &doc, "substring", "non-numerical ranges are not supported"));
                        }
                    }
                    if let Some(slice) = val.get(start..end) {
                        return Ok(SVal::String(slice.to_string()));
                    }
                    return Err(SError::string(pid, &doc, "substring", "cannot get substring at the requested range"));
                }
                if let Some(slice) = val.get(start..) {
                    Ok(SVal::String(slice.to_string()))
                } else {
                    Err(SError::string(pid, &doc, "substring", "cannot get substring at the requested range"))
                }
            },
            _ => {
                Err(SError::string(pid, &doc, "NotFound", &format!("{} is not a function in the String Library", name)))
            }
        }
    }
}
impl Library for StringLibrary {
    /// Scope.
    fn scope(&self) -> String {
        "String".to_string()
    }
    
    /// Call into the String library.
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
                SVal::String(val) => {
                    return self.operate(pid, doc, name, val, &mut params);
                },
                SVal::Boxed(val) => {
                    let mut val = val.lock().unwrap();
                    let val = val.deref_mut();
                    match val {
                        SVal::String(val) => {
                            return self.operate(pid, doc, name, val, &mut params);
                        },
                        _ => {
                            return Err(SError::string(pid, &doc, "InvalidArgument", "string argument not found"));
                        }
                    }
                },
                _ => {
                    return Err(SError::string(pid, &doc, "InvalidArgument", "string argument not found"));
                }
            }
        } else {
            return Err(SError::string(pid, &doc, "InvalidArgument", "string argument not found"));
        }
    }
}
