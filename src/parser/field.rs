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

use nom::{branch::alt, bytes::complete::tag, character::complete::{char, multispace0}, combinator::{map, opt}, sequence::{delimited, terminated}, IResult, Parser};
use rustc_hash::FxHashMap;
use crate::{model::{Field, FieldDoc}, parser::{context::ParseContext, ident::ident, parse_attributes, string::{double_string, single_string}, types::parse_type, whitespace::{doc_comment, whitespace}}, runtime::{Val, Variable}};


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

    // Optionally typed field
    let (input, field_type) = opt(terminated(parse_type, multispace0)).parse(input)?;

    // Field name - name, "name", or 'name'
    let (input, name) = alt((
        map(ident, |v| v.to_string()),
        double_string,
        single_string
    )).parse(input)?;

    // Separator
    let (input, _) = delimited(multispace0, char(':'), multispace0).parse(input)?;

    // Value (variable) TODO

    // Instert the new field in the current parse context
    let field = Field::new(Variable::val(Val::Null), Some(attributes));
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
