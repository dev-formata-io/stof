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
use crate::{model::{stof_std::{StdIns, BLOBIFY, CALLSTACK, FORMATS, FORMAT_CONTENT_TYPE, GRAPH_ID, HAS_FORMAT, HAS_LIB, LIBS, NANO_ID, PARSE, STD_LIB, STRINGIFY}, LibFunc, Param}, runtime::{instruction::Instructions, instructions::Base, Num, NumT, Type, Val}};


/// Parse.
pub fn std_parse() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "parse".into(),
        is_async: false,
        docs: r#"# Std.parse(source: str | blob, context: str | obj = self, format: str = "stof") -> bool
Parse data into this document/graph at the given location (default context is the calling object), using the given format (default is Stof). Formats are extensible and replaceable in Stof, so use whichever formats you have loaded (json, stof, images, pdfs, docx, etc.).
```rust
parse("fn hello() -> str { \"hello\" }");
assert_eq(self.hello(), "hello"); // can now call it
```
"#.into(),
        params: vector![
            Param { name: "source".into(), param_type: Type::Void, default: None, },
            Param { name: "context".into(), param_type: Type::Void, default: Some(Arc::new(Base::Literal(Val::Null))), },
            Param { name: "format".into(), param_type: Type::Str, default: Some(Arc::new(Base::Literal(Val::Null))), }, // default is stof
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(PARSE.clone());
            Ok(instructions)
        })
    }
}

/// Stringify.
pub fn std_stringify() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "stringify".into(),
        is_async: false,
        docs: r#"# Std.stringify(format: str = "json", context: obj = null) -> str
Use a loaded format to export a string from the given context (or entire graph/document). The default format is json, and the standard implementation only exports object fields. Export results will vary depending on the format, some support more than others (it is up to the format implementation to decide how it exports data). You can always create your own to use.
```rust
const object = new { x: 3.14km, y: 42m };
assert_eq(stringify("json", object), "{\"x\":3.14,\"y\":42}"); // lossy as json doesn't have a units concept
```
"#.into(),
        params: vector![
            Param { name: "format".into(), param_type: Type::Str, default: Some(Arc::new(Base::Literal(Val::Null))), },
            Param { name: "context".into(), param_type: Type::Void, default: Some(Arc::new(Base::Literal(Val::Null))), },
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(STRINGIFY.clone());
            Ok(instructions)
        })
    }
}

/// Blobify.
pub fn std_blobify() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "blobify".into(),
        is_async: false,
        docs: r#"# Std.blobify(format: str = "json", context: obj = null) -> blob
Use a loaded format to export a binary blob from the given context (or entire graph/document). The default format is json, and the standard implementation only exports object fields. Export results will vary depending on the format, some support more than others (it is up to the format implementation to decide how it exports data). You can always create your own to use.
```rust
const object = new { x: 3.14km, y: 42m };
const export = blobify("json", object); // json string like "stringify", but as a utf8 blob
assert(export.len() > 0);
```
"#.into(),
        params: vector![
            Param { name: "format".into(), param_type: Type::Str, default: Some(Arc::new(Base::Literal(Val::Null))), },
            Param { name: "context".into(), param_type: Type::Void, default: Some(Arc::new(Base::Literal(Val::Null))), },
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(BLOBIFY.clone());
            Ok(instructions)
        })
    }
}

/// Has format?
pub fn std_has_format() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "format".into(),
        is_async: false,
        docs: r#"# Std.format(format: str) -> bool
Is the given format loaded/available to use?
```rust
assert(format("json"));
assert_not(format("step"));
```
"#.into(),
        params: vector![
            Param { name: "format".into(), param_type: Type::Str, default: None, },
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(HAS_FORMAT.clone());
            Ok(instructions)
        })
    }
}

/// Formats.
pub fn std_formats() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "formats".into(),
        is_async: false,
        docs: r#"# Std.formats() -> set
A set of all available formats, available to use with parse, stringify, and blobify.
```rust
const loaded = formats();
assert(loaded.contains("json"));
```
"#.into(),
        params: vector![],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(FORMATS.clone());
            Ok(instructions)
        })
    }
}

/// Format content type.
pub fn std_format_content_type() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "format_content_type".into(),
        is_async: false,
        docs: r#"# Std.format_content_type(format: str) -> str
Returns the available format's content type (HTTP header value), or null if the format is not available. All formats are required to give a content type, even if it doesn't apply to that format.
```rust
assert_eq(format_content_type("json"), "application/json");
```
"#.into(),
        params: vector![
            Param { name: "format".into(), param_type: Type::Str, default: None, },
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(FORMAT_CONTENT_TYPE.clone());
            Ok(instructions)
        })
    }
}

/// Has lib?
pub fn std_has_lib() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "lib".into(),
        is_async: false,
        docs: r#"# Std.lib(lib: str) -> bool
Is the given library loaded/available to use?
```rust
assert(lib("Std")); // standard library is loaded
assert_not(lib("Render")); // no "Render" library loaded
```
"#.into(),
        params: vector![
            Param { name: "lib".into(), param_type: Type::Str, default: None, },
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(HAS_LIB.clone());
            Ok(instructions)
        })
    }
}

/// Libs.
pub fn std_libs() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "libs".into(),
        is_async: false,
        docs: r#"# Std.libs() -> set
Set of all available libraries. This will most likely include standard libraries like Std, Fn, Set, List, etc.
```rust
assert(libs().superset({"Std", "Fn", "Num", "Set"}));
```
"#.into(),
        params: vector![],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(LIBS.clone());
            Ok(instructions)
        })
    }
}

/// Nanoid
pub fn std_nanoid() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "nanoid".into(),
        is_async: false,
        docs: r#"# Std.nanoid(length: int = 21) -> str
Generate a URL safe random string ID, using the nanoid algorithm with a specified length (default is 21 characters). Probability of a collision is very low, and inversely proportional to ID length.
```rust
assert_neq(nanoid(), nanoid(33));
```
"#.into(),
        params: vector![
            Param { name: "length".into(), param_type: Type::Num(NumT::Int), default: Some(Arc::new(Base::Literal(Val::Num(Num::Int(21))))), },
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(NANO_ID.clone());
            Ok(instructions)
        })
    }
}

/// Graph ID.
pub fn std_graph_id() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "graph_id".into(),
        is_async: false,
        docs: r#"# Std.graph_id() -> str
Return this graph's unique string ID.
```rust
assert(graph_id().len() > 10);
```
"#.into(),
        params: vector![],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(GRAPH_ID.clone());
            Ok(instructions)
        })
    }
}

/// Max value library function.
pub fn std_max() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "max".into(),
        is_async: false,
        docs: r#"# Std.max(..) -> unknown
Return the maximum value of all given arguments. If an argument is a collection, the max value within the collection will be considered only.
```rust
assert_eq(max(1km, 2m, 3mm), 1km);
```
"#.into(),
        params: vector![],
        return_type: None,
        unbounded_args: true,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(StdIns::Max(arg_count)));
            Ok(instructions)
        })
    }
}


/// Min value library function.
pub fn std_min() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "min".into(),
        is_async: false,
        docs: r#"# Std.min(..) -> unknown
Return the minimum value of all given arguments. If an argument is a collection, the min value within the collection will be considered only.
```rust
assert_eq(min(1km, 2m, 3mm), 3mm);
```
"#.into(),
        params: vector![],
        return_type: None,
        unbounded_args: true,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(StdIns::Min(arg_count)));
            Ok(instructions)
        })
    }
}

/// Callstack.
pub fn std_callstack() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "callstack".into(),
        is_async: false,
        docs: r#"# Std.callstack() -> list
Return the current callstack as a list of function pointers (last function is 'this').
```rust
// inside a function call
for (const func in callstack()) {
    pln(func.obj().path(), ".", func.name());
}
```
"#.into(),
        params: vector![],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(CALLSTACK.clone());
            Ok(instructions)
        })
    }
}

/// Trace.
pub fn std_trace() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "trace".into(),
        is_async: false,
        docs: r#"# Std.trace(..) -> void
Trace this location within your code execution. Will print out your arguments plus process debug information and the current instruction stack. If the last argument given is an integer value, that number of executed instruction stack instructions will be shown (very helpful for deeper debugging).
```rust
trace("Getting here"); // will print "Getting here", then output a trace of the current process info and last 10 executed instructions
trace(70); // last 70 executed instructions (most recent on bottom and numbered)
```
"#.into(),
        params: vector![],
        return_type: None,
        unbounded_args: true,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(StdIns::Trace(arg_count)));
            Ok(instructions)
        })
    }
}

/// Peek.
pub fn std_peek() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "peek".into(),
        is_async: false,
        docs: r#"# Std.peek(..) -> void
Trace this location within your code execution. Will print out your arguments plus process debug information and the next instructions on the instruction stack. If the last argument given is an integer value, that number of (future) instructions will be shown (very helpful for deeper debugging).
```rust
peek("Getting here"); // will print "Getting here", then output a trace of the current process info and next 10 instructions to be executed
peek(70); // next 70 instructions
```
"#.into(),
        params: vector![],
        return_type: None,
        unbounded_args: true,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(StdIns::Peek(arg_count)));
            Ok(instructions)
        })
    }
}

/// Trace stack.
pub fn std_tracestack() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "dbg_tracestack".into(),
        is_async: false,
        docs: r#"# Std.dbg_tracestack() -> void
Print a snapshot of the current stack.
"#.into(),
        params: vector![],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(StdIns::TraceStack));
            Ok(instructions)
        })
    }
}
