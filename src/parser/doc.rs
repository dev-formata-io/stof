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

use crate::{model::InnerDoc, parser::{context::ParseContext, data::parse_data, field::parse_field, func::parse_function, ident::ident, import::import, whitespace::{parse_inner_doc_comment, whitespace_fail}}, runtime::Error};
use nanoid::nanoid;
use nom::{bytes::complete::tag, character::complete::{char, multispace0}, combinator::{eof, opt}, error::{ErrorKind, FromExternalError, ParseError}, sequence::{delimited, preceded}, Err, IResult, Parser};
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StofParseError {
    pub file_path: Option<String>,
    pub message: String,
}
impl From<&str> for StofParseError {
    fn from(value: &str) -> Self {
        Self {
            file_path: None,
            message: value.to_string(),
        }
    }
}
impl From<String> for StofParseError {
    fn from(value: String) -> Self {
        Self {
            file_path: None,
            message: value,
        }
    }
}
/// Map a Stof error from an error to a failure.
/// Used in making sure things fail at the document level.
pub fn err_fail(e: nom::Err<StofParseError>) -> nom::Err<StofParseError> {
    match e {
        nom::Err::Error(e) => nom::Err::Failure(e),
        _ => e
    }
}
impl ParseError<&str> for StofParseError {
    // on one line, we show the error code and the input that caused it
    fn from_error_kind(input: &str, kind: ErrorKind) -> Self {
        let message = format!("{:?}: {input}\n", kind, );
        StofParseError { message, file_path: None }
    }

    // if combining multiple errors, we show them one after the other
    fn append(_input: &str, _kind: ErrorKind, other: Self) -> Self {
        //let message = format!("{}{:?}:\t{:?}\n", other.message, kind, input);
        //StofParseError { message, file_path: None }
        other
    }

    fn from_char(input: &str, c: char) -> Self {
        let message = format!("expected char '{c}':\n{input}");
        StofParseError { message, file_path: None }
    }

    fn or(self, other: Self) -> Self {
        //let message = format!("{}\tOR\n{}\n", self.message, other.message);
        //StofParseError { message, file_path: None }
        other
    }
}
impl FromExternalError<&str, std::num::ParseIntError> for StofParseError {
    fn from_external_error(_input: &str, _kind: ErrorKind, e: std::num::ParseIntError) -> Self {
        Self::from(e.to_string())
    }
}


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
                match error {
                    nom::Err::Error(e) => {
                        return Err(Error::ParseError(e));
                    },
                    nom::Err::Failure(e) => {
                        return Err(Error::ParseError(e));
                    },
                    nom::Err::Incomplete(_) => {
                        return Err(Error::ParseError(StofParseError::from(error.to_string())));
                    }
                }
            }
        }
    }
    Ok(())
}


/// Parse a singular document statement.
pub fn document_statement<'a>(input: &'a str, context: &mut ParseContext) -> IResult<&'a str, (), StofParseError> {
    // Field
    {
        let field_res = parse_field(input, context);
        match field_res {
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

    // Import
    {
        let import_res = import(input, context);
        match import_res {
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

    // Data (binary)
    {
        let data_res = parse_data(input, context);
        match data_res {
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

    // New root object + statements
    {
        let root_res = root_statements(input, context);
        match root_res {
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
    if let Ok((input, _)) = whitespace_fail(input) {
        return Ok((input, ()));
    }

    // End of the document?
    let (input, _) = eof(input)?;
    Ok((input, ()))
}


/// New root document node statements.
fn root_statements<'a>(input: &'a str, context: &mut ParseContext) -> IResult<&'a str, (), StofParseError> {
    let (input, name) = preceded(tag("root"), delimited(multispace0, opt(ident), multispace0)).parse(input)?;
    let (mut input, _) = char('{')(input)?;
    context.push_root(name);
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
    context.pop_self();
    let (input, _) = char('}')(input)?;
    Ok((input, ()))
}


/// Empty brackets around some statements (accepts JSON).
fn json_statements<'a>(input: &'a str, context: &mut ParseContext) -> IResult<&'a str, (), StofParseError> {
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


#[cfg(test)]
mod tests {
    use crate::{model::Graph, parser::{context::ParseContext, doc::document}, runtime::{Runtime, Val}};

    #[test]
    fn basic_doc() {
        let mut graph = Graph::default();
        {
            let mut context = ParseContext::new(&mut graph);
            context.docs = true;

            document(r#"

            {
                "max": 200

                "object": {
                    "dude": true,
                    "hello": 450
                }

                list subobj: [
                    {
                        fn hello() -> str { 'hi' }
                    } as obj,
                    {
                        field: 'dude'
                    }
                ];

                async fn another_yet(max: int = self.max) -> int {
                    let total = 0;
                    for (let i = 0; i < max; i += 1) total += 1;
                    total
                }
        
                fn main(x: float = 5) -> float {
                    let a = self.another_yet();
                    let b = self.another_yet(4000);
                    let c = self.another_yet(1000);
                    let d = self.another_yet(800);

                    (await a) + (await b) + (await c) + (await d)
                }
            }

            "#, &mut context).unwrap();
        }

        graph.dump(true);

        let res = Runtime::call(&mut graph, "root.main", vec![Val::from(10)]).unwrap();
        assert_eq!(res, 6000.into());
    }
}
