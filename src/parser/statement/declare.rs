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
use imbl::vector;
use nom::{branch::alt, bytes::complete::tag, character::complete::{char, multispace0, multispace1}, combinator::{map, opt}, sequence::{delimited, preceded}, IResult, Parser};
use crate::{parser::{doc::{err_fail, StofParseError}, expr::expr, ident::ident, string::{double_string, single_string}, types::parse_type, whitespace::whitespace}, runtime::{instruction::Instruction, instructions::{destructure::{DestructureIns, Pattern}, Base}, Type, Val}};


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
    alt((declare_const_var, declare_mut_var, declare_null_var, declare_pattern)).parse(input)
}


/// Destructuring declaration: variables from an object's fields (or a map's keys), or a list's (or tuple's) items.
/// let { name, age: years, role = 'member', address: { city } } = user;
/// const [first, second] = list;
/// const (key, value) = pair;
pub(self) fn declare_pattern(input: &str) -> IResult<&str, Vector<Arc<dyn Instruction>>, StofParseError> {
    let (input, keyword) = alt((tag("let"), tag("const"))).parse(input)?;
    let (input, pattern) = preceded(multispace0, pattern).parse(input)?;
    let (input, _) = delimited(multispace0, char('='), multispace0).parse(input).map_err(err_fail)?;
    let (input, expr) = expr(input).map_err(err_fail)?;
    Ok((input, vector![expr, Arc::new(DestructureIns { pattern, is_const: keyword == "const" }) as Arc<dyn Instruction>]))
}

/// A destructuring pattern: `{ field, field: target, field = default, .. }`, `[target, ..]`, or `(target, ..)`,
/// where a target is a variable name ("_" to skip) or another pattern.
pub fn pattern(input: &str) -> IResult<&str, Pattern, StofParseError> {
    let (mut input, open) = alt((char('{'), char('['), char('('))).parse(input)?;
    let close = match open { '{' => '}', '[' => ']', _ => ')' };
    let mut fields = Vec::new();
    let mut items = Vec::new();
    loop {
        let (rest, _) = whitespace(input)?;
        if let Some(rest) = rest.strip_prefix(close) { input = rest; break; } // empty, or after a trailing comma

        let (rest, target) = if open == '{' {
            let (rest, (key, quoted)) = alt((map(ident, |key| (key.to_string(), false)), map(alt((double_string, single_string)), |key| (key, true)))).parse(rest).map_err(err_fail)?;
            let (rest, target) = opt(preceded(delimited(multispace0, char(':'), multispace0), target)).parse(rest).map_err(err_fail)?;
            let target = match target {
                Some(target) => target,
                None if quoted => return Err(nom::Err::Failure(StofParseError::from(format!("'{key}' needs a variable name to destructure into: {{ '{key}': name }}")))),
                None => { note_declared(&key); Pattern::Name(key.as_str().into()) },
            };
            (rest, (Some(key), target))
        } else {
            let (rest, target) = target(rest).map_err(err_fail)?;
            (rest, (None, target))
        };
        let (rest, default) = opt(preceded(delimited(multispace0, char('='), multispace0), expr)).parse(rest).map_err(err_fail)?;
        match target {
            (Some(key), target) => fields.push((key.into(), target, default)),
            (None, target) => items.push((target, default)),
        }

        let (rest, _) = whitespace(rest)?;
        if let Some(rest) = rest.strip_prefix(',') { input = rest; continue; }
        let (rest, _) = char(close)(rest).map_err(err_fail)?;
        input = rest;
        break;
    }
    Ok((input, if open == '{' { Pattern::Fields(fields) } else { Pattern::Items(items) }))
}

/// A destructuring target: a variable name or a nested pattern.
fn target(input: &str) -> IResult<&str, Pattern, StofParseError> {
    alt((pattern, map(ident, |name| {
        if name != "_" { note_declared(name); }
        Pattern::Name(name.into())
    }))).parse(input)
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


/// Using declaration: a const variable whose object is dropped when its scope ends (block or function exit,
/// including return, break, and errors).
/// using tmp = new { .. }
pub fn declare_using_var(input: &str) -> IResult<&str, Vector<Arc<dyn Instruction>>, StofParseError> {
    let (input, _) = whitespace(input)?;
    let (input, varname) = delimited(tag("using"), preceded(multispace1, ident), multispace0).parse(input)?;
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
    block.push_back(Arc::new(Base::MarkUsing(varname.to_string().into())));
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
