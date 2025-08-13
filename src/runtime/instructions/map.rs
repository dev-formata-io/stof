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
use imbl::{vector, OrdSet, Vector};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use crate::{model::Graph, runtime::{instruction::{Instruction, Instructions}, proc::ProcEnv, Error, Num, Val, Variable}};


lazy_static! {
    pub static ref NEW_MAP: Arc<dyn Instruction> = Arc::new(MapIns::NewMap);
    pub static ref PUSH_MAP: Arc<dyn Instruction> = Arc::new(MapIns::PushMap);

    pub static ref APPEND_MAP: Arc<dyn Instruction> = Arc::new(MapIns::AppendOther);
    pub static ref CLEAR_MAP: Arc<dyn Instruction> = Arc::new(MapIns::Clear);
    pub static ref CONTAINS_MAP: Arc<dyn Instruction> = Arc::new(MapIns::Contains);
    pub static ref FIRST_MAP: Arc<dyn Instruction> = Arc::new(MapIns::First);
    pub static ref LAST_MAP: Arc<dyn Instruction> = Arc::new(MapIns::Last);
    pub static ref GET_MAP: Arc<dyn Instruction> = Arc::new(MapIns::Get);
    pub static ref FIRST_REF_MAP: Arc<dyn Instruction> = Arc::new(MapIns::FirstRef);
    pub static ref LAST_REF_MAP: Arc<dyn Instruction> = Arc::new(MapIns::LastRef);
    pub static ref GET_REF_MAP: Arc<dyn Instruction> = Arc::new(MapIns::GetRef);
    pub static ref INSERT_MAP: Arc<dyn Instruction> = Arc::new(MapIns::Insert);
    pub static ref EMPTY_MAP: Arc<dyn Instruction> = Arc::new(MapIns::Empty);
    pub static ref ANY_MAP: Arc<dyn Instruction> = Arc::new(MapIns::Any);
    pub static ref KEYS_MAP: Arc<dyn Instruction> = Arc::new(MapIns::Keys);
    pub static ref VALUES_MAP: Arc<dyn Instruction> = Arc::new(MapIns::Values);
    pub static ref VALUES_REF_MAP: Arc<dyn Instruction> = Arc::new(MapIns::ValuesRef);
    pub static ref LEN_MAP: Arc<dyn Instruction> = Arc::new(MapIns::Len);
    pub static ref AT_MAP: Arc<dyn Instruction> = Arc::new(MapIns::At);
    pub static ref AT_REF_MAP: Arc<dyn Instruction> = Arc::new(MapIns::AtRef);
    pub static ref POP_FIRST_MAP: Arc<dyn Instruction> = Arc::new(MapIns::PopFirst);
    pub static ref POP_LAST_MAP: Arc<dyn Instruction> = Arc::new(MapIns::PopLast);
    pub static ref REMOVE_MAP: Arc<dyn Instruction> = Arc::new(MapIns::Remove);
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Map creation instructions.
pub enum MapIns {
    // Low-level for construction
    NewMap,
    PushMap,

    // High-level
    AppendMap((Arc<dyn Instruction>, Arc<dyn Instruction>)), // evaluate and add to the stack (push)

    // Library.
    AppendOther,
    Clear,
    Contains,
    First,
    Last,
    Get,
    FirstRef,
    LastRef,
    GetRef,
    Insert,
    Empty,
    Any,
    Keys,
    Values,
    ValuesRef,
    Len,
    At,
    AtRef,
    PopFirst,
    PopLast,
    Remove,
}
#[typetag::serde(name = "MapIns")]
impl Instruction for MapIns {
    fn exec(&self, env: &mut ProcEnv, _graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        match self {
            Self::NewMap => {
                env.stack.push(Variable::val(Val::Map(Default::default())));
                Ok(None)
            },
            Self::PushMap => {
                if let Some(value_var) = env.stack.pop() {
                    if let Some(key_var) = env.stack.pop() {
                        if let Some(map_var) = env.stack.pop() {
                            {
                                let mut val = map_var.val.write();
                                let val = val.deref_mut();
                                match &mut *val {
                                    Val::Map(map) => {
                                        map.insert(key_var.val, value_var.val);
                                    },
                                    _ => {}
                                }
                            }
                            env.stack.push(map_var);
                        }
                    }
                }
                Ok(None)
            },

            /*****************************************************************************
             * High-level.
             *****************************************************************************/
            Self::AppendMap((key, value)) => {
                let mut instructions = Instructions::default();
                instructions.push(key.clone());
                instructions.push(value.clone());
                instructions.push(PUSH_MAP.clone());
                Ok(Some(instructions))
            },

            /*****************************************************************************
             * Library.
             *****************************************************************************/
            Self::AppendOther => {
                if let Some(other) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        match other.val.read().deref() {
                            Val::Map(other) => {
                                match var.val.write().deref_mut() {
                                    Val::Map(map) => {
                                        for (k, v) in other.iter() {
                                            map.insert(k.duplicate(false), v.duplicate(false));
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
                Err(Error::MapAppendOther)
            },
            Self::Clear => {
                if let Some(var) = env.stack.pop() {
                    match var.val.write().deref_mut() {
                        Val::Map(map) => {
                            map.clear();
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::MapClear)
            },
            Self::Contains => {
                if let Some(test_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        match var.val.read().deref() {
                            Val::Map(map) => {
                                let contains = map.contains_key(&test_var.val);
                                env.stack.push(Variable::val(Val::Bool(contains)));
                                return Ok(None);
                            },
                            _ => {}
                        }
                    }
                }
                Err(Error::MapContains)
            },
            Self::First => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Map(map) => {
                            if let Some((key, val)) = map.get_min() {
                                env.stack.push(Variable::val(Val::Tup(vector![key.duplicate(false), val.duplicate(false)])));
                            } else {
                                env.stack.push(Variable::val(Val::Null));
                            }
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::MapFirst)
            },
            Self::Last => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Map(map) => {
                            if let Some((key, val)) = map.get_max() {
                                env.stack.push(Variable::val(Val::Tup(vector![key.duplicate(false), val.duplicate(false)])));
                            } else {
                                env.stack.push(Variable::val(Val::Null));
                            }
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::MapLast)
            },
            Self::FirstRef => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Map(map) => {
                            if let Some((key, val)) = map.get_min() {
                                env.stack.push(Variable::val(Val::Tup(vector![key.duplicate(false), val.duplicate(true)])));
                            } else {
                                env.stack.push(Variable::val(Val::Null));
                            }
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::MapFirst)
            },
            Self::LastRef => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Map(map) => {
                            if let Some((key, val)) = map.get_max() {
                                env.stack.push(Variable::val(Val::Tup(vector![key.duplicate(false), val.duplicate(true)])));
                            } else {
                                env.stack.push(Variable::val(Val::Null));
                            }
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::MapLast)
            },
            Self::Get => {
                if let Some(search_val) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        match var.val.read().deref() {
                            Val::Map(map) => {
                                if let Some(val) = map.get(&search_val.val) {
                                    env.stack.push(Variable::refval(val.duplicate(false)));
                                } else {
                                    env.stack.push(Variable::val(Val::Null));
                                }
                                return Ok(None);
                            },
                            _ => {}
                        }
                    }
                }
                Err(Error::MapGet)
            },
            Self::GetRef => {
                if let Some(search_val) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        match var.val.read().deref() {
                            Val::Map(map) => {
                                if let Some(val) = map.get(&search_val.val) {
                                    env.stack.push(Variable::refval(val.duplicate(true)));
                                } else {
                                    env.stack.push(Variable::val(Val::Null));
                                }
                                return Ok(None);
                            },
                            _ => {}
                        }
                    }
                }
                Err(Error::MapGet)
            },
            Self::Insert => {
                if let Some(value_var) = env.stack.pop() {
                    if let Some(key_var) = env.stack.pop() {
                        if let Some(var) = env.stack.pop() {
                            match var.val.write().deref_mut() {
                                Val::Map(map) => {
                                    if let Some(old) = map.insert(key_var.val, value_var.val) {
                                        env.stack.push(Variable::refval(old));
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
                Err(Error::MapInsert)
            },
            Self::Empty => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Map(map) => {
                            env.stack.push(Variable::val(Val::Bool(map.is_empty())));
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::MapEmpty)
            },
            Self::Any => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Map(map) => {
                            env.stack.push(Variable::val(Val::Bool(!map.is_empty())));
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::MapAny)
            },
            Self::Keys => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Map(map) => {
                            let mut keys = OrdSet::default();
                            for key in map.keys() {
                                keys.insert(key.duplicate(false));
                            }
                            env.stack.push(Variable::val(Val::Set(keys)));
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::MapKeys)
            },
            Self::Values => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Map(map) => {
                            let mut values = Vector::default();
                            for val in map.values() {
                                values.push_back(val.duplicate(false));
                            }
                            env.stack.push(Variable::val(Val::List(values)));
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::MapValues)
            },
            Self::ValuesRef => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Map(map) => {
                            let mut values = Vector::default();
                            for val in map.values() {
                                values.push_back(val.duplicate(true));
                            }
                            env.stack.push(Variable::val(Val::List(values)));
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::MapValues)
            },
            Self::Len => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Map(map) => {
                            env.stack.push(Variable::val(Val::Num(Num::Int(map.len() as i64))));
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::MapLen)
            },
            Self::At => {
                if let Some(index_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        match var.val.read().deref() {
                            Val::Map(map) => {
                                match index_var.val.read().deref() {
                                    Val::Num(num) => {
                                        let index = num.int() as usize;
                                        if let Some((key, val)) = map.iter().nth(index) {
                                            env.stack.push(Variable::val(Val::Tup(vector![key.duplicate(false), val.duplicate(false)])));
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
                Err(Error::MapAt)
            },
            Self::AtRef => {
                if let Some(index_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        match var.val.read().deref() {
                            Val::Map(map) => {
                                match index_var.val.read().deref() {
                                    Val::Num(num) => {
                                        let index = num.int() as usize;
                                        if let Some((key, val)) = map.iter().nth(index) {
                                            env.stack.push(Variable::val(Val::Tup(vector![key.duplicate(false), val.duplicate(true)])));
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
                Err(Error::MapAt)
            },
            Self::PopFirst => {
                if let Some(var) = env.stack.pop() {
                    match var.val.write().deref_mut() {
                        Val::Map(map) => {
                            let (value, new_map) = map.without_min_with_key();
                            *map = new_map;

                            if let Some((key, val)) = value {
                                env.stack.push(Variable::val(Val::Tup(vector![key, val])));
                            } else {
                                env.stack.push(Variable::val(Val::Null));
                            }
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::MapPopFirst)
            },
            Self::PopLast => {
                if let Some(var) = env.stack.pop() {
                    match var.val.write().deref_mut() {
                        Val::Map(map) => {
                            let (value, new_map) = map.without_max_with_key();
                            *map = new_map;
                            
                            if let Some((key, val)) = value {
                                env.stack.push(Variable::val(Val::Tup(vector![key, val])));
                            } else {
                                env.stack.push(Variable::val(Val::Null));
                            }
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::MapPopLast)
            },
            Self::Remove => {
                if let Some(search_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        match var.val.write().deref_mut() {
                            Val::Map(map) => {
                                if let Some(val) = map.remove(&search_var.val) {
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
                Err(Error::MapRemove)
            },
        }
    }
}
