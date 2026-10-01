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
use nom::{branch::alt, bytes::complete::{escaped_transform, tag, take_until}, character::complete::{char, none_of}, combinator::{map, opt, value}, multi::fold_many0, sequence::delimited, IResult, Parser};
use crate::{model::libraries::stof_std::StdIns, parser::{doc::StofParseError, expr::expr, whitespace::whitespace}, runtime::{instruction::Instruction, instructions::{block::Block, Base, ADD}, Val}};


/// Formatted string expression.
/// Ex: `This is a cool ${value + other}!`
/// Always a str: each ${expr} is printed like str(expr), then the parts are joined.
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


/// Stand-in for an escaped "\$" while the template is split into literals and ${expr} parts, so "\${" is
/// never read as an expression. Literal parts turn it back into "$".
const ESCAPED_DOLLAR: char = '\u{E000}';
const ESCAPED_DOLLAR_STR: &str = "\u{E000}";

fn literal(text: &str) -> Arc<dyn Instruction> {
    Arc::new(Base::Literal(Val::Str(text.replace(ESCAPED_DOLLAR, "$").into())))
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
        value(ESCAPED_DOLLAR_STR, tag("$")), // "\${" is a literal "${"
    )));
    delimited(char('`'), map(opt(inner), |opt| opt.unwrap_or_default()), char('`')).parse(input)
}


/// Parse inner string into chars and expressions that will be added together.
fn parse_inner(input: &str) -> IResult<&str, Arc<dyn Instruction>, StofParseError> {
    let (input, mut res) = fold_many0(alt((
            parse_inner_expr,
            map(take_until("${"), |lit: &str| literal(lit))
        )),
        Vec::new,
        |mut instructions, ins| {
            instructions.push(ins);
            instructions
        }).parse(input)?;
    
    if !input.is_empty() {
        res.push(literal(input));
    }
    
    if res.is_empty() { return Ok((input, Arc::new(Base::Literal(Val::Str("".into()))))); }
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
/// The value is printed to a str (like str(expr)), so the result never depends on the value types
/// (Ex. `${1}${2}` is "12", not 3; `${list}` is "[1, 2]", not a list).
fn parse_inner_expr(input: &str) -> IResult<&str, Arc<dyn Instruction>, StofParseError> {
    let (input, inner) = delimited(tag("${"), expr, tag("}")).parse(input)?;
    let mut block = Block::default();
    block.ins.push_back(inner);
    block.ins.push_back(Arc::new(StdIns::String(1)));
    Ok((input, Arc::new(block)))
}
