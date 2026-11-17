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

use nom::{branch::alt, bytes::complete::tag, character::complete::satisfy, combinator::{not, peek, value}, sequence::terminated, IResult, Parser};
use crate::{parser::{doc::StofParseError, number::number, semver::parse_semver, string::string, whitespace::whitespace}, runtime::Val};


/// Parse a literal value (bool, null, number, string, or version).
pub fn literal(input: &str) -> IResult<&str, Val, StofParseError> {
    let (input, _) = whitespace(input)?;
    alt((
        value(Val::Null, word("null")),
        value(Val::Bool(true), word("true")),
        value(Val::Bool(false), word("false")),
        string,
        parse_semver,
        number,
    )).parse(input)
}

/// A keyword literal as a whole word, so a name that starts with one (`nullable`, `trueish`, `falsey`) is a name.
fn word<'a>(keyword: &'static str) -> impl Parser<&'a str, Output = &'a str, Error = StofParseError> {
    terminated(tag(keyword), not(peek(satisfy(|c: char| c.is_alphanumeric() || c == '_'))))
}
