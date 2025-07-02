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

use std::sync::Arc;
use arcstr::{literal, ArcStr};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use crate::{model::{Field, Func, Graph, SPath, SELF_STR_KEYWORD, SUPER_STR_KEYWORD}, runtime::{instruction::{Instruction, Instructions}, proc::{ProcEnv, Process}, Error, Type, Val, Variable}};

pub mod call;
pub mod block;


// static instructions for efficiency
lazy_static! {
    pub static ref SUSPEND: Arc<dyn Instruction> = Arc::new(Base::CtrlSuspend);
    pub static ref AWAIT: Arc<dyn Instruction> = Arc::new(Base::CtrlAwait);

    pub static ref START_TAG: Arc<dyn Instruction> = Arc::new(Base::Tag(literal!("start")));
    pub static ref END_TAG: Arc<dyn Instruction> = Arc::new(Base::Tag(literal!("end")));
    pub static ref CONTINUE: Arc<dyn Instruction> = Arc::new(Base::CtrlBackTo(literal!("start")));
    pub static ref BREAK: Arc<dyn Instruction> = Arc::new(Base::CtrlForwardTo(literal!("end")));

    pub static ref PUSH_SELF: Arc<dyn Instruction> = Arc::new(Base::PushSelf);
    pub static ref POP_SELF: Arc<dyn Instruction> = Arc::new(Base::PopSelf);

    pub static ref PUSH_CALL: Arc<dyn Instruction> = Arc::new(Base::PushCall);
    pub static ref POP_CALL: Arc<dyn Instruction> = Arc::new(Base::PopCall);

    pub static ref PUSH_NEW: Arc<dyn Instruction> = Arc::new(Base::PushNew);
    pub static ref POP_NEW: Arc<dyn Instruction> = Arc::new(Base::PopNew);

    pub static ref POP_STACK: Arc<dyn Instruction> = Arc::new(Base::PopStack);

    pub static ref PUSH_SYMBOL_SCOPE: Arc<dyn Instruction> = Arc::new(Base::PushSymbolScope);
    pub static ref POP_SYMBOL_SCOPE: Arc<dyn Instruction> = Arc::new(Base::PopSymbolScope);
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Foundational instructions.
/// Higher level instructions JIT down into a subset of these as they execute.
pub enum Base {
    // Suspend instruction.
    // Used to denote going to another process now.
    // Place these after runs of instructions to make sure we keep making progress on other processes too.
    CtrlSuspend,
    // Instruct the system to wait for this process before continuing. Looks for a Promise on the stack.
    // Load a promise onto the stack, then insert this instruction to wait for the process to complete.
    CtrlAwait,

    // Tag a place in the instructions.
    // This is a form of GOTO, used for looping & control flow
    Tag(ArcStr),
    CtrlBackTo(ArcStr), // start next on instruction right after tag
    CtrlForwardTo(ArcStr), // start next on instruction right after tag

    // Self stack.
    PushSelf,
    PopSelf,

    // Call stack.
    PushCall,
    PopCall,

    // New obj stack.
    PushNew,
    PopNew,

    // Push literal to stack.
    PushStack(Val),
    PopStack,

    // Spawn a new process.
    Spawn((Instructions, Type)),

    // Symbol table / Graph.
    PushSymbolScope,
    PopSymbolScope,

    DeclareVar(ArcStr), // requires val on stack
    DeclareConstVar(ArcStr), // requires val on stack
    
    DropVariable(ArcStr), // removes from the st/graph
    LoadVariable(ArcStr), // loads st/graph to stack
    SetVariable(ArcStr), // requires val on stack

    // Values.
    Literal(Val), // load a literal onto the stack
    Cast(Type), // Cast value on the back of the stack to a specific type
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}
#[typetag::serde(name = "Base")]
impl Instruction for Base {
    /// Base instructions do not replace themselves and are used by other higher-order instructions.
    /// Know what you are doing if using these.
    fn exec(&self, _instructions: &mut Instructions, env: &mut ProcEnv, graph: &mut Graph) -> Result<(), Error> {
        match self {
            /*****************************************************************************
             * Suspend.
             *****************************************************************************/
            Self::CtrlSuspend => {}, // Nothing here...
            Self::CtrlAwait => {}, // Nothing here...
            
            /*****************************************************************************
             * Tags.
             *****************************************************************************/
            Self::Tag(_id) => {}, // Nothing here... just goes on through to mark a place
            Self::CtrlBackTo(_id) => {}, // Nothing here... used by instructions...
            Self::CtrlForwardTo(_id) => {}, // Nothing here... used by instructions...

            /*****************************************************************************
             * Special stacks.
             *****************************************************************************/
            Self::PushSelf => {
                if let Some(val) = env.stack.pop() {
                    if let Some(obj) = val.try_obj() {
                        env.self_stack.push(obj);
                        return Ok(());
                    }
                }
                return Err(Error::SelfStackError);
            },
            Self::PopSelf => { env.self_stack.pop(); },

            Self::PushCall => {
                if let Some(val) = env.stack.pop() {
                    if let Some(func) = val.try_func() {
                        env.call_stack.push(func);
                        return Ok(());
                    }
                }
                return Err(Error::CallStackError);
            },
            Self::PopCall => { env.call_stack.pop(); },
            
            Self::PushNew => {
                if let Some(val) = env.stack.pop() {
                    if let Some(obj) = val.try_obj() {
                        env.new_stack.push(obj);
                        return Ok(());
                    }
                }
                return Err(Error::NewStackError);
            },
            Self::PopNew => { env.new_stack.pop(); },
            
            Self::PushStack(val) => env.stack.push(val.clone()),
            Self::PopStack => { env.stack.pop(); },
            
            /*****************************************************************************
             * Spawn a new process.
             *****************************************************************************/
            
            Self::Spawn((async_ins, ty)) => {
                // Creates a new PID every time here, avoiding a lot of issues...
                let proc = Process::from(async_ins.clone());
                let pid = proc.env.pid.clone();
                env.spawn = Some(Box::new(proc));
                env.stack.push(Val::Promise(pid, ty.clone()));
                // up to the caller to add the suspend to actually spawn (don't want this ins replaced)
            },
            
            /*****************************************************************************
             * Variables.
             *****************************************************************************/
            
            Self::PushSymbolScope => env.table.push(),
            Self::PopSymbolScope => { env.table.pop(); },
            Self::DeclareVar(name) => {
                if !env.table.can_declare(name) { return Err(Error::DeclareExisting); }
                if name.contains('.') { return Err(Error::DeclareInvalidName); }
                if let Some(val) = env.stack.pop() {
                    env.table.insert(name, Variable::new(true, val));
                } else {
                    return Err(Error::StackError);
                }
            },
            Self::DeclareConstVar(name) => {
                if !env.table.can_declare(name) { return Err(Error::DeclareExisting); }
                if name.contains('.') { return Err(Error::DeclareInvalidName); }
                if let Some(val) = env.stack.pop() {
                    env.table.insert(name, Variable::new(false, val));
                } else {
                    return Err(Error::StackError);
                }
            },
            Self::DropVariable(name) => {
                if !name.contains('.') {
                    if let Some(var) = env.table.drop_var(name) {
                        var.drop_data(graph);
                        return Ok(());
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
                            val.read().unwrap().drop_data(graph);
                        }
                        graph.remove_data(&field, None);
                    } else if let Some(node) = SPath::node(&graph, &name, Some(self_ptr.clone())) {
                        // TODO remove types for node
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
                        val.read().unwrap().drop_data(graph);
                    }
                    graph.remove_data(&field, None);
                } else if let Some(node) = SPath::node(&graph, &name, None) {
                        // TODO remove types for node
                        graph.remove_node(&node, false);
                } else if let Some(func) = Func::func_from_path(graph, &name, None) {
                    graph.remove_data(&func, None);
                }
            },
            Self::LoadVariable(name) => {
                if !name.contains('.') {
                    if let Some(var) = env.table.get(name) {
                        env.stack.push(var.get());
                        return Ok(());
                    }
                }
                if name.starts_with(SELF_STR_KEYWORD.as_str()) || name.starts_with(SUPER_STR_KEYWORD.as_str()) {
                    let self_ptr = env.self_ptr();
                    if let Some(field) = Field::field_from_path(graph, &name, Some(self_ptr.clone())) {
                        if let Some(field) = graph.get_stof_data::<Field>(&field) {
                            env.stack.push(field.value.get());
                            return Ok(());
                        }
                    } else if let Some(node) = SPath::node(&graph, &name, Some(self_ptr.clone())) {
                        env.stack.push(Val::Obj(node));
                        return Ok(());
                    } else if let Some(func) = Func::func_from_path(graph, &name, Some(self_ptr.clone())) {
                        env.stack.push(Val::Fn(func));
                        return Ok(());
                    }
                } else if let Some(field) = Field::field_from_path(graph, &name, None) {
                    if let Some(field) = graph.get_stof_data::<Field>(&field) {
                        env.stack.push(field.value.get());
                        return Ok(());
                    }
                } else if let Some(node) = SPath::node(&graph, &name, None) {
                    env.stack.push(Val::Obj(node));
                    return Ok(());
                } else if let Some(func) = Func::func_from_path(graph, &name, None) {
                    env.stack.push(Val::Fn(func));
                    return Ok(());
                }
                env.stack.push(Val::Null);
                return Ok(());
            },
            Self::SetVariable(name) => {
                if let Some(val) = env.stack.pop() {
                    if !name.contains('.') && env.table.set(name, &val)? {
                        return Ok(());
                    }

                    if name.starts_with(SELF_STR_KEYWORD.as_str()) || name.starts_with(SUPER_STR_KEYWORD.as_str()) {
                        if name == &SELF_STR_KEYWORD {
                            return Err(Error::AssignSelf);
                        }
                        if name == &SUPER_STR_KEYWORD {
                            return Err(Error::AssignSuper);
                        }
                        
                        let self_ptr = env.self_ptr();
                        if let Some(field_ref) = Field::field_from_path(graph, &name, Some(self_ptr.clone())) {
                            if let Some(field) = graph.get_mut_stof_data::<Field>(&field_ref) {
                                field.try_set(val)?;
                            }
                            if let Some(field) = field_ref.data_mut(graph) {
                                field.invalidate_value();
                            }
                            return Ok(());
                        } else {
                            let mut path = SPath::from(name);
                            let field_name = path.path.pop().unwrap();
                            if path.path.len() < 1 { return Err(Error::AssignSelf); }
                            if let Some(node) = graph.ensure_named_nodes(path, Some(self_ptr.clone()), true, None) {
                                let field = Field::new(Variable::new(true, val), None);
                                graph.insert_stof_data(&node, field_name, Box::new(field), None);
                                return Ok(());
                            } else {
                                return Err(Error::AssignSelf);
                            }
                        }
                    } else if let Some(field_ref) = Field::field_from_path(graph, &name, None) {
                        if let Some(field) = graph.get_mut_stof_data::<Field>(&field_ref) {
                            field.try_set(val)?;
                        }
                        if let Some(field) = field_ref.data_mut(graph) {
                            field.invalidate_value();
                        }
                        return Ok(());
                    } else if name.contains('.') {
                        let mut path = SPath::from(name);
                        let field_name = path.path.pop().unwrap();
                        if path.path.len() < 1 { return Err(Error::AssignSelf); }
                        if let Some(node) = graph.ensure_named_nodes(path, None, true, None) {
                            let field = Field::new(Variable::new(true, val), None);
                            graph.insert_stof_data(&node, field_name, Box::new(field), None);
                            return Ok(());
                        } else {
                            return Err(Error::AssignSelf);
                        }
                    } else {
                        match val {
                            Val::Obj(nref) => {
                                // TODO: drop old root?
                                if let Some(node) = nref.node_mut(graph) {
                                    node.name = name.into();
                                }
                                graph.roots.insert(nref);
                                return Ok(());
                            },
                            _ => {
                                return Err(Error::AssignRootNonObj);
                            }
                        }
                    }
                } else {
                    return Err(Error::StackError);
                }
            },

            /*****************************************************************************
             * Values.
             *****************************************************************************/
            Self::Literal(val) => {
                env.stack.push(val.clone());
            },
            Self::Cast(target) => {
                if let Some(mut val) = env.stack.pop() {
                    val.cast(target, graph)?;
                    env.stack.push(val);
                } else {
                    return Err(Error::CastStackError);
                }
            },
            Self::Add => {
                let lhs = env.stack.pop();
                let rhs = env.stack.pop();
                if let Some(lhs) = lhs {
                    if let Some(rhs) = rhs {
                        env.stack.push(lhs.add(rhs)?);
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
                        //env.stack.push(lhs.sub(rhs)?);
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
                        //env.stack.push(lhs.mul(rhs)?);
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
                        //env.stack.push(lhs.div(rhs)?);
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
                        //env.stack.push(lhs.rem(rhs)?);
                    } else {
                        return Err(Error::Mod);
                    }
                } else {
                    return Err(Error::Mod);
                }
            },
        };
        Ok(())
    }
}
