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
use imbl::vector;
use crate::{model::{stof_std::{StdIns, STD_LIB}, LibFunc}, runtime::{instruction::Instructions}};


/// Standard printline function.
pub fn pln() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "pln".into(),
        is_async: false,
        docs: r#"# Std.pln(..) -> void
Prints all arguments to the standard output stream.
```rust
pln("hello, world");
```
"#.into(),
        params: vector![],
        return_type: None,
        unbounded_args: true, // allow an unbounded number of arguments
        args_to_symbol_table: false, // keep the arg on the stack instead of putting it into st
        func: Arc::new(|_as_ref, arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(StdIns::Pln(arg_count)));
            Ok(instructions)
        })
    }
}

/// Standard printline function (to string variant).
pub fn string() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "str".into(),
        is_async: false,
        docs: r#"# Std.str(..) -> str
Prints all arguments to a string, just like it would be to an output stream.
```rust
assert_eq(str("hello, world"), "hello, world");
```
"#.into(),
        params: vector![],
        return_type: None,
        unbounded_args: true, // allow an unbounded number of arguments
        args_to_symbol_table: false, // keep the arg on the stack instead of putting it into st
        func: Arc::new(|_as_ref, arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(StdIns::String(arg_count)));
            Ok(instructions)
        })
    }
}

/// Standard debug print function.
pub fn dbg() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "dbg".into(),
        is_async: false,
        docs: r#"# Std.dbg(..) -> void
Prints all arguments as debug output to the standard output stream.
```rust
dbg("hello, world");
```
"#.into(),
        params: vector![],
        return_type: None,
        unbounded_args: true, // allow an unbounded number of arguments
        args_to_symbol_table: false, // keep the arg on the stack instead of putting it into st
        func: Arc::new(|_as_ref, arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(StdIns::Dbg(arg_count)));
            Ok(instructions)
        })
    }
}

/// Standard printline function to error stream.
pub fn err() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "err".into(),
        is_async: false,
        docs: r#"# Std.err(..) -> void
Prints all arguments to the error output stream.
```rust
err("hello, world");
```
"#.into(),
        params: vector![],
        return_type: None,
        unbounded_args: true, // allow an unbounded number of arguments
        args_to_symbol_table: false, // keep the arg on the stack instead of putting it into st
        func: Arc::new(|_as_ref, arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(StdIns::Err(arg_count)));
            Ok(instructions)
        })
    }
}
