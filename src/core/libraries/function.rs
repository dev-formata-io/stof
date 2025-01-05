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
use crate::{SDoc, SFunc, Library, SVal};
use super::Object;


/// Function library.
#[derive(Default, Debug)]
pub struct FunctionLibrary;
impl Object for FunctionLibrary {}
impl Library for FunctionLibrary {
    /// Scope.
    fn scope(&self) -> String {
        "Function".to_string()     
    }
    
    /// Call into the Function library.
    fn call(&self, pid: &str, doc: &mut SDoc, name: &str, parameters: &mut Vec<SVal>) -> Result<SVal> {
        if parameters.len() > 0 {
            match name {
                "name" => {
                    match &parameters[0] {
                        SVal::FnPtr(dref) => {
                            let func: SFunc = dref.data(&doc.graph).unwrap().get_value().unwrap();
                            return Ok(SVal::String(func.name));
                        },
                        _ => return Err(anyhow!("Must provide a function pointer value when using the Function library"))
                    }
                },
                "parameters" => {
                    match &parameters[0] {
                        SVal::FnPtr(dref) => {
                            let func: SFunc = dref.data(&doc.graph).unwrap().get_value().unwrap();
                            let mut params = Vec::new();
                            for param in &func.params {
                                params.push(SVal::Tuple(vec![SVal::String(param.name.clone()), SVal::String(param.ptype.type_of())]));
                            }
                            return Ok(SVal::Array(params));
                        },
                        _ => return Err(anyhow!("Must provide a function pointer value when using the Function library"))
                    }
                },
                "returnType" => {
                    match &parameters[0] {
                        SVal::FnPtr(dref) => {
                            let func: SFunc = dref.data(&doc.graph).unwrap().get_value().unwrap();
                            return Ok(SVal::String(func.rtype.type_of()));
                        },
                        _ => return Err(anyhow!("Must provide a function pointer value when using the Function library"))
                    }
                },
                "attributes" => {
                    match &parameters[0] {
                        SVal::FnPtr(dref) => {
                            let func: SFunc = dref.data(&doc.graph).unwrap().get_value().unwrap();
                            let mut attrs = Vec::new();
                            for (key, value) in &func.attributes {
                                attrs.push(SVal::Tuple(vec![SVal::String(key.clone()), value.clone()]));
                            }
                            return Ok(SVal::Array(attrs));
                        },
                        _ => return Err(anyhow!("Must provide a function pointer value when using the Function library"))
                    }
                },
                "object" => {
                    match &parameters[0] {
                        SVal::FnPtr(dref) => {
                            let data = dref.data(&doc.graph).unwrap();
                            for node in &data.nodes {
                                return Ok(SVal::Object(node.clone()));
                            }
                        },
                        _ => return Err(anyhow!("Must provide a function pointer value when using the Function library"))
                    }
                },
                "objects" => {
                    match &parameters[0] {
                        SVal::FnPtr(dref) => {
                            let data = dref.data(&doc.graph).unwrap();
                            let mut objs = Vec::new();
                            for node in &data.nodes {
                                objs.push(SVal::Object(node.clone()));
                            }
                            return Ok(SVal::Array(objs));
                        },
                        _ => return Err(anyhow!("Must provide a function pointer value when using the Function library"))
                    }
                },
                "call" => {
                    let mut values = Vec::new();
                    if parameters.len() > 1 {
                        for i in 1..parameters.len() {
                            values.push(parameters[i].clone());
                        }
                    }
                    match &parameters[0] {
                        SVal::FnPtr(dref) => {
                            let func: SFunc = dref.data(&doc.graph).unwrap().get_value().unwrap();
                            return func.call(pid, doc, values, true);
                        },
                        _ => return Err(anyhow!("Must provide a function pointer to call using the Function library"))
                    }
                },
                _ => {}
            }
        }

        // try object scope
        if let Ok(val) = Self::object_call(pid, doc, name, parameters) {
            return Ok(val);
        }
        Err(anyhow!("Failed to find a Function library method."))
    }
}
