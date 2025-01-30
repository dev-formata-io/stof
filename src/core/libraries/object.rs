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
use crate::{lang::SError, IntoNodeRef, Library, SData, SDoc, SField, SFunc, SNodeRef, SNum, SPrototype, SVal};


#[derive(Default, Debug)]
pub struct ObjectLibrary;
impl ObjectLibrary {
    /// Call object operation.
    pub fn operate(&self, pid: &str, doc: &mut SDoc, name: &str, obj: &SNodeRef, parameters: &mut Vec<SVal>) -> Result<SVal, SError> {
        match name {
            "len" => {
                if let Some(node) = obj.node(&doc.graph) {
                    let refs = node.data_refs::<SField>(&doc.graph);
                    return Ok(SVal::Number(SNum::I64(refs.len() as i64)));
                }
                Ok(SVal::Number(SNum::I64(0)))
            },
            "at" => {
                if parameters.len() == 1 {
                    match &parameters[0] {
                        SVal::String(index) => {
                            if let Some(field) = SField::field(&doc.graph, &index, '.', Some(obj)) {
                                return Ok(field.value.clone());
                            } else if let Some(func) = SFunc::func_ref(&doc.graph, &index, '.', Some(obj)) {
                                return Ok(SVal::FnPtr(func));
                            }
                            return Ok(SVal::Null); // Not found
                        },
                        SVal::Number(val) => {
                            let mut fields = SField::fields(&doc.graph, obj);
                            let index = val.int() as usize;
                            if index < fields.len() {
                                let field = fields.remove(index);
                                let value = field.value.clone();
                                let key = SVal::String(field.name.clone());
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
                                    array.push(field.value.clone());
                                } else if let Some(func) = SFunc::func_ref(&doc.graph, &index, '.', Some(obj)) {
                                    array.push(SVal::FnPtr(func));
                                }
                            },
                            SVal::Number(val) => {
                                let mut fields = SField::fields(&doc.graph, obj);
                                let index = val.int() as usize;
                                if index < fields.len() {
                                    let field = fields.remove(index);
                                    let value = field.value.clone();
                                    let key = SVal::String(field.name.clone());
                                    array.push(SVal::Tuple(vec![key, value]));
                                }
                            },
                            _ => {}
                        }
                    }
                    return Ok(SVal::Array(array));
                }
                Err(SError::obj(pid, &doc, "at", "invalid arguments - index must be a string or number"))
            },
            "reference" => {
                if parameters.len() == 1 {
                    let field_path = parameters[0].to_string();
                    if let Some(field_ref) = SField::field_ref(&doc.graph, &field_path, '.', Some(obj)) {
                        if let Some(field) = SData::get::<SField>(&doc.graph, &field_ref) {
                            self.operate(pid, doc, "removeField", obj, &mut vec![SVal::String(field.name.clone())])?;
                        }
                        SData::attach_existing(&mut doc.graph, obj, field_ref);
                        return Ok(SVal::Bool(true));
                    } else if let Some(field_ref) = SField::field_ref(&doc.graph, &field_path, '.', None) {
                        if let Some(field) = SData::get::<SField>(&doc.graph, &field_ref) {
                            self.operate(pid, doc, "removeField", obj, &mut vec![SVal::String(field.name.clone())])?;
                        }
                        SData::attach_existing(&mut doc.graph, obj, field_ref);
                        return Ok(SVal::Bool(true));
                    } else if let Some(func) = SFunc::func_ref(&doc.graph, &field_path, '.', Some(obj)) {
                        SData::attach_existing(&mut doc.graph, obj, func);
                        return Ok(SVal::Bool(true));
                    } else if let Some(func) = SFunc::func_ref(&doc.graph, &field_path, '.', None) {
                        SData::attach_existing(&mut doc.graph, obj, func);
                        return Ok(SVal::Bool(true));
                    }
                    return Ok(SVal::Bool(false));
                } else if parameters.len() == 2 {
                    match &parameters[0] {
                        SVal::Object(context) => {
                            let field_path = parameters[1].to_string();
                            if let Some(field_ref) = SField::field_ref(&doc.graph, &field_path, '.', Some(&context)) {
                                if let Some(field) = SData::get::<SField>(&doc.graph, &field_ref) {
                                    self.operate(pid, doc, "removeField", obj, &mut vec![SVal::String(field.name.clone())])?;
                                }
                                SData::attach_existing(&mut doc.graph, obj, field_ref);
                                return Ok(SVal::Bool(true));
                            } else if let Some(func) = SFunc::func_ref(&doc.graph, &field_path, '.', Some(&context)) {
                                SData::attach_existing(&mut doc.graph, obj, func);
                                return Ok(SVal::Bool(true));
                            }
                            return Ok(SVal::Bool(false));
                        },
                        _ => {}
                    }
                }
                Err(SError::obj(pid, &doc, "reference", "path argument not found"))
            },
            "fields" => {
                let fields = SField::fields(&doc.graph, obj);
                let mut map = BTreeMap::new();
                for field in fields {
                    let value = field.value.clone();
                    let key = SVal::String(field.name.clone());
                    map.insert(key, value);
                }
                Ok(SVal::Map(map))
            },
            "attributes" => {
                if parameters.len() < 1 {
                    return Err(SError::obj(pid, &doc, "attributes", "invalid arguments - path not found"));
                }
                match &parameters[0] {
                    SVal::String(index) => {
                        if let Some(field) = SField::field(&doc.graph, &index, '.', Some(obj)) {
                            let mut attrs = BTreeMap::new();
                            for (key, value) in &field.attributes {
                                attrs.insert(SVal::String(key.clone()), value.clone());
                            }
                            return Ok(SVal::Map(attrs));
                        } else if let Some(func_ref) = SFunc::func_ref(&doc.graph, &index, '.', Some(obj)) {
                            let mut attrs = BTreeMap::new();
                            if let Some(func) = SData::get::<SFunc>(&doc.graph, &func_ref) {
                                for (key, value) in &func.attributes {
                                    attrs.insert(SVal::String(key.clone()), value.clone());
                                }
                            }
                            return Ok(SVal::Map(attrs));
                        }
                        return Ok(SVal::Null); // Not found
                    },
                    SVal::Boxed(val) => {
                        let val = val.lock().unwrap();
                        let val = val.deref();
                        match val {
                            SVal::String(index) => {
                                if let Some(field) = SField::field(&doc.graph, &index, '.', Some(obj)) {
                                    let mut attrs = BTreeMap::new();
                                    for (key, value) in &field.attributes {
                                        attrs.insert(SVal::String(key.clone()), value.clone());
                                    }
                                    return Ok(SVal::Map(attrs));
                                } else if let Some(func_ref) = SFunc::func_ref(&doc.graph, &index, '.', Some(obj)) {
                                    let mut attrs = BTreeMap::new();
                                    if let Some(func) = SData::get::<SFunc>(&doc.graph, &func_ref) {
                                        for (key, value) in &func.attributes {
                                            attrs.insert(SVal::String(key.clone()), value.clone());
                                        }
                                    }
                                    return Ok(SVal::Map(attrs));
                                }
                                return Ok(SVal::Null); // Not found
                            },
                            _ => {
                                Err(SError::obj(pid, &doc, "attributes", "invalid arguments - path must be a string"))
                            }
                        }
                    },
                    _ => {
                        Err(SError::obj(pid, &doc, "attributes", "invalid arguments - path must be a string"))
                    }
                }
            },
            "funcs" |
            "functions" => {
                let funcs = SFunc::func_refs(&doc.graph, obj);
                let mut map = BTreeMap::new();
                for func_ref in funcs {
                    if let Some(func) = SData::get::<SFunc>(&doc.graph, &func_ref) {
                        let value = SVal::FnPtr(func_ref);
                        let key = SVal::String(func.name.clone());
                        map.insert(key, value);
                    }
                }
                Ok(SVal::Map(map))
            },
            "keys" => {
                let fields = SField::fields(&doc.graph, obj);
                let mut array = Vec::new();
                for field in fields {
                    array.push(SVal::String(field.name.clone()));
                }
                Ok(SVal::Array(array))
            },
            "values" => {
                let fields = SField::fields(&doc.graph, obj);
                let mut array = Vec::new();
                for field in fields {
                    array.push(field.value.clone());
                }
                Ok(SVal::Array(array))
            },
            // Unbox a field without an assign operation.
            // Can be used like "set", but with an unbox operation in the middle.
            "unbox" => {
                if parameters.len() < 1 {
                    return Err(SError::obj(pid, &doc, "unbox", "invalid arguments - expecing a string path to a field that should be unboxed on this object"));
                }
                let path = parameters[0].to_string();
                let mut value = None;
                if parameters.len() > 1 {
                    value = Some(parameters.pop().unwrap());
                }

                // Look for an existing field to unbox
                if let Some(field_ref) = SField::field_ref(&doc.graph, &path, '.', Some(obj)) {
                    if !doc.perms.can_write_field(&doc.graph, &field_ref, Some(obj)) {
                        return Ok(SVal::Bool(false));
                    }
                    if let Some(field) = SData::get_mut::<SField>(&mut doc.graph, &field_ref) {
                        if let Some(value) = value {
                            field.value = value.unbox();
                        } else {
                            field.value.unbox_ref();
                        }
                        return Ok(SVal::Bool(true));
                    }
                    return Ok(SVal::Bool(false));
                }
                if let Some(value) = value {
                    // val is a dot separated path!
                    let mut path = path.split('.').collect::<Vec<&str>>();
                    let name = path.pop().unwrap().to_string();

                    // Ensure the path exists if we need to add objects
                    let mut fref = obj.clone();
                    if path.len() > 0 {
                        fref = doc.graph.ensure_nodes(&path.join("/"), '/', true, Some(obj.clone()));
                    }

                    // Create the field on fref with the unboxed value
                    let field = SField::new(&name, value.unbox());
                    SData::insert_new(&mut doc.graph, &fref, Box::new(field));
                    return Ok(SVal::Bool(true));
                }
                Ok(SVal::Bool(false))
            },
            // Box a field without an assign operation.
            // Can be used like "set", but with a box operation in the middle.
            "box" => {
                if parameters.len() < 1 {
                    return Err(SError::obj(pid, &doc, "box", "invalid arguments - expecing a string path to a field that should be boxed on this object"));
                }
                let path = parameters[0].to_string();
                let mut value = None;
                if parameters.len() > 1 {
                    value = Some(parameters.pop().unwrap());
                }

                // Look for an existing field to box
                if let Some(field_ref) = SField::field_ref(&doc.graph, &path, '.', Some(obj)) {
                    if !doc.perms.can_write_field(&doc.graph, &field_ref, Some(obj)) {
                        return Ok(SVal::Bool(false));
                    }
                    if let Some(field) = SData::get_mut::<SField>(&mut doc.graph, &field_ref) {
                        if let Some(value) = value {
                            field.value = value.to_box();
                        } else {
                            field.value.to_box_ref();
                        }
                        return Ok(SVal::Bool(true));
                    }
                    return Ok(SVal::Bool(false));
                }

                // val is a dot separated path!
                let mut path = path.split('.').collect::<Vec<&str>>();
                let name = path.pop().unwrap().to_string();

                // Ensure the path exists if we need to add objects
                let mut fref = obj.clone();
                if path.len() > 0 {
                    fref = doc.graph.ensure_nodes(&path.join("/"), '/', true, Some(obj.clone()));
                }

                // Create the field on fref
                let mut field = SField::new(&name, SVal::Null.to_box());
                if let Some(value) = value {
                    field.value = value.to_box();
                }
                SData::insert_new(&mut doc.graph, &fref, Box::new(field));
                Ok(SVal::Bool(true))
            },
            "set" => {
                if parameters.len() == 2 {
                    let value = parameters.pop().unwrap();
                    let name = parameters.pop().unwrap().to_string();

                    // Check for an existing field at this location
                    if let Some(field_ref) = SField::field_ref(&doc.graph, &name, '.', Some(obj)) {
                        if !doc.perms.can_write_field(&doc.graph, &field_ref, Some(obj)) {
                            return Ok(SVal::Bool(false));
                        }
                        if let Some(field) = SData::get_mut::<SField>(&mut doc.graph, &field_ref) {
                            field.value = value;
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
                    let field = SField::new(&name, value);
                    SData::insert_new(&mut doc.graph, &fref, Box::new(field));
                    return Ok(SVal::Bool(true));
                }
                Err(SError::obj(pid, &doc, "set", "invalid arguments - requires a name and value to set a field"))
            },
            // Take a map and do rename/moves with all entries.
            // Signature: Object.mapFields(obj, map: map): map
            "mapFields" => {
                if parameters.len() < 1 {
                    return Err(SError::obj(pid, &doc, "mapFields", "invalid arguments - requires a map argument"));
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
                    SVal::Boxed(val) => {
                        let val = val.lock().unwrap();
                        let val = val.deref();
                        match val {
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
                                Err(SError::obj(pid, &doc, "mapFields", "invalid arguments - map argument not found"))
                            }
                        }
                    },
                    _ => {
                        Err(SError::obj(pid, &doc, "mapFields", "invalid arguments - map argument not found"))
                    }
                }
            },
            // Rename/Move a field if this object has it and permissions allow.
            // Signature: Object.renameField(obj, field_path: str, new_path: str): bool
            "moveField" |
            "renameField" => {
                if parameters.len() < 2 {
                    return Err(SError::obj(pid, &doc, "moveField", "invalid arguments - requires two paths, a source and a destination"));
                }
                let dest = parameters.pop().unwrap().to_string();
                let source = parameters.pop().unwrap().to_string();

                if let Some(field_ref) = SField::field_ref(&doc.graph, &source, '.', Some(obj)) {
                    if !doc.perms.can_write_field(&doc.graph, &field_ref, Some(obj)) {
                        return Ok(SVal::Bool(false));
                    }

                    // union the destination field if one already exists...
                    if let Some(existing_ref) = SField::field_ref(&doc.graph, &dest, '.', Some(obj)) {
                        // Clone the field
                        let clone;
                        if let Some(field) = SData::get::<SField>(&doc.graph, &field_ref) {
                            clone = field.clone();
                        } else {
                            return Ok(SVal::Bool(false));
                        }

                        // remove this field from the graph (everywhere, I know... no way to be sure that field is on obj)
                        doc.graph.remove_data(&field_ref, None);

                        if let Some(existing) = SData::get_mut::<SField>(&mut doc.graph, existing_ref) {
                            existing.merge(&clone)?;
                        }
                    } else {
                        // Get the new field name from the destination path
                        let mut dest_path = dest.split('.').collect::<Vec<&str>>();
                        let new_field_name = dest_path.pop().unwrap();

                        // If there is a new destination node, do that
                        if dest_path.len() > 0 {
                            // Clone the field
                            let mut clone;
                            if let Some(field) = SData::get::<SField>(&doc.graph, &field_ref) {
                                clone = field.clone();
                            } else {
                                return Ok(SVal::Bool(false));
                            }

                            // remove this field from the graph (everywhere, I know... no way to be sure that field is on obj)
                            doc.graph.remove_data(&field_ref, None);

                            clone.name = new_field_name.to_owned();
                            let dest_node_path = dest_path.join(".");
                            let dest_ref = doc.graph.ensure_nodes(&dest_node_path, '.', true, Some(obj.clone()));

                            // If field is an object, move the object to the destination also and rename
                            match &clone.value {
                                SVal::Object(nref) => {
                                    doc.graph.rename_node(nref, new_field_name);

                                    let id_path: HashSet<String> = HashSet::from_iter(nref.id_path(&doc.graph).into_iter());
                                    if !id_path.contains(&dest_ref.id) && !dest_ref.is_child_of(&doc.graph, &nref) {
                                        doc.graph.move_node(nref, &dest_ref);
                                    }
                                },
                                _ => {}
                            }

                            SData::insert_new_id(&mut doc.graph, &dest_ref, Box::new(clone), &field_ref.id); // keep same id
                        } else {
                            // We've only renamed the field, so do that only
                            let mut rename_node = None;
                            if let Some(field) = SData::get_mut::<SField>(&mut doc.graph, field_ref) {
                                field.name = new_field_name.to_owned();

                                // If field is an object, rename
                                match &field.value {
                                    SVal::Object(nref) => {
                                        rename_node = Some(nref.clone());
                                    },
                                    _ => {}
                                }
                            }
                            if let Some(rename_node) = rename_node {
                                doc.graph.rename_node(&rename_node, new_field_name);
                            }
                        }
                    }
                    return Ok(SVal::Bool(true));
                }
                Ok(SVal::Bool(false))
            },
            // Delete a field (path), starting at this object.
            // Signature: Object.removeField(obj, path: str, remove_obj: bool): bool
            "removeField" => {
                if parameters.len() < 1 {
                    return Err(SError::obj(pid, &doc, "removeField", "invalid arguments - field path not found"));
                }
                let mut remove_obj = false;
                if parameters.len() > 1 {
                    remove_obj = parameters.pop().unwrap().truthy();
                }
                let path = parameters.pop().unwrap().to_string();

                if let Some(field_ref) = SField::field_ref(&doc.graph, &path, '.', Some(obj)) {
                    if !doc.perms.can_write_field(&doc.graph, &field_ref, Some(obj)) {
                        return Ok(SVal::Bool(false));
                    }
                    if let Some(field) = SData::get::<SField>(&doc.graph, &field_ref) {
                        if remove_obj && field.value.is_object() {
                            match field.value.clone().unbox() {
                                SVal::Object(nref) => {
                                    doc.types.drop_types_for(&nref, &doc.graph);
                                    doc.graph.remove_node(nref);
                                },
                                _ => {}
                            }
                        }
                    }

                    if path.contains('.') {
                        // remove from everywhere
                        doc.graph.remove_data(field_ref, None);
                    } else {
                        // remove only from this node (potentially everywhere)
                        doc.graph.remove_data(field_ref, Some(obj));
                    }
                    return Ok(SVal::Bool(true));
                }
                Ok(SVal::Bool(false))
            },
            // Delete a function (path), starting at this object.
            // Signature: Object.removeFunc(obj, path: str): bool
            "removeFunc" => {
                if parameters.len() < 1 {
                    return Err(SError::obj(pid, &doc, "removeFunc", "invalid arguments - func path not found"));
                }
                let path = parameters.pop().unwrap().to_string();

                if let Some(func_ref) = SFunc::func_ref(&doc.graph, &path, '.', Some(obj)) {
                    if !doc.perms.can_write_func(&doc.graph, &func_ref, Some(obj)) {
                        return Ok(SVal::Bool(false));
                    }
                    if path.contains('.') {
                        doc.graph.remove_data(func_ref, None);
                    } else {
                        doc.graph.remove_data(func_ref, Some(obj));
                    }
                    return Ok(SVal::Bool(true));
                }
                Ok(SVal::Bool(false))
            },
            "name" => {
                if let Some(node) = obj.node(&doc.graph) {
                    return Ok(SVal::String(node.name.clone()));
                }
                Err(SError::obj(pid, &doc, "name", "could not find object"))
            },
            "id" => {
                Ok(SVal::String(obj.id.clone()))
            },
            "parent" => {
                if let Some(node) = obj.node(&doc.graph) {
                    if let Some(parent) = &node.parent {
                        return Ok(SVal::Object(parent.clone()));
                    }
                }
                Ok(SVal::Null)
            },
            // Return this objects prototype object (if any)
            "prototype" => {
                if let Some(prototype) = SPrototype::get(&doc.graph, obj) {
                    return Ok(SVal::Object(prototype.node_ref()));
                }
                Ok(SVal::Null)
            },
            // Set this objects prototype explicitly.
            "setPrototype" => {
                if parameters.len() < 1 {
                    return Err(SError::obj(pid, &doc, "setPrototype", "invalid arguments - object prototype not found"));
                }
                match &parameters[0] {
                    SVal::Object(nref) => {
                        if let Some(prototype_ref) = SPrototype::get_ref(&doc.graph, obj) {
                            if let Some(prototype) = SData::get_mut::<SPrototype>(&mut doc.graph, prototype_ref) {
                                prototype.prototype = nref.id.clone();
                            }
                        } else {
                            let prototype = SPrototype::new(nref);
                            SData::insert_new(&mut doc.graph, obj, Box::new(prototype));
                        }
                        return Ok(SVal::Void);
                    },
                    SVal::Boxed(val) => {
                        let val = val.lock().unwrap();
                        let val = val.deref();
                        match val {
                            SVal::Object(nref) => {
                                if let Some(prototype_ref) = SPrototype::get_ref(&doc.graph, obj) {
                                    if let Some(prototype) = SData::get_mut::<SPrototype>(&mut doc.graph, prototype_ref) {
                                        prototype.prototype = nref.id.clone();
                                    }
                                } else {
                                    let prototype = SPrototype::new(nref);
                                    SData::insert_new(&mut doc.graph, obj, Box::new(prototype));
                                }
                                return Ok(SVal::Void);
                            },
                            _ => {}
                        }
                    },
                    _ => {}
                }
                Err(SError::obj(pid, &doc, "setPrototype", "invalid arguments - object prototype not found"))
            },
            // Get the attributes for the prototype of this object (if any)
            "prototypeAttributes" => {
                if let Some(prototype) = SPrototype::get(&doc.graph, obj) {
                    let attributes = prototype.attributes(&doc);
                    let mut map = BTreeMap::new();
                    for (k, v) in attributes {
                        map.insert(SVal::String(k), v);
                    }
                    return Ok(SVal::Map(map));
                }
                Ok(SVal::Null)
            }
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
                    return Err(SError::obj(pid, &doc, "instanceOf", "invalid arguments - type string not found"));
                }
                let iof = SVal::Object(obj.clone()).instance_of(&doc.graph, &parameters[0].to_string());
                Ok(SVal::Bool(iof))
            },
            "upcast" => {
                if let Some(prototype_ref) = SPrototype::get_ref(&doc.graph, obj) {
                    let mut parent_id = String::default();
                    if let Some(prototype) = SData::get::<SPrototype>(&doc.graph, &prototype_ref) {
                        if let Some(node) = prototype.node_ref().node(&doc.graph) {
                            if let Some(parent_ref) = &node.parent {
                                if let Some(parent) = parent_ref.node(&doc.graph) {
                                    if parent.name != "__stof__" && parent.name != "prototypes" {
                                        parent_id = parent.id.clone();
                                    }
                                }
                            }
                        }
                    }
                    if parent_id.len() > 0 {
                        if let Some(prototype) = SData::get_mut::<SPrototype>(&mut doc.graph, prototype_ref) {
                            prototype.prototype = parent_id;
                            return Ok(SVal::Bool(true));
                        }
                    }
                }
                Ok(SVal::Bool(false))
            },
            // Remove the prototype for this object if any, returning whether one was removed or not.
            "removePrototype" => {
                if let Some(prototype) = SPrototype::get_ref(&doc.graph, obj) {
                    doc.graph.remove_data(prototype, Some(obj));
                    return Ok(SVal::Bool(true));
                }
                Ok(SVal::Bool(false))
            },

            // dump this object (testing)
            "dbg_dump" => {
                if let Some(node) = obj.node(&doc.graph) {
                    let dump = node.dump(&doc.graph, 0, true);
                    println!("{dump}");
                }
                Ok(SVal::Void)
            },

            /*****************************************************************************
             * Search for fields.
             *****************************************************************************/
            // Searches both up and down, looking for the closest field with a given name.
            // Returns null if not found, otherwise the value and distance from this object.
            // Signature: Object.search(obj, field_name: str, search_parent_children: bool = true, obj_ignore_set: vec = []): null | (unknown, int)
            "search" => {
                if parameters.len() < 1 {
                    return Err(SError::obj(pid, &doc, "search", "invalid arguments - field name not found"));
                }
                let up = self.operate(pid, doc, "searchUp", obj, parameters)?;
                
                let mut down_parameters = vec![parameters[0].clone(), SVal::Number(SNum::I64(0))];
                if parameters.len() > 2 { down_parameters.push(parameters[2].clone()); }
                let down = self.operate(pid, doc, "searchDown", obj, &mut down_parameters)?;
                
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
            // Signature: Object.searchUp(obj, field_name: str, search_parent_children: bool = true, obj_ignore_set: vec = []): null | (unknown, int)
            "searchUp" => {
                if parameters.len() < 1 {
                    return Err(SError::obj(pid, &doc, "searchUp", "invalid arguments - field name not found"));
                }
                let mut obj_ignore_set = HashSet::new();
                if parameters.len() > 2 {
                    match &parameters[2] {
                        SVal::Array(vals) => {
                            for v in vals {
                                match v {
                                    SVal::Object(nref) => {
                                        obj_ignore_set.insert(nref.id.clone());
                                    },
                                    _ => {}
                                }
                            }
                        },
                        _ => {}
                    }
                }

                let field_name = parameters[0].to_string();

                if !obj_ignore_set.contains(&obj.id) {
                    if let Some(field_ref) = SField::field_ref(&doc.graph, &field_name, '.', Some(obj)) {
                        if doc.perms.can_read_field(&doc.graph, &field_ref, Some(obj)) {
                            if let Some(field) = SData::get::<SField>(&doc.graph, &field_ref) {
                                return Ok(SVal::Tuple(vec![field.value.clone(), SVal::Number(SNum::I64(0))]));
                            }
                        }
                    }
                }
                obj_ignore_set.insert(obj.id.clone()); // already searched in this object

                // Search up, through parent nodes
                let mut allow_parent_children = true;
                let mut child_finds = Vec::new();
                if parameters.len() > 1 {
                    allow_parent_children = parameters[1].truthy();
                }

                let mut parent = None;
                let mut parent_field = None;
                let mut parent_field_ref = None;
                if let Some(node) = obj.node(&doc.graph) {
                    parent = node.parent.clone();
                }
                let mut parent_distance = 1;
                while parent.is_some() {
                    if let Some(parent) = &parent {
                        if !obj_ignore_set.contains(&parent.id) {
                            if let Some(field_ref) = SField::field_ref(&doc.graph, &field_name, '.', Some(parent)) {
                                if let Some(field) = SData::get::<SField>(&doc.graph, &field_ref) {
                                    parent_field = Some(field);
                                }
                                parent_field_ref = Some(field_ref);
                                break;
                            }
                            obj_ignore_set.insert(parent.id.clone()); // just searched this parent
                        }
                    }
                    if allow_parent_children {
                        let mut params = vec![parameters[0].clone(), SVal::Number(SNum::I64(parent_distance))];
                        if obj_ignore_set.len() > 0 {
                            let vals = obj_ignore_set.iter().map(|id| SVal::Object(SNodeRef::new(id))).collect();
                            params.push(SVal::Array(vals));
                        }
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
                    if let Some(parent_ref) = parent_field_ref {
                        if doc.perms.can_read_field(&doc.graph, &parent_ref, Some(obj)) {
                            return Ok(SVal::Tuple(vec![field.value.clone(), SVal::Number(SNum::I64(parent_distance))]));
                        }
                    }
                } else if child_finds.len() > 0 {
                    return Ok(child_finds.remove(0));
                }
                Ok(SVal::Null)
            },

            // Search downwards through our children to find the closest field with a name.
            // Returns null if not found, otherwise the value and distance from this object.
            // Signature: Object.searchDown(obj, field_name: str, current_dist: int = 0, obj_ignore_set: vec = []): null | (unknown, int)
            "searchDown" => {
                if parameters.len() < 1 {
                    return Err(SError::obj(pid, &doc, "searchDown", "invalid arguments - field name not found"));
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
                let mut obj_ignore_set = HashSet::new();
                if parameters.len() > 2 {
                    match &parameters[2] {
                        SVal::Array(vals) => {
                            for v in vals {
                                match v {
                                    SVal::Object(nref) => {
                                        obj_ignore_set.insert(nref.id.clone());
                                    },
                                    _ => {}
                                }
                            }
                        },
                        _ => {}
                    }
                }

                let field_name = parameters[0].to_string();

                if !obj_ignore_set.contains(&obj.id) {
                    if let Some(field_ref) = SField::field_ref(&doc.graph, &field_name, '.', Some(obj)) {
                        if let Some(field) = SData::get::<SField>(&doc.graph, &field_ref) {
                            if doc.perms.can_read_field(&doc.graph, &field_ref, Some(obj)) {
                                return Ok(SVal::Tuple(vec![field.value.clone(), SVal::Number(SNum::I64(current_distance))]));
                            }
                        }
                    }
                }
                let children;
                if let Some(node) = obj.node(&doc.graph) {
                    children = node.children.clone();
                } else {
                    return Ok(SVal::Null); // no children to consider
                }
                let mut params = vec![parameters[0].clone(), SVal::Number(SNum::I64(current_distance + 1))];
                if obj_ignore_set.len() > 0 {
                    params.push(parameters[2].clone());
                }
                let mut child_finds = Vec::new();
                for child in children {
                    let val = self.operate(pid, doc, "searchDown", &child, &mut params)?;
                    if !val.is_empty() {
                        child_finds.push(val);
                    }
                }
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
                    return Ok(child_finds.remove(0));
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
                    return Err(SError::obj(pid, &doc, "shallowCopy", "invalid arguments - object to copy not found"));
                }
                match &parameters[0] {
                    SVal::Object(to_copy) => {
                        let data;
                        if let Some(copy_node) = to_copy.node(&doc.graph) {
                            data = copy_node.data.clone();
                        } else {
                            return Err(SError::obj(pid, &doc, "shallowCopy", "invalid arguments - object to copy does not exist"));
                        }
                        for data in data {
                            doc.graph.put_data_ref(obj, data);
                        }
                        Ok(SVal::Void)
                    },
                    SVal::Boxed(val) => {
                        let val = val.lock().unwrap();
                        let val = val.deref();
                        match val {
                            SVal::Object(to_copy) => {
                                let data;
                                if let Some(copy_node) = to_copy.node(&doc.graph) {
                                    data = copy_node.data.clone();
                                } else {
                                    return Err(SError::obj(pid, &doc, "shallowCopy", "invalid arguments - object to copy does not exist"));
                                }
                                for data in data {
                                    doc.graph.put_data_ref(obj, data);
                                }
                                Ok(SVal::Void)
                            },
                            _ => {
                                Err(SError::obj(pid, &doc, "shallowCopy", "invalid arguments - object to copy not found"))
                            }
                        }
                    },
                    _ => {
                        Err(SError::obj(pid, &doc, "shallowCopy", "invalid arguments - object to copy not found"))
                    }
                }
            },
            // Make this object a deep copy of the referenced object (fields only).
            // Captures attributes, deep copies sub objects, etc.. as well.
            // Signature: Object.deepCopyFields(obj, to_copy: obj): void
            "deepCopyFields" => {
                if parameters.len() < 1 {
                    return Err(SError::obj(pid, &doc, "deepCopyFields", "invalid arguments - object to copy not found"));
                }
                match &parameters[0] {
                    SVal::Object(to_copy) => {
                        let mut fields = Vec::new();
                        for field in SField::fields(&doc.graph, to_copy) {
                            fields.push(field.clone());
                        }
                        for field in fields {
                            if field.is_object() {
                                match field.value.clone().unbox() {
                                    SVal::Object(nref) => {
                                        let deep_copy = SField::new_object(&mut doc.graph, &field.name, obj);
                                        let to_copy = SVal::Object(nref);
                                        self.operate(pid, doc, "deepCopyFields", &deep_copy, &mut vec![to_copy])?;
                                    },
                                    _ => {}
                                }
                            } else {
                                SData::insert_new(&mut doc.graph, obj, Box::new(field));
                            }
                        }
                        Ok(SVal::Void)
                    },
                    SVal::Boxed(val) => {
                        let val = val.lock().unwrap();
                        let val = val.deref();
                        match val {
                            SVal::Object(to_copy) => {
                                let mut fields = Vec::new();
                                for field in SField::fields(&doc.graph, to_copy) {
                                    fields.push(field.clone());
                                }
                                for field in fields {
                                    if field.is_object() {
                                        match field.value.clone().unbox() {
                                            SVal::Object(nref) => {
                                                let deep_copy = SField::new_object(&mut doc.graph, &field.name, obj);
                                                let to_copy = SVal::Object(nref);
                                                self.operate(pid, doc, "deepCopyFields", &deep_copy, &mut vec![to_copy])?;
                                            },
                                            _ => {}
                                        }
                                    } else {
                                        SData::insert_new(&mut doc.graph, obj, Box::new(field));
                                    }
                                }
                                Ok(SVal::Void)
                            },
                            _ => {
                                Err(SError::obj(pid, &doc, "deepCopyFields", "invalid arguments - object to copy not found"))
                            }
                        }
                    },
                    _ => {
                        Err(SError::obj(pid, &doc, "deepCopyFields", "invalid arguments - object to copy not found"))
                    }
                }
            },
            _ => {
                Err(SError::obj(pid, &doc, "NotFound", &format!("{} is not a function in the Object Library", name)))
            }
        }
    }
}
impl Library for ObjectLibrary {
    fn scope(&self) -> String {
        "Object".into()
    }
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
                            return Err(SError::obj(pid, &doc, "InvalidArgument", "object argument not found"));
                        }
                    }
                },
                _ => {
                    return Err(SError::obj(pid, &doc, "InvalidArgument", "object argument not found"));
                }
            }
        } else {
            return Err(SError::obj(pid, &doc, "InvalidArgument", "object argument not found"));
        }
    }
}
