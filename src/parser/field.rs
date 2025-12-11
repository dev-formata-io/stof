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

use std::ops::Deref;
use colored::Colorize;
use imbl::vector;
use nanoid::nanoid;
use nom::{branch::alt, bytes::complete::{tag, take_until}, character::complete::{char, multispace0, multispace1, space0}, combinator::{map, opt, peek, recognize}, sequence::{delimited, pair, preceded, terminated}, IResult, Parser};
use rustc_hash::FxHashMap;
use crate::{model::{Field, FieldDoc, SId, NOFIELD_FIELD_ATTR}, parser::{context::ParseContext, doc::{document_statement, err_fail, StofParseError}, expr::expr, ident::ident, parse_attributes, string::{double_string, single_string}, types::parse_type, whitespace::{doc_comment, whitespace}}, runtime::{Val, Variable}};


/// Parse a field into a parse context.
pub fn parse_field<'a>(input: &'a str, context: &mut ParseContext) -> IResult<&'a str, (), StofParseError> {
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
        map(pair(terminated(parse_type, multispace1), alt((
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
    let (input, mut value) = value(input, &name, context, &mut attributes).map_err(err_fail)?;
    if is_const.is_some() {
        value.mutable = false; // this field is const
    }
    if let Some(cast_type) = field_type {
        let context_node = Some(context.self_ptr());
        if let Err(error) = value.cast(&cast_type, &mut context.graph, context_node) {
            let message = format!("'{name}' ({:?}) cannot be cast to type '{}': {}", value.val.read().deref(), cast_type.rt_type_of(&context.graph), error.to_string());
            return Err(nom::Err::Failure(StofParseError::from(format!("{} {message}", "field cast error:".dimmed()))));
        }
        value.vtype = Some(cast_type); // keep the field this type when assigning in the future
    }

    // Optionally end the field declaration with a semicolon or a comma
    let (input, _) = opt(preceded(multispace0, alt((char(';'), char(','))))).parse(input).map_err(err_fail)?;

    // check for #[no-field] on an object value, which only creates the object, not a field in addition
    // this is typically done within/for the stof export
    if value.try_obj().is_some() && attributes.contains_key(NOFIELD_FIELD_ATTR.as_str()) {
        return Ok((input, ()));
    }
    
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
fn value<'a>(input: &'a str, name: &str, context: &mut ParseContext, attributes: &mut FxHashMap<String, Val>) -> IResult<&'a str, Variable, StofParseError> {
    // Try an object value first
    let obj_res = object_value(input, name, context, attributes);
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
        Err(err) => {
            Err(nom::Err::Error(StofParseError::from(err.to_string())))
        }
    }
}


/// Array value.
fn array_value<'a>(input: &'a str, _name: &str, context: &mut ParseContext) -> IResult<&'a str, Variable, StofParseError> {
    let (input, _) = char('[')(input)?;
    let (mut input, _) = whitespace(input)?;
    let mut values = vector![];
    let mut default_attrs = FxHashMap::default();
    loop {
        let res = value(input, &nanoid!(17), context, &mut default_attrs);
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
            preceded(whitespace, recognize((tag(","), preceded(whitespace, tag("]"))))),
            delimited(whitespace, tag(","), whitespace),
            preceded(whitespace, tag("]"))
        )).parse(input)?;
        input = rest;
        if del.contains(']') { break; } // end of the array
    }
    Ok((input, Variable::val(Val::List(values))))
}


/// Create a new object and parse it for this field's value.
fn object_value<'a>(input: &'a str, name: &str, context: &mut ParseContext, attributes: &mut FxHashMap<String, Val>) -> IResult<&'a str, Variable, StofParseError> {
    let (input, _) = char('{')(input)?;

    // Optional custom object ID - not recommended unless you know what you're doing
    let (mut input, custom_id) = opt(delimited(
        space0,
        delimited(char('('), take_until(")"), char(')')),
        whitespace,
    )).parse(input)?;
    let mut cid = None;
    if let Some(id) = custom_id { cid = Some(SId::from(id)); }

    let value = context.push_self(name, attributes, cid);
    if !input.starts_with('}') { // account for an empty object case "{}"
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
    }
    context.pop_self();
    context.post_init_obj(&value, attributes).expect("error initializing new object field value");
    let (input, _) = char('}')(input)?;

    // Peek at the next value, if its async, then don't do the as below...
    let (input, peek_async) = opt(peek(preceded(multispace0, tag("async")))).parse(input)?;
    if peek_async.is_some() {
        return Ok((input, value));
    }

    // Optional object cast at the end (useful when creating arrays especially)
    let (input, cast_type) = opt(preceded(preceded(multispace0, tag("as")), preceded(multispace0, parse_type))).parse(input)?;
    if let Some(cast_type) = cast_type {
        let context_node = Some(context.self_ptr());
        if let Err(error) = value.cast(&cast_type, &mut context.graph, context_node) {
            let message = format!("'{}' cannot be cast to type '{}': {}", name, cast_type.rt_type_of(&context.graph), error.to_string());
            return Err(nom::Err::Failure(StofParseError::from(format!("{} {message}", "field obj cast 'as':".dimmed()))));
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
        {
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
        }

        graph.dump(true);
    }
}
