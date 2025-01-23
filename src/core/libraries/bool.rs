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
use crate::{lang::SError, Library, SDoc, SVal};


/// Bool library.
#[derive(Default, Debug)]
pub struct BoolLibrary;
impl BoolLibrary {
    /// Call bool operation.
    pub fn operate(&self, pid: &str, doc: &mut SDoc, name: &str, _bool: &mut bool, _parameters: &mut Vec<SVal>) -> Result<SVal, SError> {
        match name {
            _ => {
                Err(SError::bool(pid, &doc, "NotFound", &format!("{} is not a function in the Bool Library", name)))
            }
        }
    }
}
impl Library for BoolLibrary {
    /// Scope.
    fn scope(&self) -> String {
        "Bool".to_string()
    }
    
    /// Call into the Bool library.
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
                SVal::Bool(val) => {
                    return self.operate(pid, doc, name, val, &mut params);
                },
                SVal::Boxed(val) => {
                    let mut val = val.lock().unwrap();
                    let val = val.deref_mut();
                    match val {
                        SVal::Bool(val) => {
                            return self.operate(pid, doc, name, val, &mut params);
                        },
                        _ => {
                            return Err(SError::bool(pid, &doc, "InvalidArgument", "bool argument not found"));
                        }
                    }
                },
                _ => {
                    return Err(SError::bool(pid, &doc, "InvalidArgument", "bool argument not found"));
                }
            }
        } else {
            return Err(SError::bool(pid, &doc, "InvalidArgument", "bool argument not found"));
        }
    }
}
