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

use std::ops::Deref;
use anyhow::{anyhow, Result};
use crate::{Library, SDoc, SVal};


/// Empty (Null & Void) library.
#[derive(Default, Debug)]
pub struct EmptyLibrary;
impl EmptyLibrary {
    /// Call empty operation.
    pub fn operate(&self, _pid: &str, _doc: &mut SDoc, name: &str, _parameters: &mut Vec<SVal>) -> Result<SVal> {
        match name {
            _ => {
                Err(anyhow!("Did not find the requested Empty library function '{}'", name))
            }
        }
    }
}
impl Library for EmptyLibrary {
    /// Scope.
    fn scope(&self) -> String {
        "Empty".to_string()
    }
    
    /// Call into the Empty library.
    fn call(&self, pid: &str, doc: &mut SDoc, name: &str, parameters: &mut Vec<SVal>) -> Result<SVal> {
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
                SVal::Void |
                SVal::Null => {
                    return self.operate(pid, doc, name, &mut params);
                },
                SVal::Boxed(val) => {
                    let val = val.lock().unwrap();
                    let val = val.deref();
                    match val {
                        SVal::Void |
                        SVal::Null => {
                            return self.operate(pid, doc, name, &mut params);
                        },
                        _ => {
                            return Err(anyhow!("Empty library requires the first parameter to be a empty"));
                        }
                    }
                },
                _ => {
                    return Err(anyhow!("Empty library requires the first parameter to be a empty"));
                }
            }
        } else {
            return Err(anyhow!("Empty library requires a empty parameter to work with"));
        }
    }
}
