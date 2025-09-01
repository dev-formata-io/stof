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

use colored::Colorize;
use nom::{bytes::complete::tag, character::complete::{char, multispace0}, combinator::opt, sequence::{delimited, preceded}, IResult, Parser};
use rustc_hash::FxHashMap;
use crate::{parser::{context::ParseContext, doc::{err_fail, StofParseError}, expr::expr, ident::ident, whitespace::whitespace}, runtime::Val};


pub mod semver;
pub mod whitespace;
pub mod number;
pub mod types;
pub mod ident;
pub mod string;
pub mod literal;
pub mod expr;
pub mod statement;

pub mod context;
pub mod func;
pub mod field;
pub mod doc;
pub mod import;
pub mod data;


/// Parse attributes.
pub(self) fn parse_attributes<'a>(input: &'a str, context: &mut ParseContext) -> IResult<&'a str, FxHashMap<String, Val>, StofParseError> {
    let mut map = FxHashMap::default();
    let mut input = input;
    loop {
        let res = parse_attribute(input, context);
        match res {
            Ok(attr) => {
                input = attr.0;
                map.insert(attr.1.0, attr.1.1);
            },
            Err(error) => {
                match error {
                    nom::Err::Error(_) => {
                        break;
                    },
                    nom::Err::Incomplete(_) => {
                        break;
                    },
                    nom::Err::Failure(_) => {
                        return Err(error);
                    }
                }
            }
        }
    }
    Ok((input, map))
}


/// Parse attribute.
pub(self) fn parse_attribute<'a>(input: &'a str, context: &mut ParseContext) -> IResult<&'a str, (String, Val), StofParseError> {
    let (input, _) = whitespace(input)?;
    let (input, _) = tag("#[").parse(input)?;
    let (input, name) = preceded(multispace0, ident).parse(input).map_err(err_fail)?;
    let (input, value_expr) = opt(delimited(char('('), expr, char(')'))).parse(input).map_err(err_fail)?;
    let (input, _) = preceded(multispace0, char(']')).parse(input).map_err(err_fail)?;

    let mut val = Val::Null;
    if let Some(expr) = value_expr {
        match context.eval(expr) {
            Ok(res) => {
                val = res;
            },
            Err(error) => {
                return Err(nom::Err::Failure(StofParseError::from(format!("{} '{name}' errored with {}", "attribute eval error:".dimmed(), error.to_string()))));
            }
        }
    }
    Ok((input, (String::from(name), val)))
}
