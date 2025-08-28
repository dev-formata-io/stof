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

use arcstr::ArcStr;
use nom::{branch::alt, bytes::complete::take_while, character::complete::{char, digit1, space0}, combinator::{all_consuming, map_res, map}, sequence::{delimited, preceded}, AsChar, IResult, Parser};
use crate::{parser::doc::StofParseError, runtime::Val};


/// Parse a Semantic Version alone.
pub fn parse_semver_alone(input: &str) -> Option<Val> {
    match semver_complete(input) {
        Ok(val) => {
            Some(val.1)
        },
        Err(_) => {
            None
        }
    }
}
fn semver_complete(input: &str) -> IResult<&str, Val, StofParseError> {
    all_consuming(parse_semver).parse(input)
}

/// Parse a Semantic Version value.
pub fn parse_semver(input: &str) -> IResult<&str, Val, StofParseError> {
    delimited(
        space0,
    alt(
            (
                parse_semver_full,
                parse_semver_release,
                parse_semver_build,
                parse_semver_basic
            )
        ),
        space0
    ).parse(input)
}
pub fn parse_semver_basic(input: &str) -> IResult<&str, Val, StofParseError> {
    map(
        parse_vers,
        |vers| Val::Ver(vers.0, vers.1, vers.2, None, None)
    ).parse(input)
}
pub fn parse_semver_release(input: &str) -> IResult<&str, Val, StofParseError> {
    map(
        (
            parse_vers,
            parse_ver_release,
        ),
        |(vers, rel)| Val::Ver(vers.0, vers.1, vers.2, Some(ArcStr::from(rel)), None)
    ).parse(input)
}
pub fn parse_semver_build(input: &str) -> IResult<&str, Val, StofParseError> {
    map(
        (
            parse_vers,
            parse_ver_build,
        ),
        |(vers, build)| Val::Ver(vers.0, vers.1, vers.2, None, Some(ArcStr::from(build)))
    ).parse(input)
}
pub fn parse_semver_full(input: &str) -> IResult<&str, Val, StofParseError> {
    map(
        (
            parse_vers,
            parse_ver_release,
            parse_ver_build,
        ),
        |(vers, rel, build)| Val::Ver(vers.0, vers.1, vers.2, Some(ArcStr::from(rel)), Some(ArcStr::from(build)))
    ).parse(input)
}
fn parse_vers(input: &str) -> IResult<&str, (i32, i32, i32), StofParseError> {
    (
        parse_ver,
        preceded(char('.'), parse_ver),
        preceded(char('.'), parse_ver),
    ).parse(input)
}
fn parse_ver(input: &str) -> IResult<&str, i32, StofParseError> {
    map_res(digit1, str::parse).parse(input)
}
fn parse_ver_release(input: &str) -> IResult<&str, &str, StofParseError> {
    preceded(
        char('-'),
        take_while(|c| AsChar::is_alphanum(c) || c == '-' || c == '.')
    ).parse(input)
}
fn parse_ver_build(input: &str) -> IResult<&str, &str, StofParseError> {
    preceded(
        char('+'),
        take_while(|c| AsChar::is_alphanum(c) || c == '-')
    ).parse(input)
}


#[cfg(test)]
mod tests {
    use crate::parser::semver::parse_semver_alone;

    #[test]
    fn semver() {
        assert!(parse_semver_alone("  0.0.5  ").unwrap().ver());
        assert!(parse_semver_alone("1.43.5-beta+123").unwrap().ver());
        assert!(parse_semver_alone("1.43.5-beta.3456+123").unwrap().ver());

        assert!(parse_semver_alone("1.43.5-beta +123").is_none());
        assert!(parse_semver_alone("   1.4 3.5    - b eta  ").is_none());
        assert!(parse_semver_alone("1.43").is_none());
    }
}
