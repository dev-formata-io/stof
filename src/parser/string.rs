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

use nom::{branch::alt, bytes::complete::{escaped_transform, tag, take_until}, character::complete::{char, none_of}, combinator::{into, value}, sequence::delimited, IResult, Parser};
use crate::{parser::whitespace::whitespace, runtime::Val};


/// Parse a string literal value.
pub fn string(input: &str) -> IResult<&str, Val> {
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
pub(self) fn raw_double_string(input: &str) -> IResult<&str, String> {
    into(delimited(tag("r#\""), take_until("\"#"), tag("\"#"))).parse(input)
}

/// Parse a double quoted string.
pub fn double_string(input: &str) -> IResult<&str, String> {
    let normal = none_of("\"\\"); // everything but backslash or double quote
    let inner = escaped_transform(normal, '\\', alt((
        value("\\", tag("\\")),
        value("\"", tag("\"")),
        value("\n", tag("n")),
        value("\r", tag("r")),
        value("\t", tag("t")),
    )));
    delimited(char('"'), inner, char('"')).parse(input)
}

/// Parse a single quoted string.
pub fn single_string(input: &str) -> IResult<&str, String> {
    let normal = none_of("'\\"); // everything but backslash or double quote
    let inner = escaped_transform(normal, '\\', alt((
        value("\\", tag("\\")),
        value("'", tag("'")),
        value("\n", tag("n")),
        value("\r", tag("r")),
        value("\t", tag("t")),
    )));
    delimited(char('\''), inner, char('\'')).parse(input)
}


#[cfg(test)]
mod tests {
    use crate::parser::string::{double_string, raw_double_string, single_string, string};

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
}
