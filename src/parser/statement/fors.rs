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
use arcstr::ArcStr;
use imbl::{vector, Vector};
use nom::{bytes::complete::tag, branch::alt, character::complete::{char, multispace0}, combinator::opt, sequence::{delimited, preceded, terminated}, IResult, Parser};
use crate::{parser::{doc::StofParseError, expr::expr, ident::ident, statement::{assign::assign, declare::declare_statement, noscope_block, statement}, whitespace::whitespace}, runtime::{instruction::Instruction, instructions::{block::Block, whiles::WhileIns, Base}, Val}};


/// For loop statement.
pub fn for_loop(input: &str) -> IResult<&str, Vector<Arc<dyn Instruction>>, StofParseError> {
    let (input, _) = whitespace(input)?;

    let (input, loop_tag) = opt(terminated(preceded(char('^'), ident), multispace0)).parse(input)?;
    let (input, control) = preceded(terminated(tag("for"), multispace0), delimited(char('('), declare_test_inc, char(')'))).parse(input)?;
    let (input, ins) = alt((
        noscope_block,
        statement
    )).parse(input)?;

    let mut tag = None;
    if let Some(ltag) = loop_tag {
        tag = Some(ArcStr::from(ltag));
    }

    let mut declare = None;
    if let Some(statement) = control.0 {
        declare = Some(Arc::new(Block { ins: statement }) as Arc<dyn Instruction>);
    }
    let mut test_expr: Arc<dyn Instruction> = Arc::new(Base::Literal(Val::Bool(true)));
    if let Some(expr) = control.1 {
        test_expr = expr;
    }
    let mut inc = None;
    if let Some(statement) = control.2 {
        inc = Some(Arc::new(Block { ins: statement }) as Arc<dyn Instruction>);
    }

    let while_ins = WhileIns {
        tag,
        test: test_expr,
        ins,
        declare,
        inc,
    };
    Ok((input, vector![Arc::new(while_ins) as Arc<dyn Instruction>]))
}


/// Standard for loop inner declare, test, increment expressions.
fn declare_test_inc(input: &str) -> IResult<&str, (Option<Vector<Arc<dyn Instruction>>>, Option<Arc<dyn Instruction>>, Option<Vector<Arc<dyn Instruction>>>), StofParseError> {
    let (input, _) = multispace0(input)?;
    let (input, declaration) = opt(declare_statement).parse(input)?;
    let (input, _) = delimited(multispace0, char(';'), multispace0).parse(input)?;
    let (input, test_expr) = opt(expr).parse(input)?;
    let (input, _) = delimited(multispace0, char(';'), multispace0).parse(input)?;
    let (input, increment) = opt(assign).parse(input)?;
    let (input, _) = multispace0(input)?;
    Ok((input, (declaration, test_expr, increment)))
}
