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
use nom::{branch::alt, bytes::complete::{escaped_transform, tag, take_until}, character::complete::{char, none_of}, combinator::{map, opt, value}, multi::fold_many0, sequence::delimited, IResult, Parser};
use crate::{parser::{doc::StofParseError, expr::expr, whitespace::whitespace}, runtime::{instruction::Instruction, instructions::{block::Block, Base, ADD, NOOP}, Val}};


/// Formatted string expression.
/// Ex: `This is a cool ${value + other}!`
pub fn formatted_string_expr(input: &str) -> IResult<&str, Arc<dyn Instruction>, StofParseError> {
    let (input, _) = whitespace(input)?;
    let (input, inner) = inner_formatted(input)?;

    match parse_inner(&inner) {
        Ok((_, expr)) => {
            Ok((input, expr))
        },
        Err(error) => {
            Err(error)
        }
    }
}


/// Inner formatted string (to run additional parser on after)
fn inner_formatted(input: &str) -> IResult<&str, String, StofParseError> {
    let normal = none_of("`\\"); // everything but backslash or double quote
    let inner = escaped_transform(normal, '\\', alt((
        value("\\", tag("\\")),
        value("`", tag("`")),
        value("\n", tag("n")),
        value("\r", tag("r")),
        value("\t", tag("t")),
    )));
    delimited(char('`'), map(opt(inner), |opt| opt.unwrap_or_default()), char('`')).parse(input)
}


/// Parse inner string into chars and expressions that will be added together.
fn parse_inner(input: &str) -> IResult<&str, Arc<dyn Instruction>, StofParseError> {
    let (input, mut res) = fold_many0(alt((
            parse_inner_expr,
            map(take_until("${"), |lit: &str| Arc::new(Base::Literal(Val::Str(lit.into()))) as Arc<dyn Instruction>)
        )),
        Vec::new,
        |mut instructions, ins| {
            instructions.push(ins);
            instructions
        }).parse(input)?;
    
    if !input.is_empty() {
        res.push(Arc::new(Base::Literal(Val::Str(input.into()))));
    }
    
    if res.is_empty() { return Ok((input, NOOP.clone())); }
    else if res.len() == 1 { return Ok((input, res.pop().unwrap())); }
    else {
        let mut block = Block::default();
        
        block.ins.push_back(res.pop().unwrap()); // rhs
        while !res.is_empty() {
            block.ins.push_back(res.pop().unwrap()); // lhs
            block.ins.push_back(ADD.clone());
        }

        Ok((input, Arc::new(block)))
    }
}


/// Parse inner expr.
fn parse_inner_expr(input: &str) -> IResult<&str, Arc<dyn Instruction>, StofParseError> {
    let (input, inner) = delimited(tag("${"), expr, tag("}")).parse(input)?;
    Ok((input, inner))
}
