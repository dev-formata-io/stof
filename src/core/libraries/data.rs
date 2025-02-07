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
use crate::{lang::SError, Library, SDataRef, SDoc, SField, SFunc, SVal};


/// Data library.
#[derive(Default, Debug)]
pub struct DataLibrary;
impl DataLibrary {
    /// Call data operation.
    pub fn operate(&self, pid: &str, doc: &mut SDoc, name: &str, data: &mut SDataRef, parameters: &mut Vec<SVal>) -> Result<SVal, SError> {
        match name {
            "exists" => {
                Ok(SVal::Bool(data.exists(&doc.graph)))
            },
            "objects" => {
                let mut objects = Vec::new();
                for node in data.nodes(&doc.graph) {
                    objects.push(SVal::Object(node));
                }
                Ok(SVal::Array(objects))
            },
            "id" => {
                Ok(SVal::String(data.id.clone()))
            },
            "drop" => {
                let mut from = None;
                if parameters.len() > 0 {
                    match &parameters[0] {
                        SVal::Object(nref) => {
                            from = Some(nref.clone());
                        },
                        SVal::Boxed(val) => {
                            let val = val.lock().unwrap();
                            let val = val.deref();
                            match val {
                                SVal::Object(nref) => {
                                    from = Some(nref.clone());
                                },
                                _ => {
                                    return Err(SError::data(pid, &doc, "drop", "cannot drop from anything other than an object"));
                                }
                            }
                        },
                        _ => {
                            return Err(SError::data(pid, &doc, "drop", "cannot drop from anything other than an object"));
                        }
                    }
                }
                Ok(SVal::Bool(doc.graph.remove_data(data.clone(), from.as_ref())))
            },
            "attach" => {
                if parameters.len() < 1 {
                    return Err(SError::data(pid, &doc, "attach", "attach must have an object argument to attach this data to"));
                }
                match &parameters[0] {
                    SVal::Object(nref) => {
                        Ok(SVal::Bool(doc.graph.put_data_ref(nref, data.clone())))
                    },
                    SVal::Boxed(val) => {
                        let val = val.lock().unwrap();
                        let val = val.deref();
                        match val {
                            SVal::Object(nref) => {
                                Ok(SVal::Bool(doc.graph.put_data_ref(nref, data.clone())))
                            },
                            _ => {
                                Err(SError::data(pid, &doc, "attach", "attach must have an object argument to attach this data to"))
                            }
                        }
                    },
                    _ => {
                        Err(SError::data(pid, &doc, "attach", "attach must have an object argument to attach this data to"))
                    }
                }
            },
            _ => {
                Err(SError::data(pid, &doc, "NotFound", &format!("{} is not a function in the Data Library", name)))
            }
        }
    }
}
impl Library for DataLibrary {
    /// Scope.
    fn scope(&self) -> String {
        "Data".to_string()
    }
    
    /// Call into the Data library.
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
                // create a new opaque data pointer by ID - doesn't have to exist
                "fromId" => {
                    return Ok(SVal::Data(SDataRef::new(&parameters[0].to_string())))
                },
                // create a new opaque data pointer from a field or function
                "from" => {
                    let id = parameters[0].to_string();

                    let mut context = None;
                    if id.starts_with("self") || id.starts_with("super") {
                        context = doc.self_ptr(pid);
                    }
                    let mut context_path = id.clone();
                    {
                        let mut path: Vec<&str> = id.split('.').collect();
                        if path.len() > 1 {
                            if let Some(symbol) = doc.get_symbol(pid, path.remove(0)) {
                                match symbol.var() {
                                    SVal::Object(nref) => {
                                        context = Some(nref.clone());
                                        context_path = path.join(".");
                                    },
                                    SVal::Boxed(val) => {
                                        let val = val.lock().unwrap();
                                        let val = val.deref();
                                        match val {
                                            SVal::Object(nref) => {
                                                context = Some(nref.clone());
                                                context_path = path.join(".");
                                            },
                                            _ => {}
                                        }
                                    },
                                    _ => {}
                                }
                            }
                        }
                    }

                    if let Some(field_ref) = SField::field_ref(&doc.graph, &context_path, '.', context.as_ref()) {
                        if doc.perms.can_write_field(&doc.graph, &field_ref, doc.self_ptr(pid).as_ref()) {
                            return Ok(SVal::Data(field_ref));
                        }
                        return Ok(SVal::Null);
                    }
                    if let Some(func_ref) = SFunc::func_ref(&doc.graph, &context_path, '.', context.as_ref()) {
                        if doc.perms.can_write_func(&doc.graph, &func_ref, doc.self_ptr(pid).as_ref()) {
                            return Ok(SVal::Data(func_ref));
                        }
                        return Ok(SVal::Null);
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
                SVal::Data(val) => {
                    return self.operate(pid, doc, name, val, &mut params);
                },
                SVal::Boxed(val) => {
                    let mut val = val.lock().unwrap();
                    let val = val.deref_mut();
                    match val {
                        SVal::Data(val) => {
                            return self.operate(pid, doc, name, val, &mut params);
                        },
                        _ => {
                            return Err(SError::data(pid, &doc, "InvalidArgument", "data argument not found"));
                        }
                    }
                },
                _ => {
                    return Err(SError::data(pid, &doc, "InvalidArgument", "data argument not found"));
                }
            }
        } else {
            return Err(SError::data(pid, &doc, "InvalidArgument", "data argument not found"));
        }
    }
}
