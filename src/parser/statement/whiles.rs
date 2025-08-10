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
use nom::{branch::alt, bytes::complete::tag, character::complete::{char, multispace0}, combinator::opt, sequence::{delimited, preceded, terminated}, IResult, Parser};
use crate::{parser::{doc::StofParseError, expr::expr, ident::ident, statement::{noscope_block, statement}, whitespace::whitespace}, runtime::{instruction::Instruction, instructions::{whiles::WhileIns, Base, BREAK_LOOP, CONTINUE_LOOP}, Val}};


/// While statement.
pub fn while_statement(input: &str) -> IResult<&str, Vector<Arc<dyn Instruction>>, StofParseError> {
    let (input, _) = whitespace(input)?;

    let (input, loop_tag) = opt(terminated(preceded(char('^'), ident), multispace0)).parse(input)?;
    let (input, test_expr) = preceded(terminated(tag("while"), multispace0), delimited(char('('), expr, char(')'))).parse(input)?;
    let (input, ins) = alt((
        noscope_block,
        statement
    )).parse(input)?;

    let mut tag = None;
    if let Some(ltag) = loop_tag {
        tag = Some(ArcStr::from(ltag));
    }

    let while_ins = WhileIns {
        tag,
        test: test_expr,
        ins,
        declare: None,
        inc: None,
    };
    Ok((input, vector![Arc::new(while_ins) as Arc<dyn Instruction>]))
}


/// Loop statement.
pub fn loop_statement(input: &str) -> IResult<&str, Vector<Arc<dyn Instruction>>, StofParseError> {
    let (input, _) = whitespace(input)?;

    let (input, loop_tag) = opt(terminated(preceded(char('^'), ident), multispace0)).parse(input)?;
    let (input, _) = terminated(tag("loop"), multispace0).parse(input)?;
    let (input, ins) = alt((
        noscope_block,
        statement
    )).parse(input)?;

    let mut tag = None;
    if let Some(ltag) = loop_tag {
        tag = Some(ArcStr::from(ltag));
    }

    let while_ins = WhileIns {
        tag,
        test: Arc::new(Base::Literal(Val::Bool(true))), // while (true)
        ins,
        declare: None,
        inc: None,
    };
    Ok((input, vector![Arc::new(while_ins) as Arc<dyn Instruction>]))
}


/// Continue statement.
pub fn continue_statement(input: &str) -> IResult<&str, Vector<Arc<dyn Instruction>>, StofParseError> {
    let (input, _) = whitespace(input)?;
    let (input, loop_tag) = preceded(terminated(tag("continue"), multispace0), opt(preceded(char('^'), ident))).parse(input)?;

    let instruction: Arc<dyn Instruction>;
    if let Some(custom) = loop_tag {
        let continue_tag: ArcStr = format!("{}_con", custom).into();
        instruction = Arc::new(Base::CtrlForwardTo(continue_tag));
    } else {
        instruction = CONTINUE_LOOP.clone();
    }
    Ok((input, vector![instruction]))
}


/// Break statement.
pub fn break_statement(input: &str) -> IResult<&str, Vector<Arc<dyn Instruction>>, StofParseError> {
    let (input, _) = whitespace(input)?;
    let (input, loop_tag) = preceded(terminated(tag("break"), multispace0), opt(preceded(char('^'), ident))).parse(input)?;

    let instruction: Arc<dyn Instruction>;
    if let Some(custom) = loop_tag {
        let break_tag: ArcStr = format!("{}_brk", custom).into();
        instruction = Arc::new(Base::CtrlForwardTo(break_tag));
    } else {
        instruction = BREAK_LOOP.clone();
    }
    Ok((input, vector![instruction]))
}
