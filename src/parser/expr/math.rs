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
use nom::{branch::alt, bytes::complete::tag, character::complete::{char, multispace0}, combinator::map, multi::many0, sequence::preceded, IResult, Parser};
use crate::{parser::{doc::StofParseError, expr::{await_expr, block_expr, fmt_str::formatted_string_expr, graph::graph_expr, list_expr, literal::literal_expr, map_expr, not_expr, set_expr, switch_expr, tup_expr, typename_expr, typeof_expr, wrapped_expr}, whitespace::whitespace}, runtime::{instruction::Instruction, instructions::{ops::{Op, OpIns}, Base, NOOP}, Num, Val}};


/// Parse a math expr.
/// Logical ops are evaluated last (&&, ||) left to right.
/// Then comparison ops (==, !=, <=, >=, >, <) left to right.
/// Then addition and subtraction (+, -) left to right.
/// Then multiplication, etc. (*, /, %) left to right.
/// Then bitwise operations (|, &, ^, <<, >>) left to right.
/// Then the prefixes on a primary expr (-, !).
pub fn math_expr(input: &str) -> IResult<&str, Arc<dyn Instruction>, StofParseError> {
    let (input, _) = whitespace(input)?;
    logical(input)
}

/// Logical and and or (the highest level).
pub(self) fn logical(input: &str) -> IResult<&str, Arc<dyn Instruction>, StofParseError> {
    let (input, mut lhs) = compare(input)?;
    let (input, ops) = many0(
        alt((
            map(preceded(tag("&&"), compare), |ins| {
                OpIns { lhs: NOOP.clone(), op: Op::And, rhs: ins }
            }),
            map(preceded(tag("||"), compare), |ins| {
                OpIns { lhs: NOOP.clone(), op: Op::Or, rhs: ins }
            })
        ))
    ).parse(input)?;
    for mut op in ops {
        op.lhs = lhs;
        lhs = Arc::new(op) as Arc<dyn Instruction>;
    }
    Ok((input, lhs))
}

/// Comparison operations.
pub(self) fn compare(input: &str) -> IResult<&str, Arc<dyn Instruction>, StofParseError> {
    let (input, mut lhs) = add_sub(input)?;
    let (input, ops) = many0(
        alt((
            map(preceded(tag("=="), add_sub), |ins| {
                OpIns { lhs: NOOP.clone(), op: Op::Eq, rhs: ins }
            }),
            map(preceded(tag("!="), add_sub), |ins| {
                OpIns { lhs: NOOP.clone(), op: Op::Neq, rhs: ins }
            }),
            map(preceded(tag(">="), add_sub), |ins| {
                OpIns { lhs: NOOP.clone(), op: Op::GreaterOrEq, rhs: ins }
            }),
            map(preceded(tag("<="), add_sub), |ins| {
                OpIns { lhs: NOOP.clone(), op: Op::LessOrEq, rhs: ins }
            }),
            map(preceded(char('>'), add_sub), |ins| {
                OpIns { lhs: NOOP.clone(), op: Op::Greater, rhs: ins }
            }),
            map(preceded(char('<'), add_sub), |ins| {
                OpIns { lhs: NOOP.clone(), op: Op::Less, rhs: ins }
            })
        ))
    ).parse(input)?;
    for mut op in ops {
        op.lhs = lhs;
        lhs = Arc::new(op) as Arc<dyn Instruction>;
    }
    Ok((input, lhs))
}

/// Add subtract.
pub(self) fn add_sub(input: &str) -> IResult<&str, Arc<dyn Instruction>, StofParseError> {
    let (input, mut lhs) = mul_div(input)?;
    let (input, ops) = many0(
        alt((
            map(preceded(char('-'), mul_div), |ins| {
                OpIns { lhs: NOOP.clone(), op: Op::Sub, rhs: ins }
            }),
            map(preceded(char('+'), mul_div), |ins| {
                OpIns { lhs: NOOP.clone(), op: Op::Add, rhs: ins }
            })
        ))
    ).parse(input)?;
    for mut op in ops {
        op.lhs = lhs;
        lhs = Arc::new(op) as Arc<dyn Instruction>;
    }
    Ok((input, lhs))
}

/// Multiply, divide, or mod.
pub(self) fn mul_div(input: &str) -> IResult<&str, Arc<dyn Instruction>, StofParseError> {
    let (input, mut lhs) = bitwise(input)?;
    let (input, ops) = many0(
        alt((
            map(preceded(char('*'), bitwise), |ins| {
                OpIns { lhs: NOOP.clone(), op: Op::Mul, rhs: ins }
            }),
            map(preceded(char('/'), bitwise), |ins| {
                OpIns { lhs: NOOP.clone(), op: Op::Div, rhs: ins }
            }),
            map(preceded(char('%'), bitwise), |ins| {
                OpIns { lhs: NOOP.clone(), op: Op::Mod, rhs: ins }
            })
        ))
    ).parse(input)?;
    for mut op in ops {
        op.lhs = lhs;
        lhs = Arc::new(op) as Arc<dyn Instruction>;
    }
    Ok((input, lhs))
}

/// Bit and, or, xor, bshl, or bshr.
pub(self) fn bitwise(input: &str) -> IResult<&str, Arc<dyn Instruction>, StofParseError> {
    let (input, mut lhs) = primary(input)?;
    let (input, ops) = many0(
        alt((
            map(preceded(char('&'), primary), |ins| {
                OpIns { lhs: NOOP.clone(), op: Op::BAND, rhs: ins }
            }),
            map(preceded(char('|'), primary), |ins| {
                OpIns { lhs: NOOP.clone(), op: Op::BOR, rhs: ins }
            }),
            map(preceded(char('^'), primary), |ins| {
                OpIns { lhs: NOOP.clone(), op: Op::BXOR, rhs: ins }
            }),
            map(preceded(tag("<<"), primary), |ins| {
                OpIns { lhs: NOOP.clone(), op: Op::BSHL, rhs: ins }
            }),
            map(preceded(tag(">>"), primary), |ins| {
                OpIns { lhs: NOOP.clone(), op: Op::BSHR, rhs: ins }
            })
        ))
    ).parse(input)?;
    for mut op in ops {
        op.lhs = lhs;
        lhs = Arc::new(op) as Arc<dyn Instruction>;
    }
    Ok((input, lhs))
}

/// Primary element in a math expr.
fn primary(input: &str) -> IResult<&str, Arc<dyn Instruction>, StofParseError> {
    let (input, _) = multispace0(input)?;
    let (input, ins) = alt((
        atom,
        map(preceded(char('-'), atom), |ins| {
            Arc::new(OpIns { lhs: Arc::new(Base::Literal(Val::Num(Num::Int(-1)))), op: Op::Mul, rhs: ins }) as Arc<dyn Instruction>
        })
    )).parse(input)?;
    let (input, _) = multispace0(input)?;
    Ok((input, ins))
}

/// Smallest instruction in a math expr.
fn atom(input: &str) -> IResult<&str, Arc<dyn Instruction>, StofParseError> {
    alt([
        await_expr,
        typename_expr,
        typeof_expr,
        tup_expr,
        list_expr,
        set_expr,
        map_expr,
        not_expr,
        block_expr,
        switch_expr,
        literal_expr,
        formatted_string_expr,
        graph_expr,
        wrapped_expr,
    ]).parse(input)
}


#[cfg(test)]
mod tests {
    use crate::{model::Graph, parser::expr::math::math_expr, runtime::Runtime};

    #[test]
    fn add() {
        let (_input, res) = math_expr("6 + 3").unwrap();
        let mut graph = Graph::default();
        let val = Runtime::eval(&mut graph, res).unwrap();
        assert_eq!(val, 9.into());
    }

    #[test]
    fn sub() {
        let (_input, res) = math_expr("6 - 3").unwrap();
        let mut graph = Graph::default();
        let val = Runtime::eval(&mut graph, res).unwrap();
        assert_eq!(val, 3.into());
    }

    #[test]
    fn mul() {
        let (_input, res) = math_expr("6 * 3").unwrap();
        let mut graph = Graph::default();
        let val = Runtime::eval(&mut graph, res).unwrap();
        assert_eq!(val, 18.into());
    }

    #[test]
    fn div() {
        let (_input, res) = math_expr("6 / 3").unwrap();
        let mut graph = Graph::default();
        let val = Runtime::eval(&mut graph, res).unwrap();
        assert_eq!(val, 2.into());
    }

    #[test]
    fn rem() {
        let (_input, res) = math_expr("6 % 3").unwrap();
        let mut graph = Graph::default();
        let val = Runtime::eval(&mut graph, res).unwrap();
        assert_eq!(val, 0.into());
    }

    #[test]
    fn wrapped() {
        let (_input, res) = math_expr("6   + (3 - ( 4g   ))").unwrap();
        let mut graph = Graph::default();
        let val = Runtime::eval(&mut graph, res).unwrap();
        assert_eq!(val, 5.into());
    }

    #[test]
    fn mul_before_add() {
        let (_input, res) = math_expr("6 + 2 * 2").unwrap();
        let mut graph = Graph::default();
        let val = Runtime::eval(&mut graph, res).unwrap();
        assert_eq!(val, 10.into());
    }

    #[test]
    fn div_before_sub() {
        let (_input, res) = math_expr("35 - 10 / 2").unwrap();
        let mut graph = Graph::default();
        let val = Runtime::eval(&mut graph, res).unwrap();
        assert_eq!(val, 30.into());
    }

    #[test]
    fn order() {
        let (_input, res) = math_expr("4 + 2 * 3 / 6 - 5").unwrap();
        //println!("{res:?}");
        let mut graph = Graph::default();
        let val = Runtime::eval(&mut graph, res).unwrap();
        assert_eq!(val, 0.into());
    }

    #[test]
    fn together() {
        let (_input, res) = math_expr("4 + 2 * -(3 / 6) - -5 * (5 % 3)").unwrap();
        //println!("{res:?}");
        let mut graph = Graph::default();
        let val = Runtime::eval(&mut graph, res).unwrap();
        assert_eq!(val, (4 + 2 * -(3 / 6) - -5 * (5 % 3)).into());
    }

    #[test]
    fn less() {
        let (_input, res) = math_expr("78 < 100").unwrap();
        //println!("{res:?}");
        let mut graph = Graph::default();
        let val = Runtime::eval(&mut graph, res).unwrap();
        //println!("{val:?}");
        assert_eq!(val, true.into());
    }

    #[test]
    fn greater() {
        let (_input, res) = math_expr("78 > 100").unwrap();
        //println!("{res:?}");
        let mut graph = Graph::default();
        let val = Runtime::eval(&mut graph, res).unwrap();
        //println!("{val:?}");
        assert_eq!(val, false.into());
    }

    #[test]
    fn less_or_eq() {
        let (_input, res) = math_expr("100 <= 100").unwrap();
        //println!("{res:?}");
        let mut graph = Graph::default();
        let val = Runtime::eval(&mut graph, res).unwrap();
        //println!("{val:?}");
        assert_eq!(val, true.into());
    }

    #[test]
    fn greater_or_eq() {
        let (_input, res) = math_expr("101 >= 100").unwrap();
        //println!("{res:?}");
        let mut graph = Graph::default();
        let val = Runtime::eval(&mut graph, res).unwrap();
        //println!("{val:?}");
        assert_eq!(val, true.into());
    }

    #[test]
    fn equal() {
        let (_input, res) = math_expr("100 == 100").unwrap();
        //println!("{res:?}");
        let mut graph = Graph::default();
        let val = Runtime::eval(&mut graph, res).unwrap();
        //println!("{val:?}");
        assert_eq!(val, true.into());
    }

    #[test]
    fn not_equal() {
        let (_input, res) = math_expr("70 + 30 != 100").unwrap();
        //println!("{res:?}");
        let mut graph = Graph::default();
        let val = Runtime::eval(&mut graph, res).unwrap();
        //println!("{val:?}");
        assert_eq!(val, false.into());
    }

    #[test]
    fn bit_and() {
        let (_input, res) = math_expr("0b1010 & 0b0110").unwrap();
        //println!("{res:?}");
        let mut graph = Graph::default();
        let val = Runtime::eval(&mut graph, res).unwrap();
        //println!("{val:?}");
        assert_eq!(val, 0b0010.into());
    }

    #[test]
    fn bit_or() {
        let (_input, res) = math_expr("0b1010 | 0b0110").unwrap();
        //println!("{res:?}");
        let mut graph = Graph::default();
        let val = Runtime::eval(&mut graph, res).unwrap();
        //println!("{val:?}");
        assert_eq!(val, 0b1110.into());
    }

    #[test]
    fn bit_xor() {
        let (_input, res) = math_expr("0b1010 ^ 0b0110").unwrap();
        //println!("{res:?}");
        let mut graph = Graph::default();
        let val = Runtime::eval(&mut graph, res).unwrap();
        //println!("{val:?}");
        assert_eq!(val, 0b1100.into());
    }

    #[test]
    fn bit_shl() {
        let (_input, res) = math_expr("0b0011 << 2").unwrap();
        //println!("{res:?}");
        let mut graph = Graph::default();
        let val = Runtime::eval(&mut graph, res).unwrap();
        //println!("{val:?}");
        assert_eq!(val, 0b1100.into());
    }

    #[test]
    fn bit_shr() {
        let (_input, res) = math_expr("0b1100 >> 3").unwrap();
        //println!("{res:?}");
        let mut graph = Graph::default();
        let val = Runtime::eval(&mut graph, res).unwrap();
        //println!("{val:?}");
        assert_eq!(val, 0b0001.into());
    }

    #[test]
    fn logical_and() {
        let (_input, res) = math_expr("5 && !false").unwrap();
        //println!("{res:?}");
        let mut graph = Graph::default();
        let val = Runtime::eval(&mut graph, res).unwrap();
        //println!("{val:?}");
        assert_eq!(val, true.into());
    }

    #[test]
    fn logical_or() {
        let (_input, res) = math_expr("5 || !!(false)").unwrap();
        //println!("{res:?}");
        let mut graph = Graph::default();
        let val = Runtime::eval(&mut graph, res).unwrap();
        //println!("{val:?}");
        assert_eq!(val, true.into());
    }
}
