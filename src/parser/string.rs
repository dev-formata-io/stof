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

use nom::{branch::alt, bytes::complete::{escaped_transform, tag, take_until}, character::complete::{anychar, char, none_of}, combinator::{map, opt, value}, sequence::delimited, IResult, Parser};
use crate::{parser::{doc::StofParseError, whitespace::whitespace}, runtime::Val};


/// Parse a string literal value.
pub fn string(input: &str) -> IResult<&str, Val, StofParseError> {
    let (input, _) = whitespace(input)?;
    let (input, out) = alt((
        raw_double_string,
        double_string,
        single_string
    )).parse(input)?;
    return Ok((input, Val::Str(out.into())))
}

/// Parse a raw double quoted string with escape chars.
/// Returns everything inside a r#"..."#
pub(self) fn raw_double_string(input: &str) -> IResult<&str, String, StofParseError> {
    let (input, res) = delimited(tag("r#\""), take_until("\"#"), tag("\"#")).parse(input)?;
    Ok((input, res.into()))
}

/// Parse a triple quoted string's text: raw text for prose (Ex. an intent's description) that can sit indented in code.
/// This is the text only; `${expr}` parts are run by the expression parser (parser::expr::fmt_str), and `\$` is
/// left in for it.
/// ```stof
/// let prompt = """
///     Summarize this customer's usage,
///     flagging anything over their plan limits.
///     """;
/// ```
/// - Raw: no escapes, so quotes, braces, and backslashes are written as they are, except `\${` for a literal `${`
///   (the expression parser runs `${expr}` parts). It ends at the next `"""`.
/// - Line breaks are `\n`, also in a file saved with Windows line endings (`\r\n`), so the value doesn't change with
///   the editor or git settings that saved it.
/// - On one line (`"""a "quoted" word"""`), the text is exactly what's between the quotes.
/// - Over several lines, the line break after the opening `"""` is dropped, and so is the closing `"""` line when
///   nothing but indentation comes before it. Text right after the opening `"""` is kept as written.
/// - Indentation: when the closing `"""` is on its own line, its indentation is removed from every line (a line
///   indented less is an error, a blank line becomes empty). When it ends a line of text, the indentation every
///   non-blank line shares is removed.
pub(crate) fn triple_string(input: &str) -> IResult<&str, String, StofParseError> {
    let (rest, raw) = delimited(tag("\"\"\""), take_until("\"\"\""), tag("\"\"\"")).parse(input)?;
    match dedent(raw) {
        Ok(text) => Ok((rest, text)),
        Err(message) => {
            let mut error = StofParseError::from(message);
            error.pos = Some(input.as_ptr() as usize);
            Err(nom::Err::Failure(error))
        },
    }
}

/// The text of a triple quoted string (what's between the quotes), as triple_string describes.
fn dedent(raw: &str) -> Result<String, String> {
    if !raw.contains('\n') {
        return Ok(raw.to_string()); // one line: exactly as written (a lone '\r' is part of the text)
    }
    let mut lines: Vec<&str> = raw.split('\n').map(|line| line.strip_suffix('\r').unwrap_or(line)).collect();

    // the line break after the opening quotes (with anything blank before it), or text right after them, which
    // keeps its spacing (it has no indentation of its own to remove)
    let first = lines.remove(0);
    let first = if first.trim().is_empty() { None } else { Some(first) };

    // the closing quotes' own line, which sets the indentation
    let mut indent = None;
    if let Some(last) = lines.last() {
        if last.chars().all(|c| c == ' ' || c == '\t') {
            indent = lines.pop();
        }
    }
    let indent = match indent {
        Some(indent) => indent,
        None => common_indent(&lines),
    };

    let mut text = String::with_capacity(raw.len());
    if let Some(first) = first {
        text.push_str(first);
        if !lines.is_empty() { text.push('\n'); }
    }
    for (i, line) in lines.iter().enumerate() {
        if i > 0 { text.push('\n'); }
        if line.trim().is_empty() { continue; } // a blank line is empty, however it's indented
        match line.strip_prefix(indent) {
            Some(line) => text.push_str(line),
            None => return Err(format!("a line in a \"\"\" string is indented less than its closing \"\"\" (line {}: '{}')", i + 2, line.trim())),
        }
    }
    Ok(text)
}

/// The leading spaces and tabs every non-blank line starts with.
fn common_indent<'a>(lines: &[&'a str]) -> &'a str {
    let mut common: Option<&'a str> = None;
    for line in lines {
        if line.trim().is_empty() { continue; }
        let indent = &line[..line.len() - line.trim_start_matches([' ', '\t']).len()];
        common = Some(match common {
            None => indent,
            Some(common) => {
                let shared = common.bytes().zip(indent.bytes()).take_while(|(a, b)| a == b).count();
                &common[..shared]
            },
        });
    }
    common.unwrap_or("")
}

/// Parse a double quoted string.
pub fn double_string(input: &str) -> IResult<&str, String, StofParseError> {
    let normal = none_of("\"\\"); // everything but backslash or double quote
    let inner = escaped_transform(normal, '\\', alt((
        value("\\", tag("\\")),
        value("\"", tag("\"")),
        value("\n", tag("n")),
        value("\r", tag("r")),
        value("\t", tag("t")),
        unknown_escape,
    )));
    delimited(char('"'), map(opt(inner), |opt| opt.unwrap_or_default()), char('"')).parse(input)
}

/// An escape a string doesn't know (Ex. "\." in a regular expression): a clear error, instead of a confusing one
/// from whatever parser tries next.
pub(crate) fn unknown_escape(input: &str) -> IResult<&str, &'static str, StofParseError> {
    let (_, c) = anychar(input)?;
    let mut error = StofParseError::from(format!("unknown escape '\\{c}' in a string (write '\\\\' for a backslash, or use a raw string: r#\"...\"#)"));
    error.pos = Some(input.as_ptr() as usize);
    Err(nom::Err::Failure(error))
}

/// Parse a single quoted string.
pub fn single_string(input: &str) -> IResult<&str, String, StofParseError> {
    let normal = none_of("'\\"); // everything but backslash or double quote
    let inner = escaped_transform(normal, '\\', alt((
        value("\\", tag("\\")),
        value("'", tag("'")),
        value("\n", tag("n")),
        value("\r", tag("r")),
        value("\t", tag("t")),
        unknown_escape,
    )));
    delimited(char('\''), map(opt(inner), |opt| opt.unwrap_or_default()), char('\'')).parse(input)
}


#[cfg(test)]
mod tests {
    use crate::parser::string::{double_string, raw_double_string, single_string, string, triple_string};

    #[test]
    fn raw() {
        let res = raw_double_string("r#\"Hello, world!\"#").unwrap();
        assert_eq!(res.1, "Hello, world!");
        assert_eq!(res.0, "");
    }

    #[test]
    fn raw_double_esc_quote() {
        let res = raw_double_string("r#\"Hello, \"world\"!\"#").unwrap();
        assert_eq!(res.1, r#"Hello, "world"!"#);
        assert_eq!(res.0, "");
    }

    #[test]
    fn double() {
        let res = double_string("\"Hello, world!\"").unwrap();
        assert_eq!(res.1, "Hello, world!");
        assert_eq!(res.0, "");
    }

    #[test]
    fn double_esc() {
        let res = double_string("\"Hello,\\t \\\"world\\\"!\"").unwrap();
        assert_eq!(res.1, "Hello,\t \"world\"!");
        assert_eq!(res.0, "");
    }

    #[test]
    fn single() {
        let res = single_string("'Hello, world!'").unwrap();
        assert_eq!(res.1, "Hello, world!");
        assert_eq!(res.0, "");
    }

    #[test]
    fn single_esc() {
        let res = single_string("'Hello,\\n \\'world\\'!'").unwrap();
        assert_eq!(res.1, "Hello,\n 'world'!");
        assert_eq!(res.0, "");
    }

    #[test]
    fn value() {
        assert_eq!(string(r#""Hello, \"CJ\"!""#).unwrap().1, "Hello, \"CJ\"!".into());
        assert_eq!(string(r#"'Hello, \'CJ\'!'"#).unwrap().1, "Hello, 'CJ'!".into());
        assert_eq!(string("r#\"Hello, \"CJ\"!\"#").unwrap().1, "Hello, \"CJ\"!".into());
    }

    #[test]
    fn triple_one_line() {
        let res = triple_string(r#""""Hello, "world"! {braces} \n stays""""#).unwrap();
        assert_eq!(res.1, r#"Hello, "world"! {braces} \n stays"#);
        assert_eq!(res.0, "");
        assert_eq!(triple_string(r#""""""""#).unwrap().1, "");
        assert_eq!(triple_string(r#""""  padded  """"#).unwrap().1, "  padded  ");
    }

    #[test]
    fn triple_closing_sets_indent() {
        let src = "\"\"\"\n        Summarize this customer.\n          Indented more.\n\n        Last line.\n        \"\"\"";
        assert_eq!(triple_string(src).unwrap().1, "Summarize this customer.\n  Indented more.\n\nLast line.");
    }

    #[test]
    fn triple_closing_less_indented_keeps_relative_indent() {
        let src = "\"\"\"\n        keep two\n      \"\"\"";
        assert_eq!(triple_string(src).unwrap().1, "  keep two");
    }

    #[test]
    fn triple_line_indented_less_than_closing() {
        let src = "\"\"\"\n    fine\n  too far left\n    \"\"\"";
        let error = triple_string(src).unwrap_err();
        match error {
            nom::Err::Failure(error) => assert!(format!("{error:?}").contains("indented less"), "{error:?}"),
            other => panic!("expected a failure, got {other:?}"),
        }
    }

    #[test]
    fn triple_closing_after_text_uses_common_indent() {
        let src = "\"\"\"\n    first\n      second\n    last\"\"\"";
        assert_eq!(triple_string(src).unwrap().1, "first\n  second\nlast");
    }

    #[test]
    fn triple_text_after_opening_quotes() {
        let src = "\"\"\"Title\n    body\n    \"\"\"";
        assert_eq!(triple_string(src).unwrap().1, "Title\nbody");
    }

    #[test]
    fn triple_windows_line_endings() {
        let src = "\"\"\"\r\n    one\r\n\r\n    two\r\n    \"\"\"";
        assert_eq!(triple_string(src).unwrap().1, "one\n\ntwo");
    }

    #[test]
    fn triple_blank_lines_and_tabs() {
        let src = "\"\"\"\n\tone\n   \n\t\ttwo\n\t\"\"\"";
        assert_eq!(triple_string(src).unwrap().1, "one\n\n\ttwo");
        let mixed = "\"\"\"\n    spaces\n\ttab\n    \"\"\""; // a tab isn't four spaces
        assert!(triple_string(mixed).is_err());
    }

    #[test]
    fn triple_empty_multiline() {
        assert_eq!(triple_string("\"\"\"\n\"\"\"").unwrap().1, "");
        assert_eq!(triple_string("\"\"\"\n    \"\"\"").unwrap().1, "");
    }

    #[test]
    fn triple_unterminated() {
        assert!(triple_string("\"\"\"never closed").is_err());
    }

    #[test]
    fn triple_not_a_plain_literal() {
        // """ is an expression (it can hold ${expr}); "" is still the empty string
        assert_eq!(string("\"\"").unwrap().1, "".into());
    }
}
