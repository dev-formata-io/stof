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

//! Edit Stof text in place: set, add, remove, and rename fields and functions by path, without rewriting the rest
//! of the text. Comments, formatting, and everything an edit doesn't name stay exactly as they were.
//!
//! `outline` lists a text's fields and functions with their paths and where they are, from the real parser (the
//! same information editor tooling uses). `edit` applies edits one at a time, checking that the text still parses
//! after each one.
//!
//! Paths start at the document's top level (Ex. `["policy", "plans", "pro", "label"]`, or "policy.plans.pro.label"
//! when no name has a dot). Only objects written inline (`name: { .. }`) have children; a field whose value is a
//! reference or an expression is a value.

use rustc_hash::{FxHashMap, FxHashSet};
use serde_json::Value as Json;
use crate::{model::{Graph, NodeRef, Profile}, parser::info::{parse_text_with_info, DefKind, DefSpan}, runtime::Error};


#[derive(Debug, Clone, PartialEq, Eq)]
/// A field or function in a Stof text.
pub struct OutlineDef {
    /// Field or Func.
    pub kind: DefKind,
    /// Path from the top level, ending with the name.
    pub path: Vec<String>,
    /// A field whose value is an object written inline (it can have children).
    pub object: bool,
    /// Byte offsets in the text.
    pub span: DefSpan,
    /// Where the name is (1-based line, byte column).
    pub line: u32,
    pub col: u32,
}
impl OutlineDef {
    pub fn name(&self) -> &str {
        self.path.last().map(|name| name.as_str()).unwrap_or("")
    }
}


/// The fields and functions in a Stof text, in the order they're written (an object before its children).
/// Errors if the text doesn't parse. Definitions inside functions, `new` objects, or imports aren't listed.
/// Nothing in the text runs: field expressions and attribute values aren't evaluated, and init functions aren't
/// called, so any text is safe to outline (Ex. one a user sent).
pub fn outline(text: &str) -> Result<Vec<OutlineDef>, Error> {
    let mut graph = Graph::default();
    let (res, info, top) = parse_text_with_info(&mut graph, text, Profile::default());
    res?;

    // objects written inline: object node -> (object it's on, its name)
    let mut objects: FxHashMap<NodeRef, (NodeRef, String)> = FxHashMap::default();
    let mut roots: FxHashMap<NodeRef, String> = FxHashMap::default();
    for def in &info.defs {
        if def.start.file.is_some() { continue; } // imported
        match def.kind {
            DefKind::Field => {
                let (Some(obj), Some(span)) = (&def.obj, &def.span) else { continue; };
                if !text[span.value_start..].starts_with('{') { continue; }
                let direct = obj.node(&graph).and_then(|node| node.parent.clone()).map(|parent| parent == def.parent).unwrap_or(false);
                if direct { objects.insert(obj.clone(), (def.parent.clone(), def.name.clone())); }
            },
            DefKind::Root => {
                if let Some(obj) = &def.obj { roots.insert(obj.clone(), def.name.clone()); }
            },
            _ => {},
        }
    }
    fn path_of(node: &NodeRef, top: &NodeRef, objects: &FxHashMap<NodeRef, (NodeRef, String)>, roots: &FxHashMap<NodeRef, String>, depth: usize) -> Option<Vec<String>> {
        if node == top { return Some(Vec::new()); }
        if depth > 512 { return None; }
        if let Some(name) = roots.get(node) { return Some(vec![name.clone()]); }
        let (parent, name) = objects.get(node)?;
        let mut path = path_of(parent, top, objects, roots, depth + 1)?;
        path.push(name.clone());
        Some(path)
    }

    let mut out = Vec::new();
    for def in &info.defs {
        if def.start.file.is_some() { continue; }
        if !matches!(def.kind, DefKind::Field | DefKind::Func) { continue; }
        let Some(span) = def.span else { continue; };
        let Some(mut path) = path_of(&def.parent, &top, &objects, &roots, 0) else { continue; };
        path.push(def.name.clone());
        let object = def.kind == DefKind::Field && def.obj.as_ref().map(|obj| objects.contains_key(obj)).unwrap_or(false);
        out.push(OutlineDef { kind: def.kind, path, object, span, line: def.start.line, col: def.start.col });
    }
    // defs are recorded after their values (children first): sort into text order
    out.sort_by_key(|def| def.span.name_start);
    Ok(out)
}


/// An outline as JSON (Ex. for JavaScript or Python): one object per definition with `kind` ("field" or "fn"),
/// `path`, `name`, `object`, `line`, `col`, `value` (a field's value, or a function from `fn` to the end of its
/// body), and `source` (the whole definition with its attributes and the comments directly above it). Both are as
/// if written at column 0.
pub fn outline_json(text: &str, defs: &[OutlineDef]) -> Json {
    Json::Array(defs.iter().map(|def| {
        let indent = line_indent(text, def.span.name_start);
        serde_json::json!({
            "kind": if def.kind == DefKind::Func { "fn" } else { "field" },
            "path": def.path,
            "name": def.name(),
            "object": def.object,
            "line": def.line,
            "col": def.col,
            "value": dedent(&text[def.span.value_start..def.span.value_end], &indent),
            "source": dedent(&text[attached_start(text, def)..def.span.value_end], &indent),
        })
    }).collect())
}

/// `edit` with edits as JSON text (see `edits_from_json`).
pub fn edit_json(text: &str, edits: &str) -> Result<String, Error> {
    let edits: Json = serde_json::from_str(edits).map_err(|error| fail(&format!("edits aren't valid JSON: {error}")))?;
    edit(text, &edits_from_json(&edits)?)
}


#[derive(Debug, Clone, PartialEq)]
/// An edit to a Stof text.
pub enum Edit {
    /// Set a field's value (Stof source for the value), adding the field (and any objects on its path) if it
    /// isn't there.
    Set { path: Vec<String>, value: String },
    /// Set a function (Stof source starting with `fn` or `async fn`), adding it if it isn't there. An existing
    /// function keeps its attributes and doc comments.
    SetFn { path: Vec<String>, source: String },
    /// Remove a field or function, with its attributes and the comments directly above it.
    Remove { path: Vec<String> },
    /// Rename a field or function.
    Rename { path: Vec<String>, to: String },
}


/// Apply edits to a Stof text, in order. The rest of the text is unchanged. Errors (and nothing is applied) if the
/// text doesn't parse, an edit doesn't make sense (Ex. removing something that isn't there), or the result
/// doesn't parse.
pub fn edit(text: &str, edits: &[Edit]) -> Result<String, Error> {
    let mut current = text.to_string();
    let mut defs = outline(&current)?;
    for edit in edits {
        current = apply(&current, &defs, edit)?;
        defs = outline(&current).map_err(|error| fail(&format!("the text doesn't parse after {}: {}", describe(edit), error.to_string())))?;
    }
    Ok(current)
}


/// Edits from JSON (Ex. from JavaScript or Python), one object per edit:
/// `{ "set": path, "value": <JSON value> }`, `{ "set": path, "stof": "<Stof value>" }`,
/// `{ "set_fn": path, "stof": "fn .." }`, `{ "remove": path }`, `{ "rename": path, "to": "name" }`.
/// A path is a list of names, or a string of names separated by dots.
pub fn edits_from_json(edits: &Json) -> Result<Vec<Edit>, Error> {
    let Json::Array(items) = edits else { return Err(fail("edits must be a list")); };
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        let Json::Object(map) = item else { return Err(fail("each edit must be an object")); };
        let path_of = |key: &str| -> Result<Vec<String>, Error> {
            match map.get(key) {
                Some(Json::String(path)) => Ok(path.split('.').map(|name| name.to_string()).collect()),
                Some(Json::Array(names)) => names.iter().map(|name| match name {
                    Json::String(name) => Ok(name.clone()),
                    _ => Err(fail("path names must be strings")),
                }).collect(),
                _ => Err(fail(&format!("'{key}' must be a path"))),
            }
        };
        let stof = || -> Option<String> { map.get("stof").and_then(|value| value.as_str()).map(|value| value.to_string()) };
        let edit = if map.contains_key("set") {
            let path = path_of("set")?;
            let value = match (stof(), map.get("value")) {
                (Some(stof), _) => stof,
                (None, Some(value)) => json_value(value, ""),
                (None, None) => return Err(fail("set needs a 'value' or 'stof'")),
            };
            Edit::Set { path, value }
        } else if map.contains_key("set_fn") {
            let Some(source) = stof() else { return Err(fail("set_fn needs 'stof'")); };
            Edit::SetFn { path: path_of("set_fn")?, source }
        } else if map.contains_key("remove") {
            Edit::Remove { path: path_of("remove")? }
        } else if map.contains_key("rename") {
            let Some(to) = map.get("to").and_then(|to| to.as_str()) else { return Err(fail("rename needs 'to'")); };
            Edit::Rename { path: path_of("rename")?, to: to.to_string() }
        } else {
            return Err(fail("unknown edit (use set, set_fn, remove, or rename)"));
        };
        out.push(edit);
    }
    Ok(out)
}


/// A JSON value as Stof source, at an indent (pretty when it has objects or lists of objects).
pub fn json_value(value: &Json, indent: &str) -> String {
    format_json(value, indent, "    ", false)
}


fn fail(message: &str) -> Error {
    Error::Custom(format!("edit: {message}").into())
}

fn describe(edit: &Edit) -> String {
    match edit {
        Edit::Set { path, .. } => format!("set {}", path.join(".")),
        Edit::SetFn { path, .. } => format!("set_fn {}", path.join(".")),
        Edit::Remove { path } => format!("remove {}", path.join(".")),
        Edit::Rename { path, to } => format!("rename {} to {to}", path.join(".")),
    }
}


fn apply(text: &str, defs: &[OutlineDef], edit: &Edit) -> Result<String, Error> {
    let out = apply_lf(text, defs, edit)?;
    Ok(out)
}

/// Line endings of added text follow the text's (CRLF when it has any).
fn newlines(text: &str, added: &str) -> String {
    if text.contains("\r\n") { added.replace("\r\n", "\n").replace('\n', "\r\n") } else { added.to_string() }
}

fn apply_lf(text: &str, defs: &[OutlineDef], edit: &Edit) -> Result<String, Error> {
    let unit = indent_unit(text, defs);
    match edit {
        Edit::Set { path, value } => {
            if path.is_empty() { return Err(fail("set needs a path")); }
            if let Some(def) = find(defs, path)? {
                if def.kind == DefKind::Func { return Err(fail(&format!("{} is a function (use set_fn)", path.join(".")))); }
                let indent = line_indent(text, def.span.name_start);
                let value = reindent(value, &indent);
                return Ok(splice(text, def.span.value_start, def.span.value_end, &value));
            }
            insert(text, defs, path, &unit, |name, indent, compact| {
                let value = if compact { value.replace('\n', " ") } else { reindent(value, indent) };
                format!("{}: {}", name_text(name), value)
            })
        },
        Edit::SetFn { path, source } => {
            if path.is_empty() { return Err(fail("set_fn needs a path")); }
            let source = source.trim();
            let declared = after_attributes(source);
            let declared = declared.strip_prefix("async").map(|rest| rest.trim_start()).unwrap_or(declared);
            if !declared.starts_with("fn") { return Err(fail("set_fn source must be a function ('fn' or 'async fn', after any attributes and comments)")); }
            let leading = source.len() != after_attributes(source).len();
            if let Some(def) = find(defs, path)? {
                if def.kind != DefKind::Func { return Err(fail(&format!("{} is a field, not a function", path.join(".")))); }
                let indent = line_indent(text, def.span.name_start);
                // with attributes or comments, they replace the function's own (else it keeps them)
                let start = if leading { attached_start(text, def) } else { def.span.value_start };
                return Ok(splice(text, start, def.span.value_end, &reindent(source, &indent)));
            }
            insert(text, defs, path, &unit, |_name, indent, _compact| reindent(source, indent))
        },
        Edit::Remove { path } => {
            let Some(def) = find(defs, path)? else { return Err(fail(&format!("{} isn't there", path.join(".")))); };
            let siblings = children(defs, &path[..path.len() - 1]);
            let last = siblings.last().map(|last| last.span == def.span).unwrap_or(false);
            let (start, end) = removal(text, def, last);
            Ok(splice(text, start, end, ""))
        },
        Edit::Rename { path, to } => {
            let Some(def) = find(defs, path)? else { return Err(fail(&format!("{} isn't there", path.join(".")))); };
            if to.is_empty() { return Err(fail("rename needs a name")); }
            let mut target = path.clone();
            target.pop();
            target.push(to.clone());
            if find(defs, &target)?.is_some() { return Err(fail(&format!("{} is already there", target.join(".")))); }
            let current = &text[def.span.name_start..def.span.name_end];
            let name = if def.kind == DefKind::Func {
                if !is_ident(to) { return Err(fail(&format!("'{to}' isn't a valid function name"))); }
                to.clone()
            } else if let Some(quote) = current.chars().next().filter(|ch| *ch == '\'' || *ch == '"') {
                quote_name(to, quote)
            } else {
                name_text(to)
            };
            Ok(splice(text, def.span.name_start, def.span.name_end, &name))
        },
    }
}


/// Add a statement at a path that isn't there: into the deepest object on the path that is, creating the objects
/// in between. `statement(name, indent, compact)` writes the definition itself.
fn insert(text: &str, defs: &[OutlineDef], path: &[String], unit: &str, statement: impl Fn(&str, &str, bool) -> String) -> Result<String, Error> {
    // deepest existing object on the path
    let mut depth = 0;
    for k in (1..path.len()).rev() {
        if let Some(def) = find(defs, &path[..k])? {
            if def.kind == DefKind::Func || !def.object {
                return Err(fail(&format!("{} isn't an object", path[..k].join("."))));
            }
            depth = k;
            break;
        }
    }
    let container = if depth > 0 { find(defs, &path[..depth])? } else { None };
    let kids = children(defs, &path[..depth]);

    let (open, close) = match container {
        Some(def) => {
            let open = def.span.value_start;
            let close = def.span.value_end - 1;
            if !text[close..].starts_with('}') { return Err(fail(&format!("{} isn't an object", path[..depth].join(".")))); }
            (Some(open), close)
        },
        None => (None, text.len()),
    };
    let multiline = open.is_none() || text[open.unwrap()..close].contains('\n');
    let remaining = &path[depth..];

    // the definition, wrapped in the objects that aren't there yet
    let build = |indent: &str, compact: bool| -> String {
        let mut inner_indent = indent.to_string();
        for _ in 1..remaining.len() { inner_indent.push_str(unit); }
        let mut out = statement(&remaining[remaining.len() - 1], &inner_indent, compact);
        for level in (0..remaining.len() - 1).rev() {
            let mut level_indent = indent.to_string();
            for _ in 0..level { level_indent.push_str(unit); }
            out = if compact {
                format!("{}: {{ {out} }}", name_text(&remaining[level]))
            } else {
                format!("{}: {{\n{level_indent}{unit}{out}\n{level_indent}}}", name_text(&remaining[level]))
            };
        }
        out
    };

    if multiline {
        let (indent, pos, sep) = match kids.last() {
            Some(last) => {
                let indent = line_indent(text, last.span.name_start);
                let sep = separator(text, last);
                // after a comment at the end of the last definition's line
                let line_end = text[last.span.outer_end..].find('\n').map(|index| last.span.outer_end + index).unwrap_or(text.len());
                let rest = text[last.span.outer_end..line_end].trim();
                let pos = if rest.is_empty() || rest.starts_with("//") { line_end - if text[..line_end].ends_with('\r') { 1 } else { 0 } } else { last.span.outer_end };
                (indent, pos, sep)
            },
            None => match (container, open) {
                (Some(def), Some(open)) => (format!("{}{unit}", line_indent(text, def.span.name_start)), open + 1, String::new()),
                _ => {
                    let pos = text.trim_end().len();
                    return Ok(splice(text, pos, text.len(), &format!("{}{}\n", if pos > 0 { "\n" } else { "" }, build("", false))));
                },
            },
        };
        let added = build(&indent, false);
        return Ok(splice(text, pos, pos, &format!("\n{indent}{added}{sep}")));
    }

    // a one-line object: `{ a: 1, b: 2 }` or `{}`
    let open = open.unwrap();
    match kids.last() {
        Some(last) => {
            let sep = separator(text, last);
            if !sep.is_empty() {
                Ok(splice(text, last.span.outer_end, last.span.outer_end, &format!(" {}{sep}", build("", true))))
            } else {
                // the separator the object uses; with one field there's no style to follow, so a comma
                let any = kids.iter().map(|kid| separator(text, kid)).find(|sep| !sep.is_empty())
                    .unwrap_or_else(|| if kids.len() == 1 { ",".into() } else { String::new() });
                let joiner = if any.is_empty() { " ".to_string() } else { format!("{any} ") };
                Ok(splice(text, last.span.value_end, last.span.value_end, &format!("{joiner}{}", build("", true))))
            }
        },
        None => Ok(splice(text, open + 1, close, &format!(" {} ", build("", true)))),
    }
}


/// Where a definition starts with what's attached to it: its attributes and the comments directly above it (no
/// blank line in between), but not a comment that ends the previous definition's line.
fn attached_start(text: &str, def: &OutlineDef) -> usize {
    let span = def.span;
    let mut base = span.outer_start;
    let before = &text[line_start(text, base)..base];
    if !before.trim().is_empty() {
        if let Some(index) = text[base..span.name_start].find('\n') { base += index + 1; }
    }
    let lead = &text[base..span.name_start];
    let mut attach = base;
    let mut line = base;
    for (index, ch) in lead.char_indices() {
        if ch == '\n' {
            let next = base + index + 1;
            if text[line..base + index].trim().is_empty() && line != base { attach = next; }
            line = next;
        }
    }
    attach + text[attach..span.name_start].len() - text[attach..span.name_start].trim_start().len()
}


/// The range to remove for a definition: its attributes and the comments directly above it (no blank line in
/// between), the definition, its separator, and the rest of its line when it's on its own lines.
fn removal(text: &str, def: &OutlineDef, last: bool) -> (usize, usize) {
    let span = def.span;
    let attach = attached_start(text, def);

    let line_end = text[span.outer_end..].find('\n').map(|index| span.outer_end + index).unwrap_or(text.len());
    let rest = text[span.outer_end..line_end].trim();
    let own_line = text[line_start(text, attach)..attach].trim().is_empty() && (rest.is_empty() || rest.starts_with("//"));
    if own_line {
        let mut start = line_start(text, attach);
        if line_end < text.len() { return (start, line_end + 1); }
        // the last line, with no newline after it: take the newline before it instead
        if start > 0 { start -= 1; if start > 0 && text.as_bytes()[start - 1] == b'\r' { start -= 1; } }
        return (start, line_end);
    }

    // on a line with other definitions
    let mut end = span.outer_end;
    while end < text.len() && matches!(text.as_bytes()[end], b' ' | b'\t') { end += 1; }
    let mut start = attach;
    if last && separator(text, def).is_empty() {
        // `{ a: 1, b: 2 }` removing b: also the separator before it
        let trimmed = text[..start].trim_end();
        if trimmed.ends_with(',') || trimmed.ends_with(';') {
            start = trimmed.len() - 1;
            end = span.value_end;
        }
    }
    (start, end)
}


fn find<'a>(defs: &'a [OutlineDef], path: &[String]) -> Result<Option<&'a OutlineDef>, Error> {
    let mut found = None;
    for def in defs {
        if def.path == path {
            if found.is_some() { return Err(fail(&format!("{} is defined more than once", path.join(".")))); }
            found = Some(def);
        }
    }
    Ok(found)
}

fn children<'a>(defs: &'a [OutlineDef], path: &[String]) -> Vec<&'a OutlineDef> {
    defs.iter().filter(|def| def.path.len() == path.len() + 1 && def.path[..path.len()] == *path).collect()
}

fn separator(text: &str, def: &OutlineDef) -> String {
    let between = &text[def.span.value_end..def.span.outer_end];
    if between.contains(';') { ";".into() } else if between.contains(',') { ",".into() } else { String::new() }
}

fn splice(text: &str, start: usize, end: usize, with: &str) -> String {
    let with = &newlines(text, with);
    let mut out = String::with_capacity(text.len() + with.len());
    out.push_str(&text[..start]);
    out.push_str(with);
    out.push_str(&text[end..]);
    out
}

fn line_start(text: &str, pos: usize) -> usize {
    text[..pos].rfind('\n').map(|index| index + 1).unwrap_or(0)
}

/// The whitespace at the start of the line a position is on.
fn line_indent(text: &str, pos: usize) -> String {
    let start = line_start(text, pos);
    text[start..].chars().take_while(|ch| *ch == ' ' || *ch == '\t').collect()
}

/// The text's indent step: from the first definition indented deeper than its object, else 4 spaces.
fn indent_unit(text: &str, defs: &[OutlineDef]) -> String {
    let mut seen: FxHashSet<usize> = FxHashSet::default();
    for def in defs {
        if def.path.len() < 2 { continue; }
        let Some(parent) = defs.iter().find(|other| other.path == def.path[..def.path.len() - 1]) else { continue; };
        let (outer, inner) = (line_indent(text, parent.span.name_start), line_indent(text, def.span.name_start));
        if line_start(text, parent.span.name_start) == line_start(text, def.span.name_start) { continue; }
        if inner.len() > outer.len() && inner.starts_with(&outer) && seen.insert(inner.len() - outer.len()) {
            return inner[outer.len()..].to_string();
        }
    }
    "    ".into()
}

/// Indent every line after the first (a value written at column 0) to sit at an indent. Lines that start inside
/// a string are part of the string, so they're left alone.
pub(crate) fn reindent(value: &str, indent: &str) -> String {
    if indent.is_empty() || !value.contains('\n') { return value.to_string(); }
    let starts = code_line_starts(value);
    let mut out = String::with_capacity(value.len() + starts.len() * indent.len());
    let mut last = 0;
    for start in starts {
        out.push_str(&value[last..start]);
        let line_end = value[start..].find('\n').map(|index| start + index).unwrap_or(value.len());
        if !value[start..line_end].trim().is_empty() { out.push_str(indent); } // blank lines stay as they are
        last = start;
    }
    out.push_str(&value[last..]);
    out
}

/// The reverse of `reindent`: a value as if written at column 0 (Ex. a function's source for `Edit::SetFn`).
pub fn dedent(value: &str, indent: &str) -> String {
    if indent.is_empty() || !value.contains('\n') { return value.to_string(); }
    let mut out = String::with_capacity(value.len());
    let mut last = 0;
    for start in code_line_starts(value) {
        out.push_str(&value[last..start]);
        let line_end = value[start..].find('\n').map(|index| start + index).unwrap_or(value.len());
        let blank = value[start..line_end].trim().is_empty(); // blank lines stay as they are
        last = if !blank && value[start..].starts_with(indent) { start + indent.len() } else { start };
    }
    out.push_str(&value[last..]);
    out
}

/// CRLF line breaks as LF, except inside strings (where they're part of the value).
pub(crate) fn code_lf(value: &str) -> String {
    if !value.contains("\r\n") { return value.to_string(); }
    let mut out = String::with_capacity(value.len());
    let mut last = 0;
    for start in code_line_starts(value) {
        if start >= 2 && &value[start - 2..start] == "\r\n" {
            out.push_str(&value[last..start - 2]);
            out.push('\n');
            last = start;
        }
    }
    out.push_str(&value[last..]);
    out
}

/// Where the lines after the first start, for lines that don't start inside a string.
fn code_line_starts(value: &str) -> Vec<usize> {
    #[derive(PartialEq)]
    enum In { Code, Line, Block, Quote(u8), Raw }
    let bytes = value.as_bytes();
    let mut state = In::Code;
    let mut starts = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        match state {
            In::Code => {
                if value[index..].starts_with("//") { state = In::Line; index += 1; }
                else if value[index..].starts_with("/*") { state = In::Block; index += 1; }
                else if value[index..].starts_with("r#\"") { state = In::Raw; index += 2; }
                else if byte == b'\'' || byte == b'"' || byte == b'`' { state = In::Quote(byte); }
            },
            In::Line => if byte == b'\n' { state = In::Code; },
            In::Block => if value[index..].starts_with("*/") { state = In::Code; index += 1; },
            In::Raw => if value[index..].starts_with("\"#") { state = In::Code; index += 1; },
            In::Quote(quote) => {
                if byte == b'\\' { index += 1; }
                else if byte == quote { state = In::Code; }
            },
        }
        if byte == b'\n' && matches!(state, In::Code | In::Block) { starts.push(index + 1); }
        index += 1;
    }
    starts
}

/// A function's source after its leading attributes and comments.
fn after_attributes(source: &str) -> &str {
    let mut rest = source.trim_start();
    loop {
        if rest.starts_with("//") {
            rest = rest.find('\n').map(|index| rest[index + 1..].trim_start()).unwrap_or("");
        } else if rest.starts_with("/*") {
            rest = rest.find("*/").map(|index| rest[index + 2..].trim_start()).unwrap_or("");
        } else if rest.starts_with("#[") {
            // to the matching bracket (attribute values can hold brackets and strings)
            let mut depth = 0;
            let mut end = rest.len();
            let mut quote: Option<char> = None;
            let mut escaped = false;
            for (index, ch) in rest.char_indices().skip(1) {
                if let Some(q) = quote {
                    if escaped { escaped = false; } else if ch == '\\' { escaped = true; } else if ch == q { quote = None; }
                    continue;
                }
                match ch {
                    '\'' | '"' => quote = Some(ch),
                    '[' => depth += 1,
                    ']' => { depth -= 1; if depth == 0 { end = index + 1; break; } },
                    _ => {},
                }
            }
            rest = rest[end..].trim_start();
        } else {
            return rest;
        }
    }
}

fn is_ident(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(ch) if ch.is_ascii_alphabetic() || ch == '_' => {},
        _ => return false,
    }
    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn quote_name(name: &str, quote: char) -> String {
    let escaped = name.replace('\\', "\\\\").replace(quote, &format!("\\{quote}"));
    format!("{quote}{escaped}{quote}")
}

fn name_text(name: &str) -> String {
    if is_ident(name) { name.to_string() } else { quote_name(name, '\'') }
}

fn string_text(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('\'');
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\'' => out.push_str("\\'"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch => out.push(ch),
        }
    }
    out.push('\'');
    out
}

fn format_json(value: &Json, indent: &str, unit: &str, compact: bool) -> String {
    match value {
        Json::Null => "null".into(),
        Json::Bool(value) => value.to_string(),
        Json::Number(number) => number.to_string(),
        Json::String(value) => string_text(value),
        Json::Array(items) => {
            if items.is_empty() { return "[]".into(); }
            let nested = items.iter().any(|item| item.is_object() || item.is_array());
            if compact || !nested {
                let items: Vec<String> = items.iter().map(|item| format_json(item, indent, unit, true)).collect();
                return format!("[{}]", items.join(", "));
            }
            let inner = format!("{indent}{unit}");
            let items: Vec<String> = items.iter().map(|item| format!("{inner}{},", format_json(item, &inner, unit, false))).collect();
            format!("[\n{}\n{indent}]", items.join("\n"))
        },
        Json::Object(map) => {
            if map.is_empty() { return "{}".into(); }
            if compact {
                let fields: Vec<String> = map.iter().map(|(key, value)| format!("{}: {}", name_text(key), format_json(value, indent, unit, true))).collect();
                return format!("{{ {} }}", fields.join(", "));
            }
            let inner = format!("{indent}{unit}");
            let fields: Vec<String> = map.iter().map(|(key, value)| format!("{inner}{}: {}", name_text(key), format_json(value, &inner, unit, false))).collect();
            format!("{{\n{}\n{indent}}}", fields.join("\n"))
        },
    }
}


#[cfg(test)]
mod tests {
    use serde_json::json;
    use super::*;

    fn path(path: &str) -> Vec<String> {
        path.split('.').map(|name| name.to_string()).collect()
    }
    fn set(at: &str, value: &str) -> Edit {
        Edit::Set { path: path(at), value: value.to_string() }
    }
    fn remove(at: &str) -> Edit {
        Edit::Remove { path: path(at) }
    }

    const POLICY: &str = r#"// The policy.
Limitr policy: {
    credits: {
        /// A seat.
        seat: { label: 'Seat', unit: 'seat' }
        ai_token: {
            label: 'AI token'   // shown in the app
            price: { amount: 0.00002 }

            // what a call costs us
            #[custom]
            fn overhead(units: float, context?: obj) -> float {
                units * 0.5
            }
        }
    }

    plans: {
        free: {
            label: 'Free';
            default: true;
            entitlements: {}
        }
    }
}
"#;

    #[test]
    fn outlines_paths_and_spans() {
        let defs = outline(POLICY).unwrap();
        let paths: Vec<String> = defs.iter().map(|def| def.path.join(".")).collect();
        assert_eq!(paths, vec![
            "policy", "policy.credits", "policy.credits.seat", "policy.credits.seat.label", "policy.credits.seat.unit",
            "policy.credits.ai_token", "policy.credits.ai_token.label", "policy.credits.ai_token.price",
            "policy.credits.ai_token.price.amount", "policy.credits.ai_token.overhead",
            "policy.plans", "policy.plans.free", "policy.plans.free.label", "policy.plans.free.default",
            "policy.plans.free.entitlements",
        ]);
        let label = defs.iter().find(|def| def.path.join(".") == "policy.credits.seat.label").unwrap();
        assert_eq!(&POLICY[label.span.value_start..label.span.value_end], "'Seat'");
        assert_eq!(&POLICY[label.span.name_start..label.span.name_end], "label");
        let overhead = defs.iter().find(|def| def.name() == "overhead").unwrap();
        assert_eq!(overhead.kind, DefKind::Func);
        assert!(POLICY[overhead.span.value_start..overhead.span.value_end].starts_with("fn overhead("));
        assert!(POLICY[overhead.span.value_start..overhead.span.value_end].ends_with("}"));
        let free_label = defs.iter().find(|def| def.path.join(".") == "policy.plans.free.label").unwrap();
        assert_eq!(&POLICY[free_label.span.value_end..free_label.span.outer_end], ";");
        assert!(defs.iter().find(|def| def.name() == "policy").unwrap().object);
        assert!(!label.object);
    }

    #[test]
    fn outline_and_edit_as_json() {
        let defs = outline(POLICY).unwrap();
        let json = outline_json(POLICY, &defs);
        let overhead = json.as_array().unwrap().iter().find(|def| def["name"] == "overhead").unwrap();
        assert_eq!(overhead["kind"], "fn");
        assert_eq!(overhead["path"], json!(["policy", "credits", "ai_token", "overhead"]));
        assert_eq!(overhead["value"], "fn overhead(units: float, context?: obj) -> float {\n    units * 0.5\n}");
        assert_eq!(overhead["source"], "// what a call costs us\n#[custom]\nfn overhead(units: float, context?: obj) -> float {\n    units * 0.5\n}");
        let seat = json.as_array().unwrap().iter().find(|def| def["name"] == "seat").unwrap();
        assert_eq!(seat["source"], "/// A seat.\nseat: { label: 'Seat', unit: 'seat' }");
        assert_eq!(json[0]["object"], true);

        let out = edit_json(POLICY, r#"[{ "set": ["policy", "plans", "free", "label"], "value": "Starter" }, { "remove": "policy.credits.seat" }]"#).unwrap();
        assert!(out.contains("label: 'Starter';") && !out.contains("seat:"), "{out}");
        assert!(edit_json(POLICY, "not json").is_err());
        assert!(edit_json(POLICY, r#"[{ "nope": "a" }]"#).is_err());
    }

    #[test]
    fn set_replaces_only_the_value() {
        let out = edit(POLICY, &[set("policy.credits.seat.label", "'Team seat'"), set("policy.plans.free.default", "false")]).unwrap();
        assert_eq!(out, POLICY.replace("label: 'Seat'", "label: 'Team seat'").replace("default: true;", "default: false;"));
    }

    #[test]
    fn set_adds_fields_in_the_objects_style() {
        // multi-line object, after a trailing comment
        let out = edit(POLICY, &[set("policy.credits.ai_token.description", "'Model tokens'")]).unwrap();
        assert!(out.contains("            fn overhead(units: float, context?: obj) -> float {\n                units * 0.5\n            }\n            description: 'Model tokens'\n        }"), "{out}");

        // one-line object
        let out = edit(POLICY, &[set("policy.credits.seat.price", "{ amount: 10 }")]).unwrap();
        assert!(out.contains("seat: { label: 'Seat', unit: 'seat', price: { amount: 10 } }"), "{out}");
        let out = edit("a: { b: 1 c: 2 }\n", &[set("a.d", "3")]).unwrap();
        assert_eq!(out, "a: { b: 1 c: 2 d: 3 }\n");
        let out = edit("a: { b: 1; c: 2; }\n", &[set("a.d", "3")]).unwrap();
        assert_eq!(out, "a: { b: 1; c: 2; d: 3; }\n");

        // separators: the plan uses ';'
        let out = edit(POLICY, &[set("policy.plans.free.hidden", "true")]).unwrap();
        assert!(out.contains("            entitlements: {}\n            hidden: true\n"), "{out}");

        // an empty object, and objects that aren't there yet
        let out = edit(POLICY, &[set("policy.plans.free.entitlements.api.limit.value", "1000")]).unwrap();
        assert!(out.contains("entitlements: { api: { limit: { value: 1000 } } }"), "{out}");
        let out = edit(POLICY, &[set("policy.plans.pro.label", "'Pro'")]).unwrap();
        assert!(out.contains("        }\n        pro: {\n            label: 'Pro'\n        }\n    }\n}"), "{out}");

        // the top level
        let out = edit("a: 1\n", &[set("b.c", "2")]).unwrap();
        assert_eq!(out, "a: 1\nb: {\n    c: 2\n}\n");
        let out = edit("", &[set("a", "1")]).unwrap();
        assert_eq!(out, "a: 1\n");
    }

    #[test]
    fn set_follows_the_texts_indent() {
        let text = "policy: {\n\tplans: {\n\t\tfree: {\n\t\t\tlabel: 'Free'\n\t\t}\n\t}\n}\n";
        let out = edit(text, &[set("policy.plans.free.limit.value", "5")]).unwrap();
        assert_eq!(out, "policy: {\n\tplans: {\n\t\tfree: {\n\t\t\tlabel: 'Free'\n\t\t\tlimit: {\n\t\t\t\tvalue: 5\n\t\t\t}\n\t\t}\n\t}\n}\n");
    }

    #[test]
    fn json_values() {
        let value = json!({ "amount": 10, "tiers": [{ "up_to": 100, "price": { "amount": 1 } }], "tags": ["a", "b"], "full name": "it's" });
        let edits = edits_from_json(&json!([{ "set": "a.price", "value": value }])).unwrap();
        let out = edit("a: {\n    label: 'A'\n}\n", &edits).unwrap();
        assert_eq!(out, "a: {\n    label: 'A'\n    price: {\n        amount: 10\n        tiers: [\n            {\n                up_to: 100\n                price: {\n                    amount: 1\n                }\n            },\n        ]\n        tags: ['a', 'b']\n        'full name': 'it\\'s'\n    }\n}\n");
        let check = outline(&out).unwrap();
        assert!(check.iter().any(|def| def.path.join(".") == "a.price.tags"));
    }

    #[test]
    fn set_fn_keeps_attributes() {
        let out = edit(POLICY, &[Edit::SetFn { path: path("policy.credits.ai_token.overhead"), source: "fn overhead(units: float, context?: obj) -> float {\n    units * 2\n}".into() }]).unwrap();
        assert!(out.contains("            // what a call costs us\n            #[custom]\n            fn overhead(units: float, context?: obj) -> float {\n                units * 2\n            }\n"), "{out}");

        let out = edit(POLICY, &[Edit::SetFn { path: path("policy.plans.free.entitlements.chat.enabled"), source: "fn enabled() -> bool { true }".into() }]).unwrap();
        assert!(out.contains("entitlements: { chat: { fn enabled() -> bool { true } } }"), "{out}");

        // with attributes: they replace the function's own
        let out = edit(POLICY, &[Edit::SetFn { path: path("policy.credits.ai_token.overhead"), source: "/// Cost.\n#[run]\nfn overhead() -> float { 1 }".into() }]).unwrap();
        assert!(out.contains("            price: { amount: 0.00002 }\n\n            /// Cost.\n            #[run]\n            fn overhead() -> float { 1 }\n        }"), "{out}");
        let out = edit(POLICY, &[Edit::SetFn { path: path("policy.capabilities.notify.call"), source: "#[run({\"args\": [1]})]\nfn call() {}".into() }]).unwrap();
        assert!(out.contains("    capabilities: {\n        notify: {\n            #[run({\"args\": [1]})]\n            fn call() {}\n        }\n    }\n}"), "{out}");
        assert!(edit(POLICY, &[Edit::SetFn { path: path("policy.a"), source: "#[x] label: 1".into() }]).is_err());

        assert!(edit(POLICY, &[Edit::SetFn { path: path("policy.credits.seat.label"), source: "fn label() {}".into() }]).is_err());
        assert!(edit(POLICY, &[Edit::SetFn { path: path("policy.x"), source: "label: 1".into() }]).is_err());
    }

    #[test]
    fn remove_takes_attached_lines() {
        // with its doc comment
        let out = edit(POLICY, &[remove("policy.credits.seat")]).unwrap();
        assert!(out.contains("    credits: {\n        ai_token: {"), "{out}");

        // a function with its comment and attribute
        let out = edit(POLICY, &[remove("policy.credits.ai_token.overhead")]).unwrap();
        assert!(out.contains("            price: { amount: 0.00002 }\n\n        }"), "{out}");

        // with a comment at the end of its line
        let out = edit(POLICY, &[remove("policy.credits.ai_token.label")]).unwrap();
        assert!(out.contains("        ai_token: {\n            price: { amount: 0.00002 }"), "{out}");

        // inline, first and last
        let out = edit(POLICY, &[remove("policy.credits.seat.label")]).unwrap();
        assert!(out.contains("seat: { unit: 'seat' }"), "{out}");
        let out = edit(POLICY, &[remove("policy.credits.seat.unit")]).unwrap();
        assert!(out.contains("seat: { label: 'Seat' }"), "{out}");

        // a comment separated by a blank line stays
        let text = "a: 1\n\n// section\n\nb: 2\nc: 3\n";
        assert_eq!(edit(text, &[remove("b")]).unwrap(), "a: 1\n\n// section\n\nc: 3\n");
        assert_eq!(edit(text, &[remove("c")]).unwrap(), "a: 1\n\n// section\n\nb: 2\n");

        assert!(edit(POLICY, &[remove("policy.nope")]).is_err());
    }

    #[test]
    fn rename_keeps_everything_else() {
        let out = edit(POLICY, &[Edit::Rename { path: path("policy.plans.free"), to: "starter".into() }]).unwrap();
        assert_eq!(out, POLICY.replace("        free: {", "        starter: {"));
        let out = edit("'full name': 1\n", &[Edit::Rename { path: vec!["full name".into()], to: "it's".into() }]).unwrap();
        assert_eq!(out, "'it\\'s': 1\n");
        let out = edit("a: 1\n", &[Edit::Rename { path: path("a"), to: "b c".into() }]).unwrap();
        assert_eq!(out, "'b c': 1\n");
        assert!(edit("a: 1\nb: 2\n", &[Edit::Rename { path: path("a"), to: "b".into() }]).is_err());
    }

    #[test]
    fn strings_and_comments_with_braces() {
        let text = "a: {\n    s: '} { not an object'\n    r: r#\"}\"#\n    // }\n    /* { */\n    n: 1\n}\n";
        let out = edit(text, &[set("a.m", "2"), set("a.n", "3")]).unwrap();
        assert_eq!(out, "a: {\n    s: '} { not an object'\n    r: r#\"}\"#\n    // }\n    /* { */\n    n: 3\n    m: 2\n}\n");
    }

    #[test]
    fn refuses_what_it_cant_do() {
        assert!(edit("a: {", &[]).is_err()); // doesn't parse
        assert!(edit("a: 1\n", &[set("a.b", "1")]).is_err()); // not an object
        assert!(edit("a: 1\n", &[set("a", "{")]).is_err()); // result doesn't parse
        assert!(edit("a: { b: 1 }\na: { c: 2 }\n", &[set("a.b", "3")]).is_ok()); // a.b is once
        assert!(edit("a: { b: 1 }\na: { b: 2 }\n", &[set("a.b", "3")]).is_err()); // twice
        let failed = edit("a: 1\n", &[set("b", "1"), remove("nope")]);
        assert!(failed.is_err());
    }

    #[test]
    fn nothing_runs() {
        // an endless loop in a field value, an attribute, and an init function: outlined, never run
        let text = "x: (():int => { while (true) {} 1 })()\n#[run((():int => { while (true) {} 1 })())]\nfn f() {}\n#[init]\nfn boot() { while (true) {} }\n";
        let defs = outline(text).unwrap();
        assert_eq!(defs.iter().map(|def| def.name()).collect::<Vec<_>>(), vec!["x", "f", "boot"]);
        let out = edit(text, &[set("y", "2")]).unwrap();
        assert!(out.ends_with("y: 2\n"));
    }

    #[test]
    fn crlf_outside_strings() {
        assert_eq!(code_lf("fn f() {\r\n    'a\r\nb' // c\r\n}"), "fn f() {\n    'a\r\nb' // c\n}");
        assert_eq!(code_lf("x\r\nr#\"a\r\nb\"#\r\n"), "x\nr#\"a\r\nb\"#\n");
    }

    #[test]
    fn crlf_text() {
        let text = "a: {\r\n    b: 1\r\n}\r\n";
        let out = edit(text, &[set("a.c", "2"), remove("a.b")]).unwrap();
        assert_eq!(out, "a: {\r\n    c: 2\r\n}\r\n");
    }

    /// Every .stof file in this repo that outlines: setting each value to itself and renaming each definition
    /// there and back changes nothing; removing each one leaves a text that parses and has everything else.
    #[test]
    fn corpus_round_trips() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut files = Vec::new();
        let mut stack = vec![root.join("src"), root.join("web")];
        while let Some(dir) = stack.pop() {
            let Ok(entries) = std::fs::read_dir(&dir) else { continue; };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() { if !path.ends_with("node_modules") { stack.push(path); } }
                else if path.extension().map(|ext| ext == "stof").unwrap_or(false) { files.push(path); }
            }
        }
        let mut checked = 0;
        let mut edits = 0;
        for file in files {
            let text = std::fs::read_to_string(&file).unwrap();
            if cfg!(debug_assertions) && text.len() > 6000 { continue; } // unoptimized parsing is slow: big files in release
            let Ok(defs) = outline(&text) else { continue; };
            checked += 1;
            for def in defs.iter().take(if cfg!(debug_assertions) { 2 } else { 16 }) {
                let unique = defs.iter().filter(|other| other.path == def.path).count() == 1;
                if !unique { continue; }
                edits += 1;
                let label = format!("{} in {}", def.path.join("."), file.display());
                if def.kind == DefKind::Field {
                    // values are given as if written at column 0
                    let indent = line_indent(&text, def.span.name_start);
                    let value = dedent(&text[def.span.value_start..def.span.value_end], &indent);
                    let out = edit(&text, &[Edit::Set { path: def.path.clone(), value }]).unwrap_or_else(|error| panic!("set {label}: {error:?}"));
                    assert_eq!(out, text, "set {label}");
                }
                let mut moved = def.path.clone();
                moved.pop();
                moved.push("zz_renamed".into());
                match edit(&text, &[
                    Edit::Rename { path: def.path.clone(), to: "zz_renamed".into() },
                    Edit::Rename { path: moved, to: def.name().into() },
                ]) {
                    Ok(out) => assert_eq!(out, text, "rename {label}"),
                    // renaming a type its text uses (Ex. `#[extends]`) is refused: the text no longer works
                    Err(error) => assert!(error.to_string().contains("prototype type wasn't found"), "rename {label}: {error:?}"),
                }

                let out = match edit(&text, &[Edit::Remove { path: def.path.clone() }]) {
                    Ok(out) => out,
                    Err(error) => {
                        assert!(error.to_string().contains("prototype type wasn't found"), "remove {label}: {error:?}");
                        continue;
                    },
                };
                let after = outline(&out).unwrap();
                let expected: Vec<&Vec<String>> = defs.iter().filter(|other| !other.path.starts_with(&def.path)).map(|other| &other.path).collect();
                let found: Vec<&Vec<String>> = after.iter().map(|other| &other.path).collect();
                assert_eq!(found, expected, "remove {label}");
            }
        }
        let (files_min, edits_min) = if cfg!(debug_assertions) { (10, 15) } else { (20, 200) };
        assert!(checked >= files_min, "only {checked} files outlined");
        assert!(edits >= edits_min, "only {edits} edits");
    }

}
