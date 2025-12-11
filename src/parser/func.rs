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

use std::sync::Arc;
use nom::{bytes::complete::tag, branch::alt, character::complete::{char, multispace0}, combinator::opt, multi::separated_list0, sequence::{delimited, preceded, terminated}, IResult, Parser};
use crate::{model::{Func, FuncDoc, Param, SId, ASYNC_FUNC_ATTR}, parser::{context::ParseContext, doc::{err_fail, StofParseError}, expr::expr, ident::ident, parse_attributes, statement::block, types::parse_type, whitespace::{doc_comment, whitespace}}, runtime::{instruction::Instruction, instructions::Base, Val}};


/// Parse a function into a parse context.
pub fn parse_function<'a>(input: &'a str, context: &mut ParseContext) -> IResult<&'a str, (), StofParseError> {
    let mut func = Func::default();

    // Doc comments & whitespace before a function definition
    let (input, mut comments) = doc_comment(input)?;

    let mut do_create_func;
    let (input, (attrs, do_insert)) = parse_attributes(input, context)?;
    for (k, v) in attrs { func.attributes.insert(k, v); }
    do_create_func = do_insert;

    let (input, more_comments) = doc_comment(input)?;
    if more_comments.len() > 0 { if comments.len() > 0 { comments.push('\n'); }  comments.push_str(&more_comments); }

    let (input, (attrs, do_insert)) = parse_attributes(input, context)?;
    for (k, v) in attrs { func.attributes.insert(k, v); }
    do_create_func = do_create_func && do_insert;
    let (input, _) = whitespace(input)?; // clean up anything more before signature...

    let (input, async_fn) = opt(terminated(tag("async"), multispace0)).parse(input)?;
    if async_fn.is_some() && !func.attributes.contains_key(ASYNC_FUNC_ATTR.as_str()) {
        func.attributes.insert(ASYNC_FUNC_ATTR.to_string(), Val::Null);
    }

    let (input, _) = tag("fn").parse(input)?;
    let (input, name) = preceded(multispace0, ident).parse(input).map_err(err_fail)?;
    let (input, params) = delimited(char('('), separated_list0(char(','), alt((parameter, opt_parameter))), char(')')).parse(input).map_err(err_fail)?;
    let (input, return_type) = opt(preceded(delimited(multispace0, tag("->"), multispace0), parse_type)).parse(input).map_err(err_fail)?;
    let (input, instructions) = block(input).map_err(err_fail)?;

    // Check do_create_func now after parse
    if !do_create_func {
        return Ok((input, ()));
    }

    for param in params { func.params.push_back(param); }
    func.return_type = return_type.unwrap_or_default(); // default is void
    func.instructions = instructions;

    // Is this function an init function (has an #[init] attribute)?
    // These functions will get called automatically when the context is dropped (after parse complete).
    let mut init_func = false;
    if func.attributes.contains_key("init") { init_func = true; }

    // Instert the new function in the current parse context
    //println!("({name}){{{func:?}}}");
    let self_ptr = context.self_ptr();
    let func_ref = context.graph.insert_stof_data(&self_ptr, name, Box::new(func), None).expect("failed to insert a parsed function into this context");

    // Insert init if necessary
    if init_func {
        context.init_funcs.push(func_ref.clone());
    }

    // Insert the function doc comments also if requested
    if context.profile.docs && comments.len() > 0 {
        context.graph.insert_stof_data(&self_ptr, &format!("{name}_docs"), Box::new(FuncDoc {
            docs: comments,
            func: func_ref,
        }), None);
    }

    Ok((input, ()))
}


/// Parse a function parameter.
pub fn parameter(input: &str) -> IResult<&str, Param, StofParseError> {
    let (input, _) = multispace0(input)?;
    let (input, name) = ident(input)?;
    let (input, param_type) = preceded(preceded(multispace0, char(':')), preceded(multispace0, parse_type)).parse(input)?;

    let (input, default) = opt(
        preceded(delimited(multispace0, char('='), multispace0), expr)
    ).parse(input)?;
    let (input, _) = multispace0(input)?;

    let param = Param {
        name: SId::from(name),
        param_type,
        default
    };
    Ok((input, param))
}


/// Parse an optional function parameter.
pub fn opt_parameter(input: &str) -> IResult<&str, Param, StofParseError> {
    let (input, _) = multispace0(input)?;
    let (input, name) = ident(input)?;
    let (input, param_type) = preceded(preceded(multispace0, tag("?:")), preceded(multispace0, parse_type)).parse(input)?;

    let (input, default) = opt(
        preceded(delimited(multispace0, char('='), multispace0), expr)
    ).parse(input)?;
    let (input, _) = multispace0(input)?;

    let mut defalt_expr = Arc::new(Base::Literal(Val::Null)) as Arc<dyn Instruction>;
    if let Some(def) = default { defalt_expr = def; } // just in case...

    let param = Param {
        name: SId::from(name),
        param_type,
        default: Some(defalt_expr)
    };
    Ok((input, param))
}


#[cfg(test)]
mod tests {
    use crate::{model::{Graph, Profile}, parser::{context::ParseContext, func::parse_function}, runtime::{Runtime, Val}};

    #[test]
    fn basic_func() {
        let mut graph = Graph::default();
        {
            let mut context = ParseContext::new(&mut graph, Profile::debug());
            let (_input, ()) = parse_function(r#"
    
            // This is an ignored comment
            #[test('hello')]
            /**
             * # This is a test function.
             * This function represents the first ever function in Stof v2.
             */
            #[another] // heres another ignored comment.
            fn main(x: float = 5, optional?: str) -> float { x }

            "#, &mut context).unwrap();
        }

        let res = Runtime::call(&mut graph, "root.main", vec![Val::from(10)]).unwrap();
        assert_eq!(res, 10.into());

        //graph.dump(true);
    }
}
