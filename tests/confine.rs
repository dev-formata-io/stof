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

//! Seals for code that comes from somewhere else: confined parts (`reads`, `hosts`) reach only what they're given,
//! removable parts can be dropped whole, messages are signed under a context, and digests travel between documents.

use stof::model::{Graph, Profile, SealOptions, seal};
use stof::runtime::{Runtime, Val, Variable};

/// A document with something to protect and a tool from somewhere else (`tools.acme`).
const DOC: &str = r#"
    secrets: { key: 'abc' }
    inputs: { city: 'Boston' }

    #[type]
    Helpers: {
        fn shout(s: str) -> str { s + '!' }
        fn peek() -> unknown { root.secrets.key }
    }

    fn secret_fn() -> str { 'secret' }
    fn mixed() -> str {
        const city = root.tools.acme.run();
        root.secrets.key + city
    }

    tools: {
        acme: {
            hits: 0
            fn run() -> str { self.hits = self.hits + 1; root.inputs.city }
            fn steal() -> unknown { root.secrets.key }
            fn steal_super() -> unknown { super.super.secrets }
            fn parent() -> unknown { Obj.parent(self) }
            fn poke() { root.inputs.city = 'Nowhere'; }
            fn call_out() -> unknown { root.secret_fn() }
            fn helped() -> str { <Helpers>.shout(root.inputs.city) }
            fn helper_peeks() -> unknown { <Helpers>.peek() }
            fn export_all() -> str { stringify('json') }
            fn export_self() -> str { stringify('json', self) }
            fn make() -> int { const tmp = new { x: 3 }; const x = tmp.x; drop(tmp); x }
            fn arrow() -> int { const f = (x: int): int => x * 2; f(self.hits) }
            fn new_root() { root2 = new { a: 1 }; }
            fn reseal() { seal(root.inputs); }
            fn fetch(url: str) -> unknown { await Http.fetch(url) }
        }
    }
"#;

fn doc() -> Graph {
    let mut graph = Graph::default();
    graph.quiet(true);
    graph.parse_stof_src(DOC, None, Profile::default()).unwrap();
    graph
}

fn confined(graph: &mut Graph) {
    let reads = vec!["inputs".to_string(), "<Helpers>".to_string()];
    graph.seal_path_with("tools.acme", &SealOptions { reads: Some(reads), ..Default::default() }, None).unwrap();
}

fn call(graph: &mut Graph, path: &str) -> Result<Val, String> {
    Runtime::call(graph, path, vec![]).map_err(|error| error.to_string())
}

#[test]
fn confined_code_reaches_only_what_it_reads() {
    let mut graph = doc();
    confined(&mut graph);

    assert_eq!(call(&mut graph, "tools.acme.run").unwrap(), Val::from("Boston"), "it reads what it's given");
    assert_eq!(call(&mut graph, "tools.acme.run").unwrap(), Val::from("Boston"));
    assert_eq!(graph.field_value("root.tools.acme.hits", None), Some(Val::from(2i64)), "and changes its own state");
    assert_eq!(call(&mut graph, "tools.acme.helped").unwrap(), Val::from("Boston!"), "and calls what it reads");
    assert_eq!(call(&mut graph, "tools.acme.make").unwrap(), Val::from(3i64), "objects of its own");
    assert_eq!(call(&mut graph, "tools.acme.arrow").unwrap(), Val::from(4i64), "arrow functions");
    assert_eq!(call(&mut graph, "tools.acme.export_self").unwrap().to_string().contains("hits"), true, "exports itself");

    // everything else is out of reach
    assert_eq!(call(&mut graph, "tools.acme.steal").unwrap(), Val::Null, "a path elsewhere finds nothing");
    assert_eq!(call(&mut graph, "tools.acme.steal_super").unwrap(), Val::Null, "super too");
    assert_eq!(call(&mut graph, "tools.acme.helper_peeks").unwrap(), Val::Null, "what it calls runs confined too");
    let refused = |res: Result<Val, String>, what: &str| assert!(res.as_ref().is_err_and(|err| err.contains("sealed")), "{what}: {res:?}");
    refused(call(&mut graph, "tools.acme.parent"), "a library function handing back an object elsewhere");
    refused(call(&mut graph, "tools.acme.poke"), "changing what it reads");
    refused(call(&mut graph, "tools.acme.export_all"), "exporting everything");
    refused(call(&mut graph, "tools.acme.new_root"), "making a root");
    refused(call(&mut graph, "tools.acme.reseal"), "sealing what it reads");
    assert!(call(&mut graph, "tools.acme.call_out").is_err(), "calling a function elsewhere");
    assert_eq!(graph.field_value("root.inputs.city", None), Some(Val::from("Boston")), "nothing it read changed");
    assert!(!graph.is_sealed(&graph.seal_target("inputs").unwrap()));

    // the rest of the document isn't confined, before, during (around a confined call), or after
    assert_eq!(call(&mut graph, "Helpers.peek").unwrap(), Val::from("abc"));
    assert_eq!(call(&mut graph, "mixed").unwrap(), Val::from("abcBoston"));
    assert_eq!(graph.field_value("root.secrets.key", None), Some(Val::from("abc")));
}

#[test]
/// An object from elsewhere that it's handed (an argument, or a field the host put in it) stays opaque.
fn objects_from_elsewhere_stay_opaque() {
    let mut graph = doc();
    graph.parse_stof_src(r#"
        tools: { acme: {
            fn look(o: obj) -> unknown { o.key }
            fn keep(o: obj) -> str { self.held = o; stringify('json', self) }
            fn text(o: obj) -> str { `${o}` }
            fn hand_back(o: obj) -> obj { o }
        } }
        fn look() -> unknown { root.tools.acme.look(root.secrets) }
        fn keep() -> unknown { root.tools.acme.keep(root.secrets) }
        fn text() -> unknown { root.tools.acme.text(root.secrets) }
        fn hand_back() -> unknown { root.tools.acme.hand_back(root.secrets).key }
    "#, None, Profile::default()).unwrap();
    confined(&mut graph);
    assert_eq!(call(&mut graph, "look").unwrap(), Val::Null, "it can't look inside");
    assert!(call(&mut graph, "keep").is_err_and(|err| err.contains("can't export")), "or export it");
    assert!(!call(&mut graph, "text").unwrap().to_string().contains("abc"), "or print it");
    assert_eq!(call(&mut graph, "hand_back").unwrap(), Val::from("abc"), "it can hand it back");
}

#[test]
fn a_function_value_from_elsewhere_cant_be_called() {
    let mut graph = doc();
    graph.parse_stof_src("tools: { acme: { fn call(f: fn) -> unknown { f() } } }", None, Profile::default()).unwrap();
    confined(&mut graph);
    // the host (or other code) hands it a function from elsewhere: holding it is fine, calling it isn't
    let res = Runtime::call_source(&mut graph, "fn f() -> unknown { root.tools.acme.call(root.secret_fn) }", None, vec![]);
    assert!(res.is_err_and(|err| err.to_string().contains("can't call 'secret_fn'")));
}

#[test]
fn async_and_found_functions_stay_confined() {
    let mut graph = doc();
    graph.parse_stof_src(r#"
        tools: {
            acme: {
                async fn later() -> unknown { root.secrets.key }
                fn wait() -> unknown { await self.later() }
                fn found() -> int { funcs().len() }
                fn call_found() -> unknown { for (const f in funcs()) { if (f.name() == 'secret_fn') return f(); } null }
            }
        }
    "#, None, Profile::default()).unwrap();
    confined(&mut graph);
    assert_eq!(call(&mut graph, "tools.acme.wait").unwrap(), Val::Null, "async calls run confined too");
    assert!(call(&mut graph, "tools.acme.found").is_err(), "functions found elsewhere aren't handed to it");
    assert_eq!(call(&mut graph, "mixed").unwrap(), Val::from("abcBoston"));
}

#[test]
fn confinement_only_narrows() {
    let mut graph = doc();
    confined(&mut graph);
    // the document can confine it more (Ex. no reads at all), but nothing lifts the host's confinement
    graph.parse_stof_src("fn narrow() { seal(root.tools.acme, reads = []); }", None, Profile::default()).unwrap();
    call(&mut graph, "narrow").unwrap();
    assert_eq!(call(&mut graph, "tools.acme.run").unwrap(), Val::Null, "now it reads only itself");
    graph.parse_stof_src("fn widen() { seal(root.tools.acme, reads = [root.secrets]); }", None, Profile::default()).unwrap();
    call(&mut graph, "widen").unwrap();
    assert_eq!(call(&mut graph, "tools.acme.steal").unwrap(), Val::Null, "confinements add up");

    // the host unseals to start over
    let acme = graph.seal_target("tools.acme").unwrap();
    graph.unseal(&acme);
    assert_eq!(call(&mut graph, "tools.acme.steal").unwrap(), Val::from("abc"));
}

#[test]
fn confined_from_stof() {
    let mut graph = doc();
    graph.parse_stof_src(r#"
        fn add_tool(source: str) {
            const tool = new {} on self.tools;
            parse(source, tool);
            seal(tool, removable = true, reads = [self.inputs], hosts = ['api.example.com']);
            self.tools.latest = tool;
        }
    "#, None, Profile::default()).unwrap();
    let source = "fn city() -> unknown { root.inputs.city } fn key() -> unknown { root.secrets.key }";
    Runtime::call(&mut graph, "add_tool", vec![Val::from(source)]).unwrap();
    assert_eq!(call(&mut graph, "tools.latest.city").unwrap(), Val::from("Boston"));
    assert_eq!(call(&mut graph, "tools.latest.key").unwrap(), Val::Null);
    let res = Runtime::call_source(&mut graph, r#"fn f() { parse("fn key() -> str { 'x' }", root.tools.latest); }"#, None, vec![]);
    assert!(res.is_err_and(|err| err.to_string().contains("sealed")), "sealed");
    Runtime::call_source(&mut graph, "fn f() { drop(root.tools.latest); }", None, vec![]).unwrap();
    assert!(call(&mut graph, "tools.latest.city").is_err(), "and removable");
}

#[test]
#[cfg(feature = "http")]
fn confined_hosts() {
    let mut graph = doc();
    graph.allow_http_hosts(["other.example.com"]);
    graph.seal_path_with("tools.acme", &SealOptions { hosts: Some(vec!["api.example.com".into()]), ..Default::default() }, None).unwrap();
    let fetch = |graph: &mut Graph, url: &str| Runtime::call(graph, "tools.acme.fetch", vec![Val::from(url)]).map_err(|e| e.to_string());

    // another host: refused by the part's own list
    let res = fetch(&mut graph, "https://other.example.com/x");
    assert!(res.as_ref().is_err_and(|err| err.contains("not a host this sealed part may reach")), "{res:?}");
    // its own host: allowed by the part, then refused by the document's allowlist (it isn't on it), before sending
    let res = fetch(&mut graph, "https://api.example.com/x");
    assert!(res.as_ref().is_err_and(|err| err.contains("not an allowed host for this document")), "{res:?}");
    // hosts alone don't limit what it reads
    assert_eq!(call(&mut graph, "tools.acme.steal").unwrap(), Val::from("abc"));
    // other code isn't limited by the part's list
    graph.parse_stof_src("fn fetch(url: str) -> unknown { await Http.fetch(url) }", None, Profile::default()).unwrap();
    graph.disallow_http_hosts(["other.example.com"]);
    let res = Runtime::call(&mut graph, "fetch", vec![Val::from("https://other.example.com/x")]).map_err(|e| e.to_string());
    assert!(res.as_ref().is_err_and(|err| err.contains("not an allowed host for this document")), "disallowed: {res:?}");
}

#[test]
fn confined_libs_and_time() {
    let mut graph = doc();
    graph.parse_stof_src(r#"
        tools: { timed: {
            fn split() -> list { 'a/b'.split('/') }
            fn signs() -> str { Key.sign(Key.generate(), 'limitr-network/v1/voucher', 'more spend') }
            fn spin() -> int { let i = 0; while (i < 100000000) i += 1; i }
        } }
    "#, None, Profile::default()).unwrap();
    let options = SealOptions { libs: Some(vec!["Str".into()]), time: Some(std::time::Duration::from_millis(300)), ..Default::default() };
    graph.seal_path_with("tools.timed", &options, None).unwrap();

    // a listed library (method calls count as theirs), and one that isn't: a library that signs stays out
    assert!(call(&mut graph, "tools.timed.split").is_ok());
    let res = call(&mut graph, "tools.timed.signs");
    assert!(res.as_ref().is_err_and(|err| err.contains("Key library isn't one it may use")), "{res:?}");
    // its time, each time it's entered (the call's own limit is longer)
    let res = call(&mut graph, "tools.timed.spin");
    assert!(res.as_ref().is_err_and(|err| err.contains("past its time limit")), "{res:?}");
    assert!(call(&mut graph, "tools.timed.split").is_ok(), "entered again: a new time");

    // libs alone don't limit what it reads
    let info = graph.seal_info_path("tools.timed").unwrap().unwrap().to_json(&graph);
    assert_eq!(info["confined"][0]["libs"], serde_json::json!(["Str"]));
    assert_eq!(info["confined"][0]["time"], serde_json::json!(300));
    assert_eq!(info["confined"][0]["reads"], serde_json::Value::Null);
}

#[test]
fn gates() {
    let mut graph = doc();
    graph.parse_stof_src(r#"
        keys: {
            fn sign(text: str) -> str { Key.sign(Key.generate(), 'tools/v1', text).len() + ' ' + root.secrets.key }
            fn other() -> str { root.secrets.key }
        }
        tools: { gated: {
            fn signs() -> str { self.api.sign('order') }
            fn direct() -> str { Key.sign(Key.generate(), 'tools/v1', 'order') }
            fn other() -> unknown { self.api.other() }
            fn after() -> unknown { self.api.sign('x'); root.secrets }
        } }
    "#, None, Profile::default()).unwrap();
    // its gate, handed in (on the part, so it's within its reach), then sealed once with everything
    let part = graph.find_node_named(&graph.host_path("tools.gated", None), graph.main_root());
    assert!(part.is_some());
    graph.parse_stof_src("api: { sign: root.keys.sign, other: root.keys.other }", part, Profile::default()).unwrap();
    let calls = graph.seal_calls(&["keys.sign".into()]).map_err(|e| e.to_string()).unwrap();
    let options = SealOptions { reads: Some(vec![]), libs: Some(vec!["Str".into()]), calls: Some(calls), removable: true, ..Default::default() };
    graph.seal_path_with("tools.gated", &options, None).unwrap();

    // through the gate it signs (Key, and the secret, are the gate's); itself it can't
    assert_eq!(call(&mut graph, "tools.gated.signs").unwrap(), Val::from("88 abc"));
    let res = call(&mut graph, "tools.gated.direct");
    assert!(res.as_ref().is_err_and(|err| err.contains("Key library isn't one it may use")), "{res:?}");
    // a function it holds that isn't a gate stays out; and it's confined again after a gate
    let res = call(&mut graph, "tools.gated.other");
    assert!(res.as_ref().is_err_and(|err| err.contains("can't call 'other'")), "{res:?}");
    assert_eq!(call(&mut graph, "tools.gated.after").unwrap(), Val::Null);

    // gates are functions, outside the part, and shown by path
    assert!(graph.seal_calls(&["keys.missing".into()]).is_err());
    let inside = graph.seal_calls(&["tools.gated.signs".into()]).unwrap();
    let res = graph.seal_path_with("tools.gated", &SealOptions { calls: Some(inside), ..Default::default() }, None);
    assert!(res.as_ref().is_err_and(|err| err.to_string().contains("outside the part")), "{res:?}");
    let info = graph.seal_info_path("tools.gated").unwrap().unwrap().to_json(&graph);
    assert_eq!(info["confined"][0]["calls"], serde_json::json!(["root.keys.sign"]));
}

#[test]
fn host_lists() {
    let hosts = ["api.example.com", "localhost:8080", "[::1]"].iter().map(|h| h.to_string()).collect();
    assert!(seal::host_listed("https://api.example.com/v1?x=1", &hosts));
    assert!(seal::host_listed("https://API.example.com:443/v1", &hosts));
    assert!(seal::host_listed("http://user:pass@api.example.com/v1", &hosts));
    assert!(seal::host_listed("http://localhost:8080/", &hosts));
    assert!(!seal::host_listed("http://localhost:9090/", &hosts));
    assert!(seal::host_listed("http://[::1]:3000/", &hosts));
    assert!(!seal::host_listed("https://api.example.com.evil.com/", &hosts));
    assert!(!seal::host_listed("https://evil.com/?api.example.com", &hosts));
    assert!(!seal::host_listed("https://evil.com#@api.example.com", &hosts));
    assert!(!seal::host_listed("not a url", &hosts));
}

#[test]
#[cfg(feature = "http")]
fn removing_hosts() {
    let mut graph = Graph::default();
    assert!(!graph.disallow_http_hosts(["a.com"]), "nothing to remove from: no list");
    graph.allow_http_hosts(["a.com", "b.com"]);
    assert!(graph.disallow_http_hosts(["A.com"]));
    assert_eq!(graph.http_hosts.as_ref().unwrap().len(), 1);
    graph.disallow_http_hosts(["b.com"]);
    assert!(graph.http_hosts.as_ref().is_some_and(|hosts| hosts.is_empty()), "none left means none allowed");
}

#[test]
fn removable_seals() {
    let mut graph = doc();
    graph.seal_path_with("tools.acme", &SealOptions { removable: true, ..Default::default() }, None).unwrap();
    let acme = graph.seal_target("tools.acme").unwrap();

    // still sealed: no changes, moves, or renames
    for attack in [
        r#"parse("fn run() -> str { 'x' }", root.tools.acme);"#,
        "drop(root.tools.acme.run);",
        "Obj.move(root.tools.acme, root.inputs);",
        "Obj.move_field(root.tools, 'acme', 'other');",
    ] {
        let res = Runtime::call_source(&mut graph, &format!("fn f() {{ {attack} }}"), None, vec![]);
        assert!(res.as_ref().is_err_and(|err| err.to_string().contains("sealed")), "{attack}: {res:?}");
    }
    assert!(graph.is_sealed(&acme));

    // but it can be dropped whole, and the seal goes with it
    Runtime::call_source(&mut graph, "fn f() { drop(root.tools.acme); }", None, vec![]).unwrap();
    assert!(!acme.node_exists(&graph));
    assert!(graph.seals.nodes.is_empty() && graph.seals.roots.is_empty() && graph.seals.removable.is_empty());

    // by the host too
    let mut graph = doc();
    graph.seal_path_with("tools.acme", &SealOptions { removable: true, ..Default::default() }, None).unwrap();
    assert!(graph.remove("tools.acme", false));

    // not when something else seals it (a seal around it, or one inside it that isn't removable)
    let mut graph = doc();
    graph.seal_path_with("tools.acme", &SealOptions { removable: true, ..Default::default() }, None).unwrap();
    graph.seal_path("tools", false, None).unwrap();
    assert!(!graph.remove("tools.acme", false));
    let _ = graph.take_seal_violation();
    let mut graph = doc();
    graph.parse_stof_src("tools: { acme: { inner: { fn x() {} } } }", None, Profile::default()).unwrap();
    graph.seal_path("tools.acme.inner", false, None).unwrap();
    graph.seal_path_with("tools.acme", &SealOptions { removable: true, ..Default::default() }, None).unwrap();
    assert!(!graph.remove("tools.acme", false));
    let _ = graph.take_seal_violation();

    // sealing again without removable makes it stay; sealing again with it doesn't loosen a seal
    let mut graph = doc();
    graph.seal_path_with("tools.acme", &SealOptions { removable: true, ..Default::default() }, None).unwrap();
    graph.seal_path("tools.acme", false, None).unwrap();
    assert!(!graph.remove("tools.acme", false));
    let _ = graph.take_seal_violation();
    graph.seal_path_with("tools.acme", &SealOptions { removable: true, ..Default::default() }, None).unwrap();
    assert!(!graph.remove("tools.acme", false));
}

#[test]
fn signatures() {
    let secret = seal::generate_key();
    assert_ne!(secret, seal::generate_key(), "random");
    let public = seal::public_key(&secret).unwrap();
    let order = r#"{"item":"lamp","qty":2}"#;

    let signature = seal::sign(&secret, "orders/v1", order.as_bytes()).unwrap();
    assert!(seal::verify(&public, "orders/v1", order.as_bytes(), &signature));
    assert!(!seal::verify(&public, "orders/v2", order.as_bytes(), &signature), "another context");
    assert!(!seal::verify(&public, "orders/v1", order.replace("2", "20").as_bytes(), &signature), "another message");
    assert!(!seal::verify(&public, "orders/v1", format!("{order} ").as_bytes(), &signature), "the exact bytes");
    let other = seal::public_key(&seal::generate_key()).unwrap();
    assert!(!seal::verify(&other, "orders/v1", order.as_bytes(), &signature), "another key");
    assert!(!seal::verify("not a key", "orders/v1", order.as_bytes(), &signature));
    assert!(!seal::verify(&public, "orders/v1", order.as_bytes(), "not a signature"));
    // the context is part of what's signed: moving bytes between the context and the message changes it
    assert!(!seal::verify(&public, "orders/v1{", r#""item":"lamp","qty":2}"#.as_bytes(), &signature));
    assert!(seal::sign("not a key", "orders/v1", b"x").is_err_and(|err| err.to_string().starts_with("key:")));
    assert!(seal::public_key("AAAA").is_err_and(|err| err.to_string().contains("32 bytes")));
    // hex keys work too
    let hex: String = {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.decode(&secret).unwrap().iter().map(|b| format!("{b:02x}")).collect()
    };
    assert_eq!(seal::public_key(&hex).unwrap(), public);

    // the same functions in a document (the Key library), interchangeable with the host's
    let mut graph = Graph::default();
    graph.parse_stof_src(r#"
        fn check(message: str, key: str, signature: str) -> bool { Key::verify(key, 'orders/v1', message, signature) }
        fn check_blob(message: str, key: str, signature: str) -> bool { Key::verify(key, 'orders/v1', message as blob, signature) }
        fn sign(secret: str, message: str) -> str { Key::sign(secret, 'orders/v1', message) }
        fn public(secret: str) -> str { Key::public(secret) }
        fn round_trip() -> bool {
            const secret = Key::generate();
            const signature = Key::sign(secret, 'orders/v1', 'hello');
            Key::verify(Key::public(secret), 'orders/v1', 'hello', signature)
        }
    "#, None, Profile::default()).unwrap();
    let args = |message: &str| vec![Val::from(message), Val::from(public.as_str()), Val::from(signature.as_str())];
    assert_eq!(Runtime::call(&mut graph, "check", args(order)).unwrap(), Val::Bool(true), "the host's signature in a document");
    assert_eq!(Runtime::call(&mut graph, "check_blob", args(order)).unwrap(), Val::Bool(true));
    assert_eq!(Runtime::call(&mut graph, "check", args("{}")).unwrap(), Val::Bool(false));
    let made = Runtime::call(&mut graph, "sign", vec![Val::from(secret.as_str()), Val::from(order)]).unwrap().to_string();
    assert!(seal::verify(&public, "orders/v1", order.as_bytes(), &made), "the document's signature on the host");
    assert_eq!(Runtime::call(&mut graph, "public", vec![Val::from(secret.as_str())]).unwrap(), Val::from(public.as_str()));
    assert_eq!(Runtime::call(&mut graph, "round_trip", vec![]).unwrap(), Val::Bool(true));
}

#[test]
/// A document signs a part of itself; the host checks the signature when it seals it.
fn seal_signatures_from_stof() {
    let mut graph = doc();
    graph.parse_stof_src(r#"
        fn sign_tool(secret: str) -> str { Key::sign(secret, 'stof/seal', digest(self.tools.acme)) }
        fn sign_elsewhere(secret: str) -> str { Key::sign(secret, 'tools/v1', digest(self.tools.acme)) }
    "#, None, Profile::default()).unwrap();
    let secret = seal::generate_key();
    let public = seal::public_key(&secret).unwrap();
    let elsewhere = Runtime::call(&mut graph, "sign_elsewhere", vec![Val::from(secret.as_str())]).unwrap().to_string();
    assert!(graph.seal_signed("tools.acme", false, &public, &elsewhere).is_err(), "not under 'stof/seal'");
    let signature = Runtime::call(&mut graph, "sign_tool", vec![Val::from(secret.as_str())]).unwrap().to_string();
    graph.seal_signed("tools.acme", false, &public, &signature).unwrap();
    assert!(graph.is_sealed(&graph.seal_target("tools.acme").unwrap()));
}

#[test]
/// reads are paths: an object replaced at a path is the one the part reads, and a path with nothing there reads
/// nothing (until something is there).
fn reads_follow_paths() {
    let mut graph = doc();
    confined(&mut graph);
    assert_eq!(call(&mut graph, "tools.acme.run").unwrap(), Val::from("Boston"));
    graph.parse_stof_src("fn replace() { drop(root.inputs); root.inputs = new { city: 'Denver' } on root; }", None, Profile::default()).unwrap();
    call(&mut graph, "replace").unwrap();
    assert_eq!(call(&mut graph, "tools.acme.run").unwrap(), Val::from("Denver"), "the new object at the path");

    let mut graph = doc();
    graph.parse_stof_src(r#"
        tools: { acme: { fn later() -> unknown { root.later.x } } }
        fn make() { root.later = new { x: 1 } on root; }
    "#, None, Profile::default()).unwrap();
    graph.seal_path_with("tools.acme", &SealOptions { reads: Some(vec!["later".into()]), ..Default::default() }, None).unwrap();
    assert_eq!(call(&mut graph, "tools.acme.later").unwrap(), Val::Null, "nothing there yet");
    call(&mut graph, "make").unwrap();
    assert_eq!(call(&mut graph, "tools.acme.later").unwrap(), Val::from(1i64), "there now");
}

#[test]
fn seal_info() {
    let mut graph = doc();
    assert!(graph.seal_info_path("tools.acme").unwrap().is_none());
    graph.seal_path_with("tools.acme", &SealOptions {
        removable: true, reads: Some(vec!["inputs".into()]), hosts: Some(vec!["API.example.com".into()]), ..Default::default()
    }, None).unwrap();
    let info = graph.seal_info_path("tools.acme").unwrap().unwrap();
    assert!(info.removable && !info.fields);
    assert_eq!(info.confines.len(), 1);
    let json = info.to_json(&graph);
    assert_eq!(json["part"], "root.tools.acme");
    assert_eq!(json["confined"][0]["reads"][0], "inputs");
    assert_eq!(json["confined"][0]["hosts"][0], "api.example.com");

    // something inside the part: the part, not removable itself
    graph.parse_stof_src("tools: { acme: { inner: {} } }", None, Profile::default()).ok();
    let _ = graph.take_seal_violation();
    graph.seal_path("tools", true, None).unwrap();
    let info = graph.seal_info_path("tools.acme").unwrap().unwrap();
    assert!(!info.removable, "a seal around it holds it");
    assert!(info.fields);
    assert_eq!(info.to_json(&graph)["part"], "root.tools.acme", "the nearest part");

    // from Stof
    let mut graph = doc();
    graph.parse_stof_src(r#"
        fn info() -> unknown {
            seal(self.tools.acme, removable = true, reads = [self.inputs, 'secrets'], hosts = ['a.example.com']);
            sealed(self.tools.acme)
        }
        fn none() -> unknown { sealed(self.inputs) }
    "#, None, Profile::default()).unwrap();
    let info = call(&mut graph, "info").unwrap();
    let Val::Map(map) = info else { panic!("{info:?}") };
    let get = |key: &str| map.iter().find(|(k, _)| k.read().to_string() == key).map(|(_, v)| v.read().clone()).unwrap();
    assert_eq!(get("removable"), Val::Bool(true));
    assert_eq!(get("part"), Val::from("root.tools.acme"));
    let confined = get("confined").to_string();
    assert!(confined.contains("root.inputs") && confined.contains("secrets") && confined.contains("a.example.com"), "{confined}");
    assert_eq!(call(&mut graph, "none").unwrap(), Val::Null);
}

#[test]
/// A part's digest is the same in another document, wherever it's put (Ex. a provider's terms, sent to a consumer).
fn digests_travel() {
    let mut provider = Graph::default();
    provider.parse_stof_src(r#"
        manifest: {
            name: 'acme'
            plans: { pro: { price: 0.002, limit: 1000 }, free: { limit: 10 } }
            tags: ['weather', 'maps']
            fn estimate(units: int) -> float { units * self.plans.pro.price }
        }
    "#, None, Profile::default()).unwrap();
    let manifest = provider.seal_target("manifest").unwrap();
    let terms = provider.digest(&manifest, true).unwrap();
    let code = provider.digest(&manifest, false).unwrap();

    // as BSTF (functions too), into another document at another path
    let bytes = provider.binary_export("bstf", Some(manifest.clone())).unwrap();
    let mut consumer = Graph::default();
    consumer.parse_stof_src("providers: { acme: { terms: {} } }", None, Profile::default()).unwrap();
    let at = consumer.seal_target("providers.acme.terms").unwrap();
    consumer.binary_import("bstf", bytes, Some(at.clone()), &Profile::default()).unwrap();
    assert_eq!(consumer.digest(&at, false).unwrap(), code, "the same code");
    assert_eq!(consumer.digest(&at, true).unwrap(), terms, "the same terms");
    // using it doesn't change it
    assert_eq!(Runtime::call(&mut consumer, "providers.acme.terms.estimate", vec![Val::from(10i64)]).unwrap(), Val::from(0.02));
    assert_eq!(consumer.field_value("root.providers.acme.terms.plans.pro.limit", None), Some(Val::from(1000i64)));
    assert_eq!(consumer.digest(&at, true).unwrap(), terms, "after reading it");

    // data alone as JSON
    let json = provider.string_export("json", Some(provider.seal_target("manifest.plans").unwrap())).unwrap();
    let plans = provider.digest(&provider.seal_target("manifest.plans").unwrap(), true).unwrap();
    let mut other = Graph::default();
    other.parse_stof_src("offer: {}", None, Profile::default()).unwrap();
    let offer = other.seal_target("offer").unwrap();
    other.string_import("json", &json, Some(offer.clone()), &Profile::default()).unwrap();
    assert_eq!(other.digest(&offer, true).unwrap(), plans);
    assert_eq!(other.field_value("root.offer.pro.limit", None), Some(Val::from(1000i64)));
    assert_eq!(other.digest(&offer, true).unwrap(), plans, "after reading it (a field is made for pro)");
    assert_eq!(other.seal_with(&offer, &SealOptions { fields: true, ..Default::default() }).unwrap(), plans);

    // objects made with new (random names) digest by the fields that hold them
    let made = |graph: &mut Graph| {
        graph.parse_stof_src("fn make() { const o = new { a: new { x: 1 }, b: new { y: 2 } } on self; self.made = o; }", None, Profile::default()).unwrap();
        Runtime::call(graph, "make", vec![]).unwrap();
        let made = graph.seal_target("made").unwrap();
        graph.digest(&made, true).unwrap()
    };
    assert_eq!(made(&mut Graph::default()), made(&mut Graph::default()));
}

#[test]
/// A confined call that fails (caught or not) leaves nothing confined behind it.
fn errors_unwind_confinement() {
    let mut graph = doc();
    graph.parse_stof_src(r#"
        fn caught() -> str {
            try root.tools.acme.poke();
            catch {}
            root.secrets.key
        }
        fn nested_catch() -> str {
            let res = '';
            try { const city = root.tools.acme.run(); root.tools.acme.poke(); }
            catch { res = root.secrets.key; }
            res
        }
    "#, None, Profile::default()).unwrap();
    confined(&mut graph);
    assert!(call(&mut graph, "tools.acme.poke").is_err());
    assert_eq!(call(&mut graph, "Helpers.peek").unwrap(), Val::from("abc"), "after an uncaught failure");
    assert_eq!(call(&mut graph, "caught").unwrap(), Val::from("abc"), "after a caught failure, in the same call");
    assert_eq!(call(&mut graph, "nested_catch").unwrap(), Val::from("abc"));
    assert!(graph.confined.is_empty());
}

#[test]
/// Every export format refuses an object that holds something out of reach (not only JSON).
fn exports_hold_nothing_out_of_reach() {
    let mut graph = doc();
    graph.parse_stof_src(r#"
        tools: { acme: {
            fn export(format: str, o: obj) -> str { self.held = o; const s = stringify(format, self); self.held = null; s }
            fn blob(o: obj) -> blob { self.held = o; const b = blobify('bstf', self); self.held = null; b }
        } }
    "#, None, Profile::default()).unwrap();
    confined(&mut graph);
    let secrets = graph.seal_target("secrets").unwrap();
    for format in ["json", "yaml", "toml", "stof", "urlencoded"] {
        if !graph.formats.contains_key(format) { continue; }
        let res = Runtime::call(&mut graph, "tools.acme.export", vec![Val::from(format), Val::Obj(secrets.clone())]);
        assert!(res.as_ref().is_err_and(|err| err.to_string().contains("can't export")), "{format}: {res:?}");
    }
    let res = Runtime::call(&mut graph, "tools.acme.blob", vec![Val::Obj(secrets.clone())]);
    assert!(res.is_err_and(|err| err.to_string().contains("can't export")), "bstf");
    let inputs = graph.seal_target("inputs").unwrap();
    let res = Runtime::call(&mut graph, "tools.acme.export", vec![Val::from("json"), Val::Obj(inputs)]).unwrap();
    assert!(res.to_string().contains("held"), "without it, the export works");
}

#[test]
/// Seals are the host's, never stored: a document sent as BSTF arrives unsealed, and the receiver seals it (Ex. after
/// checking its digest).
fn seals_are_not_exported() {
    let mut graph = doc();
    confined(&mut graph);
    let acme = graph.seal_target("tools.acme").unwrap();
    let digest = graph.digest(&acme, false).unwrap();
    let bytes = graph.binary_export("bstf", None).unwrap();

    let mut received = Graph::default();
    received.binary_import("bstf", bytes, None, &Profile::default()).unwrap();
    assert!(received.seals.nodes.is_empty());
    assert_eq!(call(&mut received, "tools.acme.steal").unwrap(), Val::from("abc"), "unsealed on arrival");
    received.seal_path_with("tools.acme", &SealOptions { reads: Some(vec!["inputs".into()]), ..Default::default() }, Some(&digest)).unwrap();
    assert_eq!(call(&mut received, "tools.acme.steal").unwrap(), Val::Null);
    assert_eq!(call(&mut received, "tools.acme.run").unwrap(), Val::from("Boston"));
}

#[test]
#[cfg(feature = "http")]
/// A redirect away from a confined part's host: natively, the final URL is checked and the response withheld.
fn confined_redirects() {
    use std::io::{Read, Write};
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    std::thread::spawn(move || {
        for stream in listener.incoming().take(3) {
            let Ok(mut stream) = stream else { continue };
            let mut buf = [0u8; 2048];
            let n = stream.read(&mut buf).unwrap_or(0);
            let request = String::from_utf8_lossy(&buf[..n]).to_string();
            let response = if request.starts_with("GET /redirect") {
                format!("HTTP/1.1 302 Found\r\nLocation: http://localhost:{port}/\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
            } else {
                "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nhi".to_string()
            };
            let _ = stream.write_all(response.as_bytes());
        }
    });

    let mut graph = doc();
    graph.parse_stof_src("tools: { acme: { fn text(url: str) -> unknown { const res = await Http.fetch(url); if (res.get('error') != null) return res.get('error'); Http.text(res) } } }", None, Profile::default()).unwrap();
    graph.allow_http_hosts(["127.0.0.1", "localhost"]);
    graph.seal_path_with("tools.acme", &SealOptions { hosts: Some(vec!["127.0.0.1".into()]), ..Default::default() }, None).unwrap();
    let text = |graph: &mut Graph, path: &str| Runtime::call(graph, "tools.acme.text", vec![Val::from(format!("http://127.0.0.1:{port}{path}").as_str())]);

    let res = text(&mut graph, "/");
    assert_eq!(res.as_ref().map(|v| v.to_string()).ok(), Some("hi".to_string()), "its own host: {res:?}");
    let res = text(&mut graph, "/redirect").map(|v| v.to_string());
    assert!(res.as_ref().is_ok_and(|error| error.contains("redirected to") && error.contains("not an allowed host")), "redirected away (the document allows localhost: the part doesn't): {res:?}");
}

#[test]
/// Types are reached like objects: confined code can't make or cast to a type it isn't given (that would read its
/// defaults), and can't export a function value it was handed.
fn types_and_function_values_out_of_reach() {
    let mut graph = doc();
    graph.parse_stof_src(r#"
        #[type] Outside: { secret: 'type-default' }
        #[type] Inside: { v: 1 }
        tools: { acme: {
            fn new_outside() -> unknown { const o = new Outside {}; const s = o.secret; drop(o); s }
            fn cast_outside() -> unknown { const o = new {} on self; const t = o as Outside; t.secret }
            fn new_inside() -> unknown { const o = new Inside {}; const v = o.v; drop(o); v }
            fn keep(f: fn, format: str) -> unknown { const o = new {} on self; o.f = f; o.l = [f]; const s = stringify(format, o); drop(o); s }
            fn keep_b(f: fn) -> unknown { const o = new {} on self; o.f = f; const b = blobify('bstf', o); drop(o); b }
        } }
        fn give_bstf() -> unknown { root.tools.acme.keep_b(root.secret_fn) }
        fn give(format: str) -> unknown { root.tools.acme.keep(root.secret_fn, format) }
    "#, None, Profile::default()).unwrap();
    graph.seal_path_with("tools.acme", &SealOptions { reads: Some(vec!["<Inside>".into()]), ..Default::default() }, None).unwrap();
    for path in ["tools.acme.new_outside", "tools.acme.cast_outside"] {
        let res = call(&mut graph, path);
        assert!(res.as_ref().is_err_and(|err| err.contains("can't use the type 'Outside'")), "{path}: {res:?}");
    }
    assert_eq!(call(&mut graph, "tools.acme.new_inside").unwrap(), Val::from(1i64), "a type it reads");
    for format in ["stof", "json", "bstf"] {
        let res = if format == "bstf" {
            call(&mut graph, "give_bstf")
        } else {
            Runtime::call(&mut graph, "give", vec![Val::from(format)]).map_err(|e| e.to_string())
        };
        assert!(res.as_ref().is_err_and(|err| err.contains("can't export")), "{format}: {res:?}");
    }
}


#[test]
/// Lists and maps read from a field the code may not change are copies: changing them in place (push_back, insert,
/// swap, `+=`) never changes the field, and setting a guarded field is refused before anything is written.
fn in_place_changes() {
    let mut graph = Graph::default();
    graph.quiet(true);
    graph.parse_stof_src(r#"
        secrets: { key: 'abc' }
        inputs: { tags: ['a'] }
        frozen: { tags: ['a'] }
        tools: { acme: {
            mine: 1
            fn push() { root.inputs.tags.push_back('x'); }
            fn swapped() { swap(root.inputs.tags, self.mine); }
            fn added() { root.inputs.tags += ['z']; }
            fn by_ref() { const t = &root.inputs.tags; t.push_back('r'); }
            fn fields(o: obj) -> unknown { const { key } = o; key }
        } }
        fn frozen_push() { root.frozen.tags.push_back('x'); }
        fn frozen_set() { root.frozen.tags = ['b']; }
        fn fields() -> unknown { root.tools.acme.fields(root.secrets) }
    "#, None, Profile::default()).unwrap();
    graph.seal_path_with("frozen", &SealOptions { fields: true, ..Default::default() }, None).unwrap();
    graph.seal_path_with("tools.acme", &SealOptions { reads: Some(vec!["inputs".into()]), ..Default::default() }, None).unwrap();

    for func in ["tools.acme.push", "tools.acme.swapped", "tools.acme.by_ref", "frozen_push"] {
        assert!(Runtime::call(&mut graph, func, vec![]).is_ok(), "{func} changes a copy");
    }
    assert!(Runtime::call(&mut graph, "tools.acme.added", vec![]).is_err());
    assert!(Runtime::call(&mut graph, "frozen_set", vec![]).is_err());
    assert!(Runtime::call(&mut graph, "fields", vec![]).unwrap_err().to_string().contains("outside what the part may reach"));

    let tags = |graph: &mut Graph, path: &str| graph.field_value(path, None).map(|val| match val { Val::List(items) => items.len(), _ => 0 });
    assert_eq!(tags(&mut graph, "root.inputs.tags"), Some(1));
    assert_eq!(tags(&mut graph, "root.frozen.tags"), Some(1));
    assert!(!graph.set_field(Variable::val(Val::List(Default::default())), "root.frozen.tags", None), "the host is refused too");
    assert_eq!(tags(&mut graph, "root.frozen.tags"), Some(1));
}

