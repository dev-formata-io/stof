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

//! Parse info for editor tooling (Ex. a language server): where everything is defined, and warnings with
//! locations. Only collected by `parse_with_info`; normal parsing doesn't record anything.

use rustc_hash::FxHashMap;
use crate::{model::{DataRef, Graph, NodeRef, Profile}, parser::{context::{resolve_import_path, ParseContext}, source::SrcLoc}, runtime::Error};


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefKind {
    /// `root Name { .. }`
    Root,
    /// A field (its value might be an object, see `Def::obj`).
    Field,
    /// A function.
    Func,
    /// An import statement (see `Def::target`).
    Import,
}


#[derive(Debug, Clone)]
/// A definition in a Stof document.
pub struct Def {
    pub kind: DefKind,
    /// Field, function, or root name (import: the path as written).
    pub name: String,
    /// Object it is defined on (what `self` is for it).
    pub parent: NodeRef,
    /// Field or function data.
    pub data: Option<DataRef>,
    /// Object value of a field, or the root node.
    pub obj: Option<NodeRef>,
    /// Import: the resolved file path.
    pub target: Option<String>,
    /// Where the name is (import: the path string).
    pub start: SrcLoc,
    /// Where the definition ends (after its value or body).
    pub end: Option<SrcLoc>,
    /// Doc comments.
    pub docs: String,
    /// Byte offsets in the text it was parsed from (fields and functions only).
    pub span: Option<DefSpan>,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Where a field or function is in its text, as byte offsets (see `parser::edit`).
pub struct DefSpan {
    /// Where the statement starts: right after the statement before it, so it includes the whitespace, comments,
    /// and attributes before the definition.
    pub outer_start: usize,
    /// The name (with its quotes, if quoted).
    pub name_start: usize,
    pub name_end: usize,
    /// The value: a field's value after the `:`, or a function from `fn` (or `async`) to the end of its body.
    pub value_start: usize,
    pub value_end: usize,
    /// After the value and its separator (`,` or `;`), if it has one.
    pub outer_end: usize,
}


#[derive(Debug, Clone)]
/// A parse warning.
pub struct Warning {
    pub message: String,
    /// Where it is, when known.
    pub loc: Option<SrcLoc>,
    /// File being parsed when it was found.
    pub file: Option<String>,
}


#[derive(Debug, Clone, Default)]
/// Everything recorded while parsing with `parse_with_info`.
pub struct ParseInfo {
    pub defs: Vec<Def>,
    pub warnings: Vec<Warning>,
    /// Files parsed (resolved paths), in order.
    pub files: Vec<String>,
    /// Stof texts parsed instead of the files at these paths (see `parse_with_info`).
    pub overlay: FxHashMap<String, String>,
    /// Only the definitions matter (see `parser::edit`): nothing in the text runs (field expressions and attribute
    /// values aren't evaluated, init functions aren't called), and typed fields aren't cast, so a text can use types
    /// it doesn't define (Ex. `Limitr policy: {..}` without the engine).
    pub outline_only: bool,
}


/// Parse a file, recording definitions and warnings: a Stof file ("stof"), or a package ("pkg": a pkg.stof
/// file, or a directory with one) to see a file in the context of its whole project.
/// `overlay` texts are parsed instead of the Stof files at those paths (Ex. editor buffers that aren't saved).
/// Init functions are not run. On a parse error, the info has everything before the error.
pub fn parse_with_info(graph: &mut Graph, format: &str, path: &str, overlay: FxHashMap<String, String>, profile: Profile) -> (Result<(), Error>, ParseInfo) {
    // same path form as imports (Ex. "a/./b" is "a/b")
    let overlay = overlay.into_iter().map(|(path, text)| {
        let resolved = resolve_import_path(None, "", &path).ok().and_then(|path| path.into_os_string().into_string().ok());
        (resolved.unwrap_or(path), text)
    }).collect();

    let mut context = ParseContext::new(graph, profile);
    context.info = Some(Box::new(ParseInfo { overlay, ..Default::default() }));
    let res = context.parse_from_file(format, path, None);

    let mut info = context.info.take().map(|info| *info).unwrap_or_default();
    info.overlay.clear();
    context.warnings.clear(); // in the info instead of stderr
    context.init_funcs.clear(); // don't run code in an editor
    (res, info)
}


/// Parse one Stof text (no file), recording its definitions and warnings, with the object its top-level
/// definitions are on. Init functions are not run. Used by `parser::edit`.
pub fn parse_text_with_info(graph: &mut Graph, text: &str, profile: Profile) -> (Result<(), Error>, ParseInfo, NodeRef) {
    let mut context = ParseContext::new(graph, profile);
    context.info = Some(Box::new(ParseInfo { outline_only: true, ..Default::default() }));
    let top = context.self_ptr();
    let res = crate::parser::doc::document(text, &mut context);
    let info = context.info.take().map(|info| *info).unwrap_or_default();
    context.warnings.clear();
    context.init_funcs.clear();
    (res, info, top)
}


#[cfg(test)]
mod tests {
    use rustc_hash::FxHashMap;
    use crate::{model::{Graph, Profile}, parser::info::{parse_with_info, DefKind}};

    /// A path in the form the parser uses (on Windows, "/proj/main.stof" is "\proj\main.stof").
    fn normal(path: &str) -> String {
        crate::parser::context::resolve_import_path(None, "", path).unwrap().to_string_lossy().to_string()
    }

    fn overlay(path: &str, text: &str) -> FxHashMap<String, String> {
        let mut overlay = FxHashMap::default();
        overlay.insert(path.to_string(), text.to_string());
        overlay
    }

    #[test]
    fn records_defs() {
        let text = r#"
import './missing_is_an_error_later' as self.Other;
/// The user.
#[type]
User: {
    str name: 'Bob'
    "full name": 'Bob Smith';

    /** Greet. */
    fn greet() -> str { `Hi ${self.name}` }
}
root Extra {
    count: 1
}
"#;
        let mut graph = Graph::default();
        let (res, info) = parse_with_info(&mut graph, "stof", "/proj/main.stof", overlay("/proj/main.stof", text), Profile::default());
        assert!(res.is_err()); // the import is missing

        // nothing after the error (the import is first)
        assert!(info.defs.is_empty());

        let text = text.replace("import './missing_is_an_error_later' as self.Other;", "");
        let mut graph = Graph::default();
        let (res, info) = parse_with_info(&mut graph, "stof", "/proj/main.stof", overlay("/proj/./main.stof", &text), Profile::default());
        assert!(res.is_ok());
        assert_eq!(info.files, vec![normal("/proj/main.stof")]);

        let found: Vec<(DefKind, &str, u32, u32)> = info.defs.iter().map(|def| (def.kind, def.name.as_str(), def.start.line, def.start.col)).collect();
        assert_eq!(found, vec![
            (DefKind::Field, "name", 6, 9),
            (DefKind::Field, "full name", 7, 5),
            (DefKind::Func, "greet", 10, 8),
            (DefKind::Field, "User", 5, 1),
            (DefKind::Field, "count", 13, 5),
            (DefKind::Root, "Extra", 12, 1),
        ]);
        let user = &info.defs[3];
        assert_eq!(user.docs.trim(), "The user.");
        assert_eq!(user.end.as_ref().map(|end| (end.line, end.col)), Some((11, 2)));
        assert!(user.obj.is_some() && user.data.is_some());
        assert_eq!(info.defs[2].docs.trim(), "Greet.");
        assert_eq!(info.defs[0].start.file.as_deref(), Some(normal("/proj/main.stof").as_str()));
    }

    #[test]
    #[cfg(feature = "system")]
    fn records_imports() {
        let dir = std::env::temp_dir().join(format!("stof_info_{}", nanoid::nanoid!(8)));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("other.stof"), "fn helper() -> int { 1 }\n").unwrap();
        let main = dir.join("main.stof").to_string_lossy().to_string();

        let mut graph = Graph::default();
        graph.allow_system();
        let (res, info) = parse_with_info(&mut graph, "stof", &main, overlay(&main, "import './other' as self.Other;\n"), Profile::default());
        std::fs::remove_dir_all(&dir).ok();
        assert!(res.is_ok(), "{res:?}");

        let other = dir.join("other.stof").to_string_lossy().to_string();
        assert_eq!(info.defs.len(), 2);
        let helper = &info.defs[0];
        assert_eq!((helper.kind, helper.name.as_str()), (DefKind::Func, "helper"));
        assert_eq!(helper.start.file.as_deref(), Some(other.as_str()));
        let import = &info.defs[1];
        assert_eq!((import.kind, import.name.as_str()), (DefKind::Import, "./other"));
        assert_eq!((import.start.line, import.start.col), (1, 8));
        assert_eq!(import.target.as_deref(), Some(other.as_str()));
    }

    #[test]
    #[cfg(all(feature = "system", feature = "pkg"))]
    /// A file that uses types from other files in its package: parsed with the package (and an unsaved edit).
    fn package_context() {
        let dir = std::env::temp_dir().join(format!("stof_info_{}", nanoid::nanoid!(8)));
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(dir.join("pkg.stof"), "import: 'src/mod.stof'\n").unwrap();
        std::fs::write(dir.join("src/mod.stof"), "import './types';\nimport './user';\n").unwrap();
        std::fs::write(dir.join("src/types.stof"), "#[type]\nPoint: { x: 0 }\n").unwrap();
        std::fs::write(dir.join("src/user.stof"), "Point home: {}\n").unwrap();
        let user = dir.join("src").join("user.stof").to_string_lossy().to_string();
        let pkg = dir.join("pkg.stof").to_string_lossy().to_string();

        // alone, the type isn't known
        let mut graph = Graph::default();
        graph.allow_system();
        let (res, _) = parse_with_info(&mut graph, "stof", &user, FxHashMap::default(), Profile::default());
        assert!(res.is_err());

        // with its package (and an unsaved edit), it is
        let mut graph = Graph::default();
        graph.allow_system();
        let (res, info) = parse_with_info(&mut graph, "pkg", &pkg, overlay(&user, "Point home: {}\nPoint work: {}\n"), Profile::default());
        std::fs::remove_dir_all(&dir).ok();
        assert!(res.is_ok(), "{res:?}");
        assert!(info.files.contains(&user));
        let names: Vec<&str> = info.defs.iter().filter(|def| def.start.file.as_deref() == Some(user.as_str())).map(|def| def.name.as_str()).collect();
        assert_eq!(names, vec!["home", "work"]);
    }

    #[test]
    fn records_warnings() {
        let text = "fn main() {\n    let x = 1;\n    pln(y);\n}\n";
        let mut graph = Graph::default();
        let (res, info) = parse_with_info(&mut graph, "stof", "/main.stof", overlay("/main.stof", text), Profile::default());
        assert!(res.is_ok());
        assert_eq!(info.warnings.len(), 1);
        let warning = &info.warnings[0];
        assert!(warning.message.contains("unknown name 'y'"), "{}", warning.message);
        assert_eq!(warning.loc.as_ref().map(|loc| (loc.line, loc.col)), Some((3, 9)));
    }
}
