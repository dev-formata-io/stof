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

use std::{path::PathBuf, sync::Arc};
use colored::Colorize;
use imbl::vector;
use lazy_static::lazy_static;
use nanoid::nanoid;
use rustc_hash::{FxHashMap, FxHashSet};
use crate::{model::{DataRef, Graph, NodeRef, SId, PROTOTYPE_EXTENDS_ATTR, PROTOTYPE_TYPE_ATTR}, runtime::{instruction::Instruction, instructions::call::FuncCall, proc::Process, Error, Runtime, Type, Val, Variable}};


lazy_static! {
    static ref PARSE_ID: SId = SId::from("parse");
}


/// Parse context.
pub struct ParseContext<'ctx> {
    pub graph: &'ctx mut Graph,
    pub runtime: Runtime,
    pub docs: bool,
    pub init_funcs: Vec<DataRef>,
    
    relative_import_stack: Vec<PathBuf>,
    seen_import_paths: FxHashMap<NodeRef, FxHashSet<String>>,
}
impl<'ctx> ParseContext<'ctx> {
    /// Create a new parse context with a default config.
    pub fn new(graph: &'ctx mut Graph) -> Self {
        let mut runtime = Runtime::default();
        
        // Stage the process for eval in done
        let mut process = Process::default();
        process.env.pid = PARSE_ID.clone();
        runtime.done.insert(process.env.pid.clone(), process);

        Self {
            graph,
            runtime,
            docs: false,
            init_funcs: Default::default(),
            relative_import_stack: Default::default(),
            seen_import_paths: Default::default(),
        }
    }

    /// Parse from a file path into a node or self.
    pub fn parse_from_file(&mut self, format: &str, path: &str, node: Option<NodeRef>) -> Result<(), Error> {
        let node = node.unwrap_or(self.self_ptr());
        let mut path = path.to_string();

        path = self.create_import_path(format, &path).expect("could not create Stof import path");
        if !self.fresh_import_for_node(&node, &path) {
            self.pop_relative_import_stack();
            return Ok(()); // already parsed this path
        }

        self.push_self_node(node);
        if let Some(format_impl) = self.graph.get_format(format) {
            match format_impl.parser_import(format, &path, self) {
                Ok(_) => {},
                Err(mut error) => {
                    self.pop_relative_import_stack();
                    self.pop_self();

                    match &mut error {
                        Error::ParseError(error) => {
                            error.file_path = Some(path);
                        },
                        _ => {}
                    }
                    return Err(error);
                }
            }
        }
        self.pop_self();
        self.pop_relative_import_stack();
        
        Ok(())
    }

    /// Create an import path.
    /// Takes a possibly relative import path and returns a full path.
    fn create_import_path(&mut self, format: &str, path: &str) -> Result<String, Error> {
        if self.relative_import_stack.is_empty() {
            if let Ok(working) = std::env::current_dir() {
                self.relative_import_stack.push(working);
            }
        }

        let mut path = path.replace("@", "stof/").replace(" ", "");
        if path.starts_with(".") {
            if self.relative_import_stack.is_empty() {
                return Err(Error::RelativeImportWithoutContext);
            }

            let mut prefix = self.relative_import_stack.last().unwrap().as_path();
            while path.starts_with("../") && !prefix.parent().is_some() {
                prefix = prefix.parent().unwrap();
                path = path.strip_prefix("../").unwrap().to_string();
            }
            path = path.trim_start_matches("./").to_string();
            path = path.replace("./", "/");

            let prefix_path = prefix.as_os_str().to_os_string().into_string();
            if prefix_path.is_err() { return Err(Error::ImportOsStringError); }

            path = format!("{}/{}", prefix_path.unwrap(), path.trim_start_matches("/").trim_end_matches("/"));
        }

        if format == "stof" && !path.ends_with(".stof") {
            path.push_str(".stof");
        }

        let mut relative_buffer = PathBuf::from(&path);
        relative_buffer.pop();
        self.relative_import_stack.push(relative_buffer);

        Ok(path)
    }

    /// Push relative import stack.
    pub fn push_relative_import_stack_file(&mut self, file_path: &str) {
        let mut relative_buffer = PathBuf::from(file_path);
        relative_buffer.pop();
        self.relative_import_stack.push(relative_buffer);
    }

    /// Pop relative import stack.
    pub fn pop_relative_import_stack(&mut self) {
        self.relative_import_stack.pop();
    }

    /// Chech to see that the import path hasn't been seen before.
    /// If it hasnt, add it and return true.
    fn fresh_import_for_node(&mut self, node: &NodeRef, path: &str) -> bool {
        if let Some(seen) = self.seen_import_paths.get_mut(node) {
            if seen.contains(path) {
                return false;
            }
            seen.insert(path.to_string());
        } else {
            let mut set = FxHashSet::default();
            set.insert(path.to_string());
            self.seen_import_paths.insert(node.clone(), set);
        }
        true
    }

    /// Get the current parse process.
    pub fn parse_proc<'a>(&'a mut self) -> &'a mut Process {
        self.runtime.done.get_mut(&PARSE_ID).unwrap()
    }

    /// Get the current self pointer.
    pub fn self_ptr(&mut self) -> NodeRef {
        let proc = self.parse_proc();
        if proc.env.self_stack.len() > 0 {
            proc.env.self_ptr()
        } else {
            self.graph.ensure_main_root()
        }
    }

    /// Push a new root node to the self stack.
    pub fn push_root(&mut self, name: Option<&str>) {
        let mut obj_name = nanoid!(12);
        if let Some(name) = name {
            obj_name = name.to_string();
        }
        let nref = self.graph.insert_root(&obj_name);
        let proc = self.parse_proc();
        proc.env.self_stack.push(nref);
    }

    /// Post push object (cast to an extends type here).
    pub fn post_init_obj(&mut self, value: &Variable, attributes: &mut FxHashMap<String, Val>) -> Result<(), Error> {
        if let Some(extends_attr) = attributes.get(PROTOTYPE_EXTENDS_ATTR.as_str()) {
            if let Some(obj) = value.try_obj() {
                let context = self.self_ptr();
                match extends_attr {
                    Val::Str(typename) => {
                        let cast_type = Type::Obj(typename.as_str().into());
                        Val::Obj(obj).cast(&cast_type, &mut self.graph, Some(context))?;
                    },
                    Val::Obj(proto) => {
                        let cast_type = Type::Obj(proto.clone());
                        Val::Obj(obj).cast(&cast_type, &mut self.graph, Some(context))?;
                    },
                    _ => {}
                }
            }
        }
        Ok(())
    }

    /// Push self stack as a variable.
    pub fn push_self(&mut self, name: &str, field: bool, attributes: &mut FxHashMap<String, Val>) -> Variable {
        let parent = self.self_ptr();
        
        // TODO: collisions?

        let nref = self.graph.insert_node(name, Some(parent), field);
        if let Some(node) = nref.node_mut(&mut self.graph) {
            node.attributes = attributes.clone(); // set node attributes as the same as field attrs
        }

        // Is this object a type? If so, put it in the typemap for quick lookup.
        if let Some(type_attr) = attributes.get(PROTOTYPE_TYPE_ATTR.as_str()) {
            match type_attr {
                Val::Str(name) => {
                    // Overridden type name
                    self.graph.insert_type(name.as_str(), &nref);
                },
                _ => {
                    // Use the object name
                    self.graph.insert_type(name, &nref);
                }
            }
        }

        let proc = self.parse_proc();
        proc.env.self_stack.push(nref.clone());
        Variable::val(Val::Obj(nref))
    }

    /// Push self node.
    pub fn push_self_node(&mut self, node: NodeRef) {
        let proc = self.parse_proc();
        proc.env.self_stack.push(node);
    }

    /// Pop self stack.
    pub fn pop_self(&mut self) {
        let proc = self.parse_proc();
        if proc.env.self_stack.len() > 1 {
            proc.env.self_stack.pop();
        }
    }

    /// Reset the process when things go badly.
    fn reset_proc(&mut self) {
        self.runtime.clear();

        let mut process = Process::default();
        process.env.pid = PARSE_ID.clone();
        self.runtime.done.insert(process.env.pid.clone(), process);
    }

    /// Use this to quickly evaluate one instruction in the parse process.
    /// Must have a process in done.
    pub fn eval(&mut self, instruction: Arc<dyn Instruction>) -> Result<Val, Error> {
        // get the process and clear it (preserving memory allocations)
        let mut proc = self.runtime.done.remove(&PARSE_ID).unwrap();
        //proc.env.self_stack.clear(); // use this stack as the parse self stack, so dont clear!
        proc.env.call_stack.clear();
        proc.env.new_stack.clear();
        proc.env.stack.clear();
        proc.env.table.clear();
        proc.instructions.clear();
        proc.result = None;
        proc.error = None;
        proc.waiting = None;

        // load the instruction and push to running
        proc.instructions.push(instruction);
        self.runtime.push_running_proc(proc, &mut self.graph); // makes sure there is a self stack

        // run to end and grab the result
        self.runtime.run_to_complete(&mut self.graph);

        if let Some(proc) = self.runtime.done.get_mut(&PARSE_ID) {
            if let Some(res) = proc.result.take() {
                Ok(res.get())
            } else {
                Ok(Val::Void)
            }
        } else if let Some(mut proc) = self.runtime.errored.remove(&PARSE_ID) {
            let res;
            if let Some(err) = proc.error.take() {
                res = Err(err);
            } else {
                res = Err(Error::NotImplemented);
            }

            // Move proc back to done for next time
            self.runtime.done.insert(proc.env.pid.clone(), proc);
            res
        } else {
            self.reset_proc();
            Err(Error::NotImplemented)
        }
    }
}

impl<'ctx> Drop for ParseContext<'ctx> {
    fn drop(&mut self) {
        // If we parsed docs, instruct the graph to insert library documentation
        if self.docs {
            self.graph.insert_lib_docs();
        }

        // Call all init functions that were parsed
        if self.init_funcs.len() > 0 {
            for init in self.init_funcs.clone() {
                let ins: Arc<dyn Instruction> = Arc::new(FuncCall {
                    as_ref: false,
                    stack: false,
                    func: Some(init),
                    search: None,
                    args: vector![],
                    oself: None,
                });
                self.runtime.push_running_proc(Process::from(ins), &mut self.graph);
            }
            self.runtime.err_callback = Some(Box::new(|graph, errored| {
                if errored.env.call_stack.len() > 0 {
                    let func_ref = errored.env.call_stack.first().unwrap();
                    if let Some(name) = func_ref.data_name(graph) {
                        let mut func_path = String::from("<unknown>");
                        for node in func_ref.data_nodes(graph) { func_path = node.node_path(graph, true).unwrap().join("."); }
                        
                        let mut err_str = String::from("<unknown>");
                        if let Some(err) = &errored.error {
                            err_str = err.to_string();
                        }

                        println!("{} {} {} {} {}\n\t{}\n", "init".purple(), func_path.italic().dimmed(), name.as_ref().italic().blue(), "...".dimmed(), "failed".bold().red(), err_str.bold().bright_cyan());
                    }
                }
                true
            }));
            self.runtime.run_to_complete(&mut self.graph);
        }
    }
}
