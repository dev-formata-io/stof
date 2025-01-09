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
use crate::{Library, SDataRef, SDoc, SFunc, SVal};


/// Function library.
#[derive(Default, Debug)]
pub struct FunctionLibrary;
impl FunctionLibrary {
    /// Call function operation.
    pub fn operate(&self, pid: &str, doc: &mut SDoc, name: &str, dref: &SDataRef, parameters: &mut Vec<SVal>) -> Result<SVal> {
        match name {
            // Get the name of this function.
            "name" => {
                let func: SFunc = dref.data(&doc.graph).unwrap().get_value().unwrap();
                Ok(SVal::String(func.name))
            },
            // Get the parameters on this function.
            "parameters" => {
                let func: SFunc = dref.data(&doc.graph).unwrap().get_value().unwrap();
                let mut params = Vec::new();
                for param in &func.params {
                    params.push(SVal::Tuple(vec![SVal::String(param.name.clone()), SVal::String(param.ptype.type_of())]));
                }
                Ok(SVal::Array(params))
            },
            // Get the return type of this function.
            "returnType" => {
                let func: SFunc = dref.data(&doc.graph).unwrap().get_value().unwrap();
                Ok(SVal::String(func.rtype.type_of()))
            },
            // Has the given attribute on this function?
            "hasAttribute" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("Function.hasAttribute(fn, attribute: str) requires an attribute parameter to look for"));
                }
                let func: SFunc = dref.data(&doc.graph).unwrap().get_value().unwrap();
                Ok(SVal::Bool(func.attributes.contains_key(&parameters[0].to_string())))
            },
            // Attributes on this function.
            "attributes" => {
                let func: SFunc = dref.data(&doc.graph).unwrap().get_value().unwrap();
                let mut attrs = Vec::new();
                for (key, value) in &func.attributes {
                    attrs.push(SVal::Tuple(vec![SVal::String(key.clone()), value.clone()]));
                }
                Ok(SVal::Array(attrs))
            },
            // One object that this function exists on.
            "object" => {
                let data = dref.data(&doc.graph).unwrap();
                for node in &data.nodes {
                    return Ok(SVal::Object(node.clone()));
                }
                Ok(SVal::Null)
            },
            // All objects that this function exists on.
            "objects" => {
                let data = dref.data(&doc.graph).unwrap();
                let mut objs = Vec::new();
                for node in &data.nodes {
                    objs.push(SVal::Object(node.clone()));
                }
                Ok(SVal::Array(objs))
            },
            // Call this function.
            "call" => {
                let func: SFunc = dref.data(&doc.graph).unwrap().get_value().unwrap();
                func.call(pid, doc, parameters.drain(..).collect(), true)
            },
            _ => {
                Err(anyhow!("Did not find the requested Function library function '{}'", name))
            }
        }
    }
}
impl Library for FunctionLibrary {
    /// Scope.
    fn scope(&self) -> String {
        "Function".to_string()     
    }
    
    /// Call into the Function library.
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
            match &parameters[0] {
                SVal::FnPtr(data) => {
                    return self.operate(pid, doc, name, data, &mut params);
                },
                SVal::Boxed(val) => {
                    let val = val.lock().unwrap();
                    let val = val.deref();
                    match val {
                        SVal::FnPtr(data) => {
                            return self.operate(pid, doc, name, data, &mut params);
                        },
                        _ => {
                            return Err(anyhow!("Function library requires the first parameter to be a fn (function)"));
                        }
                    }
                },
                _ => {
                    return Err(anyhow!("Function library requires the first parameter to be a fn (function)"));
                }
            }
        } else {
            return Err(anyhow!("Function library requires a 'fn' parameter to work with"));
        }
    }
}
