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

use std::{ops::Deref, sync::Arc};
use arcstr::{literal, ArcStr};
use imbl::Vector;
use serde::{Deserialize, Serialize};
use crate::{model::{DataRef, Field, Func, Graph, LibFunc, NodeRef, Prototype, SId, ASYNC_FUNC_ATTR, PROTOTYPE_TYPE_ATTR, SELF_STR_KEYWORD, SUPER_STR_KEYWORD, UNSELF_FUNC_ATTR}, runtime::{instruction::{Instruction, Instructions}, instructions::{Base, DUPLICATE, POP_CALL, POP_RETURN, POP_SELF, PUSH_CALL, PUSH_RETURN, PUSH_SELF, PUSH_SYMBOL_SCOPE, PUSH_VAL_RET, PUSH_VOID_RET, SUSPEND, VALIDATE_FN_RET}, proc::ProcEnv, Error, Type, Val, ValRef, Variable}};


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

    /// Is this function call by reference?
    pub as_ref: bool,

    /// To null (null check)?
    /// Means instead of a FuncDne, you'll get a null value.
    pub cnull: bool,
    
    /// Single instruction for each argument (think of it like an expr)!
    pub args: Vector<Arc<dyn Instruction>>,

    /// Override self for this function call?
    pub oself: Option<Arc<dyn Instruction>>,
}
impl FuncCall {
    /// Find function (Or library name & function).
    /// Uses search or the stack to find the function we are going to call if needed.
    pub(self) fn get_func_context(&self, env: &mut ProcEnv, graph: &mut Graph) -> Result<CallContext, Error> {
        if let Some(dref) = &self.func {
            return Ok(CallContext { lib: None, stack_arg: None, prototype_self: None, func: dref.clone() });
        }
        if let Some(search) = &self.search {
            // remove all '?', because they may be present in the path (Ex. hello?.hi()).
            let path = search.replace('?', "");
            return self.search_func(&path, env, graph);
        }
        Err(Error::FuncDne("No Func or Search".into()))
    }

    /// Search for a function to call using a path.
    /// If "stack" is set, pop the stack and use the result as a context or library name.
    fn search_func(&self, path: &str, env: &mut ProcEnv, graph: &mut Graph) -> Result<CallContext, Error> {
        let mut split_path = path.split('.').collect::<Vec<_>>();

        // In this case, we have a chained value already on the stack that we are adding a call to
        if self.stack {
            if let Some(var) = env.stack.pop() {
                if split_path.len() > 1 {
                    // {val}.additional...function_call() case, where val is a stack variable and not in path
                    // In this case, val must be an object to continue the lookup
                    if let Some(obj) = var.try_obj() {
                        return self.object_search(path, Some(obj), env, graph, false);
                    }
                } else {
                    // {val}.function_call() case, where val is a stack variable and not in path
                    if let Some(obj) = var.try_obj() {
                        // Try finding a function with this name on the object before using the obj lib
                        if let Ok(res) = self.object_search(path, Some(obj), env, graph, false) {
                            return Ok(res);
                        }
                    }

                    // val is a func case {val}()
                    if split_path[0].len() < 1 {
                        if let Some(dref) = var.try_func() {
                            return Ok(CallContext { lib: None, prototype_self: None, func: dref, stack_arg: Some(Arc::new(Base::Variable(var))) });
                        }
                    }

                    let libname = var.lib_name(&graph);
                    return Ok(CallContext { lib: Some(libname), stack_arg: Some(Arc::new(Base::Variable(var))), prototype_self: None, func: SId::from(split_path[0]) });
                }
            }
            return Err(Error::FuncDne("Stack Search Failed".into()));
        }

        // In this case, we are calling into the standard library functions (or a variable function)
        if split_path.len() < 2 {
            let name = split_path[0];
            if let Some(var) = env.table.get(name) {
                if let Some(func) = var.try_func() {
                    // Calling directly into a variable function
                    return Ok(CallContext { lib: None, prototype_self: None, func, stack_arg: None });
                }
            }
            if name == "this" && env.call_stack.len() > 0 {
                return Ok(CallContext { lib: None, stack_arg: None, prototype_self: None, func: env.call_stack.last().unwrap().clone() });
            }
            return Ok(CallContext { lib: Some(literal!("Std")), stack_arg: None, prototype_self: None, func: SId::from(split_path[0]) });
        }
        
        // In this case, we are searching for a generic path, using the symbol table, libraries, and graph
        let context;
        if split_path[0] == SELF_STR_KEYWORD.as_str() {
            context = ValRef::new(Val::Obj(env.self_ptr()));
            split_path.remove(0);
        } else if split_path[0] == SUPER_STR_KEYWORD.as_str() {
            if let Some(parent) = env.self_ptr().node_parent(&graph) {
                context = ValRef::new(Val::Obj(parent));
                split_path.remove(0);
            } else {
                context = ValRef::new(Val::Obj(env.self_ptr()));
            }
        } else if let Some(var) = env.table.get(split_path[0]) {
            context = var.val.clone();
            split_path.remove(0);
        } else if split_path[0] == "this" && env.call_stack.len() > 0 {
            context = ValRef::new(Val::Fn(env.call_stack.last().unwrap().clone()));
            split_path.remove(0);
        } else {
            // Look for a function at the root of the graph before resorting to a library
            if let Ok(res) = self.object_search(path, None, env, graph, false) {
                return Ok(res);
            }

            // Only a valid libcall if the length is 2 and a library function exists
            if split_path.len() == 2 {
                let libname = ArcStr::from(split_path[0]);
                if graph.libfunc(&libname, split_path[1]).is_some() {
                    return Ok(CallContext { lib: Some(libname), stack_arg: None, prototype_self: None, func: SId::from(split_path[1]) });
                }
            }

            // Now we are looking for a path + implied library function
            if split_path.len() > 1 {
                let func_name = split_path.pop().unwrap();
                let path = split_path.join(".");

                // using Base.LoadVariable as a meta instruction for accuracy
                // var will be loaded onto the env.stack!
                Base::LoadVariable(path.into(), false, false).exec(env, graph)?;
                let var = env.stack.pop().unwrap(); // yup, cool

                let libname = var.lib_name(&graph);
                return Ok(CallContext { lib: Some(libname), stack_arg: Some(Arc::new(Base::Variable(var))), prototype_self: None, func: SId::from(func_name) });
            }
            return Err(Error::FuncDne(path.into()));
        }

        let context_path = split_path.join(".");
        if let Some(obj) = context.read().try_obj() {
            // self.path.function();
            // super.path.function();
            if let Ok(res) = self.object_search(&context_path, Some(obj), env, graph, false) {
                return Ok(res);
            }
        }
        if split_path.len() < 2 {
            // var.split('.'); // string variable for example
            let libname = context.read().lib_name(&graph);
            return Ok(CallContext { lib: Some(libname), stack_arg: Some(Arc::new(Base::Variable(Variable::refval(context)))), prototype_self: None, func: SId::from(split_path[0]) });
        }

        Err(Error::FuncDne(path.into()))
    }

    /// Use the remaining path to find a function at the path starting at an object.
    /// This should include any prototypes that the object has.
    fn object_search(&self, path: &str, start: Option<NodeRef>, env: &mut ProcEnv, graph: &mut Graph, in_proto: bool) -> Result<CallContext, Error> {
        let mut allow_node_contemplation = true;

        // If we are in a prototype, check to see if the path has a specific type associated with it Ex. special_func<MyType>().
        // If there's a special type and this node has the wrong typename, don't allow a function to resolve on it.
        let mut adjusted_path = path.to_string();
        if in_proto && path.contains("<") {
            if let Some(node) = &start {
                if let Some(node) = node.node(&graph) {
                    let mut type_path = path.split("<").collect::<Vec<_>>();
                    if let Some(type_attr) = node.attributes.get(PROTOTYPE_TYPE_ATTR.as_str()) {
                        match type_attr {
                            Val::Str(name) => {
                                adjusted_path = type_path.pop().unwrap().trim_end_matches(">").to_string();
                                if adjusted_path != name.as_str() {
                                    allow_node_contemplation = false;
                                } else {
                                    adjusted_path = type_path.join("<");
                                }
                            },
                            _ => {
                                adjusted_path = type_path.pop().unwrap().trim_end_matches(">").to_string();
                                if adjusted_path != node.name.as_ref() {
                                    allow_node_contemplation = false;
                                } else {
                                    adjusted_path = type_path.join("<");
                                }
                            }
                        }
                    }
                }
            }
        }
        
        if allow_node_contemplation {
            // Look for a function on the object at the path first (always highest priority)
            if let Some(func) = Func::func_from_path(graph, &adjusted_path, start.clone()) {
                // prototype_self gets set below
                return Ok(CallContext { lib: None, stack_arg: None, prototype_self: None, func });
            }

            // Look for a field on the object at the path next that is a function
            if let Some(field) = Field::field_from_path(graph, &adjusted_path, start.clone()) {
                if let Some(field) = graph.get_stof_data::<Field>(&field) {
                    if let Some(func) = field.value.try_func() {
                        // prototype_self will get set below
                        return Ok(CallContext { lib: None, stack_arg: None, prototype_self: None, func });
                    }
                }
            }
        }

        // Look for a prototype that this object has next
        {
            let mut proto_context = start.clone();
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
                for prototype in Prototype::prototype_nodes(graph, &node, false) {
                    // by making this recursive, we fulfill the sub-typing lookups ("extends" types)
                    if let Ok(mut res) = self.object_search(func_name, Some(prototype), env, graph, true) {
                        if !in_proto {
                            // add this node to the self stack and mark as a prototype
                            res.prototype_self = Some(Arc::new(Base::Literal(Val::Obj(node))));
                        }
                        return Ok(res);
                    }
                }
            }
        }

        // Look for a static function on a prototype (only works with "type" objects, not regular objects as a prototype)
        // Ex. <MyType>.static_function();
        if !in_proto && path.starts_with('<') && path.contains('.') && path.contains('>') {
            let end_index = path.find('>').unwrap();
            let (mut first, mut last) = path.split_at(end_index);
            first = first.trim_start_matches('<');
            last = last.trim_start_matches('>').trim_start_matches('.');

            let mut obj_type = Type::Obj(first.into());
            obj_type.obj_to_proto(graph, Some(env.self_ptr()));
            match obj_type {
                Type::Obj(proto_id) => {
                    if proto_id.node_exists(graph) {
                        return self.object_search(last, Some(proto_id), env, graph, false);
                    }
                },
                _ => {}
            }
        }

        if allow_node_contemplation {
            // Look for a field (or obj) on the object at the path minus the func name for a library call on that field
            let mut field_path = adjusted_path.split('.').collect::<Vec<_>>();
            let func_name = field_path.pop().unwrap();
            let pth = field_path.join(".");
            if start.is_some() {
                if let Some(field) = Field::field_from_path(graph, &pth, start.clone()) {
                    if let Some(field) = graph.get_stof_data::<Field>(&field) {
                        let libname = field.value.val.read().lib_name(&graph);
                        return Ok(CallContext {
                            lib: Some(libname),
                            stack_arg: Some(Arc::new(Base::Variable(Variable::refval(field.value.val.duplicate(false))))),
                            prototype_self: None,
                            func: SId::from(func_name),
                        });
                    }
                }

                // Only search for a node if there is a designated start, as we don't want to match everything in the graph
                // Ex. self.a.b.c.parent(), where c is a node but not a field
                if let Some(obj) = graph.find_node_named(&pth, start.clone()) {
                    return Ok(CallContext {
                        lib: Some(literal!("Obj")),
                        stack_arg: Some(Arc::new(Base::Literal(Val::Obj(obj)))),
                        prototype_self: None,
                        func: SId::from(func_name),
                    });
                }
            } else if start.is_none() && graph.roots.len() > 0 {
                if let Some(obj) = graph.find_node_named(&pth, graph.main_root()) {
                    return Ok(CallContext {
                        lib: Some(literal!("Obj")),
                        stack_arg: Some(Arc::new(Base::Literal(Val::Obj(obj)))),
                        prototype_self: None,
                        func: SId::from(func_name),
                    });
                }
            }
        }

        Err(Error::FuncDne(path.into()))
    }

    /// Call library function.
    /// This is from exec after we've concluded this is a lib func.
    pub(self) fn call_libfunc(&self, func: LibFunc, stack_arg: Option<Arc<dyn Instruction>>, env: &mut ProcEnv, graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        // Record symbol scope depth for poping later
        let scope_depth = env.table.scopes.len();
        
        // Push call stack, start a new scope, and add self if needed
        let mut instructions = Instructions::default();
        instructions.push(PUSH_SYMBOL_SCOPE.clone());

        let params = func.params;
        let rtype = func.return_type;
        let is_async = func.is_async;
        let unbounded = func.unbounded_args;
        
        // Arguments
        let mut named_args = Vec::new();
        let mut args = Vec::new();
        if let Some(sarg) = stack_arg {
            args.push(sarg);
        }
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
                    if !unbounded {
                        return Err(Error::FuncArgs);
                    } else {
                        args.push(arg.clone());
                    }
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
        if !unbounded && (args.len() != params.len()) {
            return Err(Error::FuncArgs);
        }
        for index in 0..args.len() {
            let arg = &args[index];
            instructions.push(arg.clone());

            if params.len() > 0 && index < params.len() {
                let param = &params[index];
                if !param.param_type.empty() {
                    instructions.push(Arc::new(Base::Cast(param.param_type.clone())));
                }
                if func.args_to_symbol_table {
                    instructions.push(Arc::new(Base::DeclareVar(param.name.to_string().into(), param.param_type.clone()))); // these must keep their type
                }
            }
        }

        // Push the function instructions
        let func_instructions = func.func.deref()(self.as_ref, args.len(), env, graph)?;
        instructions.append(&func_instructions.instructions);
        
        if !is_async {
            if let Some(rtype) = &rtype {
                instructions.push(Arc::new(Base::Cast(rtype.clone())));
            }
        }

        // Cleanup stacks
        instructions.push(Arc::new(Base::PopSymbolScopeUntilDepth(scope_depth)));

        // Handle async function call
        if is_async {
            let mut inner_rtype = Type::Void;
            if let Some(rtype) = rtype {
                inner_rtype = rtype;
            }
            let mut async_instructions = Instructions::default();
            async_instructions.push(Arc::new(Base::Spawn((instructions, inner_rtype)))); // adds a Promise<rtype> to the stack when executed!
            async_instructions.push(SUSPEND.clone()); // make sure to spawn the process right after with the runtime... this is not an await
            Ok(Some(async_instructions))
        } else {
            Ok(Some(instructions))
        }
    }
}


#[derive(Debug)]
pub(self) struct CallContext {
    pub lib: Option<ArcStr>,
    pub prototype_self: Option<Arc<dyn Instruction>>,
    pub func: SId,
    pub stack_arg: Option<Arc<dyn Instruction>>,
}


#[typetag::serde(name = "FuncCall")]
impl Instruction for FuncCall {
    fn exec(&self, env: &mut ProcEnv, graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        let func_context;
        match self.get_func_context(env, graph) {
            Ok(ctx) => func_context = ctx,
            Err(err) => {
                if self.cnull {
                    let mut instructions = Instructions::default();
                    instructions.push(Arc::new(Base::Literal(Val::Null)));
                    return Ok(Some(instructions));
                }
                return Err(err);
            }
        }
        
        // If this is a library function context, then make that call instead
        if let Some(libname) = func_context.lib {
            let name = func_context.func.as_ref();
            if let Some(func) = graph.libfunc(&libname, name) {
                return self.call_libfunc(func, func_context.stack_arg, env, graph);
            }
            if self.cnull {
                let mut instructions = Instructions::default();
                instructions.push(Arc::new(Base::Literal(Val::Null)));
                return Ok(Some(instructions));
            }
            return Err(Error::FuncDne(format!("{libname}.{name}")));
        }

        let func = func_context.func;
        let params;
        let mut func_instructions;
        let rtype;
        let is_async;
        let unself;
        if let Some(func) = graph.get_stof_data::<Func>(&func) {
            params = func.params.clone();
            func_instructions = func.instructions.clone();
            rtype = func.return_type.clone();

            // Only async if we have the attribute and we are not a top level function
            is_async = func.attributes.contains_key(ASYNC_FUNC_ATTR.as_str()) && env.call_stack.len() > 0;

            // Should this function add itself to the self stack?
            unself = func.attributes.contains_key(UNSELF_FUNC_ATTR.as_str());
        } else {
            if self.cnull {
                let mut instructions = Instructions::default();
                instructions.push(Arc::new(Base::Literal(Val::Null)));
                return Ok(Some(instructions));
            }
            return Err(Error::FuncDne(format!("Data Ptr not Func")));
        }

        // Add return tag to the end of the func statements
        func_instructions.push_back(Arc::new(Base::Tag(func.as_ref().into())));

        // Record the current table depth, because we need to pop until we get back here at the end
        let scope_depth = env.table.scopes.len();
       
        // Push call stack, start a new scope, and add self if needed
        let mut instructions = Instructions::default();
        instructions.push(Arc::new(Base::Literal(Val::Fn(func.clone()))));
        instructions.push(DUPLICATE.clone());
        instructions.push(PUSH_CALL.clone());
        instructions.push(PUSH_RETURN.clone());
        instructions.push(PUSH_SYMBOL_SCOPE.clone());

        // Proto self instruction needs to come before args, because of a potential collision in names
        // From bug, where cont.push(cont: Container) caused an issue (test: root.Lang.Control.For.list_of_outputs).
        if let Some(proto_self) = &func_context.prototype_self {
            instructions.push(proto_self.clone());
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
            instructions.push(Arc::new(Base::DeclareVar(param.name.to_string().into(), param.param_type.clone()))); // these must keep their type
        }

        // Add self to self stack if not a prototype function
        let mut pushed_self = false;
        if let Some(_proto_self) = &func_context.prototype_self {
            //instructions.push(proto_self); // happens before the arg instructions
            instructions.push(PUSH_SELF.clone());
            pushed_self = true;
        } else if let Some(oself) = &self.oself {
            instructions.push(oself.clone());
            instructions.push(PUSH_SELF.clone());
            pushed_self = true;
        } else if !unself {
            pushed_self = true;
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

        // Push the function instructions
        if !rtype.empty() {
            instructions.push(PUSH_VAL_RET.clone());
            instructions.append(&func_instructions);
            instructions.push(VALIDATE_FN_RET.clone());
            if !is_async { // await will perform the cast lazily
                instructions.push(Arc::new(Base::Cast(rtype.clone())));
            }
        } else {
            instructions.push(PUSH_VOID_RET.clone());
            instructions.append(&func_instructions);
            instructions.push(VALIDATE_FN_RET.clone());
        }

        // Cleanup stacks
        instructions.push(Arc::new(Base::PopSymbolScopeUntilDepth(scope_depth)));
        instructions.push(POP_CALL.clone());
        instructions.push(POP_RETURN.clone());
        
        // Pop self stack
        if pushed_self {
            instructions.push(POP_SELF.clone());
        }

        // Handle async function call
        if is_async {
            let mut async_instructions = Instructions::default();
            async_instructions.push(Arc::new(Base::Spawn((instructions, rtype)))); // adds a Promise<rtype> to the stack when executed!
            async_instructions.push(SUSPEND.clone()); // make sure to spawn the process right after with the runtime... this is not an await
            Ok(Some(async_instructions))
        } else {
            Ok(Some(instructions))
        }
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
    fn exec(&self, _env: &mut ProcEnv, _graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        let mut instructions = Instructions::default();
        instructions.push(self.ins.clone());
        Ok(Some(instructions))
    }
}
