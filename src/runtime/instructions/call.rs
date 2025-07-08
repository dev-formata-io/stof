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
use imbl::Vector;
use serde::{Deserialize, Serialize};
use crate::{model::{DataRef, Field, Func, Graph, NodeRef, Prototype, SId, ASYNC_FUNC_ATTR, SELF_STR_KEYWORD, SUPER_STR_KEYWORD, TYPENAME}, runtime::{instruction::{Instruction, Instructions}, instructions::{Base, POP_CALL, POP_SELF, POP_SYMBOL_SCOPE, PUSH_CALL, PUSH_SELF, PUSH_SYMBOL_SCOPE, SUSPEND}, proc::ProcEnv, Error, Type, Val, ValRef}};


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Call a function instruction (expr).
/// An expression will add this as the next instruction after a lookup to an internal function.
pub struct FuncCall {
    /// Specific function we are calling.
    pub func: Option<DataRef>,

    /// Optionally look up the function from a path in the graph.
    pub search: Option<ArcStr>,

    /// Look on the stack for the context of this call?
    /// Will pop a value from the stack to use it.
    /// Used when chaining stuff together Ex. hello[15].my_func('hi').dude()
    pub stack: bool,
    
    /// Single instruction for each argument (think of it like an expr)!
    pub args: Vector<Arc<dyn Instruction>>,
}
impl FuncCall {
    /// Find function (Or library name & function).
    /// Uses search or the stack to find the function we are going to call if needed.
    pub(self) fn get_func_context(&self, env: &mut ProcEnv, graph: &mut Graph) -> Result<CallContext, Error> {
        if let Some(dref) = &self.func {
            return Ok(CallContext { lib: None, prototype: false, func: dref.clone() });
        }
        if let Some(search) = &self.search {
            return self.search_func(&search, env, graph);
        }
        Err(Error::FuncDne)
    }

    /// Search for a function to call using a path.
    /// If "stack" is set, pop the stack and use the result as a context or library name.
    fn search_func(&self, path: &str, env: &mut ProcEnv, graph: &mut Graph) -> Result<CallContext, Error> {
        let mut split_path = path.split('.').collect::<Vec<_>>();
        if split_path.len() < 1 { return Err(Error::FuncDne); }

        // In this case, we have a chained value already on the stack that we are adding a call to
        if self.stack {
            if let Some(var) = env.stack.pop() {
                if split_path.len() > 1 {
                    // {val}.additional...function_call() case, where val is a stack variable and not in path
                    // In this case, val must be an object to continue the lookup
                    if let Some(obj) = var.try_obj() {
                        return self.object_search(path, Some(obj), graph, false);
                    }
                } else {
                    // {val}.function_call() case, where val is a stack variable and not in path
                    if let Some(obj) = var.try_obj() {
                        // Try finding a function with this name on the object before using the obj lib
                        if let Ok(res) = self.object_search(path, Some(obj), graph, false) {
                            return Ok(res);
                        }
                    }
                    let libname = var.lib_name(&graph);
                    return Ok(CallContext { lib: Some(libname), prototype: false, func: SId::from(split_path[0]) });
                }
            }
            return Err(Error::FuncDne);
        }

        // In this case, we are calling into the standard library functions
        if split_path.len() < 2 {
            return Ok(CallContext { lib: Some(literal!("Std")), prototype: false, func: SId::from(split_path[0]) });
        }
        
        // In this case, we are searching for a generic path, using the symbol table, libraries, and graph
        let context;
        if split_path[0] == SELF_STR_KEYWORD.as_str() {
            // Note: keep "self" on the path otherwise drops to lib call
            context = ValRef::new(Val::Obj(env.self_ptr()));
        } else if split_path[0] == SUPER_STR_KEYWORD.as_str() {
            context = ValRef::new(Val::Obj(env.self_ptr()));
        } else if let Some(var) = env.table.get(split_path[0]) {
            context = var.val.clone();
            split_path.remove(0);
        } else {
            // Look for a function at the root of the graph before resorting to a library
            if let Ok(res) = self.object_search(path, None, graph, false) {
                return Ok(res);
            }
            // Only a valid libcall if the length is 2
            if split_path.len() == 2 {
                return Ok(CallContext { lib: Some(split_path[0].to_string().into()), prototype: false, func: SId::from(split_path[1]) });
            }
            return Err(Error::FuncDne);
        }

        let context_path = split_path.join(".");
        if let Some(obj) = context.read().try_obj() {
            // self.path.function();
            // super.path.function();
            if let Ok(res) = self.object_search(&context_path, Some(obj), graph, false) {
                return Ok(res);
            }
        }
        if split_path.len() < 2 {
            // var.split('.'); // string variable for example
            let libname = context.read().lib_name(&graph);
            return Ok(CallContext { lib: Some(libname), prototype: false, func: SId::from(split_path[0]) });
        }

        Err(Error::FuncDne)
    }

    /// Use the remaining path to find a function at the path starting at an object.
    /// This should include any prototypes that the object has.
    fn object_search(&self, path: &str, start: Option<NodeRef>, graph: &mut Graph, in_proto: bool) -> Result<CallContext, Error> {
        let mut allow_node_contemplation = true;

        // If we are in a prototype, check to see if the path has a specific type associated with it Ex. MyType::special_func().
        // If there's a special type and this node has the wrong typename, don't allow a function to resolve on it.
        if in_proto && path.contains("::") {
            if let Some(node) = &start {
                if let Some(node) = node.node(&graph) {
                    let type_path = path.split("::").collect::<Vec<_>>();
                    if let Some(val) = node.attributes.get(TYPENAME.as_str()) {
                        if type_path[0] != val.to_string() {
                            allow_node_contemplation = false;
                        }
                    }
                }
            }
        }
        
        if allow_node_contemplation {
            // Look for a function on the object at the path first (always highest priority)
            if let Some(func) = Func::func_from_path(graph, path, start.clone()) {
                return Ok(CallContext { lib: None, prototype: in_proto, func });
            }

            // Look for a field on the object at the path next that is a function
            // TODO: test this out and see if its wierd. means self.myobj.field() will work if field points to a function...
            if let Some(field) = Field::field_from_path(graph, path, start.clone()) {
                if let Some(field) = graph.get_stof_data::<Field>(&field) {
                    if let Some(func) = field.value.try_func() {
                        return Ok(CallContext { lib: None, prototype: in_proto, func });
                    }
                }
            }
        }

        // Look for a prototype that this object has next
        {
            let mut proto_context = start;
            let mut proto_path = path.split('.').collect::<Vec<_>>();
            let func_name = proto_path.pop().unwrap();

            if proto_path.len() > 0 {
                if let Some(node) = graph.find_node_named(&proto_path.join("."), proto_context.clone()) {
                    proto_context = Some(node);
                } else {
                    proto_context = None; // not valid since we have additional path
                }
            }
            if let Some(node) = proto_context {
                for prototype in Prototype::prototype_nodes(graph, &node) {
                    // by making this recursive, we fulfill the sub-typing lookups ("extends" types)
                    if let Ok(res) = self.object_search(func_name, Some(prototype), graph, true) {
                        return Ok(res);
                    }
                }
            }
        }

        // TODO
        // Look for a static function on a prototype with "::" (only works with "type" objects, not regular objects as a prototype)

        Err(Error::FuncDne)
    }
}


#[derive(Debug)]
pub(self) struct CallContext {
    pub lib: Option<ArcStr>,
    pub prototype: bool,
    pub func: SId,
}


#[typetag::serde(name = "FuncCall")]
impl Instruction for FuncCall {
    fn exec(&self, instructions: &mut Instructions, env: &mut ProcEnv, graph: &mut Graph) -> Result<(), Error> {
        let func_context = self.get_func_context(env, graph)?;
        if let Some(_libname) = func_context.lib {
            // TODO call into a library function, using context.func as lib name
            return Ok(());
        }
        let func = func_context.func;
        
        let params;
        let func_instructions;
        let rtype;
        let is_async;
        if let Some(func) = graph.get_stof_data::<Func>(&func) {
            params = func.params.clone();
            func_instructions = func.instructions.clone();
            rtype = func.return_type.clone();

            // Only async if we have the attribute and we are not a top level function
            is_async = func.attributes.contains_key(ASYNC_FUNC_ATTR.as_str()) && env.call_stack.len() > 0;
        } else {
            return Err(Error::FuncDne);
        }
       
        // Push call stack, start a new scope, and add self if needed
        instructions.push(Arc::new(Base::Literal(Val::Fn(func.clone()))));
        instructions.push(PUSH_CALL.clone());
        instructions.push(PUSH_SYMBOL_SCOPE.clone());
        
        // Add self to self stack if not a prototype function
        if !func_context.prototype {
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

        // Push the function instructions
        instructions.push(PUSH_SYMBOL_SCOPE.clone());
        instructions.append(&func_instructions);
        if !rtype.empty() {
            instructions.push(Arc::new(Base::Cast(rtype.clone())));
        } else {
            // Make sure we get an error if the last value is not void (or doesn't exist on stack)
            instructions.push(Arc::new(Base::Cast(Type::Void)));
        }

        // Cleanup stacks
        instructions.push(POP_SYMBOL_SCOPE.clone());
        instructions.push(POP_SYMBOL_SCOPE.clone());
        instructions.push(POP_CALL.clone());
        
        // Pop self stack
        if !func_context.prototype {
            instructions.push(POP_SELF.clone());
        }

        // Handle async function call
        if is_async {
            let async_instructions = instructions.clone();
            instructions.clear();

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
