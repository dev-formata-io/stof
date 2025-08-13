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
use arcstr::{literal, ArcStr};
use imbl::Vector;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use crate::{model::{string::ops::{str_at, str_contains, str_ends_with, str_first, str_index_of, str_last, str_len, str_lower, str_push, str_replace, str_split, str_starts_with, str_substr, str_trim, str_trim_end, str_trim_start, str_upper}, Graph}, runtime::{instruction::{Instruction, Instructions}, proc::ProcEnv, Error, Num, Val, ValRef, Variable}};

mod ops;

/// Library name.
pub(self) const STR_LIB: ArcStr = literal!("Str");


/// Add the string library to a graph.
pub fn insert_string_lib(graph: &mut Graph) {
    graph.insert_libfunc(str_len());
    graph.insert_libfunc(str_at());
    graph.insert_libfunc(str_first());
    graph.insert_libfunc(str_last());
    graph.insert_libfunc(str_starts_with());
    graph.insert_libfunc(str_ends_with());
    graph.insert_libfunc(str_push());
    graph.insert_libfunc(str_contains());
    graph.insert_libfunc(str_index_of());
    graph.insert_libfunc(str_replace());
    graph.insert_libfunc(str_split());
    graph.insert_libfunc(str_upper());
    graph.insert_libfunc(str_lower());
    graph.insert_libfunc(str_trim());
    graph.insert_libfunc(str_trim_start());
    graph.insert_libfunc(str_trim_end());
    graph.insert_libfunc(str_substr());
}


lazy_static! {
    pub(self) static ref LEN: Arc<dyn Instruction> = Arc::new(StrIns::Len);
    pub(self) static ref AT: Arc<dyn Instruction> = Arc::new(StrIns::At);
    pub(self) static ref FIRST: Arc<dyn Instruction> = Arc::new(StrIns::First);
    pub(self) static ref LAST: Arc<dyn Instruction> = Arc::new(StrIns::Last);
    pub(self) static ref STARTS_WITH: Arc<dyn Instruction> = Arc::new(StrIns::StartsWith);
    pub(self) static ref ENDS_WITH: Arc<dyn Instruction> = Arc::new(StrIns::EndsWith);
    pub(self) static ref PUSH: Arc<dyn Instruction> = Arc::new(StrIns::Push);
    pub(self) static ref CONTAINS: Arc<dyn Instruction> = Arc::new(StrIns::Contains);
    pub(self) static ref INDEX_OF: Arc<dyn Instruction> = Arc::new(StrIns::IndexOf);
    pub(self) static ref REPLACE: Arc<dyn Instruction> = Arc::new(StrIns::Replace);
    pub(self) static ref SPLIT: Arc<dyn Instruction> = Arc::new(StrIns::Split);
    pub(self) static ref UPPER: Arc<dyn Instruction> = Arc::new(StrIns::Upper);
    pub(self) static ref LOWER: Arc<dyn Instruction> = Arc::new(StrIns::Lower);
    pub(self) static ref TRIM: Arc<dyn Instruction> = Arc::new(StrIns::Trim);
    pub(self) static ref TRIM_START: Arc<dyn Instruction> = Arc::new(StrIns::TrimStart);
    pub(self) static ref TRIM_END: Arc<dyn Instruction> = Arc::new(StrIns::TrimEnd);
    pub(self) static ref SUBSTRING: Arc<dyn Instruction> = Arc::new(StrIns::Substring);
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// String instructions.
pub enum StrIns {
    Len,
    At,
    First,
    Last,
    StartsWith,
    EndsWith,
    Push,
    Contains,
    IndexOf,
    Replace,
    Split,
    Upper,
    Lower,
    Trim,
    TrimStart,
    TrimEnd,
    Substring,
}
#[typetag::serde(name = "StrIns")]
impl Instruction for StrIns {
    fn exec(&self, env: &mut ProcEnv, _graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        match self {
            Self::Len => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Str(str) => {
                            env.stack.push(Variable::val(Val::Num(Num::Int(str.len() as i64))));
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::StrLen)
            },
            Self::At => {
                if let Some(index_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        if let Some(num) = index_var.val.write().try_num() {
                            let mut index = num.int() as usize;
                            match var.val.read().deref() {
                                Val::Str(str) => {
                                    if index >= str.len() {
                                        index = str.len() - 1;
                                    }
                                    let char = str.as_bytes()[index] as char;
                                    env.stack.push(Variable::val(Val::Str(String::from(char).into())));
                                    return Ok(None);
                                },
                                _ => {}
                            }
                        }
                    }
                }
                Err(Error::StrAt)
            },
            Self::First => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Str(str) => {
                            let char;
                            if str.len() < 1 {
                                char = ' ';
                            } else {
                                char = str.as_bytes()[0] as char;
                            }
                            env.stack.push(Variable::val(Val::Str(String::from(char).into())));
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::StrFirst)
            },
            Self::Last => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Str(str) => {
                            let char;
                            if str.len() < 1 {
                                char = ' ';
                            } else {
                                char = str.as_bytes()[str.len() - 1] as char;
                            }
                            env.stack.push(Variable::val(Val::Str(String::from(char).into())));
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::StrLast)
            },
            Self::StartsWith => {
                if let Some(test_val) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        match test_val.val.read().deref() {
                            Val::Str(test) => {
                                match var.val.read().deref() {
                                    Val::Str(str) => {
                                        let val = str.starts_with(test.as_str());
                                        env.stack.push(Variable::val(Val::Bool(val)));
                                        return Ok(None);
                                    },
                                    _ => {}
                                }
                            },
                            _ => {},
                        }
                    }
                }
                Err(Error::StrStartsWith)
            },
            Self::EndsWith => {
                if let Some(test_val) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        match test_val.val.read().deref() {
                            Val::Str(test) => {
                                match var.val.read().deref() {
                                    Val::Str(str) => {
                                        let val = str.ends_with(test.as_str());
                                        env.stack.push(Variable::val(Val::Bool(val)));
                                        return Ok(None);
                                    },
                                    _ => {}
                                }
                            },
                            _ => {},
                        }
                    }
                }
                Err(Error::StrEndsWith)
            },
            Self::Push => {
                if let Some(to_push) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        let mut push = false;
                        match to_push.val.read().deref() {
                            Val::Str(to_push) => {
                                match var.val.write().deref_mut() {
                                    Val::Str(val) => {
                                        *val = ArcStr::from(format!("{val}{to_push}"));
                                        push = true;
                                    },
                                    _ => {}
                                }
                            },
                            _ => {}
                        }
                        if push {
                            return Ok(None);
                        }
                    }
                }
                Err(Error::StrPush)
            },
            Self::Contains => {
                if let Some(test_val) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        match test_val.val.read().deref() {
                            Val::Str(test) => {
                                match var.val.read().deref() {
                                    Val::Str(str) => {
                                        let val = str.contains(test.as_str());
                                        env.stack.push(Variable::val(Val::Bool(val)));
                                        return Ok(None);
                                    },
                                    _ => {}
                                }
                            },
                            _ => {},
                        }
                    }
                }
                Err(Error::StrContains)
            },
            Self::IndexOf => {
                if let Some(test_val) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        match test_val.val.read().deref() {
                            Val::Str(test) => {
                                match var.val.read().deref() {
                                    Val::Str(str) => {
                                        let val = str.find(test.as_str());
                                        if let Some(val) = val {
                                            env.stack.push(Variable::val(Val::Num(Num::Int(val as i64))));
                                        } else {
                                            env.stack.push(Variable::val(Val::Num(Num::Int(-1))));
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
                Err(Error::StrIndexOf)
            },
            Self::Replace => {
                if let Some(replace_var) = env.stack.pop() {
                    if let Some(find_var) = env.stack.pop() {
                        if let Some(var) = env.stack.pop() {
                            match replace_var.val.read().deref() {
                                Val::Str(replace_with) => {
                                    match find_var.val.read().deref() {
                                        Val::Str(find) => {
                                            match var.val.read().deref() {
                                                Val::Str(val) => {
                                                    let v = val.replace(find.as_str(), replace_with.as_str());
                                                    env.stack.push(Variable::val(Val::Str(ArcStr::from(v))));
                                                    return Ok(None);
                                                },
                                                _ => {}
                                            }
                                        },
                                        _ => {}
                                    }
                                },
                                _ => {},
                            }
                        }
                    }
                }
                Err(Error::StrReplace)
            },
            Self::Split => {
                if let Some(split_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        match split_var.val.read().deref() {
                            Val::Str(split) => {
                                match var.val.read().deref() {
                                    Val::Str(str) => {
                                        let list = str.split(split.as_str())
                                            .map(|v| ValRef::new(Val::Str(ArcStr::from(v))))
                                            .collect::<Vector<ValRef<Val>>>();
                                        env.stack.push(Variable::val(Val::List(list)));
                                        return Ok(None);
                                    },
                                    _ => {}
                                }
                            },
                            _ => {},
                        }
                    }
                }
                Err(Error::StrSplit)
            },
            Self::Upper => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Str(str) => {
                            env.stack.push(Variable::val(Val::Str(ArcStr::from(str.to_uppercase()))));
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::StrUpper)
            },
            Self::Lower => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Str(str) => {
                            env.stack.push(Variable::val(Val::Str(ArcStr::from(str.to_lowercase()))));
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::StrLower)
            },
            Self::Trim => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Str(str) => {
                            env.stack.push(Variable::val(Val::Str(ArcStr::from(str.trim()))));
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::StrTrim)
            },
            Self::TrimStart => {
                if let Some(var) = env.stack.pop() {
                    match var.val.write().deref_mut() {
                        Val::Str(str) => {
                            env.stack.push(Variable::val(Val::Str(ArcStr::from(str.trim_start()))));
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::StrTrimStart)
            },
            Self::TrimEnd => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Str(str) => {
                            env.stack.push(Variable::val(Val::Str(ArcStr::from(str.trim_end()))));
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::StrTrimEnd)
            },
            Self::Substring => {
                if let Some(end_var) = env.stack.pop() {
                    if let Some(start_var) = env.stack.pop() {
                        if let Some(var) = env.stack.pop() {
                            match end_var.val.read().deref() {
                                Val::Num(end) => {
                                    match start_var.val.read().deref() {
                                        Val::Num(start) => {
                                            match var.val.read().deref() {
                                                Val::Str(val) => {
                                                    let start = start.int();
                                                    let end = end.int();

                                                    let start_u;
                                                    let end_u;
                                                    
                                                    if start < 0 { start_u = 0; }
                                                    else if start as usize > val.len() { start_u = val.len(); }
                                                    else { start_u = start as usize; }

                                                    if end < 0 { end_u = val.len(); }
                                                    else if end as usize > val.len() { end_u = val.len(); }
                                                    else if end < start { end_u = start_u; }
                                                    else { end_u = end as usize; }

                                                    let substr = val.substr(start_u..end_u);
                                                    env.stack.push(Variable::val(Val::Str(ArcStr::from(substr.as_str()))));
                                                    return Ok(None);
                                                },
                                                _ => {}
                                            }
                                        },
                                        _ => {}
                                    }
                                },
                                _ => {}
                            }
                        }
                    }
                }
                Err(Error::StrSubstring)
            },
        }
    }
}
