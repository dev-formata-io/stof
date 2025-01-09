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

use std::ops::DerefMut;
use anyhow::{anyhow, Result};
use crate::{SDoc, Library, SNum, SVal};


/// String library.
#[derive(Default, Debug)]
pub struct StringLibrary;
impl StringLibrary {
    /// Call string operation.
    pub fn operate(&self, _pid: &str, _doc: &mut SDoc, name: &str, val: &mut String, parameters: &mut Vec<SVal>) -> Result<SVal> {
        match name {
            // Get the length of this string.
            "len" => {
                Ok(SVal::Number(SNum::I64(val.len() as i64)))
            },
            // Get a char from this string at a specific index.
            "at" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("String.at(val, index) requires an index parameter"));
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
                    _ => {
                        Err(anyhow!("String.at(val, index) index must be a number value"))
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
                Err(anyhow!("String.startsWith(val, test: str) requires a string parameter to test with"))
            },
            // Test whether the string ends with a value.
            "endsWith" => {
                if parameters.len() == 1 {
                    let second = parameters[0].to_string();
                    return Ok(SVal::Bool(val.ends_with(&second)));
                }
                Err(anyhow!("String.endsWith(val, test: str) requires a string parameter to test with"))
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
                    return Err(anyhow!("String.contains(val, substr) requires a substring parameter to test with"));
                }
                Ok(SVal::Bool(val.contains(&parameters[0].to_string())))
            },
            // Index of a substring in this string. -1 if not found, index of first char otherwise.
            "indexOf" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("String.indexOf(val, substr) requires a substring parameter to find"))
                }
                if let Some(index) = val.find(&parameters[0].to_string()) {
                    return Ok(SVal::Number(SNum::I64(index as i64)));
                }
                Ok(SVal::Number(SNum::I64(-1 as i64)))
            },
            // Replace all instances of a substring in this string.
            "replace" => {
                if parameters.len() < 2 {
                    return Err(anyhow!("String.replace(val, from, to) requires 3 string parameters"));
                }
                let from = parameters[0].to_string();
                let to = parameters[1].to_string();
                Ok(SVal::String(val.replace(&from, &to)))
            },
            // Split this string at every occurrence of a substring.
            "split" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("String.split(val, substr) requires a substring to split this string with"));
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
                    return Err(anyhow!("String.substring(val, start, end) requires a starting index and optional end index"));
                }
                let start;
                match &parameters[0] {
                    SVal::Number(num) => {
                        start = num.int() as usize;
                    },
                    _ => {
                        return Err(anyhow!("String.substring(val, start, end) starting index must be a number value"));
                    }
                }
                if parameters.len() > 1 {
                    let end;
                    match &parameters[1] {
                        SVal::Number(num) => {
                            end = num.int() as usize;
                        },
                        _ => {
                            return Err(anyhow!("String.substring(val, start, end) ending index must be a number value"));
                        }
                    }
                    if let Some(slice) = val.get(start..end) {
                        return Ok(SVal::String(slice.to_string()));
                    }
                    return Err(anyhow!("String.substring(val, start, end) could not get substring at the requested range"));
                }
                if let Some(slice) = val.get(start..) {
                    Ok(SVal::String(slice.to_string()))
                } else {
                    Err(anyhow!("String.substring(val, start, end) could not get substring at the requested range"))
                }
            },
            _ => {
                Err(anyhow!("Did not find the requested String library function '{}'", name))
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
    fn call(&self, pid: &str, doc: &mut SDoc, name: &str, parameters: &mut Vec<SVal>) -> Result<SVal> {
        if parameters.len() > 0 {
            match name {
                "toString" => {
                    return Ok(SVal::String(parameters[0].print(doc)));
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
                            return Err(anyhow!("String library requires the first parameter to be a str"));
                        }
                    }
                },
                _ => {
                    return Err(anyhow!("String library requires the first parameter to be a str"));
                }
            }
        } else {
            return Err(anyhow!("String library requires a 'str' parameter to work with"));
        }
    }
}
