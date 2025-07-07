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
use imbl::{vector, Vector};
use nom::{branch::alt, combinator::map, bytes::complete::tag, character::complete::{char, multispace0}, combinator::{opt, value}, multi::fold_many0, sequence::{delimited, pair, preceded, terminated}, IResult, Parser};
use crate::{parser::{expr::expr, statement::{assign::assign, declare::declare_statement}, whitespace::whitespace}, runtime::{instruction::Instruction, instructions::{empty::EmptyIns, ret::RetIns, POP_SYMBOL_SCOPE, PUSH_SYMBOL_SCOPE}}};

pub mod declare;
pub mod assign;


/// Parse a block of statements.
pub fn block(input: &str) -> IResult<&str, Vector<Arc<dyn Instruction>>> {
    let (input, _) = whitespace(input)?;
    let (input, mut statements) = delimited(
        char('{'), 
        multistatements,
        preceded(whitespace, char('}'))
    ).parse(input)?;
    if statements.is_empty() { return Ok((input, Default::default())); }

    statements.push_front(PUSH_SYMBOL_SCOPE.clone());
    statements.push_back(POP_SYMBOL_SCOPE.clone());
    Ok((input, statements))
}


/// Fold many statements in the same scope into a singular instruction vector.
fn multistatements(input: &str) -> IResult<&str, Vector<Arc<dyn Instruction>>> {
    let mut seen_return = false;
    fold_many0(
        statement,
        Vector::default,
        move |mut statements, current| {
            if !seen_return { // only push instructions up to and including a return statement per scope
                for ins in current {
                    if let Some(_) = ins.as_dyn_any().downcast_ref::<RetIns>() { seen_return = true; }
                    statements.push_back(ins);
                    if seen_return { break; }
                }
            }
            statements
        }
    ).parse(input)
}


/// Parse a singular statement into instructions.
pub fn statement(input: &str) -> IResult<&str, Vector<Arc<dyn Instruction>>> {
    let (input, statements) = alt((
        // return
        return_statement,

        // declarations & assignment
        terminated(declare_statement, preceded(multispace0, char(';'))),
        terminated(assign, preceded(multispace0, char(';'))),
        
        // block, standalone expr, and empty statement
        expr_statement,
        block,
        value(Vector::default(), preceded(whitespace, char(';'))) // empty statement ";"
    )).parse(input)?;
    Ok((input, statements))
}


/// Return statement.
/// Either an empty "return;" or with an expression "return 5;".
fn return_statement(input: &str) -> IResult<&str, Vector<Arc<dyn Instruction>>> {
    let (input, _) = whitespace(input)?;
    let (input, res) = alt((
        value(Arc::new(RetIns { expr: None }) as Arc<dyn Instruction>, terminated(tag("return"), preceded(multispace0, char(';')))),
        map(delimited(tag("return"), expr, preceded(multispace0, char(';'))), |expr| Arc::new(RetIns { expr: Some(expr) }) as Arc<dyn Instruction>)
    )).parse(input)?;
    Ok((input, vector![res]))
}


/// Empty expression.
/// Clears the stack of all pushed values during this expression if there's a ';' at the end.
/// Otherwise, it functions as a return statement.
fn expr_statement(input: &str) -> IResult<&str, Vector<Arc<dyn Instruction>>> {
    let (input, _) = whitespace(input)?;
    let (input, ins) = pair(expr, opt(char(';'))).parse(input)?;
    
    let mut res = Vector::default();
    if ins.1.is_some() {
        res.push_back(Arc::new(EmptyIns { ins: ins.0 }) as Arc<dyn Instruction>);
    } else {
        res.push_back(ins.0); // return variant of the expr (put here for parse performance reasons)
    }
    Ok((input, res))
}


#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use crate::{model::Graph, parser::statement::block, runtime::{instructions::block::Block, Runtime}};

    #[test]
    fn declare_ret_block() {
        let (_input, res) = block("{  const x = 10; ;; ; ; { let u = &x; u }  }").unwrap();
        //println!("{res:?}");
        let mut graph = Graph::default();
        let val = Runtime::eval(&mut graph, Arc::new(Block { ins: res })).unwrap();
        assert_eq!(val, 10.into());
    }

    #[test]
    fn assignment() {
        let (_input, res) = block(r#"{
            let v = 42;
            
            v *= 8;
            v -= 300.;
            v += 4;

            v as int
        }"#).unwrap();

        //println!("{res:?}");
        let mut graph = Graph::default();
        let val = Runtime::eval(&mut graph, Arc::new(Block { ins: res })).unwrap();
        assert_eq!(val, 40.into());
    }
}
