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

use std::{ops::Deref, sync::Arc};
use arcstr::{literal, ArcStr};
use imbl::{vector, OrdMap, Vector};
use lazy_static::lazy_static;
use nanoid::nanoid;
use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};
use crate::{model::{obj::validate::validation, stof_std::StdIns, Field, Func, Graph, Prototype, SId}, runtime::{instruction::{Instruction, Instructions}, instructions::{call::{FuncCall, NamedArg}, empty::EmptyIns, Base, ConsumeStack, POP_SELF, PUSH_SELF, TRUTHY}, proc::ProcEnv, Error, Num, Type, Val, ValRef, Variable}};
mod validate;


/// Obj library name.
pub(self) const OBJ_LIB: ArcStr = literal!("Obj");


/// Add the obj library to a graph.
pub fn insert_obj_lib(graph: &mut Graph) {

}


lazy_static! {
    pub(self) static ref NAME: Arc<dyn Instruction> = Arc::new(ObjIns::Name);
    pub(self) static ref ID: Arc<dyn Instruction> = Arc::new(ObjIns::Id);
    pub(self) static ref PATH: Arc<dyn Instruction> = Arc::new(ObjIns::Path);
    pub(self) static ref PARENT: Arc<dyn Instruction> = Arc::new(ObjIns::Parent);
    pub(self) static ref IS_PARENT: Arc<dyn Instruction> = Arc::new(ObjIns::IsParent);
    pub(self) static ref EXISTS: Arc<dyn Instruction> = Arc::new(ObjIns::Exists);
    pub(self) static ref CHILDREN: Arc<dyn Instruction> = Arc::new(ObjIns::Children);

    pub(self) static ref ROOT: Arc<dyn Instruction> = Arc::new(ObjIns::Root);
    pub(self) static ref IS_ROOT: Arc<dyn Instruction> = Arc::new(ObjIns::IsRoot);

    pub(self) static ref PROTO: Arc<dyn Instruction> = Arc::new(ObjIns::Proto);
    pub(self) static ref SET_PROTO: Arc<dyn Instruction> = Arc::new(ObjIns::SetProto);
    pub(self) static ref REMOVE_PROTO: Arc<dyn Instruction> = Arc::new(ObjIns::RemoveProto);
    pub(self) static ref INSTANCE_OF: Arc<dyn Instruction> = Arc::new(ObjIns::InstanceOf);
    pub(self) static ref UPCAST: Arc<dyn Instruction> = Arc::new(ObjIns::Upcast);
    pub(self) static ref CREATE_TYPE: Arc<dyn Instruction> = Arc::new(ObjIns::CreateType);

    pub(self) static ref LEN: Arc<dyn Instruction> = Arc::new(ObjIns::Len);
    pub(self) static ref AT: Arc<dyn Instruction> = Arc::new(ObjIns::At);
    pub(self) static ref AT_REF: Arc<dyn Instruction> = Arc::new(ObjIns::AtRef);
    pub(self) static ref GET: Arc<dyn Instruction> = Arc::new(ObjIns::Get);
    pub(self) static ref GET_REF: Arc<dyn Instruction> = Arc::new(ObjIns::GetRef);
    pub(self) static ref CONTAINS: Arc<dyn Instruction> = Arc::new(ObjIns::Contains);
    pub(self) static ref INSERT: Arc<dyn Instruction> = Arc::new(ObjIns::Insert);
    pub(self) static ref REMOVE: Arc<dyn Instruction> = Arc::new(ObjIns::Remove);
    pub(self) static ref MOVE_FIELD: Arc<dyn Instruction> = Arc::new(ObjIns::MoveField);
    pub(self) static ref FIELDS: Arc<dyn Instruction> = Arc::new(ObjIns::Fields);
    pub(self) static ref FUNCS: Arc<dyn Instruction> = Arc::new(ObjIns::Funcs);
    pub(self) static ref EMPTY: Arc<dyn Instruction> = Arc::new(ObjIns::Empty);
    pub(self) static ref ANY: Arc<dyn Instruction> = Arc::new(ObjIns::Any);
    pub(self) static ref ATTRIBUTES: Arc<dyn Instruction> = Arc::new(ObjIns::Attributes);
    pub(self) static ref MOVE: Arc<dyn Instruction> = Arc::new(ObjIns::Move);
    pub(self) static ref DISTANCE: Arc<dyn Instruction> = Arc::new(ObjIns::Distance);

    pub(self) static ref RUN: Arc<dyn Instruction> = Arc::new(ObjIns::Run);
    pub(self) static ref SCHEMAFY: Arc<dyn Instruction> = Arc::new(ObjIns::Schemafy);
    pub(self) static ref TO_MAP: Arc<dyn Instruction> = Arc::new(ObjIns::ToMap);
    pub(self) static ref TO_MAP_REF: Arc<dyn Instruction> = Arc::new(ObjIns::ToMapRef);
    pub(self) static ref FROM_MAP: Arc<dyn Instruction> = Arc::new(ObjIns::FromMap);
    pub(self) static ref FROM_ID: Arc<dyn Instruction> = Arc::new(ObjIns::FromId);
    pub(self) static ref DUMP: Arc<dyn Instruction> = Arc::new(ObjIns::Dump);
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Object instructions.
pub enum ObjIns {
    Name,
    Id,
    Path,
    Parent,
    IsParent,
    Exists,
    Children,

    Root,
    IsRoot,

    Proto,
    SetProto,
    RemoveProto,
    InstanceOf,
    Upcast,
    CreateType,

    Len,
    At,
    AtRef,
    Get,
    GetRef,
    Contains,
    Insert,
    Remove,
    MoveField,
    Fields,
    Funcs,
    Empty,
    Any,
    Attributes,
    Move,
    Distance,

    Run,
    Schemafy,

    ToMap,
    ToMapRef,
    FromMap,

    FromId,
    Dump,
}
#[typetag::serde(name = "ObjIns")]
impl Instruction for ObjIns {
    fn exec(&self, env: &mut ProcEnv, graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        match self {
            Self::Name => {
                if let Some(var) = env.stack.pop() {
                    if let Some(obj) = var.try_obj() {
                        if let Some(name) = obj.node_name(&graph) {
                            env.stack.push(Variable::val(Val::Str(name.as_ref().into())));
                            return Ok(None);
                        }
                    }
                }
                Err(Error::ObjName)
            },
            Self::Id => {
                if let Some(var) = env.stack.pop() {
                    if let Some(obj) = var.try_obj() {
                        env.stack.push(Variable::val(Val::Str(obj.as_ref().into())));
                        return Ok(None);
                    }
                }
                Err(Error::ObjId)
            },
            Self::Path => {
                if let Some(var) = env.stack.pop() {
                    if let Some(obj) = var.try_obj() {
                        if let Some(path) = obj.node_path(&graph, true) {
                            env.stack.push(Variable::val(Val::Str(path.join(".").into())));
                            return Ok(None);
                        }
                    }
                }
                Err(Error::ObjPath)
            },
            Self::Parent => {
                if let Some(var) = env.stack.pop() {
                    if let Some(obj) = var.try_obj() {
                        if let Some(parent) = obj.node_parent(&graph) {
                            env.stack.push(Variable::val(Val::Obj(parent)));
                            return Ok(None);
                        }
                    }
                }
                Err(Error::ObjParent)
            },
            Self::IsParent => {
                if let Some(child_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        if let Some(child) = child_var.try_obj() {
                            if let Some(obj) = var.try_obj() {
                                let parent = child.child_of(&graph, &obj) && child != obj;
                                env.stack.push(Variable::val(Val::Bool(parent)));
                                return Ok(None);
                            }
                        }
                    }
                }
                Err(Error::ObjIsParent)
            },
            Self::Exists => {
                if let Some(var) = env.stack.pop() {
                    if let Some(obj) = var.try_obj() {
                        env.stack.push(Variable::val(Val::Bool(obj.node_exists(&graph))));
                        return Ok(None);
                    }
                }
                Err(Error::ObjExists)
            },
            Self::Children => {
                if let Some(var) = env.stack.pop() {
                    if let Some(obj) = var.try_obj() {
                        if let Some(node) = obj.node(&graph) {
                            let children = node.children
                                .iter()
                                .map(|nref| ValRef::new(Val::Obj(nref.clone())))
                                .collect::<Vector<_>>();
                            env.stack.push(Variable::val(Val::List(children)));
                            return Ok(None);
                        }
                    }
                }
                Err(Error::ObjChildren)
            },

            Self::Root => {
                if let Some(var) = env.stack.pop() {
                    if let Some(obj) = var.try_obj() {
                        if let Some(root) = obj.root(&graph) {
                            env.stack.push(Variable::val(Val::Obj(root)));
                            return Ok(None);
                        }
                    }
                }
                Err(Error::ObjRoot)
            },
            Self::IsRoot => {
                if let Some(var) = env.stack.pop() {
                    if let Some(obj) = var.try_obj() {
                        let is_root = obj.is_root(&graph);
                        env.stack.push(Variable::val(Val::Bool(is_root)));
                        return Ok(None);
                    }
                }
                Err(Error::ObjIsRoot)
            },

            Self::Proto => {
                if let Some(var) = env.stack.pop() {
                    if let Some(obj) = var.try_obj() {
                        let mut proto_nrefs = Prototype::prototype_nodes(graph, &obj);
                        if proto_nrefs.len() == 1 {
                            env.stack.push(Variable::val(Val::Obj(proto_nrefs.pop().unwrap())));
                        } else if proto_nrefs.len() > 1 {
                            let protos = proto_nrefs.into_iter()
                                .map(|nref| ValRef::new(Val::Obj(nref)))
                                .collect::<Vector<_>>();
                            env.stack.push(Variable::val(Val::List(protos)));
                        } else {
                            env.stack.push(Variable::val(Val::Null));
                        }
                        return Ok(None);
                    }
                }
                Err(Error::ObjProto)
            },
            Self::CreateType => {
                if let Some(typename_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        if let Some(obj) = var.try_obj() {
                            match typename_var.val.read().deref() {
                                Val::Str(typename) => {
                                    // If this object is deleted, the type will be too
                                    graph.insert_type(typename.as_str(), &obj);
                                    return Ok(None);
                                },
                                _ => {}
                            }
                        }
                    }
                }
                Err(Error::ObjCreateType)
            },
            Self::Upcast => {
                if let Some(var) = env.stack.pop() {
                    if let Some(obj) = var.try_obj() {
                        let prototypes = Prototype::prototype_refs(&graph, &obj);
                        if prototypes.len() > 0 {
                            // Remove all prototypes and look for a proto on the proto to set
                            let mut proto_obj = None;
                            for dref in &prototypes {
                                if let Some(proto) = graph.get_stof_data::<Prototype>(dref) {
                                    proto_obj = Some(proto.node.clone());
                                }
                                graph.remove_data(dref, Some(obj.clone()));
                            }
                            
                            let mut set_proto = false;
                            if let Some(proto) = proto_obj {
                                for proto_proto in Prototype::prototype_refs(&graph, &proto) {
                                    graph.insert_stof_data(&obj, "__proto__", Box::new(Prototype { node: proto_proto }), None);
                                    set_proto = true;
                                    break;
                                }
                            }
                            env.stack.push(Variable::val(Val::Bool(set_proto)));
                        } else {
                            env.stack.push(Variable::val(Val::Bool(false)));
                        }
                        return Ok(None);
                    }
                }
                Err(Error::ObjUpcast)
            },
            Self::SetProto => {
                if let Some(proto_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        if let Some(proto_ref) = proto_var.try_obj() {
                            if let Some(obj) = var.try_obj() {
                                let existing_prototypes = Prototype::prototype_refs(graph, &obj);
                                for dref in existing_prototypes { graph.remove_data(&dref, Some(obj.clone())); }
                                graph.insert_stof_data(&obj, "__proto__", Box::new(Prototype { node: proto_ref }), None);
                                return Ok(None);
                            }
                        }
                    }
                }
                Err(Error::ObjSetProto)
            },
            Self::RemoveProto => {
                if let Some(var) = env.stack.pop() {
                    if let Some(obj) = var.try_obj() {
                        for dref in Prototype::prototype_refs(&graph, &obj) {
                            graph.remove_data(&dref, Some(obj.clone()));
                        }
                        return Ok(None);
                    }
                }
                Err(Error::ObjRemoveProto)
            },
            Self::InstanceOf => {
                if let Some(instance_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        if let Some(obj) = var.try_obj() {
                            match instance_var.val.read().deref() {
                                Val::Str(typename) => {
                                    let mut otype = Type::Obj(typename.as_str().into());
                                    otype.obj_to_proto(&graph, Some(env.self_ptr())); // takes care of self, paths, etc.
                                    match otype {
                                        Type::Obj(proto_id) => {
                                            let val = Val::Obj(obj);
                                            let instance_of = val.instance_of(&proto_id, &graph)?;
                                            env.stack.push(Variable::val(Val::Bool(instance_of)));
                                            return Ok(None);
                                        },
                                        _ => {}
                                    }
                                },
                                Val::Obj(proto_id) => {
                                    let val = Val::Obj(obj);
                                    let instance_of = val.instance_of(proto_id, &graph)?;
                                    env.stack.push(Variable::val(Val::Bool(instance_of)));
                                    return Ok(None);
                                },
                                _ => {}
                            }
                        }
                    }
                }
                Err(Error::ObjInstanceOf)
            },

            Self::Len => {
                if let Some(var) = env.stack.pop() {
                    if let Some(obj) = var.try_obj() {
                        let fields = Field::fields(graph, &obj);
                        env.stack.push(Variable::val(Val::Num(Num::Int(fields.len() as i64))));
                        return Ok(None);
                    }
                }
                Err(Error::ObjLen)
            },
            Self::At => {
                if let Some(index_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        if let Some(obj) = var.try_obj() {
                            match index_var.val.read().deref() {
                                Val::Num(num) => {
                                    let index = num.int() as usize;
                                    if let Some((name, field_ref)) = Field::fields(graph, &obj).into_iter().nth(index) {
                                        if let Some(field) = graph.get_stof_data::<Field>(&field_ref) {
                                            env.stack.push(Variable::val(Val::Tup(vector![ValRef::new(Val::Str(name.into())), field.value.val.duplicate(false)])));
                                        } else {
                                            env.stack.push(Variable::val(Val::Null));
                                        }
                                    } else {
                                        env.stack.push(Variable::val(Val::Null));
                                    }
                                    return Ok(None);
                                },
                                _ => {}
                            }
                        }
                    }
                }
                Err(Error::ObjAt)
            },
            Self::AtRef => {
                if let Some(index_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        if let Some(obj) = var.try_obj() {
                            match index_var.val.read().deref() {
                                Val::Num(num) => {
                                    let index = num.int() as usize;
                                    if let Some((name, field_ref)) = Field::fields(graph, &obj).into_iter().nth(index) {
                                        if let Some(field) = graph.get_stof_data::<Field>(&field_ref) {
                                            env.stack.push(Variable::val(Val::Tup(vector![ValRef::new(Val::Str(name.into())), field.value.val.duplicate(true)])));
                                        } else {
                                            env.stack.push(Variable::val(Val::Null));
                                        }
                                    } else {
                                        env.stack.push(Variable::val(Val::Null));
                                    }
                                    return Ok(None);
                                },
                                _ => {}
                            }
                        }
                    }
                }
                Err(Error::ObjAtRef)
            },
            Self::Get => {
                if let Some(get_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        if let Some(obj) = var.try_obj() {
                            match get_var.val.read().deref() {
                                Val::Str(name) => {
                                    if let Some(node) = obj.node(&graph) {
                                        if let Some(dref) = node.get_data(name.as_str()) {
                                            if let Some(field) = graph.get_stof_data::<Field>(dref) {
                                                env.stack.push(field.value.stack_var(false));
                                            } else if let Some(_func) = graph.get_stof_data::<Func>(dref) {
                                                env.stack.push(Variable::val(Val::Fn(dref.clone())));
                                            } else {
                                                env.stack.push(Variable::val(Val::Data(dref.clone())));
                                            }
                                        } else {
                                            env.stack.push(Variable::val(Val::Null));
                                        }
                                        return Ok(None);
                                    }
                                },
                                _ => {}
                            }
                        }
                    }
                }
                Err(Error::ObjGet)
            },
            Self::GetRef => {
                if let Some(get_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        if let Some(obj) = var.try_obj() {
                            match get_var.val.read().deref() {
                                Val::Str(name) => {
                                    if let Some(node) = obj.node(&graph) {
                                        if let Some(dref) = node.get_data(name.as_str()) {
                                            if let Some(field) = graph.get_stof_data::<Field>(dref) {
                                                env.stack.push(field.value.stack_var(true));
                                            } else if let Some(_func) = graph.get_stof_data::<Func>(dref) {
                                                env.stack.push(Variable::val(Val::Fn(dref.clone())));
                                            } else {
                                                env.stack.push(Variable::val(Val::Data(dref.clone())));
                                            }
                                        } else {
                                            env.stack.push(Variable::val(Val::Null));
                                        }
                                        return Ok(None);
                                    }
                                },
                                _ => {}
                            }
                        }
                    }
                }
                Err(Error::ObjGetRef)
            },
            Self::Contains => {
                if let Some(search_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        if let Some(obj) = var.try_obj() {
                            match search_var.val.read().deref() {
                                Val::Str(name) => {
                                    if let Some(node) = obj.node(&graph) {
                                        let contains = node.data.contains_key(name.as_str());
                                        env.stack.push(Variable::val(Val::Bool(contains)));
                                        return Ok(None);
                                    }
                                },
                                _ => {}
                            }
                        }
                    }
                }
                Err(Error::ObjContains)
            },
            Self::Insert => {
                if let Some(value_var) = env.stack.pop() {
                    if let Some(name_var) = env.stack.pop() {
                        if let Some(var) = env.stack.pop() {
                            if let Some(obj) = var.try_obj() {
                                match name_var.val.read().deref() {
                                    Val::Str(name) => {
                                        let mut name = name.clone();
                                        if !name.starts_with("self.") {
                                            // Make sure the path starts with this object
                                            name = format!("self.{name}").into();
                                        }

                                        let mut instructions = Instructions::default();
                                        instructions.push(Arc::new(Base::Literal(Val::Obj(obj))));
                                        instructions.push(PUSH_SELF.clone());
                                        instructions.push(Arc::new(Base::Variable(value_var)));
                                        instructions.push(Arc::new(Base::SetVariable(name)));
                                        instructions.push(POP_SELF.clone());
                                        return Ok(Some(instructions));
                                    },
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                Err(Error::ObjInsert)
            },
            Self::Remove => {
                if let Some(name_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        if let Some(obj) = var.try_obj() {
                            match name_var.val.read().deref() {
                                Val::Str(name) => {
                                    let mut name = name.clone();
                                    if !name.starts_with("self.") {
                                        // Make sure the path starts with this object
                                        name = format!("self.{name}").into();
                                    }

                                    let mut instructions = Instructions::default();
                                    instructions.push(Arc::new(Base::Literal(Val::Obj(obj))));
                                    instructions.push(PUSH_SELF.clone());
                                    instructions.push(Arc::new(Base::Literal(Val::Str(name))));
                                    instructions.push(Arc::new(StdIns::Drop(1)));
                                    instructions.push(POP_SELF.clone());
                                    return Ok(Some(instructions));
                                },
                                _ => {}
                            }
                        }
                    }
                }
                Err(Error::ObjRemove)
            },
            Self::MoveField => {
                if let Some(dest_var) = env.stack.pop() {
                    if let Some(source_var) = env.stack.pop() {
                        if let Some(var) = env.stack.pop() {
                            if let Some(obj) = var.try_obj() {
                                match dest_var.val.read().deref() {
                                    Val::Str(dest_path) => {
                                        match source_var.val.read().deref() {
                                            Val::Str(source_path) => {
                                                let mut moved = false;
                                                if let Some(source_field_ref) = Field::field_from_path(graph, source_path.as_str(), Some(obj.clone())) {
                                                    let mut source_field = None;
                                                    if let Some(field) = graph.get_stof_data::<Field>(&source_field_ref) {
                                                        source_field = Some(field.clone());
                                                    }
                                                    if let Some(field) = source_field {
                                                        if let Some(existing) = Field::field_from_path(graph, dest_path.as_str(), Some(obj.clone())) {
                                                            // A field already exists at this destination - set value
                                                            if let Some(dest_field) = graph.get_mut_stof_data::<Field>(&existing) {
                                                                dest_field.value = field.value;
                                                                moved = true;
                                                                graph.remove_data(&source_field_ref, None);
                                                            }
                                                        } else {
                                                            let mut dest_path = dest_path.split('.').collect::<Vec<_>>();
                                                            let new_field_name = dest_path.pop().unwrap();

                                                            if dest_path.len() > 0 {
                                                                // new location
                                                                let path = dest_path.join(".");
                                                                if let Some(location) = graph.ensure_named_nodes(&path, Some(obj.clone()), true, None) {
                                                                    graph.remove_data(&source_field_ref, None);
                                                                    moved = true;
                                                                    graph.insert_stof_data(&location, new_field_name, Box::new(field), Some(source_field_ref));
                                                                }
                                                            } else {
                                                                // rename only
                                                                moved = graph.rename_data(&source_field_ref, new_field_name);
                                                                if let Some(obj) = field.value.try_obj() {
                                                                    if let Some(node) = obj.node_mut(graph) {
                                                                        node.name = new_field_name.into();
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                                env.stack.push(Variable::val(Val::Bool(moved)));
                                                return Ok(None);
                                            },
                                            _ => {}
                                        }
                                    },
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                Err(Error::ObjMoveField)
            },
            Self::Fields => {
                if let Some(var) = env.stack.pop() {
                    if let Some(obj) = var.try_obj() {
                        let mut fields = Vector::default();
                        for (name, field_ref) in Field::fields(graph, &obj) {
                            if let Some(field) = graph.get_stof_data::<Field>(&field_ref) {
                                let value = field.value.val.duplicate(false);
                                fields.push_back(ValRef::new(Val::Tup(vector![ValRef::new(Val::Str(name.into())), value])));
                            }
                        }
                        env.stack.push(Variable::val(Val::List(fields)));
                        return Ok(None);
                    }
                }
                Err(Error::ObjFields)
            },
            Self::Funcs => {
                if let Some(var) = env.stack.pop() {
                    if let Some(obj) = var.try_obj() {
                        let funcs = Func::functions(&graph, &obj, &None, false)
                            .into_iter()
                            .map(|fref| ValRef::new(Val::Fn(fref)))
                            .collect::<Vector<_>>();
                        env.stack.push(Variable::val(Val::List(funcs)));
                        return Ok(None);
                    }
                }
                Err(Error::ObjFuncs)
            },
            Self::Empty => {
                if let Some(var) = env.stack.pop() {
                    if let Some(obj) = var.try_obj() {
                        if let Some(node) = obj.node(&graph) {
                            env.stack.push(Variable::val(Val::Bool(node.data.is_empty())));
                            return Ok(None);
                        }
                    }
                }
                Err(Error::ObjEmpty)
            },
            Self::Any => {
                if let Some(var) = env.stack.pop() {
                    if let Some(obj) = var.try_obj() {
                        if let Some(node) = obj.node(&graph) {
                            env.stack.push(Variable::val(Val::Bool(!node.data.is_empty())));
                            return Ok(None);
                        }
                    }
                }
                Err(Error::ObjAny)
            },
            Self::Attributes => {
                if let Some(var) = env.stack.pop() {
                    if let Some(obj) = var.try_obj() {
                        if let Some(node) = obj.node(&graph) {
                            let attributes = node.attributes.clone()
                                .into_iter()
                                .map(|(name, val)| ValRef::new(Val::Tup(vector![ValRef::new(Val::Str(name.into())), ValRef::new(val)])))
                                .collect::<Vector<_>>();
                            env.stack.push(Variable::val(Val::List(attributes)));
                            return Ok(None);
                        }
                    }
                }
                Err(Error::ObjAttributes)
            },
            Self::Move => {
                if let Some(dest_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        if let Some(obj) = var.try_obj() {
                            if let Some(dest) = dest_var.try_obj() {
                                let moved = graph.move_node(&obj, &dest);
                                env.stack.push(Variable::val(Val::Bool(moved)));
                                return Ok(None);
                            }
                        }
                    }
                }
                Err(Error::ObjMove)
            },
            Self::Distance => {
                if let Some(first_var) = env.stack.pop() {
                    if let Some(second_var) = env.stack.pop() {
                        if let Some(first) = first_var.try_obj() {
                            if let Some(second) = second_var.try_obj() {
                                let distance = first.distance_to(&graph, &second);
                                env.stack.push(Variable::val(Val::Num(Num::Int(distance as i64))));
                                return Ok(None);
                            }
                        }
                    }
                }
                Err(Error::ObjDistance)
            },

            Self::Run => {
                if let Some(var) = env.stack.pop() {
                    if let Some(obj) = var.try_obj() {
                        let mut run_instructions = Vec::new();

                        for (_, field_ref) in Field::fields(graph, &obj) {
                            if let Some(field) = graph.get_stof_data::<Field>(&field_ref) {
                                if let Some(attr) = field.attributes.get("run") {
                                    let mut order = -2;
                                    let mut args = None;
                                    match attr {
                                        Val::Num(num) => {
                                            order = num.int();
                                        },
                                        Val::Map(map) => {
                                            if let Some(ord) = map.get(&ValRef::new(Val::Str("order".into()))) {
                                                match ord.read().deref() {
                                                    Val::Num(num) => {
                                                        order = num.int();
                                                    },
                                                    _ => {}
                                                }
                                            }
                                            if let Some(arguments) = map.get(&ValRef::new(Val::Str("args".into()))) {
                                                args = Some(arguments.clone());
                                            }
                                        },
                                        _ => {}
                                    }
                                    match field.value.val.read().deref() {
                                        Val::Obj(other) => {
                                            let instructions = vector![
                                                Arc::new(Base::Literal(Val::Obj(other.clone()))) as Arc<dyn Instruction>,
                                                RUN.clone(),
                                            ];
                                            run_instructions.push((order, instructions));
                                        },
                                        Val::Fn(dref) => {
                                            let mut arguments: Vector<Arc<dyn Instruction>> = vector![];
                                            if let Some(args) = args {
                                                match args.read().deref() {
                                                    Val::List(args) => {
                                                        for arg in args {
                                                            arguments.push_back(Arc::new(Base::Variable(Variable::refval(arg.clone()))));
                                                        }
                                                    },
                                                    _ => {}
                                                }
                                            }
                                            let instructions = vector![
                                                Arc::new(EmptyIns {
                                                    ins: Arc::new(FuncCall { func: Some(dref.clone()), search: None, stack: false, as_ref: false, args: arguments })
                                                }) as Arc<dyn Instruction>
                                            ];
                                            run_instructions.push((order, instructions));
                                        },
                                        Val::List(list) => {
                                            let mut instructions = vector![];

                                            let mut arguments: Vector<Arc<dyn Instruction>> = vector![];
                                            if let Some(args) = args {
                                                match args.read().deref() {
                                                    Val::List(args) => {
                                                        for arg in args {
                                                            arguments.push_back(Arc::new(Base::Variable(Variable::refval(arg.clone()))));
                                                        }
                                                    },
                                                    _ => {}
                                                }
                                            }

                                            for val in list {
                                                match val.read().deref() {
                                                    Val::Obj(other) => {
                                                        instructions.push_back(Arc::new(Base::Literal(Val::Obj(other.clone()))) as Arc<dyn Instruction>);
                                                        instructions.push_back(RUN.clone());
                                                    },
                                                    Val::Fn(dref) => {
                                                        instructions.push_back(Arc::new(EmptyIns {
                                                                ins: Arc::new(FuncCall { func: Some(dref.clone()), search: None, stack: false, as_ref: false, args: arguments.clone() })
                                                            }) as Arc<dyn Instruction>);
                                                    },
                                                    _ => {}
                                                }
                                            }
                                            run_instructions.push((order, instructions));
                                        },
                                        _ => {}
                                    }
                                }
                            }
                        }

                        for func_ref in Func::functions(&graph, &obj, &None, false) {
                            if let Some(func) = graph.get_stof_data::<Func>(&func_ref) {
                                if let Some(attr) = func.attributes.get("run") {
                                    let mut order = -1;
                                    let mut args = None;
                                    match attr {
                                        Val::Num(num) => {
                                            order = num.int();
                                        },
                                        Val::Map(map) => {
                                            if let Some(ord) = map.get(&ValRef::new(Val::Str("order".into()))) {
                                                match ord.read().deref() {
                                                    Val::Num(num) => {
                                                        order = num.int();
                                                    },
                                                    _ => {}
                                                }
                                            }
                                            if let Some(arguments) = map.get(&ValRef::new(Val::Str("args".into()))) {
                                                args = Some(arguments.clone());
                                            }
                                        },
                                        _ => {}
                                    }

                                    let mut arguments: Vector<Arc<dyn Instruction>> = vector![];
                                    if let Some(args) = args {
                                        match args.read().deref() {
                                            Val::List(args) => {
                                                for arg in args {
                                                    arguments.push_back(Arc::new(Base::Variable(Variable::refval(arg.clone()))));
                                                }
                                            },
                                            _ => {}
                                        }
                                    }

                                    let instructions = vector![
                                        Arc::new(EmptyIns {
                                            ins: Arc::new(FuncCall { func: Some(func_ref), search: None, stack: false, as_ref: false, args: arguments })
                                        }) as Arc<dyn Instruction>
                                    ];
                                    run_instructions.push((order, instructions));
                                }
                            }
                        }

                        let mut highest_order = -2;
                        for (ord, _) in &run_instructions { if *ord > highest_order { highest_order = *ord; } }
                        for nref in Prototype::prototype_nodes(&graph, &obj) {
                            let instructions = vector![
                                Arc::new(Base::Literal(Val::Obj(nref))) as Arc<dyn Instruction>,
                                RUN.clone()
                            ];
                            highest_order += 1;
                            run_instructions.push((highest_order, instructions));
                        }

                        run_instructions.sort_by(|a, b| a.0.cmp(&b.0));
                        let mut instructions = Instructions::default();
                        for (_, ins) in run_instructions {
                            instructions.append(&ins);
                        }

                        return Ok(Some(instructions));
                    }
                }
                Err(Error::ObjRun)
            },
            Self::Schemafy => {
                // Obj.schemafy(schema: obj, target: obj, remove_invalid: bool, remove_undefined: bool) -> bool;
                if let Some(remove_undefined_var) = env.stack.pop() {
                    if let Some(remove_invalid_fields) = env.stack.pop() {
                        if let Some(target_var) = env.stack.pop() {
                            if let Some(schema_var) = env.stack.pop() {
                                if let Some(schema) = schema_var.try_obj() {
                                    if let Some(target) = target_var.try_obj() {
                                        let remove_undefined = remove_undefined_var.truthy();
                                        let remove_invalid = remove_invalid_fields.truthy();

                                        let mut validation_instructions: Vec<Vector<Arc<dyn Instruction>>> = Vec::new();
                                        let mut defined_field_names = FxHashSet::default();
                                        for (schema_field_name, schema_field_ref) in Field::fields(graph, &schema) {
                                            defined_field_names.insert(schema_field_name.clone());

                                            let mut schema_attr_val = None;
                                            let mut schema_field_val = None;
                                            if let Some(field) = graph.get_stof_data::<Field>(&schema_field_ref) {
                                                if let Some(attr) = field.attributes.get("schema") {
                                                    schema_attr_val = Some(attr.clone());
                                                    schema_field_val = Some(field.value.val.duplicate(false));
                                                }
                                            }
                                            if let Some(schema_val) = schema_field_val {
                                                if let Some(validate) = schema_attr_val {
                                                    if let Some(target_field_ref) = Field::direct_field(&graph, &target, &schema_field_name) {
                                                        let mut target_val = None;
                                                        if let Some(field) = graph.get_stof_data::<Field>(&target_field_ref) {
                                                            target_val = Some(field.value.val.clone()); // reference to the value outright
                                                        }
                                                        if let Some(target_val) = target_val {
                                                            let mut field_instructions = validation(graph, &schema, &target, schema_field_name, validate, schema_val, target_val, remove_invalid, remove_undefined);
                                                            validation_instructions.append(&mut field_instructions);
                                                        }
                                                    }
                                                }
                                            }
                                        }

                                        if remove_undefined {
                                            for (field_name, field_ref) in Field::fields(graph, &target) {
                                                if !defined_field_names.contains(&field_name) {
                                                    graph.remove_data(&field_ref, Some(target.clone()));
                                                }
                                            }
                                        }
                                        
                                        let mut instructions = Instructions::default();
                                        if validation_instructions.is_empty() {
                                            instructions.push(Arc::new(Base::Literal(Val::Bool(true))));
                                        } else if validation_instructions.len() == 1 {
                                            instructions.append(&validation_instructions[0]);
                                            instructions.push(TRUTHY.clone());
                                        } else {
                                            let end_tag: ArcStr = nanoid!(12).into();
                                            instructions.append(&validation_instructions[0]);
                                            instructions.push(TRUTHY.clone());
                                            for ins in validation_instructions.iter().skip(1) {
                                                instructions.push(Arc::new(Base::CtrlForwardToIfNotTruthy(end_tag.clone(), ConsumeStack::IfTrue)));
                                                instructions.append(ins);
                                                instructions.push(TRUTHY.clone());
                                            }
                                            instructions.push(Arc::new(Base::Tag(end_tag)));
                                        }
                                        return Ok(Some(instructions));
                                    }
                                }
                            }
                        }
                    }
                }
                Err(Error::ObjSchemafy)
            },

            Self::ToMap => {
                if let Some(var) = env.stack.pop() {
                    if let Some(obj) = var.try_obj() {
                        let mut map = OrdMap::default();
                        for (name, fref) in Field::fields(graph, &obj) {
                            if let Some(field) = graph.get_stof_data::<Field>(&fref) {
                                map.insert(ValRef::new(Val::Str(name.into())), field.value.val.duplicate(false));
                            }
                        }
                        env.stack.push(Variable::val(Val::Map(map)));
                        return Ok(None);
                    }
                }
                Err(Error::ObjToMap)
            },
            Self::ToMapRef => {
                if let Some(var) = env.stack.pop() {
                    if let Some(obj) = var.try_obj() {
                        let mut map = OrdMap::default();
                        for (name, fref) in Field::fields(graph, &obj) {
                            if let Some(field) = graph.get_stof_data::<Field>(&fref) {
                                map.insert(ValRef::new(Val::Str(name.into())), field.value.val.duplicate(true));
                            }
                        }
                        env.stack.push(Variable::val(Val::Map(map)));
                        return Ok(None);
                    }
                }
                Err(Error::ObjToMapRef)
            },
            Self::FromMap => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Map(map) => {
                            let obj = graph.insert_node(&nanoid!(10), Some(env.self_ptr()), false);
                            for (k, v) in map {
                                match k.read().deref() {
                                    Val::Str(name) => {
                                        let field = Field::new(Variable::refval(v.duplicate(false)), None);
                                        graph.insert_stof_data(&obj, name.as_str(), Box::new(field), None);
                                    },
                                    _ => {}
                                }
                            }
                            env.stack.push(Variable::val(Val::Obj(obj)));
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::ObjFromMap)
            },

            Self::FromId => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Str(id) => {
                            env.stack.push(Variable::val(Val::Obj(SId::from(id.as_str()))));
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::ObjFromId)
            },
            Self::Dump => {
                graph.dump(true);
                Ok(None)
            }
        }
    }
}
