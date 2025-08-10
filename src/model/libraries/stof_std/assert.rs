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
use arcstr::literal;
use imbl::vector;
use crate::{model::{stof_std::{ASSERT, ASSERT_EQ, ASSERT_NEQ, ASSERT_NOT, STD_LIB, THROW}, LibFunc, Param}, runtime::{instruction::Instructions, instructions::Base, Type, Val}};


/// Throw error function.
pub fn throw() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "throw".into(),
        is_async: false,
        docs: "# Throw an error\nUsed to force an error anywhere inside Stof.".into(),
        params: vector![
            Param { name: "value".into(), param_type: Type::Void, default: Some(Arc::new(Base::Literal(Val::Str(literal!("Std.throw()"))))) }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(THROW.clone());
            Ok(instructions)
        })
    }
}


/// Standard assert function.
pub fn assert() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "assert".into(),
        is_async: false,
        docs: "# Make an assertion\nUsed in testing and to assert truthiness.".into(),
        params: vector![
            Param { name: "value".into(), param_type: Type::Void, default: Some(Arc::new(Base::Literal(Val::Bool(false)))) }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(ASSERT.clone());
            Ok(instructions)
        })
    }
}


/// Standard assert not function.
pub fn assert_not() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "assert_not".into(),
        is_async: false,
        docs: "# Make a Falsy assertion\nUsed in testing and to assert that a value is falsy.".into(),
        params: vector![
            Param { name: "value".into(), param_type: Type::Void, default: Some(Arc::new(Base::Literal(Val::Bool(true)))) }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(ASSERT_NOT.clone());
            Ok(instructions)
        })
    }
}


/// Standard assert equal function.
pub fn assert_eq() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "assert_eq".into(),
        is_async: false,
        docs: "# Make a equal assertion\nUsed in testing and to assert that two values are equal.".into(),
        params: vector![
            Param { name: "first".into(), param_type: Type::Void, default: None },
            Param { name: "second".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(ASSERT_EQ.clone());
            Ok(instructions)
        })
    }
}


/// Standard assert not equal function.
pub fn assert_neq() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "assert_neq".into(),
        is_async: false,
        docs: "# Make a not equals assertion\nUsed in testing and to assert that two values are not equal.".into(),
        params: vector![
            Param { name: "first".into(), param_type: Type::Void, default: None },
            Param { name: "second".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(ASSERT_NEQ.clone());
            Ok(instructions)
        })
    }
}
