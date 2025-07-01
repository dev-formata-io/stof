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
use arcstr::ArcStr;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use crate::{model::{DataRef, Field, Func, Graph, NodeRef, SPath, SELF_STR_KEYWORD, SUPER_STR_KEYWORD}, runtime::{instruction::{Instruction, Instructions}, proc::{ProcEnv, Process}, Error, Type, Val, Variable}};

//pub mod declare;
//pub mod call;


// static instructions for efficiency
lazy_static! {
    pub static ref SUSPEND: Arc<dyn Instruction> = Arc::new(Base::Suspend);

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
    Suspend,

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
    Spawn(Process),

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
}
#[typetag::serde(name = "Base")]
impl Instruction for Base {
    fn exec(&self, instructions: &mut Instructions, env: &mut ProcEnv, graph: &mut Graph) -> Result<(), Error> {
        match self {
            /*****************************************************************************
             * Suspend.
             *****************************************************************************/
            Self::Suspend => {}, // Nothing here...
            

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
            
            Self::Spawn(proc) => {
                env.spawn = Some(Box::new(proc.clone()));
                instructions.push(SUSPEND.clone()); // make sure to suspend this proc after a spawn!
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
                            // creating a new field with a path from self
                            let mut path = SPath::from(name);
                            let field_name = path.path.pop().unwrap();
                            if path.path.len() < 1 { return Err(Error::AssignSelf); }
                            if let Some(node) = graph.ensure_named_nodes(path, Some(self_ptr.clone()), true, None) {
                                
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
                    } else {

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
        };
        Ok(())
    }

    /// Is a suspend operation?
    /// This kind of operation will not get executed, nor will it be placed in the instruction stack.
    /// It will prompt the rotating of processes though... so make sure to include them!
    fn suspend_op(&self) -> bool {
        match self {
            Self::Suspend => true,
            _ => false,
        }
    }
}
