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

//! Std.load: run the #[load] functions in an object (and the objects in it), or in the whole document.
//!
//! A #[load] function brings in what a document depends on, usually with `import`, which skips what's already there.
//! So load is safe to run any number of times, and a host runs it after parsing (`doc.load()`). Each import that
//! loads something runs load on what it loaded, so dependencies of dependencies load too.

use std::sync::Arc;
use arcstr::{literal, ArcStr};
use imbl::{vector, Vector};
use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};
use crate::{model::{stof_std::STD_LIB, DataRef, Func, Graph, LibFunc, NodeRef, Param}, runtime::{instruction::{Instruction, Instructions}, instructions::{call::FuncCall, trycatch::TryCatchIns, Base, AWAIT, SUSPEND}, proc::ProcEnv, Error, Num, Type, Val, Variable}};


/// The attribute that marks a function to run on load.
pub const LOAD_ATTR: ArcStr = literal!("load");


/// Std.load(obj) -> Promise<int>
pub fn std_load() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "load".into(),
        is_async: true,
        docs: r#"# async Std::load(obj: obj = null) -> Promise<int>
Run the `#[load]` functions in this object and the objects in it, or in the whole document, and wait for them (and everything they start) to finish. Returns how many it ran. A host runs it after parsing (`doc.load()`), and the CLI before `stof run` and `stof test`.

A `#[load]` function brings in what the document depends on, usually with `import`, which skips what's already there without fetching. So `load` is safe to run any number of times. Each `import` that loads something runs `load` on what it loaded, so dependencies of dependencies load too.

- The functions run at the same time, and a function that's already running (Ex. one that calls `load` on its own object) isn't started again.
- Other work in a `#[load]` function runs again each time.
- Await what it starts (Ex. `await import(..)`): an error in something it didn't await doesn't fail `load`.
- A sealed part's functions are run only by code in that part.
```rust
#[load]
fn deps() {
    await import('https://example.com/intent.bstf', '9f2c...', 'root.deps.intent');
}

#[main]
fn main() {
    await load(); // the CLI does this for you
}
```
"#.into(),
        params: crate::params![
            Param { name: "obj".into(), param_type: Type::Void, default: Some(Arc::new(Base::Literal(Val::Null))) },
        ],
        return_type: Some(Type::Num(crate::runtime::NumT::Int)),
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(LoadIns::Start));
            Ok(instructions)
        })
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Load instructions (their own type, so BSTF's variant indexes for the other libraries never shift).
pub enum LoadIns {
    /// (obj) -> runs the #[load] functions, then Done.
    Start,
    /// -> runs these #[load] functions (the ones an import added), then Done.
    Run { funcs: Vector<DataRef> },
    /// -> the count, after the functions finished (they're no longer running).
    Done { funcs: Vector<DataRef> },
    /// (error message, error value) -> the functions are no longer running, and the error goes on.
    Failed { funcs: Vector<DataRef> },
}
#[typetag::serde(name = "LoadIns")]
impl Instruction for LoadIns {
    fn exec(&self, env: &mut ProcEnv, graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        match self {
            Self::Start => {
                let scope = match env.stack.pop().map(|var| var.val.read().clone()) {
                    Some(Val::Obj(node)) => {
                        if !node.node_exists(graph) { return Err(Error::FuncArgsInfo("load: that object doesn't exist".into())); }
                        Some(node)
                    },
                    Some(Val::Null) | Some(Val::Void) | None => None,
                    Some(_) => return Err(Error::FuncArgsInfo("load takes an object (or nothing, for the whole document)".into())),
                };
                let funcs: Vector<DataRef> = load_functions(graph, &scope).into_iter().collect();
                run_functions(env, graph, funcs)
            },
            Self::Run { funcs } => {
                let funcs: Vector<DataRef> = funcs.iter().filter(|func| runnable(graph, func)).cloned().collect();
                run_functions(env, graph, funcs)
            },
            Self::Done { funcs } => {
                for func in funcs { graph.loading.remove(func); }
                env.stack.push(Variable::val(Val::Num(Num::Int(funcs.len() as i64))));
                Ok(None)
            },
            Self::Failed { funcs } => {
                for func in funcs { graph.loading.remove(func); }
                env.stack.pop(); // the error value (what a catch block gets)
                let message = env.stack.pop().map(|var| var.val.read().to_string()).unwrap_or_default();
                let mut message = message.as_str();
                while let Some(inner) = message.strip_prefix("awaited process failed: ") { message = inner; }
                Err(Error::Custom(format!("load failed: {message}").into()))
            },
        }
    }
}


/// Run these #[load] functions, each in its own process (at the same time), and wait for them; then Done (the count).
fn run_functions(env: &mut ProcEnv, graph: &mut Graph, funcs: Vector<DataRef>) -> Result<Option<Instructions>, Error> {
    if funcs.is_empty() {
        env.stack.push(Variable::val(Val::Num(Num::Int(0))));
        return Ok(None);
    }
    // running while this runtime lives: one that never finished (Ex. a call that timed out) can run again later
    for func in &funcs { graph.loading.insert(func.clone(), std::sync::Arc::downgrade(&env.alive)); }

    // each runs in its own process (at the same time), then each is awaited
    let base = env.stack.len();
    let mut run = Vector::new();
    for func in &funcs {
        let call = Arc::new(FuncCall {
            as_ref: false,
            cnull: false,
            stack: false,
            func: Some(func.clone().into()),
            search: None,
            args: Default::default(),
            oself: None,
        }) as Arc<dyn Instruction>;
        run.push_back(Arc::new(Base::Spawn(Arc::new((Instructions::from(vector![call]), Type::Void)))) as Arc<dyn Instruction>);
        run.push_back(SUSPEND.clone()); // start it now (a process spawns one at a time)
    }
    for left in (0..funcs.len()).rev() {
        run.push_back(AWAIT.clone());
        run.push_back(Arc::new(Base::PopUntilStackCount(base + left)) as Arc<dyn Instruction>); // its result, if any
    }
    let mut instructions = Instructions::default();
    instructions.push(Arc::new(TryCatchIns {
        try_ins: run,
        err_ins: vector![Arc::new(LoadIns::Failed { funcs: funcs.clone() }) as Arc<dyn Instruction>],
        catch_ins: vector![],
    }));
    instructions.push(Arc::new(LoadIns::Done { funcs }));
    Ok(Some(instructions))
}


/// The #[load] functions to run: in the scope (an object and the objects in it, or the whole document), not running
/// already, and reachable by the code asking. Code in a sealed part runs its own part's functions; other code never
/// runs a sealed part's functions (as a scan by attribute never finds them).
fn load_functions(graph: &Graph, scope: &Option<NodeRef>) -> Vec<DataRef> {
    let mut funcs: Vec<DataRef> = load_attr_functions(graph, scope).into_iter().filter(|dref| runnable(graph, dref)).collect();
    funcs.sort();
    funcs
}


/// Every #[load] function in the scope (an object and the objects in it, or the whole document), running or not.
pub(crate) fn load_attr_functions(graph: &Graph, scope: &Option<NodeRef>) -> FxHashSet<DataRef> {
    let mut funcs = FxHashSet::default();
    for (dref, data) in &graph.data {
        let Some(func) = data.data.as_dyn_any().downcast_ref::<Func>() else { continue };
        if !func.attributes.contains_key(LOAD_ATTR.as_str()) { continue; }
        if let Some(scope) = scope {
            if !data.nodes.iter().any(|node| node.child_of(graph, scope)) { continue; }
        }
        funcs.insert(dref.clone());
    }
    funcs
}


/// May this #[load] function run now? Not while it's running, and only for code that reaches it (code in a sealed part
/// runs its own part's functions; other code never runs a sealed part's functions).
fn runnable(graph: &Graph, dref: &DataRef) -> bool {
    if graph.loading.get(dref).is_some_and(|alive| alive.strong_count() > 0) || !graph.data.contains_key(dref) { return false; }
    if !graph.confined.is_empty() {
        graph.confined_reaches_data(dref)
    } else {
        graph.seals.confines.is_empty() || graph.func_confines(dref).is_empty()
    }
}


#[cfg(test)]
mod tests {
    use crate::{model::{Graph, Profile}, runtime::Val};

    fn doc(src: &str) -> Graph {
        let mut graph = Graph::default();
        graph.quiet(true);
        graph.parse_stof_src(src, None, Profile::default()).unwrap();
        graph
    }

    fn call(graph: &mut Graph, func: &str) -> Val {
        match graph.call(func, None, vec![]) {
            Ok(val) => val,
            Err(error) => panic!("{func} failed: {error}"),
        }
    }

    #[test]
    /// Runs every #[load] function in the document, sync and async, waiting for each; again each time.
    fn runs_all_each_time() {
        let mut graph = doc(r#"
            count: 0
            later: 0
            a: { #[load] fn one() { root.count += 1; } }
            b: { c: { #[load] async fn two() { sleep(5ms); root.count += 10; } } }
            #[load] fn three() { root.count += 100; }
            fn plain() { root.count += 1000; }
            fn run() -> int await load()
            fn counted() -> int self.count
        "#);
        assert_eq!(call(&mut graph, "run"), Val::from(3));
        assert_eq!(call(&mut graph, "counted"), Val::from(111), "plain functions don't run; the async one finished");
        assert_eq!(call(&mut graph, "run"), Val::from(3));
        assert_eq!(call(&mut graph, "counted"), Val::from(222), "other work in a #[load] runs again");
    }

    #[test]
    /// load(obj) runs the functions in that object and the objects in it.
    fn scope() {
        let mut graph = doc(r#"
            ran: []
            a: { #[load] fn in_a() { root.ran.push_back('a'); } inner: { #[load] fn in_inner() { root.ran.push_back('inner'); } } }
            b: { #[load] fn in_b() { root.ran.push_back('b'); } }
            fn run_a() -> int await load(self.a)
            fn run_inner() -> int await load(self.a.inner)
            fn listed() -> str { const items = self.ran; items.sort(); items.join(',') }
        "#);
        assert_eq!(call(&mut graph, "run_a"), Val::from(2));
        assert_eq!(call(&mut graph, "listed"), Val::from("a,inner"));
        assert_eq!(call(&mut graph, "run_inner"), Val::from(1));
        assert_eq!(call(&mut graph, "listed"), Val::from("a,inner,inner"));
    }

    #[test]
    /// With no #[load] functions, nothing runs; a call that isn't awaited is finished when the host call returns.
    fn none_and_not_awaited() {
        let mut graph = doc(r#"
            done: false
            part: { #[load] async fn slow() { sleep(5ms); root.done = true; } }
            empty: {}
            fn none() -> int await load(self.empty)
            fn start() { load(self.part); }
            fn finished() -> bool self.done
        "#);
        assert_eq!(call(&mut graph, "none"), Val::from(0));
        call(&mut graph, "start");
        assert_eq!(call(&mut graph, "finished"), Val::from(true));
    }

    #[test]
    /// A #[load] function that calls load on its own object isn't started again (no endless recursion).
    fn no_recursion() {
        let mut graph = doc(r#"
            count: 0
            part: {
                #[load] fn again() -> int { root.count += 1; await load(self) }
                #[load] fn other() { root.count += 10; }
            }
            fn run() -> int await load(self.part)
            fn counted() -> int self.count
        "#);
        assert_eq!(call(&mut graph, "run"), Val::from(2));
        assert_eq!(call(&mut graph, "counted"), Val::from(11), "each ran once");
        assert_eq!(call(&mut graph, "run"), Val::from(2), "nothing is left marked as running");
    }

    #[test]
    /// A failing #[load] function fails load, and doesn't stay marked as running.
    fn errors() {
        let mut graph = doc(r#"
            fail: true
            part: { #[load] fn check() { if (root.fail) throw('not ready'); } }
            fn run() -> str {
                try { await load(self.part); return 'loaded'; }
                catch (e, msg) { return msg; }
            }
            fn fix() { self.fail = false; }
            fn bad_arg() -> str { try { await load(5); return 'loaded'; } catch (e, msg) { return msg; } }
        "#);
        let msg = call(&mut graph, "run").to_string();
        assert!(msg.contains("load failed") && msg.contains("not ready"), "{msg}");
        call(&mut graph, "fix");
        assert_eq!(call(&mut graph, "run"), Val::from("loaded"));
        let msg = call(&mut graph, "bad_arg").to_string();
        assert!(msg.contains("load takes an object"), "{msg}");
    }

    #[test]
    /// Untrusted source can't have #[load] functions (unless parsed by code in a sealed part).
    fn untrusted() {
        let mut graph = doc(r#"
            fn attempt() -> str {
                try { parse('#[load] fn get() {}', self.target, 'stof', 'untrusted'); return 'parsed'; }
                catch (e, msg) { return msg; }
            }
            fn plain() -> bool parse('fn get() {}', self.target, 'stof', 'untrusted')
            target: {}
        "#);
        let msg = call(&mut graph, "attempt").to_string();
        assert!(msg.contains("untrusted source can't have #[load] functions"), "{msg}");
        assert_eq!(call(&mut graph, "plain"), Val::from(true));
    }

    #[test]
    /// A sealed part's #[load] functions run only for code in that part.
    fn sealed_parts() {
        let mut graph = doc(r#"
            count: 0
            part: {
                hits: 0
                #[load] fn inside() { self.hits += 1; }
                fn load_own() -> int await load(self)
            }
            #[load] fn outside() { root.count += 1; }
            #[init] fn lock() { seal(self.part, hosts = []); }
            fn run() -> int await load()
            fn run_part() -> int self.part.load_own()
            fn hits() -> int self.part.hits
        "#);
        assert_eq!(call(&mut graph, "run"), Val::from(1), "the document's load doesn't run the sealed part's");
        assert_eq!(call(&mut graph, "hits"), Val::from(0));
        assert_eq!(call(&mut graph, "run_part"), Val::from(1), "the part runs its own");
        assert_eq!(call(&mut graph, "hits"), Val::from(1));
    }

    #[test]
    /// The host's Graph::load: the whole document, or an object and the objects in it.
    fn host_load() {
        let mut graph = doc(r#"
            count: 0
            a: { #[load] fn one() { root.count += 1; } }
            b: { #[load] async fn two() { sleep(2ms); root.count += 10; } }
            fn counted() -> int self.count
        "#);
        assert_eq!(graph.load(None).unwrap(), Val::from(2));
        assert_eq!(call(&mut graph, "counted"), Val::from(11));
        let a = graph.host_node("a").unwrap();
        assert_eq!(graph.load(Some(a)).unwrap(), Val::from(1));
        assert_eq!(call(&mut graph, "counted"), Val::from(12));

        let mut failing = doc("#[load] fn bad() { throw('nope'); }");
        let error = failing.load(None).unwrap_err().to_string();
        assert!(error.contains("load failed") && error.contains("nope"), "{error}");
    }

    #[test]
    /// An import a #[load] function awaits fails load; one it doesn't await fails on its own.
    fn await_imports() {
        let hash = "a".repeat(64);
        let mut graph = doc(&format!(r#"
            awaited: {{ #[load] fn get() {{ await import('https://example.com/a.stof', '{hash}', 'root.a'); }} }}
            not_awaited: {{ #[load] fn get() {{ import('https://example.com/a.stof', '{hash}', 'root.a'); }} }}
        "#));
        let awaited = graph.host_node("awaited").unwrap();
        let error = graph.load(Some(awaited)).unwrap_err().to_string();
        assert!(error.contains("needs network access"), "{error}");
        let not_awaited = graph.host_node("not_awaited").unwrap();
        assert_eq!(graph.load(Some(not_awaited)).unwrap(), Val::from(1));
    }
}
