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

use anyhow::{anyhow, Result};
use crate::{SDoc, Library, SNum, SNumType, SType, SVal};
use super::Object;


/// String library.
#[derive(Default, Debug)]
pub struct StringLibrary;
impl Object for StringLibrary {}
impl Library for StringLibrary {
    /// Scope.
    fn scope(&self) -> String {
        "String".to_string()
    }
    
    /// Call into the String library.
    fn call(&mut self, doc: &mut SDoc, name: &str, parameters: &mut Vec<SVal>) -> Result<SVal> {
        if parameters.len() > 0 {
            let this = &parameters[0].cast(SType::String, doc)?;
            match this {
                SVal::String(val) => {
                    match name {
                        "len" => {
                            return Ok(SVal::Number(SNum::I64(val.len() as i64)));
                        },
                        "at" => {
                            if parameters.len() == 2 {
                                let second = &parameters[1].cast(SType::Number(SNumType::I64), doc)?;
                                match second {
                                    SVal::Number(nval) => {
                                        let index = nval.int();
                                        if index as usize >= val.len() || index < 0 {
                                            return Ok(SVal::Null);
                                        }
                                        let char = val.as_bytes()[index as usize] as char;
                                        return Ok(SVal::from(char));
                                    },
                                    _ => {}
                                }
                            }
                            return Err(anyhow!("String.at requires an index parameter"));
                        },
                        "first" => {
                            if val.len() < 1 {
                                return Ok(SVal::Null);
                            }
                            let char = val.as_bytes()[0] as char;
                            return Ok(SVal::from(char));
                        },
                        "last" => {
                            if val.len() < 1 {
                                return Ok(SVal::Null);
                            }
                            let char = val.as_bytes()[val.len() - 1] as char;
                            return Ok(SVal::from(char));
                        },
                        "startsWith" => {
                            if parameters.len() == 2 {
                                let second = &parameters[1].cast(SType::String, doc)?;
                                match second {
                                    SVal::String(second) => {
                                        return Ok(SVal::Bool(val.starts_with(second)));
                                    },
                                    _ => {}
                                }
                            }
                            return Err(anyhow!("String.startsWith requires a string parameter"));
                        },
                        "endsWith" => {
                            if parameters.len() == 2 {
                                let second = &parameters[1].cast(SType::String, doc)?;
                                match second {
                                    SVal::String(second) => {
                                        return Ok(SVal::Bool(val.ends_with(second)));
                                    },
                                    _ => {}
                                }
                            }
                            return Err(anyhow!("String.endsWith requires a string parameter"));
                        },
                        "push" => {
                            if parameters.len() == 2 {
                                let second = &parameters[1].cast(SType::String, doc)?;
                                match second {
                                    SVal::String(second) => {
                                        return Ok(SVal::String(format!("{}{}", val, second)));
                                    },
                                    _ => {}
                                }
                            }
                            return Err(anyhow!("String.push requires a string parameter"));
                        },
                        "concat" => {
                            if parameters.len() == 3 {
                                let second = &parameters[1].cast(SType::String, doc)?;
                                match second {
                                    SVal::String(second) => {
                                        let third = &parameters[2].cast(SType::String, doc)?;
                                        match third {
                                            SVal::String(third) => {
                                                return Ok(SVal::String(format!("{}{}{}", val, second, third)));
                                            },
                                            _ => {}
                                        }
                                    },
                                    _ => {}
                                }
                            }
                            return Err(anyhow!("String.concat requires a string separator and a string to push"));
                        },
                        "contains" => {
                            if parameters.len() == 2 {
                                let second = &parameters[1].cast(SType::String, doc)?;
                                match second {
                                    SVal::String(second) => {
                                        return Ok(SVal::Bool(val.contains(second)));
                                    },
                                    _ => {}
                                }
                            }
                            return Err(anyhow!("String.contains requires a string parameter"));
                        },
                        "indexOf" => {
                            if parameters.len() == 2 {
                                let second = &parameters[1].cast(SType::String, doc)?;
                                match second {
                                    SVal::String(second) => {
                                        if let Some(index) = val.find(second) {
                                            return Ok(SVal::Number(SNum::I64(index as i64)));
                                        }
                                        return Ok(SVal::Number(SNum::I64(-1 as i64)));
                                    },
                                    _ => {}
                                }
                            }
                            return Err(anyhow!("String.indexOf requires a string parameter"));
                        },
                        "replace" => {
                            if parameters.len() == 3 {
                                let second = &parameters[1].cast(SType::String, doc)?;
                                match second {
                                    SVal::String(second) => {
                                        let third = &parameters[2].cast(SType::String, doc)?;
                                        match third {
                                            SVal::String(third) => {
                                                return Ok(SVal::String(val.replace(second, third)));
                                            },
                                            _ => {}
                                        }
                                    },
                                    _ => {}
                                }
                            }
                            return Err(anyhow!("String.replace requires a string pattern and a string to replace"));
                        },
                        "split" => {
                            if parameters.len() == 2 {
                                let second = &parameters[1].cast(SType::String, doc)?;
                                match second {
                                    SVal::String(second) => {
                                        let vals = val.split(second).collect::<Vec<&str>>();
                                        
                                        let mut array = Vec::new();
                                        for v in vals { array.push(SVal::from(v)); }

                                        return Ok(SVal::Array(array));
                                    },
                                    _ => {}
                                }
                            }
                            return Err(anyhow!("String.split requires a string split parameter"));
                        },
                        "substring" => {
                            if parameters.len() == 2 {
                                let second = &parameters[1].cast(SType::Number(SNumType::I64), doc)?;
                                match second {
                                    SVal::Number(start) => {
                                        if let Some(slice) = val.get(start.int() as usize..) {
                                            return Ok(SVal::String(slice.to_string()));
                                        }
                                    },
                                    _ => {}
                                }
                            } else if parameters.len() == 3 {
                                let second = &parameters[1].cast(SType::Number(SNumType::I64), doc)?;
                                match second {
                                    SVal::Number(start) => {
                                        let third = &parameters[2].cast(SType::Number(SNumType::I64), doc)?;
                                        match third {
                                            SVal::Number(end) => {
                                                if let Some(slice) = val.get(start.int() as usize..end.int() as usize) {
                                                    return Ok(SVal::String(slice.to_string()));
                                                }
                                            },
                                            _ => {}
                                        }
                                    },
                                    _ => {}
                                }
                            }
                            return Err(anyhow!("String.substring requires a start index and an optional end index (up to, but not including)"));
                        },
                        "toUpper" => {
                            return Ok(SVal::String(val.to_uppercase()));
                        },
                        "toLower" => {
                            return Ok(SVal::String(val.to_lowercase()));
                        },
                        "trim" => {
                            return Ok(SVal::String(val.trim().to_string()));
                        },
                        "trimStart" => {
                            return Ok(SVal::String(val.trim_start().to_string()));
                        },
                        "trimEnd" => {
                            return Ok(SVal::String(val.trim_end().to_string()));
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        }
        if let Ok(val) = Self::object_call(doc, name, parameters) {
            return Ok(val);
        }
        Err(anyhow!("Failed to find a String library method"))
    }
}
