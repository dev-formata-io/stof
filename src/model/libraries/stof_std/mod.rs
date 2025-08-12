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

use std::{mem::swap, ops::{Deref, DerefMut}, sync::Arc, time::Duration};
use arcstr::{literal, ArcStr};
use bytes::Bytes;
use imbl::{vector, OrdSet, Vector};
use lazy_static::lazy_static;
use nanoid::nanoid;
use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};
use crate::{model::{stof_std::{assert::{assert, assert_eq, assert_neq, assert_not, throw}, containers::{std_copy, std_drop, std_funcs, std_list, std_map, std_set, std_shallow_drop, std_swap}, exit::stof_exit, ops::{std_blobify, std_callstack, std_format_content_type, std_formats, std_graph_id, std_has_format, std_has_lib, std_libs, std_max, std_min, std_nanoid, std_parse, std_stringify, std_trace, std_tracestack}, print::{dbg, err, pln, string}, sleep::stof_sleep}, Field, Func, Graph, Prototype, SPath, SELF_STR_KEYWORD, SUPER_STR_KEYWORD}, runtime::{instruction::{Instruction, Instructions}, instructions::{call::FuncCall, list::{NEW_LIST, PUSH_LIST}, map::{NEW_MAP, PUSH_MAP}, set::{NEW_SET, PUSH_SET}, Base, DUPLICATE, EXIT}, proc::ProcEnv, Error, Type, Units, Val, ValRef, Variable}};

mod print;
mod sleep;
mod assert;
mod exit;
mod containers;
mod ops;


/// Add the std library to a graph.
pub fn stof_std_lib(graph: &mut Graph) {
    graph.insert_libfunc(string());
    graph.insert_libfunc(pln());
    graph.insert_libfunc(dbg());
    graph.insert_libfunc(err());
    graph.insert_libfunc(stof_sleep());
    graph.insert_libfunc(throw());
    graph.insert_libfunc(stof_exit());

    graph.insert_libfunc(assert());
    graph.insert_libfunc(assert_not());
    graph.insert_libfunc(assert_eq());
    graph.insert_libfunc(assert_neq());

    graph.insert_libfunc(std_list());
    graph.insert_libfunc(std_set());
    graph.insert_libfunc(std_map());

    graph.insert_libfunc(std_copy());
    graph.insert_libfunc(std_swap());
    graph.insert_libfunc(std_drop());
    graph.insert_libfunc(std_shallow_drop());

    graph.insert_libfunc(std_funcs());

    graph.insert_libfunc(std_parse());
    graph.insert_libfunc(std_stringify());
    graph.insert_libfunc(std_blobify());

    graph.insert_libfunc(std_has_format());
    graph.insert_libfunc(std_formats());
    graph.insert_libfunc(std_format_content_type());

    graph.insert_libfunc(std_has_lib());
    graph.insert_libfunc(std_libs());

    graph.insert_libfunc(std_nanoid());
    graph.insert_libfunc(std_graph_id());

    graph.insert_libfunc(std_max());
    graph.insert_libfunc(std_min());

    graph.insert_libfunc(std_callstack());
    graph.insert_libfunc(std_trace());
    graph.insert_libfunc(std_tracestack());
}


/// Library name.
pub(self) const STD_LIB: ArcStr = literal!("Std");


// Static instructions.
lazy_static! {
    pub(self) static ref SLEEP: Arc<dyn Instruction> = Arc::new(StdIns::Sleep);
    pub(crate) static ref THROW: Arc<dyn Instruction> = Arc::new(StdIns::Throw);
    pub(self) static ref ASSERT: Arc<dyn Instruction> = Arc::new(StdIns::Assert);
    pub(self) static ref ASSERT_NOT: Arc<dyn Instruction> = Arc::new(StdIns::AssertNot);
    pub(self) static ref ASSERT_EQ: Arc<dyn Instruction> = Arc::new(StdIns::AssertEq);
    pub(self) static ref ASSERT_NEQ: Arc<dyn Instruction> = Arc::new(StdIns::AssertNeq);

    pub(crate) static ref COPY: Arc<dyn Instruction> = Arc::new(StdIns::Copy);
    pub(self) static ref SWAP: Arc<dyn Instruction> = Arc::new(StdIns::Swap);

    pub(self) static ref FUNCTIONS: Arc<dyn Instruction> = Arc::new(StdIns::Functions);

    pub(self) static ref PARSE: Arc<dyn Instruction> = Arc::new(StdIns::Parse);
    pub(self) static ref BLOBIFY: Arc<dyn Instruction> = Arc::new(StdIns::Blobify);
    pub(self) static ref STRINGIFY: Arc<dyn Instruction> = Arc::new(StdIns::Stringify);

    pub(self) static ref HAS_FORMAT: Arc<dyn Instruction> = Arc::new(StdIns::HasFormat);
    pub(self) static ref FORMATS: Arc<dyn Instruction> = Arc::new(StdIns::Formats);
    pub(self) static ref FORMAT_CONTENT_TYPE: Arc<dyn Instruction> = Arc::new(StdIns::FormatContentType);
    pub(self) static ref HAS_LIB: Arc<dyn Instruction> = Arc::new(StdIns::HasLib);
    pub(self) static ref LIBS: Arc<dyn Instruction> = Arc::new(StdIns::Libs);
    pub(self) static ref NANO_ID: Arc<dyn Instruction> = Arc::new(StdIns::NanoId);
    pub(self) static ref GRAPH_ID: Arc<dyn Instruction> = Arc::new(StdIns::GraphId);
    pub(self) static ref CALLSTACK: Arc<dyn Instruction> = Arc::new(StdIns::Callstack);
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Standard Lib Instruction.
pub enum StdIns {
    String(usize),
    Pln(usize),
    Dbg(usize),
    Err(usize),

    Throw,
    Sleep,

    Exit(usize),

    Assert,
    AssertNot,
    AssertEq,
    AssertNeq,

    List(usize),
    Set(usize),
    Map(usize),

    Copy,
    Swap,

    Functions,

    ObjDropped(usize),
    Drop(usize),
    ShallowDrop(usize),

    Parse,
    Blobify,
    Stringify,

    HasFormat,
    Formats,
    FormatContentType,

    HasLib,
    Libs,

    NanoId,
    GraphId,

    Min(usize),
    Max(usize),

    Callstack,
    Trace(usize),
    TraceStack,
}
#[typetag::serde(name = "StdIns")]
impl Instruction for StdIns {
    fn exec(&self, env: &mut ProcEnv, graph: &mut Graph) -> Result<Option<Instructions> , Error> {
        match self {
            Self::String(arg_count) => {
                let mut values = Vec::new();
                for _ in 0..*arg_count {
                    if let Some(var) = env.stack.pop() {
                        values.push(var);
                    } else {
                        return Err(Error::StackError);
                    }
                }
                let mut output = Vec::new();
                let mut seen_str = false;
                for var in values.into_iter().rev() {
                    if !seen_str {
                        if var.gen_type() == Type::Str { seen_str = true; }
                    }
                    let out = var.val.read().print(&graph);
                    output.push(out);
                }
                let mut sep = "";
                if !seen_str { sep = ", " }

                let out = format!("{}", output.join(sep));
                env.stack.push(Variable::val(Val::Str(out.into())));
            },
            Self::Pln(arg_count) => {
                let mut values = Vec::new();
                for _ in 0..*arg_count {
                    if let Some(var) = env.stack.pop() {
                        values.push(var);
                    } else {
                        return Err(Error::StackError);
                    }
                }
                let mut output = Vec::new();
                let mut seen_str = false;
                for var in values.into_iter().rev() {
                    if !seen_str {
                        if var.gen_type() == Type::Str { seen_str = true; }
                    }
                    let out = var.val.read().print(&graph);
                    output.push(out);
                }
                let mut sep = "";
                if !seen_str { sep = ", " }
                println!("{}", output.join(sep));
            },
            Self::Dbg(arg_count) => {
                let mut values = Vec::new();
                for _ in 0..*arg_count {
                    if let Some(var) = env.stack.pop() {
                        values.push(var);
                    } else {
                        return Err(Error::StackError);
                    }
                }
                let mut output = Vec::new();
                let mut seen_str = false;
                for var in values.into_iter().rev() {
                    if !seen_str {
                        if var.gen_type() == Type::Str { seen_str = true; }
                    }
                    let out = var.val.read().debug(&graph);
                    output.push(out);
                }
                let mut sep = "";
                if !seen_str { sep = ", " }
                println!("{}", output.join(sep));
            },
            Self::Err(arg_count) => {
                let mut values = Vec::new();
                for _ in 0..*arg_count {
                    if let Some(var) = env.stack.pop() {
                        values.push(var);
                    } else {
                        return Err(Error::StackError);
                    }
                }
                let mut output = Vec::new();
                let mut seen_str = false;
                for var in values.into_iter().rev() {
                    if !seen_str {
                        if var.gen_type() == Type::Str { seen_str = true; }
                    }
                    let out = var.val.read().print(&graph);
                    output.push(out);
                }
                let mut sep = "";
                if !seen_str { sep = ", " }
                eprintln!("{}", output.join(sep));
            }
            
            Self::Sleep => {
                let duration;
                if let Some(val) = env.stack.pop() {
                    if let Some(num) = val.val.write().try_num() {
                        duration = num.float(Some(Units::Milliseconds));
                    } else {
                        return Err(Error::StackError);
                    }
                } else {
                    return Err(Error::StackError);
                }

                let mut instructions = Instructions::default();
                instructions.push(Arc::new(Base::CtrlSleepFor(Duration::from_millis(duration.abs() as u64))));
                return Ok(Some(instructions));
            },

            Self::Exit(arg_count) => {
                let mut instructions = Instructions::default();

                if *arg_count < 1 {
                    instructions.push(EXIT.clone());
                } else {
                    let mut promises = Vec::new();
                    for _ in 0..*arg_count {
                        if let Some(var) = env.stack.pop() {
                            if var.try_promise().is_some() {
                                promises.push(var);
                            }
                        }
                    }
                    for promise in promises.into_iter().rev() {
                        instructions.push(Arc::new(Base::Variable(promise)));
                        instructions.push(EXIT.clone());
                    }
                }

                return Ok(Some(instructions));
            },

            Self::Throw => {
                if let Some(val) = env.stack.pop() {
                    return Err(Error::Thrown(val.get()));
                } else {
                    return Err(Error::Thrown(Val::Null));
                }
            },

            Self::Assert => {
                if let Some(val) = env.stack.pop() {
                    if !val.val.read().truthy() {
                        let message = format!("'{}' is not truthy", val.val.read().print(&graph));
                        return Err(Error::AssertFailed(message));
                    }
                }
            },
            Self::AssertNot => {
                if let Some(val) = env.stack.pop() {
                    if val.val.read().truthy() {
                        let message = format!("'{}' is truthy", val.val.read().print(&graph));
                        return Err(Error::AssertNotFailed(message));
                    }
                }
            },
            Self::AssertEq => {
                if let Some(val) = env.stack.pop() {
                    if let Some(other) = env.stack.pop() {
                        if let Ok(res) = val.equal(&other) {
                            if !res.val.read().truthy() {
                                let message = format!("'{}' does not equal '{}'", other.val.read().print(&graph), val.val.read().print(&graph));
                                return Err(Error::AssertEqFailed(message));
                            }
                        }
                    }
                }
            },
            Self::AssertNeq => {
                if let Some(val) = env.stack.pop() {
                    if let Some(other) = env.stack.pop() {
                        if let Ok(res) = val.equal(&other) {
                            if res.val.read().truthy() {
                                let message = format!("'{}' equals '{}'", other.val.read().print(&graph), val.val.read().print(&graph));
                                return Err(Error::AssertNotEqFailed(message));
                            }
                        }
                    }
                }
            },

            Self::List(arg_count) => {
                let mut instructions = Instructions::default();
                instructions.push(NEW_LIST.clone());

                let mut args = Vec::new();
                for _ in 0..*arg_count {
                    args.push(env.stack.pop().unwrap());
                }
                for arg in args.into_iter().rev() {
                    instructions.push(Arc::new(Base::Variable(arg)));
                    instructions.push(PUSH_LIST.clone());
                }

                return Ok(Some(instructions));
            },
            Self::Set(arg_count) => {
                let mut instructions = Instructions::default();
                instructions.push(NEW_SET.clone());

                let mut args = Vec::new();
                for _ in 0..*arg_count {
                    args.push(env.stack.pop().unwrap());
                }
                for arg in args.into_iter().rev() {
                    instructions.push(Arc::new(Base::Variable(arg)));
                    instructions.push(PUSH_SET.clone());
                }

                return Ok(Some(instructions));
            },
            Self::Map(arg_count) => {
                let mut instructions = Instructions::default();
                instructions.push(NEW_MAP.clone());

                let mut args = Vec::new();
                for _ in 0..*arg_count {
                    args.push(env.stack.pop().unwrap());
                }
                for arg in args.into_iter().rev() {
                    match arg.val.read().deref() {
                        Val::Tup(vals) => {
                            if vals.len() == 2 {
                                instructions.push(Arc::new(Base::Variable(Variable::refval(vals[0].duplicate(false)))));
                                instructions.push(Arc::new(Base::Variable(Variable::refval(vals[1].duplicate(false)))));
                                instructions.push(PUSH_MAP.clone());
                            } else {
                                return Err(Error::MapConstructor("map init must have a key-value pair in the form of a list or tuple".into()));
                            }
                        },
                        Val::List(vals) => {
                            if vals.len() == 2 {
                                instructions.push(Arc::new(Base::Variable(Variable::refval(vals[0].duplicate(false)))));
                                instructions.push(Arc::new(Base::Variable(Variable::refval(vals[1].duplicate(false)))));
                                instructions.push(PUSH_MAP.clone());
                            } else {
                                return Err(Error::MapConstructor("map init must have a key-value pair in the form of a list or tuple".into()));
                            }
                        },
                        _ => {
                            return Err(Error::MapConstructor("unrecognized map init value (has to be a tuple or list with a key and value)".into()));
                        }
                    }
                }

                return Ok(Some(instructions));
            },

            Self::Copy => {
                if let Some(var) = env.stack.pop() {
                    env.stack.push(var.deep_copy(graph, Some(env.self_ptr())));
                }
            },
            Self::Swap => {
                if let Some(first) = env.stack.pop() {
                    if let Some(second) = env.stack.pop() {
                        let mut first = first.val.write();
                        let mut second = second.val.write();

                        let first = first.deref_mut();
                        let second = second.deref_mut();
                        
                        swap(first, second);
                    }
                }
            },

            Self::Functions => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Void |
                        Val::Null => {
                            let functions = Func::all_functions(&graph, &None)
                                .into_iter()
                                .map(|dref| ValRef::new(Val::Fn(dref)))
                                .collect::<Vector<_>>();
                            env.stack.push(Variable::val(Val::List(functions)));
                            return Ok(None);
                        },
                        Val::Str(attr) => {
                            let mut attributes = FxHashSet::default();
                            attributes.insert(attr.to_string());
                            let functions = Func::all_functions(&graph, &Some(attributes))
                                .into_iter()
                                .map(|dref| ValRef::new(Val::Fn(dref)))
                                .collect::<Vector<_>>();
                            env.stack.push(Variable::val(Val::List(functions)));
                            return Ok(None);
                        },
                        Val::Tup(attrs) |
                        Val::List(attrs) => {
                            let mut attributes = FxHashSet::default();
                            for attr in attrs {
                                match attr.read().deref() {
                                    Val::Str(attr) => { attributes.insert(attr.to_string()); },
                                    _ => {}
                                }
                            }
                            let functions = Func::all_functions(&graph, &Some(attributes))
                                .into_iter()
                                .map(|dref| ValRef::new(Val::Fn(dref)))
                                .collect::<Vector<_>>();
                            env.stack.push(Variable::val(Val::List(functions)));
                            return Ok(None);
                        },
                        Val::Set(attrs) => {
                            let mut attributes = FxHashSet::default();
                            for attr in attrs {
                                match attr.read().deref() {
                                    Val::Str(attr) => { attributes.insert(attr.to_string()); },
                                    _ => {}
                                }
                            }
                            let functions = Func::all_functions(&graph, &Some(attributes))
                                .into_iter()
                                .map(|dref| ValRef::new(Val::Fn(dref)))
                                .collect::<Vector<_>>();
                            env.stack.push(Variable::val(Val::List(functions)));
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                return Err(Error::StdFunctions);
            },
            Self::ObjDropped(arg_count) => {
                let mut vars = Vec::new();
                for _ in 0..*arg_count {
                    vars.push(env.stack.pop().unwrap());
                }

                // If the variable is an object, call all #[dropped] on that object
                let mut instructions = Instructions::default();
                for var in vars.iter() {
                    if let Some(obj) = var.try_obj() {
                        let mut objects = vec![obj.clone()];
                        objects.append(&mut Prototype::prototype_nodes(&graph, &obj, true));
                        
                        let mut attrs = FxHashSet::default();
                        attrs.insert("dropped".to_string());
                        let attrs = Some(attrs);

                        let mut names = FxHashSet::default();
                        for obj in objects {
                            let funcs = Func::functions(&graph, &obj, &attrs, false);
                            for func in funcs {
                                names.insert(func.data_name(graph).unwrap());
                            }
                        }
                        for name in names {
                            instructions.push(DUPLICATE.clone());
                            instructions.push(Arc::new(FuncCall {
                                func: None,
                                search: Some(name.as_ref().into()),
                                stack: true,
                                as_ref: false,
                                args: vector![], // no args for a constructor
                            }));
                        }
                    }
                }

                for var in vars.into_iter().rev() {
                    env.stack.push(var);
                }
                return Ok(Some(instructions));
            },
            Self::Drop(arg_count) => {
                let mut vars = Vec::new();
                for _ in 0..*arg_count {
                    vars.push(env.stack.pop().unwrap());
                }
                let mut results = Vector::default();
                for var in vars.into_iter().rev() {
                    match var.val.read().deref() {
                        Val::Str(path) => {
                            let mut context = None;
                            if path.starts_with(SELF_STR_KEYWORD.as_str()) || path.starts_with(SUPER_STR_KEYWORD.as_str()) {
                                context = Some(env.self_ptr());
                            }
                            let mut dropped = false;
                            if let Some(field_ref) = Field::field_from_path(graph, path.as_str(), context.clone()) {
                                let mut val = None;
                                if let Some(field) = graph.get_stof_data::<Field>(&field_ref) {
                                    val = Some(field.value.val.clone());
                                }
                                if let Some(val) = val {
                                    val.read().drop_data(graph);
                                }
                                dropped = graph.remove_data(&field_ref, None);
                            } else if let Some(node) = SPath::node(&graph, path.as_str(), context.clone()) {
                                dropped = graph.remove_node(&node, true);
                            } else if let Some(func_ref) = Func::func_from_path(graph, path.as_str(), context) {
                                dropped = graph.remove_data(&func_ref, None);
                            }
                            results.push_back(ValRef::new(Val::Bool(dropped)));
                        },
                        Val::Obj(nref) => {
                            // cleans up fields that reference this obj as well
                            results.push_back(ValRef::new(Val::Bool(graph.remove_node(nref, true))));
                        },
                        Val::Fn(dref) => {
                            results.push_back(ValRef::new(Val::Bool(graph.remove_data(dref, None))));
                        },
                        Val::Data(dref) => {
                            results.push_back(ValRef::new(Val::Bool(graph.remove_data(dref, None))));
                        },
                        _ => {}
                    }
                }
                if results.len() == 1 {
                    env.stack.push(Variable::refval(results.pop_front().unwrap()));
                } else if results.len() > 1 {
                    env.stack.push(Variable::val(Val::List(results)));
                }
            },
            Self::ShallowDrop(arg_count) => {
                let mut vars = Vec::new();
                for _ in 0..*arg_count {
                    vars.push(env.stack.pop().unwrap());
                }
                let mut results = Vector::default();
                for var in vars.into_iter().rev() {
                    match var.val.read().deref() {
                        Val::Str(path) => {
                            let mut context = None;
                            if path.starts_with(SELF_STR_KEYWORD.as_str()) || path.starts_with(SUPER_STR_KEYWORD.as_str()) {
                                context = Some(env.self_ptr());
                            }
                            let mut dropped = false;
                            if let Some(field_ref) = Field::field_from_path(graph, path.as_str(), context.clone()) {
                                dropped = graph.remove_data(&field_ref, None);
                            } else if let Some(node) = SPath::node(&graph, path.as_str(), context.clone()) {
                                dropped = graph.remove_node(&node, true);
                            } else if let Some(func_ref) = Func::func_from_path(graph, path.as_str(), context) {
                                dropped = graph.remove_data(&func_ref, None);
                            }
                            results.push_back(ValRef::new(Val::Bool(dropped)));
                        },
                        Val::Obj(nref) => {
                            // cleans up fields that reference this obj as well
                            results.push_back(ValRef::new(Val::Bool(graph.remove_node(nref, true))));
                        },
                        Val::Fn(dref) => {
                            results.push_back(ValRef::new(Val::Bool(graph.remove_data(dref, None))));
                        },
                        Val::Data(dref) => {
                            results.push_back(ValRef::new(Val::Bool(graph.remove_data(dref, None))));
                        },
                        _ => {}
                    }
                }
                if results.len() == 1 {
                    env.stack.push(Variable::refval(results.pop_front().unwrap()));
                } else if results.len() > 1 {
                    env.stack.push(Variable::val(Val::List(results)));
                }
            },

            Self::Parse => {
                // Std.parse("source", context = self, format = 'stof') -> bool
                if let Some(format_var) = env.stack.pop() {
                    if let Some(context_var) = env.stack.pop() {
                        if let Some(source_var) = env.stack.pop() {
                            let mut context = env.self_ptr();
                            match context_var.val.read().deref() {
                                Val::Str(path) => {
                                    let mut ctx = None;
                                    if path.starts_with(SELF_STR_KEYWORD.as_str()) || path.starts_with(SUPER_STR_KEYWORD.as_str()) {
                                        ctx = Some(env.self_ptr());
                                    }
                                    if let Some(field_ref) = Field::field_from_path(graph, path.as_str(), ctx.clone()) {
                                        if let Some(field) = graph.get_stof_data::<Field>(&field_ref) {
                                            if let Some(obj) = field.value.try_obj() {
                                                context = obj;
                                            } else {
                                                // Context was not an object
                                                env.stack.push(Variable::val(Val::Bool(false)));
                                                return Ok(None);
                                            }
                                        }
                                    } else if let Some(node) = SPath::node(&graph, path.as_str(), ctx) {
                                        context = node;
                                    } else {
                                        // context given, but not found (return false)
                                        env.stack.push(Variable::val(Val::Bool(false)));
                                        return Ok(None);
                                    }
                                },
                                Val::Obj(nref) => {
                                    context = nref.clone();
                                },
                                _ => {}
                            }
                            
                            let mut format = "stof".to_string();
                            match format_var.val.read().deref() {
                                Val::Str(fmt) => {
                                    format = fmt.to_string();
                                },
                                Val::Void |
                                Val::Null => {}, // keep as stof
                                _ => {
                                    return Err(Error::StdParse("format must be a string content type or stof format identifier".to_string()));
                                }
                            }

                            match source_var.val.read().deref() {
                                Val::Str(src) => {
                                    graph.string_import(&format, src.as_str(), Some(context))?;
                                    env.stack.push(Variable::val(Val::Bool(true)));
                                    return Ok(None);
                                },
                                Val::Blob(bytes) => {
                                    graph.binary_import(&format, Bytes::from(bytes.clone()), Some(context))?;
                                    env.stack.push(Variable::val(Val::Bool(true)));
                                    return Ok(None);
                                },
                                _ => {
                                    return Err(Error::StdParse("parse source data must be a string or blob".to_string()));
                                }
                            }
                        }
                    }
                }
                return Err(Error::StdParse("stack variables not found".to_string()));
            },
            Self::Blobify => {
                if let Some(context_var) = env.stack.pop() {
                    if let Some(format_var) = env.stack.pop() {
                        let mut format = "json".to_string();
                        match format_var.val.read().deref() {
                            Val::Str(fmt) => {
                                format = fmt.to_string();
                            },
                            Val::Void |
                            Val::Null => {},
                            _ => {
                                return Err(Error::StdBlobify("format must be a string content type or format identifier and must be made available to the graph explicitely by each runtime".to_string()))
                            }
                        }

                        let mut ctx = None;
                        match context_var.val.read().deref() {
                            Val::Obj(nref) => {
                                ctx = Some(nref.clone());
                            },
                            Val::Void |
                            Val::Null => {},
                            _ => {
                                return Err(Error::StdBlobify("context must be an object".to_string()));
                            },
                        }

                        let bytes = graph.binary_export(&format, ctx)?;
                        env.stack.push(Variable::val(Val::Blob(bytes.to_vec())));
                        return Ok(None);
                    }
                }
                return Err(Error::StdBlobify("blobify stack variables do not exist".to_string()));
            },
            Self::Stringify => {
                if let Some(context_var) = env.stack.pop() {
                    if let Some(format_var) = env.stack.pop() {
                        let mut format = "json".to_string();
                        match format_var.val.read().deref() {
                            Val::Str(fmt) => {
                                format = fmt.to_string();
                            },
                            Val::Void |
                            Val::Null => {},
                            _ => {
                                return Err(Error::StdStringify("format must be a string content type or format identifier and must be made available to the graph explicitely by each runtime".to_string()))
                            }
                        }

                        let mut ctx = None;
                        match context_var.val.read().deref() {
                            Val::Obj(nref) => {
                                ctx = Some(nref.clone());
                            },
                            Val::Void |
                            Val::Null => {},
                            _ => {
                                return Err(Error::StdStringify("context must be an object".to_string()));
                            },
                        }

                        let src = graph.string_export(&format, ctx)?;
                        env.stack.push(Variable::val(Val::Str(src.into())));
                        return Ok(None);
                    }
                }
                return Err(Error::StdStringify("stringify stack variables do not exist".to_string()));
            },

            Self::HasFormat => {
                if let Some(format_var) = env.stack.pop() {
                    match format_var.val.read().deref() {
                        Val::Str(format) => {
                            let mut has = graph.get_format(format.as_str()).is_some();
                            if !has {
                                has = graph.get_format_by_content_type(format.as_str()).is_some();
                            }
                            env.stack.push(Variable::val(Val::Bool(has)));
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                return Err(Error::StdHasFormat("format must be a string".into()));
            },
            Self::Formats => {
                let formats = graph.available_formats()
                    .into_iter()
                    .map(|fmt| ValRef::new(Val::Str(fmt.into())))
                    .collect::<OrdSet<ValRef<Val>>>();
                env.stack.push(Variable::val(Val::Set(formats)));
            },
            Self::FormatContentType => {
                if let Some(format_var) = env.stack.pop() {
                    match format_var.val.read().deref() {
                        Val::Str(format) => {
                            let mut has = graph.get_format(format.as_str());
                            if has.is_none() {
                                has = graph.get_format_by_content_type(format.as_str());
                            }
                            if let Some(fmt) = has {
                                env.stack.push(Variable::val(Val::Str(fmt.content_type().into())));
                            } else {
                                env.stack.push(Variable::val(Val::Null));
                            }
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                return Err(Error::StdFormatContentType("format must be a string".into()));
            },

            Self::HasLib => {
                if let Some(lib_var) = env.stack.pop() {
                    match lib_var.val.read().deref() {
                        Val::Str(lib) => {
                            let has = graph.libfuncs.contains_key(lib.as_str());
                            env.stack.push(Variable::val(Val::Bool(has)));
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                return Err(Error::StdHasLib("lib must be a string".into()));
            },
            Self::Libs => {
                let libs = graph.libfuncs.keys()
                    .into_iter()
                    .map(|fmt| ValRef::new(Val::Str(fmt.into())))
                    .collect::<OrdSet<ValRef<Val>>>();
                env.stack.push(Variable::val(Val::Set(libs)));
            },

            Self::NanoId => {
                if let Some(length_var) = env.stack.pop() {
                    match length_var.val.read().deref() {
                        Val::Num(num) => {
                            let size = num.int() as usize;
                            env.stack.push(Variable::val(Val::Str(nanoid!(size).into())));
                        },
                        _ => {}
                    }
                }
            },
            Self::GraphId => {
                env.stack.push(Variable::val(Val::Str(graph.id.to_string().into())));
            },

            Self::Min(arg_count) => {
                let mut res = None;
                for _ in 0..*arg_count {
                    if let Some(var) = env.stack.pop() {
                        let min_var = var.val.read().minimum(graph)?;
                        if let Some(current) = res {
                            let lt = min_var.lt(&current, &graph)?;
                            if lt.truthy() {
                                res = Some(min_var);
                            } else {
                                res = Some(current);
                            }
                        } else {
                            res = Some(min_var);
                        }
                    }
                }
                if let Some(res) = res {
                    env.stack.push(Variable::val(res));
                } else {
                    env.stack.push(Variable::val(Val::Null));
                }
            },
            Self::Max(arg_count) => {
                let mut res = None;
                for _ in 0..*arg_count {
                    if let Some(var) = env.stack.pop() {
                        let max_var = var.val.read().maximum(graph)?;
                        if let Some(current) = res {
                            let gt = max_var.gt(&current, &graph)?;
                            if gt.truthy() {
                                res = Some(max_var);
                            } else {
                                res = Some(current);
                            }
                        } else {
                            res = Some(max_var);
                        }
                    }
                }
                if let Some(res) = res {
                    env.stack.push(Variable::val(res));
                } else {
                    env.stack.push(Variable::val(Val::Null));
                }
            },

            Self::Callstack => {
                let callstack = env.call_stack.iter()
                    .cloned()
                    .map(|id| ValRef::new(Val::Fn(id)))
                    .collect::<Vector<_>>();
                env.stack.push(Variable::val(Val::List(callstack)));
            },
            Self::Trace(arg_count) => {
                let mut n = 10;
                let mut arg_count = *arg_count;
                if arg_count > 0 {
                    let last = env.stack.pop().unwrap();
                    let mut found = false;
                    {
                        let val = last.val.read();
                        match val.deref() {
                            Val::Num(num) => {
                                n = num.int() as usize;
                                found = true;
                                arg_count -= 1;
                            },
                            _ => {}
                        }
                    }
                    if !found {
                        env.stack.push(last);
                    }
                }

                let mut instructions = Instructions::default();
                instructions.push(Arc::new(StdIns::Pln(arg_count)));
                instructions.push(Arc::new(Base::CtrlTrace(n)));
                return Ok(Some(instructions));
            },
            Self::TraceStack => {
                println!("{:?}", &env.stack);
            },
        }
        Ok(None)
    }
}
