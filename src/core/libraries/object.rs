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

use std::{cmp::Ordering, collections::{BTreeMap, HashSet}, ops::Deref};
use anyhow::{anyhow, Result};
use crate::{Data, IntoDataRef, IntoNodeRef, Library, SDoc, SField, SFunc, SNodeRef, SNum, SVal};


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
                        self.operate(pid, doc, "removeField", obj, &mut vec![SVal::String(field.name.clone())])?;
                        field.attach(obj, &mut doc.graph);
                        return Ok(SVal::Bool(true));
                    } else if let Some(mut field) = SField::field(&doc.graph, &field_path, '.', None) {
                        self.operate(pid, doc, "removeField", obj, &mut vec![SVal::String(field.name.clone())])?;
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
                                self.operate(pid, doc, "removeField", obj, &mut vec![SVal::String(field.name.clone())])?;
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
                    let value = parameters.pop().unwrap().unbox();
                    let name = parameters.pop().unwrap().to_string();

                    // Check for an existing field at this location
                    if let Some(mut field) = SField::field(&doc.graph, &name, '.', Some(obj)) {
                        if doc.perms.can_write_field(&doc.graph, &field, Some(obj)) {
                            field.value = value;
                            field.set(&mut doc.graph);
                            return Ok(SVal::Bool(true));
                        }
                        return Ok(SVal::Bool(false));
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
                    let mut field = SField::new(&name, value);
                    field.attach(&fref, &mut doc.graph);
                    return Ok(SVal::Bool(true));
                }
                Err(anyhow!("Object.set(obj, name, value) requires a name and a value to set with"))
            },
            // Take a map and do rename/moves with all entries.
            // Signature: Object.mapFields(obj, map: map): void
            "mapFields" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("Object.mapFields(obj, map) requires a map parameter with source and destinations"));
                }
                match &parameters[0] {
                    SVal::Map(map) => {
                        let mut mapped_values = BTreeMap::new();
                        for (k, v) in map {
                            let res = self.operate(pid, doc, "renameField", obj, &mut vec![k.clone(), v.clone()])?;
                            if res.truthy() {
                                mapped_values.insert(k.clone(), v.clone());
                            }
                        }
                        Ok(SVal::Map(mapped_values))
                    },
                    _ => {
                        Err(anyhow!("Object.mapFields(obj, map) requires a map parameter with source and destinations"))
                    }
                }
            },
            // Rename/Move a field if this object has it and permissions allow.
            // Signature: Object.renameField(obj, field_path: str, new_path: str): bool
            "moveField" |
            "renameField" => {
                if parameters.len() < 2 {
                    return Err(anyhow!("Object.renameField(obj, path: str, new_path: str) requires two path parameters"));
                }
                let dest = parameters.pop().unwrap().to_string();
                let source = parameters.pop().unwrap().to_string();

                if let Some(mut field) = SField::field(&doc.graph, &source, '.', Some(obj)) {
                    if doc.perms.can_write_field(&doc.graph, &field, Some(obj)) {
                        // union the destination field if one already exists...
                        if let Some(mut existing) = SField::field(&doc.graph, &dest, '.', Some(obj)) {
                            // remove this field from the graph (everywhere, I know... no way to be sure that field is on obj)
                            field.remove(&mut doc.graph, None);
                            field.id = String::default(); // force the graph to create a new ID for this field for deadpool purposes...

                            existing.union(&field);
                            existing.set(&mut doc.graph);
                        } else {
                            // Get the new field name from the destination path
                            let mut dest_path = dest.split('.').collect::<Vec<&str>>();
                            let new_field_name = dest_path.pop().unwrap();
                            field.name = new_field_name.to_owned();

                            // If there is a new destination node, do that
                            if dest_path.len() > 0 {
                                // remove this field from the graph (everywhere, I know... no way to be sure that field is on obj)
                                field.remove(&mut doc.graph, None);
                                field.id = String::default(); // force the graph to create a new ID for this field for deadpool purposes...

                                let dest_node_path = dest_path.join(".");
                                let dest_ref = doc.graph.ensure_nodes(&dest_node_path, '.', true, Some(obj.clone()));
                                field.attach(&dest_ref, &mut doc.graph);

                                // If field is an object, move the object to the destination also and rename
                                match field.value {
                                    SVal::Object(nref) => {
                                        doc.graph.rename_node(&nref, new_field_name);

                                        let id_path: HashSet<String> = HashSet::from_iter(nref.id_path(&doc.graph).into_iter());
                                        if !id_path.contains(&dest_ref.id) && !dest_ref.is_child_of(&doc.graph, &nref) {
                                            doc.graph.move_node(nref, dest_ref);
                                        }
                                    },
                                    _ => {}
                                }
                            } else {
                                // We've just renamed the field, so just set it
                                field.set(&mut doc.graph);

                                // If field is an object, rename
                                match field.value {
                                    SVal::Object(nref) => {
                                        doc.graph.rename_node(&nref, new_field_name);
                                    },
                                    _ => {}
                                }
                            }
                        }
                        return Ok(SVal::Bool(true));
                    }
                }
                Ok(SVal::Bool(false))
            },
            // Delete a field (path), starting at this object.
            // Signature: Object.removeField(obj, path: str, remove_obj: bool): bool
            "removeField" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("Object.removeField(obj, path: str) requires a field path parameter"));
                }
                let mut remove_obj = false;
                if parameters.len() > 1 {
                    remove_obj = parameters.pop().unwrap().truthy();
                }
                let path = parameters.pop().unwrap().to_string();

                if let Some(field) = SField::field(&doc.graph, &path, '.', Some(obj)) {
                    if doc.perms.can_write_field(&doc.graph, &field, Some(obj)) {
                        if path.contains('.') {
                            field.remove(&mut doc.graph, None);
                        } else {
                            field.remove(&mut doc.graph, Some(obj));
                        }
                        
                        if remove_obj && field.value.is_object() {
                            match field.value.clone().unbox() {
                                SVal::Object(nref) => {
                                    doc.graph.remove_node(nref);
                                },
                                _ => {}
                            }
                        }

                        return Ok(SVal::Bool(true));
                    }
                }
                Ok(SVal::Bool(false))
            },
            // Delete a function (path), starting at this object.
            // Signature: Object.removeFunc(obj, path: str): bool
            "removeFunc" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("Object.removeFunc(obj, path: str) requires a func path parameter"));
                }
                let path = parameters.pop().unwrap().to_string();

                if let Some(func) = SFunc::func(&doc.graph, &path, '.', Some(obj)) {
                    if doc.perms.can_write_func(&doc.graph, &func, Some(obj)) {
                        if path.contains('.') {
                            func.remove(&mut doc.graph, None);
                        } else {
                            func.remove(&mut doc.graph, Some(obj));
                        }
                        return Ok(SVal::Bool(true));
                    }
                }
                Ok(SVal::Bool(false))
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
            // Return this objects root object.
            // Signature: Object.root(obj): obj
            "root" => {
                if let Some(root) = obj.root(&doc.graph) {
                    return Ok(SVal::Object(root.node_ref()));
                }
                Ok(SVal::Null)
            },
            // Is this object a root object?
            "isRoot" => {
                if let Some(node) = obj.node(&doc.graph) {
                    Ok(SVal::Bool(node.parent.is_none()))
                } else {
                    Ok(SVal::Bool(true)) // unreachable case
                }
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

            /*****************************************************************************
             * Search for fields.
             *****************************************************************************/
            // Searches both up and down, looking for the closest field with a given name.
            // Returns null if not found, otherwise the value and distance from this object.
            // Signature: Object.search(obj, field_name: str, search_parent_children: bool = true): null | (unknown, int)
            "search" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("Object.search(obj, field_name: str) requires a field name to search for"));
                }
                let up = self.operate(pid, doc, "searchUp", obj, parameters)?;
                let down = self.operate(pid, doc, "searchDown", obj, &mut vec![parameters[0].clone()])?;
                if !up.is_empty() && !down.is_empty() {
                    match up {
                        SVal::Tuple(up) => {
                            match down {
                                SVal::Tuple(down) => {
                                    let down_lte_up = down.last().unwrap().lte(up.last().unwrap())?;
                                    if down_lte_up.truthy() {
                                        // down is closer (or equal) - return down
                                        return Ok(SVal::Tuple(down));
                                    } else {
                                        // up is closer
                                        return Ok(SVal::Tuple(up));
                                    }
                                },
                                _ => {}
                            }
                        },
                        _ => {}
                    }
                } else if !up.is_empty() {
                    return Ok(up);
                } else if !down.is_empty() {
                    return Ok(down);
                }
                Ok(SVal::Null)
            },

            // Search upwards through our parents to find the closest field with a name.
            // Returns null if not found, otherwise the value and distance from this object.
            // Signature: Object.searchUp(obj, field_name: str, search_parent_children: bool = true): null | (unknown, int)
            "searchUp" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("Object.searchUp(obj, field_name: str) requires a field name to search for"));
                }
                let field_name = parameters[0].to_string();
                if let Some(field) = SField::field(&doc.graph, &field_name, '.', Some(obj)) {
                    if doc.perms.can_read_field(&doc.graph, &field, Some(obj)) {
                        return Ok(SVal::Tuple(vec![field.value, SVal::Number(SNum::I64(0))]));
                    }
                } else {
                    // Search up, through parent nodes
                    let mut allow_parent_children = true;
                    let mut child_finds = Vec::new();
                    if parameters.len() > 1 {
                        allow_parent_children = parameters[1].truthy();
                    }

                    let mut parent = None;
                    let mut parent_field = None;
                    if let Some(node) = obj.node(&doc.graph) {
                        parent = node.parent.clone();
                    }
                    let mut parent_distance = 1;
                    while parent.is_some() {
                        if let Some(field) = SField::field(&doc.graph, &field_name, '.', parent.as_ref()) {
                            parent_field = Some(field);
                            break;
                        }
                        if allow_parent_children {
                            let mut params = vec![parameters[0].clone(), SVal::Number(SNum::I64(parent_distance))];
                            let val = self.operate(pid, doc, "searchDown", &parent.clone().unwrap(), &mut params)?;
                            if !val.is_empty() {
                                child_finds.push(val);
                            }
                        }
                        
                        if let Some(node) = parent.unwrap().node(&doc.graph) {
                            parent = node.parent.clone();
                        } else {
                            parent = None;
                        }
                        parent_distance += 1;
                    }

                    // sort child finds by distance if any
                    if child_finds.len() > 0 {
                        child_finds.sort_by(|a, b| {
                            match a {
                                SVal::Tuple(a) => {
                                    match b {
                                        SVal::Tuple(b) => {
                                            let a_lt = a.last().unwrap().lt(b.last().unwrap()).unwrap();
                                            if a_lt.truthy() {
                                                return Ordering::Less;
                                            }
                                            let a_gt = a.last().unwrap().gt(b.last().unwrap()).unwrap();
                                            if a_gt.truthy() {
                                                return Ordering::Greater;
                                            }
                                        },
                                        _ => {}
                                    }
                                },
                                _ => {}
                            }
                            Ordering::Equal
                        });
                    }

                    if let Some(field) = parent_field { // will be the first in-line value found if any
                        if child_finds.len() > 0 {
                            // compare the closest child find to this parent find, preferring the parent find when equal
                            let first_child = child_finds.remove(0);
                            let mut return_child = false;
                            match &first_child {
                                SVal::Tuple(tup) => {
                                    match &tup.last().unwrap() {
                                        SVal::Number(num) => {
                                            let dist = num.int();
                                            if dist < parent_distance {
                                                return_child = true;
                                            }
                                        },
                                        _ => {}
                                    }
                                },
                                _ => {}
                            }
                            if return_child {
                                return Ok(first_child);
                            }
                        }
                        if doc.perms.can_read_field(&doc.graph, &field, Some(obj)) {
                            return Ok(SVal::Tuple(vec![field.value, SVal::Number(SNum::I64(parent_distance))]));
                        }
                    } else if child_finds.len() > 0 {
                        return Ok(child_finds.remove(0));
                    }
                }
                Ok(SVal::Null)
            },

            // Search downwards through our children to find the closest field with a name.
            // Returns null if not found, otherwise the value and distance from this object.
            // Signature: Object.searchDown(obj, field_name: str): null | (unknown, int)
            "searchDown" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("Object.searchDown(obj, field_name: str) requires a field name to search for"));
                }
                let mut current_distance = 0;
                if parameters.len() > 1 {
                    match &parameters[1] {
                        SVal::Number(num) => {
                            current_distance = num.int();
                        },
                        _ => {}
                    }
                }
                let field_name = parameters[0].to_string();
                if let Some(field) = SField::field(&doc.graph, &field_name, '.', Some(obj)) {
                    if doc.perms.can_read_field(&doc.graph, &field, Some(obj)) {
                        return Ok(SVal::Tuple(vec![field.value, SVal::Number(SNum::I64(current_distance))]));
                    }
                } else {
                    let children;
                    if let Some(node) = obj.node(&doc.graph) {
                        children = node.children.clone();
                    } else {
                        return Ok(SVal::Null); // no children to consider
                    }
                    let mut params = vec![parameters[0].clone(), SVal::Number(SNum::I64(current_distance + 1))];
                    for child in children {
                        let val = self.operate(pid, doc, "searchDown", &child, &mut params)?;
                        if !val.is_empty() {
                            return Ok(val);
                        }
                    }
                }
                Ok(SVal::Null)
            },

            /*****************************************************************************
             * Copy object helpers.
             *****************************************************************************/
            // Make this object a shallow copy of the referenced object by attaching all of its fields.
            // Signature: Object.shallowCopy(obj, to_copy: obj): void
            "shallowCopy" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("Object.shallowCopy(obj, to_copy: obj) requires an object parameter to copy"));
                }
                match &parameters[0] {
                    SVal::Object(to_copy) => {
                        let data;
                        if let Some(copy_node) = to_copy.node(&doc.graph) {
                            data = copy_node.data.clone();
                        } else {
                            return Err(anyhow!("Object.shallowCopy(obj, to_copy: obj) copy node does not exist"));
                        }
                        for data in data {
                            doc.graph.put_data_ref(obj, data);
                        }
                        Ok(SVal::Void)
                    },
                    _ => {
                        Err(anyhow!("Object.shallowCopy(obj, to_copy: obj) requires an object parameter to copy"))
                    }
                }
            },
            // Make this object a deep copy of the referenced object (fields only).
            // Captures attributes, deep copies sub objects, etc.. as well.
            // Signature: Object.deepCopyFields(obj, to_copy: obj): void
            "deepCopyFields" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("Object.deepCopyFields(obj, to_copy: obj) requires an object parameter to copy"));
                }
                match &parameters[0] {
                    SVal::Object(to_copy) => {
                        for mut field in SField::fields(&doc.graph, to_copy) {
                            if field.is_object() {
                                match field.value.unbox() {
                                    SVal::Object(nref) => {
                                        let deep_copy = SField::new_object(&mut doc.graph, &field.name, obj);
                                        let to_copy = SVal::Object(nref);
                                        self.operate(pid, doc, "deepCopyFields", &deep_copy, &mut vec![to_copy])?;
                                    },
                                    _ => {}
                                }
                            } else {
                                field.id = String::default(); // new data in the graph, already cloned
                                field.attach(obj, &mut doc.graph);
                            }
                        }
                        Ok(SVal::Void)
                    },
                    _ => {
                        Err(anyhow!("Object.deepCopyFields(obj, to_copy: obj) requires an object parameter to copy"))
                    }
                }
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
