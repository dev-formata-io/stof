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

use nom::{bytes::complete::tag, character::complete::{char, multispace0}, combinator::opt, multi::separated_list0, sequence::{delimited, preceded, terminated}, IResult, Parser};
use crate::{model::{Func, Param, SId, ASYNC_FUNC_ATTR}, parser::{context::ParseContext, expr::expr, ident::ident, parse_attributes, statement::block, types::parse_type, whitespace::whitespace}, runtime::Val};


/// Parse a function into a parse context.
pub fn parse_function<'a>(input: &'a str, context: &mut ParseContext) -> IResult<&'a str, ()> {
    let mut func = Func::default();
    let (input, attrs) = parse_attributes(input, context)?;
    func.attributes = attrs;

    // TODO doc comments before whitespace
    let (input, _) = whitespace(input)?;

    let (input, attrs) = parse_attributes(input, context)?;
    for (k, v) in attrs { func.attributes.insert(k, v); }

    let (input, async_fn) = opt(terminated(tag("async"), multispace0)).parse(input)?;
    if async_fn.is_some() && !func.attributes.contains_key(&ASYNC_FUNC_ATTR) {
        func.attributes.insert(ASYNC_FUNC_ATTR.clone(), Val::Null);
    }

    let (input, name) = preceded(tag("fn"), preceded(multispace0, ident)).parse(input)?;
    let (input, params) = delimited(char('('), separated_list0(char(','), parameter), char(')')).parse(input)?;
    let (input, return_type) = opt(preceded(delimited(multispace0, tag("->"), multispace0), parse_type)).parse(input)?;
    let (input, instructions) = block(input)?;

    for param in params { func.params.push_back(param); }
    func.return_type = return_type.unwrap_or_default(); // default is void
    func.instructions = instructions;

    // Instert the new function in the current parse context
    //println!("({name}){{{func:?}}}");
    let self_ptr = context.self_ptr();
    context.graph.insert_stof_data(&self_ptr, name, Box::new(func), None);

    Ok((input, ()))
}


/// Parse a function parameter.
pub fn parameter(input: &str) -> IResult<&str, Param> {
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


#[cfg(test)]
mod tests {
    use crate::{model::Graph, parser::{context::ParseContext, func::parse_function}, runtime::{Runtime, Val}};

    #[test]
    fn basic_func() {
        let mut graph = Graph::default();
        let mut context = ParseContext::new(&mut graph);

        let (_input, ()) = parse_function(r#"
 
        #[test('hello')]
        /**
         * This is a test function.
         */
        fn main(x: float = 5) -> float { x }

        "#, &mut context).unwrap();

        let res = Runtime::call(&mut graph, "root.main", vec![Val::from(10)]).unwrap();
        assert_eq!(res, 10.into());
    }
}
