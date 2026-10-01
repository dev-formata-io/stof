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
use arcstr::ArcStr;
use imbl::Vector;
use nom::{bytes::complete::tag, branch::alt, character::complete::{char, multispace0}, combinator::{not, recognize}, multi::separated_list1, sequence::{delimited, terminated}, IResult, Parser};
use crate::{parser::{doc::StofParseError, expr::{expr, graph::graph_expr}, ident::ident, whitespace::whitespace}, runtime::{instruction::Instruction, instructions::{assign::SetFieldIns, block::Block, Base, ADD, BIT_AND, BIT_OR, BIT_SHIFT_LEFT, BIT_SHIFT_RIGHT, BIT_XOR, DIVIDE, MODULUS, MULTIPLY, SUBTRACT}}};


/// Assign statement.
pub fn assign(input: &str) -> IResult<&str, Vector<Arc<dyn Instruction>>, StofParseError> {
    let (input, _) = whitespace(input)?;
    alt((
        assign_variable,
        add_assign_variable,
        sub_assign_variable,
        mul_assign_variable,
        div_assign_variable,
        mod_assign_variable,
        band_assign_variable,
        bor_assign_variable,
        bxor_assign_variable,
        bshl_assign_variable,
        bshr_assign_variable,
        assign_computed_target,
    )).parse(input)
}


/// Assign to a field of a computed object (Ex. `self.customer(id).type = x`, `list[0].name = 'a'`).
/// The left side is a normal expression ending in a field (after a call or index); the object is evaluated
/// first, then the value, then the field is set on that object (created if needed).
pub(self) fn assign_computed_target(input: &str) -> IResult<&str, Vector<Arc<dyn Instruction>>, StofParseError> {
    let (rest, target) = delimited(multispace0, graph_expr, multispace0).parse(input)?;
    let not_assignable = || nom::Err::Error(StofParseError::from("not an assignable target"));

    // Ex. [call self.customer(id), load chained "type"]: the object steps, then the field path
    let Some(block) = target.as_dyn_any().downcast_ref::<Block>() else { return Err(not_assignable()) };
    if block.ins.len() < 2 { return Err(not_assignable()); }
    let Some(Base::LoadVariable(path, true, false)) = block.ins.last().and_then(|ins| ins.as_dyn_any().downcast_ref::<Base>()) else {
        return Err(not_assignable());
    };
    let path = path.clone();

    let (rest, _) = terminated(terminated(char('='), not(char('='))), multispace0).parse(rest)?;
    let (rest, value) = expr(rest)?;

    let mut object = Block::default();
    object.ins = block.ins.clone();
    object.ins.pop_back();

    let mut instructions = Vector::default();
    instructions.push_back(Arc::new(object) as Arc<dyn Instruction>);
    instructions.push_back(value);
    instructions.push_back(Arc::new(SetFieldIns { path }));
    Ok((rest, instructions))
}


/// Assign a variable statement.
pub(self) fn assign_variable(input: &str) -> IResult<&str, Vector<Arc<dyn Instruction>>, StofParseError> {
    let (input, varname) = delimited(multispace0, recognize(separated_list1(char('.'), ident)), multispace0).parse(input)?;
    let (input, _) = terminated(char('='), multispace0).parse(input)?;
    let (input, expr) = expr(input)?;

    let mut block = Vector::default();
    block.push_back(expr);
    block.push_back(Arc::new(Base::SetVariable(varname.to_string().into())));
    Ok((input, block))
}


/// Add assign a variable statement. "+="
pub(self) fn add_assign_variable(input: &str) -> IResult<&str, Vector<Arc<dyn Instruction>>, StofParseError> {
    let (input, varname) = delimited(multispace0, recognize(separated_list1(char('.'), ident)), multispace0).parse(input)?;
    let (input, _) = terminated(tag("+="), multispace0).parse(input)?;
    let (input, expr) = expr(input)?;

    let mut block = Vector::default();
    let varname: ArcStr = varname.to_string().into();

    // push rhs, then lhs (pop off stack in reverse..), op, then set
    block.push_back(expr);
    block.push_back(Arc::new(Base::LoadVariable(varname.clone(), false, false)) as Arc<dyn Instruction>);
    block.push_back(ADD.clone());
    block.push_back(Arc::new(Base::SetVariable(varname)));
    Ok((input, block))
}


/// Sub assign a variable statement. "-="
pub(self) fn sub_assign_variable(input: &str) -> IResult<&str, Vector<Arc<dyn Instruction>>, StofParseError> {
    let (input, varname) = delimited(multispace0, recognize(separated_list1(char('.'), ident)), multispace0).parse(input)?;
    let (input, _) = terminated(tag("-="), multispace0).parse(input)?;
    let (input, expr) = expr(input)?;

    let mut block = Vector::default();
    let varname: ArcStr = varname.to_string().into();

    // push rhs, then lhs (pop off stack in reverse..), op, then set
    block.push_back(expr);
    block.push_back(Arc::new(Base::LoadVariable(varname.clone(), false, false)) as Arc<dyn Instruction>);
    block.push_back(SUBTRACT.clone());
    block.push_back(Arc::new(Base::SetVariable(varname)));
    Ok((input, block))
}


/// Multiply assign a variable statement. "*="
pub(self) fn mul_assign_variable(input: &str) -> IResult<&str, Vector<Arc<dyn Instruction>>, StofParseError> {
    let (input, varname) = delimited(multispace0, recognize(separated_list1(char('.'), ident)), multispace0).parse(input)?;
    let (input, _) = terminated(tag("*="), multispace0).parse(input)?;
    let (input, expr) = expr(input)?;

    let mut block = Vector::default();
    let varname: ArcStr = varname.to_string().into();

    // push rhs, then lhs (pop off stack in reverse..), op, then set
    block.push_back(expr);
    block.push_back(Arc::new(Base::LoadVariable(varname.clone(), false, false)) as Arc<dyn Instruction>);
    block.push_back(MULTIPLY.clone());
    block.push_back(Arc::new(Base::SetVariable(varname)));
    Ok((input, block))
}


/// Divide assign a variable statement. "/="
pub(self) fn div_assign_variable(input: &str) -> IResult<&str, Vector<Arc<dyn Instruction>>, StofParseError> {
    let (input, varname) = delimited(multispace0, recognize(separated_list1(char('.'), ident)), multispace0).parse(input)?;
    let (input, _) = terminated(tag("/="), multispace0).parse(input)?;
    let (input, expr) = expr(input)?;

    let mut block = Vector::default();
    let varname: ArcStr = varname.to_string().into();

    // push rhs, then lhs (pop off stack in reverse..), op, then set
    block.push_back(expr);
    block.push_back(Arc::new(Base::LoadVariable(varname.clone(), false, false)) as Arc<dyn Instruction>);
    block.push_back(DIVIDE.clone());
    block.push_back(Arc::new(Base::SetVariable(varname)));
    Ok((input, block))
}


/// Mod assign a variable statement. "%="
pub(self) fn mod_assign_variable(input: &str) -> IResult<&str, Vector<Arc<dyn Instruction>>, StofParseError> {
    let (input, varname) = delimited(multispace0, recognize(separated_list1(char('.'), ident)), multispace0).parse(input)?;
    let (input, _) = terminated(tag("%="), multispace0).parse(input)?;
    let (input, expr) = expr(input)?;

    let mut block = Vector::default();
    let varname: ArcStr = varname.to_string().into();

    // push rhs, then lhs (pop off stack in reverse..), op, then set
    block.push_back(expr);
    block.push_back(Arc::new(Base::LoadVariable(varname.clone(), false, false)) as Arc<dyn Instruction>);
    block.push_back(MODULUS.clone());
    block.push_back(Arc::new(Base::SetVariable(varname)));
    Ok((input, block))
}


/// Bit and assign a variable statement. "&="
pub(self) fn band_assign_variable(input: &str) -> IResult<&str, Vector<Arc<dyn Instruction>>, StofParseError> {
    let (input, varname) = delimited(multispace0, recognize(separated_list1(char('.'), ident)), multispace0).parse(input)?;
    let (input, _) = terminated(tag("&="), multispace0).parse(input)?;
    let (input, expr) = expr(input)?;

    let mut block = Vector::default();
    let varname: ArcStr = varname.to_string().into();

    // push rhs, then lhs (pop off stack in reverse..), op, then set
    block.push_back(expr);
    block.push_back(Arc::new(Base::LoadVariable(varname.clone(), false, false)) as Arc<dyn Instruction>);
    block.push_back(BIT_AND.clone());
    block.push_back(Arc::new(Base::SetVariable(varname)));
    Ok((input, block))
}


/// Bit or assign a variable statement. "|="
pub(self) fn bor_assign_variable(input: &str) -> IResult<&str, Vector<Arc<dyn Instruction>>, StofParseError> {
    let (input, varname) = delimited(multispace0, recognize(separated_list1(char('.'), ident)), multispace0).parse(input)?;
    let (input, _) = terminated(tag("|="), multispace0).parse(input)?;
    let (input, expr) = expr(input)?;

    let mut block = Vector::default();
    let varname: ArcStr = varname.to_string().into();

    // push rhs, then lhs (pop off stack in reverse..), op, then set
    block.push_back(expr);
    block.push_back(Arc::new(Base::LoadVariable(varname.clone(), false, false)) as Arc<dyn Instruction>);
    block.push_back(BIT_OR.clone());
    block.push_back(Arc::new(Base::SetVariable(varname)));
    Ok((input, block))
}


/// Bit xor assign a variable statement. "^="
pub(self) fn bxor_assign_variable(input: &str) -> IResult<&str, Vector<Arc<dyn Instruction>>, StofParseError> {
    let (input, varname) = delimited(multispace0, recognize(separated_list1(char('.'), ident)), multispace0).parse(input)?;
    let (input, _) = terminated(tag("^="), multispace0).parse(input)?;
    let (input, expr) = expr(input)?;

    let mut block = Vector::default();
    let varname: ArcStr = varname.to_string().into();

    // push rhs, then lhs (pop off stack in reverse..), op, then set
    block.push_back(expr);
    block.push_back(Arc::new(Base::LoadVariable(varname.clone(), false, false)) as Arc<dyn Instruction>);
    block.push_back(BIT_XOR.clone());
    block.push_back(Arc::new(Base::SetVariable(varname)));
    Ok((input, block))
}


/// Bit shift left assign a variable statement. "<<="
pub(self) fn bshl_assign_variable(input: &str) -> IResult<&str, Vector<Arc<dyn Instruction>>, StofParseError> {
    let (input, varname) = delimited(multispace0, recognize(separated_list1(char('.'), ident)), multispace0).parse(input)?;
    let (input, _) = terminated(tag("<<="), multispace0).parse(input)?;
    let (input, expr) = expr(input)?;

    let mut block = Vector::default();
    let varname: ArcStr = varname.to_string().into();

    // push rhs, then lhs (pop off stack in reverse..), op, then set
    block.push_back(expr);
    block.push_back(Arc::new(Base::LoadVariable(varname.clone(), false, false)) as Arc<dyn Instruction>);
    block.push_back(BIT_SHIFT_LEFT.clone());
    block.push_back(Arc::new(Base::SetVariable(varname)));
    Ok((input, block))
}


/// Bit shift right assign a variable statement. ">>="
pub(self) fn bshr_assign_variable(input: &str) -> IResult<&str, Vector<Arc<dyn Instruction>>, StofParseError> {
    let (input, varname) = delimited(multispace0, recognize(separated_list1(char('.'), ident)), multispace0).parse(input)?;
    let (input, _) = terminated(tag(">>="), multispace0).parse(input)?;
    let (input, expr) = expr(input)?;

    let mut block = Vector::default();
    let varname: ArcStr = varname.to_string().into();

    // push rhs, then lhs (pop off stack in reverse..), op, then set
    block.push_back(expr);
    block.push_back(Arc::new(Base::LoadVariable(varname.clone(), false, false)) as Arc<dyn Instruction>);
    block.push_back(BIT_SHIFT_RIGHT.clone());
    block.push_back(Arc::new(Base::SetVariable(varname)));
    Ok((input, block))
}
