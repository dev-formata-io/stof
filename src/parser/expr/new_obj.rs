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
use nom::{branch::alt, bytes::complete::tag, character::complete::{char, multispace0, multispace1}, combinator::{map, opt}, multi::many0, sequence::{delimited, pair, preceded, terminated}, IResult, Parser};
use crate::{parser::{expr::expr, ident::ident, string::{double_string, single_string}, types::parse_type, whitespace::whitespace}, runtime::{instruction::Instruction, instructions::{block::Block, new_obj::NewObjIns, Base, DUPLICATE, POP_NEW, PUSH_NEW}, Type}};


/// Create a new object expression.
pub fn new_obj_expr(input: &str) -> IResult<&str, Arc<dyn Instruction>> {
    let (input, _) = whitespace(input)?;
    let (input, _) = tag("new").parse(input)?;
    let (input, cast_type) = opt(delimited(multispace0, parse_type, multispace0)).parse(input)?;
    let (input, fields) = delimited(preceded(multispace0, char('{')), obj_fields, preceded(whitespace, char('}'))).parse(input)?;
    let (input, on_expr) = opt(preceded(multispace0, preceded(tag("on"), expr))).parse(input)?;

    let mut block = Block::default();
    let mut parent = false;
    if let Some(on_expr) = on_expr {
        parent = true;
        block.ins.push_back(on_expr);
    }
    block.ins.push_back(Arc::new(NewObjIns {
        parent,
        cast_type,
    }));

    // Now new obj is on the stack
    if !fields.is_empty() {
        block.ins.push_back(DUPLICATE.clone());
        block.ins.push_back(PUSH_NEW.clone());
        for field in fields {
            block.ins.push_back(field.expr);
            
            if let Some(cast_type) = field.cast_type {
                block.ins.push_back(Arc::new(Base::Cast(cast_type)));
            }
            if field.is_const {
                block.ins.push_back(Arc::new(Base::ConstNewObjField(field.name)));
            } else {
                block.ins.push_back(Arc::new(Base::NewObjField(field.name)));
            }
        }
        block.ins.push_back(POP_NEW.clone());
    }

    Ok((input, Arc::new(block) as Arc<dyn Instruction>))
}

/// Field construction object.
struct ObjField {
    is_const: bool,
    cast_type: Option<Type>,
    name: ArcStr,
    expr: Arc<dyn Instruction>,
}

/// Object fields.
fn obj_fields(input: &str) -> IResult<&str, Vec<ObjField>> {
    let (input, _) = whitespace(input)?;
    let (input, fields) = many0(obj_field).parse(input)?;
    Ok((input, fields))
}

/// New object field.
fn obj_field(input: &str) -> IResult<&str, ObjField> {
    let (input, _) = whitespace(input)?;

    // Optionally a const field
    let (input, is_const) = opt(terminated(tag("const"), multispace0)).parse(input)?;

    // Type (optional) and name
    let (input, (cast_type, name)) = alt((
        map(pair(terminated(parse_type, multispace1), alt((
            map(ident, |v| v.to_string()),
            double_string,
            single_string
        ))), |(ty, nm)| (Some(ty), nm)),
        map(alt((
            map(ident, |v| v.to_string()),
            double_string,
            single_string
        )), |nm| (None, nm))
    )).parse(input)?;

    let (input, opt_expr) = opt(preceded(delimited(multispace0, char(':'), multispace0), expr)).parse(input)?;
    let (input, _) = opt(preceded(multispace0, alt((char(','), char(';'))))).parse(input)?;

    let name: ArcStr = name.into();
    let mut expr = Arc::new(Base::LoadVariable(name.clone(), false, false)) as Arc<dyn Instruction>;
    if let Some(oexpr) = opt_expr {
        expr = oexpr;
    }
    Ok((input, ObjField {
        is_const: is_const.is_some(),
        cast_type,
        name,
        expr
    }))
}
