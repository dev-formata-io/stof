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
use serde::{Deserialize, Serialize};
use crate::{model::{libraries::func::ops::{fn_attributes, fn_call, fn_data, fn_exp_call, fn_has_attr, fn_id, fn_is_async, fn_name, fn_obj, fn_objs, fn_params, fn_return_type}, Func, Graph, ASYNC_FUNC_ATTR}, runtime::{instruction::{Instruction, Instructions}, instructions::{call::FuncCall, Base}, proc::ProcEnv, Error, Val, ValRef, Variable}};

mod ops;


/// Library name.
pub(self) const FUNC_LIB: ArcStr = literal!("Fn");


/// Add the function library to a graph.
pub fn insert_fn_lib(graph: &mut Graph) {
    graph.insert_libfunc(fn_id());
    graph.insert_libfunc(fn_data());
    graph.insert_libfunc(fn_name());
    graph.insert_libfunc(fn_params());
    graph.insert_libfunc(fn_return_type());
    graph.insert_libfunc(fn_has_attr());
    graph.insert_libfunc(fn_attributes());
    graph.insert_libfunc(fn_obj());
    graph.insert_libfunc(fn_objs());
    graph.insert_libfunc(fn_is_async());
    graph.insert_libfunc(fn_call());
    graph.insert_libfunc(fn_exp_call());
}


lazy_static! {
    pub(self) static ref ID: Arc<dyn Instruction> = Arc::new(FuncIns::Id);
    pub(self) static ref DATA: Arc<dyn Instruction> = Arc::new(FuncIns::Data);
    pub(self) static ref NAME: Arc<dyn Instruction> = Arc::new(FuncIns::Name);
    pub(self) static ref PARAMS: Arc<dyn Instruction> = Arc::new(FuncIns::Params);
    pub(self) static ref RETURN_TYPE: Arc<dyn Instruction> = Arc::new(FuncIns::ReturnType);
    pub(self) static ref HAS_ATTR: Arc<dyn Instruction> = Arc::new(FuncIns::HasAttr);
    pub(self) static ref ATTRIBUTES: Arc<dyn Instruction> = Arc::new(FuncIns::Attributes);
    pub(self) static ref OBJ: Arc<dyn Instruction> = Arc::new(FuncIns::Obj);
    pub(self) static ref OBJS: Arc<dyn Instruction> = Arc::new(FuncIns::Objs);
    pub(self) static ref IS_ASYNC: Arc<dyn Instruction> = Arc::new(FuncIns::IsAsync);
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Func instructions.
pub enum FuncIns {
    Id,
    Data,
    Name,
    Params,
    ReturnType,
    HasAttr,
    Attributes,
    Obj,
    Objs,
    IsAsync,
    Call(usize),
    ExpandCall(usize),
}
#[typetag::serde(name = "FuncIns")]
impl Instruction for FuncIns {
    fn exec(&self, env: &mut ProcEnv, graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        match self {
            Self::Id => {
                if let Some(val) = env.stack.pop() {
                    if let Some(dref) = val.try_func() {
                        env.stack.push(Variable::val(Val::Str(dref.as_ref().into())));
                        return Ok(None);
                    }
                }
                Err(Error::FnId)
            },
            Self::Data => {
                if let Some(val) = env.stack.pop() {
                    if let Some(dref) = val.try_func() {
                        env.stack.push(Variable::val(Val::Data(dref)));
                        return Ok(None);
                    }
                }
                Err(Error::FnData)
            },
            Self::Name => {
                if let Some(var) = env.stack.pop() {
                    if let Some(dref) = var.try_func() {
                        if let Some(data) = dref.data(&graph) {
                            env.stack.push(Variable::val(Val::Str(data.name.as_ref().into())));
                            return Ok(None);
                        }
                    }
                }
                Err(Error::FnName)
            },
            Self::Params => {
                if let Some(var) = env.stack.pop() {
                    if let Some(dref) = var.try_func() {
                        if let Some(func) = graph.get_stof_data::<Func>(&dref) {
                            let mut params = vector![];
                            for param in &func.params {
                                let tup = vector![
                                    ValRef::new(Val::Str(param.name.as_ref().into())),
                                    ValRef::new(Val::Str(param.param_type.rt_type_of(&graph)))
                                ];
                                params.push_back(ValRef::new(Val::Tup(tup)));
                            }
                            env.stack.push(Variable::val(Val::List(params)));
                            return Ok(None);
                        }
                    }
                }
                Err(Error::FnParams)
            },
            Self::ReturnType => {
                if let Some(var) = env.stack.pop() {
                    if let Some(dref) = var.try_func() {
                        if let Some(func) = graph.get_stof_data::<Func>(&dref) {
                            env.stack.push(Variable::val(Val::Str(func.return_type.rt_type_of(&graph))));
                            return Ok(None);
                        }
                    }
                }
                Err(Error::FnReturnType)
            },
            Self::HasAttr => {
                if let Some(attr_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        if let Some(dref) = var.try_func() {
                            if let Some(func) = graph.get_stof_data::<Func>(&dref) {
                                match attr_var.val.read().deref() {
                                    Val::Str(attr) => {
                                        let res = func.attributes.contains_key(attr.as_str());
                                        env.stack.push(Variable::val(Val::Bool(res)));
                                        return Ok(None);
                                    },
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                Err(Error::FnHasAttr)
            },
            Self::Attributes => {
                if let Some(var) = env.stack.pop() {
                    if let Some(dref) = var.try_func() {
                        if let Some(func) = graph.get_stof_data::<Func>(&dref) {
                            let mut map = OrdMap::default();
                            for (k, v) in &func.attributes {
                                map.insert(ValRef::new(Val::Str(ArcStr::from(k))), ValRef::new(v.clone()));
                            }
                            env.stack.push(Variable::val(Val::Map(map)));
                            return Ok(None);
                        }
                    }
                }
                Err(Error::FnAttributes)
            },
            Self::Obj => {
                if let Some(var) = env.stack.pop() {
                    if let Some(dref) = var.try_func() {
                        for node in dref.data_nodes(&graph) {
                            env.stack.push(Variable::val(Val::Obj(node)));
                            return Ok(None);
                        }
                    }
                }
                Err(Error::FnObj)
            },
            Self::Objs => {
                if let Some(var) = env.stack.pop() {
                    if let Some(dref) = var.try_func() {
                        let mut nodes = vector![];
                        for node in dref.data_nodes(&graph) {
                            nodes.push_back(ValRef::new(Val::Obj(node)));
                        }
                        env.stack.push(Variable::val(Val::List(nodes)));
                        return Ok(None);
                    }
                }
                Err(Error::FnObjs)
            },
            Self::IsAsync => {
                if let Some(var) = env.stack.pop() {
                    if let Some(dref) = var.try_func() {
                        if let Some(func) = graph.get_stof_data::<Func>(&dref) {
                            env.stack.push(Variable::val(Val::Bool(func.attributes.contains_key(ASYNC_FUNC_ATTR.as_str()))));
                            return Ok(None);
                        }
                    }
                }
                Err(Error::FnIsAsync)
            },
            Self::Call(stack_count) => {
                let mut args = vector![];
                if *stack_count > 1 {
                    for _ in 0..(*stack_count - 1) {
                        // Args are pushed in reverse, so push to the front
                        args.push_front(Arc::new(Base::Variable(env.stack.pop().unwrap())) as Arc<dyn Instruction>);
                    }
                }
                if let Some(var) = env.stack.pop() {
                    if let Some(dref) = var.try_func() {
                        let mut instructions = Instructions::default();
                        instructions.push(Arc::new(FuncCall {
                            func: Some(dref),
                            search: None,
                            stack: false,
                            args
                        }));
                        return Ok(Some(instructions));
                    }
                }
                Err(Error::FnCall)
            },
            Self::ExpandCall(stack_count) => {
                let mut args: Vector<Arc<dyn Instruction>> = vector![];
                if *stack_count > 1 {
                    let mut vars = Vec::new();
                    for _ in 0..(*stack_count - 1) {
                        if let Some(var) = env.stack.pop() {
                            vars.push(var);
                        }
                    }
                    for var in vars.into_iter().rev() {
                        match var.val.read().deref() {
                            Val::List(list_args) => {
                                for larg in list_args {
                                    args.push_back(Arc::new(Base::Variable(Variable::refval(larg.clone()))));
                                }
                            },
                            Val::Set(set_args) => {
                                for sarg in set_args {
                                    args.push_back(Arc::new(Base::Variable(Variable::refval(sarg.clone()))));
                                }
                            },
                            _ => {
                                args.push_back(Arc::new(Base::Variable(Variable::refval(var.val.clone()))));
                            }
                        }
                    }
                }
                if let Some(var) = env.stack.pop() {
                    if let Some(dref) = var.try_func() {
                        let mut instructions = Instructions::default();
                        instructions.push(Arc::new(FuncCall {
                            func: Some(dref),
                            search: None,
                            stack: false,
                            args
                        }));
                        return Ok(Some(instructions));
                    }
                }
                Err(Error::FnExpandCall)
            },
        }
    }
}
