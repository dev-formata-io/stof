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

use std::{mem::swap, sync::Arc};
use arcstr::ArcStr;
use imbl::Vector;
use serde::{Deserialize, Serialize};
use crate::{model::{DataRef, Field, Func, Graph, SId, ASYNC_FUNC_ATTR, SELF_STR_KEYWORD, SUPER_STR_KEYWORD}, runtime::{instruction::{Instruction, Instructions}, instructions::{Base, POP_CALL, POP_SELF, POP_SYMBOL_SCOPE, PUSH_CALL, PUSH_SELF, PUSH_SYMBOL_SCOPE, SUSPEND}, proc::ProcEnv, Error, Val}};


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Call a function instruction (expr).
/// An expression will add this as the next instruction after a lookup to an internal function.
pub struct FuncCall {
    pub add_self: bool,

    /// Function we are calling.
    /// If none, will look on the stack when this gets executed.
    pub func: Option<DataRef>,

    /// Optionally look up the function instead (just like LoadVariable).
    pub func_lookup: Option<ArcStr>,
    
    /// Single instruction for each argument (think of it like an expr)!
    pub args: Vector<Arc<dyn Instruction>>,
}


#[typetag::serde(name = "FuncCall")]
impl Instruction for FuncCall {
    fn exec(&self, instructions: &mut Instructions, env: &mut ProcEnv, graph: &mut Graph) -> Result<(), Error> {
        let func;
        if let Some(dref) = &self.func {
            func = dref.clone();
        } else if let Some(name) = &self.func_lookup {
            if name.starts_with(SELF_STR_KEYWORD.as_str()) || name.starts_with(SUPER_STR_KEYWORD.as_str()) {
                let self_ptr = env.self_ptr();
                if let Some(field) = Field::field_from_path(graph, &name, Some(self_ptr.clone())) {
                    if let Some(field) = graph.get_stof_data::<Field>(&field) {
                        if let Some(dref) = field.value.try_func() {
                            func = dref;
                        } else {
                            return Err(Error::FuncDne);
                        }
                    } else {
                        return Err(Error::FuncDne);
                    }
                } else if let Some(func_ref) = Func::func_from_path(graph, &name, Some(self_ptr.clone())) {
                    func = func_ref;
                } else {
                    return Err(Error::FuncDne);
                }
            } else if let Some(field) = Field::field_from_path(graph, &name, None) {
                if let Some(field) = graph.get_stof_data::<Field>(&field) {
                    if let Some(dref) = field.value.try_func() {
                        func = dref;
                    } else {
                        return Err(Error::FuncDne);
                    }
                } else {
                    return Err(Error::FuncDne);
                }
            } else if let Some(func_ref) = Func::func_from_path(graph, &name, None) {
                func = func_ref;
            } else {
                return Err(Error::FuncDne);
            }
        } else if let Some(val) = env.stack.pop() {
            if let Some(dref) = val.try_func() {
                func = dref;
            } else {
                return Err(Error::FuncDne);
            }
        } else {
            return Err(Error::FuncDne);
        }
        
        let params;
        let func_instructions;
        let rtype;
        let is_async;
        if let Some(func) = graph.get_stof_data::<Func>(&func) {
            params = func.params.clone();
            func_instructions = func.instructions.clone();
            rtype = func.return_type.clone();
            is_async = func.attributes.contains_key(&ASYNC_FUNC_ATTR);
        } else {
            return Err(Error::FuncDne);
        }
       
        // Push call stack, start a new scope, and add self if needed
        instructions.push(Arc::new(Base::Literal(Val::Fn(func.clone()))));
        instructions.push(PUSH_CALL.clone());
        instructions.push(PUSH_SYMBOL_SCOPE.clone());
        if self.add_self {
            let mut set = false;
            for nref in func.data_nodes(graph) {
                if nref.node_exists(graph) {
                    instructions.push(Arc::new(Base::Literal(Val::Obj(nref))));
                    instructions.push(PUSH_SELF.clone());
                    set = true; break;
                }
            }
            if !set {
                instructions.push(Arc::new(Base::Literal(Val::Obj(graph.ensure_main_root()))));
                instructions.push(PUSH_SELF.clone());
            }
        }
        
        // Arguments
        let mut named_args = Vec::new();
        let mut args = Vec::new();
        for arg in &self.args {
            if let Some(named) = arg.as_dyn_any().downcast_ref::<NamedArg>() {
                let mut index = 0;
                let mut found = false;
                for pn in &params {
                    if pn.name == named.name {
                        named_args.push((index, named.ins.clone()));
                        found = true; break;
                    }
                    index += 1;
                }
                if !found {
                    // TODO
                    return Err(Error::FuncArgs);
                }
            } else {
                args.push(arg.clone());
            }
        }
        if !named_args.is_empty() {
            named_args.sort_by(|a, b| a.0.cmp(&b.0));
            for (index, ins) in named_args {
                while index > args.len() {
                    if let Some(param) = params.get(args.len()) {
                        if let Some(default) = &param.default {
                            args.push(default.clone());
                        } else {
                            return Err(Error::FuncArgs);
                        }
                    } else {
                        return Err(Error::FuncArgs);
                    }
                }
                args.insert(index, ins);
            }
        }
        if args.len() < params.len() {
            let mut index = args.len();
            while index < params.len() {
                let param = &params[index];
                if let Some(default) = &param.default {
                    args.push(default.clone());
                } else {
                    break;
                }
                index += 1;
            }
        }
        if args.len() != params.len() {
            return Err(Error::FuncArgs);
        }
        for index in 0..args.len() {
            let param = &params[index];
            let arg = &args[index];
            instructions.push(arg.clone());
            instructions.push(Arc::new(Base::Cast(param.param_type.clone())));
            instructions.push(Arc::new(Base::DeclareVar(param.name.to_string().into(), true))); // these must keep their type
        }
        for arg in args {
            instructions.push(arg);
        }

        // Push the function instructions
        instructions.push(PUSH_SYMBOL_SCOPE.clone());
        instructions.append(&func_instructions);
        if !rtype.empty() {
            instructions.push(Arc::new(Base::Cast(rtype.clone())));
        }

        // Cleanup stacks
        instructions.push(POP_SYMBOL_SCOPE.clone());
        instructions.push(POP_SYMBOL_SCOPE.clone());
        instructions.push(POP_CALL.clone());
        if self.add_self {
            instructions.push(POP_SELF.clone());
        }

        // Handle async function call
        if is_async {
            let mut async_instructions = Instructions::default();
            swap(&mut async_instructions, instructions); // instructions now empty again

            instructions.push(Arc::new(Base::Spawn((async_instructions, rtype)))); // adds a Promise<rtype> to the stack when executed!
            instructions.push(SUSPEND.clone()); // make sure to spawn the process right after with the runtime... this is not an await
        }
        Ok(())
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Named argument instruction.
/// Use this in function args when you want to insert a named argument.
/// Function knows how to take care of this.
pub struct NamedArg {
    pub name: SId,
    pub ins: Arc<dyn Instruction>,
}
#[typetag::serde(name = "NamedArg")]
impl Instruction for NamedArg {
    fn exec(&self, instructions: &mut Instructions, _env: &mut ProcEnv, _graph: &mut Graph) -> Result<(), Error> {
        instructions.push(self.ins.clone());
        Ok(())
    }
}
