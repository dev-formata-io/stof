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
use nom::{branch::alt, bytes::complete::tag, character::complete::{char, multispace0}, combinator::opt, sequence::{delimited, preceded}, IResult, Parser};
use crate::{model::{Param, SId}, parser::{doc::StofParseError, ident::ident, statement::{block, statement}, types::parse_type, whitespace::whitespace}, runtime::{instruction::Instruction, instructions::{trycatch::TryCatchIns, Base, POP_STACK}, Type}};


/// Try catch statement.
pub fn try_catch_statement(input: &str) -> IResult<&str, Vector<Arc<dyn Instruction>>, StofParseError> {
    let (input, _) = whitespace(input)?;

    // Try instructions
    let (input, _) = tag("try").parse(input)?;
    let (input, try_ins) = alt((
        block,
        statement
    )).parse(input)?;

    // Catch instructions
    let (input, _) = delimited(multispace0, tag("catch"), multispace0).parse(input)?;
    let (input, error_param) = opt(delimited(char('('), error_parameter, char(')'))).parse(input)?;
    let (input, catch_ins) = alt((
        block,
        statement
    )).parse(input)?;

    let mut err_ins = vector![];
    if let Some(error_param) = error_param {
        // declare const variable that is the error (type irrelivant)
        err_ins.push_back(Arc::new(Base::Cast(error_param.param_type.clone())) as Arc<dyn Instruction>);
        err_ins.push_back(Arc::new(Base::DeclareConstVar(ArcStr::from(error_param.name.as_ref()), error_param.param_type)));
    } else {
        // pop error from stack
        err_ins.push_back(POP_STACK.clone() as Arc<dyn Instruction>);
    }

    Ok((input, vector![Arc::new(TryCatchIns { try_ins, err_ins, catch_ins }) as Arc<dyn Instruction>]))
}


/// Parse catch parameter.
fn error_parameter(input: &str) -> IResult<&str, Param, StofParseError> {
    let (input, _) = multispace0(input)?;
    let (input, name) = ident(input)?;
    let (input, param_type) = opt(preceded(preceded(multispace0, char(':')), preceded(multispace0, parse_type))).parse(input)?;
    let (input, _) = multispace0(input)?;

    let mut ptype = Type::Unknown;
    if let Some(pty) = param_type {
        ptype = pty;
    }

    let param = Param {
        name: SId::from(name),
        param_type: ptype,
        default: None
    };
    Ok((input, param))
}
