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

use std::{collections::BTreeMap, ops::Deref};
use crate::{lang::SError, Library, SDataRef, SDoc, SFunc, SVal};


/// Function library.
#[derive(Default, Debug)]
pub struct FunctionLibrary;
impl FunctionLibrary {
    /// Call function operation.
    pub fn operate(&self, pid: &str, doc: &mut SDoc, name: &str, dref: &SDataRef, parameters: &mut Vec<SVal>) -> Result<SVal, SError> {
        match name {
            // Get the name of this function.
            "name" => {
                let func: &SFunc = dref.data(&doc.graph).unwrap().get_data().unwrap();
                Ok(SVal::String(func.name.clone()))
            },
            // Get the parameters on this function.
            "parameters" => {
                let func: &SFunc = dref.data(&doc.graph).unwrap().get_data().unwrap();
                let mut params = Vec::new();
                for param in &func.params {
                    params.push(SVal::Tuple(vec![SVal::String(param.name.clone()), SVal::String(param.ptype.type_of())]));
                }
                Ok(SVal::Array(params))
            },
            // Get the return type of this function.
            "returnType" => {
                let func: &SFunc = dref.data(&doc.graph).unwrap().get_data().unwrap();
                Ok(SVal::String(func.rtype.type_of()))
            },
            // Has the given attribute on this function?
            "hasAttribute" => {
                if parameters.len() < 1 {
                    return Err(SError::func(pid, &doc, "hasAttribute", "attribute str argument not found"));
                }
                let func: &SFunc = dref.data(&doc.graph).unwrap().get_data().unwrap();
                Ok(SVal::Bool(func.attributes.contains_key(&parameters[0].to_string())))
            },
            // Attributes on this function.
            "attributes" => {
                let func: &SFunc = dref.data(&doc.graph).unwrap().get_data().unwrap();
                let mut attrs = BTreeMap::new();
                for (key, value) in &func.attributes {
                    attrs.insert(SVal::String(key.clone()), value.clone());
                }
                Ok(SVal::Map(attrs))
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
                SFunc::call(dref, pid, doc, parameters.drain(..).collect(), true)
            },
            // Call this function with an array of parameters as an argument.
            "expandCall" => {
                if parameters.len() < 1 {
                    return Err(SError::func(pid, &doc, "expandCall", "parameters value not found"));
                }
                let param = parameters.pop().unwrap();
                match param {
                    SVal::Array(vals) => {
                        SFunc::call(dref, pid, doc, vals, true)
                    },
                    SVal::Tuple(vals) => {
                        SFunc::call(dref, pid, doc, vals, true)
                    },
                    SVal::Set(set) => {
                        SFunc::call(dref, pid, doc, set.into_iter().collect(), true)
                    },
                    _ => {
                        Err(SError::func(pid, &doc, "expandCall", "must provide an (array, tuple, or set) of parameters to use"))
                    }
                }
            },
            _ => {
                Err(SError::func(pid, &doc, "NotFound", &format!("{} is not a function in the Function Library", name)))
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
                            return Err(SError::func(pid, &doc, "InvalidArgument", "function (fn) argument not found"));
                        }
                    }
                },
                _ => {
                    return Err(SError::func(pid, &doc, "InvalidArgument", "function (fn) argument not found"));
                }
            }
        } else {
            return Err(SError::func(pid, &doc, "InvalidArgument", "function (fn) argument not found"));
        }
    }
}
