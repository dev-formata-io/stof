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
use crate::{model::{stof_std::{StdIns, STD_LIB, XMLTAG}, LibFunc, Param}, runtime::{instruction::Instructions, Type}};


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

/// Create a string that has XML tags.
pub fn xmltag() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "xml".into(),
        is_async: false,
        docs: r#"# Std.xml(text: str, tag: str) -> str
A helper function to create an XML-tagged string.
```rust
assert_eq(xml("hello, world", "msg"), "<msg>hello, world</msg>");
```
"#.into(),
        params: vector![
            Param { name: "text".into(), param_type: Type::Str, default: None },
            Param { name: "tag".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(XMLTAG.clone());
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

#[cfg(feature = "log")]
/// Log error function.
pub fn std_log_error() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "log_error".into(),
        is_async: false,
        docs: r#"# Std.log_error(..) -> void
Logs all arguments as an error using the "log" crate.
```rust
log_error("we have a problem");
```
"#.into(),
        params: vector![],
        return_type: None,
        unbounded_args: true,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(StdIns::ErrorLog(arg_count)));
            Ok(instructions)
        })
    }
}

#[cfg(feature = "log")]
/// Log warn function.
pub fn std_log_warn() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "log_warn".into(),
        is_async: false,
        docs: r#"# Std.log_warn(..) -> void
Logs all arguments as a warnging using the "log" crate.
```rust
log_warn("we encountered something, but are handling it");
```
"#.into(),
        params: vector![],
        return_type: None,
        unbounded_args: true,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(StdIns::WarnLog(arg_count)));
            Ok(instructions)
        })
    }
}

#[cfg(feature = "log")]
/// Log info function.
pub fn std_log_info() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "log_info".into(),
        is_async: false,
        docs: r#"# Std.log_info(..) -> void
Logs all arguments as info using the "log" crate.
```rust
log_info("we just did something cool");
```
"#.into(),
        params: vector![],
        return_type: None,
        unbounded_args: true,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(StdIns::InfoLog(arg_count)));
            Ok(instructions)
        })
    }
}

#[cfg(feature = "log")]
/// Log debug function.
pub fn std_log_debug() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "log_debug".into(),
        is_async: false,
        docs: r#"# Std.log_debug(..) -> void
Logs all arguments as debug info using the "log" crate.
```rust
log_debug("this is what just happened, in case you need to debug me");
```
"#.into(),
        params: vector![],
        return_type: None,
        unbounded_args: true,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(StdIns::DebugLog(arg_count)));
            Ok(instructions)
        })
    }
}

#[cfg(feature = "log")]
/// Log trace function.
pub fn std_log_trace() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "log_trace".into(),
        is_async: false,
        docs: r#"# Std.log_trace(..) -> void
Logs all arguments as a trace using the "log" crate.
```rust
log_trace("we have a problem");
```
"#.into(),
        params: vector![],
        return_type: None,
        unbounded_args: true,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(StdIns::TraceLog(arg_count)));
            Ok(instructions)
        })
    }
}
