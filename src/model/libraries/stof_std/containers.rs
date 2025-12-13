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
use crate::{model::{stof_std::{StdIns, COPY, FUNCTIONS, STD_LIB, SWAP}, LibFunc, Param}, runtime::{instruction::Instructions, instructions::Base, Type, Val}};


/// List constructor function.
pub fn std_list() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "list".into(),
        is_async: false,
        docs: r#"# Std.list(..) -> list
Construct a new list with the given arguments.
```rust
assert_eq(list(1, 2, 3), [1, 2, 3]);
```
"#.into(),
        params: vector![],
        return_type: None,
        unbounded_args: true,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(StdIns::List(arg_count)));
            Ok(instructions)
        })
    }
}

/// Set constructor function.
pub fn std_set() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "set".into(),
        is_async: false,
        docs: r#"# Std.set(..) -> set
Construct a new set with the given arguments.
```rust
assert_eq(set(1, 2, 3), {1, 2, 3});
```
"#.into(),
        params: vector![],
        return_type: None,
        unbounded_args: true,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(StdIns::Set(arg_count)));
            Ok(instructions)
        })
    }
}

/// Map constructor function.
pub fn std_map() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "map".into(),
        is_async: false,
        docs: r#"# Std.map(..) -> map
Construct a new map with the given arguments (tuples of key & value). Helpful as a way to create an empty map.
```rust
assert_eq(map(("a", 1), ("b", 2)), {"a": 1, "b": 2});
```
"#.into(),
        params: vector![],
        return_type: None,
        unbounded_args: true,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(StdIns::Map(arg_count)));
            Ok(instructions)
        })
    }
}

/// Copy.
pub fn std_copy() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "copy".into(),
        is_async: false,
        docs: r#"# Std.copy(val: unknown) -> unknown
Deep copy the given value. If this value is an object (or contains one), recursively deep copy the object (all fields, funcs, & data).
```rust
const a = {1, 2, 3};
const b = copy(a);
b.clear();
assert_neq(a, b);
```
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(COPY.clone());
            Ok(instructions)
        })
    }
}

/// Swap.
pub fn std_swap() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "swap".into(),
        is_async: false,
        docs: r#"# Std.swap(first: unknown, second: unknown) -> void
Swap the memory addresses of any two values.
```rust
const a = 42;
const b = -55;
swap(&a, &b); // '&' because int is a value type (not automatically a reference)
assert_eq(a, -55);
assert_eq(b, 42);
```
"#.into(),
        params: vector![
            Param { name: "first".into(), param_type: Type::Void, default: None },
            Param { name: "second".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(SWAP.clone());
            Ok(instructions)
        })
    }
}

/// Drop.
pub fn std_drop() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "drop".into(),
        is_async: false,
        docs: r#"# Std.drop(..) -> bool | list
Drop fields (by str path), functions (path or fn), objects (path or obj), and data from the graph. Objects will have their #[dropped] functions called when dropped. When dropping multiple values at once, this will return a list of booleans indicating a successful removal or not for each value.
```rust
const func = () => {};
const object = new {};
const results = drop("self.field", func, object);
assert_eq(results, [true, true, true]);
```
"#.into(),
        params: vector![],
        return_type: None,
        unbounded_args: true,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(StdIns::ObjDropped(arg_count)));
            instructions.push(Arc::new(StdIns::Drop(arg_count)));
            Ok(instructions)
        })
    }
}

/// Functions.
pub fn std_funcs() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "funcs".into(),
        is_async: false,
        docs: r#"# Std.funcs(attributes: str | list | set = null) -> list
Get a list of all functions in this graph, optionally filtering by attributes (single string, list of strings, set of strings, or tuple of strings).
```rust
for (const func in funcs({"test", "main"})) {
    // all test and main functions in the graph
    // call them or whatever you need
}
```
"#.into(),
        params: vector![
            Param { name: "attributes".into(), param_type: Type::Void, default: Some(Arc::new(Base::Literal(Val::Null))) },
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(FUNCTIONS.clone());
            Ok(instructions)
        })
    }
}

/// Shallow drop.
pub fn std_shallow_drop() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "shallow_drop".into(),
        is_async: false,
        docs: r#"# Std.shallow_drop(..) -> bool | list
Operates the same way Std.drop(..) does, however, if dropping a field and the field points to an object or data, only remove the field and not the associated object/data. This is used instead of drop in instances where multiple fields might point to the same object and you'd like to remove the field without removing the object.
```rust
const object = self.field; // field is an obj value
assert(shallow_drop("self.field"));
assert_not(self.field); // note: this will still work if the objects name is "field"
assert(object.exists()); // object was kept around
```
"#.into(),
        params: vector![],
        return_type: None,
        unbounded_args: true,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(StdIns::ShallowDrop(arg_count)));
            Ok(instructions)
        })
    }
}
