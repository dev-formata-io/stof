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
use imbl::{vector, Vector};
use nom::{bytes::complete::tag, branch::alt, character::complete::{char, multispace0, satisfy}, combinator::{not, opt, peek}, sequence::{delimited, preceded, terminated}, IResult, Parser};
use crate::{model::{Func, FuncDoc, Param, SId, ASYNC_FUNC_ATTR}, parser::{source, context::ParseContext, info::{Def, DefKind, DefSpan}, doc::{err_fail, StofParseError}, expr::expr, ident::ident, parse_attributes, statement::{block, declare::{clear_used_names, take_declared_names, take_referenced_names}}, types::parse_type, whitespace::{doc_comment, paren_list, whitespace}}, runtime::{instruction::Instruction, instructions::Base, Val}};


/// Parse a function into a parse context.
pub fn parse_function<'a>(input: &'a str, context: &mut ParseContext) -> IResult<&'a str, (), StofParseError> {
    let mut func = Func::default();
    let outer_start = input;

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
    let start = input;
    let fn_start = input;

    let (input, async_fn) = opt(terminated(tag("async"), multispace0)).parse(input)?;
    if async_fn.is_some() && !func.attributes.contains_key(ASYNC_FUNC_ATTR.as_str()) {
        func.attributes.insert(ASYNC_FUNC_ATTR.to_string(), Val::Null);
    }

    let (input, _) = tag("fn").parse(input)?;
    let (input, _) = multispace0(input)?;
    let name_pos = input;
    let (input, name) = ident(input).map_err(err_fail)?;
    let (input, params) = paren_list(input, any_parameter).map_err(err_fail)?;
    let (input, return_type) = opt(preceded(delimited(multispace0, tag("->"), multispace0), parse_type)).parse(input).map_err(err_fail)?;
    take_declared_names(); // drop anything declared outside of this function body
    take_referenced_names();
    clear_used_names();
    let (input, instructions) = function_body(input).map_err(err_fail)?;
    let value_end = input;
    let (input, _) = opt(preceded(multispace0, alt((char(';'), char(','))))).parse(input)?; // like a field's value
    let mut var_names = take_declared_names();
    var_names.extend(params.iter().map(|param| param.name.as_ref().to_string()));
    let referenced = take_referenced_names();

    // Check do_create_func now after parse
    if !do_create_func {
        return Ok((input, ()));
    }
    context.warn_if_vars_shadow_lib(name, &var_names);
    context.check_names(name, &var_names, referenced);

    func.params = params.into_iter().collect();
    func.return_type = return_type.unwrap_or_default(); // default is void
    func.instructions = instructions;
    func.src = source::locate(start);
    if source::keep_source_enabled() { func.source = Some(fn_start[..fn_start.len() - input.len()].into()); }

    // Is this function an init function (has an #[init] attribute)?
    // These functions will get called automatically when the context is dropped (after parse complete).
    let mut init_func = false;
    if func.attributes.contains_key("init") { init_func = true; }
    if init_func && context.profile.untrusted {
        return Err(nom::Err::Failure(StofParseError::from(format!("untrusted source can't have #[init] functions (nothing runs while it's parsed): fn {name}"))));
    }
    if context.profile.untrusted && func.attributes.contains_key("load") && context.graph.confined.is_empty() {
        // a #[load] function brings in more code when the host calls load(): code from somewhere else would choose what
        // runs with the document's own access. Allowed when the parse runs in a sealed part (Ex. its own import),
        // where whatever it loads is confined by the same seal.
        return Err(nom::Err::Failure(StofParseError::from(format!("untrusted source can't have #[load] functions (they'd load more code when the host calls load()): fn {name}"))));
    }
    if context.profile.untrusted && func.attributes.contains_key("dropped") {
        // a #[dropped] function runs when its object is dropped: code from somewhere else would run (and could refuse
        // to go) when the host removes it
        return Err(nom::Err::Failure(StofParseError::from(format!("untrusted source can't have #[dropped] functions (they'd run when the host removes it): fn {name}"))));
    }

    // Instert the new function in the current parse context
    //println!("({name}){{{func:?}}}");
    let self_ptr = context.self_ptr();
    let Some(func_ref) = context.graph.insert_stof_data(&self_ptr, name, Box::new(func), None) else {
        let why = context.graph.take_seal_violation().map(|why| format!("sealed: {why}")).unwrap_or_else(|| "the object it belongs to no longer exists".into());
        return Err(nom::Err::Failure(StofParseError::from(format!("could not create fn {name}: {why}"))));
    };

    // Record the definition (editor tooling only)
    if context.info.is_some() {
        if let Some(start) = source::locate(name_pos) {
            let span = match (source::offset(outer_start), source::offset(name_pos), source::offset(fn_start), source::offset(value_end), source::offset(value_end)) {
                (Some(outer_start), Some(name_start), Some(value_start), Some(value_end), Some(outer_end)) => Some(DefSpan { outer_start, name_start, name_end: name_start + name.len(), value_start, value_end, outer_end }),
                _ => None,
            };
            let def = Def { kind: DefKind::Func, name: name.to_string(), parent: self_ptr.clone(), data: Some(func_ref.clone()), obj: None, target: None, start, end: source::locate(value_end), docs: comments.clone(), span };
            if let Some(info) = &mut context.info { info.defs.push(def); }
        }
    }

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


/// A function's body: a block, or one expression whose value is returned (like an arrow function's).
/// `fn add(a: float, b: float) -> float a + b`, or with an optional `return`: `fn hello() -> str return 'hello';`
/// - A body that starts with `{` is always a block, so `{ 'a': 1 }` is never read as a map (return a map from a block).
/// - An expression starts on the signature's line (it can run onto more lines, Ex. a `"""` string). A function with
///   no body is then an error instead of taking what comes next as its body (`fn a() -> int` then `str name: 'x'`
///   would read `str` as the body and `name` as a field).
pub fn function_body(input: &str) -> IResult<&str, Vector<Arc<dyn Instruction>>, StofParseError> {
    let before = input;
    let (input, _) = whitespace(input)?;
    if input.starts_with('{') {
        return block(input);
    }
    let missing = || {
        let mut error = StofParseError::from("expected the function's body: a block { .. }, or an expression on the same line".to_string());
        error.pos = Some(input.as_ptr() as usize);
        nom::Err::Failure(error)
    };
    if before[..before.len() - input.len()].contains('\n') {
        return Err(missing());
    }
    let (input, _) = return_keyword(input)?;
    let Ok((input, body)) = expr(input) else { return Err(missing()); };
    Ok((input, vector![body]))
}


/// An optional `return` before an expression body (`fn hello() -> str return 'hello';`, `(x) => return x * 2`), as a
/// whole word (a body named `returned` is a name).
pub fn return_keyword(input: &str) -> IResult<&str, (), StofParseError> {
    let (input, _) = opt(terminated(tag("return"), not(peek(satisfy(|c: char| c.is_alphanumeric() || c == '_'))))).parse(input)?;
    Ok((input, ()))
}


/// A parameter: `name: type` or optional `name?: type`, either with an optional default.
pub fn any_parameter(input: &str) -> IResult<&str, Param, StofParseError> {
    alt((parameter, opt_parameter)).parse(input)
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
            let mut context = ParseContext::new(&mut graph, Profile::docs(true));
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
