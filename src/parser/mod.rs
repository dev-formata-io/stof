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

pub mod semver;
pub use semver::*;

pub mod whitespace;
pub mod number;
pub mod types;

use nom::{character::complete::{alpha1, alphanumeric0}, combinator::recognize, sequence::pair, IResult, Parser};


/// Parse an identifier.
pub fn ident(input: &str) -> IResult<&str, &str> {
    recognize(
        pair(
            alpha1,
            alphanumeric0
        )
    ).parse(input)
}


#[cfg(test)]
mod tests {
    use crate::parser::ident;

    #[test]
    fn ident_parse() {
        assert_eq!(ident("a").unwrap().1, "a");
        assert_eq!(ident("a1345: str").unwrap().1, "a1345");
        assert!(ident("1").is_err());
    }
}
