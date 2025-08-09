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
use crate::{model::{libraries::data::ops::{data_attach, data_drop, data_drop_from, data_exists, data_from_field, data_from_id, data_id, data_libname, data_move, data_objs}, Field, Graph, SId, SELF_STR_KEYWORD, SUPER_STR_KEYWORD}, runtime::{instruction::{Instruction, Instructions}, proc::ProcEnv, Error, Val, ValRef, Variable}};
mod ops;


/// Data lib.
pub(self) const DATA_LIB: ArcStr = literal!("Data");


/// Add the data lib to a graph.
pub fn insert_data_lib(graph: &mut Graph) {
    graph.insert_libfunc(data_id());
    graph.insert_libfunc(data_libname());
    graph.insert_libfunc(data_exists());
    graph.insert_libfunc(data_objs());
    graph.insert_libfunc(data_drop());
    graph.insert_libfunc(data_drop_from());
    graph.insert_libfunc(data_attach());
    graph.insert_libfunc(data_move());

    graph.insert_libfunc(data_from_id());
    graph.insert_libfunc(data_from_field());
}


lazy_static! {
    pub(self) static ref ID: Arc<dyn Instruction> = Arc::new(DataIns::Id);
    pub(self) static ref TAGNAME: Arc<dyn Instruction> = Arc::new(DataIns::Tagname);
    pub(self) static ref EXISTS: Arc<dyn Instruction> = Arc::new(DataIns::Exists);
    pub(self) static ref OBJS: Arc<dyn Instruction> = Arc::new(DataIns::Objs);
    pub(self) static ref DROP: Arc<dyn Instruction> = Arc::new(DataIns::Drop);
    pub(self) static ref DROP_FROM: Arc<dyn Instruction> = Arc::new(DataIns::DropFrom);
    pub(self) static ref ATTACH: Arc<dyn Instruction> = Arc::new(DataIns::Attach);
    pub(self) static ref MOVE: Arc<dyn Instruction> = Arc::new(DataIns::Move);
    pub(self) static ref FIELD: Arc<dyn Instruction> = Arc::new(DataIns::Field);
    pub(self) static ref FROM_ID: Arc<dyn Instruction> = Arc::new(DataIns::FromId);
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Data instructions.
pub enum DataIns {
    Id,
    Exists,
    Objs,
    Drop,
    DropFrom,
    Attach,
    Move,
    Tagname,

    Field,
    FromId,
}
#[typetag::serde(name = "DataIns")]
impl Instruction for DataIns {
    fn exec(&self, env: &mut ProcEnv, graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        match self {
            Self::Id => {
                if let Some(var) = env.stack.pop() {
                    if let Some(dref) = var.try_data_or_func() {
                        env.stack.push(Variable::val(Val::Str(dref.as_ref().into())));
                        return Ok(None);
                    }
                }
                Err(Error::DataId)
            },
            Self::Tagname => {
                if let Some(var) = env.stack.pop() {
                    if let Some(dref) = var.try_data_or_func() {
                        if let Some(tagname) = dref.tagname(&graph) {
                            env.stack.push(Variable::val(Val::Str(tagname.into())));
                        } else {
                            env.stack.push(Variable::val(Val::Null));
                        }
                        return Ok(None);
                    }
                }
                Err(Error::DataTagname)
            },
            Self::Exists => {
                if let Some(var) = env.stack.pop() {
                    if let Some(dref) = var.try_data_or_func() {
                        env.stack.push(Variable::val(Val::Bool(dref.data_exists(&graph))));
                        return Ok(None);
                    }
                }
                Err(Error::DataExists)
            },
            Self::Objs => {
                if let Some(var) = env.stack.pop() {
                    if let Some(dref) = var.try_data_or_func() {
                        let nodes = dref.data_nodes(&graph)
                            .into_iter()
                            .map(|nref| ValRef::new(Val::Obj(nref)))
                            .collect::<Vector<_>>();
                        env.stack.push(Variable::val(Val::List(nodes)));
                        return Ok(None);
                    }
                }
                Err(Error::DataObjs)
            },
            Self::Drop => {
                if let Some(var) = env.stack.pop() {
                    if let Some(dref) = var.try_data_or_func() {
                        let dropped = graph.remove_data(&dref, None);
                        env.stack.push(Variable::val(Val::Bool(dropped)));
                        return Ok(None);
                    }
                }
                Err(Error::DataDrop)
            },
            Self::DropFrom => {
                if let Some(from) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        if let Some(dref) = var.try_data_or_func() {
                            if let Some(nref) = from.try_obj() {
                                let dropped = graph.remove_data(&dref, Some(nref));
                                env.stack.push(Variable::val(Val::Bool(dropped)));
                                return Ok(None);
                            }
                        }
                    }
                }
                Err(Error::DataDrop)
            },
            Self::Attach => {
                if let Some(node_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        if let Some(nref) = node_var.try_obj() {
                            if let Some(dref) = var.try_data_or_func() {
                                let attached = graph.attach_data(&nref, &dref);
                                env.stack.push(Variable::val(Val::Bool(attached)));
                                return Ok(None);
                            }
                        }
                    }
                }
                Err(Error::DataAttach)
            },
            Self::Move => {
                if let Some(to_var) = env.stack.pop() {
                    if let Some(from_var) = env.stack.pop() {
                        if let Some(var) = env.stack.pop() {
                            if let Some(to_ref) = to_var.try_obj() {
                                if let Some(from_ref) = from_var.try_obj() {
                                    if let Some(dref) = var.try_data_or_func() {
                                        let mut moved = false;
                                        if graph.attach_data(&to_ref, &dref) {
                                            if graph.remove_data(&dref, Some(from_ref)) {
                                                moved = true;
                                            }
                                        }
                                        env.stack.push(Variable::val(Val::Bool(moved)));
                                        return Ok(None);
                                    }
                                }
                            }
                        }
                    }
                }
                Err(Error::DataMove)
            },

            Self::FromId => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Str(id) => {
                            env.stack.push(Variable::val(Val::Data(SId::from(id.as_str()))));
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::DataFromId)
            },
            Self::Field => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Str(path) => {
                            let mut context = None;
                            if path.starts_with(SELF_STR_KEYWORD.as_str()) || path.starts_with(SUPER_STR_KEYWORD.as_str()) {
                                context = Some(env.self_ptr());
                            }
                            if let Some(field_ref) = Field::field_from_path(graph, path.as_str(), context) {
                                env.stack.push(Variable::val(Val::Data(field_ref)));
                            } else {
                                env.stack.push(Variable::val(Val::Null));
                            }
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::DataField)
            },
        }
    }
}
