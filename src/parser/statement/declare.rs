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
use imbl::Vector;
use nom::{branch::alt, bytes::complete::tag, character::complete::{char, multispace0}, combinator::opt, sequence::{delimited, preceded}, IResult, Parser};
use crate::{parser::{expr::expr, ident::ident, types::parse_type, whitespace::whitespace}, runtime::{instruction::Instruction, instructions::Base}};


/// Declare a variable.
pub fn declare_statement(input: &str) -> IResult<&str, Vector<Arc<dyn Instruction>>> {
    let (input, _) = whitespace(input)?;
    alt((declare_const_var, declare_mut_var)).parse(input)
}


/// Mutable variable declaration.
/// let var: int = 45
pub(self) fn declare_mut_var(input: &str) -> IResult<&str, Vector<Arc<dyn Instruction>>> {
    let (input, varname) = delimited(tag("let"), preceded(multispace0, ident), multispace0).parse(input)?;
    let (input, cast_type) = opt(preceded(char(':'), parse_type)).parse(input)?; 
    let (input, _) = delimited(multispace0, char('='), multispace0).parse(input)?;
    let (input, expr) = expr(input)?;

    let mut block = Vector::default();
    block.push_back(expr);
    let typed = cast_type.is_some();
    if let Some(cast_type) = cast_type {
        block.push_back(Arc::new(Base::Cast(cast_type)));
    }
    block.push_back(Arc::new(Base::DeclareVar(varname.to_string().into(), typed)));
    Ok((input, block))
}


/// Const variable declaration.
/// const var: int = 45
pub(self) fn declare_const_var(input: &str) -> IResult<&str, Vector<Arc<dyn Instruction>>> {
    let (input, varname) = delimited(tag("const"), preceded(multispace0, ident), multispace0).parse(input)?;
    let (input, cast_type) = opt(preceded(char(':'), parse_type)).parse(input)?; 
    let (input, _) = delimited(multispace0, char('='), multispace0).parse(input)?;
    let (input, expr) = expr(input)?;

    let mut block = Vector::default();
    block.push_back(expr);
    let typed = cast_type.is_some();
    if let Some(cast_type) = cast_type {
        block.push_back(Arc::new(Base::Cast(cast_type)));
    }
    block.push_back(Arc::new(Base::DeclareConstVar(varname.to_string().into(), typed)));
    Ok((input, block))
}
