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
use imbl::vector;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use crate::{model::Graph, parser::types::parse_type_complete, runtime::{instruction::{Instruction, Instructions}, proc::ProcEnv, Error, Num, Type, Val, ValRef, Variable}};


lazy_static! {
    pub static ref NEW_SET: Arc<dyn Instruction> = Arc::new(SetIns::NewSet);
    pub static ref PUSH_SET: Arc<dyn Instruction> = Arc::new(SetIns::PushSet);

    pub static ref APPEND_SET: Arc<dyn Instruction> = Arc::new(SetIns::AppendOther);
    pub static ref CLEAR_SET: Arc<dyn Instruction> = Arc::new(SetIns::Clear);
    pub static ref CONTAINS_SET: Arc<dyn Instruction> = Arc::new(SetIns::Contains);
    pub static ref FIRST_SET: Arc<dyn Instruction> = Arc::new(SetIns::First);
    pub static ref LAST_SET: Arc<dyn Instruction> = Arc::new(SetIns::Last);
    pub static ref INSERT_SET: Arc<dyn Instruction> = Arc::new(SetIns::Insert);
    pub static ref SPLIT_SET: Arc<dyn Instruction> = Arc::new(SetIns::Split);
    pub static ref EMPTY_SET: Arc<dyn Instruction> = Arc::new(SetIns::Empty);
    pub static ref ANY_SET: Arc<dyn Instruction> = Arc::new(SetIns::Any);
    pub static ref LEN_SET: Arc<dyn Instruction> = Arc::new(SetIns::Len);
    pub static ref AT_SET: Arc<dyn Instruction> = Arc::new(SetIns::At);
    pub static ref POP_FIRST_SET: Arc<dyn Instruction> = Arc::new(SetIns::PopFirst);
    pub static ref POP_LAST_SET: Arc<dyn Instruction> = Arc::new(SetIns::PopLast);
    pub static ref REMOVE_SET: Arc<dyn Instruction> = Arc::new(SetIns::Remove);
    pub static ref UNION_SET: Arc<dyn Instruction> = Arc::new(SetIns::Union);
    pub static ref DIFF_SET: Arc<dyn Instruction> = Arc::new(SetIns::Difference);
    pub static ref INTERSECTION_SET: Arc<dyn Instruction> = Arc::new(SetIns::Intersection);
    pub static ref SYMMETRIC_DIFF_SET: Arc<dyn Instruction> = Arc::new(SetIns::SymmetricDifference);
    pub static ref DISJOINT_SET: Arc<dyn Instruction> = Arc::new(SetIns::Disjoint);
    pub static ref SUBSET_SET: Arc<dyn Instruction> = Arc::new(SetIns::Subset);
    pub static ref SUPERSET_SET: Arc<dyn Instruction> = Arc::new(SetIns::Superset);
    pub static ref IS_UNIFORM_SET: Arc<dyn Instruction> = Arc::new(SetIns::IsUniform);
    pub static ref TO_UNIFORM_SET: Arc<dyn Instruction> = Arc::new(SetIns::ToUniform);
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Set creation instructions.
pub enum SetIns {
    // Low-level for construction
    NewSet,
    PushSet,

    // High-level
    AppendSet(Arc<dyn Instruction>), // evaluate and add to the stack (push)

    // Library functions
    AppendOther,
    Clear,
    Contains,
    First,
    Last,
    Insert,
    Split,
    Empty,
    Any,
    Len,
    At,
    PopFirst,
    PopLast,
    Remove,
    Union,
    Difference,
    Intersection,
    SymmetricDifference,
    Disjoint,
    Subset,
    Superset,
    IsUniform,
    ToUniform,
}
#[typetag::serde(name = "SetIns")]
impl Instruction for SetIns {
    fn exec(&self, env: &mut ProcEnv, graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        match self {
            Self::NewSet => {
                env.stack.push(Variable::val(Val::Set(Default::default())));
                Ok(None)
            },
            Self::PushSet => {
                if let Some(push_var) = env.stack.pop() {
                    if let Some(set_var) = env.stack.pop() {
                        {
                            let mut val = set_var.val.write();
                            let val = val.deref_mut();
                            match &mut *val {
                                Val::Set(values) => {
                                    values.insert(push_var.val);
                                },
                                _ => {}
                            }
                        }
                        env.stack.push(set_var);
                    }
                }
                Ok(None)
            },

            /*****************************************************************************
             * High-level.
             *****************************************************************************/
            Self::AppendSet(ins) => {
                let mut instructions = Instructions::default();
                instructions.push(ins.clone());
                instructions.push(PUSH_SET.clone());
                return Ok(Some(instructions));
            },

            /*****************************************************************************
             * Library.
             *****************************************************************************/
            Self::AppendOther => {
                if let Some(other_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        match other_var.val.read().deref() {
                            Val::Set(other) => {
                                match var.val.write().deref_mut() {
                                    Val::Set(set) => {
                                        for val in other {
                                            set.insert(val.duplicate(false));
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
                Err(Error::SetAppendOther)
            },
            Self::Clear => {
                if let Some(var) = env.stack.pop() {
                    match var.val.write().deref_mut() {
                        Val::Set(set) => {
                            set.clear();
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::SetClear)
            },
            Self::Contains => {
                if let Some(test_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        match var.val.read().deref() {
                            Val::Set(set) => {
                                let contains = set.contains(&test_var.val);
                                env.stack.push(Variable::val(Val::Bool(contains)));
                                return Ok(None);
                            },
                            _ => {}
                        }
                    }
                }
                Err(Error::SetContains)
            },
            Self::First => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Set(set) => {
                            if let Some(first) = set.get_min() {
                                env.stack.push(Variable::refval(first.duplicate(false)));
                            } else {
                                env.stack.push(Variable::val(Val::Null));
                            }
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::SetFirst)
            },
            Self::Last => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Set(set) => {
                            if let Some(last) = set.get_max() {
                                env.stack.push(Variable::refval(last.duplicate(false)));
                            } else {
                                env.stack.push(Variable::val(Val::Null));
                            }
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::SetLast)
            },
            Self::Insert => {
                if let Some(insert_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        match var.val.write().deref_mut() {
                            Val::Set(set) => {
                                let newly_inserted = set.insert(insert_var.val).is_none();
                                env.stack.push(Variable::val(Val::Bool(newly_inserted)));
                                return Ok(None);
                            },
                            _ => {}
                        }
                    }
                }
                Err(Error::SetInsert)
            },
            Self::Split => {
                if let Some(split_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        match var.val.write().deref_mut() {
                            Val::Set(set) => {
                                let (smaller, larger) = set.clone().split(&split_var.val);
                                env.stack.push(Variable::val(Val::Tup(vector![ValRef::new(Val::Set(smaller)), ValRef::new(Val::Set(larger))])));
                                return Ok(None);
                            },
                            _ => {}
                        }
                    }
                }
                Err(Error::SetSplit)
            },
            Self::Empty => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Set(set) => {
                            env.stack.push(Variable::val(Val::Bool(set.is_empty())));
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::SetEmpty)
            },
            Self::Any => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Set(set) => {
                            env.stack.push(Variable::val(Val::Bool(!set.is_empty())));
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::SetAny)
            },
            Self::Len => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Set(set) => {
                            env.stack.push(Variable::val(Val::Num(Num::Int(set.len() as i64))));
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::SetLen)
            },
            Self::At => {
                if let Some(index_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        match var.val.read().deref() {
                            Val::Set(set) => {
                                match index_var.val.read().deref() {
                                    Val::Num(num) => {
                                        let index = num.int() as usize;
                                        if let Some(val) = set.iter().nth(index) {
                                            env.stack.push(Variable::refval(val.duplicate(false)));
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
                Err(Error::SetAt)
            },
            Self::PopFirst => {
                if let Some(var) = env.stack.pop() {
                    match var.val.write().deref_mut() {
                        Val::Set(set) => {
                            if let Some(min) = set.remove_min() {
                                env.stack.push(Variable::refval(min));
                            } else {
                                env.stack.push(Variable::val(Val::Null));
                            }
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::SetPopFirst)
            },
            Self::PopLast => {
                if let Some(var) = env.stack.pop() {
                    match var.val.write().deref_mut() {
                        Val::Set(set) => {
                            if let Some(max) = set.remove_max() {
                                env.stack.push(Variable::refval(max));
                            } else {
                                env.stack.push(Variable::val(Val::Null));
                            }
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::SetPopLast)
            },
            Self::Remove => {
                if let Some(remove_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        match var.val.write().deref_mut() {
                            Val::Set(set) => {
                                if let Some(val) = set.remove(&remove_var.val) {
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
                Err(Error::SetRemove)
            },
            Self::Union => {
                if let Some(other_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        match other_var.val.read().deref() {
                            Val::Set(other) => {
                                match var.val.write().deref_mut() {
                                    Val::Set(set) => {
                                        let new = set.clone().union(other.clone());
                                        env.stack.push(Variable::val(Val::Set(new)));
                                        return Ok(None);
                                    },
                                    _ => {}
                                }
                            },
                            _ => {}
                        }
                    }
                }
                Err(Error::SetUnion)
            },
            Self::Difference => {
                if let Some(other_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        match other_var.val.read().deref() {
                            Val::Set(other) => {
                                match var.val.write().deref_mut() {
                                    Val::Set(set) => {
                                        let new = set.clone().relative_complement(other.clone());
                                        env.stack.push(Variable::val(Val::Set(new)));
                                        return Ok(None);
                                    },
                                    _ => {}
                                }
                            },
                            _ => {}
                        }
                    }
                }
                Err(Error::SetDifference)
            },
            Self::Intersection => {
                if let Some(other_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        match other_var.val.read().deref() {
                            Val::Set(other) => {
                                match var.val.write().deref_mut() {
                                    Val::Set(set) => {
                                        let new = set.clone().intersection(other.clone());
                                        env.stack.push(Variable::val(Val::Set(new)));
                                        return Ok(None);
                                    },
                                    _ => {}
                                }
                            },
                            _ => {}
                        }
                    }
                }
                Err(Error::SetIntersection)
            },
            Self::SymmetricDifference => {
                if let Some(other_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        match other_var.val.read().deref() {
                            Val::Set(other) => {
                                match var.val.write().deref_mut() {
                                    Val::Set(set) => {
                                        let new = set.clone().symmetric_difference(other.clone());
                                        env.stack.push(Variable::val(Val::Set(new)));
                                        return Ok(None);
                                    },
                                    _ => {}
                                }
                            },
                            _ => {}
                        }
                    }
                }
                Err(Error::SetSymmetricDifference)
            },
            Self::Disjoint => {
                if let Some(other_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        match other_var.val.read().deref() {
                            Val::Set(other) => {
                                match var.val.write().deref_mut() {
                                    Val::Set(set) => {
                                        let new = set.clone().intersection(other.clone());
                                        env.stack.push(Variable::val(Val::Bool(new.is_empty())));
                                        return Ok(None);
                                    },
                                    _ => {}
                                }
                            },
                            _ => {}
                        }
                    }
                }
                Err(Error::SetDisjoint)
            },
            Self::Subset => {
                if let Some(other_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        match other_var.val.read().deref() {
                            Val::Set(other) => {
                                match var.val.read().deref() {
                                    Val::Set(set) => {
                                        let sub = set.is_subset(other);
                                        env.stack.push(Variable::val(Val::Bool(sub)));
                                        return Ok(None);
                                    },
                                    _ => {}
                                }
                            },
                            _ => {}
                        }
                    }
                }
                Err(Error::SetSubset)
            },
            Self::Superset => {
                if let Some(other_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        match other_var.val.read().deref() {
                            Val::Set(other) => {
                                match var.val.read().deref() {
                                    Val::Set(set) => {
                                        let sub = other.is_subset(set);
                                        env.stack.push(Variable::val(Val::Bool(sub)));
                                        return Ok(None);
                                    },
                                    _ => {}
                                }
                            },
                            _ => {}
                        }
                    }
                }
                Err(Error::SetSuperset)
            },
            Self::IsUniform => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Set(set) => {
                            let mut uniform_type;
                            if set.is_empty() {
                                uniform_type = Some(Type::Void.rt_type_of(&graph));
                            } else {
                                uniform_type = Some(set.get_min().unwrap().read().spec_type(&graph).rt_type_of(&graph));
                                for val in set.iter().skip(1) {
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
                Err(Error::SetIsUniform)
            },
            Self::ToUniform => {
                if let Some(type_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        match type_var.val.read().deref() {
                            Val::Str(type_str) => {
                                match parse_type_complete(type_str.as_str()) {
                                    Ok(ctype) => {
                                        match var.val.read().deref() {
                                            Val::Set(set) => {
                                                let context = env.self_ptr();
                                                for val in set.iter() {
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
                Err(Error::SetToUniform)
            },
        }
    }
}
