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

use crate::{parser::{context::ParseContext, func::parse_function, whitespace::whitespace}, runtime::Error};
use nom::{combinator::eof, Err, IResult};


/// Parse a Stof document into a context (graph).
/// TODO types
/// TODO extern
/// TODO import
/// TODO field
/// TODO ref field
/// TODO json fields (object value outright)
/// TODO inner comment
pub fn document(mut input: &str, context: &mut ParseContext) -> Result<(), Error> {
    loop {
        let res = document_statement(input, context);
        match res {
            Ok((rest, _)) => {
                if rest.is_empty() { break; }
                input = rest;
            },
            Err(error) => {
                // didn't match a singular statement (including whitespace)
                return Err(Error::ParseFailure(error.to_string()));
            }
        }
    }
    Ok(())
}


/// Parse a singular document statement.
pub fn document_statement<'a>(input: &'a str, context: &mut ParseContext) -> IResult<&'a str, ()> {
    // Function?
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

    // Whitespace in the document
    if let Ok((input, _)) = whitespace(input) {
        return Ok((input, ()));
    }

    // End of the document?
    let (input, _) = eof(input)?;
    Ok((input, ()))
}
