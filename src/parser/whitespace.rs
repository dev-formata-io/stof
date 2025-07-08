//
// Copyright 2024 Formata, Inc. All rights reserved.
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

use nom::{branch::alt, bytes::complete::{tag, take_until}, character::complete::{multispace0, multispace1, not_line_ending}, multi::separated_list0, IResult, Parser};


/// Doc comment.
/// Also eats up whitespace before and after.
pub fn doc_comment(input: &str) -> IResult<&str, String> {
    let (input, _) = whitespace(input)?;
    let (input, inner) = separated_list0(multispace0, alt((parse_block_doc_comment, parse_single_line_doc_comment))).parse(input)?;

    let mut comments = String::default();
    for inner in inner { comments.push_str(inner); }
    let (input, _) = whitespace(input)?;

    // filter * from the start of all comment lines if they start with it.
    let mut filtered = String::default();
    for mut line in comments.split('\n') {
        line = line.trim();
        if line.starts_with('*') {
            line = line.trim_start_matches('*').trim(); // get to the content
        }
        filtered.push_str(&format!("{line}\n")); // preserves spacing & formatting
    }
    filtered = filtered.trim().into();

    Ok((input, filtered))
}

/// Whitespace.
pub fn whitespace(input: &str) -> IResult<&str, &str> {
    let mut rest = input;
    while let Ok(res) = alt((
        parse_block_comment,
        parse_single_line_comment,
        multispace1
    )).parse(rest) {
        rest = res.0;
    }
    Ok((rest, ""))
}

/// Parse a single line comment "// comment here \n"
pub(self) fn parse_single_line_comment(input: &str) -> IResult<&str, &str> {
    let (input, _) = tag("//").parse(input)?;

    if input.starts_with('/') {
        // this is actually a doc comment, so error
        return Err(nom::Err::Error(nom::error::Error { input: "", code: nom::error::ErrorKind::IsNot }));
    }

    let (input, out) = not_line_ending(input)?;
    Ok((input, out))
}

/// Parse a block style comment.
pub(self) fn parse_block_comment(input: &str) -> IResult<&str, &str> {
    let (input, _) = tag("/*").parse(input)?;
    
    if input.starts_with('*') || input.starts_with('!') {
        // this is actually a doc comment, so error
        return Err(nom::Err::Error(nom::error::Error { input: "", code: nom::error::ErrorKind::IsNot }));
    }

    let (input, _) = take_until("*/").parse(input)?;
    let (input, out) = tag("*/").parse(input)?;
    Ok((input, out))
}

/// Parse a single line doc comment "/// comment here \n"
pub(self) fn parse_single_line_doc_comment(input: &str) -> IResult<&str, &str> {
    let (input, _) = tag("///").parse(input)?;
    let (input, out) = not_line_ending(input)?;
    Ok((input, out))
}

/// Parse a block style doc comment.
pub(self) fn parse_block_doc_comment(input: &str) -> IResult<&str, &str> {
    let (input, _) = tag("/**").parse(input)?;
    let (input, out) = take_until("*/").parse(input)?;
    let (input, _) = tag("*/").parse(input)?;
    Ok((input, out))
}

/// Parse a block style inner doc comment.
pub fn parse_inner_doc_comment(input: &str) -> IResult<&str, String> {
    let (input, _) = tag("/*!").parse(input)?;
    let (input, out) = take_until("*/").parse(input)?;
    let (input, _) = tag("*/").parse(input)?;

    // filter * from the start of all comment lines if they start with it.
    let mut filtered = String::default();
    for mut line in out.split('\n') {
        line = line.trim();
        if line.starts_with('*') {
            line = line.trim_start_matches('*').trim(); // get to the content
        }
        filtered.push_str(&format!("{line}\n")); // preserves spacing & formatting
    }
    filtered = filtered.trim().into();

    Ok((input, filtered))
}


#[cfg(test)]
mod tests {
    use crate::parser::whitespace::{doc_comment, parse_block_comment, parse_single_line_comment, whitespace};

    #[test]
    fn single_line_comment() {
        let res = parse_single_line_comment("// This is a comment\n").unwrap();
        assert_eq!(res.0, "\n");
    }

    #[test]
    fn block_comment() {
        let res = parse_block_comment(r#"/*
         * This is a block comment!
         * With many lines.
         */hello"#).unwrap();
        assert_eq!(res.0, "hello");
    }

    #[test]
    fn whitespace_test() {
        let res = whitespace(r#"
        
            // This is a line comment.

            /*
             * This is a block comment.
             */

            /* This is another block. */

            hello"#).unwrap();
        assert_eq!(res.0, "hello");
    }

    #[test]
    fn doc_comments() {
        let res = doc_comment(r#"
        /**
         * This is a doc comment!
         * 
         * With many lines.
         */
        /// This is a line doc comment.
        hello"#).unwrap();
        assert_eq!(res.0, "hello");
        assert_eq!(res.1, r#"This is a doc comment!

With many lines.
This is a line doc comment."#);
    }
}
