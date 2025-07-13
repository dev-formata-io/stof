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
use nom::{branch::alt, character::complete::{char, multispace0}, combinator::{opt, recognize}, multi::{many0, separated_list0, separated_list1}, sequence::{delimited, preceded}, IResult, Parser};
use crate::{model::SId, parser::{expr::expr, ident::ident, whitespace::whitespace}, runtime::{instruction::Instruction, instructions::{block::Block, call::{FuncCall, NamedArg}, Base}}};


/// Graph interaction expression.
/// This is a variable lookup (symbol table or graph).
/// Or a function call.
/// Or an index operator (also a function call).
/// And is a chain of them all!
pub fn graph_expr(input: &str) -> IResult<&str, Arc<dyn Instruction>> {
    let (input, _) = whitespace(input)?;

    // Is this a reference?
    let (input, as_ref) = opt(char('&')).parse(input)?;

    // Get a variable or function call onto the stack, then optionally chain on more!
    let (input, mut first) = var_func(input, false)?;
    if as_ref.is_some() {
        if let Some(base) = first.as_dyn_any().downcast_ref::<Base>() {
            match base {
                Base::LoadVariable(path, chained, _) => {
                    first = Arc::new(Base::LoadVariable(path.clone(), *chained, true));
                },
                _ => {}
            }
        }
    }

    // Get the rest if any more!
    let (input, additional) = many0(alt((
        preceded(char('.'), chained_var_func),
        chained_var_func
    ))).parse(input)?;

    // If only one, then don't create a block...
    if additional.is_empty() {
        return Ok((input, first));
    }

    // Put em all together
    let mut block = Block::default();
    block.ins.push_back(first);
    for ins in additional { block.ins.push_back(ins); }
    Ok((input, Arc::new(block)))
}
pub fn chained_var_func(input: &str) -> IResult<&str, Arc<dyn Instruction>> {
    // TODO add null check operator instruction "?."... will be an additional [dup, if [nullcheck, jump], ..var_func.., jumptag] sequence
    var_func(input, true)
}
pub(self) fn var_func(input: &str, chained: bool) -> IResult<&str, Arc<dyn Instruction>> {
    // Special case of [idx][idx][idx] chaining
    if chained && input.starts_with('[') {
        let (input, idx) = index_expr(input)?;
        return Ok((input, Arc::new(FuncCall {
            stack: chained,
            func: None,
            search: Some("at".into()),
            args: idx.into_iter().collect(),
        })));
    }

    // Variable portion is not optional
    let (input, path) = variable_expr(input)?;
    let mut path = path.to_string();

    // Optional call arguments portion is next
    let (mut input, mut call) = opt(call_expr).parse(input)?;

    // Optional index expr if call expr fails (Ex. "self.hello[5]" -> "self.hello.at(5)")
    if call.is_none() {
        let (inner, idx) = opt(index_expr).parse(input)?;
        if idx.is_some() {
            path.push_str(".at");
            input = inner;
            call = idx;
        }
    }

    // Return a call if there is a call, otherwise return a variable lookup.
    if let Some(args) = call {
        Ok((input, Arc::new(FuncCall {
            stack: chained,
            func: None,
            search: Some(path.into()),
            args: args.into_iter().collect(),
        })))
    } else {
        Ok((input, Arc::new(Base::LoadVariable(path.into(), chained, false))))
    }
}


/// Variable expression.
/// This is the optional first part of the graph interaction, and is a path into the graph or symbol table.
///
/// Ex. "a.my_func()" -> "a.my_func" would be the variable expr.
/// Ex. "myFunc()" -> "myFunc" would be the variable expr.
/// Ex. "self.child.func()" -> "self.child.func" would be the variable expr.
pub(self) fn variable_expr(input: &str) -> IResult<&str, &str> {
    recognize(separated_list1(char('.'), ident)).parse(input)
}


/// Call expression.
/// This is what comes after the variable expression.
/// If this exists, the last section of the variable expr was actually a function name.
pub(self) fn call_expr(input: &str) -> IResult<&str, Vec<Arc<dyn Instruction>>> {
    delimited(
        char('('),
        separated_list0(char(','), call_arg),
        char(')')
    ).parse(input)
}
pub(self) fn call_arg(input: &str) -> IResult<&str, Arc<dyn Instruction>> {
    let (input, _) = multispace0(input)?;

    let (input, ins) = alt((
        named_arg,
        expr
    )).parse(input)?;

    let (input, _) = multispace0(input)?;
    Ok((input, ins))
}
pub(self) fn named_arg(input: &str) -> IResult<&str, Arc<dyn Instruction>> {
    let (input, name) = ident(input)?;
    
    let (input, _) = multispace0(input)?;
    let (input, _) = char('=').parse(input)?;
    let (input, _) = multispace0(input)?;

    let (input, ins) = expr(input)?;
    Ok((input, Arc::new(NamedArg { name: SId::from(name), ins })))
}


/// Index expression.
/// Gets transformed into an "at(args)" call.
pub(self) fn index_expr(input: &str) -> IResult<&str, Vec<Arc<dyn Instruction>>> {
    delimited(
        char('['),
        separated_list1(char(','), call_arg),
        char(']')
    ).parse(input)
}
