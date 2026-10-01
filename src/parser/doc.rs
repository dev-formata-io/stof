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

use arcstr::ArcStr;
use crate::{model::{InnerDoc, SId}, parser::{source, context::ParseContext, data::parse_data, field::parse_field, func::parse_function, ident::ident, import::import, string::{double_string, single_string}, whitespace::{parse_inner_doc_comment, whitespace, whitespace_fail}}, runtime::Error};
use nanoid::nanoid;
use nom::{branch::alt, bytes::complete::{tag, take_until}, character::complete::{char, multispace0, space0}, combinator::{eof, opt, map}, error::{ErrorKind, FromExternalError, ParseError}, sequence::{delimited, preceded}, Err, IResult, Parser};
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
/// Parse error.
/// Errors from nom combinators are cheap (no formatting or allocation): they record what failed and where.
/// `document()` turns that into a location and code frame once, only for the error reported.
/// Kept small: one is returned by value from every parser, so its size is stack used per nesting level.
pub struct StofParseError {
    /// Explicit message (custom errors). Empty for combinator errors (see `describe`).
    pub message: String,

    /// Where the error happened: address of the input position (parsers work on slices of one text).
    /// Resolved to a line and column by `document()`; positions outside the text are ignored.
    #[serde(skip)]
    pub pos: Option<usize>,

    /// Character that was expected (Ex. '}').
    #[serde(default)]
    pub expected: Option<char>,

    /// Combinator that failed.
    #[serde(skip)]
    pub kind: Option<ErrorKind>,

    /// The error is at the end of the input.
    #[serde(default)]
    pub at_end: bool,

    /// File and location details (boxed to keep the error small).
    #[serde(default)]
    pub info: Option<Box<ParseErrorInfo>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
/// Where a parse error is: file, (line, column), and a code frame.
pub struct ParseErrorInfo {
    pub file_path: Option<String>,
    pub location: Option<(usize, usize)>,
    pub frame: String,
}
impl StofParseError {
    /// File the error is in, if known.
    pub fn file_path(&self) -> Option<&str> { self.info.as_ref().and_then(|info| info.file_path.as_deref()) }

    /// (line, column), once located.
    pub fn location(&self) -> Option<(usize, usize)> { self.info.as_ref().and_then(|info| info.location) }

    /// Set the file if not already known (the innermost file wins).
    pub fn set_file_if_unknown(&mut self, path: String) {
        let info = self.info.get_or_insert_with(Default::default);
        if info.file_path.is_none() { info.file_path = Some(path); }
    }
}
impl From<&str> for StofParseError {
    fn from(value: &str) -> Self {
        Self { message: value.to_string(), ..Default::default() }
    }
}
impl From<String> for StofParseError {
    fn from(value: String) -> Self {
        Self { message: value, ..Default::default() }
    }
}
impl StofParseError {
    /// Human readable description of what went wrong (without the location).
    pub fn describe(&self) -> String {
        if !self.message.is_empty() {
            return self.message.clone();
        }
        if self.at_end {
            return match self.expected {
                Some(c @ ('\'' | '"' | '`')) => format!("unterminated string (missing the closing {c})"),
                Some(c) => format!("unexpected end of input (expected '{c}')"),
                None => "unexpected end of input".into(),
            };
        }
        if let Some(c) = self.expected {
            return format!("expected '{c}'");
        }
        match self.kind {
            Some(ErrorKind::Eof) => "unexpected input (expected the end of the document)".into(),
            Some(ErrorKind::Digit) | Some(ErrorKind::HexDigit) | Some(ErrorKind::OctDigit) => "invalid number".into(),
            _ => "invalid syntax".into(),
        }
    }
}
/// Enter one level of recursive parsing (see `source::NestingGuard`), failing on input nested too deeply.
pub(crate) fn nest(input: &str) -> Result<source::NestingGuard, nom::Err<StofParseError>> {
    source::NestingGuard::enter().ok_or_else(|| nom::Err::Failure(StofParseError {
        message: format!("nested too deeply (more than {} levels)", source::MAX_NESTING),
        pos: Some(input.as_ptr() as usize),
        ..Default::default()
    }))
}

/// Run a recursive parser one nesting level deeper: errors past MAX_NESTING, and makes sure there's stack.
#[inline]
pub(crate) fn nested<'a, T>(input: &'a str, f: impl FnOnce() -> IResult<&'a str, T, StofParseError>) -> IResult<&'a str, T, StofParseError> {
    let _depth = nest(input)?;
    source::grow(f)
}

/// Map a Stof error from an error to a failure.
/// Used in making sure things fail at the document level.
pub fn err_fail(e: nom::Err<StofParseError>) -> nom::Err<StofParseError> {
    match e {
        nom::Err::Error(e) => nom::Err::Failure(e),
        _ => e
    }
}

impl ParseError<&str> for StofParseError {
    // nom creates one of these for every alternative that fails: keep it allocation free
    fn from_error_kind(input: &str, kind: ErrorKind) -> Self {
        StofParseError { pos: Some(input.as_ptr() as usize), kind: Some(kind), ..Default::default() }
    }

    // keep the innermost error
    fn append(_input: &str, _kind: ErrorKind, other: Self) -> Self {
        other
    }

    fn from_char(input: &str, c: char) -> Self {
        StofParseError { pos: Some(input.as_ptr() as usize), expected: Some(c), ..Default::default() }
    }

    // between alternatives, report the one that got furthest (most likely what the author meant)
    fn or(self, other: Self) -> Self {
        match (self.pos, other.pos) {
            (Some(mine), Some(theirs)) if mine > theirs => self,
            (Some(mine), Some(theirs)) if mine == theirs && !self.message.is_empty() && other.message.is_empty() => self, // more specific
            _ => other,
        }
    }
}
impl FromExternalError<&str, std::num::ParseIntError> for StofParseError {
    fn from_external_error(input: &str, _kind: ErrorKind, e: std::num::ParseIntError) -> Self {
        StofParseError { message: e.to_string(), pos: Some(input.as_ptr() as usize), ..Default::default() }
    }
}


thread_local! {
    /// Furthest statement error in the current document statement. Statement lists (blocks) stop at the
    /// first statement that doesn't parse and drop its error, so the error that surfaces is usually a
    /// generic "expected '}'"; the furthest statement error is what the author got wrong.
    static FURTHEST: std::cell::RefCell<Option<StofParseError>> = const { std::cell::RefCell::new(None) };
}

/// Record a statement error if it got further than any so far (cheap: no allocation for combinator errors).
pub(crate) fn note_statement_error(error: &StofParseError) {
    if error.pos.is_none() { return; }
    FURTHEST.with(|furthest| {
        let mut furthest = furthest.borrow_mut();
        if furthest.as_ref().map(|current| error.pos > current.pos).unwrap_or(true) {
            *furthest = Some(error.clone());
        }
    });
}

fn take_furthest() -> Option<StofParseError> {
    FURTHEST.with(|furthest| furthest.borrow_mut().take())
}


/// Locate a parse error in the document text it came from: line, column, and a code frame.
/// `statement` is the input at the start of the statement that failed (fallback position).
fn locate_parse_error(text: &str, statement: &str, mut error: StofParseError) -> StofParseError {
    if error.location().is_some() { return error; } // from an imported document (already located)
    let start = text.as_ptr() as usize;
    let statement_offset = text.len() - statement.len();
    let offset_of = |error: &StofParseError| error.pos.and_then(|pos| {
        // must be in this statement (sub-parsers of other strings report elsewhere)
        if pos >= start + statement_offset && pos <= start + text.len() { Some(pos - start) } else { None }
    });

    // a deeper statement error is more precise than a generic error at the end of the block
    if let Some(furthest) = take_furthest() {
        if let Some(further) = offset_of(&furthest) {
            if error.message.is_empty() && offset_of(&error).map(|offset| further >= offset).unwrap_or(true) {
                error = StofParseError { info: error.info, ..furthest };
            }
        }
    }

    let mut offset = offset_of(&error).unwrap_or(statement_offset);
    if error.expected.is_some() {
        // "expected ';'": point right after the previous token, not at the next line
        let before = text[..offset].trim_end();
        if before.len() >= statement_offset { offset = before.len(); }
    } else {
        // point at the next token rather than the whitespace before it
        let rest = &text[offset..];
        let trimmed = rest.trim_start();
        if !trimmed.is_empty() { offset += rest.len() - trimmed.len(); }
    }
    error.at_end = text[offset..].trim().is_empty();

    let (line, col, line_text) = source::line_col(text, offset);
    let info = error.info.get_or_insert_with(Default::default);
    info.location = Some((line, col));
    info.frame = source::code_frame(line, col, line_text);
    error
}


/// Parse a Stof document into a context (graph).
pub fn document(mut input: &str, context: &mut ParseContext) -> Result<(), Error> {
    let text = input;
    let _source = source::push_source(text, context.current_file().map(ArcStr::from), context.profile.debug_info);
    loop {
        take_furthest(); // per statement
        let res = document_statement(input, context);
        match res {
            Ok((rest, _)) => {
                if rest.is_empty() { break; }
                input = rest;
            },
            Err(error) => {
                // didn't match a singular statement (including whitespace)
                let error = match error {
                    nom::Err::Error(e) |
                    nom::Err::Failure(e) => e,
                    nom::Err::Incomplete(_) => StofParseError::from("unexpected end of input"),
                };
                return Err(Error::ParseError(locate_parse_error(text, input, error)));
            }
        }
    }
    context.finish_name_checks();
    Ok(())
}


/// Parse a singular document statement.
pub fn document_statement<'a>(input: &'a str, context: &mut ParseContext) -> IResult<&'a str, (), StofParseError> {
    // Field
    {
        let field_res = parse_field(input, context);
        match field_res {
            Ok((input, _)) => {
                return Ok((input, ()));
            },
            Err(error) => {
                match error {
                    Err::Incomplete(_) |
                    Err::Error(_) => {},
                    Err::Failure(_) => {
                        return Err(error);
                    }
                }
            }
        }
    }

    // Function
    {
        let func_res = parse_function(input, context);
        match func_res {
            Ok((input, _)) => {
                return Ok((input, ()));
            },
            Err(error) => {
                match error {
                    Err::Incomplete(_) |
                    Err::Error(_) => {},
                    Err::Failure(_) => {
                        return Err(error);
                    }
                }
            }
        }
    }

    // Import
    {
        let import_res = import(input, context);
        match import_res {
            Ok((input, _)) => {
                return Ok((input, ()));
            },
            Err(error) => {
                match error {
                    Err::Incomplete(_) |
                    Err::Error(_) => {},
                    Err::Failure(_) => {
                        return Err(error);
                    }
                }
            }
        }
    }

    // Data (binary)
    {
        let data_res = parse_data(input, context);
        match data_res {
            Ok((input, _)) => {
                return Ok((input, ()));
            },
            Err(error) => {
                match error {
                    Err::Incomplete(_) |
                    Err::Error(_) => {},
                    Err::Failure(_) => {
                        return Err(error);
                    }
                }
            }
        }
    }

    // New root object + statements
    {
        let root_res = root_statements(input, context);
        match root_res {
            Ok((input, _)) => {
                return Ok((input, ()));
            },
            Err(error) => {
                match error {
                    Err::Incomplete(_) |
                    Err::Error(_) => {},
                    Err::Failure(_) => {
                        return Err(error);
                    }
                }
            }
        }
    }

    // JSON-like brackets
    {
        let json_res = json_statements(input, context);
        match json_res {
            Ok((input, _)) => {
                return Ok((input, ()));
            },
            Err(error) => {
                match error {
                    Err::Incomplete(_) |
                    Err::Error(_) => {},
                    Err::Failure(_) => {
                        return Err(error);
                    }
                }
            }
        }
    }

    // Inner comment?
    if let Ok((input, docs)) = parse_inner_doc_comment(input) {
        if context.profile.docs {
            let self_ptr = context.self_ptr();
            context.graph.insert_stof_data(&self_ptr, &nanoid!(15), Box::new(InnerDoc { docs }), None);
        }
        return Ok((input, ()));
    }

    // Whitespace in the document
    if let Ok((input, _)) = whitespace_fail(input) {
        return Ok((input, ()));
    }

    // End of the document?
    let (input, _) = eof(input)?;
    Ok((input, ()))
}


/// New root document node statements.
fn root_statements<'a>(input: &'a str, context: &mut ParseContext) -> IResult<&'a str, (), StofParseError> {
    let (input, name) = preceded(tag("root"), delimited(multispace0, opt(alt((map(ident, |s| s.to_owned()), double_string, single_string))), multispace0)).parse(input)?;
    let (input, _) = char('{')(input)?;

    // Optional custom object ID - not recommended unless you know what you're doing
    let (mut input, custom_id) = opt(delimited(
        space0,
        delimited(char('('), take_until(")"), char(')')),
        whitespace,
    )).parse(input)?;
    let mut cid = None;
    if let Some(id) = custom_id { cid = Some(SId::from(id)); }

    context.push_root(name, cid);
    loop {
        let res = document_statement(input, context);
        match res {
            Ok((rest, _)) => {
                if rest.is_empty() || rest.len() == input.len() {
                    context.pop_self();
                    return Err(nom::Err::Failure(StofParseError::from_char(rest, '}')));
                }
                input = rest;
                if input.starts_with('}') {
                    break;
                }
            },
            Err(error) => {
                return Err(error);
            }
        }
    }
    context.pop_self();
    let (input, _) = char('}')(input)?;
    Ok((input, ()))
}


/// Empty brackets around some statements (accepts JSON).
fn json_statements<'a>(input: &'a str, context: &mut ParseContext) -> IResult<&'a str, (), StofParseError> {
    let (mut input, _) = char('{')(input)?;
    loop {
        let res = document_statement(input, context);
        match res {
            Ok((rest, _)) => {
                if rest.is_empty() || rest.len() == input.len() {
                    // end of input (or no progress) before the closing brace: was an endless loop
                    return Err(nom::Err::Failure(StofParseError::from_char(rest, '}')));
                }
                input = rest;
                if input.starts_with('}') {
                    break;
                }
            },
            Err(error) => {
                return Err(error);
            }
        }
    }
    let (input, _) = char('}')(input)?;
    Ok((input, ()))
}


#[cfg(test)]
mod tests {
    use crate::{model::{Graph, Profile}, parser::{context::ParseContext, doc::document}, runtime::{Runtime, Val}};

    fn parse_err(src: &str) -> String {
        let mut graph = Graph::default();
        let err = graph.parse_stof_src(src, None, Profile::default()).unwrap_err();
        err.to_string()
    }

    #[test]
    /// Parse errors say what's wrong, where (line:col), and show the line with a caret.
    fn parse_errors_are_located() {
        let err = parse_err("a: 1\nfn main() {\n    let x = 5\n    pln(x);\n}");
        assert!(err.starts_with("parse error: expected ';'"), "{err}");
        assert!(err.contains("--> line 3, column 14"), "{err}");
        assert!(err.contains(" 3 |     let x = 5\n   |              ^"), "{err}");

        let err = parse_err("fn main() {\n    if (x > 2 {\n    }\n}");
        assert!(err.starts_with("parse error: expected ')'") && err.contains("line 2, column 14"), "{err}");

        let err = parse_err("a: {\n    b: 'x\n}");
        assert!(err.starts_with("parse error: unterminated string"), "{err}");

        let err = parse_err("fn f() -> str { `hi ${ 1 + }` }");
        assert!(err.starts_with("parse error: invalid expression in template string") && err.contains("column 17"), "{err}");

        // input that used to hang, crash, or panic is a parse error
        assert!(parse_err("a: new {").contains("expected '}'"));                     // endless loop
        assert!(parse_err("a: { b: 1").contains("expected '}'"));                    // endless loop
        assert!(parse_err("root X { b: 1").contains("expected '}'"));                // endless loop
        assert!(parse_err(&format!("a: {}{}", "[".repeat(3000), "]".repeat(3000))).contains("nested too deeply")); // stack overflow
        assert!(parse_err(&format!("fn f() {{ {}1{} }}", "(".repeat(3000), ")".repeat(3000))).contains("nested too deeply"));
        assert!(parse_err("a: |300|").contains("blob values are bytes"));            // panic
        assert!(parse_err("fn f() { switch (1) { case (1 / 'x').nope(): 1 } }").contains("switch case values must be constants")); // panic

        // nested parentheses parse in linear time (each level was parsed 3 times: 12 levels took seconds)
        let start = std::time::Instant::now();
        let mut graph = Graph::default();
        let depth = crate::parser::source::MAX_NESTING - 4;
        graph.parse_stof_src(&format!("fn f() -> int {{ {}1{} }}", "(".repeat(depth), ")".repeat(depth)), None, Profile::default()).unwrap();
        assert!(start.elapsed().as_secs() < 2);
        assert_eq!(Runtime::call(&mut graph, "root.f", vec![]).unwrap(), Val::from(1));

        // "letter" is not "let ter"; "return x" needs a ';'
        assert!(parse_err("fn f() {\n    return 5\n}").contains("expected ';'"));
        let mut graph = Graph::default();
        graph.parse_stof_src("fn f() -> int { const letter = 3; letter }", None, Profile::default()).unwrap();
        assert_eq!(Runtime::call(&mut graph, "root.f", vec![]).unwrap(), Val::from(3));
    }

    #[test]
    fn basic_doc() {
        let mut graph = Graph::default();
        {
            let mut context = ParseContext::new(&mut graph, Profile::docs(true));
            document(r#"

            {
                "max": 200

                "object": {
                    "dude": true,
                    "hello": 450
                }

                list subobj: [
                    {
                        fn hello() -> str { 'hi' }
                    } as obj,
                    {
                        field: 'dude'
                    }
                ];

                async fn another_yet(max: int = self.max) -> int {
                    let total = 0;
                    for (let i = 0; i < max; i += 1) total += 1;
                    total
                }
        
                fn main(x: float = 5) -> float {
                    let a = self.another_yet();
                    let b = self.another_yet(4000);
                    let c = self.another_yet(1000);
                    let d = self.another_yet(800);

                    (await a) + (await b) + (await c) + (await d)
                }
            }

            "#, &mut context).unwrap();
        }

        graph.dump(true);

        let res = Runtime::call(&mut graph, "root.main", vec![Val::from(10)]).unwrap();
        assert_eq!(res, 6000.into());
    }
}
