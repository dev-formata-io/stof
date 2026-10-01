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

use std::{cell::RefCell, sync::Arc};
use imbl::Vector;
use nom::{branch::alt, bytes::complete::tag, character::complete::{char, multispace0}, combinator::opt, sequence::{delimited, preceded}, IResult, Parser};
use crate::{parser::{doc::StofParseError, expr::expr, ident::ident, types::parse_type, whitespace::whitespace}, runtime::{instruction::Instruction, instructions::Base, Type, Val}};


thread_local! {
    /// Names declared by let/const statements since the last take (statement parsers have no parse context).
    /// The function parser drains this to check its locals (Ex. library shadowing warnings).
    static DECLARED_NAMES: RefCell<Vec<String>> = RefCell::new(Vec::new());
}

/// Take (and clear) the variable names declared since the last call.
pub(crate) fn take_declared_names() -> Vec<String> {
    DECLARED_NAMES.with(|names| std::mem::take(&mut *names.borrow_mut()))
}

/// Note a variable name declared in a function body (let/const, loop and catch variables, arrow params).
pub(crate) fn note_declared(name: &str) {
    DECLARED_NAMES.with(|names| names.borrow_mut().push(name.to_string()));
}


thread_local! {
    /// Bare names read or called in function bodies since the last take: (name, is call, source address).
    /// Checked against the function's variables after it is parsed (unknown names are always null).
    static REFERENCED_NAMES: RefCell<Vec<(String, bool, usize)>> = RefCell::new(Vec::new());
}

/// Note a bare name used in an expression (Ex. "total" in "total + 1", "foo" in "foo()").
pub(crate) fn note_referenced(name: &str, call: bool, position: &str) {
    REFERENCED_NAMES.with(|names| names.borrow_mut().push((name.to_string(), call, position.as_ptr() as usize)));
}

thread_local! {
    /// First names of assignment targets anywhere in the document (Ex. "Storage" in "Storage.graph = map()").
    /// Assigning to a name that doesn't exist creates a root, which other functions can then use.
    static ASSIGNED_NAMES: RefCell<Vec<String>> = RefCell::new(Vec::new());
}

/// Note the first name of an assignment target.
pub(crate) fn note_assigned(name: &str) {
    ASSIGNED_NAMES.with(|names| names.borrow_mut().push(name.to_string()));
}

/// Take (and clear) the assignment target names noted so far.
pub(crate) fn take_assigned_names() -> Vec<String> {
    ASSIGNED_NAMES.with(|names| std::mem::take(&mut *names.borrow_mut()))
}

/// Take (and clear) the names referenced since the last call.
pub(crate) fn take_referenced_names() -> Vec<(String, bool, usize)> {
    REFERENCED_NAMES.with(|names| std::mem::take(&mut *names.borrow_mut()))
}


/// Declare a variable.
pub fn declare_statement(input: &str) -> IResult<&str, Vector<Arc<dyn Instruction>>, StofParseError> {
    let (input, _) = whitespace(input)?;
    alt((declare_const_var, declare_mut_var, declare_null_var)).parse(input)
}


/// Mutable variable declaration.
/// let var: int = 45
pub(self) fn declare_mut_var(input: &str) -> IResult<&str, Vector<Arc<dyn Instruction>>, StofParseError> {
    let (input, varname) = delimited(tag("let"), preceded(multispace0, ident), multispace0).parse(input)?;
    let (input, cast_type) = opt(preceded(char(':'), parse_type)).parse(input)?; 
    let (input, _) = delimited(multispace0, char('='), multispace0).parse(input)?;
    let (input, expr) = expr(input)?;
    note_declared(varname);

    let mut block = Vector::default();
    block.push_back(expr);
    if let Some(cast_type) = cast_type {
        block.push_back(Arc::new(Base::Cast(cast_type.clone())));
        block.push_back(Arc::new(Base::DeclareVar(varname.to_string().into(), cast_type)));
    } else {
        block.push_back(Arc::new(Base::DeclareVar(varname.to_string().into(), Type::Void))); // no type enforcement
    }
    Ok((input, block))
}


/// Const variable declaration.
/// const var: int = 45
pub(self) fn declare_const_var(input: &str) -> IResult<&str, Vector<Arc<dyn Instruction>>, StofParseError> {
    let (input, varname) = delimited(tag("const"), preceded(multispace0, ident), multispace0).parse(input)?;
    let (input, cast_type) = opt(preceded(char(':'), parse_type)).parse(input)?; 
    let (input, _) = delimited(multispace0, char('='), multispace0).parse(input)?;
    let (input, expr) = expr(input)?;
    note_declared(varname);

    let mut block = Vector::default();
    block.push_back(expr);
    if let Some(cast_type) = cast_type {
        block.push_back(Arc::new(Base::Cast(cast_type.clone())));
        block.push_back(Arc::new(Base::DeclareConstVar(varname.to_string().into(), cast_type)));
    } else {
        block.push_back(Arc::new(Base::DeclareConstVar(varname.to_string().into(), Type::Void))); // no type enforcement
    }
    Ok((input, block))
}


/// Mutable empty var declaration (initialized to null).
/// let var;
pub(self) fn declare_null_var(input: &str) -> IResult<&str, Vector<Arc<dyn Instruction>>, StofParseError> {
    let (input, varname) = delimited(tag("let"), preceded(multispace0, ident), multispace0).parse(input)?;
    let (input, cast_type) = opt(preceded(char(':'), parse_type)).parse(input)?; 
    note_declared(varname);

    let mut block = Vector::default();
    block.push_back(Arc::new(Base::Literal(Val::Null)) as Arc<dyn Instruction>);
    if let Some(cast_type) = cast_type {
        block.push_back(Arc::new(Base::Cast(cast_type.clone())));
        block.push_back(Arc::new(Base::DeclareVar(varname.to_string().into(), cast_type)));
    } else {
        block.push_back(Arc::new(Base::DeclareVar(varname.to_string().into(), Type::Void))); // no type enforcement
    }
    Ok((input, block))
}
