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
use nom::{branch::alt, bytes::complete::tag, character::complete::{char, multispace0}, combinator::{opt, peek}, multi::fold_many0, sequence::{delimited, preceded, terminated}, IResult, Parser};
use rustc_hash::FxHashMap;
use crate::{model::Graph, parser::{context::ParseContext, expr::expr, statement::{block, statement}, whitespace::whitespace}, runtime::{instruction::Instruction, instructions::{block::Block, switch::SwitchIns}}};


/// Switch statement.
/// Case values must be literal exprs or evaluatable without the graph.
pub fn switch_statement(input: &str) -> IResult<&str, Vector<Arc<dyn Instruction>>> {
    let (input, _) = whitespace(input)?;
    let (input, value_expr) = preceded(terminated(tag("switch"), multispace0), delimited(char('('), expr, char(')'))).parse(input)?;
    let (input, cases) = preceded(preceded(multispace0, char('{')), case_statements).parse(input)?;

    let (input, default_ins) = opt(default_case).parse(input)?;
    let (input, _) = whitespace(input)?;
    let (input, _) = char('}')(input)?;

    let mut map = FxHashMap::default();
    let mut graph = Graph::default();
    let mut context = ParseContext::new(&mut graph); // need an eval context for values
    let mut stacked_values = Vec::new();
    for case in cases {
        let val = context.eval(case.expr).expect("failed to evaluate switch statement value expr");
        if let Some(ins) = case.ins {
            let block = Arc::new(Block { ins }) as Arc<dyn Instruction>;
            for sv in stacked_values.drain(..) {
                map.insert(sv, block.clone());
            }
            map.insert(val, block);
        } else {
            stacked_values.push(val);
        }
    }

    let switch_ins = SwitchIns {
        map,
        def: default_ins
    };
    
    Ok((input, vector![value_expr, Arc::new(switch_ins)]))
}


/// Fold many case statements into a singular instruction vector.
fn case_statements(input: &str) -> IResult<&str, Vector<Case>> {
    fold_many0(
        case,
        Vector::default,
        move |mut statements, current| {
            statements.push_back(current);
            statements
        }
    ).parse(input)
}


#[derive(Debug, Clone)]
struct Case {
    pub expr: Arc<dyn Instruction>,
    pub ins: Option<Vector<Arc<dyn Instruction>>>, // fallthrough if None
}


/// Parse an individual case.
/// case expr: block | statement // do the thing
/// case expr: // fallthrough
fn case(input: &str) -> IResult<&str, Case> {
    let (input, _) = whitespace(input)?;
    let (input, expr) = preceded(tag("case"), preceded(multispace0, expr)).parse(input)?;
    let (input, _) = delimited(multispace0, char(':'), multispace0).parse(input)?;

    let (input, peeked) = opt(peek(alt((tag("default"), tag("case"))))).parse(input)?;
    if peeked.is_some() { return Ok((input, Case { expr, ins: None })); }

    let (input, ins) = opt(alt((
        block,
        statement
    ))).parse(input)?;

    Ok((input, Case {
        expr,
        ins
    }))
}


/// Parse default case.
/// default: block | statement
fn default_case(input: &str) -> IResult<&str, Vector<Arc<dyn Instruction>>> {
    let (input, _) = whitespace(input)?;
    let (input, _) = preceded(tag("default"), delimited(multispace0, char(':'), multispace0)).parse(input)?;
    let (input, ins) = alt((
        block,
        statement
    )).parse(input)?;
    Ok((input, ins))
}
