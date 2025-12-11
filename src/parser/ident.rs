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

use nom::{bytes::complete::take_while1, combinator::{opt, recognize}, AsChar, IResult, Parser};
use crate::parser::doc::StofParseError;


/// Parse an identifier.
/// Identifiers are used as function names, field names, variable names, etc.
pub fn ident(input: &str) -> IResult<&str, &str, StofParseError> {
    recognize(
(
            take_while1(|c| AsChar::is_alpha(c) || c == '_' || c == '@' || c == '<'),
            opt(take_while1(|c| AsChar::is_alphanum(c) || c == '_' || c == '@' || c == '-' || c == '<' || c == '>'))
        )
    ).parse(input)
}


/// Parse an identifier.
/// Identifiers are used as function names, field names, variable names, etc.
pub fn ident_type(input: &str) -> IResult<&str, &str, StofParseError> {
    recognize(
(
            take_while1(|c| AsChar::is_alpha(c) || c == '_' || c == '@'),
            opt(take_while1(|c| AsChar::is_alphanum(c) || c == '_' || c == '@' || c == '-'))
        )
    ).parse(input)
}


#[cfg(test)]
mod tests {
    use crate::parser::ident::ident;

    #[test]
    fn ident_parse() {
        assert_eq!(ident("a").unwrap().1, "a");
        assert_eq!(ident("a1345: str").unwrap().1, "a1345");
        assert!(ident("1").is_err());
    }

    #[test]
    fn start_underscore() {
        assert_eq!(ident("_a").unwrap().1, "_a");
        assert_eq!(ident("__a1345: str").unwrap().1, "__a1345");
    }

    #[test]
    fn start_at() {
        assert_eq!(ident("@a").unwrap().1, "@a");
        assert_eq!(ident("@a13@45: str").unwrap().1, "@a13@45");
    }
}
