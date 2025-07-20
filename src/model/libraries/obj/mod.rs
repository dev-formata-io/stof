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
use imbl::Vector;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use crate::{model::{Graph, Prototype}, runtime::{instruction::{Instruction, Instructions}, proc::ProcEnv, Error, Type, Val, ValRef, Variable}};


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
    MoveData,
    Fields,
    Funcs,
    Empty,
    Any,
    Attributes,
    Move,
    Distance,
    Drop,

    Run,
    Schemafy,

    SearchDown,
    SearchUp,

    ToMap,
    FromMap,

    ShallowCopy,
    DeepCopy,

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
                        let mut prototypes = Prototype::prototype_refs(&graph, &obj);
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
        }
    }
}
