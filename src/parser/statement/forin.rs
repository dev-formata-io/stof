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
use arcstr::{literal, ArcStr};
use imbl::{vector, Vector};
use nom::{branch::alt, bytes::complete::tag, character::complete::{char, multispace0}, combinator::opt, sequence::{delimited, preceded, terminated}, IResult, Parser};
use crate::{model::stof_std::COPY, parser::{doc::StofParseError, expr::expr, ident::ident, statement::{noscope_block, statement}, types::parse_type, whitespace::whitespace}, runtime::{instruction::{Instruction, Instructions}, instructions::{block::Block, call::FuncCall, nullcheck::NullcheckIns, ops::{Op, OpIns}, whiles::WhileIns, Base}, Num, NumT, Type, Val}};


/// For in loop.
/// for (const x: int in thing) {}
pub fn for_in_loop(input: &str) -> IResult<&str, Vector<Arc<dyn Instruction>>, StofParseError> {
    let (input, _) = whitespace(input)?;

    let (input, loop_tag) = opt(terminated(preceded(char('^'), ident), multispace0)).parse(input)?;
    let (input, inner) = preceded(terminated(tag("for"), multispace0), delimited(char('('), inner_loop, char(')'))).parse(input)?;
    let (input, ins) = alt((
        noscope_block,
        statement
    )).parse(input)?;

    // Declare iterable (has "at" and a "len" functions)
    // Declare length (stopping condition)
    // Declare index variable
    // Declare first & last variables
    let mut declare_instructions = Instructions::default();
    let length_var = literal!("length");
    let index_var = literal!("index");
    let first_var = literal!("first");
    let last_var = literal!("last");
    {
        declare_instructions.push(inner.expr);
        declare_instructions.push(Arc::new(Base::DeclareConstVar(literal!("iterable"), Type::Void)));
        
        declare_instructions.push(Arc::new(NullcheckIns {
            ins: Arc::new(FuncCall { func: None, search: Some(literal!("iterable.len")), stack: false, args: vector![], as_ref: false, cnull: true, oself: None, }),
            ifnull: Arc::new(Base::Literal(Val::Num(Num::Int(0))))
        }));
        declare_instructions.push(COPY.clone()); // make sure the length is not a reference
        declare_instructions.push(Arc::new(Base::DeclareConstVar(length_var.clone(), Type::Void)));

        declare_instructions.push(Arc::new(Base::Literal(Val::Num(Num::Int(0)))));
        declare_instructions.push(Arc::new(Base::DeclareVar(index_var.clone(), Type::Num(NumT::Int))));

        declare_instructions.push(Arc::new(Base::Literal(Val::Bool(true))));
        declare_instructions.push(Arc::new(Base::DeclareVar(first_var.clone(), Type::Bool)));
        declare_instructions.push(Arc::new(Base::Literal(Val::Bool(false))));
        declare_instructions.push(Arc::new(Base::DeclareVar(last_var.clone(), Type::Bool)));
    }

    // Test instruction
    // index < length
    let test: Arc<dyn Instruction> = Arc::new(OpIns {
        lhs: Arc::new(Base::LoadVariable(index_var.clone(), false, false)),
        op: Op::Less,
        rhs: Arc::new(Base::LoadVariable(length_var.clone(), false, false))
    });

    // Increment instructions
    // index += 1
    // first = false
    // last = index == length - 1
    let mut inc_instructions = Instructions::default();
    {
        inc_instructions.push(Arc::new(OpIns {
            lhs: Arc::new(Base::LoadVariable(index_var.clone(), false, false)),
            op: Op::Add,
            rhs: Arc::new(Base::Literal(Val::Num(Num::Int(1)))),
        }));
        inc_instructions.push(Arc::new(Base::SetVariable(index_var.clone())));
        
        inc_instructions.push(Arc::new(Base::Literal(Val::Bool(false))));
        inc_instructions.push(Arc::new(Base::SetVariable(first_var)));
        inc_instructions.push(Arc::new(OpIns {
            lhs: Arc::new(Base::LoadVariable(index_var.clone(), false, false)),
            op: Op::Eq,
            rhs: Arc::new(OpIns {
                lhs: Arc::new(Base::LoadVariable(length_var, false, false)),
                op: Op::Sub,
                rhs: Arc::new(Base::Literal(Val::Num(Num::Int(1)))),
            }),
        }));
        inc_instructions.push(Arc::new(Base::SetVariable(last_var)));
    }


    // Inner instructions (declare the variable each time)
    let mut inner_instructions = Instructions::default();
    let mut vartype = Type::Void;
    if let Some(vt) = inner.typed { vartype = vt; }

    inner_instructions.push(Arc::new(FuncCall {
        func: None,
        search: Some(literal!("iterable.at")),
        stack: false,
        args: vector![Arc::new(Base::LoadVariable(index_var, false, false)) as Arc<dyn Instruction>],
        as_ref: inner.as_ref,
        cnull: true,
        oself: None,
    }));

    if !vartype.empty() { // cast to the right type
        inner_instructions.push(Arc::new(Base::Cast(vartype.clone())));
    }
    if inner.is_const {
        inner_instructions.push(Arc::new(Base::DeclareConstVar(inner.varname, vartype)));
    } else {
        inner_instructions.push(Arc::new(Base::DeclareVar(inner.varname, vartype)));
    }

    inner_instructions.append(&ins);

    let mut tag = None;
    if let Some(ltag) = loop_tag {
        tag = Some(ArcStr::from(ltag));
    }

    let while_ins: Arc<dyn Instruction> = Arc::new(WhileIns {
        tag,
        test,
        ins: inner_instructions.instructions,
        declare: Some(Arc::new(Block { ins: declare_instructions.instructions })),
        inc: Some(Arc::new(Block { ins: inc_instructions.instructions })),
    });
    Ok((input, vector![while_ins]))
}

/// Inner info.
fn inner_loop(input: &str) -> IResult<&str, LoopInner, StofParseError> {
    alt((const_var_in, var_in)).parse(input)
}

struct LoopInner {
    pub varname: ArcStr,
    pub is_const: bool,
    pub typed: Option<Type>,
    pub expr: Arc<dyn Instruction>,
    pub as_ref: bool,
}

/// Const var in.
fn const_var_in(input: &str) -> IResult<&str, LoopInner, StofParseError> {
    let (input, _) = multispace0(input)?;
    let (input, varname) = preceded(tag("const"), preceded(multispace0, ident)).parse(input)?;
    let (input, typed) = opt(preceded(multispace0, preceded(char(':'), parse_type))).parse(input)?;
    let (input, _) = delimited(multispace0, tag("in"), multispace0).parse(input)?;
    let (input, as_ref) = opt(terminated(char('&'), multispace0)).parse(input)?;
    let (input, expr) = expr(input)?;
    Ok((input, LoopInner { varname: varname.into(), is_const: true, typed, expr, as_ref: as_ref.is_some() }))
}

/// Var in.
fn var_in(input: &str) -> IResult<&str, LoopInner, StofParseError> {
    let (input, _) = multispace0(input)?;
    let (input, varname) = preceded(tag("let"), preceded(multispace0, ident)).parse(input)?;
    let (input, typed) = opt(preceded(multispace0, preceded(char(':'), parse_type))).parse(input)?;
    let (input, _) = delimited(multispace0, tag("in"), multispace0).parse(input)?;
    let (input, as_ref) = opt(terminated(char('&'), multispace0)).parse(input)?;
    let (input, expr) = expr(input)?;
    Ok((input, LoopInner { varname: varname.into(), is_const: false, typed, expr, as_ref: as_ref.is_some() }))
}
