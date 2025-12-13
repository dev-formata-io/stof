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
use nom::{branch::alt, bytes::complete::tag, character::complete::{char, multispace0}, combinator::opt, multi::many0, sequence::{delimited, preceded, terminated}, IResult, Parser};
use crate::{parser::{doc::StofParseError, expr::expr, statement::{block, statement}, whitespace::whitespace}, runtime::{instruction::Instruction, instructions::ifs::IfIns}};


/// If statement.
pub fn if_statement(input: &str) -> IResult<&str, Vector<Arc<dyn Instruction>>, StofParseError> {
    let (input, _) = whitespace(input)?;
    let (input, if_expr_test) = preceded(terminated(tag("if"), multispace0), delimited(char('('), expr, char(')'))).parse(input)?;

    let (input, if_statements) = alt((
        block,
        statement
    )).parse(input)?;
    
    let (input, mut else_if_statements) = many0(else_if_statement).parse(input)?;
    let (input, else_statement) = opt(else_statement).parse(input)?;

    let mut all_else_statements = Vector::default();
    if else_if_statements.len() > 0 {
        
        let mut current = else_if_statements.pop().unwrap();
        if let Some(else_statements) = else_statement {
            current.el_ins = else_statements;
        }

        while let Some(mut next) = else_if_statements.pop() {
            next.el_ins.push_back(Arc::new(current));
            current = next;
        }

        all_else_statements.push_back(Arc::new(current) as Arc<dyn Instruction>);
    } else if let Some(else_statements) = else_statement {
        all_else_statements.append(else_statements);
    }

    let ins = IfIns {
        if_test: Some(if_expr_test),
        if_ins: if_statements,
        el_ins: all_else_statements,
    };
    Ok((input, vector![Arc::new(ins) as Arc<dyn Instruction>]))
}


/// Else if statement.
pub(self) fn else_if_statement(input: &str) -> IResult<&str, IfIns, StofParseError> {
    let (input, _) = whitespace(input)?;
    let (input, if_expr_test) = preceded(terminated(tag("else if"), multispace0), delimited(char('('), expr, char(')'))).parse(input)?;

    let (input, if_statements) = alt((
        block,
        statement
    )).parse(input)?;


    let ins = IfIns {
        if_test: Some(if_expr_test),
        if_ins: if_statements,
        el_ins: Default::default(),
    };
    Ok((input, ins))
}


/// Else statement.
pub(self) fn else_statement(input: &str) -> IResult<&str, Vector<Arc<dyn Instruction>>, StofParseError> {
    let (input, _) = whitespace(input)?;
    let (input, else_statements) = preceded(
        terminated(
            tag("else"),
            multispace0),
            alt((
                block,
                statement
        ))).parse(input)?;
    Ok((input, else_statements))
}
