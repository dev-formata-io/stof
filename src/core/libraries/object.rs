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

use std::{collections::HashSet, ops::Deref};
use anyhow::{anyhow, Result};
use crate::{Data, IntoDataRef, Library, SDoc, SField, SFunc, SNodeRef, SVal};


#[derive(Default, Debug)]
pub struct ObjectLibrary;
impl ObjectLibrary {
    /// Call object operation.
    pub fn operate(&self, pid: &str, doc: &mut SDoc, name: &str, obj: &SNodeRef, parameters: &mut Vec<SVal>) -> Result<SVal> {
        match name {
            "len" => {
                let fields = SField::fields(&doc.graph, obj);
                Ok(SVal::Number((fields.len() as i32).into()))
            },
            "at" => {
                if parameters.len() == 1 {
                    match &parameters[0] {
                        SVal::String(index) => {
                            if let Some(field) = SField::field(&doc.graph, &index, '.', Some(obj)) {
                                return Ok(field.value);
                            } else if let Some(func) = SFunc::func(&doc.graph, &index, '.', Some(obj)) {
                                return Ok(SVal::FnPtr(func.data_ref()));
                            }
                            return Ok(SVal::Null); // Not found
                        },
                        SVal::Number(val) => {
                            let mut fields = SField::fields(&doc.graph, obj);
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
                }
                if parameters.len() > 1 {
                    let mut array = Vec::new();
                    for param in parameters.drain(..) {
                        match param {
                            SVal::String(index) => {
                                if let Some(field) = SField::field(&doc.graph, &index, '.', Some(obj)) {
                                    array.push(field.value);
                                } else if let Some(func) = SFunc::func(&doc.graph, &index, '.', Some(obj)) {
                                    array.push(SVal::FnPtr(func.data_ref()));
                                }
                            },
                            SVal::Number(val) => {
                                let mut fields = SField::fields(&doc.graph, obj);
                                let index = val.int() as usize;
                                if index < fields.len() {
                                    let field = fields.remove(index);
                                    let value = field.value;
                                    let key = SVal::String(field.name);
                                    array.push(SVal::Tuple(vec![key, value]));
                                }
                            },
                            _ => {}
                        }
                    }
                    return Ok(SVal::Array(array));
                }
                Err(anyhow!("Object.at(obj, index, ..) requires an index to be a number or string"))
            },
            "reference" => {
                if parameters.len() == 1 {
                    let field_path = parameters[0].to_string();
                    if let Some(mut field) = SField::field(&doc.graph, &field_path, '.', Some(obj)) {
                        self.operate(pid, doc, "remove", obj, &mut vec![SVal::String(field.name.clone())])?;
                        field.attach(obj, &mut doc.graph);
                        return Ok(SVal::Bool(true));
                    } else if let Some(mut field) = SField::field(&doc.graph, &field_path, '.', None) {
                        self.operate(pid, doc, "remove", obj, &mut vec![SVal::String(field.name.clone())])?;
                        field.attach(obj, &mut doc.graph);
                        return Ok(SVal::Bool(true));
                    } else if let Some(mut func) = SFunc::func(&doc.graph, &field_path, '.', Some(obj)) {
                        func.attach(obj, &mut doc.graph);
                        return Ok(SVal::Bool(true));
                    } else if let Some(mut func) = SFunc::func(&doc.graph, &field_path, '.', None) {
                        func.attach(obj, &mut doc.graph);
                        return Ok(SVal::Bool(true));
                    }
                    return Ok(SVal::Bool(false));
                } else if parameters.len() == 2 {
                    match &parameters[0] {
                        SVal::Object(context) => {
                            let field_path = parameters[1].to_string();
                            if let Some(mut field) = SField::field(&doc.graph, &field_path, '.', Some(&context)) {
                                self.operate(pid, doc, "remove", obj, &mut vec![SVal::String(field.name.clone())])?;
                                field.attach(obj, &mut doc.graph);
                                return Ok(SVal::Bool(true));
                            } else if let Some(mut func) = SFunc::func(&doc.graph, &field_path, '.', Some(&context)) {
                                func.attach(obj, &mut doc.graph);
                                return Ok(SVal::Bool(true));
                            }
                            return Ok(SVal::Bool(false));
                        },
                        _ => {}
                    }
                }
                Err(anyhow!("Object.reference(obj, context, path) requires a path parameter (optional context parameter)"))
            },
            "fields" => {
                let fields = SField::fields(&doc.graph, obj);
                let mut array = Vec::new();
                for field in fields {
                    let value = field.value;
                    let key = SVal::String(field.name);
                    array.push(SVal::Tuple(vec![key, value]));
                }
                Ok(SVal::Array(array))
            },
            "attributes" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("Object.attributes(obj, path: str) requires a path parameter to get attributes for (func or field)"));
                }
                match &parameters[0] {
                    SVal::String(index) => {
                        if let Some(field) = SField::field(&doc.graph, &index, '.', Some(obj)) {
                            let mut attrs = Vec::new();
                            for (key, value) in &field.attributes {
                                attrs.push(SVal::Tuple(vec![SVal::String(key.clone()), value.clone()]));
                            }
                            return Ok(SVal::Array(attrs));
                        } else if let Some(func) = SFunc::func(&doc.graph, &index, '.', Some(obj)) {
                            let mut attrs = Vec::new();
                            for (key, value) in &func.attributes {
                                attrs.push(SVal::Tuple(vec![SVal::String(key.clone()), value.clone()]));
                            }
                            return Ok(SVal::Array(attrs));
                        }
                        return Ok(SVal::Null); // Not found
                    },
                    _ => {
                        Err(anyhow!("Object.attributes(obj, path) requires an object and path parameters"))
                    }
                }
            },
            "funcs" |
            "functions" => {
                let funcs = SFunc::funcs(&doc.graph, obj);
                let mut array = Vec::new();
                for func in funcs {
                    let value = SVal::FnPtr(func.id.into());
                    let key = SVal::String(func.name);
                    array.push(SVal::Tuple(vec![key, value]));
                }
                Ok(SVal::Array(array))
            },
            "keys" => {
                let fields = SField::fields(&doc.graph, obj);
                let mut array = Vec::new();
                for field in fields {
                    array.push(SVal::String(field.name));
                }
                Ok(SVal::Array(array))
            },
            "values" => {
                let fields = SField::fields(&doc.graph, obj);
                let mut array = Vec::new();
                for field in fields {
                    array.push(field.value);
                }
                Ok(SVal::Array(array))
            },
            "set" => {
                if parameters.len() == 2 {
                    let value = parameters.pop().unwrap();
                    let name = parameters.pop().unwrap().to_string();

                    // Check for an existing field at this location
                    if let Some(mut field) = SField::field(&doc.graph, &name, '.', Some(obj)) {
                        field.value = value.clone();
                        field.set(&mut doc.graph);
                        return Ok(value);
                    }

                    // val is a dot separated path!
                    let mut path = name.split('.').collect::<Vec<&str>>();
                    let name = path.pop().unwrap().to_string();

                    // Ensure the path exists if we need to add objects
                    let mut fref = obj.clone();
                    if path.len() > 0 {
                        fref = doc.graph.ensure_nodes(&path.join("/"), '/', true, Some(obj.clone()));
                    }

                    // Create the field on fref
                    let mut field = SField::new(&name, value.clone());
                    field.attach(&fref, &mut doc.graph);
                    return Ok(value);
                }
                Err(anyhow!("Object.set(obj, name, value) requires a name and a value to set with"))
            },
            "remove" => {
                let mut field_names = HashSet::new();
                for val in parameters.drain(..) {
                    field_names.insert(val.to_string());
                }

                let fields = SField::fields(&doc.graph, obj);
                let mut array = Vec::new();
                for field in fields {
                    if field_names.contains(&field.name) {
                        // Remove field from the graph!
                        field.remove(&mut doc.graph, Some(obj));
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
                Ok(SVal::Array(array))
            },
            "name" => {
                if let Some(node) = obj.node(&doc.graph) {
                    return Ok(SVal::String(node.name.clone()));
                }
                Err(anyhow!("Object.name(obj) could not find object"))
            },
            "parent" => {
                if let Some(node) = obj.node(&doc.graph) {
                    if let Some(parent) = &node.parent {
                        return Ok(SVal::Object(parent.clone()));
                    }
                }
                Ok(SVal::Null)
            },
            "path" => {
                Ok(SVal::String(obj.path(&doc.graph).replace('/', ".")))
            },
            "children" => {
                if let Some(node) = obj.node(&doc.graph) {
                    let mut children = Vec::new();
                    for child in &node.children {
                        children.push(SVal::Object(child.clone()));
                    }
                    return Ok(SVal::Array(children));
                }
                Ok(SVal::Array(vec![]))
            },
            "typename" => {
                let typename = SVal::Object(obj.clone()).type_name(&doc.graph);
                Ok(SVal::String(typename))
            },
            "typestack" => {
                let typestack = SVal::Object(obj.clone()).type_stack(&doc.graph);
                Ok(SVal::Array(typestack.into_iter().map(|x| SVal::String(x)).collect()))
            },
            "instanceOf" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("Object.instanceOf(obj, type: str) requires a type string parameter to test type with"));
                }
                let iof = SVal::Object(obj.clone()).instance_of(&doc.graph, &parameters[0].to_string());
                Ok(SVal::Bool(iof))
            },
            "upcast" => {
                if let Some(mut prototype_field) = SField::field(&doc.graph, "__prototype__", '.', Some(obj)) {
                    if let Some(prototype) = doc.graph.node_from(&prototype_field.to_string(), None) {
                        if let Some(parent_ref) = &prototype.parent {
                            if let Some(parent) = parent_ref.node(&doc.graph) {
                                if parent.name != "__stof__" && parent.name != "prototypes" {
                                    prototype_field.value = SVal::String(parent_ref.path(&doc.graph));
                                    prototype_field.set(&mut doc.graph);
                                    return Ok(SVal::Bool(true));
                                }
                            }
                        }
                    }
                }
                Ok(SVal::Bool(false))
            },
            _ => {
                Err(anyhow!("Did not find the requested Object library function '{}'", name))
            }
        }
    }
}
impl Library for ObjectLibrary {
    fn scope(&self) -> String {
        "Object".into()
    }
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
                SVal::Object(nref) => {
                    return self.operate(pid, doc, name, nref, &mut params);
                },
                SVal::Boxed(val) => {
                    let val = val.lock().unwrap();
                    let val = val.deref();
                    match val {
                        SVal::Object(nref) => {
                            return self.operate(pid, doc, name, nref, &mut params);
                        },
                        _ => {
                            return Err(anyhow!("Object library requires the first parameter to be an obj"));
                        }
                    }
                },
                _ => {
                    return Err(anyhow!("Object library requires the first parameter to be an obj"));
                }
            }
        } else {
            return Err(anyhow!("Object library requires an 'obj' parameter to work with"));
        }
    }
}
