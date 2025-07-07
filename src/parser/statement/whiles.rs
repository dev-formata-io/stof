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
use arcstr::{literal, ArcStr};
use imbl::{vector, Vector};
use nom::{branch::alt, bytes::complete::tag, character::complete::{char, multispace0}, combinator::opt, sequence::{delimited, preceded, terminated}, IResult, Parser};
use crate::{parser::{expr::expr, ident::ident, statement::{block, statement}, whitespace::whitespace}, runtime::{instruction::Instruction, instructions::{whiles::WhileIns, Base}}};


/// Default tagname for continue statements.
pub const CONTINUE: ArcStr = literal!("CLP");

/// Default tagname for break statements.
pub const BREAK: ArcStr = literal!("BLP");


/// While statement.
pub fn while_statement(input: &str) -> IResult<&str, Vector<Arc<dyn Instruction>>> {
    let (input, _) = whitespace(input)?;

    let (input, loop_tag) = opt(terminated(preceded(char('\''), ident), multispace0)).parse(input)?;
    let (input, test_expr) = preceded(terminated(tag("while"), multispace0), delimited(char('('), expr, char(')'))).parse(input)?;
    let (input, ins) = alt((
        block,
        statement
    )).parse(input)?;

    let mut continue_tag = CONTINUE.clone();
    let mut break_tag = BREAK.clone();
    if let Some(custom) = loop_tag {
        continue_tag = format!("{custom}{}", &CONTINUE).into();
        break_tag = format!("{custom}{}", &BREAK).into();
    }

    let while_ins = WhileIns {
        continue_tag,
        break_tag,
        test: test_expr,
        ins,
        declare: None,
        inc: None,
    };
    Ok((input, vector![Arc::new(while_ins) as Arc<dyn Instruction>]))
}


/// Continue statement.
pub fn continue_statement(input: &str) -> IResult<&str, Vector<Arc<dyn Instruction>>> {
    let (input, _) = whitespace(input)?;
    let (input, loop_tag) = preceded(terminated(tag("continue"), multispace0), opt(preceded(char('\''), ident))).parse(input)?;

    let mut continue_tag = CONTINUE.clone();
    if let Some(custom) = loop_tag {
        continue_tag = format!("{custom}{}", &CONTINUE).into();
    }
    Ok((input, vector![Arc::new(Base::CtrlForwardTo(continue_tag)) as Arc<dyn Instruction>]))
}


/// Break statement.
pub fn break_statement(input: &str) -> IResult<&str, Vector<Arc<dyn Instruction>>> {
    let (input, _) = whitespace(input)?;
    let (input, loop_tag) = preceded(terminated(tag("break"), multispace0), opt(preceded(char('\''), ident))).parse(input)?;

    let mut break_tag = BREAK.clone();
    if let Some(custom) = loop_tag {
        break_tag = format!("{custom}{}", &BREAK).into();
    }
    Ok((input, vector![Arc::new(Base::CtrlForwardTo(break_tag)) as Arc<dyn Instruction>]))
}
