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
use imbl::vector;
use crate::{model::{num::{AT, NUM_LIB}, LibFunc, Param}, runtime::{instruction::Instructions, NumT, Type}};


/// Len function.
pub fn num_len() -> LibFunc {
    LibFunc {
        library: NUM_LIB.clone(),
        name: "len".into(),
        is_async: false,
        docs: r#"# Num.len(val: int | float) -> int
Length of this number (helpful for iteration).
```rust
assert_eq((10).len(), 10);
```
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: Some(Type::Num(NumT::Int)),
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            // The argument is the length! Will be cast to an integer though.
            Ok(Instructions::default())
        })
    }
}


/// At function.
pub fn num_at() -> LibFunc {
    LibFunc {
        library: NUM_LIB.clone(),
        name: "at".into(),
        is_async: false,
        docs: r#"# Num.at(val: int | float, index: int) -> int
Index into this number (helpful for iteration of single value ranges).
```rust
assert_eq((10).at(5), 5);
assert_eq((10).at(20), 10);
```
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None },
            Param { name: "index".into(), param_type: Type::Num(NumT::Int), default: None }
        ],
        return_type: Some(Type::Num(NumT::Int)),
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(AT.clone());
            Ok(instructions)
        })
    }
}
