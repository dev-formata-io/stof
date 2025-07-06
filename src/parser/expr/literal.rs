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
use nom::{character::complete::char, combinator::{map, opt}, multi::separated_list1, sequence::preceded, IResult, Parser};
use crate::{parser::{expr::graph::chained_var_func, literal::literal}, runtime::{instruction::Instruction, instructions::{block::Block, Base}}};


/// Parse a literal expr (instruction).
/// Pushes a value onto the stack if found.
pub fn literal_expr(input: &str) -> IResult<&str, Arc<dyn Instruction>> {
    let (input, ins) = map(literal, |val| Arc::new(Base::Literal(val)) as Arc<dyn Instruction>).parse(input)?;
    
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
