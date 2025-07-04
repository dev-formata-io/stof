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

use nom::{branch::alt, bytes::complete::{tag, take_until}, character::complete::{multispace1, not_line_ending}, IResult, Parser};


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
pub fn parse_single_line_comment(input: &str) -> IResult<&str, &str> {
    let (input, _) = tag("//").parse(input)?;
    let (input, out) = not_line_ending(input)?;
    Ok((input, out))
}

/// Parse a block style comment.
pub fn parse_block_comment(input: &str) -> IResult<&str, &str> {
    let (input, _) = tag("/*").parse(input)?;
    let (input, _) = take_until("*/").parse(input)?;
    let (input, out) = tag("*/").parse(input)?;
    Ok((input, out))
}


#[cfg(test)]
mod tests {
    use crate::parser::whitespace::{parse_block_comment, parse_single_line_comment, whitespace};

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
}
