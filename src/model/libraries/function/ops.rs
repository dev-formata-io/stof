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
use crate::{model::{LibFunc, Param, function::BIND, libraries::function::{ATTRIBUTES, DATA, FUNC_LIB, FuncIns, HAS_ATTR, ID, IS_ASYNC, NAME, OBJ, OBJS, PARAMS, RETURN_TYPE}}, runtime::{Type, instruction::Instructions}};


/// Id.
pub fn fn_id() -> LibFunc {
    LibFunc {
        library: FUNC_LIB.clone(),
        name: "id".into(),
        is_async: false,
        docs: r#"# Fn.id(func: fn) -> str
Get the data ID for this function (shorthand for "func.data().id()").
```rust
const func: fn = self.hi;
assert_eq(func.id(), func.data().id());
```"#.into(),
        params: vector![
            Param { name: "fn".into(), param_type: Type::Fn, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(ID.clone());
            Ok(instructions)
        })
    }
}

/// Data.
pub fn fn_data() -> LibFunc {
    LibFunc {
        library: FUNC_LIB.clone(),
        name: "data".into(),
        is_async: false,
        docs: r#"# Fn.data(func: fn) -> data
Get the data pointer for this function.
```rust
const func: fn = self.hi;
assert(func.data().exists());
```"#.into(),
        params: vector![
            Param { name: "fn".into(), param_type: Type::Fn, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(DATA.clone());
            Ok(instructions)
        })
    }
}

/// Bind.
pub fn fn_bind() -> LibFunc {
    LibFunc {
        library: FUNC_LIB.clone(),
        name: "bind".into(),
        is_async: false,
        docs: r#"# Fn.bind(func: fn, to: obj) -> bool
Bind a function to an object. This will remove the object from the nodes that currently reference it and place it on the "to" object.
```rust
const func = ():str => self.msg ?? 'dne';

const to = new { msg: 'hi' };
func.bind(to);

assert_eq(func(), 'hi');
```"#.into(),
        params: vector![
            Param { name: "fn".into(), param_type: Type::Fn, default: None },
            Param { name: "to".into(), param_type: Type::Void, default: None },
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(BIND.clone());
            Ok(instructions)
        })
    }
}

/// Name.
pub fn fn_name() -> LibFunc {
    LibFunc {
        library: FUNC_LIB.clone(),
        name: "name".into(),
        is_async: false,
        docs: r#"# Fn.name(func: fn) -> str
Get the name of this function.
```rust
const func: fn = self.hi; // fn hi() {}
assert_eq(func.name(), "hi");
```"#.into(),
        params: vector![
            Param { name: "fn".into(), param_type: Type::Fn, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(NAME.clone());
            Ok(instructions)
        })
    }
}

/// Params.
pub fn fn_params() -> LibFunc {
    LibFunc {
        library: FUNC_LIB.clone(),
        name: "params".into(),
        is_async: false,
        docs: r#"# Fn.params(func: fn) -> list
Get a list of expected parameters for this function (tuple containing the name and type).
```rust
const func: fn = self.hi; // fn hi(a: int) {}
assert_eq(func.params(), [("a", "int")]);
```"#.into(),
        params: vector![
            Param { name: "fn".into(), param_type: Type::Fn, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(PARAMS.clone());
            Ok(instructions)
        })
    }
}

/// Return type.
pub fn fn_return_type() -> LibFunc {
    LibFunc {
        library: FUNC_LIB.clone(),
        name: "return_type".into(),
        is_async: false,
        docs: r#"# Fn.return_type(func: fn) -> str
Get the return type for the given function.
```rust
const func: fn = self.hi; // fn hi() -> int { 42 }
assert_eq(func.return_type(), "int");
```"#.into(),
        params: vector![
            Param { name: "fn".into(), param_type: Type::Fn, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(RETURN_TYPE.clone());
            Ok(instructions)
        })
    }
}

/// Has attribute?
pub fn fn_has_attr() -> LibFunc {
    LibFunc {
        library: FUNC_LIB.clone(),
        name: "has_attribute".into(),
        is_async: false,
        docs: r#"# Fn.has_attribute(func: fn, name: str) -> bool
Returns true if the given function has an attribute with the given name.
```rust
const func: fn = self.hi; // #[hi] fn hi() {}
assert(func.has_attribute("hi"));
```"#.into(),
        params: vector![
            Param { name: "fn".into(), param_type: Type::Fn, default: None },
            Param { name: "name".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(HAS_ATTR.clone());
            Ok(instructions)
        })
    }
}

/// Attributes.
pub fn fn_attributes() -> LibFunc {
    LibFunc {
        library: FUNC_LIB.clone(),
        name: "attributes".into(),
        is_async: false,
        docs: r#"# Fn.attributes(func: fn) -> map
Get a map of attributes (name & value) that this function has, if any.
```rust
const func: fn = self.hi; // #[hi] fn hi() {}
assert_eq(func.attributes(), {"hi": null});
```"#.into(),
        params: vector![
            Param { name: "fn".into(), param_type: Type::Fn, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(ATTRIBUTES.clone());
            Ok(instructions)
        })
    }
}

/// Obj.
pub fn fn_obj() -> LibFunc {
    LibFunc {
        library: FUNC_LIB.clone(),
        name: "obj".into(),
        is_async: false,
        docs: r#"# Fn.obj(func: fn) -> obj
Get the first object found that references this function.
```rust
const func: fn = self.hi;
assert_eq(func.obj(), self);
```"#.into(),
        params: vector![
            Param { name: "fn".into(), param_type: Type::Fn, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(OBJ.clone());
            Ok(instructions)
        })
    }
}

/// Objs.
pub fn fn_objs() -> LibFunc {
    LibFunc {
        library: FUNC_LIB.clone(),
        name: "objs".into(),
        is_async: false,
        docs: r#"# Fn.objs(func: fn) -> list
Get a list of all objects that this function is attached to.
```rust
const func: fn = self.hi;
assert_eq(func.objs(), [self]);
```"#.into(),
        params: vector![
            Param { name: "fn".into(), param_type: Type::Fn, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(OBJS.clone());
            Ok(instructions)
        })
    }
}

/// Is Async?
pub fn fn_is_async() -> LibFunc {
    LibFunc {
        library: FUNC_LIB.clone(),
        name: "is_async".into(),
        is_async: false,
        docs: r#"# Fn.is_async(func: fn) -> bool
Is this function async? This is just shorthand for checking if an "async" attribute exists (what makes a func async).
```rust
const func: fn = self.hi; // async fn hi() {}
assert(func.is_async());
```"#.into(),
        params: vector![
            Param { name: "fn".into(), param_type: Type::Fn, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(IS_ASYNC.clone());
            Ok(instructions)
        })
    }
}

/// Call.
pub fn fn_call() -> LibFunc {
    LibFunc {
        library: FUNC_LIB.clone(),
        name: "call".into(),
        is_async: false,
        docs: r#"# Fn.call(func: fn, ..) -> unknown
Call this function, using any arguments given after the function itself (some library functions can take N arguments, this is one of them).
```rust
const func: fn = (name: str):str => "Hi, " + name;
assert_eq(func.call("Bob"), "Hi, Bob");
```"#.into(),
        params: vector![
            Param { name: "fn".into(), param_type: Type::Fn, default: None }
            // Unbounded parameters after the first function reference
        ],
        return_type: None,
        unbounded_args: true,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(FuncIns::Call(arg_count)));
            Ok(instructions)
        })
    }
}

/// Expanded Call.
pub fn fn_exp_call() -> LibFunc {
    LibFunc {
        library: FUNC_LIB.clone(),
        name: "call_expanded".into(),
        is_async: false,
        docs: r#"# Fn.call_expanded(func: fn, ..) -> unknown
Call this function, using any arguments given after the function itself. However, if an argument is a collection (ex. list), expand the list values out as arguments themselves.
```rust
const func: fn = (name: str):str => "Hi, " + name;
assert_eq(func.call_expanded(["Bob"]), "Hi, Bob");
```"#.into(),
        params: vector![
            Param { name: "fn".into(), param_type: Type::Fn, default: None }
            // Unbounded parameters after the first function reference
        ],
        return_type: None,
        unbounded_args: true,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(FuncIns::ExpandCall(arg_count)));
            Ok(instructions)
        })
    }
}
