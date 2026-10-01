//
// Copyright 2025 Formata, Inc. All rights reserved.
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

use std::{path::{Component, Path, PathBuf}, sync::Arc};
use colored::Colorize;
use imbl::vector;
use lazy_static::lazy_static;
use nanoid::nanoid;
use rustc_hash::{FxHashMap, FxHashSet};
use arcstr::literal;
use crate::{parser::source::{self, SrcLoc}, model::{DataRef, Graph, NodeRef, PROTOTYPE_EXTENDS_ATTR, PROTOTYPE_TYPE_ATTR, Profile, SId, libraries::prof::insert_profile_lib}, runtime::{Error, Runtime, Type, Val, Variable, instruction::Instruction, instructions::call::FuncCall, proc::Process}};


lazy_static! {
    static ref PARSE_ID: SId = SId::from("parse");
}


/// Parse context.
pub struct ParseContext<'ctx> {
    pub graph: &'ctx mut Graph,
    pub runtime: Runtime,
    pub profile: Profile,
    pub init_funcs: Vec<DataRef>,
    
    relative_import_stack: Vec<PathBuf>,
    seen_import_paths: FxHashMap<NodeRef, FxHashSet<String>>,

    /// Files being parsed (innermost last), for warning locations.
    file_stack: Vec<String>,

    /// Parse warnings, printed (stderr) when the context is dropped.
    pub warnings: Vec<String>,

    /// Unknown names found in functions, checked against roots when the document is done:
    /// (function, name, is call, location).
    pending_names: Vec<(String, String, bool, Option<SrcLoc>)>,

}
impl<'ctx> ParseContext<'ctx> {
    /// Create a new parse context with a default config.
    pub fn new(graph: &'ctx mut Graph, profile: Profile) -> Self {
        let mut runtime = Runtime::default();
        
        // Stage the process for eval in done
        let mut process = Process::default();
        process.env.pid = PARSE_ID.clone();

        // Parse into the main root by default. The self stack never pops its last node, so without this
        // base, the first object pushed would never be popped and everything after it would nest inside it.
        process.env.self_stack.push(graph.ensure_main_root());
        runtime.done.insert(process.env.pid.clone(), process);

        // Insert the updated profile lib into the graph with this context (assume we use the context)
        insert_profile_lib(graph, &profile);

        Self {
            graph,
            runtime,
            profile,
            init_funcs: Default::default(),
            relative_import_stack: Default::default(),
            seen_import_paths: Default::default(),
            file_stack: Default::default(),
            warnings: Default::default(),
            pending_names: Default::default(),
        }
    }

    /// Parse from a file path into a node or self.
    pub fn parse_from_file(&mut self, format: &str, path: &str, node: Option<NodeRef>) -> Result<(), Error> {
        let Some(format_impl) = self.graph.get_format(format) else {
            return Err(Error::Custom(format!("unknown import format '{format}' for '{path}'").into()));
        };
        let node = node.unwrap_or(self.self_ptr());
        let path = self.create_import_path(format, path)?;
        if !self.fresh_import_for_node(&node, &path, format) {
            self.pop_relative_import_stack();
            return Ok(()); // already parsed this path
        }

        self.push_self_node(node);
        self.file_stack.push(path.clone());
        let res = format_impl.parser_import(format, &path, self);
        self.file_stack.pop();
        self.pop_self();
        self.pop_relative_import_stack();

        if let Err(mut error) = res {
            if let Error::ParseError(error) = &mut error {
                error.set_file_if_unknown(path); // keep the innermost file
            }
            return Err(error);
        }
        Ok(())
    }

    /// Import a file with an explicit format, or the format implied by its extension.
    /// - Explicit: the format must exist (Ex. import json './data').
    /// - Implied: no extension is stof; a known extension is that format; any other file is imported as
    ///   text, or as bytes when it isn't text.
    pub fn import_file(&mut self, format: Option<&str>, path: &str, node: Option<NodeRef>) -> Result<(), Error> {
        if let Some(format) = format {
            return self.parse_from_file(format, path, node);
        }
        let extension = Path::new(path.trim()).extension().and_then(|ext| ext.to_str()).map(|ext| ext.to_string());
        match extension {
            None => self.parse_from_file("stof", path, node),
            Some(ext) if self.graph.get_format(&ext).is_some() => self.parse_from_file(&ext, path, node),
            Some(_) => match self.parse_from_file("text", path, node.clone()) {
                Ok(()) => Ok(()),
                Err(text_error) => self.parse_from_file("bytes", path, node).map_err(|_| text_error),
            },
        }
    }

    /// File being parsed (innermost import), if any.
    pub fn current_file(&self) -> Option<&str> {
        self.file_stack.last().map(|file| file.as_str())
    }

    /// Add a parse warning (deduplicated), tagged with the file being parsed.
    pub fn warn(&mut self, message: impl Into<String>) {
        let mut message = message.into();
        if let Some(file) = self.file_stack.last() {
            message = format!("{message}\n\t{} {file}", "in".dimmed());
        }
        if !self.warnings.contains(&message) {
            self.warnings.push(message);
        }
    }

    /// Take all parse warnings so far (printed on drop otherwise).
    pub fn take_warnings(&mut self) -> Vec<String> {
        std::mem::take(&mut self.warnings)
    }

    /// Is this name a library (Ex. "Num", "Str", "Std")?
    pub fn is_lib_name(&self, name: &str) -> bool {
        self.graph.libfuncs.contains_key(name)
    }

    /// Warn if this node shadows a library in "Lib.func()" calls.
    /// "Lib.func()" looks for a graph root named Lib before the library.
    pub fn warn_if_shadows_lib(&mut self, node: &NodeRef) {
        if node.node_parent(&self.graph).is_some() { return; }
        if let Some(name) = node.node_name(&self.graph) {
            let name = name.as_ref().to_string();
            if self.is_lib_name(&name) {
                let path = node.node_path(&self.graph, true).map(|path| path.join(".")).unwrap_or(name.clone());
                self.warn(format!("root '{path}' shadows the {name} library: {name}.func(..) calls look for functions on this root first (use {name}::func(..) to always call the library)"));
            }
        }
    }

    /// Warn for variables (params & locals) that shadow a library in "Lib.func()" calls.
    pub fn warn_if_vars_shadow_lib(&mut self, func: &str, names: &[String]) {
        for name in names {
            if self.is_lib_name(name) {
                self.warn(format!("variable '{name}' in fn {func} shadows the {name} library: {name}.func(..) in this function calls through the variable (use {name}::func(..) to always call the library)"));
            }
        }
    }

    /// Check the bare names a function uses against its variables (params, locals, loop/catch variables).
    /// A bare name can only be a variable, a graph root, or (when called) a standard library function, so
    /// anything else is a mistake that silently reads null at runtime (Ex. a misspelled variable).
    pub fn check_names(&mut self, func: &str, vars: &[String], referenced: Vec<(String, bool, usize)>) {
        let mut seen = FxHashSet::default();
        let func_path = match self.self_ptr().node_path(&self.graph, true) {
            Some(path) => format!("{}.{func}", path.join(".")),
            None => func.to_string(),
        };
        for (name, call, addr) in referenced {
            if vars.iter().any(|var| var == &name) || !seen.insert(name.clone()) { continue; }
            if call {
                if self.graph.libfunc(&literal!("Std"), &name).is_some() { continue; }
            } else if self.is_lib_name(&name) {
                continue; // Ex. Num.abs(x)
            }
            self.pending_names.push((func_path.clone(), name, call, source::locate_addr(addr)));
        }
    }

    /// Warn for unknown names once the document is parsed (roots can be declared after the function).
    pub(crate) fn finish_name_checks(&mut self) {
        let assigned = crate::parser::statement::declare::take_assigned_names();
        for (func, name, call, loc) in std::mem::take(&mut self.pending_names) {
            if !call && (self.graph.find_root_named(name.as_str()).is_some() || assigned.contains(&name)) { continue; }
            let place = loc.map(|loc| format!(" ({})", loc.display())).unwrap_or_default();
            let message = if call {
                format!("unknown function '{name}' in fn {func}{place}: not a variable or standard library function (use self.{name}() for a function on this object)")
            } else {
                format!("unknown name '{name}' in fn {func}{place}: not a parameter, variable, or root, so it is always null (use self.{name} for a field)")
            };
            if !self.warnings.contains(&message) { self.warnings.push(message); }
        }
    }

    /// Create an import path.
    /// Takes a possibly relative import path and returns a full path.
    /// Pushes the new path's directory onto the relative import stack (the caller pops it).
    fn create_import_path(&mut self, format: &str, path: &str) -> Result<String, Error> {
        if self.relative_import_stack.is_empty() {
            if let Ok(working) = std::env::current_dir() {
                self.relative_import_stack.push(working);
            }
        }
        let resolved = resolve_import_path(self.relative_import_stack.last().map(|dir| dir.as_path()), format, path)?;

        let mut relative_buffer = resolved.clone();
        relative_buffer.pop();
        self.relative_import_stack.push(relative_buffer);

        resolved.into_os_string().into_string().map_err(|_| Error::ImportOsStringError)
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

    /// Chech to see that the import path hasn't been seen before (with the given format).
    /// If it hasnt, add it and return true.
    fn fresh_import_for_node(&mut self, node: &NodeRef, path: &str, format: &str) -> bool {
        let cmp = format!("{format}{path}"); // combine format and path
        if let Some(seen) = self.seen_import_paths.get_mut(node) {
            if seen.contains(&cmp) {
                return false;
            }
            seen.insert(cmp);
        } else {
            let mut set = FxHashSet::default();
            set.insert(cmp);
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
    pub fn push_root(&mut self, name: Option<String>, cid: Option<SId>) {
        let mut obj_name = nanoid!(12);
        if let Some(name) = name {
            obj_name = name;
        }
        let nref;
        if let Some(id) = cid {
            if id.node_exists(&self.graph) {
                nref = self.graph.insert_root(&obj_name); // no collisions
            } else {
                nref = self.graph.insert_node_id(&obj_name, id, None, false);
            }
        } else {
            nref = self.graph.insert_root(&obj_name);
        }
        self.warn_if_shadows_lib(&nref);
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
    pub fn push_self(&mut self, name: &str, attributes: &mut FxHashMap<String, Val>, id: Option<SId>) -> Variable {
        let parent = self.self_ptr();

        // Insert the new node, not as a field (we're overridding attributes anyways)
        let nref;
        if let Some(cid) = id {
            if cid.node_exists(&self.graph) {
                nref = self.graph.insert_node(name, Some(parent), false); // no collisions
            } else {
                nref = self.graph.insert_node_id(name, cid, Some(parent), false);
            }
        } else {
            nref = self.graph.insert_node(name, Some(parent), false);
        }
        if let Some(node) = nref.node_mut(&mut self.graph) {
            node.attributes = attributes.clone(); // set node attributes as the same as field attrs
        }
        self.warn_if_shadows_lib(&nref);

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
        process.env.self_stack.push(self.graph.ensure_main_root());
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

/// Resolve an import path (pure: no file system access, so it works the same everywhere, including wasm).
///
/// - Relative paths ("./x", "../x", "lib/x") are relative to `dir`, the directory of the importing file
///   (the working directory for the entry file), like includes and module paths in other languages.
/// - A leading "@" is a package: "@pkg/x" is "stof/pkg/x" in the working directory (the project root),
///   whichever file imports it. "@" starting a later segment is the same shorthand relative to the file
///   (Ex. "./@geo" is "./stof/geo"); an "@" inside a name (Ex. "user@host") is left alone.
/// - Rooted paths ("/x", "C:\\x") are used as given.
/// - "." and ".." segments are normalized lexically, so the same file always has the same path
///   (import dedupe compares paths).
/// - Stof imports without a ".stof" or ".json" extension get ".stof".
pub fn resolve_import_path(dir: Option<&Path>, format: &str, path: &str) -> Result<PathBuf, Error> {
    let path = path.trim();
    let package = path.starts_with('@');
    let mut expanded = String::with_capacity(path.len() + 8);
    let mut segment_start = true;
    for c in path.chars() {
        if c == '@' && segment_start {
            expanded.push_str("stof/");
        } else {
            expanded.push(c);
        }
        segment_start = c == '/' || c == '\\';
    }
    let path = expanded;

    let mut full = PathBuf::from(&path);
    if !package && !full.has_root() {
        match dir {
            Some(dir) => full = dir.join(&path),
            None if path.starts_with('.') => return Err(Error::RelativeImportWithoutContext),
            None => {}, // no directory known (Ex. wasm): used as given
        }
    }
    let mut full = normalize_path(&full);

    if format == "stof" {
        let ext = full.extension().and_then(|ext| ext.to_str()).map(|ext| ext.to_ascii_lowercase());
        if ext.as_deref() != Some("stof") && ext.as_deref() != Some("json") {
            let mut name = full.file_name().map(|name| name.to_os_string()).unwrap_or_default();
            name.push(".stof");
            full.set_file_name(name);
        }
    }
    Ok(full)
}

/// Lexically normalize a path: drop "." segments and fold "dir/.." pairs.
/// Leading ".." segments of a relative path are kept; ".." at a root stays at the root.
fn normalize_path(path: &Path) -> PathBuf {
    let mut normal: Vec<Component> = Vec::new();
    for component in path.components() {
        match component {
            Component::CurDir => {},
            Component::ParentDir => match normal.last() {
                Some(Component::Normal(_)) => { normal.pop(); },
                Some(Component::RootDir) | Some(Component::Prefix(_)) => {},
                _ => normal.push(component),
            },
            _ => normal.push(component),
        }
    }
    normal.iter().collect()
}


impl<'ctx> Drop for ParseContext<'ctx> {
    fn drop(&mut self) {
        // Parse warnings (stderr; a no-op in wasm)
        for warning in self.take_warnings() {
            eprintln!("{} {}", "warning:".bold().yellow(), warning);
        }

        // If we parsed docs, instruct the graph to insert library documentation
        if self.profile.docs {
            self.graph.insert_lib_docs();
        }

        // Call all init functions that were parsed
        if self.init_funcs.len() > 0 {
            for init in self.init_funcs.clone() {
                let ins: Arc<dyn Instruction> = Arc::new(FuncCall {
                    as_ref: false,
                    cnull: false,
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
                        for node in func_ref.data_nodes(graph) { func_path = node.node_path(graph, true).map(|path| path.join(".")).unwrap_or_default(); }
                        let err_str = errored.error_report(graph);
                        println!("{} {} {} {} {}\n{}\n", "init".purple(), func_path.italic().dimmed(), name.as_ref().italic().blue(), "...".dimmed(), "failed".bold().red(), err_str);
                    }
                }
                true
            }));
            self.runtime.run_to_complete(&mut self.graph);
        }
    }
}


#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use crate::{model::{Graph, Profile}, parser::context::resolve_import_path, runtime::Error};

    fn resolve(dir: Option<&str>, path: &str) -> PathBuf {
        resolve_import_path(dir.map(Path::new), "stof", path).expect("resolves")
    }

    #[test]
    fn relative_to_importing_file() {
        assert_eq!(resolve(Some("/proj/src"), "./mod"), PathBuf::from("/proj/src/mod.stof"));
        assert_eq!(resolve(Some("/proj/src"), "./a/b.stof"), PathBuf::from("/proj/src/a/b.stof"));
        assert_eq!(resolve(Some("/proj/src"), "../lib/x"), PathBuf::from("/proj/lib/x.stof"));
        assert_eq!(resolve(Some("/proj/src"), "../../x"), PathBuf::from("/x.stof"));
        assert_eq!(resolve(Some("/proj/src"), "./a/../b"), PathBuf::from("/proj/src/b.stof"));
        assert_eq!(resolve(Some("/proj/src"), "././a/./b"), PathBuf::from("/proj/src/a/b.stof"));
        assert_eq!(resolve(Some("/"), "../x"), PathBuf::from("/x.stof")); // can't go above the root
    }

    #[test]
    /// The entry file given by bare name ("stof run main.stof") has an empty directory: its relative
    /// imports stay relative to the working directory (they used to become "/x").
    fn entry_file_in_working_directory() {
        assert_eq!(resolve(Some(""), "./x"), PathBuf::from("x.stof"));
        assert_eq!(resolve(Some(""), "./sub/x"), PathBuf::from("sub/x.stof"));
        assert_eq!(resolve(Some(""), "../x"), PathBuf::from("../x.stof"));
        assert_eq!(resolve(Some("sub"), "../x"), PathBuf::from("x.stof"));
        assert_eq!(resolve(Some("sub"), "../../x"), PathBuf::from("../x.stof"));
    }

    #[test]
    fn other_paths_are_used_as_given() {
        assert_eq!(resolve(Some("/proj/src"), "/abs/x"), PathBuf::from("/abs/x.stof"));
        assert_eq!(resolve(None, "/abs/x.stof"), PathBuf::from("/abs/x.stof"));
        assert_eq!(resolve(None, "lib/x"), PathBuf::from("lib/x.stof")); // no directory known: as given
    }

    #[test]
    /// Bare relative paths are relative to the importing file, like "./" (not the working directory).
    fn bare_paths_are_relative_to_the_importing_file() {
        assert_eq!(resolve(Some("/proj/src"), "lib/x"), PathBuf::from("/proj/src/lib/x.stof"));
        assert_eq!(resolve(Some("/proj/src"), "lib/x"), resolve(Some("/proj/src"), "./lib/x"));
        assert_eq!(resolve(Some(""), "src/mod"), PathBuf::from("src/mod.stof")); // entry file in the working dir
    }

    #[test]
    fn packages_spaces_and_at_signs() {
        assert_eq!(resolve(Some("/p"), "@limitr/types"), PathBuf::from("stof/limitr/types.stof")); // project packages
        assert_eq!(resolve(Some("/p/deep/dir"), "@limitr/types"), PathBuf::from("stof/limitr/types.stof"));
        assert_eq!(resolve(Some("/p"), "./@geo"), PathBuf::from("/p/stof/geo.stof")); // any segment
        assert_eq!(resolve(Some("/p"), "./a/@geo/x"), PathBuf::from("/p/a/stof/geo/x.stof"));
        assert_eq!(resolve(Some("/p"), "./user@host"), PathBuf::from("/p/user@host.stof")); // not inside a name
        assert_eq!(resolve(Some("/p"), "./My Docs/a b"), PathBuf::from("/p/My Docs/a b.stof")); // spaces kept
        assert_eq!(resolve(Some("/p"), "  ./x  "), PathBuf::from("/p/x.stof")); // surrounding whitespace trimmed
    }

    #[test]
    fn extensions() {
        assert_eq!(resolve(Some("/p"), "./x.stof"), PathBuf::from("/p/x.stof"));
        assert_eq!(resolve(Some("/p"), "./x.json"), PathBuf::from("/p/x.json"));
        assert_eq!(resolve(Some("/p"), "./x.STOF"), PathBuf::from("/p/x.STOF"));
        assert_eq!(resolve(Some("/p"), "./config.v2"), PathBuf::from("/p/config.v2.stof"));
        assert_eq!(resolve(Some("/p"), "./v1.2/mod"), PathBuf::from("/p/v1.2/mod.stof"));
        assert_eq!(resolve_import_path(Some(Path::new("/p")), "json", "./data").unwrap(), PathBuf::from("/p/data")); // only stof adds one
    }

    #[test]
    fn relative_without_a_directory_errors() {
        assert!(matches!(resolve_import_path(None, "stof", "./x"), Err(Error::RelativeImportWithoutContext)));
    }

    #[test]
    /// Real files: nested relative imports, "..", a folder with a space, an "@" in a file name, and the same
    /// file imported twice through different spellings (imported once).
    fn relative_imports_on_disk() {
        let root = std::env::temp_dir().join(format!("stof_imports_{}", nanoid::nanoid!(8)));
        let write = |rel: &str, src: &str| {
            let path = root.join(rel);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, src).unwrap();
        };
        write("main.stof", r#"
            import './sub/a' as self.A;
            import './sub/../sub/a.stof' as self.A;   // same file, different spelling: skipped
            import './with space/b' as self.B;
            import './user@host' as self.U;
        "#);
        write("sub/a.stof", "import '../c' as self.C;\nimport 'deeper/d' as self.D;\nfrom_a: { x: 1 }");
        write("sub/deeper/d.stof", "from_d: 5");
        write("c.stof", "from_c: 3");
        write("with space/b.stof", "from_b: 2");
        write("user@host.stof", "from_u: 4");

        let mut graph = Graph::default();
        let entry = root.join("main.stof");
        graph.parse_stof_file("stof", entry.to_str().unwrap(), None, Profile::default()).expect("imports resolve");
        let _ = std::fs::remove_dir_all(&root);

        let main = graph.main_root().unwrap();
        let node = |path: &str| graph.find_node_named(path, Some(main.clone()));
        assert!(node("A.from_a").is_some());
        assert!(node("A.C").is_some());
        assert!(node("A.D").is_some()); // bare path: relative to sub/a.stof
        assert!(node("B").is_some());
        assert!(node("U").is_some());
        let a = node("A").unwrap();
        let from_a_count = a.node(&graph).unwrap().children.iter()
            .filter(|child| child.node_name(&graph).map(|name| name.as_ref() == "from_a").unwrap_or(false))
            .count();
        assert_eq!(from_a_count, 1, "a.stof was imported twice");
    }

    #[test]
    /// Explicit formats must exist; implied formats come from the file's own extension, with text (or bytes
    /// when the file isn't text) for unknown extensions.
    fn import_formats() {
        let root = std::env::temp_dir().join(format!("stof_formats_{}", nanoid::nanoid!(8)));
        std::fs::create_dir_all(root.join("v1.2")).unwrap();
        std::fs::write(root.join("v1.2/mod.stof"), "from_dotted_dir: 1").unwrap();
        std::fs::write(root.join("notes.log"), "hello log").unwrap();
        std::fs::write(root.join("data.bin"), [0u8, 159, 146, 150, 255]).unwrap();
        std::fs::write(root.join("main.stof"), r#"
            import './v1.2/mod' as self.Dotted;      // "." in a folder name isn't an extension: stof
            import './notes.log' as self.Log;       // unknown extension, text
            import './data.bin' as self.Bin;        // unknown extension, not text: bytes
        "#).unwrap();
        std::fs::write(root.join("bad.stof"), "import notaformat './x' as self.X;").unwrap();

        let mut graph = Graph::default();
        graph.parse_stof_file("stof", root.join("main.stof").to_str().unwrap(), None, Profile::default()).expect("imports");
        let bad = Graph::default().parse_stof_file("stof", root.join("bad.stof").to_str().unwrap(), None, Profile::default());
        let _ = std::fs::remove_dir_all(&root);

        let main = graph.main_root().unwrap();
        let field = |graph: &mut Graph, path: &str| crate::model::Field::field_from_path(graph, path, Some(main.clone()))
            .and_then(|dref| graph.get_stof_data::<crate::model::Field>(&dref).map(|field| field.value.get()));
        assert!(field(&mut graph, "Dotted.from_dotted_dir").is_some());
        assert_eq!(field(&mut graph, "Log.text"), Some(crate::runtime::Val::Str("hello log".into())));
        assert!(matches!(field(&mut graph, "Bin.bytes"), Some(crate::runtime::Val::Blob(_))));

        let error = bad.expect_err("unknown explicit format").to_string();
        assert!(error.contains("notaformat"), "{error}");
    }
}
