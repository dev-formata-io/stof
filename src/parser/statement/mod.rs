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
use crate::{parser::{expr::expr, statement::{assign::assign, declare::declare_statement, fors::for_loop, ifs::if_statement, switch::switch_statement, trycatch::try_catch_statement, whiles::{break_statement, continue_statement, while_statement}}, whitespace::whitespace}, runtime::{instruction::Instruction, instructions::{empty::EmptyIns, ret::RetIns, POP_SYMBOL_SCOPE, PUSH_SYMBOL_SCOPE}}};

pub mod declare;
pub mod assign;
pub mod ifs;
pub mod whiles;
pub mod fors;
pub mod switch;
pub mod trycatch;


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
        // control
        if_statement,
        while_statement,
        for_loop,
        switch_statement,
        try_catch_statement,
        terminated(continue_statement, preceded(multispace0, char(';'))),
        terminated(break_statement, preceded(multispace0, char(';'))),
        
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
        res.push_back(Arc::new(RetIns { expr: Some(ins.0) })); // return variant of the expr (put here for parse performance reasons)
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

    #[test]
    fn if_statement() {
        let (_input, res) = block(r#"{
            let x = 10;
            if (x > 40) 100
            else if (x == 32) { 42 } // this is a comment here
            else if (x == 10) 300
            else 200
        }"#).unwrap();

        //println!("{res:?}");
        let mut graph = Graph::default();
        let val = Runtime::eval(&mut graph, Arc::new(Block { ins: res })).unwrap();
        assert_eq!(val, 300.into());
    }

    #[test]
    fn while_statement() {
        let (_input, res) = block(r#"{
            let x = 0;
            let y = 200;
            'outer while (x < 1e4) {
                while (y > 0) {
                    if (y < 5 || x > 300) break 'outer;
                    y -= 1;
                }
                x += 1;
            }
            y
        }"#).unwrap();

        //println!("{res:?}");
        let mut graph = Graph::default();
        let val = Runtime::eval(&mut graph, Arc::new(Block { ins: res })).unwrap();
        //println!("{val:?}");
        assert_eq!(val, 4.into());
    }

    #[test]
    fn for_statement() {
        let (_input, res) = block(r#"{
            let total = 0;
            'outer for (let x = 0; x < 100; x += 1) {
                for (let y = 0; y < 50; y += 1) {
                    total += 1;
                    if (x > 80 && y > 30) break 'outer;
                }
                total += 1;
            }
            total
        }"#).unwrap();

        //println!("{res:?}");
        let mut graph = Graph::default();
        let val = Runtime::eval(&mut graph, Arc::new(Block { ins: res })).unwrap();
        //println!("{val:?}");
        assert_eq!(val, 4163.into());
    }

    #[test]
    fn switch_statement() {
        let (_input, res) = block(r#"{
            const val = 'hii';
            switch (val) {
                case 'a': 42
                case 'hi': 100
                case 'hello': 32
                default: {
                    700
                }
            }
        }"#).unwrap();

        //println!("{res:?}");
        let mut graph = Graph::default();
        let val = Runtime::eval(&mut graph, Arc::new(Block { ins: res })).unwrap();
        //println!("{val:?}");
        assert_eq!(val, 700.into());
    }

    #[test]
    fn try_catch_statement() {
        let (_input, res) = block(r#"{
            try 3.4.6 + 5
            catch (error: str) {
                error
            }
        }"#).unwrap();

        //println!("{res:?}");
        let mut graph = Graph::default();
        let val = Runtime::eval(&mut graph, Arc::new(Block { ins: res })).unwrap();
        //println!("{val:?}");
        assert_eq!(val, "NotImplemented".into());
    }
}
