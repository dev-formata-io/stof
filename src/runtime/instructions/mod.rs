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

use std::{sync::Arc, time::Duration};
use arcstr::ArcStr;
use lazy_static::lazy_static;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use crate::{model::{Field, Func, Graph, SPath, SELF_STR_KEYWORD, SUPER_STR_KEYWORD}, runtime::{instruction::{Instruction, Instructions}, proc::{ProcEnv, Process}, Error, Type, Val, Variable, WakeRef}};

pub mod call;
pub mod block;
pub mod ops;
pub mod ifs;
pub mod switch;
pub mod whiles;
pub mod new_obj;
pub mod empty;
pub mod trycatch;
pub mod list;
pub mod tup;
pub mod set;
pub mod map;
pub mod ret;
pub mod func;
pub mod nullcheck;


// static instructions for efficiency
lazy_static! {
    pub static ref SUSPEND: Arc<dyn Instruction> = Arc::new(Base::CtrlSuspend);
    pub static ref AWAIT: Arc<dyn Instruction> = Arc::new(Base::CtrlAwait);
    pub static ref NOOP: Arc<dyn Instruction> = Arc::new(Base::CtrlNoOp);
    pub static ref END_TRY: Arc<dyn Instruction> = Arc::new(Base::CtrlTryEnd);
    pub static ref EXIT: Arc<dyn Instruction> = Arc::new(Base::CtrlExit);

    pub static ref PUSH_SELF: Arc<dyn Instruction> = Arc::new(Base::PushSelf);
    pub static ref POP_SELF: Arc<dyn Instruction> = Arc::new(Base::PopSelf);

    pub static ref PUSH_CALL: Arc<dyn Instruction> = Arc::new(Base::PushCall);
    pub static ref POP_CALL: Arc<dyn Instruction> = Arc::new(Base::PopCall);

    pub static ref PUSH_NEW: Arc<dyn Instruction> = Arc::new(Base::PushNew);
    pub static ref POP_NEW: Arc<dyn Instruction> = Arc::new(Base::PopNew);

    pub static ref POP_STACK: Arc<dyn Instruction> = Arc::new(Base::PopStack);

    pub static ref FN_RETURN: Arc<dyn Instruction> = Arc::new(Base::CtrlFnReturn);
    pub static ref POP_RETURN: Arc<dyn Instruction> = Arc::new(Base::PopReturn);
    pub static ref PUSH_RETURN: Arc<dyn Instruction> = Arc::new(Base::PushReturn);

    pub static ref PUSH_SYMBOL_SCOPE: Arc<dyn Instruction> = Arc::new(Base::PushSymbolScope);
    pub static ref POP_SYMBOL_SCOPE: Arc<dyn Instruction> = Arc::new(Base::PopSymbolScope);
    pub static ref POP_LOOP: Arc<dyn Instruction> = Arc::new(Base::PopLoop);
    pub static ref BREAK_LOOP: Arc<dyn Instruction> = Arc::new(Base::CtrlBreak);
    pub static ref CONTINUE_LOOP: Arc<dyn Instruction> = Arc::new(Base::CtrlContinue);

    pub static ref DUPLICATE: Arc<dyn Instruction> = Arc::new(Base::Dup);
    pub static ref TRUTHY: Arc<dyn Instruction> = Arc::new(Base::Truthy);
    pub static ref IS_NULL: Arc<dyn Instruction> = Arc::new(Base::IsNull);
    pub static ref NOT_TRUTHY: Arc<dyn Instruction> = Arc::new(Base::NotTruthy);
    pub static ref TYPE_OF: Arc<dyn Instruction> = Arc::new(Base::TypeOf);
    pub static ref TYPE_NAME: Arc<dyn Instruction> = Arc::new(Base::TypeName);
    pub static ref INSTANCE_OF: Arc<dyn Instruction> = Arc::new(Base::InstanceOf);

    pub static ref ADD: Arc<dyn Instruction> = Arc::new(Base::Add);
    pub static ref SUBTRACT: Arc<dyn Instruction> = Arc::new(Base::Sub);
    pub static ref MULTIPLY: Arc<dyn Instruction> = Arc::new(Base::Mul);
    pub static ref DIVIDE: Arc<dyn Instruction> = Arc::new(Base::Div);
    pub static ref MODULUS: Arc<dyn Instruction> = Arc::new(Base::Mod);
    pub static ref BIT_AND: Arc<dyn Instruction> = Arc::new(Base::AND);
    pub static ref BIT_OR: Arc<dyn Instruction> = Arc::new(Base::OR);
    pub static ref BIT_XOR: Arc<dyn Instruction> = Arc::new(Base::XOR);
    pub static ref BIT_SHIFT_LEFT: Arc<dyn Instruction> = Arc::new(Base::SHL);
    pub static ref BIT_SHIFT_RIGHT: Arc<dyn Instruction> = Arc::new(Base::SHR);

    pub static ref GREATER_THAN: Arc<dyn Instruction> = Arc::new(Base::GreaterThan);
    pub static ref GREATER_THAN_OR_EQ: Arc<dyn Instruction> = Arc::new(Base::GreaterOrEq);
    pub static ref LESS_THAN: Arc<dyn Instruction> = Arc::new(Base::LessThan);
    pub static ref LESS_THAN_OR_EQ: Arc<dyn Instruction> = Arc::new(Base::LessOrEq);
    pub static ref EQUAL: Arc<dyn Instruction> = Arc::new(Base::Eq);
    pub static ref NOT_EQUAL: Arc<dyn Instruction> = Arc::new(Base::Neq);
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsumeStack {
    Dont,
    Consume,
    IfTrue,
    IfFalse,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Foundational instructions.
/// Higher order instructions JIT down into a subset of these as they execute.
pub enum Base {
    // Suspend instruction.
    // Used to denote going to another process now.
    // Place these after runs of instructions to make sure we keep making progress on other processes too.
    CtrlSuspend,
    // Instruct the system to wait for this process before continuing. Looks for a Promise on the stack.
    // Load a promise onto the stack, then insert this instruction to wait for the process to complete.
    CtrlAwait,
    CtrlAwaitCast(Type), // Special cast instruction for awaits with custom rules
    CtrlAwaitError(Error), // Special error propagation for other processes

    // Does nothing...
    CtrlNoOp,

    // Tag a place in the instructions.
    // This is a form of GOTO, used for looping & control flow
    Tag(ArcStr),
    CtrlBackTo(ArcStr), // start next on instruction right after tag
    CtrlForwardTo(ArcStr), // start next on instruction right after tag
    CtrlForwardToIfTruthy(ArcStr, ConsumeStack), // forward to if a truthy value is on the stack
    CtrlForwardToIfNotTruthy(ArcStr, ConsumeStack), // forward to if a non-truthy value is on the stack
    CtrlJumpTable(FxHashMap<Val, ArcStr>, Option<ArcStr>), // values to jump tags (switch)

    // Return jump
    CtrlFnReturn,
    PushReturn,
    PopReturn,

    // Try catch control instructions.
    // Go forward to this tag if an error occurrs.
    CtrlTry(ArcStr),
    CtrlTryEnd,

    // Sleep instructions.
    CtrlSleepFor(Duration),
    CtrlSleepRef(WakeRef),
    
    // Exit a process. Looks for a promise on the stack - if not found, terminates the current process.
    CtrlExit,

    // Self stack.
    PushSelf,
    PopSelf,

    // Call stack.
    PushCall,
    PopCall,

    // New obj stack.
    PushNew,
    PopNew,

    // Loop stack.
    PushLoop(ArcStr),
    PopLoop,
    CtrlBreak,
    CtrlContinue,

    // Pop a variable from the stack. (drop val)
    PopStack,
    PopUntilAndIncluding(Val),
    FuncVoidRet(Val),

    // Spawn a new process.
    Spawn((Instructions, Type)),

    // Symbol table / Graph.
    PushSymbolScope,
    PopSymbolScope,
    PopSymbolScopeUntilDepth(usize),

    DeclareVar(ArcStr, Type), // requires val on stack (optionally typed)
    DeclareConstVar(ArcStr, Type), // requires val on stack (optionally typed)
    
    DropVariable(ArcStr), // removes from the st/graph
    LoadVariable(ArcStr, bool, bool), // loads st/graph to stack
    SetVariable(ArcStr), // requires val on stack

    // Values.
    Dup,
    Literal(Val), // load a literal onto the stack
    Variable(Variable), // load a variable onto the stack
    Cast(Type), // Cast value on the back of the stack to a specific type
    TypeOf,
    TypeName,
    InstanceOf,
    
    Truthy,
    NotTruthy,
    IsNull,
    
    LessThan,
    GreaterThan,
    LessOrEq,
    GreaterOrEq,
    Eq,
    Neq,

    Add,
    Sub,
    Mul,
    Div,
    Mod,

    AND, // bit
    OR,
    XOR,
    SHL,
    SHR,
}
#[typetag::serde(name = "Base")]
impl Instruction for Base {
    /// Base instructions do not replace themselves and are used by other higher-order instructions.
    /// Know what you are doing if using these.
    fn exec(&self, env: &mut ProcEnv, graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        match self {
            /*****************************************************************************
             * Suspend.
             *****************************************************************************/
            Self::CtrlSuspend => {}, // Nothing here...
            Self::CtrlAwait => {}, // Nothing here...
            Self::CtrlExit => {}, // Nothing here...
            Self::CtrlAwaitCast(_) => {}, // Nothing here...
            Self::CtrlAwaitError(err) => { return Err(err.clone()); },
            Self::CtrlNoOp => {}, // Does nothing

            /*****************************************************************************
             * Sleep.
             *****************************************************************************/
            Self::CtrlSleepFor(_) => {}, // Nothing here... used by instructions...
            Self::CtrlSleepRef(_) => {}, // Nothing here... used by instructions...
            
            /*****************************************************************************
             * Tags.
             *****************************************************************************/
            Self::Tag(_id) => {}, // Nothing here... just goes on through to mark a place
            Self::CtrlBackTo(_id) => {}, // Nothing here... used by instructions...
            Self::CtrlForwardTo(_id) => {}, // Nothing here... used by instructions...
            Self::CtrlForwardToIfTruthy(_id, _) => {}, // Nothing here... used by instructions...
            Self::CtrlForwardToIfNotTruthy(_id, _) => {}, // Nothing here... used by instructions...

            Self::CtrlJumpTable(..) => {}, // Nothing here... used by instructions...

            Self::CtrlTry(_) => {}, // Nothing here... used by instructions...
            Self::CtrlTryEnd => {}, // Nothing here... used by instructions...

            Self::CtrlFnReturn => {}, // Nothing here...
            Self::PushReturn => {
                if let Some(var) = env.stack.pop() {
                    if let Some(func) = var.try_func() {
                        env.return_stack.push(func.as_ref().into());
                        return Ok(None);
                    }
                }
                return Err(Error::CallStackError);
            },
            Self::PopReturn => { env.return_stack.pop(); },

            /*****************************************************************************
             * Special stacks.
             *****************************************************************************/
            Self::PushSelf => {
                if let Some(var) = env.stack.pop() {
                    if let Some(obj) = var.try_obj() {
                        env.self_stack.push(obj);
                        return Ok(None);
                    }
                }
                return Err(Error::SelfStackError);
            },
            Self::PopSelf => { env.self_stack.pop(); },

            Self::PushCall => {
                if let Some(var) = env.stack.pop() {
                    if let Some(func) = var.try_func() {
                        env.call_stack.push(func);
                        return Ok(None);
                    }
                }
                return Err(Error::CallStackError);
            },
            Self::PopCall => { env.call_stack.pop(); },
            
            Self::PushNew => {
                if let Some(var) = env.stack.pop() {
                    if let Some(obj) = var.try_obj() {
                        env.new_stack.push(obj);
                        return Ok(None);
                    }
                }
                return Err(Error::NewStackError);
            },
            Self::PopNew => { env.new_stack.pop(); },

            Self::PushLoop(tag) => {
                env.loop_stack.push(tag.clone());
                env.table.push();
            },
            Self::PopLoop => {
                env.loop_stack.pop();
                env.table.pop();
            },
            Self::CtrlBreak => {}, // Nothing here...
            Self::CtrlContinue => {}, // Nothing here...

            
            /*****************************************************************************
             * Spawn a new process.
             *****************************************************************************/
            
            Self::Spawn((async_ins, ty)) => {
                // Creates a new PID every time here, avoiding a lot of issues...
                let mut proc = Process::from(async_ins.clone());
                let pid = proc.env.pid.clone();
                proc.env = env.clone(); // clone this environment
                proc.env.stack.clear(); // new stack for this new proc
                proc.env.spawn = None;
                proc.env.pid = pid.clone();

                env.spawn = Some(Box::new(proc));
                env.stack.push(Variable::val(Val::Promise(pid, ty.clone())));
                // up to the caller to add the suspend to actually spawn (don't want this ins replaced)
            },
            
            /*****************************************************************************
             * Variables.
             *****************************************************************************/
            
            Self::PushSymbolScope => env.table.push(),
            Self::PopSymbolScope => { env.table.pop(); },
            Self::PopSymbolScopeUntilDepth(depth) => {
                while env.table.scopes.len() > *depth {
                    env.table.pop();
                }
            },
            Self::DeclareVar(name, vtype) => {
                if !env.table.can_declare(name) { return Err(Error::DeclareExisting); }
                if name.contains('.') || name == &SELF_STR_KEYWORD || name == &SUPER_STR_KEYWORD { return Err(Error::DeclareInvalidName); }
                if let Some(mut var) = env.stack.pop() {
                    var.mutable = true;
                    if !vtype.empty() {
                        var.vtype = Some(vtype.clone());
                    }
                    env.table.insert(name, var);
                } else {
                    return Err(Error::StackError);
                }
            },
            Self::DeclareConstVar(name, vtype) => {
                if !env.table.can_declare(name) { return Err(Error::DeclareExisting); }
                if name.contains('.') || name == &SELF_STR_KEYWORD || name == &SUPER_STR_KEYWORD { return Err(Error::DeclareInvalidName); }
                if let Some(mut var) = env.stack.pop() {
                    var.mutable = false;
                    if !vtype.empty() {
                        var.vtype = Some(vtype.clone());
                    }
                    env.table.insert(name, var);
                } else {
                    return Err(Error::StackError);
                }
            },
            Self::DropVariable(name) => {
                if !name.contains('.') {
                    if let Some(var) = env.table.drop_var(name) {
                        var.drop_data(graph);
                        return Ok(None);
                    }
                }
                
                if name.starts_with(SELF_STR_KEYWORD.as_str()) || name.starts_with(SUPER_STR_KEYWORD.as_str()) {
                    let self_ptr = env.self_ptr();
                    if let Some(field) = Field::field_from_path(graph, &name, Some(self_ptr.clone())) {
                        // Special case for this instruction - we drop the object/data behind the field
                        let mut to_remove = None;
                        if let Some(field) = graph.get_mut_stof_data::<Field>(&field) {
                            to_remove = Some(field.value.val.clone());
                        }
                        if let Some(val) = to_remove {
                            val.read().drop_data(graph);
                        }
                        graph.remove_data(&field, None);
                    } else if let Some(node) = SPath::node(&graph, &name, Some(self_ptr.clone())) {
                        graph.remove_node(&node, false);
                    } else if let Some(func) = Func::func_from_path(graph, &name, Some(self_ptr.clone())) {
                        graph.remove_data(&func, None);
                    }
                } else if let Some(field) = Field::field_from_path(graph, &name, None) {
                    // Special case for this instruction - we drop the object/data behind the field
                    let mut to_remove = None;
                    if let Some(field) = graph.get_mut_stof_data::<Field>(&field) {
                        to_remove = Some(field.value.val.clone());
                    }
                    if let Some(val) = to_remove {
                        val.read().drop_data(graph);
                    }
                    graph.remove_data(&field, None);
                } else if let Some(node) = SPath::node(&graph, &name, None) {
                        graph.remove_node(&node, false);
                } else if let Some(func) = Func::func_from_path(graph, &name, None) {
                    graph.remove_data(&func, None);
                }
            },
            Self::LoadVariable(name, stack, by_ref) => {
                if *stack {
                    if let Some(var) = env.stack.pop() {
                        if let Some(obj) = var.try_obj() {
                            if let Some(field_ref) = Field::field_from_path(graph, &name, Some(obj.clone())) {
                                if let Some(field) = graph.get_stof_data::<Field>(&field_ref) {
                                    if field.is_private() {
                                        let self_ptr = env.self_ptr();
                                        let field_nodes = field_ref.data_nodes(&graph);
                                        if !field_nodes.contains(&self_ptr) {
                                            return Err(Error::FieldPrivate);
                                        }
                                    }
                                    env.stack.push(field.value.stack_var(*by_ref));
                                    return Ok(None);
                                }
                            } else if let Some(node) = SPath::node(&graph, &name, Some(obj.clone())) {
                                env.stack.push(Variable::val(Val::Obj(node)));
                                return Ok(None);
                            } else if let Some(func) = Func::func_from_path(graph, &name, Some(obj)) {
                                env.stack.push(Variable::val(Val::Fn(func)));
                                return Ok(None);
                            }
                        }
                    }
                    env.stack.push(Variable::val(Val::Null));
                    return Ok(None);
                }

                let mut split_path = name.split('.').collect::<Vec<_>>();
                let context;
                if split_path[0] == SELF_STR_KEYWORD.as_str() {
                    // Self case
                    context = Variable::val(Val::Obj(env.self_ptr()));
                    split_path.remove(0);
                } else if split_path[0] == SUPER_STR_KEYWORD.as_str() {
                    // Super case
                    context = Variable::val(Val::Obj(env.self_ptr()));
                } else if let Some(var) = env.table.get(split_path[0]) {
                    // Variable case
                    context = var.stack_var(*by_ref);
                    split_path.remove(0);
                } else if split_path[0] == "this" && env.call_stack.len() > 0 {
                    // Shortcut for referencing the current function as a context
                    context = Variable::val(Val::Fn(env.call_stack.last().unwrap().clone()));
                    split_path.remove(0);
                } else {
                    // Global case
                    if let Some(field_ref) = Field::field_from_path(graph, &name, None) {
                        if let Some(field) = graph.get_stof_data::<Field>(&field_ref) {
                            if field.is_private() {
                                let self_ptr = env.self_ptr();
                                let field_nodes = field_ref.data_nodes(&graph);
                                if !field_nodes.contains(&self_ptr) {
                                    return Err(Error::FieldPrivate);
                                }
                            }
                            env.stack.push(field.value.stack_var(*by_ref));
                            return Ok(None);
                        }
                    } else if let Some(node) = SPath::node(&graph, &name, None) {
                        env.stack.push(Variable::val(Val::Obj(node)));
                        return Ok(None);
                    } else if let Some(func) = Func::func_from_path(graph, &name, None) {
                        env.stack.push(Variable::val(Val::Fn(func)));
                        return Ok(None);
                    }
                    env.stack.push(Variable::val(Val::Null));
                    return Ok(None);
                }

                // If the split path is empty, add the context and return now
                if split_path.is_empty() {
                    env.stack.push(context);
                    return Ok(None);
                }

                // Else, the context needs to be an object to continue the lookup!
                let name = split_path.join(".");
                if let Some(obj) = context.try_obj() {
                    if let Some(field_ref) = Field::field_from_path(graph, &name, Some(obj.clone())) {
                        if let Some(field) = graph.get_stof_data::<Field>(&field_ref) {
                            if field.is_private() {
                                let self_ptr = env.self_ptr();
                                let field_nodes = field_ref.data_nodes(&graph);
                                if !field_nodes.contains(&self_ptr) {
                                    return Err(Error::FieldPrivate);
                                }
                            }
                            env.stack.push(field.value.stack_var(*by_ref));
                            return Ok(None);
                        }
                    } else if let Some(node) = SPath::node(&graph, &name, Some(obj.clone())) {
                        env.stack.push(Variable::val(Val::Obj(node)));
                        return Ok(None);
                    } else if let Some(func) = Func::func_from_path(graph, &name, Some(obj)) {
                        env.stack.push(Variable::val(Val::Fn(func)));
                        return Ok(None);
                    }
                }
                env.stack.push(Variable::val(Val::Null));
                return Ok(None);
            },
            Self::SetVariable(name) => {
                if let Some(var) = env.stack.pop() {
                    if !name.contains('.') && env.table.set(name, &var, graph, Some(env.self_ptr()))? {
                        return Ok(None);
                    }

                    if name == &SELF_STR_KEYWORD {
                        return Err(Error::AssignSelf);
                    }
                    if name == &SUPER_STR_KEYWORD {
                        return Err(Error::AssignSuper);
                    }

                    if name.starts_with("self.") || name.starts_with("super.") {
                        let self_ptr = env.self_ptr();
                        if let Some(field_ref) = Field::field_from_path(graph, &name, Some(self_ptr.clone())) {
                            let mut fvar = None;
                            if let Some(field) = graph.get_stof_data::<Field>(&field_ref) {
                                if !field.can_set() { return Err(Error::FieldReadOnlySet); }
                                fvar = Some(field.value.clone());
                            }
                            if let Some(mut fvar) = fvar {
                                fvar.set(&var, graph, Some(env.self_ptr()))?;

                                if let Some(field) = graph.get_mut_stof_data::<Field>(&field_ref) {
                                    field.value = fvar;
                                }
                            }
                            if let Some(field) = field_ref.data_mut(graph) {
                                field.invalidate_value();
                            }
                            return Ok(None);
                        } else {
                            let mut path = SPath::from(name);
                            let field_name = path.path.pop().unwrap();
                            if path.path.len() < 1 { return Err(Error::AssignSelf); }
                            if let Some(node) = graph.ensure_named_nodes(path, Some(self_ptr.clone()), true, None) {
                                let field = Field::new(var, None);
                                graph.insert_stof_data(&node, field_name, Box::new(field), None);
                                return Ok(None);
                            } else {
                                return Err(Error::AssignSelf);
                            }
                        }
                    } else if name.contains('.') {
                        let mut path = SPath::from(name);
                        if path.path.len() < 2 { return Err(Error::AssignSelf); }

                        // Look for an object variable as context
                        let mut context = None;
                        if let Some(var) = env.table.get(path.path[0].as_ref()) {
                            if let Some(var_obj) = var.try_obj() {
                                context = Some(var_obj);
                                path.path.remove(0);
                            }
                        }

                        if let Some(field_ref) = Field::field_from_path(graph, &path.join("."), context.clone()) {
                            let mut fvar = None;
                            if let Some(field) = graph.get_stof_data::<Field>(&field_ref) {
                                if !field.can_set() { return Err(Error::FieldReadOnlySet); }
                                fvar = Some(field.value.clone());
                            }
                            if let Some(mut fvar) = fvar {
                                fvar.set(&var, graph, context.clone())?;
                                
                                if let Some(field) = graph.get_mut_stof_data::<Field>(&field_ref) {
                                    field.value = fvar;
                                }
                            }
                            if let Some(field) = field_ref.data_mut(graph) {
                                field.invalidate_value();
                            }
                            return Ok(None);
                        }

                        let field_name = path.path.pop().unwrap();
                        if path.path.len() > 0 {
                            if let Some(node) = graph.ensure_named_nodes(path, context, true, None) {
                                let field = Field::new(var, None);
                                graph.insert_stof_data(&node, field_name, Box::new(field), None);
                                return Ok(None);
                            } else {
                                return Err(Error::AssignSelf);
                            }
                        } else if let Some(node) = context {
                            let field = Field::new(var, None);
                            graph.insert_stof_data(&node, field_name, Box::new(field), None);
                            return Ok(None);
                        } else {
                            return Err(Error::AssignSelf);
                        }
                    } else {
                        if let Some(nref) = var.try_obj() {
                            // If a root with this name already exists, then error instead of drop or collide
                            // This is because it's not a desireable behavior to merge, collide, or drop large sections of data without explicitly saying so
                            if let Some(_) = graph.find_root_named(name) {
                                return Err(Error::AssignExistingRoot);
                            }

                            if let Some(node) = nref.node_mut(graph) {
                                node.name = name.into();
                            }
                            graph.roots.insert(nref);
                            return Ok(None);
                        }
                        return Err(Error::AssignRootNonObj);
                    }
                } else {
                    return Err(Error::StackError);
                }
            },

            /*****************************************************************************
             * Values.
             *****************************************************************************/
            Self::Dup => {
                if let Some(val) = env.stack.pop() {
                    env.stack.push(val.stack_var(false));
                    env.stack.push(val);
                } else {
                    return Err(Error::StackError);
                }
            },
            Self::Literal(val) => {
                env.stack.push(Variable::val(val.clone()));
            },
            Self::Variable(var) => {
                env.stack.push(var.clone());
            },
            Self::PopStack => { env.stack.pop(); },
            Self::PopUntilAndIncluding(val) => {
                while let Some(var) = env.stack.pop() {
                    let res = var.val.read().equal(val)?;
                    if res.truthy() { break; }
                }
            },
            Self::FuncVoidRet(val) => {
                while let Some(var) = env.stack.pop() {
                    let res = var.val.read().equal(val)?;
                    if !res.truthy() {
                        // The function ended and has added to the stack, so error
                        return Err(Error::FuncNotVoid);
                    } else {
                        break;
                    }
                }
            },
            Self::Cast(target) => {
                if let Some(var) = env.stack.pop() {
                    var.cast(target, graph, Some(env.self_ptr()))?;
                    env.stack.push(var);
                } else if target.empty() {
                    // nothing to do in this case
                } else {
                    return Err(Error::CastStackError);
                }
            },
            Self::TypeOf => {
                if let Some(var) = env.stack.pop() {
                    let vt = var.gen_type();
                    env.stack.push(Variable::val(Val::Str(vt.rt_type_of(&graph))));
                } else {
                    return Err(Error::StackError);
                }
            },
            Self::TypeName => {
                if let Some(var) = env.stack.pop() {
                    let vt = var.spec_type(&graph);
                    env.stack.push(Variable::val(Val::Str(vt.rt_type_of(&graph))));
                } else {
                    return Err(Error::StackError);
                }
            },
            Self::InstanceOf => {
                if let Some(lhs) = env.stack.pop() {
                    if let Some(rhs) = env.stack.pop() {
                        if let Ok(instanceof) = lhs.instance_of(&rhs, &graph) {
                            env.stack.push(Variable::val(instanceof.into()));
                        } else {
                            return Err(Error::StackError);
                        }
                    } else {
                        return Err(Error::StackError);
                    }
                } else {
                    return Err(Error::StackError);
                }
            },
            Self::Truthy => {
                if let Some(var) = env.stack.pop() {
                    env.stack.push(Variable::val(var.truthy().into()));
                } else {
                    return Err(Error::Truthy);
                }
            },
            Self::NotTruthy => {
                if let Some(var) = env.stack.pop() {
                    env.stack.push(Variable::val((!var.truthy()).into()));
                } else {
                    return Err(Error::Truthy);
                }
            },
            Self::IsNull => {
                if let Some(var) = env.stack.pop() {
                    env.stack.push(Variable::val(var.null().into()));
                } else {
                    return Err(Error::IsNull);
                }
            },
            Self::GreaterThan => {
                let lhs = env.stack.pop();
                let rhs = env.stack.pop();
                if let Some(lhs) = lhs {
                    if let Some(rhs) = rhs {
                        env.stack.push(lhs.gt(&rhs, graph)?);
                    } else {
                        return Err(Error::GreaterThan);
                    }
                } else {
                    return Err(Error::GreaterThan);
                }
            },
            Self::GreaterOrEq => {
                let lhs = env.stack.pop();
                let rhs = env.stack.pop();
                if let Some(lhs) = lhs {
                    if let Some(rhs) = rhs {
                        env.stack.push(lhs.gte(&rhs, graph)?);
                    } else {
                        return Err(Error::GreaterOrEq);
                    }
                } else {
                    return Err(Error::GreaterOrEq);
                }
            },
            Self::LessThan => {
                let lhs = env.stack.pop();
                let rhs = env.stack.pop();
                if let Some(lhs) = lhs {
                    if let Some(rhs) = rhs {
                        env.stack.push(lhs.lt(&rhs, graph)?);
                    } else {
                        return Err(Error::LessThan);
                    }
                } else {
                    return Err(Error::LessThan);
                }
            },
            Self::LessOrEq => {
                let lhs = env.stack.pop();
                let rhs = env.stack.pop();
                if let Some(lhs) = lhs {
                    if let Some(rhs) = rhs {
                        env.stack.push(lhs.lte(&rhs, graph)?);
                    } else {
                        return Err(Error::LessOrEq);
                    }
                } else {
                    return Err(Error::LessOrEq);
                }
            },
            Self::Eq => {
                let lhs = env.stack.pop();
                let rhs = env.stack.pop();
                if let Some(lhs) = lhs {
                    if let Some(rhs) = rhs {
                        env.stack.push(lhs.equal(&rhs)?);
                    } else {
                        return Err(Error::Eq);
                    }
                } else {
                    return Err(Error::Eq);
                }
            },
            Self::Neq => {
                let lhs = env.stack.pop();
                let rhs = env.stack.pop();
                if let Some(lhs) = lhs {
                    if let Some(rhs) = rhs {
                        env.stack.push(lhs.not_equal(&rhs)?);
                    } else {
                        return Err(Error::Eq);
                    }
                } else {
                    return Err(Error::Eq);
                }
            },
            Self::Add => {
                let lhs = env.stack.pop();
                let rhs = env.stack.pop();
                if let Some(lhs) = lhs {
                    if let Some(rhs) = rhs {
                        lhs.add(rhs, graph)?;
                        env.stack.push(lhs);
                    } else {
                        return Err(Error::Add);
                    }
                } else {
                    return Err(Error::Add);
                }
            },
            Self::Sub => {
                let lhs = env.stack.pop();
                let rhs = env.stack.pop();
                if let Some(lhs) = lhs {
                    if let Some(rhs) = rhs {
                        lhs.sub(rhs, graph)?;
                        env.stack.push(lhs);
                    } else {
                        return Err(Error::Sub);
                    }
                } else {
                    return Err(Error::Sub);
                }
            },
            Self::Mul => {
                let lhs = env.stack.pop();
                let rhs = env.stack.pop();
                if let Some(lhs) = lhs {
                    if let Some(rhs) = rhs {
                        lhs.mul(rhs, graph)?;
                        env.stack.push(lhs);
                    } else {
                        return Err(Error::Mul);
                    }
                } else {
                    return Err(Error::Mul);
                }
            },
            Self::Div => {
                let lhs = env.stack.pop();
                let rhs = env.stack.pop();
                if let Some(lhs) = lhs {
                    if let Some(rhs) = rhs {
                        lhs.div(rhs, graph)?;
                        env.stack.push(lhs);
                    } else {
                        return Err(Error::Div);
                    }
                } else {
                    return Err(Error::Div);
                }
            },
            Self::Mod => {
                let lhs = env.stack.pop();
                let rhs = env.stack.pop();
                if let Some(lhs) = lhs {
                    if let Some(rhs) = rhs {
                        lhs.rem(rhs, graph)?;
                        env.stack.push(lhs);
                    } else {
                        return Err(Error::Mod);
                    }
                } else {
                    return Err(Error::Mod);
                }
            },
            Self::AND => {
                let lhs = env.stack.pop();
                let rhs = env.stack.pop();
                if let Some(lhs) = lhs {
                    if let Some(rhs) = rhs {
                        lhs.bit_and(rhs)?;
                        env.stack.push(lhs);
                    } else {
                        return Err(Error::AND);
                    }
                } else {
                    return Err(Error::AND);
                }
            },
            Self::OR => {
                let lhs = env.stack.pop();
                let rhs = env.stack.pop();
                if let Some(lhs) = lhs {
                    if let Some(rhs) = rhs {
                        lhs.bit_or(rhs)?;
                        env.stack.push(lhs);
                    } else {
                        return Err(Error::OR);
                    }
                } else {
                    return Err(Error::OR);
                }
            },
            Self::XOR => {
                let lhs = env.stack.pop();
                let rhs = env.stack.pop();
                if let Some(lhs) = lhs {
                    if let Some(rhs) = rhs {
                        lhs.bit_xor(rhs)?;
                        env.stack.push(lhs);
                    } else {
                        return Err(Error::XOR);
                    }
                } else {
                    return Err(Error::XOR);
                }
            },
            Self::SHL => {
                let lhs = env.stack.pop();
                let rhs = env.stack.pop();
                if let Some(lhs) = lhs {
                    if let Some(rhs) = rhs {
                        lhs.bit_shl(rhs)?;
                        env.stack.push(lhs);
                    } else {
                        return Err(Error::SHL);
                    }
                } else {
                    return Err(Error::SHL);
                }
            },
            Self::SHR => {
                let lhs = env.stack.pop();
                let rhs = env.stack.pop();
                if let Some(lhs) = lhs {
                    if let Some(rhs) = rhs {
                        lhs.bit_shr(rhs)?;
                        env.stack.push(lhs);
                    } else {
                        return Err(Error::SHR);
                    }
                } else {
                    return Err(Error::SHR);
                }
            },
        };
        Ok(None)
    }
}
