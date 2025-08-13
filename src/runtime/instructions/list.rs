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

use std::{ops::{Deref, DerefMut}, sync::Arc};
use imbl::Vector;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use crate::{model::Graph, parser::types::parse_type_complete, runtime::{instruction::{Instruction, Instructions}, proc::ProcEnv, Error, Num, Type, Val, Variable}};


lazy_static! {
    pub static ref NEW_LIST: Arc<dyn Instruction> = Arc::new(ListIns::NewList);
    pub static ref PUSH_LIST: Arc<dyn Instruction> = Arc::new(ListIns::PushList);

    pub static ref APPEND_LIST: Arc<dyn Instruction> = Arc::new(ListIns::AppendOther);
    pub static ref POP_FRONT_LIST: Arc<dyn Instruction> = Arc::new(ListIns::PopFront);
    pub static ref POP_BACK_LIST: Arc<dyn Instruction> = Arc::new(ListIns::PopBack);
    pub static ref CLEAR_LIST: Arc<dyn Instruction> = Arc::new(ListIns::Clear);
    pub static ref REVERSE_LIST: Arc<dyn Instruction> = Arc::new(ListIns::Reverse);
    pub static ref REVERSED_LIST: Arc<dyn Instruction> = Arc::new(ListIns::Reversed);
    pub static ref LEN_LIST: Arc<dyn Instruction> = Arc::new(ListIns::Len);
    pub static ref AT_LIST: Arc<dyn Instruction> = Arc::new(ListIns::At);
    pub static ref AT_REF_LIST: Arc<dyn Instruction> = Arc::new(ListIns::AtRef);
    pub static ref EMPTY_LIST: Arc<dyn Instruction> = Arc::new(ListIns::Empty);
    pub static ref ANY_LIST: Arc<dyn Instruction> = Arc::new(ListIns::Any);
    pub static ref FIRST_LIST: Arc<dyn Instruction> = Arc::new(ListIns::First);
    pub static ref LAST_LIST: Arc<dyn Instruction> = Arc::new(ListIns::Last);
    pub static ref FIRST_REF_LIST: Arc<dyn Instruction> = Arc::new(ListIns::FirstRef);
    pub static ref LAST_REF_LIST: Arc<dyn Instruction> = Arc::new(ListIns::LastRef);
    pub static ref JOIN_LIST: Arc<dyn Instruction> = Arc::new(ListIns::Join);
    pub static ref CONTAINS_LIST: Arc<dyn Instruction> = Arc::new(ListIns::Contains);
    pub static ref INDEX_OF_LIST: Arc<dyn Instruction> = Arc::new(ListIns::IndexOf);
    pub static ref REMOVE_LIST: Arc<dyn Instruction> = Arc::new(ListIns::Remove);
    pub static ref REMOVE_FIRST_LIST: Arc<dyn Instruction> = Arc::new(ListIns::RemoveFirst);
    pub static ref REMOVE_LAST_LIST: Arc<dyn Instruction> = Arc::new(ListIns::RemoveLast);
    pub static ref REMOVE_ALL_LIST: Arc<dyn Instruction> = Arc::new(ListIns::RemoveAll);
    pub static ref INSERT_LIST: Arc<dyn Instruction> = Arc::new(ListIns::Insert);
    pub static ref REPLACE_LIST: Arc<dyn Instruction> = Arc::new(ListIns::Replace);
    pub static ref SORT_LIST: Arc<dyn Instruction> = Arc::new(ListIns::Sort);
    pub static ref IS_UNIFORM_LIST: Arc<dyn Instruction> = Arc::new(ListIns::IsUniform);
    pub static ref TO_UNIFORM_LIST: Arc<dyn Instruction> = Arc::new(ListIns::ToUniform);
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// List creation instructions.
pub enum ListIns {
    // Low-level for list construction
    NewList,
    PushList,

    // High-level
    AppendList(Arc<dyn Instruction>), // evaluate and add to the stack (push)

    // Library instructions
    AppendOther,
    PushBack(usize),
    PushFront(usize),
    PopFront,
    PopBack,
    Clear,
    Reverse,
    Reversed,
    Len,
    At,
    AtRef,
    Empty,
    Any,
    First,
    FirstRef,
    Last,
    LastRef,
    Join,
    Contains,
    IndexOf,
    Remove, // by index
    RemoveFirst,
    RemoveLast,
    RemoveAll,
    Insert,
    Replace,
    Sort,
    IsUniform,
    ToUniform,
}
#[typetag::serde(name = "ListIns")]
impl Instruction for ListIns {
    fn exec(&self, env: &mut ProcEnv, graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        match self {
            Self::NewList => {
                env.stack.push(Variable::val(Val::List(Default::default())));
                Ok(None)
            },
            Self::PushList => {
                if let Some(push_var) = env.stack.pop() {
                    if let Some(list_var) = env.stack.pop() {
                        {
                            let mut val = list_var.val.write();
                            let val = val.deref_mut();
                            match &mut *val {
                                Val::List(values) => {
                                    values.push_back(push_var.val);
                                },
                                _ => {}
                            }
                        }
                        env.stack.push(list_var);
                    }
                }
                Ok(None)
            },

            /*****************************************************************************
             * High-level.
             *****************************************************************************/
            Self::AppendList(ins) => {
                let mut instructions = Instructions::default();
                instructions.push(ins.clone());
                instructions.push(PUSH_LIST.clone());
                return Ok(Some(instructions));
            },

            /*****************************************************************************
             * Library instructions.
             *****************************************************************************/
            Self::AppendOther => {
                if let Some(other_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        match other_var.val.read().deref() {
                            Val::List(other) => {
                                match var.val.write().deref_mut() {
                                    Val::List(list) => {
                                        list.append(other.clone());
                                        return Ok(None); // does not return anything
                                    },
                                    _ => {}
                                }
                            },
                            _ => {}
                        }
                    }
                }
                Err(Error::ListAppendOther)
            },
            Self::PushBack(stack_count) => {
                let mut to_push = Vec::new();
                if *stack_count > 1 {
                    for _ in 0..(*stack_count - 1) {
                        to_push.push(env.stack.pop().unwrap());
                    }
                }
                if let Some(var) = env.stack.pop() {
                    match var.val.write().deref_mut() {
                        Val::List(list) => {
                            for var in to_push.into_iter().rev() {
                                list.push_back(var.val);
                            }
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::ListPushBack)
            },
            Self::PushFront(stack_count) => {
                let mut to_push = Vec::new();
                if *stack_count > 1 {
                    for _ in 0..(*stack_count - 1) {
                        to_push.push(env.stack.pop().unwrap());
                    }
                }
                if let Some(var) = env.stack.pop() {
                    match var.val.write().deref_mut() {
                        Val::List(list) => {
                            for var in to_push.into_iter().rev() {
                                list.push_front(var.val);
                            }
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::ListPushFront)
            },
            Self::PopFront => {
                if let Some(var) = env.stack.pop() {
                    let mut res = None;
                    match var.val.write().deref_mut() {
                        Val::List(list) => {
                            res = list.pop_front();
                        },
                        _ => {}
                    }
                    if let Some(res) = res {
                        env.stack.push(Variable::refval(res));
                    } else {
                        env.stack.push(Variable::val(Val::Null));
                    }
                    return Ok(None);
                }
                Err(Error::ListPopFront)
            },
            Self::PopBack => {
                if let Some(var) = env.stack.pop() {
                    let mut res = None;
                    match var.val.write().deref_mut() {
                        Val::List(list) => {
                            res = list.pop_back();
                        },
                        _ => {}
                    }
                    if let Some(res) = res {
                        env.stack.push(Variable::refval(res));
                    } else {
                        env.stack.push(Variable::val(Val::Null));
                    }
                    return Ok(None);
                }
                Err(Error::ListPopBack)
            },
            Self::Clear => {
                if let Some(var) = env.stack.pop() {
                    match var.val.write().deref_mut() {
                        Val::List(list) => {
                            list.clear();
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::ListClear)
            },
            Self::Reverse => {
                if let Some(var) = env.stack.pop() {
                    match var.val.write().deref_mut() {
                        Val::List(list) => {
                            let new = list
                                .iter()
                                .rev()
                                .map(|v| v.clone())
                                .collect::<Vector<_>>();
                            *list = new;
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::ListReverse)
            },
            Self::Reversed => {
                if let Some(var) = env.stack.pop() {
                    let mut new = Vector::default();
                    match var.val.read().deref() {
                        Val::List(list) => {
                            new = list
                                .iter()
                                .rev()
                                .map(|v| {
                                    v.duplicate(false)
                                })
                                .collect::<Vector<_>>();
                        },
                        _ => {}
                    }
                    env.stack.push(Variable::val(Val::List(new)));
                    return Ok(None);
                }
                Err(Error::ListReversed)
            },
            Self::Len => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::List(list) => {
                            env.stack.push(Variable::val(Val::Num(Num::Int(list.len() as i64))));
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::ListLen)
            },
            Self::At => {
                if let Some(index_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        match index_var.val.read().deref() {
                            Val::Num(num) => {
                                match var.val.read().deref() {
                                    Val::List(list) => {
                                        if let Some(val) = list.get(num.int() as usize) {
                                            env.stack.push(Variable::refval(val.duplicate(false)));
                                        } else {
                                            env.stack.push(Variable::val(Val::Null));
                                        }
                                        return Ok(None);
                                    },
                                    _ => {}
                                }
                            },
                            _ => {},
                        }
                    }
                }
                Err(Error::ListAt)
            },
            Self::AtRef => {
                if let Some(index_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        match index_var.val.read().deref() {
                            Val::Num(num) => {
                                match var.val.read().deref() {
                                    Val::List(list) => {
                                        if let Some(val) = list.get(num.int() as usize) {
                                            env.stack.push(Variable::refval(val.duplicate(true)));
                                        } else {
                                            env.stack.push(Variable::val(Val::Null));
                                        }
                                        return Ok(None);
                                    },
                                    _ => {}
                                }
                            },
                            _ => {},
                        }
                    }
                }
                Err(Error::ListAt)
            },
            Self::Empty => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::List(list) => {
                            env.stack.push(Variable::val(Val::Bool(list.is_empty())));
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::ListEmpty)
            },
            Self::Any => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::List(list) => {
                            env.stack.push(Variable::val(Val::Bool(!list.is_empty())));
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::ListAny)
            },
            Self::First => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::List(list) => {
                            if let Some(val) = list.front() {
                                env.stack.push(Variable::refval(val.duplicate(false)));
                            } else {
                                env.stack.push(Variable::val(Val::Null));
                            }
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::ListFirst)
            },
            Self::Last => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::List(list) => {
                            if let Some(val) = list.back() {
                                env.stack.push(Variable::refval(val.duplicate(false)));
                            } else {
                                env.stack.push(Variable::val(Val::Null));
                            }
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::ListLast)
            },
            Self::FirstRef => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::List(list) => {
                            if let Some(val) = list.front() {
                                env.stack.push(Variable::refval(val.duplicate(true)));
                            } else {
                                env.stack.push(Variable::val(Val::Null));
                            }
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::ListFirst)
            },
            Self::LastRef => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::List(list) => {
                            if let Some(val) = list.back() {
                                env.stack.push(Variable::refval(val.duplicate(true)));
                            } else {
                                env.stack.push(Variable::val(Val::Null));
                            }
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::ListLast)
            },
            Self::Join => {
                if let Some(sep_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        match sep_var.val.read().deref() {
                            Val::Str(sep) => {
                                match var.val.read().deref() {
                                    Val::List(list) => {
                                        let mut joined = String::default();
                                        if let Some(first) = list.front() {
                                            joined.push_str(&first.read().print(&graph));
                                        }
                                        for item in list.iter().skip(1) {
                                            joined.push_str(&format!("{sep}{}", item.read().print(&graph)));
                                        }
                                        env.stack.push(Variable::val(Val::Str(joined.into())));
                                        return Ok(None);
                                    },
                                    _ => {}
                                }
                            },
                            _ => {}
                        }
                    }
                }
                Err(Error::ListJoin)
            },
            Self::Contains => {
                if let Some(search_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        match var.val.read().deref() {
                            Val::List(list) => {
                                for list_var in list {
                                    if search_var.val.read().equal(&list_var.read())?.truthy() {
                                        env.stack.push(Variable::val(Val::Bool(true)));
                                        return Ok(None);
                                    }
                                }
                                env.stack.push(Variable::val(Val::Bool(false)));
                                return Ok(None);
                            },
                            _ => {}
                        }
                    }
                }
                Err(Error::ListContains)
            },
            Self::IndexOf => {
                if let Some(search_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        match var.val.read().deref() {
                            Val::List(list) => {
                                let mut index: i64 = 0;
                                for list_var in list {
                                    if search_var.val.read().equal(&list_var.read())?.truthy() {
                                        env.stack.push(Variable::val(Val::Num(Num::Int(index))));
                                        return Ok(None);
                                    }
                                    index += 1;
                                }
                                env.stack.push(Variable::val(Val::Num(Num::Int(-1))));
                                return Ok(None);
                            },
                            _ => {}
                        }
                    }
                }
                Err(Error::ListIndexOf)
            },
            Self::Remove => {
                if let Some(index_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        match index_var.val.read().deref() {
                            Val::Num(index) => {
                                match var.val.write().deref_mut() {
                                    Val::List(list) => {
                                        let index = index.int() as usize;
                                        if index < list.len() {
                                            let val = list.remove(index);
                                            env.stack.push(Variable::refval(val));
                                        } else {
                                            env.stack.push(Variable::val(Val::Null));
                                        }
                                        return Ok(None);
                                    },
                                    _ => {}
                                }
                            },
                            _ => {}
                        }
                    }
                }
                Err(Error::ListRemove)
            },
            Self::RemoveFirst => {
                if let Some(search_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        match var.val.write().deref_mut() {
                            Val::List(list) => {
                                let mut index: usize = 0;
                                let mut found = false;
                                for list_var in list.iter() {
                                    if search_var.val.read().equal(&list_var.read())?.truthy() {
                                        found = true;
                                        break;
                                    }
                                    index += 1;
                                }
                                if found {
                                    let val = list.remove(index);
                                    env.stack.push(Variable::refval(val));
                                } else {
                                    env.stack.push(Variable::val(Val::Null));
                                }
                                return Ok(None);
                            },
                            _ => {}
                        }
                    }
                }
                Err(Error::ListRemoveFirst)
            },
            Self::RemoveLast => {
                if let Some(search_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        match var.val.write().deref_mut() {
                            Val::List(list) => {
                                let mut index: usize = list.len() - 1;
                                let mut found = false;
                                for list_var in list.iter().rev() {
                                    if search_var.val.read().equal(&list_var.read())?.truthy() {
                                        found = true;
                                        break;
                                    }
                                    index -= 1;
                                }
                                if found {
                                    let val = list.remove(index);
                                    env.stack.push(Variable::refval(val));
                                } else {
                                    env.stack.push(Variable::val(Val::Null));
                                }
                                return Ok(None);
                            },
                            _ => {}
                        }
                    }
                }
                Err(Error::ListRemoveLast)
            },
            Self::RemoveAll => {
                if let Some(search_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        match var.val.write().deref_mut() {
                            Val::List(list) => {
                                let mut index: usize = 0;
                                let mut matches = Vec::new();
                                for list_var in list.iter() {
                                    if search_var.val.read().equal(&list_var.read())?.truthy() {
                                        matches.push(index);
                                    }
                                    index += 1;
                                }

                                let res = matches.len() > 0;
                                for index in matches.into_iter().rev() {
                                    list.remove(index);
                                }

                                env.stack.push(Variable::val(Val::Bool(res)));
                                return Ok(None);
                            },
                            _ => {}
                        }
                    }
                }
                Err(Error::ListRemoveAll)
            },
            Self::Insert => {
                if let Some(insert_var) = env.stack.pop() {
                    if let Some(index_var) = env.stack.pop() {
                        if let Some(var) = env.stack.pop() {
                            match index_var.val.read().deref() {
                                Val::Num(index) => {
                                    match var.val.write().deref_mut() {
                                        Val::List(list) => {
                                            let int = index.int();
                                            if int < 0 {
                                                list.push_front(insert_var.val);
                                            } else if int as usize > list.len() - 1 {
                                                list.push_back(insert_var.val);
                                            } else {
                                                list.insert(int as usize, insert_var.val);
                                            }
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
                Err(Error::ListInsert)
            },
            Self::Replace => {
                if let Some(replace_var) = env.stack.pop() {
                    if let Some(index_var) = env.stack.pop() {
                        if let Some(var) = env.stack.pop() {
                            match index_var.val.read().deref() {
                                Val::Num(index) => {
                                    match var.val.write().deref_mut() {
                                        Val::List(list) => {
                                            let int = index.int();
                                            let index: usize;
                                            if int < 0 {
                                                index = 0;
                                            } else if int as usize > list.len() - 1 {
                                                index = list.len() - 1;
                                            } else {
                                                index = int as usize;
                                            }
                                            if let Some(val) = list.get_mut(index) {
                                                env.stack.push(Variable::refval(val.clone()));
                                                *val = replace_var.val;
                                            } else {
                                                env.stack.push(Variable::val(Val::Null));
                                            }
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
                Err(Error::ListReplace)
            },
            Self::Sort => {
                if let Some(var) = env.stack.pop() {
                    match var.val.write().deref_mut() {
                        Val::List(list) => {
                            list.sort();
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::ListSort)
            },
            Self::IsUniform => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::List(list) => {
                            let mut uniform_type;
                            if list.is_empty() {
                                uniform_type = Some(Type::Void.rt_type_of(&graph));
                            } else {
                                uniform_type = Some(list.front().unwrap().read().spec_type(&graph).rt_type_of(&graph));
                                for val in list.iter().skip(1) {
                                    let other = Some(val.read().spec_type(&graph).rt_type_of(&graph));
                                    if other != uniform_type {
                                        uniform_type = None;
                                        break;
                                    }
                                }
                            }
                            if let Some(uniform) = uniform_type {
                                env.stack.push(Variable::val(Val::Str(uniform)));
                            } else {
                                env.stack.push(Variable::val(Val::Null));
                            }
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::ListIsUniform)
            },
            Self::ToUniform => {
                if let Some(type_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        match type_var.val.read().deref() {
                            Val::Str(type_str) => {
                                match parse_type_complete(type_str.as_str()) {
                                    Ok(ctype) => {
                                        match var.val.write().deref_mut() {
                                            Val::List(list) => {
                                                let context = env.self_ptr();
                                                for val in list.iter_mut() {
                                                    val.write().cast(&ctype, graph, Some(context.clone()))?;
                                                }
                                                return Ok(None);
                                            },
                                            _ => {}
                                        }
                                    },
                                    Err(_) => {}
                                }
                            },
                            _ => {}
                        }
                    }
                }
                Err(Error::ListToUniform)
            },
        }
    }
}
