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

use nom::{branch::alt, bytes::complete::tag, combinator::{map, value}, IResult, Parser};
use crate::{parser::{number::number, semver::parse_semver, string::string, whitespace::whitespace}, runtime::Val};


/// Parse a literal value (bool, null, number, string, or version).
pub fn literal(input: &str) -> IResult<&str, Val> {
    let (input, _) = whitespace(input)?;
    alt((
        value(Val::Null, tag("null")),
        value(Val::Bool(true), tag("true")),
        value(Val::Bool(false), tag("false")),
        string,
        number,
        parse_semver,
    )).parse(input)
}
