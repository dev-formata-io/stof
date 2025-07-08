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
use nom::{branch::alt, bytes::complete::tag, character::complete::{char, multispace0}, combinator::{opt, peek}, multi::{separated_list0, separated_list1}, sequence::{delimited, preceded, separated_pair}, IResult, Parser};
use crate::{parser::{expr::{graph::{chained_var_func, graph_expr}, literal::literal_expr, math::math_expr}, statement::{block, switch::switch_statement}, types::parse_type, whitespace::whitespace}, runtime::{instruction::Instruction, instructions::{block::Block, list::{ListIns, NEW_LIST}, map::{MapIns, NEW_MAP}, set::{SetIns, NEW_SET}, tup::{TupIns, NEW_TUP}, Base, AWAIT, NOOP, NOT_TRUTHY, TYPE_NAME, TYPE_OF}}};

pub mod literal;
pub mod math;
pub mod graph;


/// Parse an expression.
pub fn expr(input: &str) -> IResult<&str, Arc<dyn Instruction>> {
    let (input, ins) = alt([
        await_expr,
        typename_expr,
        typeof_expr,
        tup_expr,
        list_expr,
        set_expr,
        map_expr,
        math_expr,
        not_expr,
        block_expr,
        switch_expr,
        literal_expr,
        graph_expr,
        wrapped_expr,
    ]).parse(input)?;

    // TODO: nullcheck operator "??"

    // Peek at the next value, if its async, then don't do the as below...
    let (input, peek_async) = opt(peek(preceded(multispace0, tag("async")))).parse(input)?;
    if peek_async.is_some() {
        return Ok((input, ins));
    }

    // Optional "as Type" cast
    let (input, cast_type) = opt(preceded(delimited(multispace0, tag("as"), multispace0), parse_type)).parse(input)?;
    if let Some(cast) = cast_type {
        let mut block = Block::default();
        block.ins.push_back(ins);
        block.ins.push_back(Arc::new(Base::Cast(cast)));
        Ok((input, Arc::new(block)))
    } else {
        Ok((input, ins))
    }
}


/// Block expression.
pub fn block_expr(input: &str) -> IResult<&str, Arc<dyn Instruction>> {
    let (input, statements) = block(input)?;
    if statements.is_empty() { return Ok((input, NOOP.clone())); }
    Ok((input, Arc::new(Block { ins: statements })))
}


/// Switch expression.
pub fn switch_expr(input: &str) -> IResult<&str, Arc<dyn Instruction>> {
    let (input, statements) = switch_statement(input)?;
    if statements.is_empty() { return Ok((input, NOOP.clone())); }
    Ok((input, Arc::new(Block { ins: statements })))
}


/// List contructor expression.
pub fn list_expr(input: &str) -> IResult<&str, Arc<dyn Instruction>> {
    let (input, _) = whitespace(input)?;
    let (input, exprs) = delimited(
        char('['),
        separated_list0(char(','), expr),
        char(']')
    ).parse(input)?;

    // Optional chained calls here
    // Ex. [3, 4].at(0)
    let (input, additional) = opt(preceded(char('.'), separated_list1(char('.'), chained_var_func))).parse(input)?;

    let mut block = Block::default();
    block.ins.push_back(NEW_LIST.clone());
    for expr in exprs {
        block.ins.push_back(Arc::new(ListIns::AppendList(expr)));
    }
    if let Some(additional) = additional {
        for ins in additional {
            block.ins.push_back(ins);
        }
    }

    Ok((input, Arc::new(block)))
}


/// Tuple contructor expression.
pub fn tup_expr(input: &str) -> IResult<&str, Arc<dyn Instruction>> {
    let (input, _) = whitespace(input)?;
    let (input, exprs) = delimited(
        char('('),
        separated_list1(char(','), expr),
        char(')')
    ).parse(input)?;

    if exprs.len() < 2 {
        return Err(nom::Err::Error(nom::error::Error {
            input: "a tuple requires at least 2 values",
            code: nom::error::ErrorKind::Count
        }));
    }

    // Optional chained calls here
    // Ex. (3, 4).at(0)
    let (input, additional) = opt(preceded(char('.'), separated_list1(char('.'), chained_var_func))).parse(input)?;

    let mut block = Block::default();
    block.ins.push_back(NEW_TUP.clone());
    for expr in exprs {
        block.ins.push_back(Arc::new(TupIns::AppendTup(expr)));
    }
    if let Some(additional) = additional {
        for ins in additional {
            block.ins.push_back(ins);
        }
    }

    Ok((input, Arc::new(block)))
}


/// Set contructor expression.
pub fn set_expr(input: &str) -> IResult<&str, Arc<dyn Instruction>> {
    let (input, _) = whitespace(input)?;
    let (input, exprs) = delimited(
        char('{'),
        separated_list0(char(','), expr),
        char('}')
    ).parse(input)?;

    // Optional chained calls here
    // Ex. {3, 4}.at(0)
    let (input, additional) = opt(preceded(char('.'), separated_list1(char('.'), chained_var_func))).parse(input)?;

    let mut block = Block::default();
    block.ins.push_back(NEW_SET.clone());
    for expr in exprs {
        block.ins.push_back(Arc::new(SetIns::AppendSet(expr)));
    }
    if let Some(additional) = additional {
        for ins in additional {
            block.ins.push_back(ins);
        }
    }

    Ok((input, Arc::new(block)))
}


/// Map contructor expression.
pub fn map_expr(input: &str) -> IResult<&str, Arc<dyn Instruction>> {
    let (input, _) = whitespace(input)?;
    let (input, exprs) = delimited(
        char('{'),
        separated_list0(char(','), separated_pair(expr, char(':'), expr)),
        char('}')
    ).parse(input)?;

    // Optional chained calls here
    // Ex. {'a': 3, 'b': 4}.at('b')
    let (input, additional) = opt(preceded(char('.'), separated_list1(char('.'), chained_var_func))).parse(input)?;

    let mut block = Block::default();
    block.ins.push_back(NEW_MAP.clone());
    for expr in exprs {
        block.ins.push_back(Arc::new(MapIns::AppendMap(expr)));
    }
    if let Some(additional) = additional {
        for ins in additional {
            block.ins.push_back(ins);
        }
    }

    Ok((input, Arc::new(block)))
}


/// Await expression.
pub fn await_expr(input: &str) -> IResult<&str, Arc<dyn Instruction>> {
    let (input, _) = whitespace(input)?;
    let (input, ins) = preceded(tag("await"), expr).parse(input)?;
    
    let mut block = Block::default();
    block.ins.push_back(ins); // a promise (maybe)
    block.ins.push_back(AWAIT.clone()); // will only do something if its a promise
    
    Ok((input, Arc::new(block)))
}


/// Wrapped expression.
pub fn wrapped_expr(input: &str) -> IResult<&str, Arc<dyn Instruction>> {
    let (input, _) = whitespace(input)?;
    let (input, ins) = delimited(char('('), delimited(multispace0, expr, multispace0), char(')')).parse(input)?;
    let (input, additional) = opt(preceded(char('.'), separated_list1(char('.'), chained_var_func))).parse(input)?;

    if additional.is_none() { return Ok((input, ins)); }

    let mut block = Block::default();
    block.ins.push_back(ins);
    if let Some(additional) = additional {
        for ins in additional {
            block.ins.push_back(ins);
        }
    }
    Ok((input, Arc::new(block)))
}


/// Not expression.
pub fn not_expr(input: &str) -> IResult<&str, Arc<dyn Instruction>> {
    let (input, _) = whitespace(input)?;
    let (input, ins) = preceded(char('!'), expr).parse(input)?;
    
    let mut block = Block::default();
    block.ins.push_back(ins);
    block.ins.push_back(NOT_TRUTHY.clone());
    
    Ok((input, Arc::new(block)))
}


/// TypeOf expression.
pub fn typeof_expr(input: &str) -> IResult<&str, Arc<dyn Instruction>> {
    let (input, _) = whitespace(input)?;
    let (input, ins) = preceded(tag("typeof"), expr).parse(input)?;

    let mut block = Block::default();
    block.ins.push_back(ins);
    block.ins.push_back(TYPE_OF.clone());
    
    Ok((input, Arc::new(block)))
}


/// TypeName expression.
/// A specific type instead of a general one Ex. "MyObj" instead of "obj"
pub fn typename_expr(input: &str) -> IResult<&str, Arc<dyn Instruction>> {
    let (input, _) = whitespace(input)?;
    let (input, ins) = preceded(tag("typename"), expr).parse(input)?;

    let mut block = Block::default();
    block.ins.push_back(ins);
    block.ins.push_back(TYPE_NAME.clone());
    
    Ok((input, Arc::new(block)))
}


#[cfg(test)]
mod tests {
    use crate::{model::Graph, parser::expr::expr, runtime::Runtime};

    #[test]
    fn parse_map_expr() {
        let (_input, res) = expr("{'a': 1, 'b': 2, 'c': 3}").unwrap();
        let mut graph = Graph::default();
        let res = Runtime::eval(&mut graph, res).unwrap();
        //println!("{}", res.print(&graph));
        assert!(res.map());
    }

    #[test]
    fn parse_list_expr() {
        let (_input, res) = expr("['a', 2, 'c']").unwrap();
        let mut graph = Graph::default();
        let res = Runtime::eval(&mut graph, res).unwrap();
        //println!("{}", res.print(&graph));
        assert!(res.list());
    }

    #[test]
    fn parse_tup_expr() {
        let (_input, res) = expr("('a', 2, 'c')").unwrap();
        let mut graph = Graph::default();
        let res = Runtime::eval(&mut graph, res).unwrap();
        //println!("{}", res.print(&graph));
        assert!(res.tup());
    }

    #[test]
    fn parse_set_expr() {
        let (_input, res) = expr("{'a', 2, 'c'}").unwrap();
        let mut graph = Graph::default();
        let res = Runtime::eval(&mut graph, res).unwrap();
        //println!("{}", res.print(&graph));
        assert!(res.set());
    }

    #[test]
    fn parse_wrapped_expr() {
        let (_input, res) = expr("(['a', 2, 'c'])").unwrap();
        let mut graph = Graph::default();
        let res = Runtime::eval(&mut graph, res).unwrap();
        //println!("{}", res.print(&graph));
        assert!(res.list());
    }

    #[test]
    fn await_passthrough_expr() {
        let (_input, res) = expr("await 42").unwrap();
        let mut graph = Graph::default();
        let res = Runtime::eval(&mut graph, res).unwrap();
        assert_eq!(res, 42.into());
    }
}
