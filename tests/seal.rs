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

//! Sealing (`Graph::seal`): sealed behavior can be called but not changed, by Stof code or the host, while the
//! rest of the document (and the sealed part's state) keeps working, and the digest is stable.

use stof::model::{Graph, Profile, seal};
use stof::runtime::{Runtime, Val};

const ENGINE: &str = r#"
    #[type]
    Engine: {
        rate: 2
        fn price(units: int) -> int { self.rate * units * 50 }
        fn doubled(units: int) -> int {
            const twice = (x: int): int => x * 2; // arrows evaluated by sealed code still work
            twice(self.price(units))
        }
        fn scaled(units: int) -> int {
            let factor = 3;
            const by = (x: int): int => x * factor; // and capture
            by(units)
        }
        helpers: {
            fn one() -> int { 1 }
        }
        fn count() -> int { self.rate = self.rate + 1; self.rate } // state: fields still change
    }
    policy: {
        fn price() -> int { <Engine>.price(1) }
    }
"#;

fn engine() -> Graph {
    let mut graph = Graph::default();
    graph.quiet(true);
    graph.parse_stof_src(ENGINE, None, Profile::default()).unwrap();
    graph
}

fn price(graph: &mut Graph) -> Val {
    Runtime::call_source(graph, "fn f() -> int { <Engine>.price(1) }", None, vec![]).unwrap()
}

#[test]
fn sealed_behavior_cant_change() {
    let attacks = [
        ("parse a new function into the type", r#"parse("fn price(units: int) -> int { 0 }", <Engine>);"#),
        ("drop the function", "Std.drop(<Engine>.price);"),
        ("drop then parse", r#"const e = <Engine>; drop(e.price); parse("fn price(units: int) -> int { 1 }", e);"#),
        ("assign over the function", "const e = <Engine>; e.price = (units: int): int => 0;"),
        ("add a new function", r#"parse("fn extra() -> int { 0 }", <Engine>);"#),
        ("parse into a sealed child", r#"parse("fn one() -> int { 0 }", <Engine>.helpers);"#),
        ("replace a sealed child object", "const e = <Engine>; e.helpers = new { x: 1 };"),
        ("drop a sealed child object", "drop(<Engine>.helpers);"),
        ("drop the sealed type", "drop(<Engine>);"),
        ("rename a sealed child's field", "Obj.move_field(<Engine>, 'helpers', 'other');"),
        ("move a sealed child's field out", "Obj.move_field(<Engine>, 'helpers', 'root.policy.helpers');"),
        ("move a sealed child object", "Obj.move(<Engine>.helpers, root.policy);"),
        ("change a sealed function's attributes", "const f = <Engine>.price; f.set_attribute('main');"),
        ("remove a sealed function's attributes", "Obj.remove_attribute(<Engine>, 'x', 'price');"),
        ("change a sealed field's attributes", "Obj.set_attribute(<Engine>, 'readonly', null, 'rate');"),
        ("change a sealed object's attributes", "Obj.set_attribute(<Engine>, 'type', 'Other');"),
        ("shadow the type with another of its name", r##"parse("#[type] Engine: { fn price(units: int) -> int { 0 } }", self);"##),
    ];
    for (what, attack) in attacks {
        let mut graph = engine();
        graph.seal_path("<Engine>", false, None).unwrap();
        graph.parse_stof_src(&format!("evil: {{ fn attack() {{ {attack} }} }}"), None, Profile::default()).unwrap();
        let res = Runtime::call(&mut graph, "root.evil.attack", vec![]);
        assert!(res.as_ref().is_err_and(|err| { let err = format!("{err:?}"); err.contains("Sealed") || err.contains("sealed") }), "{what}: {res:?}");
        assert_eq!(price(&mut graph), Val::from(100i64), "{what}: the price changed");
        assert_eq!(Runtime::call_source(&mut graph, "fn f() -> int { <Engine>.helpers.one() }", None, vec![]).unwrap(), Val::from(1i64), "{what}");
    }
}

#[test]
fn the_host_is_refused_too() {
    let mut graph = engine();
    graph.seal_path("<Engine>", false, None).unwrap();
    let engine = graph.seal_target("<Engine>").unwrap();
    let err = graph.parse_stof_src("fn price(units: int) -> int { 0 }", Some(engine.clone()), Profile::default()).unwrap_err();
    assert!(err.to_string().contains("sealed"), "{err}");
    assert!(!graph.remove_node(&engine, true));
    let _ = graph.take_seal_violation();
    assert_eq!(price(&mut graph), Val::from(100i64));

    // until it unseals
    graph.unseal(&engine);
    graph.parse_stof_src("fn price(units: int) -> int { 0 }", Some(engine), Profile::default()).unwrap();
    assert_eq!(price(&mut graph), Val::from(0i64));
}

#[test]
fn sealed_code_still_runs() {
    let mut graph = engine();
    graph.seal_path("<Engine>", false, None).unwrap();
    let call = |graph: &mut Graph, src: &str| Runtime::call_source(graph, src, None, vec![]).unwrap();
    assert_eq!(call(&mut graph, "fn f() -> int { <Engine>.doubled(1) }"), Val::from(200i64));
    assert_eq!(call(&mut graph, "fn f() -> int { <Engine>.doubled(2) }"), Val::from(400i64));
    assert_eq!(call(&mut graph, "fn f() -> int { <Engine>.scaled(5) }"), Val::from(15i64));
    assert_eq!(call(&mut graph, "fn f() -> int { (new Engine {}).price(2) }"), Val::from(200i64)); // instances
    assert_eq!(call(&mut graph, "fn f() -> int { <Engine>.count() }"), Val::from(3i64)); // state changes
    assert_eq!(Runtime::call(&mut graph, "root.policy.price", vec![]).unwrap(), Val::from(150i64));

    // the host can still call source on a sealed object (attached for the call, then removed)
    let engine = graph.seal_target("<Engine>").unwrap();
    assert_eq!(Runtime::call_source(&mut graph, "fn f() -> int { self.price(1) }", Some(engine.clone()), vec![]).unwrap(), Val::from(150i64));
    assert!(!engine.node(&graph).unwrap().data.contains_key("f"));

    // the rest of the document is unchanged
    graph.parse_stof_src("policy: { fn price() -> int { 7 } }", None, Profile::default()).unwrap();
    assert_eq!(Runtime::call(&mut graph, "root.policy.price", vec![]).unwrap(), Val::from(7i64));
}

#[test]
fn fields_too() {
    let mut graph = engine();
    graph.seal_path("<Engine>", true, None).unwrap();
    let res = Runtime::call_source(&mut graph, "fn f() -> int { <Engine>.count() }", None, vec![]);
    assert!(res.is_err_and(|err| format!("{err:?}").contains("Sealed")));
    assert_eq!(price(&mut graph), Val::from(100i64));
}

#[test]
fn digests() {
    let digest = |src: &str| {
        let mut graph = Graph::default();
        graph.parse_stof_src(src, None, Profile::default()).unwrap();
        let engine = graph.seal_target("<Engine>").unwrap();
        (graph.digest(&engine, false).unwrap(), graph.digest(&engine, true).unwrap())
    };
    let (functions, fields) = digest(ENGINE);
    assert_eq!(functions.len(), 64);
    assert_eq!(digest(ENGINE), (functions.clone(), fields.clone()), "the same source (with arrows) gives the same digest");

    // the same from BSTF
    let mut graph = engine();
    let bytes = graph.blobify("bstf", None).unwrap();
    let mut loaded = Graph::default();
    loaded.binary_import("bstf", bytes, None, &Profile::default()).unwrap();
    let engine = loaded.seal_target("<Engine>").unwrap();
    assert_eq!(loaded.digest(&engine, false).unwrap(), functions, "loaded from BSTF");
    let node = graph.seal_target("<Engine>").unwrap();
    assert_eq!(graph.digest(&node, false).unwrap(), functions);

    // changing a function changes it; changing state changes only the fields digest
    let changed = ENGINE.replace("self.rate * units * 50", "self.rate * units * 51");
    assert_ne!(digest(&changed).0, functions);
    let state = ENGINE.replace("rate: 2", "rate: 3");
    assert_eq!(digest(&state).0, functions);
    assert_ne!(digest(&state).1, fields);
    // where it sits doesn't matter, what it calls does
    let nested = format!("lib: {{ {ENGINE} }}");
    assert_eq!(digest(&nested).0, functions);
    let arrow = ENGINE.replace("x * 2;", "x * 3;");
    assert_ne!(digest(&arrow).0, functions, "arrow bodies are in it");
}

#[test]
fn expected_digests_and_signatures() {
    let mut graph = engine();
    let node = graph.seal_target("<Engine>").unwrap();
    let digest = graph.digest(&node, false).unwrap();

    let err = graph.seal_path("<Engine>", false, Some(&"0".repeat(64))).unwrap_err();
    assert!(format!("{err:?}").contains("isn't what was expected"));
    assert!(!graph.is_sealed(&node), "not sealed when the digest doesn't match");
    assert_eq!(graph.seal_path("<Engine>", false, Some(&digest.to_uppercase())).unwrap(), digest);

    let secret = "nWGxne/9WmC6hEr0kuwsxERJxWl7MmkZcDusAxyuf2A="; // a test key
    let public = seal::public_key(secret).unwrap();
    let signature = seal::sign(&digest, secret).unwrap();
    assert!(seal::verify(&digest, &public, &signature));
    assert!(!seal::verify(&"1".repeat(64), &public, &signature), "another digest");
    let other = seal::public_key("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=").unwrap();
    assert!(!seal::verify(&digest, &other, &signature), "another key");
    assert!(!seal::verify(&digest, "not a key", &signature));

    let mut fresh = engine();
    assert_eq!(fresh.seal_signed("<Engine>", false, &public, &signature).unwrap(), digest);
    let mut tampered = Graph::default();
    tampered.parse_stof_src(&ENGINE.replace("* 50", "* 5"), None, Profile::default()).unwrap();
    assert!(tampered.seal_signed("<Engine>", false, &public, &signature).is_err());
}

#[test]
/// Block and switch expressions share one return tag now (so compiling is deterministic): returns still go to the
/// right place, nested and recursive.
fn block_returns() {
    let mut graph = Graph::default();
    graph.parse_stof_src(r#"
        fn nested(x: int) -> int {
            const a = { if (x > 0) return 1; 2 };
            const b = {
                const inner = { if (x > 5) return 10; 20 };
                if (inner > 15) return inner + 1;
                inner
            };
            const c = switch (x) { case 1: { 100 } default: { 200 } };
            a + b + c
        }
        fn recurse(n: int) -> int {
            const v = { if (n < 1) return 0; n };
            if (v == 0) return 0;
            v + self.recurse(n - 1)
        }
    "#, None, Profile::default()).unwrap();
    assert_eq!(Runtime::call(&mut graph, "nested", vec![Val::from(1i64)]).unwrap(), Val::from(1i64 + 21 + 100));
    assert_eq!(Runtime::call(&mut graph, "nested", vec![Val::from(7i64)]).unwrap(), Val::from(1i64 + 10 + 200));
    assert_eq!(Runtime::call(&mut graph, "nested", vec![Val::from(0i64)]).unwrap(), Val::from(2i64 + 21 + 200));
    assert_eq!(Runtime::call(&mut graph, "recurse", vec![Val::from(4i64)]).unwrap(), Val::from(10i64));
}

#[test]
/// A document can seal parts of itself (Ex. in #[init]): the same seal as the host's, which only the host can undo.
fn sealed_from_stof() {
    let mut graph = Graph::default();
    graph.quiet(true);
    graph.parse_stof_src(&format!("{ENGINE}\n#[init] fn lock() {{ self.digest = seal(<Engine>); }}"), None, Profile::default()).unwrap();
    let engine = graph.seal_target("<Engine>").unwrap();
    assert!(graph.is_sealed(&engine));
    let from_stof = Runtime::call_source(&mut graph, "fn f() -> str { self.digest }", None, vec![]).unwrap();
    assert_eq!(from_stof, Val::from(graph.digest(&engine, false).unwrap().as_str()));
    let res = Runtime::call(&mut graph, "root.policy.price", vec![]);
    assert_eq!(res.unwrap(), Val::from(100i64), "sealed code still runs");
    let attack = Runtime::call_source(&mut graph, r#"fn f() { parse("fn price(units: int) -> int { 0 }", <Engine>); }"#, None, vec![]);
    assert!(attack.is_err_and(|err| err.to_string().contains("sealed")));
    // documents have no unseal; the host does
    graph.unseal(&engine);
    assert!(!graph.is_sealed(&engine));
}

#[test]
fn unsealed_documents_are_unchanged() {
    // nothing sealed: every change works as before (and the checks are skipped)
    let mut graph = engine();
    graph.parse_stof_src(r#"evil: { fn attack() { parse("fn price(units: int) -> int { 0 }", <Engine>); } }"#, None, Profile::default()).unwrap();
    Runtime::call(&mut graph, "root.evil.attack", vec![]).unwrap();
    assert_eq!(price(&mut graph), Val::from(0i64));
}
