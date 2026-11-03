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

//! Robustness: no input makes Stof panic or hang.
//!
//! A panic kills the whole engine in WebAssembly, and hosts parse and run text people and models write. Anything is
//! allowed to fail with an error; nothing is allowed to panic, and nothing is allowed to run forever. Three phases:
//! 1. Every `.stof` file in the repo, cut off at many points and with small random edits: each is parsed, and some
//!    have their tests run.
//! 2. Every library function, called with edge values (nothing, extremes, every kind of value, the document itself).
//! 3. Every format: garbage parsed, and odd objects written and read back.
//!
//! A watchdog fails the run when one input runs for 15 seconds (a loop inside one call that the execution time limit
//! can't interrupt). `STOF_FUZZ_SCALE=10 cargo test --release --test robustness -- --nocapture` runs many more
//! inputs and prints each panic's location with the shortest input that reproduces it.

use std::collections::BTreeMap;
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;
use stof::model::{Graph, Profile};
use stof::runtime::Runtime;

static LAST_PANIC: Mutex<Option<String>> = Mutex::new(None);

/// The input running now, and a counter that moves with each input (for the hang watchdog).
static CURRENT: Mutex<String> = Mutex::new(String::new());
static TICK: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// A hang can't be caught like a panic (Ex. a loop inside one library call, which the execution time limit can't
/// interrupt), so a watchdog fails the run with the input that hung.
fn start_watchdog() {
    use std::sync::atomic::Ordering;
    std::thread::spawn(|| {
        let mut last = u64::MAX;
        let mut same_for = 0;
        loop {
            std::thread::sleep(Duration::from_secs(1));
            let tick = TICK.load(Ordering::Relaxed);
            if tick == last { same_for += 1; } else { same_for = 0; last = tick; }
            if same_for >= 15 {
                let input = CURRENT.lock().map(|c| c.clone()).unwrap_or_default();
                eprintln!("\nHANG: one input ran for 15 seconds without finishing:\n{input}\n");
                std::process::exit(1);
            }
        }
    });
}

fn started(input: &str) {
    if let Ok(mut current) = CURRENT.lock() { current.clear(); current.push_str(input); }
    TICK.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}

/// A small deterministic generator, so a failure reproduces.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: usize) -> usize { if n == 0 { 0 } else { (self.next() % n as u64) as usize } }
}

/// Text that tends to break parsers: closers, openers, quotes, operators, keywords, and literals at their edges.
const PIECES: &[&str] = &[
    "{", "}", "(", ")", "[", "]", "<", ">", ";", ":", ",", ".", "..", "'", "\"", "`", "${", "#[", "#", "@", "\\", "/", "/*",
    "*/", "//", "=", "==", "=>", "->", "::", "?", "??", "!", "&&", "||", "+", "-", "*", "%", "&", "|", "^", "~", " ", "\n",
    "fn", "async", "await", "let", "const", "using", "new", "on", "self", "super", "root", "return", "if", "else", "while",
    "for", "in", "loop", "break", "continue", "switch", "case", "try", "catch", "throw", "drop", "as", "typeof", "null",
    "true", "false", "0", "-0", "1e999", "1e-999", "9223372036854775808", "-9223372036854775809", "0x", "0b", "1.", ".5",
    "1_000", "3kg", "5ms", "1GB", "Infinity", "NaN", "map(", "set(", "list(", "tup(", "str", "int", "float", "obj", "fn",
    "<Shape>", "Shape", "x", "é", "日本", "\u{0}", "\r\n", "\t",
];

/// Arguments at the edges: nothing, extremes, every kind of value, and things that point back at the document.
const VALUES: &[&str] = &[
    "null", "true", "0", "1", "-1", "2.5", "-0.0", "Infinity", "-Infinity", "NaN", "9223372036854775807",
    "-9223372036854775808", "1e308", "5kg", "-3ms", "1.2.3", "''", "'a'", "'日本語 é'", "'<a>&'", "'0'", "'-1'",
    "[]", "[1, 'a', null, [2]]", "map()", "map(('k', 1), (2, 'v'))", "set()", "set(1, 'a')", "(1, 'a')", "self",
    "new {}", "new { a: 1, b: new { c: [1, 2] } }", "((x: int): int => x + 1)", "(() => { throw('in arrow'); })",
    "'a' as blob", "Time::now()",
];

/// Text that isn't any format (or barely is).
const GARBAGE: &[&str] = &[
    "''", "'{'", "'['", "'}'", "'{\"a\":'", "'{\"a\": 1e999}'", "'[1, 2,'", "'<a>'", "'<a'", "'</a>'", "'<a b=>'",
    "'a=1&b'", "'%zz=%'", "'a[b]=1&a[b][c]=2'", "'a[]=1&a[x]=2'", "'---\n: :'", "'x = '", "'[[a]]\n[a]'", "'# h\n```'",
    "'\\u0000'", "'\u{0}'", "'日本'", "'a' as blob", "'' as blob", "'\u{ff}\u{fe}' as blob", "null", "1", "[1]", "new {}",
];

/// Functions that print, wait, or end the process: they don't add anything here.
const SKIP: &[&str] = &[
    "Std::exit", "Std::sleep", "Time::sleep", "Std::pln", "Std::err", "Std::dbg", "Std::dbg_tracestack", "Std::trace",
    "Std::peek", "Std::log_debug", "Std::log_error", "Std::log_info", "Std::log_trace", "Std::log_warn", "Std::prof",
    "Obj::dbg_graph",
];

/// Functions that are slow on purpose (passphrase encryption runs scrypt, about a second a call): a few calls each.
const SLOW: &[&str] = &["Age::pass_blobify", "Age::pass_parse"];

/// A graph for running broken programs: nothing printed (they're expected to fail and warn), and calls time out fast.
fn quiet_graph() -> Graph {
    let mut graph = Graph::default();
    graph.quiet(true);
    graph.max_execution_time = Some(Duration::from_millis(250));
    graph
}

/// Run a source function in the graph (a fresh one when the last call broke it). Returns the panic message if it panicked.
fn try_call(graph: &mut Graph, src: &str) -> Option<String> {
    started(src);
    *LAST_PANIC.lock().unwrap() = None;
    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        if graph.main_root().is_none() || graph.nodes.len() > 2_000 {
            *graph = quiet_graph();
        }
        let _ = Runtime::call_source(graph, src, None, vec![]);
    }));
    match result {
        Ok(()) => None,
        Err(_) => {
            *graph = quiet_graph();
            Some(LAST_PANIC.lock().unwrap().take().unwrap_or_else(|| "unknown panic".into()))
        }
    }
}

fn stof_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if path.is_dir() {
            if name != "target" && name != "node_modules" && !name.starts_with('.') { stof_files(&path, out); }
        } else if name.ends_with(".stof") {
            out.push(path);
        }
    }
}

/// The byte index of every char boundary.
fn boundaries(text: &str) -> Vec<usize> {
    let mut b: Vec<usize> = text.char_indices().map(|(i, _)| i).collect();
    b.push(text.len());
    b
}

/// Parse the text, then run its tests, in a fresh graph. An error is fine. Returns the panic message if it panicked.
fn try_input(text: &str, run: bool) -> Option<String> {
    started(text);
    *LAST_PANIC.lock().unwrap() = None;
    // passphrase encryption (Age.pass_*) runs scrypt, about a second a call: parse those tests, don't run them
    let run = run && !text.contains("pass_");
    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        let mut graph = quiet_graph();
        if graph.parse_stof_src(text, None, Profile::test()).is_ok() && run {
            let _ = graph.test(None, false);
        }
    }));
    match result {
        Ok(()) => None,
        Err(_) => Some(LAST_PANIC.lock().unwrap().take().unwrap_or_else(|| "unknown panic".into())),
    }
}

#[test]
fn no_input_panics() {
    let scale: usize = std::env::var("STOF_FUZZ_SCALE").ok().and_then(|s| s.parse().ok()).unwrap_or(1);
    let verbose = std::env::var("STOF_FUZZ_SCALE").is_ok();

    start_watchdog();
    let run_started = std::time::Instant::now();

    // Record where each panic happened (the default hook would print every one).
    panic::set_hook(Box::new(|info| {
        let location = info.location().map(|l| format!("{}:{}", l.file(), l.line())).unwrap_or_default();
        let message = info.payload().downcast_ref::<&str>().map(|s| s.to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_default();
        let message: String = message.chars().take(160).collect();
        *LAST_PANIC.lock().unwrap() = Some(format!("{location}  {message}"));
    }));

    let mut files = Vec::new();
    stof_files(Path::new(env!("CARGO_MANIFEST_DIR")), &mut files);
    files.sort();
    assert!(files.len() >= 20, "found only {} .stof files", files.len());

    // panic location -> (count, a reproducing input)
    let mut panics: BTreeMap<String, (usize, String)> = BTreeMap::new();
    let mut inputs = 0;
    let mut record = |panic: Option<String>, input: &str, inputs: &mut usize| {
        *inputs += 1;
        if let Some(p) = panic {
            let entry = panics.entry(p).or_insert((0, input.to_string()));
            entry.0 += 1;
            if input.len() < entry.1.len() { entry.1 = input.to_string(); }
        }
    };

    let mut rng = Rng(0x5eed_57af_u64.wrapping_add(0x9e37_79b9));
    for path in &files {
        let Ok(text) = std::fs::read_to_string(path) else { continue };
        let cuts = boundaries(&text);

        // Cut off at evenly spaced points, and at random ones.
        let n = 24 * scale;
        for k in 0..n {
            let i = if k % 2 == 0 { cuts[(k * cuts.len()) / n] } else { cuts[rng.below(cuts.len())] };
            let input = &text[..i];
            record(try_input(input, false), input, &mut inputs);
        }

        // Small edits: insert a piece, delete a span, or swap a piece in. Run the tests of what still parses.
        for _ in 0..(24 * scale) {
            let mut input = text.clone();
            for _ in 0..(1 + rng.below(3)) {
                let b = boundaries(&input);
                let at = b[rng.below(b.len())];
                match rng.below(3) {
                    0 => input.insert_str(at, PIECES[rng.below(PIECES.len())]),
                    1 => {
                        let end = b.iter().copied().filter(|&e| e >= at).nth(1 + rng.below(8)).unwrap_or(input.len());
                        input.replace_range(at..end, "");
                    }
                    _ => {
                        let end = b.iter().copied().filter(|&e| e >= at).nth(1).unwrap_or(input.len());
                        input.replace_range(at..end, PIECES[rng.below(PIECES.len())]);
                    }
                }
            }
            let run = rng.below(4) == 0;
            record(try_input(&input, run), &input, &mut inputs);
        }
    }

    // Small inputs on their own: each piece, and pairs of pieces.
    for a in PIECES {
        record(try_input(a, true), a, &mut inputs);
        for b in PIECES.iter().step_by(7) {
            let input = format!("{a}{b}");
            record(try_input(&input, true), &input, &mut inputs);
            let input = format!("x: {a}{b}");
            record(try_input(&input, true), &input, &mut inputs);
        }
    }

    if verbose { eprintln!("mutation phase done: {inputs} inputs, {} ms", run_started.elapsed().as_millis()); }

    // Every library function, called with odd arguments (as `Lib::name(..)`, which is also how methods are called).
    let mut graph = quiet_graph();
    let mut libfuncs: Vec<(String, String)> = graph.libfuncs.iter()
        .flat_map(|(lib, funcs)| funcs.keys().map(move |name| (lib.to_string(), name.clone())))
        .filter(|(lib, name)| !SKIP.contains(&format!("{lib}::{name}").as_str()))
        .collect();
    libfuncs.sort();
    assert!(libfuncs.len() > 250, "only {} library functions", libfuncs.len());
    let pairs: Vec<&str> = VALUES.iter().step_by(if scale > 1 { 2 } else { 12 }).copied().collect();
    for (lib, name) in &libfuncs {
        let started = std::time::Instant::now();
        let mut calls = vec![String::new()];
        if SLOW.contains(&format!("{lib}::{name}").as_str()) {
            for a in ["null", "'pw'"] { for b in ["self", "'a' as blob"] { calls.push(format!("{a}, {b}")); } }
        } else {
            for a in VALUES { calls.push(a.to_string()); }
            for a in VALUES { for b in &pairs { calls.push(format!("{a}, {b}")); } }
            for a in &pairs { calls.push(format!("{a}, {a}, {a}")); calls.push(format!("{a}, 'x', 1, 2, 3, 4")); }
        }
        for args in calls {
            let src = format!("fn probe() {{ {lib}::{name}({args}); }}");
            record(try_call(&mut graph, &src), &src, &mut inputs);
        }
        if verbose { eprintln!("{:>7} ms  {lib}::{name}", started.elapsed().as_millis()); }
    }
    if verbose { eprintln!("library phase done: {inputs} inputs, {} ms", run_started.elapsed().as_millis()); }

    // Every format: parse garbage, and write odd objects.
    let formats: Vec<String> = { let mut f: Vec<String> = graph.formats.keys().cloned().collect(); f.sort(); f };
    for format in &formats {
        let started = std::time::Instant::now();
        for text in GARBAGE {
            let src = format!("fn probe() {{ const o = new {{}}; parse({text}, o, '{format}'); drop(o); }}");
            record(try_call(&mut graph, &src), &src, &mut inputs);
            let src = format!("fn probe() {{ parse({text}, self, '{format}'); }}");
            record(try_call(&mut graph, &src), &src, &mut inputs);
        }
        for value in VALUES {
            for src in [
                format!("fn probe() {{ const o = new {{ v: {value}, l: [{value}], n: new {{ v: {value} }} }}; stringify('{format}', o); blobify('{format}', o); drop(o); }}"),
                format!("fn probe() {{ stringify('{format}', {value}); blobify('{format}', {value}); }}"),
                format!("fn probe() {{ const o = new {{}}; parse(stringify('{format}', new {{ v: {value} }}), o, '{format}'); drop(o); }}"),
                format!("fn probe() {{ const o = new {{}}; parse(blobify('{format}', new {{ v: {value} }}), o, '{format}'); drop(o); }}"),
            ] {
                record(try_call(&mut graph, &src), &src, &mut inputs);
            }
        }
        if verbose { eprintln!("{:>7} ms  format {format}", started.elapsed().as_millis()); }
    }

    let _ = panic::take_hook();
    let total: usize = panics.values().map(|(c, _)| c).sum();
    if verbose || !panics.is_empty() {
        println!("{} inputs from {} files, {} panics at {} locations", inputs, files.len(), total, panics.len());
        for (location, (count, input)) in &panics {
            let shown: String = input.chars().rev().take(120).collect::<Vec<_>>().into_iter().rev().collect();
            println!("\n{count:>6}  {location}\n        input ({} bytes) ends: {:?}", input.len(), shown);
        }
    }
    assert!(panics.is_empty(), "{} inputs panicked, at {} locations (listed above)", total, panics.len());
}
