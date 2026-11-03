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
use arcstr::literal;
use crate::{model::{stof_std::{ASSERT, ASSERT_EQ, ASSERT_NEQ, ASSERT_NOT, ASSERT_NOT_NULL, ASSERT_NULL, STD_LIB, THROW}, LibFunc, Param}, runtime::{instruction::Instructions, instructions::Base, Type, Val}};


/// Throw error function.
pub fn throw() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "throw".into(),
        is_async: false,
        docs: r#"# Std.throw(value: unknown = "Error") -> void
Throw an error with an optional value. Optionally catch this value within a try-catch block. Otherwise, this process will immediately hault executing with the given error.
```rust
throw("error message");
```
"#.into(),
        params: crate::params![
            Param { name: "value".into(), param_type: Type::Void, default: Some(Arc::new(Base::Literal(Val::Str(literal!("Error"))))) }
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
        docs: r#"# Std.assert(value: unknown = false, message: str = null) -> void
Throw an error if the given value is not truthy. An optional message comes first in the error.
```rust
assert(true);
assert(false); // errors
assert(self.items.len() > 0, 'items should not be empty'); // errors with the message
```
"#.into(),
        params: crate::params![
            Param { name: "value".into(), param_type: Type::Void, default: Some(Arc::new(Base::Literal(Val::Bool(false)))) },
            Param { name: "message".into(), param_type: Type::Void, default: Some(Arc::new(Base::Literal(Val::Null))) }
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
        docs: r#"# Std.assert_not(value: unknown = true, message: str = null) -> void
Throw an error if the given value is truthy. An optional message comes first in the error.
```rust
assert_not(false);
assert_not(true); // errors
assert_not(self.deleted, 'the record was deleted'); // errors with the message
```
"#.into(),
        params: crate::params![
            Param { name: "value".into(), param_type: Type::Void, default: Some(Arc::new(Base::Literal(Val::Bool(true)))) },
            Param { name: "message".into(), param_type: Type::Void, default: Some(Arc::new(Base::Literal(Val::Null))) }
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
        docs: r#"# Std.assert_eq(first: unknown, second: unknown, message: str = null) -> void
Throw an error if the first value does not equal the second. An optional message comes first in the error.
```rust
assert_eq('a', 'a');
assert_eq(43, 42); // errors
assert_eq(total, 42, 'total after the refund'); // errors: "total after the refund: '43' does not equal '42'"
```
"#.into(),
        params: crate::params![
            Param { name: "first".into(), param_type: Type::Void, default: None },
            Param { name: "second".into(), param_type: Type::Void, default: None },
            Param { name: "message".into(), param_type: Type::Void, default: Some(Arc::new(Base::Literal(Val::Null))) }
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
        docs: r#"# Std.assert_neq(first: unknown, second: unknown, message: str = null) -> void
Throw an error if the first value equals the second. An optional message comes first in the error.
```rust
assert_neq('a', 'b');
assert_neq(34, 34); // errors
assert_neq(new_id, old_id, 'a copy gets a new id'); // errors with the message
```
"#.into(),
        params: crate::params![
            Param { name: "first".into(), param_type: Type::Void, default: None },
            Param { name: "second".into(), param_type: Type::Void, default: None },
            Param { name: "message".into(), param_type: Type::Void, default: Some(Arc::new(Base::Literal(Val::Null))) }
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


/// Standard assert null function.
pub fn assert_null() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "assert_null".into(),
        is_async: false,
        docs: r#"# Std.assert_null(value: unknown, message: str = null) -> void
Throw an error if the given value is not null (void, the result of a function without a return value, counts as null). An optional message comes first in the error.
```rust
assert_null(null);
assert_null(42); // errors
```
"#.into(),
        params: crate::params![
            Param { name: "value".into(), param_type: Type::Void, default: None },
            Param { name: "message".into(), param_type: Type::Void, default: Some(Arc::new(Base::Literal(Val::Null))) }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(ASSERT_NULL.clone());
            Ok(instructions)
        })
    }
}


/// Standard assert not null function.
pub fn assert_not_null() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "assert_not_null".into(),
        is_async: false,
        docs: r#"# Std.assert_not_null(value: unknown, message: str = null) -> void
Throw an error if the given value is null (or void). An optional message comes first in the error.
```rust
assert_not_null(42);
assert_not_null(null); // errors
assert_not_null(self.customer, 'customer was not created'); // errors with the message
```
"#.into(),
        params: crate::params![
            Param { name: "value".into(), param_type: Type::Void, default: None },
            Param { name: "message".into(), param_type: Type::Void, default: Some(Arc::new(Base::Literal(Val::Null))) }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(ASSERT_NOT_NULL.clone());
            Ok(instructions)
        })
    }
}
