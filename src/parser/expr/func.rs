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

use std::sync::Arc;
use imbl::vector;
use nom::{branch::alt, bytes::complete::tag, character::complete::{char, multispace0}, combinator::{map, opt}, multi::separated_list0, sequence::{delimited, preceded, terminated}, IResult, Parser};
use rustc_hash::FxHashMap;
use crate::{model::{DataRef, Func, ASYNC_FUNC_ATTR}, parser::{doc::StofParseError, expr::expr, func::{opt_parameter, parameter}, statement::block, types::parse_type, whitespace::whitespace}, runtime::{instruction::{Instruction, Instructions}, instructions::func::FuncLit, Type, Val}};


/// Arrow function "literal" value.
pub fn func_expr(input: &str) -> IResult<&str, Arc<dyn Instruction>, StofParseError> {
    let (input, _) = whitespace(input)?;
    let (input, async_fn) = opt(terminated(tag("async"), multispace0)).parse(input)?;
    let (input, params) = delimited(char('('), separated_list0(char(','), alt((parameter, opt_parameter))), char(')')).parse(input)?;
    let (input, return_type) = opt(preceded(delimited(multispace0, alt((tag(":"), tag("->"))), multispace0), parse_type)).parse(input)?;
    let (input, _) = delimited(multispace0, tag("=>"), multispace0).parse(input)?;
    let (input, instructions) = alt((
        block,
        map(expr, |ins| vector![ins])
    )).parse(input)?;

    let mut rtype = Type::Void;
    if let Some(ty) = return_type {
        rtype = ty;
    }
    
    let mut attrs = FxHashMap::default();
    if async_fn.is_some() {
        attrs.insert(ASYNC_FUNC_ATTR.to_string(), Val::Null); // this is an async arrow function
    }

    let dref = DataRef::default();
    let arrow_func = Func::new(params.into_iter().collect(), rtype, Instructions::from(instructions), Some(attrs));
    Ok((input, Arc::new(FuncLit { dref, func: arrow_func }) as Arc<dyn Instruction>))
}
