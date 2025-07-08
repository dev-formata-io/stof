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

use crate::{model::InnerDoc, parser::{context::ParseContext, field::parse_field, func::parse_function, whitespace::{parse_inner_doc_comment, whitespace}}, runtime::Error};
use nanoid::nanoid;
use nom::{character::complete::char, combinator::eof, Err, IResult};


/// Parse a Stof document into a context (graph).
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
/// TODO types
/// TODO extern
/// TODO import
/// TODO ref field
pub fn document_statement<'a>(input: &'a str, context: &mut ParseContext) -> IResult<&'a str, ()> {
    // Function
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

    // Field
    {
        let func_res = parse_field(input, context);
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

    // JSON-like brackets
    {
        let json_res = json_statements(input, context);
        match json_res {
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

    // Inner comment?
    if let Ok((input, docs)) = parse_inner_doc_comment(input) {
        if context.docs {
            let self_ptr = context.self_ptr();
            context.graph.insert_stof_data(&self_ptr, &nanoid!(15), Box::new(InnerDoc { docs }), None);
        }
        return Ok((input, ()));
    }

    // Whitespace in the document
    if let Ok((input, _)) = whitespace(input) {
        return Ok((input, ()));
    }

    // End of the document?
    let (input, _) = eof(input)?;
    Ok((input, ()))
}


/// Empty brackets around some statements (accepts JSON).
fn json_statements<'a>(input: &'a str, context: &mut ParseContext) -> IResult<&'a str, ()> {
    let (mut input, _) = char('{')(input)?;
    loop {
        let res = document_statement(input, context);
        match res {
            Ok((rest, _)) => {
                input = rest;
                if input.starts_with('}') {
                    break;
                }
            },
            Err(error) => {
                return Err(error);
            }
        }
    }
    let (input, _) = char('}')(input)?;
    Ok((input, ()))
}
