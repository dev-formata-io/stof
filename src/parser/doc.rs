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

use crate::{model::InnerDoc, parser::{context::ParseContext, field::parse_field, func::parse_function, whitespace::{parse_inner_doc_comment, whitespace_fail}}, runtime::Error};
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
    if let Ok((input, _)) = whitespace_fail(input) {
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


#[cfg(test)]
mod tests {
    use crate::{model::Graph, parser::{context::ParseContext, doc::document}, runtime::{Runtime, Val}};

    #[test]
    fn basic_doc() {
        let mut graph = Graph::default();
        let mut context = ParseContext::new(&mut graph);
        context.docs = true;

        document(r#"

        {
            "max": 200

            "object": {
                "dude": true,
                "hello": 450
            }

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

        graph.dump(true);

        let res = Runtime::call(&mut graph, "root.main", vec![Val::from(10)]).unwrap();
        assert_eq!(res, 6000.into());
    }
}
