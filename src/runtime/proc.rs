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

use std::{sync::Arc, time::Duration};
use arcstr::ArcStr;
use colored::Colorize;
use crate::{model::{DataRef, Field, Func, Graph, NodeRef, SId}, parser::source::{self, SrcLoc}, runtime::{instruction::{Instruction, Instructions}, instructions::Base, table::SymbolTable, Error, Val, Variable, WakeRef, Waker}};


#[derive(Debug)]
/// Process Result.
pub enum ProcRes {
    Done,
    More,
    Trace(usize),
    Peek(usize),
    Wait(SId),
    SleepFor(Duration),
    Sleep(WakeRef),
    Exit(Option<SId>),
}


#[derive(Clone, Debug)]
/// Process Env.
pub struct ProcEnv {
    pub pid: SId,
    pub start_time: Option<web_time::Instant>,
    pub max_execution_time: Option<web_time::Duration>,
    pub self_stack: Vec<NodeRef>,
    pub max_call_stack_depth: usize,
    pub call_stack: Vec<DataRef>,
    pub new_stack: Vec<NodeRef>,
    pub max_stack_size: usize,
    pub stack: Vec<Variable>,
    pub table: Box<SymbolTable>,
    pub loop_stack: Vec<ArcStr>,
    pub return_stack: Vec<ArcStr>,
    pub ret_valid_stack: Vec<usize>,
    pub try_stack: Vec<ArcStr>,
    pub yield_enabled: bool,

    // Setting this will put the process into a waiting mode
    pub spawn: Option<Box<Process>>,

    #[cfg(feature = "tokio")]
    pub tokio_runtime: Option<tokio::runtime::Handle>,
}
impl Default for ProcEnv {
    fn default() -> Self {
        Self {
            pid: Default::default(),
            start_time: None,
            max_execution_time: Some(Duration::from_secs(120)),
            self_stack: Default::default(),
            max_call_stack_depth: 10_000,
            call_stack: Default::default(),
            new_stack: Default::default(),
            max_stack_size: 100_000,
            stack: Default::default(),
            table: Default::default(),
            loop_stack: Default::default(),
            return_stack: Default::default(),
            ret_valid_stack: Default::default(),
            try_stack: Default::default(),
            spawn: None,
            yield_enabled: true,

            #[cfg(feature = "tokio")]
            tokio_runtime: None,
        }
    }
}
impl ProcEnv {
    // Get the current self ptr.
    pub fn self_ptr(&self) -> NodeRef {
        self.self_stack.last().unwrap().clone()
    }

    // Trace.
    pub fn trace(&self, graph: &Graph) -> String {
        let mut output = format!("\t{} {}", "PID:".dimmed().italic(), self.pid.to_string().bright_green());

        if let Some(path) = self.self_ptr().node_path(graph, true) {
            output.push_str(&format!("\n\t{} {}", "Self:".dimmed().italic(), path.join(".").bright_cyan()));
        }

        let mut callstack = String::default();
        for index in 0..self.call_stack.len() {
            let func = &self.call_stack[index];
            
            let mut func_path = String::default();
            let nodes = func.data_nodes(graph);
            if nodes.len() > 0 {
                for node in nodes {
                    if let Some(path) = node.node_path(graph, true) {
                        func_path = path.join(".");
                        break;
                    }
                }
            }

            if let Some(this) = graph.get_stof_data::<Func>(func) {
                let mut params = String::default();
                let mut first = true;
                for param in &this.params {
                    if first {
                        first = false;
                        params.push_str(&format!("{}: {}", param.name.as_ref(), param.param_type.rt_type_of(graph)));
                    } else {
                        params.push_str(&format!(", {}: {}", param.name.as_ref(), param.param_type.rt_type_of(graph)));
                    }
                }
                let prefix = format!("{index}.");
                let signature = format!("{} {}.{}({params}) -> {};", prefix.dimmed(), func_path.cyan().dimmed(), func.data_name(graph).unwrap().as_ref().bright_purple(), this.return_type.rt_type_of(graph).as_str().blue());
                callstack.push_str(&format!("\n\t\t{}", signature));
            }
        }
        output.push_str(&format!("\n\t{} {callstack}", "Call-stack:".dimmed().italic()));
        
        output
    }
}


#[derive(Clone, Debug)]
/// A function call in an error stack.
pub struct ErrorFrame {
    pub func: DataRef,
    /// Function path (Ex. "root.config.helper").
    pub path: String,
    /// Statement being executed in this function (debug profiles).
    pub at: Option<SrcLoc>,
    /// Where the function is defined.
    pub def: Option<SrcLoc>,
}
impl ErrorFrame {
    /// " (main.stof:12:5)", " (defined at main.stof:10:1)", or nothing.
    pub fn place(&self) -> String {
        match (&self.at, &self.def) {
            (Some(at), _) => format!(" ({})", at.display()),
            (None, Some(def)) => format!(" (defined at {})", def.display()),
            (None, None) => String::new(),
        }
    }
}


#[derive(Clone, Debug, Default)]
/// Process.
pub struct Process {
    pub env: ProcEnv,
    pub instructions: Instructions,
    pub result: Option<Variable>,
    pub error: Option<Error>,
    pub waiting: Option<SId>,
}
impl From<Instructions> for Process {
    fn from(value: Instructions) -> Self {
        Self {
            instructions: value,
            ..Default::default()
        }
    }
}
impl From<Arc<dyn Instruction>> for Process {
    fn from(value: Arc<dyn Instruction>) -> Self {
        Self {
            instructions: Instructions::from(value),
            ..Default::default()
        }
    }
}
impl Process {
    #[inline(always)]
    /// Progress this process.
    pub(super) fn progress(&mut self, graph: &mut Graph, limit: i32) -> Result<ProcRes, Error> {
        match self.instructions.exec(&mut self.env, graph, limit) {
            Ok(res) => {
                Ok(res)
            },
            Err(error) => {
                Err(error)
            }
        }
    }

    /// Trace.
    pub fn trace(&self, graph: &Graph, n: usize) -> String {
        let mut output = self.env.trace(graph);
        
        if let Some(waiting) = &self.waiting {
            output.push_str(&format!("\n\t{} {}", "Waiting:".dimmed().italic(), waiting.to_string().bright_green()));
        }

        if let Some(error) = &self.error {
            output.push_str(&format!("\n\t{} {}", "Error:".dimmed().italic(), error.to_string().red()));
        }

        if let Some(result) = &self.result {
            output.push_str(&format!("\n\t{} {}", "Result:".dimmed().italic(), result.val.read().print(graph).dimmed()));
        }

        output.push_str(&format!("\n\t{}{}", "Executed Instructions:\n".dimmed().italic(), self.instructions.trace_n(n)));

        output
    }

    /// Stack of the function calls at the point of an error, innermost first.
    /// Statement locations come from Base::Src markers (debug profiles); otherwise frames have the location
    /// of the function definition. Only computed on error (walks the executed history once).
    pub fn error_frames(&self, graph: &Graph) -> Vec<ErrorFrame> {
        // Follow the call depth through the history (mirrors the runtime), keeping the last marker per depth.
        let mut markers: Vec<Option<(u32, u32)>> = vec![None];
        for ins in &self.instructions.executed {
            if let Some(base) = ins.as_dyn_any().downcast_ref::<Base>() {
                match base {
                    Base::PushCall => markers.push(None),
                    Base::PopCall => { if markers.len() > 1 { markers.pop(); } },
                    Base::PopCallUntilDepth(depth) => markers.truncate(depth + 1),
                    Base::Src(line, col) => { if let Some(last) = markers.last_mut() { *last = Some((*line, *col)); } },
                    _ => {}
                }
            }
        }

        let calls = &self.env.call_stack;
        // align innermost frames with the innermost markers (in case early history was trimmed)
        let offset = markers.len() as isize - 1 - calls.len() as isize;
        let mut frames = Vec::new();
        for (index, func_ref) in calls.iter().enumerate().rev() {
            let mut path = String::new();
            for node in func_ref.data_nodes(graph) {
                if let Some(node_path) = node.node_path(graph, true) {
                    path = node_path.join(".");
                    break;
                }
            }
            let name = func_ref.data_name(graph).map(|name| name.as_ref().to_string()).unwrap_or_else(|| "<fn>".into());
            if path.is_empty() { path = name; } else { path = format!("{path}.{name}"); }

            let def = graph.get_stof_data::<Func>(func_ref).and_then(|func| func.src.clone());
            let marker_index = index as isize + 1 + offset;
            let at = if marker_index >= 1 && (marker_index as usize) < markers.len() {
                markers[marker_index as usize].map(|(line, col)| SrcLoc { file: def.as_ref().and_then(|def| def.file.clone()), line, col })
            } else {
                None
            };
            frames.push(ErrorFrame { func: func_ref.clone(), path, at, def });
        }
        frames
    }

    /// Readable error report: message, location with the source line (when the file can be read), and stack.
    /// ```text
    /// error: cannot call 'round' on null (the value is null or missing - check the name or path)
    ///   --> main.stof:6:5
    ///    |
    ///  6 |     const r = self.config.rat.round();
    ///    |     ^
    ///   at root.helper (main.stof:6:5)
    ///   at root.main (main.stof:12:5)
    /// ```
    pub fn error_report(&self, graph: &Graph) -> String {
        let Some(error) = &self.error else { return String::new(); };
        let message = match error {
            Error::Thrown(val) => match val {
                Val::Str(message) => message.to_string(),
                val => format!("error thrown: {}", val.print(graph)),
            },
            error => error.message(),
        };
        let mut out = format!("{} {}", "error:".bold().red(), message.bold());
        if let Some(hint) = self.diagnose(graph) {
            out.push_str(&format!("\n  {} {}", "note:".yellow(), hint));
        }

        let frames = self.error_frames(graph);
        if let Some(loc) = frames.first().and_then(|frame| frame.at.as_ref()) {
            out.push_str(&format!("\n  {} {}", "-->".blue(), loc.display()));
            if let Some(file) = &loc.file {
                if let Ok(text) = std::fs::read_to_string(file.as_str()) {
                    if let Some(line_text) = text.lines().nth(loc.line as usize - 1) {
                        out.push_str(&format!("\n{}", source::code_frame(loc.line as usize, loc.col as usize, line_text).blue()));
                    }
                }
            }
        }
        for frame in &frames {
            out.push_str(&format!("\n  {} {}{}", "at".dimmed(), frame.path.cyan(), frame.place().dimmed()));
        }

        // Internal details for runtime debugging: STOF_TRACE=1
        if std::env::var("STOF_TRACE").map(|v| v == "1").unwrap_or(false) {
            out.push_str(&format!("\n{}", self.trace(graph, 20)));
        }
        out
    }

    /// Extra help for common mistakes: for a function path that doesn't resolve (Ex. a misspelled field in
    /// "self.config.rat.round()"), say which part of the path is missing.
    pub fn diagnose(&self, graph: &Graph) -> Option<String> {
        let Some(Error::FuncDne(path)) = self.error.as_ref().map(|error| error.inner()) else { return None; };
        if path.contains("::") { return None; }
        let segments: Vec<&str> = path.split('.').collect();
        if segments.len() < 3 { return None; }
        let mut node = match segments[0] {
            "self" => self.env.self_stack.last().cloned()?,
            "super" => self.env.self_stack.last()?.node_parent(graph)?,
            root => graph.find_root_named(root)?,
        };
        let mut prefix = segments[0].to_string();
        for segment in &segments[1..segments.len() - 1] {
            let mut next = None;
            if let Some(field) = Field::direct_field(graph, &node, segment) {
                if let Some(field) = graph.get_stof_data::<Field>(&field) {
                    let val = field.value.val.read();
                    match val.try_obj() {
                        Some(obj) => next = Some(obj),
                        None => {
                            let what = if val.empty() { "null".to_string() } else { format!("a {} (not an object)", val.spec_type(graph).type_of()) };
                            return Some(format!("'{prefix}.{segment}' is {what}"));
                        },
                    }
                }
            }
            if next.is_none() {
                if let Some(nref) = node.node(graph) {
                    for child in &nref.children {
                        if child.node_name(graph).map(|name| name.as_ref() == *segment).unwrap_or(false) {
                            next = Some(child.clone());
                            break;
                        }
                    }
                }
            }
            match next {
                Some(child) => node = child,
                None => return Some(format!("'{prefix}' has no field '{segment}'")),
            }
            prefix.push('.');
            prefix.push_str(segment);
        }
        Some(format!("'{prefix}' has no function '{}'", segments[segments.len() - 1]))
    }

    /// Plain text call stack for errors returned to a host (JS, Python, Rust): "  at root.main (main.stof:12:5)" lines.
    pub fn error_stack(&self, graph: &Graph) -> String {
        let mut out = String::new();
        if let Some(hint) = self.diagnose(graph) {
            out.push_str(&format!("  note: {hint}"));
        }
        for frame in self.error_frames(graph) {
            if !out.is_empty() { out.push('\n'); }
            out.push_str(&format!("  at {}{}", frame.path, frame.place()));
        }
        out
    }

    /// Peek.
    pub fn peek(&self, graph: &Graph, n: usize) -> String {
        let mut output = self.env.trace(graph);
        
        if let Some(waiting) = &self.waiting {
            output.push_str(&format!("\n\t{} {}", "Waiting:".dimmed().italic(), waiting.to_string().bright_green()));
        }

        if let Some(error) = &self.error {
            output.push_str(&format!("\n\t{} {}", "Error:".dimmed().italic(), error.to_string().red()));
        }

        if let Some(result) = &self.result {
            output.push_str(&format!("\n\t{} {}", "Result:".dimmed().italic(), result.val.read().print(graph).dimmed()));
        }

        output.push_str(&format!("\n\t{}{}", "Next Instructions:\n".dimmed().italic(), self.instructions.peek_n(n)));

        output
    }

    #[inline]
    /// Create a waker for this process with a wake reference.
    pub(super) fn waker_ref(&self, wref: WakeRef) -> Waker {
        Waker { pid: self.env.pid.clone(), at: None, with: wref }
    }

    #[inline]
    /// Create a waker for this process with a wake time.
    pub(super) fn waker_time(&self, at: Duration) -> Waker {
        Waker { pid: self.env.pid.clone(), at: Some(at), with: Default::default() }
    }
}
