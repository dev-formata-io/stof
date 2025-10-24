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

mod ops;
use ops::{prompt_str};
use std::{ops::{Deref, DerefMut}, sync::Arc};
use arcstr::{literal, ArcStr};
use imbl::Vector;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use crate::{model::{prompt::ops::{prompt_any, prompt_at, prompt_clear, prompt_empty, prompt_insert, prompt_len, prompt_pop, prompt_prompts, prompt_push, prompt_remove, prompt_replace, prompt_reverse, prompt_set_tag, prompt_set_text, prompt_tag, prompt_text}, Graph}, runtime::{instruction::{Instruction, Instructions}, proc::ProcEnv, Error, Num, Prompt, Val, ValRef, Variable}};


/// Prompt lib.
pub(self) const PROMPT_LIB: ArcStr = literal!("Prompt");


/// Add the prompt lib to a graph.
pub fn insert_prompt_lib(graph: &mut Graph) {
    graph.insert_libfunc(prompt_str());
    graph.insert_libfunc(prompt_text());
    graph.insert_libfunc(prompt_tag());
    graph.insert_libfunc(prompt_prompts());

    graph.insert_libfunc(prompt_set_text());
    graph.insert_libfunc(prompt_set_tag());

    graph.insert_libfunc(prompt_len());
    graph.insert_libfunc(prompt_at());
    graph.insert_libfunc(prompt_any());
    graph.insert_libfunc(prompt_empty());

    graph.insert_libfunc(prompt_push());
    graph.insert_libfunc(prompt_pop());
    graph.insert_libfunc(prompt_clear());
    graph.insert_libfunc(prompt_reverse());
    graph.insert_libfunc(prompt_remove());
    graph.insert_libfunc(prompt_insert());
    graph.insert_libfunc(prompt_replace());
}


lazy_static! {
    pub static ref PROMPT_STR: Arc<dyn Instruction> = Arc::new(PromptIns::Str);
    pub static ref PROMPT_TEXT: Arc<dyn Instruction> = Arc::new(PromptIns::Text);
    pub static ref PROMPT_TAG: Arc<dyn Instruction> = Arc::new(PromptIns::Tag);
    pub static ref PROMPT_PROMPTS: Arc<dyn Instruction> = Arc::new(PromptIns::Prompts);
    pub static ref PROMPT_SET_TEXT: Arc<dyn Instruction> = Arc::new(PromptIns::SetText);
    pub static ref PROMPT_SET_TAG: Arc<dyn Instruction> = Arc::new(PromptIns::SetTag);
    pub static ref PROMPT_LEN: Arc<dyn Instruction> = Arc::new(PromptIns::Len);
    pub static ref PROMPT_AT: Arc<dyn Instruction> = Arc::new(PromptIns::At);
    pub static ref PROMPT_EMPTY: Arc<dyn Instruction> = Arc::new(PromptIns::Empty);
    pub static ref PROMPT_ANY: Arc<dyn Instruction> = Arc::new(PromptIns::Any);
    pub static ref PROMPT_PUSH: Arc<dyn Instruction> = Arc::new(PromptIns::Push);
    pub static ref PROMPT_POP: Arc<dyn Instruction> = Arc::new(PromptIns::Pop);
    pub static ref PROMPT_CLEAR: Arc<dyn Instruction> = Arc::new(PromptIns::Clear);
    pub static ref PROMPT_REVERSE: Arc<dyn Instruction> = Arc::new(PromptIns::Reverse);
    pub static ref PROMPT_REMOVE: Arc<dyn Instruction> = Arc::new(PromptIns::Remove);
    pub static ref PROMPT_INSERT: Arc<dyn Instruction> = Arc::new(PromptIns::Insert);
    pub static ref PROMPT_REPLACE: Arc<dyn Instruction> = Arc::new(PromptIns::Replace);
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Prompt instructions.
pub enum PromptIns {
    Str,
    Text,
    Tag,
    Prompts,

    SetText,
    SetTag, // set to null to clear tag

    Len,
    At,
    Empty,
    Any,

    Push,
    Pop,
    Clear,
    Reverse,
    Remove, // by index
    Insert, // at index
    Replace, // replace at index
}
#[typetag::serde(name = "PromptIns")]
impl Instruction for PromptIns {
    fn exec(&self, env: &mut ProcEnv, graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        match self {
            Self::Str => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Prompt(prompt) => {
                            env.stack.push(Variable::val(Val::Str(prompt.to_string().into())));
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::PromptStr)
            },
            Self::Text => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Prompt(prompt) => {
                            env.stack.push(Variable::val(Val::Str(prompt.text.clone())));
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::PromptText)
            },
            Self::Tag => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Prompt(prompt) => {
                            if let Some(tag) = &prompt.tag {
                                env.stack.push(Variable::val(Val::Str(tag.clone())));
                            } else {
                                env.stack.push(Variable::val(Val::Null));
                            }
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::PromptTag)
            },
            Self::Prompts => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Prompt(prompt) => {
                            let mut list = Vector::default();
                            for sub in &prompt.prompts {
                                list.push_back(ValRef::new(Val::Prompt(sub.clone())));
                            }
                            env.stack.push(Variable::val(Val::List(list)));
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::PromptPrompts)
            },

            Self::SetText => {
                if let Some(text_var) = env.stack.pop() {
                    if let Some(prompt) = env.stack.pop() {
                        let text = text_var.val.read().print(&graph);
                        match prompt.val.write().deref_mut() {
                            Val::Prompt(prompt) => {
                                prompt.text = text.into();
                                return Ok(None);
                            },
                            _ => {}
                        }
                    }
                }
                Err(Error::PromptSetText)
            },
            Self::SetTag => {
                if let Some(tag_var) = env.stack.pop() {
                    if let Some(prompt) = env.stack.pop() {
                        match prompt.val.write().deref_mut() {
                            Val::Prompt(prompt) => {
                                match tag_var.val.read().deref() {
                                    Val::Str(tag) => {
                                        prompt.tag = Some(tag.clone());
                                    },
                                    _ => {
                                        prompt.tag = None;
                                    },
                                }
                                return Ok(None);
                            },
                            _ => {}
                        }
                    }
                }
                Err(Error::PromptSetTag)
            },

            Self::Len => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Prompt(prompt) => {
                            env.stack.push(Variable::val(Val::Num(Num::Int(prompt.prompts.len() as i64))));
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::PromptLen)
            },
            Self::At => {
                if let Some(index_var) = env.stack.pop() {
                    if let Some(prompt_var) = env.stack.pop() {
                        match index_var.val.read().deref() {
                            Val::Num(num) => {
                                match prompt_var.val.read().deref() {
                                    Val::Prompt(prompt) => {
                                        if prompt.prompts.is_empty() {
                                            env.stack.push(Variable::val(Val::Null));
                                        } else {
                                            let idx = i64::min(i64::max(num.int(), 0), (prompt.prompts.len() - 1) as i64) as usize;
                                            if let Some(prompt) = prompt.prompts.get(idx) {
                                                env.stack.push(Variable::val(Val::Prompt(prompt.clone())));
                                            }
                                        }
                                        return Ok(None);
                                    },
                                    _ => {},
                                }
                            },
                            _ => {}
                        }
                    }
                }
                Err(Error::PromptAt)
            },
            Self::Any => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Prompt(prompt) => {
                            env.stack.push(Variable::val(Val::Bool(!prompt.prompts.is_empty())));
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::PromptAny)
            },
            Self::Empty => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Prompt(prompt) => {
                            env.stack.push(Variable::val(Val::Bool(prompt.prompts.is_empty())));
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::PromptEmpty)
            },

            Self::Push => {
                if let Some(to_push_var) = env.stack.pop() {
                    if let Some(prompt_var) = env.stack.pop() {
                        match prompt_var.val.write().deref_mut() {
                            Val::Prompt(prompt) => {
                                match to_push_var.val.read().deref() {
                                    Val::Prompt(push_prompt) => {
                                        prompt.prompts.push_back(push_prompt.clone());
                                        return Ok(None);
                                    },
                                    Val::Str(push_str) => {
                                        prompt.prompts.push_back(Prompt::from(push_str));
                                        return Ok(None);
                                    },
                                    _ => {}
                                }
                            },
                            _ => {},
                        }
                    }
                }
                Err(Error::PromptPush)
            },
            Self::Pop => {
                if let Some(prompt_var) = env.stack.pop() {
                    match prompt_var.val.write().deref_mut() {
                        Val::Prompt(prompt) => {
                            if let Some(popped) = prompt.prompts.pop_back() {
                                env.stack.push(Variable::val(Val::Prompt(popped)));
                            } else {
                                env.stack.push(Variable::val(Val::Null));
                            }
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::PromptPop)
            },
            Self::Clear => {
                if let Some(prompt_var) = env.stack.pop() {
                    match prompt_var.val.write().deref_mut() {
                        Val::Prompt(prompt) => {
                            prompt.prompts.clear();
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::PromptClear)
            },
            Self::Reverse => {
                if let Some(prompt_var) = env.stack.pop() {
                    match prompt_var.val.write().deref_mut() {
                        Val::Prompt(prompt) => {
                            let mut prompts = Vector::default();
                            for p in prompt.prompts.iter().rev() {
                                prompts.push_back(p.clone());
                            }
                            prompt.prompts = prompts;
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::PromptReverse)
            },
            Self::Remove => {
                if let Some(index_var) = env.stack.pop() {
                    if let Some(prompt_var) = env.stack.pop() {
                        match prompt_var.val.write().deref_mut() {
                            Val::Prompt(prompt) => {
                                match index_var.val.read().deref() {
                                    Val::Num(num) => {
                                        if prompt.prompts.is_empty() {
                                            env.stack.push(Variable::val(Val::Null));
                                        } else {
                                            let idx = i64::min(i64::max(num.int(), 0), (prompt.prompts.len() - 1) as i64);
                                            let removed = prompt.prompts.remove(idx as usize);
                                            env.stack.push(Variable::val(Val::Prompt(removed)));
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
                Err(Error::PromptRemove)
            },
            Self::Insert => {
                if let Some(to_insert_var) = env.stack.pop() {
                    if let Some(index_var) = env.stack.pop() {
                        if let Some(prompt_var) = env.stack.pop() {
                            match prompt_var.val.write().deref_mut() {
                                Val::Prompt(prompt) => {
                                    match to_insert_var.val.read().deref() {
                                        Val::Prompt(insert) => {
                                            match index_var.val.read().deref() {
                                                Val::Num(num) => {
                                                    if prompt.prompts.is_empty() {
                                                        prompt.prompts.push_back(insert.clone());
                                                    } else {
                                                        let idx = i64::min(i64::max(num.int(), 0), prompt.prompts.len() as i64);
                                                        prompt.prompts.insert(idx as usize, insert.clone());
                                                    }
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
                Err(Error::PromptInsert)
            },
            Self::Replace => {
                if let Some(to_insert_var) = env.stack.pop() {
                    if let Some(index_var) = env.stack.pop() {
                        if let Some(prompt_var) = env.stack.pop() {
                            match prompt_var.val.write().deref_mut() {
                                Val::Prompt(prompt) => {
                                    match to_insert_var.val.read().deref() {
                                        Val::Prompt(insert) => {
                                            match index_var.val.read().deref() {
                                                Val::Num(num) => {
                                                    if prompt.prompts.is_empty() {
                                                        prompt.prompts.push_back(insert.clone());
                                                    } else {
                                                        let idx = i64::min(i64::max(num.int(), 0), (prompt.prompts.len() - 1) as i64) as usize;
                                                        prompt.prompts.remove(idx);
                                                        prompt.prompts.insert(idx, insert.clone());
                                                    }
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
                Err(Error::PromptReplace)
            },
        }
    }
}
