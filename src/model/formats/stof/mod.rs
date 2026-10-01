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

use std::fs;
use rustc_hash::FxHashSet;
use crate::{model::{FS_LIB, Format, Graph, NodeRef, Profile, stof::export::StofExportContext}, parser::{context::ParseContext, doc::document}, runtime::Error};
mod export;


#[derive(Debug, Default)]
/// Stof language format.
pub struct StofFormat;
impl Format for StofFormat {
    fn identifiers(&self) -> Vec<String> {
        vec!["stof".into(), "stof:human".into()]
    }
    fn content_type(&self) -> String {
        "application/stof".into()
    }
    fn string_import(&self, graph: &mut Graph, _format: &str, src: &str, node: Option<NodeRef>, profile: &Profile) -> Result<(), Error> {
        if src.is_empty() { return Ok(()); }
        let mut context = ParseContext::new(graph, profile.clone());
        if let Some(node) = node {
            context.push_self_node(node);
        }
        document(src, &mut context)?;
        Ok(())
    }
    fn file_import(&self, graph: &mut Graph, format: &str, path: &str, node: Option<NodeRef>, profile: &Profile) -> Result<(), Error> {
        let mut context = ParseContext::new(graph, profile.clone());
        context.parse_from_file(format, path, node)
    }
    fn parser_import(&self, _format: &str, path: &str, context: &mut ParseContext) -> Result<(), Error> {
        if let Some(_lib) = context.graph.libfunc(&FS_LIB, "read_string") {
            #[cfg(not(feature = "system"))]
            {
                use imbl::vector;
                use std::sync::Arc;
                use crate::{runtime::{Val, instruction::Instruction, instructions::{Base, call::FuncCall}}};

                let ins: Arc<dyn Instruction> = Arc::new(FuncCall {
                    func: None,
                    search: Some("fs::read_string".into()), // explicit library: a "fs" root can't intercept it
                    stack: false,
                    as_ref: false,
                    cnull: false,
                    args: vector![Arc::new(Base::Literal(Val::Str(path.into()))) as Arc<dyn Instruction>],
                    oself: None,
                });
                match context.eval(ins) {
                    Ok(res) => {
                        match res {
                            Val::Str(src) => {
                                if !src.is_empty() {
                                    document(&src, context)?;
                                }
                                return Ok(());
                            },
                            _ => {
                                // Try FS
                            }
                        }
                    },
                    Err(_error) => {
                        // Try FS
                    }
                }
            }

            match fs::read(path) {
                Ok(content) => {
                    match std::str::from_utf8(&content) {
                        Ok(src) => {
                            if !src.is_empty() {
                                document(src, context)?;
                            }
                            return Ok(());
                        },
                        Err(_error) => {
                            return Err(Error::FormatBinaryImportUtf8Error);
                        }
                    }
                },
                Err(error) => {
                    return Err(Error::FormatFileImportFsError(format!("{}: {}", error.to_string(), path)));
                }
            }
        } else if let Some(_lib) = context.graph.libfunc(&FS_LIB, "read") {
            match fs::read(path) {
                Ok(content) => {
                    match std::str::from_utf8(&content) {
                        Ok(src) => {
                            if !src.is_empty() {
                                document(src, context)?;
                            }
                            return Ok(());
                        },
                        Err(_error) => {
                            return Err(Error::FormatBinaryImportUtf8Error);
                        }
                    }
                },
                Err(error) => {
                    return Err(Error::FormatFileImportFsError(format!("{}: {}", error.to_string(), path)));
                }
            }
        }
        Err(Error::FormatFileImportNotAllowed)
    }
    fn string_export(&self, graph: &Graph, format: &str, node: Option<NodeRef>) -> Result<String, Error> {
        let mut context = StofExportContext::default();
        context.human = format.contains("human");
        let mut seen = FxHashSet::default();
        if let Some(node) = node {
            context.export_node(graph, &node, &mut seen);
        } else {
            for root in &graph.roots {
                context.export_node(graph, root, &mut seen);
            }
        }
        Ok(context.stof)
    }
}


#[derive(Debug)]
/// .bstf format (serialized graph)
pub struct BstfFormat;
impl Format for BstfFormat {
    fn identifiers(&self) -> Vec<String> {
        vec!["bstf".into()]
    }
    fn content_type(&self) -> String {
        "application/bstf".into()
    }
    fn binary_export(&self, graph: &Graph, _format: &str, node: Option<NodeRef>) -> Result<bytes::Bytes, Error> {
        if let Some(node) = node {
            let mut context = FxHashSet::default();
            context.insert(node);
            let graph = graph.context_clone(context);
            match bincode::serialize(&graph) {
                Ok(bytes) => {
                    Ok(bytes.into())
                },
                Err(error) => {
                    Err(Error::BSTFExport(error.to_string()))
                }
            }
        } else {
            match bincode::serialize(graph) {
                Ok(bytes) => {
                    Ok(bytes.into())
                },
                Err(error) => {
                    Err(Error::BSTFExport(error.to_string()))
                }
            }
        }
    }
    fn binary_import(&self, graph: &mut Graph, _format: &str, bytes: bytes::Bytes, node: Option<NodeRef>, _profile: &Profile) -> Result<(), Error> {
        if bytes.is_empty() { return Ok(()); }
        match crate::model::cautious::bincode_deserialize::<Graph>(bytes.as_ref()) {
            Ok(mut imported) => {
                // Insert types
                for (k, v) in &imported.typemap {
                    for nref in v {
                        graph.insert_type(k, nref);
                    }
                }

                if let Some(node) = node {
                    // absorb the main root onto this graph node
                    if let Some(main) = imported.ensure_main_root().node(&imported) {
                        graph.absorb_external_node(&imported, main, &node, true);
                    }
                } else {
                    // insert all roots into the graph
                    for import_root in &imported.roots {
                        if let Some(import_root_name) = import_root.node_name(&imported) {
                            if let Some(existing_root) = graph.find_root_named(&import_root_name) {
                                if let Some(import_root_node) = import_root.node(&imported) {
                                    graph.absorb_external_node(&imported, import_root_node, &existing_root, true);
                                }
                            } else if let Some(import_root_node) = import_root.node(&imported) {
                                graph.insert_external_node(&imported, import_root_node, None, None, None);
                            }
                        }
                    }
                }
                Ok(())
            },
            Err(error) => {
                Err(Error::BSTFImport(error.to_string()))
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use colored::Colorize;
    use crate::{model::{Graph, Profile}, parser::{context::ParseContext, doc::document}};

    #[test]
    fn stof_suite() {
        let mut graph = Graph::default();
        match graph.parse_stof_file("stof", "src/model/formats/stof/tests/tests.stof", None, Profile::test()) {
            Ok(_) => {},
            Err(error) => {
                panic!("{} @ {}", "Stof Suite Parse Error".red(), error);
            }
        }
        let res = graph.test(None, true);
        match res {
            Ok(res) => println!("{res}"),
            Err(err) => panic!("{err}")
        }
    }

    #[test]
    /// #[main] (and #[test]) reporting must name every finished function, including ones that loop.
    fn main_reports_functions_that_loop() {
        let mut graph = Graph::default();
        graph.parse_stof_src(r#"
            #[main]
            fn straight_line() -> int { 1 }

            #[main]
            fn looping() -> int {
                let total = 0;
                for (const i in 3) total += i;
                total
            }

            #[main]
            fn calls_a_loop() -> int { self.helper() }
            fn helper() -> int { let t = 0; while (t < 4) t += 1; t }
        "#, None, Profile::default()).expect("parses");
        let output = graph.run(None, true).expect("runs");
        for name in ["straight_line", "looping", "calls_a_loop"] {
            assert!(output.contains(name), "missing a report line for {name}:\n{output}");
        }
    }

    #[test]
    /// Parsing a string with no target node must keep sibling objects as siblings.
    fn string_parse_keeps_root_objects_as_siblings() {
        let mut graph = Graph::default();
        graph.parse_stof_src("a: { x: 1 }\nb: { y: 2 }\nfn f() -> int { 1 }", None, Profile::default()).expect("parses");
        let root = graph.main_root().expect("main root");
        let names = root.node(&graph).expect("root node").children.iter()
            .filter_map(|child| child.node_name(&graph).map(|name| name.as_ref().to_string()))
            .collect::<Vec<_>>();
        assert_eq!(names, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    /// Name.func() looks for an object first, then the library; Name::func() is always the library.
    fn library_calls_with_root_objects() {
        let mut graph = Graph::default();
        graph.parse_stof_src(r#"
            root Num { x: 1 }                                     // no "round": falls back to the library
            root Str { fn upper(v: str) -> str { return 'mine'; } } // intentional override
            Time: { fn now() -> str { 'top-level object' } }      // not a root: never consulted

            #[main]
            fn resolution() -> bool {
                assert_eq(Num.round(2.567, 1), 2.6);               // object lacks it: library
                assert_eq(Str.upper('abc'), 'mine');               // object defines it: object
                assert_eq(Str::upper('abc'), 'ABC');               // explicit: always the library
                assert_eq(Num::round(-2.567, 1).abs(), 2.6);       // chains
                assert_eq(Num.name(), 'Num');                      // Obj library on the root still works
                assert(Time.now() != 'top-level object');           // top-level objects don't shadow
                assert_eq(self.Time.now(), 'top-level object');
                let failed = false;
                try { Num::nope(1); } catch { failed = true; }
                assert(failed);                                    // missing library function errors
                true
            }
        "#, None, Profile::default()).expect("parses");
        let output = graph.run(None, true).expect("library resolution");
        assert!(output.contains("resolution"), "{output}");
    }

    #[test]
    /// A path without self/super is absolute: its first segment names a graph root ("root.a.b", or
    /// "Other.z" for a "root Other {..}"). Objects are never found by name alone.
    fn paths_without_self_are_absolute() {
        let mut graph = Graph::default();
        graph.parse_stof_src(r#"
            top: { x: 1  fn f() -> int { 10 } }
            root Other { z: 5  fn h() -> int { 30 } }
            outer: {
                inner: { y: 2  fn g() -> int { 20 } }
                Num: { fn round(v: float, places: int) -> str { 'nested' } }
            }

            #[main]
            fn paths() -> bool {
                assert_eq(root.top.x, 1);                   // full path from the main root
                assert_eq(root.top.f(), 10);
                assert_eq(self.top.x, 1);                   // relative
                assert_eq(root.outer.inner.y, 2);
                assert_eq(Other.z, 5);                      // another graph root
                assert_eq(Other.h(), 30);
                assert_eq(top.x, null);                     // objects aren't found by name alone
                assert_eq(inner.y, null);
                let failed = false;
                try { inner.g(); } catch { failed = true; }
                assert(failed);
                assert_eq(Num.round(2.567, 1), 2.6);        // an object named "Num" can't take over the library
                assert_eq(root.outer.Num.round(2.567, 1), 'nested');
                true
            }
        "#, None, Profile::default()).expect("parses");
        let output = graph.run(None, true).expect("absolute paths");
        assert!(output.contains("paths"), "{output}");
    }

    #[test]
    /// Objects and variables that shadow a library for Name.func() calls produce parse warnings.
    fn warns_on_library_shadowing() {
        let mut graph = Graph::default();
        let mut context = ParseContext::new(&mut graph, Profile::prod());
        document(r#"
            root Num { x: 1 }                // warn: a root named like a library
            Data: { x: 1 }                   // top-level object, not a root: no warning
            Time: 42                         // field value, not an object: no warning
            nested: {
                Str: { x: 1 }                // nested: no warning
                Map: {                       // nested: can't be found by name alone, so no warning
                    fn keys() -> list { [] }
                }
            }
            fn uses(Blob: int) -> int {      // warn: param
                let Set = 1;                 // warn: local
                const total = Blob + Set;
                let fine = 2;                // no warning
                total + fine
            }
            #[test]
            fn excluded() { let List = 1; }  // not created under prod: no warning
        "#, &mut context).expect("parses");
        let warnings = context.take_warnings();
        drop(context);

        let warned = |name: &str| warnings.iter().any(|w| w.contains(&format!("'{name}'")));
        assert!(warned("Num"), "{warnings:?}");
        assert!(warned("Blob"), "{warnings:?}");
        assert!(warned("Set"), "{warnings:?}");
        for quiet in ["Time", "root.Data", "root.nested.Str", "root.nested.Map", "fine", "total", "List"] {
            assert!(!warned(quiet), "unexpected warning for {quiet}: {warnings:?}");
        }
        assert_eq!(warnings.len(), 3, "{warnings:?}");
    }

    #[test]
    /// Bare names that can't be a variable, root, or std function are always null: warn where they are.
    fn warns_on_unknown_names() {
        let mut graph = Graph::default();
        let mut context = ParseContext::new(&mut graph, Profile::prod());
        document(r#"
            config: { rate: 4 }
            fn total(items: list) -> int {
                let sum = 0;
                for (const item in items) { sum += item * index; }   // loop variables are known
                try { sum += 1; } catch (err: str) { pln(err); }     // catch variable
                const double = (x: int): int => x * 2 + sum;         // arrow params & enclosing locals
                NewRoot = new root {};                               // assignment creates the name
                const r = Other.x + <Cfg>.v + Num.abs(-1) + max(1, 2) + ?maybe;
                sum + totl + double(1)                               // warn: totl
            }
            fn calls() { helper(); self.total([]); }                 // warn: helper (not std)
            root Other { x: 1 }                                      // roots declared later are fine
            #[type] Cfg: { v: 1 }
        "#, &mut context).expect("parses");
        let warnings = context.take_warnings();
        drop(context);

        assert!(warnings.iter().any(|w| w.contains("unknown name 'totl' in fn root.total (line") || w.contains("unknown name 'totl' in fn root.total (10:")), "{warnings:?}");
        assert!(warnings.iter().any(|w| w.contains("unknown function 'helper' in fn root.calls")), "{warnings:?}");
        assert_eq!(warnings.len(), 2, "{warnings:?}");
    }

    #[test]
    fn stof_docs() {
        let mut graph = Graph::default();
        graph.insert_lib_docs();

        // For testing purposes, document the test suite...
        //graph.parse_stof_file("stof", "src/model/formats/stof/tests/tests.stof", None, true).unwrap();

        graph.docs("docs/libs", None).unwrap();
    }
}
