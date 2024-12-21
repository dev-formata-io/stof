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

use std::{collections::HashSet, ops::DerefMut};
use anyhow::{anyhow, Result};
use crate::{Data, IntoDataRef, Library, SDoc, SField, SFunc, SType, SVal};


/// Object trait for calls.
pub trait Object {
    /// Call into the Object library.
    fn object_call(doc: &mut SDoc, name: &str, parameters: &mut Vec<SVal>) -> Result<SVal> {
        if parameters.len() < 1 { return Err(anyhow!("Must provide an object parameter")); }

        // Make sure the object lib works for all refs
        match &parameters[0] {
            SVal::Ref(rf) => {
                let mut val = rf.write().unwrap();
                let v = val.deref_mut();
                match v {
                    SVal::Object(nref) => {
                        let mut params = vec![SVal::Object(nref.clone())];
                        for i in 1..parameters.len() {
                            params.push(parameters[i].clone());
                        }
                        return Self::object_call(doc, name, &mut params);
                    },
                    _ => {}
                }
            },
            _ => {}
        }

        match name {
            "toString" => Self::to_string(doc, parameters),
            "len" => {
                if parameters.len() > 0 {
                    match &parameters[0] {
                        SVal::Object(nref) => {
                            let fields = SField::fields(&doc.graph, nref);
                            return Ok(SVal::Number((fields.len() as i32).into()));
                        },
                        _ => {}
                    }
                }
                Err(anyhow!("Cannot determin the length of the object (number of fields)"))
            },
            "at" => {
                if parameters.len() == 2 {
                    // Just one value returned - looks for fields, then functions
                    match &parameters[0] {
                        SVal::Object(nref) => {
                            match &parameters[1] {
                                SVal::String(index) => {
                                    if let Some(field) = SField::field(&doc.graph, &index, '.', Some(nref)) {
                                        return Ok(field.value);
                                    } else if let Some(func) = SFunc::func(&doc.graph, &index, '.', Some(nref)) {
                                        return Ok(SVal::FnPtr(func.data_ref()));
                                    }
                                    return Ok(SVal::Null); // Not found
                                },
                                SVal::Number(val) => {
                                    let mut fields = SField::fields(&doc.graph, nref);
                                    let index = val.int() as usize;
                                    if index < fields.len() {
                                        let field = fields.remove(index);
                                        let value = field.value;
                                        let key = SVal::String(field.name);
                                        return Ok(SVal::Tuple(vec![key, value]));
                                    }
                                },
                                _ => {}
                            }
                            let second = &parameters[1].cast(SType::String, doc)?;
                            match second {
                                SVal::String(index) => {
                                    if let Some(field) = SField::field(&doc.graph, &index, '.', Some(nref)) {
                                        return Ok(field.value);
                                    } else if let Some(func) = SFunc::func(&doc.graph, &index, '.', Some(nref)) {
                                        return Ok(SVal::FnPtr(func.data_ref()));
                                    }
                                    return Ok(SVal::Null); // Not found
                                },
                                _ => {}
                            }
                        },
                        _ => {}
                    }
                } else if parameters.len() > 1 {
                    // Array of items that we got
                    match &parameters[0] {
                        SVal::Object(nref) => {
                            let mut array = Vec::new();
                            for i in 1..parameters.len() {
                                let second = &parameters[i].cast(SType::String, doc)?;
                                match second {
                                    SVal::String(index) => {
                                        if let Some(field) = SField::field(&doc.graph, &index, '.', Some(nref)) {
                                            array.push(field.value);
                                        } else if let Some(func) = SFunc::func(&doc.graph, &index, '.', Some(nref)) {
                                            array.push(SVal::FnPtr(func.data_ref()));
                                        }
                                    },
                                    _ => {}
                                }
                            }
                            return Ok(SVal::Array(array));
                        },
                        _ => {}
                    }
                }
                Err(anyhow!("Object.at(obj, index, ...) requires one object parameter and at least one index parameter"))
            },
            "reference" => {
                // Create a reference on this object to a field or function
                if parameters.len() == 2 {
                    match &parameters[0] {
                        SVal::Object(nref) => {
                            let field_path = parameters[1].to_string();
                            if let Some(mut field) = SField::field(&doc.graph, &field_path, '.', Some(&nref)) {
                                Self::object_call(doc, "remove", &mut vec![SVal::Object(nref.clone()), SVal::String(field.name.clone())])?;
                                field.attach(&nref, &mut doc.graph);
                                return Ok(SVal::Bool(true));
                            } else if let Some(mut field) = SField::field(&doc.graph, &field_path, '.', None) {
                                Self::object_call(doc, "remove", &mut vec![SVal::Object(nref.clone()), SVal::String(field.name.clone())])?;
                                field.attach(&nref, &mut doc.graph);
                                return Ok(SVal::Bool(true));
                            } else if let Some(mut func) = SFunc::func(&doc.graph, &field_path, '.', Some(&nref)) {
                                func.attach(&nref, &mut doc.graph);
                                return Ok(SVal::Bool(true));
                            } else if let Some(mut func) = SFunc::func(&doc.graph, &field_path, '.', None) {
                                func.attach(&nref, &mut doc.graph);
                                return Ok(SVal::Bool(true));
                            }
                            return Ok(SVal::Bool(false));
                        },
                        _ => {}
                    }
                } else if parameters.len() == 3 {
                    match &parameters[0] {
                        SVal::Object(destination) => {
                            match &parameters[1] {
                                SVal::Object(context) => {
                                    let field_path = parameters[2].to_string();
                                    if let Some(mut field) = SField::field(&doc.graph, &field_path, '.', Some(&context)) {
                                        Self::object_call(doc, "remove", &mut vec![SVal::Object(destination.clone()), SVal::String(field.name.clone())])?;
                                        field.attach(&destination, &mut doc.graph);
                                        return Ok(SVal::Bool(true));
                                    } else if let Some(mut func) = SFunc::func(&doc.graph, &field_path, '.', Some(&context)) {
                                        func.attach(&destination, &mut doc.graph);
                                        return Ok(SVal::Bool(true));
                                    }
                                    return Ok(SVal::Bool(false));
                                },
                                _ => {}
                            }
                        },
                        _ => {}
                    }
                }
                Err(anyhow!("Object.reference(obj, 'location') requires one object parameter and one path parameter to a field or function"))
            },
            "fields" => {
                // Return an array of tuples with field name and value
                if parameters.len() == 1 {
                    match &parameters[0] {
                        SVal::Object(nref) => {
                            let fields = SField::fields(&doc.graph, nref);
                            let mut array = Vec::new();
                            for field in fields {
                                let value = field.value;
                                let key = SVal::String(field.name);
                                array.push(SVal::Tuple(vec![key, value]));
                            }
                            return Ok(SVal::Array(array));
                        },
                        _ => {}
                    }
                }
                Err(anyhow!("Object.fields(obj) requires one object parameter"))
            },
            "funcs" |
            "functions" => {
                // Return an array of tuples with field name and value
                if parameters.len() == 1 {
                    match &parameters[0] {
                        SVal::Object(nref) => {
                            let funcs = SFunc::funcs(&doc.graph, nref);
                            let mut array = Vec::new();
                            for func in funcs {
                                let value = SVal::FnPtr(func.id.into());
                                let key = SVal::String(func.name);
                                array.push(SVal::Tuple(vec![key, value]));
                            }
                            return Ok(SVal::Array(array));
                        },
                        _ => {}
                    }
                }
                Err(anyhow!("Object.functions(obj) requires one object parameter"))
            },
            "keys" => {
                // Return an array of tuples with field name and value
                if parameters.len() == 1 {
                    match &parameters[0] {
                        SVal::Object(nref) => {
                            let fields = SField::fields(&doc.graph, nref);
                            let mut array = Vec::new();
                            for field in fields {
                                array.push(SVal::String(field.name));
                            }
                            return Ok(SVal::Array(array));
                        },
                        _ => {}
                    }
                }
                Err(anyhow!("Object.keys(obj) requires one object parameter"))
            },
            "values" => {
                // Return an array of tuples with field name and value
                if parameters.len() == 1 {
                    match &parameters[0] {
                        SVal::Object(nref) => {
                            let fields = SField::fields(&doc.graph, nref);
                            let mut array = Vec::new();
                            for field in fields {
                                array.push(field.value);
                            }
                            return Ok(SVal::Array(array));
                        },
                        _ => {}
                    }
                }
                Err(anyhow!("Object.values(obj) requires one object parameter"))
            },
            "set" => {
                if parameters.len() == 3 {
                    match &parameters[0] {
                        SVal::Object(nref) => {
                            let name = parameters[1].cast(SType::String, doc)?;
                            match name {
                                SVal::String(val) => {
                                    // Check for an existing field at this location
                                    if let Some(mut field) = SField::field(&doc.graph, &val, '.', Some(nref)) {
                                        field.value = parameters[2].clone();
                                        field.set(&mut doc.graph);
                                        return Ok(parameters[2].clone());
                                    }

                                    // val is a dot separated path!
                                    let mut path = val.split('.').collect::<Vec<&str>>();
                                    let name = path.pop().unwrap().to_string();

                                    // Ensure the path exists if we need to add objects
                                    let mut fref = nref.clone();
                                    if path.len() > 0 {
                                        fref = doc.graph.ensure_nodes(&path.join("/"), '/', true, Some(nref.clone()));
                                    }

                                    // Create the field on fref
                                    let value = parameters[2].clone();
                                    let mut field = SField::new(&name, value);
                                    field.attach(&fref, &mut doc.graph);

                                    return Ok(parameters[2].clone());
                                },
                                _ => {}
                            };
                        },
                        _ => {}
                    }
                }
                Err(anyhow!("Object.set(obj) requires one object parameter, string field name to set, and a value to set"))
            },
            "remove" => { // returns array of values removed
                if parameters.len() > 1 {
                    match &parameters[0] {
                        SVal::Object(nref) => {
                            let mut field_names = HashSet::new();
                            for i in 1..parameters.len() {
                                let value = parameters[i].cast(SType::String, doc)?;
                                match value {
                                    SVal::String(val) => field_names.insert(val),
                                    _ => false
                                };
                            }

                            let fields = SField::fields(&doc.graph, nref);
                            let mut array = Vec::new();
                            for field in fields {
                                if field_names.contains(&field.name) {
                                    // Remove field from the graph!
                                    field.remove(&mut doc.graph, Some(nref));
                                    // Remove object if object TODO: array objects (same with drop)
                                    match &field.value {
                                        SVal::Object(nref) => {
                                            doc.graph.remove_node(nref);
                                        },
                                        _ => {}
                                    }

                                    let value = field.value;
                                    let key = SVal::String(field.name);
                                    array.push(SVal::Tuple(vec![key, value]));
                                }
                            }
                            return Ok(SVal::Array(array));
                        },
                        _ => {}
                    }
                }
                Err(anyhow!("Object.remove(obj) requires one object parameter and string field names to remove"))
            },
            "name" => {
                if parameters.len() == 1 {
                    match &parameters[0] {
                        SVal::Object(nref) => {
                            if let Some(node) = nref.node(&doc.graph) {
                                return Ok(SVal::String(node.name.clone()));
                            }
                        },
                        _ => {}
                    }
                }
                Err(anyhow!("Object.name(obj) requires one object parameter"))
            },
            "parent" => {
                if parameters.len() == 1 {
                    match &parameters[0] {
                        SVal::Object(nref) => {
                            if let Some(node) = nref.node(&doc.graph) {
                                if let Some(parent) = &node.parent {
                                    return Ok(SVal::Object(parent.clone()));
                                }
                            }
                            return Ok(SVal::Null);
                        },
                        _ => {}
                    }
                }
                Err(anyhow!("Object.parent(obj) requires one object parameter"))
            },
            "children" => {
                if parameters.len() == 1 {
                    match &parameters[0] {
                        SVal::Object(nref) => {
                            if let Some(node) = nref.node(&doc.graph) {
                                let mut children = Vec::new();
                                for child in &node.children {
                                    children.push(SVal::Object(child.clone()));
                                }
                                return Ok(SVal::Array(children));
                            }
                            return Ok(SVal::Array(vec![]));
                        },
                        _ => {}
                    }
                }
                Err(anyhow!("Object.children(obj) requires one object parameter"))
            },
            "typename" => {
                if parameters.len() == 1 {
                    let typename = parameters[0].type_name(&doc.graph);
                    return Ok(SVal::String(typename));
                }
                Err(anyhow!("Object.typename(obj) requires one object parameter"))
            },
            "typestack" => {
                if parameters.len() == 1 {
                    let typestack = parameters[0].type_stack(&doc.graph);
                    return Ok(SVal::Array(typestack.into_iter().map(|x| SVal::String(x)).collect()));
                }
                Err(anyhow!("Object.typestack(obj) requires one object parameter"))
            },
            "instanceOf" => {
                if parameters.len() == 2 {
                    let iof = parameters[0].instance_of(&doc.graph, &parameters[1].to_string());
                    return Ok(SVal::Bool(iof));
                }
                Err(anyhow!("Object.instanceOf(obj, type) requires one object parameter and one typename string"))
            },
            _ => Err(anyhow!("No Object implementation"))
        }
    }

    /// To string.
    fn to_string(doc: &mut SDoc, parameters: &mut Vec<SVal>) -> Result<SVal> {
        if parameters.len() > 0 {
            let value = parameters[0].print(doc);
            return Ok(SVal::String(value));
        }
        Err(anyhow!("Failed to find a value to convert to a string"))
    }
}
#[derive(Default, Debug)]
pub struct ObjectLibrary;
impl Object for ObjectLibrary {}
impl Library for ObjectLibrary {
    fn scope(&self) -> String {
        "Object".into()
    }
    fn call(&mut self, doc: &mut SDoc, name: &str, parameters: &mut Vec<SVal>) -> Result<SVal> {
        Self::object_call(doc, name, parameters)
    }
}
