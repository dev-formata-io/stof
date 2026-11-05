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

//! Memory: calling a function many times must not grow the heap.
//!
//! This binary counts live heap bytes with its own allocator, so it holds a single test
//! (tests in one binary run in parallel and would count each other's allocations).
//! Each case runs a function many times and checks that both the graph and the heap stay flat.

// Its own counting allocator: not with the JavaScript or Python builds, which install StofAlloc.
#![cfg(not(any(feature = "js", feature = "py")))]

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicIsize, Ordering};
use stof::model::Graph;
use stof::runtime::Runtime;

struct Counting;
static LIVE: AtomicIsize = AtomicIsize::new(0);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        LIVE.fetch_add(layout.size() as isize, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size() as isize, Ordering::Relaxed);
        unsafe { System.dealloc(ptr, layout) }
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        LIVE.fetch_add(size as isize - layout.size() as isize, Ordering::Relaxed);
        unsafe { System.realloc(ptr, layout, size) }
    }
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

const WARMUP: usize = 500;
const CALLS: usize = 20_000;

/// Allowed heap growth per call, in bytes. A leaked object costs over 1 KB, so this catches one
/// in every ~100 calls, while leaving room for hash maps that grew to a new capacity once.
const MAX_BYTES_PER_CALL: f64 = 8.;

struct Growth {
    name: &'static str,
    bytes_per_call: f64,
    nodes: (usize, usize),
    data: (usize, usize),
}

fn measure(name: &'static str, src: &str, call: impl Fn(&mut Graph)) -> Growth {
    let mut graph = Graph::default();
    graph.parse_stof_src(src, None, Default::default()).expect(name);
    for _ in 0..WARMUP { call(&mut graph); }

    let nodes = graph.nodes.len();
    let data = graph.data.len();
    let before = LIVE.load(Ordering::Relaxed);
    for _ in 0..CALLS { call(&mut graph); }
    let after = LIVE.load(Ordering::Relaxed);

    Growth {
        name,
        bytes_per_call: (after - before) as f64 / CALLS as f64,
        nodes: (nodes, graph.nodes.len()),
        data: (data, graph.data.len()),
    }
}

fn call(path: &'static str) -> impl Fn(&mut Graph) {
    move |graph: &mut Graph| { Runtime::call(graph, path, vec![]).expect(path); }
}

#[test]
fn repeated_calls_do_not_grow_memory() {
    let cases = vec![
        measure("read a field", "x: 1\nfn f() -> int { self.x }", call("f")),
        measure("write a number field", "x: 1\nfn f() { self.x = self.x + 1; }", call("f")),
        measure("write a string field", "s: 'a'\nfn f() { self.s = `hello ${self.s.len()}`; }", call("f")),
        measure("using an object", "fn f() { using t = new { a: 1, b: 'x' }; t.a = 2; }", call("f")),
        measure("nested objects with using", "fn f() { using t = new { a: new { b: new { c: 1 } }, l: [1, 2, 3] }; t.a.b.c = 2; }", call("f")),
        measure("new on an arena, drop the arena", "fn f() { const arena = new {}; for (let i = 0; i < 5; i += 1) { new { i: i } on arena; } drop(arena); }", call("f")),
        measure("replace an object field, drop the old one", "o: { a: 1 }\nfn f() { const old = self.o; self.o = new { a: 2 }; drop(old); }", call("f")),
        measure("remove a field holding an object", "fn f() { self.tmp = new { a: 1 }; self.remove('tmp'); }", call("f")),
        measure("typed object with a constructor", r#"
            #[type] Point: { x: 0, y: 0, #[constructor] fn made() { self.made = true; } fn len() -> float { self.x + self.y } }
            fn f() -> float { using p = new Point { x: 1, y: 2 }; p.len() }
        "#, call("f")),
        measure("list push and pop", "l: []\nfn f() { self.l.push_back(1); self.l.pop_back(); }", call("f")),
        measure("map insert of the same key", "m: map()\nfn f() { self.m.insert('k', 1); }", call("f")),
        measure("try and catch", "fn f() { try { throw('no'); } catch (e: str) { const x = e; } }", call("f")),
        measure("call source, removed after", "x: 1", |graph: &mut Graph| {
            Runtime::call_source(graph, "fn g() -> int { using t = new { a: self.x }; t.a }", None, vec![]).expect("call source");
        }),
    ];

    let mut failed = Vec::new();
    for case in &cases {
        println!("{:<44} {:>8.1} B/call   nodes {} -> {}   data {} -> {}",
            case.name, case.bytes_per_call, case.nodes.0, case.nodes.1, case.data.0, case.data.1);
        if case.nodes.0 != case.nodes.1 || case.data.0 != case.data.1 || case.bytes_per_call > MAX_BYTES_PER_CALL {
            failed.push(case.name);
        }
    }
    assert!(failed.is_empty(), "memory grew across calls in: {:?}", failed);
}
