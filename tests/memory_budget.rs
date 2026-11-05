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

//! The memory budget (`Graph::max_memory`), with the counting allocator installed like the JS and Python builds.
//! Each runaway document must fail with a memory limit error (not run the process out of memory), stay usable,
//! and give its memory back.

use stof::model::Graph;
use stof::runtime::{Runtime, Val, memory};

// the JavaScript and Python builds install it themselves
#[cfg(not(any(feature = "js", feature = "py")))]
#[global_allocator]
static ALLOC: memory::StofAlloc = memory::StofAlloc;

/// The heap count is process-wide, so these tests run one at a time.
static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());
fn serial() -> std::sync::MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Small, so the runaways stop quickly.
const BUDGET: usize = 16 * 1024 * 1024;

const RUNAWAYS: &[(&str, &str)] = &[
    ("string doubling", r#"fn probe() { let s = 'aaaaaaaaaaaaaaaa'; loop { s = s + s; } }"#),
    ("string doubling in place", r#"fn probe() { let s = 'aaaaaaaaaaaaaaaa'; loop { s += s; } }"#),
    ("list growth", r#"fn probe() { const l = []; loop { l.push_back('xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx'); } }"#),
    ("object growth", r#"fn probe() { loop { new { a: 1, b: 'xxxxxxxxxxxxxxxxxxxxxxxxx' }; } }"#),
    ("map growth", r#"fn probe() { const m = map(); let i = 0; loop { m.insert(i, 'xxxxxxxxxxxxxxxx'); i += 1; } }"#),
    ("field growth", r#"fn probe() { let i = 0; loop { Obj.insert(self, `f${i}`, 'xxxxxxxxxxxxxxxx'); i += 1; } }"#),
    ("string repeat", r#"fn probe() { const s = 'abc' * 1000000000000; }"#),
    ("replace", r#"fn probe() { let s = 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa' * 10000; s = s.replace('a', 'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb'); }"#),
    ("split", r#"fn probe() { const s = 'a' * 4000000; const parts = s.split(''); }"#),
    ("join", r#"fn probe() { const big = 'x' * 1000000; const l = []; for (const i in 40) l.push_back(big); const s = l.join(','); }"#),
    ("blob to list", r#"fn probe() { const b: blob = 'a' * 4000000; const l = b as list; }"#),
    ("string to list", r#"fn probe() { const l = ('a' * 4000000) as list; }"#),
    ("parse", r#"fn probe() { const src = '[' + ('1,' * 3000000) + '1]'; parse(src, self, 'json'); }"#),
];

fn graph(src: &str) -> Graph {
    let mut graph = Graph::default();
    graph.quiet(true);
    graph.max_memory = Some(BUDGET);
    graph.parse_stof_src(src, None, Default::default()).unwrap();
    graph
}

#[test]
fn runaways_hit_the_limit() {
    let _serial = serial();
    for (name, src) in RUNAWAYS {
        let before = memory::mark();
        let mut graph = graph(src);
        let error = Runtime::call(&mut graph, "probe", vec![]).expect_err(name);
        assert!(format!("{error:?}").contains("MemoryLimit"), "{name}: {error:?}");

        // the document still works, and the memory comes back
        graph.parse_stof_src("fn ok() -> int { 40 + 2 }", None, Default::default()).unwrap();
        assert_eq!(Runtime::call(&mut graph, "ok", vec![]).unwrap(), Val::from(42i64), "{name}");
        drop(graph);
        let held = memory::since(before);
        assert!(held < BUDGET as isize, "{name}: {} MB still held", held / (1024 * 1024));
    }
}

#[test]
/// Like a timeout, a catch block can't keep the call going while the memory it used is still held.
fn the_call_fails_even_in_a_try() {
    let _serial = serial();
    let mut graph = graph(r#"
        fn probe() -> str {
            try { let s = 'aaaaaaaaaaaaaaaa'; loop { s = s + s; } }
            catch (e) { return e; }
            'none'
        }"#);
    let error = Runtime::call(&mut graph, "probe", vec![]).unwrap_err();
    assert!(format!("{error:?}").contains("MemoryLimit"), "{error:?}");
}

#[test]
fn under_the_limit_is_fine() {
    let _serial = serial();
    let mut graph = graph(r#"
        fn probe() -> int {
            const s = 'abc' * 20000;
            const parts = s.split('b');
            const l = [];
            for (const i in 10000) l.push_back(i);
            s.len() + parts.len() + l.len()
        }"#);
    assert_eq!(Runtime::call(&mut graph, "probe", vec![]).unwrap(), Val::from(60_000i64 + 20_001 + 10_000));
}

#[test]
fn state_kept_between_calls_is_not_charged_again() {
    let _serial = serial();
    // each call keeps 4 MB in the document: the budget is growth during a call, so many calls are fine
    let mut graph = graph(r#"
        state: []
        fn probe() { self.state.push_back('x' * 4000000); }"#);
    for _ in 0..8 { Runtime::call(&mut graph, "probe", vec![]).unwrap(); }
}

#[test]
fn no_limit() {
    let _serial = serial();
    let mut graph = graph(r#"fn probe() -> int { const s = 'x' * 40000000; s.len() }"#);
    assert!(Runtime::call(&mut graph, "probe", vec![]).is_err());
    graph.max_memory = None;
    assert_eq!(Runtime::call(&mut graph, "probe", vec![]).unwrap(), Val::from(40_000_000i64));
}

#[test]
#[ignore]
fn measure_element_cost() {
    let _serial = serial();
    for (name, src, n) in [
        ("split", "state: null\nfn probe() { self.state = ('abc' * 100000).split('b'); }", 100_001usize),
        ("push", "state: []\nfn probe() { for (const i in 100000) self.state.push_back(i); }", 100_000),
        ("chars", "state: null\nfn probe() { self.state = ('a' * 100000) as list; }", 100_000),
    ] {
        let mut graph = Graph::default();
        graph.max_memory = None;
        graph.parse_stof_src(src, None, Default::default()).unwrap();
        let before = memory::mark();
        Runtime::call(&mut graph, "probe", vec![]).unwrap();
        println!("{name}: {} bytes per element", memory::since(before) / n as isize);
    }
}
