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
use arcstr::ArcStr;
use serde::{Deserialize, Serialize};
use crate::runtime::instructions::unique_tag;
use crate::{model::Graph, runtime::{instruction::{Instruction, Instructions}, instructions::{Base, ConsumeStack, TRUTHY, BINARY_ADD, BINARY_BIT_AND, BINARY_BIT_OR, BINARY_BIT_SHIFT_LEFT, BINARY_BIT_SHIFT_RIGHT, BINARY_BIT_XOR, BINARY_DIVIDE, BINARY_EQUAL, BINARY_GREATER_THAN, BINARY_GREATER_THAN_OR_EQ, BINARY_LESS_THAN, BINARY_LESS_THAN_OR_EQ, BINARY_MODULUS, BINARY_MULTIPLY, BINARY_NOT_EQUAL, BINARY_SUBTRACT}, proc::ProcEnv, Error}};


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Ops.
pub enum Op {
    And,
    Or,

    Add,
    Sub,
    Mul,
    Div,
    Mod,

    Greater,
    Less,
    GreaterOrEq,
    LessOrEq,
    Eq,
    Neq,
    
    BAND,
    BOR,
    BXOR,
    BSHL,
    BSHR,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
/// Value operations.
pub struct OpIns {
    pub lhs: Arc<dyn Instruction>,
    pub rhs: Arc<dyn Instruction>,
    pub op: Op,
}
#[typetag::serde(name = "OpIns")]
impl Instruction for OpIns {
    fn exec(&self, _env: &mut ProcEnv, _graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        let mut instructions = Instructions::default();
        match self.op {
            Op::And => {
                instructions.push(self.lhs.clone());

                let end_tag: ArcStr = unique_tag();
                instructions.push(TRUTHY.clone()); // put truthy onto stack
                instructions.push(Arc::new(Base::CtrlForwardToIfNotTruthy(end_tag.clone(), ConsumeStack::IfTrue)));
                instructions.push(self.rhs.clone());
                instructions.push(TRUTHY.clone());
                instructions.push(Arc::new(Base::Tag(end_tag)));
            },
            Op::Or => {
                instructions.push(self.lhs.clone());

                let end_tag: ArcStr = unique_tag();
                instructions.push(TRUTHY.clone()); // put truthy onto stack
                instructions.push(Arc::new(Base::CtrlForwardToIfTruthy(end_tag.clone(), ConsumeStack::IfTrue)));
                instructions.push(self.rhs.clone());
                instructions.push(TRUTHY.clone());
                instructions.push(Arc::new(Base::Tag(end_tag)));
            },
            _ => {
                // left to right, so the right value ends up on top of the stack
                instructions.push(self.lhs.clone());
                instructions.push(self.rhs.clone());
                if let Some(binary) = self.op.binary_instruction() {
                    instructions.push(binary);
                }
            },
        }
        Ok(Some(instructions))
    }
}

impl Op {
    /// The error when an op's values are missing from the stack (the same as the original op instructions).
    pub(crate) fn stack_error(&self) -> Error {
        match self {
            Op::Add => Error::Add,
            Op::Sub => Error::Sub,
            Op::Mul => Error::Mul,
            Op::Div => Error::Div,
            Op::Mod => Error::Mod,
            Op::BAND => Error::AND,
            Op::BOR => Error::OR,
            Op::BXOR => Error::XOR,
            Op::BSHL => Error::SHL,
            Op::BSHR => Error::SHR,
            Op::Greater => Error::GreaterThan,
            Op::GreaterOrEq => Error::GreaterOrEq,
            Op::Less => Error::LessThan,
            Op::LessOrEq => Error::LessOrEq,
            Op::Eq | Op::Neq => Error::Eq,
            Op::And | Op::Or => Error::StackError,
        }
    }

    /// The op instruction for values evaluated left to right (the right value on top of the stack).
    pub(crate) fn binary_instruction(&self) -> Option<Arc<dyn Instruction>> {
        Some(match self {
            Op::Add => BINARY_ADD.clone(),
            Op::Sub => BINARY_SUBTRACT.clone(),
            Op::Mul => BINARY_MULTIPLY.clone(),
            Op::Div => BINARY_DIVIDE.clone(),
            Op::Mod => BINARY_MODULUS.clone(),
            Op::BAND => BINARY_BIT_AND.clone(),
            Op::BOR => BINARY_BIT_OR.clone(),
            Op::BXOR => BINARY_BIT_XOR.clone(),
            Op::BSHL => BINARY_BIT_SHIFT_LEFT.clone(),
            Op::BSHR => BINARY_BIT_SHIFT_RIGHT.clone(),
            Op::Less => BINARY_LESS_THAN.clone(),
            Op::LessOrEq => BINARY_LESS_THAN_OR_EQ.clone(),
            Op::Greater => BINARY_GREATER_THAN.clone(),
            Op::GreaterOrEq => BINARY_GREATER_THAN_OR_EQ.clone(),
            Op::Eq => BINARY_EQUAL.clone(),
            Op::Neq => BINARY_NOT_EQUAL.clone(),
            Op::And | Op::Or => return None,
        })
    }
}


#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use crate::{model::Graph, runtime::{instructions::{ops::{Op, OpIns}, Base}, Num, Runtime, Val}};

    #[test]
    fn and_op_true() {
        let ins = OpIns {
            lhs: Arc::new(Base::Literal(Val::Bool(true))),
            rhs: Arc::new(Base::Literal(Val::Num(Num::Int(2)))),
            op: Op::And,
        };
        let mut graph = Graph::default();
        let res = Runtime::eval(&mut graph, Arc::new(ins)).expect("expected pass");
        assert_eq!(res, true.into());
    }

    #[test]
    fn and_op_false_first() {
        let ins = OpIns {
            lhs: Arc::new(Base::Literal(Val::Bool(false))),
            rhs: Arc::new(Base::Literal(Val::Num(Num::Int(2)))),
            op: Op::And,
        };
        let mut graph = Graph::default();
        let res = Runtime::eval(&mut graph, Arc::new(ins)).expect("expected pass");
        assert_eq!(res, false.into());
    }

    #[test]
    fn and_op_false_second() {
        let ins = OpIns {
            lhs: Arc::new(Base::Literal(Val::Bool(true))),
            rhs: Arc::new(Base::Literal(Val::Num(Num::Int(0)))),
            op: Op::And,
        };
        let mut graph = Graph::default();
        let res = Runtime::eval(&mut graph, Arc::new(ins)).expect("expected pass");
        assert_eq!(res, false.into());
    }

    #[test]
    fn or_op_true_first() {
        let ins = OpIns {
            lhs: Arc::new(Base::Literal(Val::Bool(true))),
            rhs: Arc::new(Base::Literal(Val::Bool(false))),
            op: Op::Or,
        };
        let mut graph = Graph::default();
        let res = Runtime::eval(&mut graph, Arc::new(ins)).expect("expected pass");
        assert_eq!(res, true.into());
    }

    #[test]
    fn or_op_true_second() {
        let ins = OpIns {
            lhs: Arc::new(Base::Literal(Val::Bool(false))),
            rhs: Arc::new(Base::Literal(Val::Bool(true))),
            op: Op::Or,
        };
        let mut graph = Graph::default();
        let res = Runtime::eval(&mut graph, Arc::new(ins)).expect("expected pass");
        assert_eq!(res, true.into());
    }

    #[test]
    fn or_op_false() {
        let ins = OpIns {
            lhs: Arc::new(Base::Literal(Val::Bool(false))),
            rhs: Arc::new(Base::Literal(Val::Bool(false))),
            op: Op::Or,
        };
        let mut graph = Graph::default();
        let res = Runtime::eval(&mut graph, Arc::new(ins)).expect("expected pass");
        assert_eq!(res, false.into());
    }

    #[test]
    fn add_op() {
        let ins = OpIns {
            lhs: Arc::new(Base::Literal(Val::Num(Num::Int(7)))),
            rhs: Arc::new(Base::Literal(Val::Num(Num::Int(2)))),
            op: Op::Add,
        };
        let mut graph = Graph::default();
        let res = Runtime::eval(&mut graph, Arc::new(ins)).expect("expected 9");
        assert_eq!(res, 9.into());
    }

    #[test]
    fn sub_op() {
        let ins = OpIns {
            lhs: Arc::new(Base::Literal(Val::Num(Num::Int(7)))),
            rhs: Arc::new(Base::Literal(Val::Num(Num::Int(2)))),
            op: Op::Sub,
        };
        let mut graph = Graph::default();
        let res = Runtime::eval(&mut graph, Arc::new(ins)).expect("expected 5");
        assert_eq!(res, 5.into());
    }

    #[test]
    fn mul_op() {
        let ins = OpIns {
            lhs: Arc::new(Base::Literal(Val::Num(Num::Int(7)))),
            rhs: Arc::new(Base::Literal(Val::Num(Num::Int(2)))),
            op: Op::Mul,
        };
        let mut graph = Graph::default();
        let res = Runtime::eval(&mut graph, Arc::new(ins)).expect("expected 14");
        assert_eq!(res, 14.into());
    }

    #[test]
    fn div_op() {
        let ins = OpIns {
            lhs: Arc::new(Base::Literal(Val::Num(Num::Int(8)))),
            rhs: Arc::new(Base::Literal(Val::Num(Num::Int(2)))),
            op: Op::Div,
        };
        let mut graph = Graph::default();
        let res = Runtime::eval(&mut graph, Arc::new(ins)).expect("expected 4");
        assert_eq!(res, 4.into());
    }

    #[test]
    fn rem_op() {
        let ins = OpIns {
            lhs: Arc::new(Base::Literal(Val::Num(Num::Int(7)))),
            rhs: Arc::new(Base::Literal(Val::Num(Num::Int(2)))),
            op: Op::Mod,
        };
        let mut graph = Graph::default();
        let res = Runtime::eval(&mut graph, Arc::new(ins)).expect("expected 1");
        assert_eq!(res, 1.into());
    }

    #[test]
    fn bit_and() {
        let ins = OpIns {
            lhs: Arc::new(Base::Literal(Val::Num(Num::Int(0b1010)))),
            rhs: Arc::new(Base::Literal(Val::Num(Num::Int(0b1000)))),
            op: Op::BAND,
        };
        let mut graph = Graph::default();
        let res = Runtime::eval(&mut graph, Arc::new(ins)).expect("expected pass");
        assert_eq!(res, 0b1000.into());
    }

    #[test]
    fn bit_or() {
        let ins = OpIns {
            lhs: Arc::new(Base::Literal(Val::Num(Num::Int(0b1010)))),
            rhs: Arc::new(Base::Literal(Val::Num(Num::Int(0b1000)))),
            op: Op::BOR,
        };
        let mut graph = Graph::default();
        let res = Runtime::eval(&mut graph, Arc::new(ins)).expect("expected pass");
        assert_eq!(res, 0b1010.into());
    }

    #[test]
    fn bit_xor() {
        let ins = OpIns {
            lhs: Arc::new(Base::Literal(Val::Num(Num::Int(0b1010)))),
            rhs: Arc::new(Base::Literal(Val::Num(Num::Int(0b1000)))),
            op: Op::BXOR,
        };
        let mut graph = Graph::default();
        let res = Runtime::eval(&mut graph, Arc::new(ins)).expect("expected pass");
        assert_eq!(res, 0b0010.into());
    }

    #[test]
    fn bit_shift_left() {
        let ins = OpIns {
            lhs: Arc::new(Base::Literal(Val::Num(Num::Int(0b0011)))),
            rhs: Arc::new(Base::Literal(Val::Num(Num::Int(2)))),
            op: Op::BSHL,
        };
        let mut graph = Graph::default();
        let res = Runtime::eval(&mut graph, Arc::new(ins)).expect("expected pass");
        assert_eq!(res, 0b1100.into());
    }

    #[test]
    fn bit_shift_right() {
        let ins = OpIns {
            lhs: Arc::new(Base::Literal(Val::Num(Num::Int(0b1100)))),
            rhs: Arc::new(Base::Literal(Val::Num(Num::Int(3)))),
            op: Op::BSHR,
        };
        let mut graph = Graph::default();
        let res = Runtime::eval(&mut graph, Arc::new(ins)).expect("expected pass");
        assert_eq!(res, 0b0001.into());
    }

    #[test]
    fn less_than_op_true() {
        let ins = OpIns {
            lhs: Arc::new(Base::Literal(Val::Num(Num::Int(2)))),
            rhs: Arc::new(Base::Literal(Val::Num(Num::Int(4)))),
            op: Op::Less,
        };
        let mut graph = Graph::default();
        let res = Runtime::eval(&mut graph, Arc::new(ins)).expect("expected pass");
        assert_eq!(res, true.into());
    }

    #[test]
    fn less_than_op_false() {
        let ins = OpIns {
            lhs: Arc::new(Base::Literal(Val::Num(Num::Int(2)))),
            rhs: Arc::new(Base::Literal(Val::Num(Num::Int(2)))),
            op: Op::Less,
        };
        let mut graph = Graph::default();
        let res = Runtime::eval(&mut graph, Arc::new(ins)).expect("expected pass");
        assert_eq!(res, false.into());
    }

    #[test]
    fn less_than_eq_op_true() {
        let ins = OpIns {
            lhs: Arc::new(Base::Literal(Val::Num(Num::Int(2)))),
            rhs: Arc::new(Base::Literal(Val::Num(Num::Int(2)))),
            op: Op::LessOrEq,
        };
        let mut graph = Graph::default();
        let res = Runtime::eval(&mut graph, Arc::new(ins)).expect("expected pass");
        assert_eq!(res, true.into());
    }

    #[test]
    fn greater_than_op_true() {
        let ins = OpIns {
            lhs: Arc::new(Base::Literal(Val::Num(Num::Int(3)))),
            rhs: Arc::new(Base::Literal(Val::Num(Num::Int(2)))),
            op: Op::Greater,
        };
        let mut graph = Graph::default();
        let res = Runtime::eval(&mut graph, Arc::new(ins)).expect("expected pass");
        assert_eq!(res, true.into());
    }

    #[test]
    fn greater_than_op_false() {
        let ins = OpIns {
            lhs: Arc::new(Base::Literal(Val::Num(Num::Int(2)))),
            rhs: Arc::new(Base::Literal(Val::Num(Num::Int(2)))),
            op: Op::Greater,
        };
        let mut graph = Graph::default();
        let res = Runtime::eval(&mut graph, Arc::new(ins)).expect("expected pass");
        assert_eq!(res, false.into());
    }

    #[test]
    fn greater_than_eq_op_true() {
        let ins = OpIns {
            lhs: Arc::new(Base::Literal(Val::Num(Num::Int(2)))),
            rhs: Arc::new(Base::Literal(Val::Num(Num::Int(2)))),
            op: Op::GreaterOrEq,
        };
        let mut graph = Graph::default();
        let res = Runtime::eval(&mut graph, Arc::new(ins)).expect("expected pass");
        assert_eq!(res, true.into());
    }

    #[test]
    fn eq_op_true() {
        let ins = OpIns {
            lhs: Arc::new(Base::Literal(Val::Num(Num::Int(4)))),
            rhs: Arc::new(Base::Literal(Val::Num(Num::Int(4)))),
            op: Op::Eq,
        };
        let mut graph = Graph::default();
        let res = Runtime::eval(&mut graph, Arc::new(ins)).expect("expected pass");
        assert_eq!(res, true.into());
    }

    #[test]
    fn eq_op_false() {
        let ins = OpIns {
            lhs: Arc::new(Base::Literal(Val::Num(Num::Int(2)))),
            rhs: Arc::new(Base::Literal(Val::Num(Num::Int(4)))),
            op: Op::Eq,
        };
        let mut graph = Graph::default();
        let res = Runtime::eval(&mut graph, Arc::new(ins)).expect("expected pass");
        assert_eq!(res, false.into());
    }

    #[test]
    fn neq_op_false() {
        let ins = OpIns {
            lhs: Arc::new(Base::Literal(Val::Num(Num::Int(4)))),
            rhs: Arc::new(Base::Literal(Val::Num(Num::Int(4)))),
            op: Op::Neq,
        };
        let mut graph = Graph::default();
        let res = Runtime::eval(&mut graph, Arc::new(ins)).expect("expected pass");
        assert_eq!(res, false.into());
    }

    #[test]
    fn neq_op_true() {
        let ins = OpIns {
            lhs: Arc::new(Base::Literal(Val::Num(Num::Int(2)))),
            rhs: Arc::new(Base::Literal(Val::Num(Num::Int(4)))),
            op: Op::Neq,
        };
        let mut graph = Graph::default();
        let res = Runtime::eval(&mut graph, Arc::new(ins)).expect("expected pass");
        assert_eq!(res, true.into());
    }
}
