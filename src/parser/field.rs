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

use imbl::vector;
use nanoid::nanoid;
use nom::{branch::alt, bytes::complete::tag, character::complete::{char, multispace0}, combinator::{map, opt}, sequence::{delimited, pair, preceded, terminated}, IResult, Parser};
use rustc_hash::FxHashMap;
use crate::{model::{Field, FieldDoc}, parser::{context::ParseContext, doc::document_statement, expr::expr, ident::ident, parse_attributes, string::{double_string, single_string}, types::parse_type, whitespace::{doc_comment, whitespace}}, runtime::{Val, Variable}};


/// Parse a field into a parse context.
pub fn parse_field<'a>(input: &'a str, context: &mut ParseContext) -> IResult<&'a str, ()> {
    // Doc comments & whitespace before a field definition
    let (input, mut comments) = doc_comment(input)?;

    let mut attributes = FxHashMap::default();
    let (input, attrs) = parse_attributes(input, context)?;
    for (k, v) in attrs { attributes.insert(k, v); }

    let (input, more_comments) = doc_comment(input)?;
    if more_comments.len() > 0 { if comments.len() > 0 { comments.push('\n'); }  comments.push_str(&more_comments); }

    let (input, attrs) = parse_attributes(input, context)?;
    for (k, v) in attrs { attributes.insert(k, v); }
    let (input, _) = whitespace(input)?; // clean up anything more before signature...

    // Optionally a const field
    let (input, is_const) = opt(terminated(tag("const"), multispace0)).parse(input)?;

    // Type (optional) and name
    let (input, (field_type, name)) = alt((
        map(pair(terminated(parse_type, multispace0), alt((
            map(ident, |v| v.to_string()),
            double_string,
            single_string
        ))), |(ty, nm)| (Some(ty), nm)),
        map(alt((
            map(ident, |v| v.to_string()),
            double_string,
            single_string
        )), |nm| (None, nm))
    )).parse(input)?;

    // Separator
    let (input, _) = delimited(multispace0, char(':'), multispace0).parse(input)?;

    // Value (variable)
    let (input, mut value) = value(input, &name, context)?;
    if is_const.is_some() {
        value.mutable = false; // this field is const
    }
    if let Some(cast_type) = field_type {
        if let Err(_error) = value.cast(&cast_type, &mut context.graph) {
            return Err(nom::Err::Failure(nom::error::Error {
                input: "cast error",
                code: nom::error::ErrorKind::Fail
            }));
        }
        value.vtype = Some(cast_type); // keep the field this type when assigning in the future
    }

    // Optionally end the field declaration with a semicolon
    let (input, _) = opt(preceded(multispace0, char(';'))).parse(input)?;

    // Instert the new field in the current parse context
    let field = Field::new(value, Some(attributes));
    let self_ptr = context.self_ptr();
    let field_ref = context.graph.insert_stof_data(&self_ptr, &name, Box::new(field), None).expect("failed to insert a parsed field into this context");

    // Insert the field doc comments also if requested
    if context.docs && comments.len() > 0 {
        context.graph.insert_stof_data(&self_ptr, &format!("{name}_field_docs"), Box::new(FieldDoc {
            docs: comments,
            field: field_ref
        }), None);
    }

    Ok((input, ()))
}


/// Parse a field value.
fn value<'a>(input: &'a str, name: &str, context: &mut ParseContext) -> IResult<&'a str, Variable> {
    // Try an object value first
    let obj_res = object_value(input, name, context);
    match obj_res {
        Ok((input, var)) => {
            return Ok((input, var));
        },
        Err(error) => {
            match error {
                nom::Err::Failure(_) => {
                    return Err(error);
                },
                _ => {} // keep trying the others
            }
        }
    }

    // Try an array value next
    let arr_res = array_value(input, name, context);
    match arr_res {
        Ok((input, var)) => {
            return Ok((input, var));
        },
        Err(error) => {
            match error {
                nom::Err::Failure(_) => {
                    return Err(error);
                },
                _ => {} // keep trying the others
            }
        }
    }

    // Finally try an expression
    let (input, expr) = expr(input)?;
    match context.eval(expr) {
        Ok(val) => {
            Ok((input, Variable::val(val)))
        },
        Err(_) => {
            Err(nom::Err::Error(nom::error::Error {
                input: "failure",
                code: nom::error::ErrorKind::Fail
            }))
        }
    }
}


/// Array value.
fn array_value<'a>(input: &'a str, _name: &str, context: &mut ParseContext) -> IResult<&'a str, Variable> {
    let (input, _) = char('[')(input)?;
    let (mut input, _) = whitespace(input)?;
    let mut values = vector![];
    loop {
        let res = value(input, &nanoid!(17), context);
        match res {
            Ok((rest, var)) => {
                input = rest;
                values.push_back(var.val);
            },
            Err(error) => {
                return Err(error); // not a valid value
            },
        }

        let (rest, del) = alt((
            delimited(whitespace, char(','), whitespace),
            preceded(whitespace, char(']'))
        )).parse(input)?;
        input = rest;
        if del == ']' { break; } // end of the array
    }
    Ok((input, Variable::val(Val::List(values))))
}


/// Create a new object and parse it for this field's value.
fn object_value<'a>(input: &'a str, name: &str, context: &mut ParseContext) -> IResult<&'a str, Variable> {
    let (mut input, _) = char('{')(input)?;
    let value = context.push_self(name, true);
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

    // Optional object cast at the end (useful when creating arrays especially)
    let (input, cast_type) = opt(preceded(preceded(multispace0, tag("as")), preceded(multispace0, parse_type))).parse(input)?;
    if let Some(cast_type) = cast_type {
        if let Err(_error) = value.cast(&cast_type, &mut context.graph) {
            return Err(nom::Err::Failure(nom::error::Error {
                input: "cast error",
                code: nom::error::ErrorKind::Fail
            }));
        }
    }

    Ok((input, value))
}


#[cfg(test)]
mod tests {
    use crate::{model::Graph, parser::{context::ParseContext, field::parse_field}};

    #[test]
    fn basic_field() {
        let mut graph = Graph::default();
        let mut context = ParseContext::new(&mut graph);
        context.docs = true;

        let (_input, ()) = parse_field(r#"
 
        // This is an ignored comment
        #[test('hello')]
        /**
         * # This is a test field.
         */
        #[another] // heres another ignored comment.
        const field: {
            subfield: 56;
        }

        "#, &mut context).unwrap();

        graph.dump(true);
    }
}
