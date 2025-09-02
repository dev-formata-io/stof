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
use crate::{model::{libraries::data::{ATTACH, DATA_LIB, DROP, DROP_FROM, EXISTS, FIELD, FROM_BLOB, FROM_ID, ID, INVALIDATE, MOVE, OBJS, TAGNAME, TO_BLOB, VALIDATE}, LibFunc, Param}, runtime::{instruction::Instructions, instructions::Base, Type, Val}};


/// Id.
pub fn data_id() -> LibFunc {
    LibFunc {
        library: DATA_LIB.clone(),
        name: "id".into(),
        is_async: false,
        docs: r#"# Data.id(ptr: data) -> str
Get the id for this data pointer, which can be used to later construct another reference.
```rust
const func: fn = self.hi;
const id = func.data().id();
assert_eq(Data.from_id(id), func.data());
```"#.into(),
        params: vector![
            Param { name: "data".into(), param_type: Type::Void, default: None }
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

/// Libname.
pub fn data_libname() -> LibFunc {
    LibFunc {
        library: DATA_LIB.clone(),
        name: "libname".into(),
        is_async: false,
        docs: r#"# Data.libname(ptr: data) -> str
Get the library name for this data pointer, if applicable.
```rust
const func: fn = self.hi;
assert_eq(func.data().libname(), "Fn");
```"#.into(),
        params: vector![
            Param { name: "data".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(TAGNAME.clone());
            Ok(instructions)
        })
    }
}

/// Exists?
pub fn data_exists() -> LibFunc {
    LibFunc {
        library: DATA_LIB.clone(),
        name: "exists".into(),
        is_async: false,
        docs: r#"# Data.exists(ptr: data) -> bool
Does this data pointer point to existing data? Will be false if the data has been dropped from the document.
```rust
const func: fn = self.hi;
const ptr = func.data();
drop(func);
assert_not(ptr.exists());
```"#.into(),
        params: vector![
            Param { name: "data".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(EXISTS.clone());
            Ok(instructions)
        })
    }
}

/// Invalidate.
pub fn data_invalidate() -> LibFunc {
    LibFunc {
        library: DATA_LIB.clone(),
        name: "invalidate".into(),
        is_async: false,
        docs: r#"# Data.invalidate(data: data, symbol: str = 'value') -> bool
Invalidate this data, optionally with the given symbol. Will throw an error if the data doesn't exist. Returns true if the data is newly invalidated with the given symbol.
```rust
const func: fn = self.hi;
const ptr = func.data();
assert(ptr.invalidate('something_happened')); // marks data as invalid
assert(ptr.validate('something_happened'));
assert_not(ptr.validate('something_happened')); // already validated
```"#.into(),
        params: vector![
            Param { name: "data".into(), param_type: Type::Void, default: None },
            Param { name: "symbol".into(), param_type: Type::Str, default: Some(Arc::new(Base::Literal(Val::Null))) }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(INVALIDATE.clone());
            Ok(instructions)
        })
    }
}

/// Validate.
pub fn data_validate() -> LibFunc {
    LibFunc {
        library: DATA_LIB.clone(),
        name: "validate".into(),
        is_async: false,
        docs: r#"# Data.validate(data: data, symbol?: str) -> bool
Validate this data, optionally with the given symbol. Will throw an error if the data doesn't exist. Returns true if the data was previously invalidated with the given symbol (or any symbol if null). This will remove the symbol (or all symbols if null) from this data's dirty set (no longer invalid).
```rust
const func: fn = self.hi;
const ptr = func.data();
assert(ptr.invalidate('something_happened'));
assert(ptr.validate('something_happened'));     // marks the data as valid again
assert_not(ptr.validate('something_happened')); // already validated
```"#.into(),
        params: vector![
            Param { name: "data".into(), param_type: Type::Void, default: None },
            Param { name: "symbol".into(), param_type: Type::Str, default: Some(Arc::new(Base::Literal(Val::Null))) }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(VALIDATE.clone());
            Ok(instructions)
        })
    }
}

/// Objects.
pub fn data_objs() -> LibFunc {
    LibFunc {
        library: DATA_LIB.clone(),
        name: "objs".into(),
        is_async: false,
        docs: r#"# Data.objs(ptr: data) -> list
List of objects that this data is attached to (will always have at least one).
```rust
const func: fn = self.hi;
assert_eq(func.data().objs().front(), self);
```"#.into(),
        params: vector![
            Param { name: "data".into(), param_type: Type::Void, default: None }
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

/// To blob.
pub fn data_to_blob() -> LibFunc {
    LibFunc {
        library: DATA_LIB.clone(),
        name: "blob".into(),
        is_async: false,
        docs: r#"# Data.blob(ptr: data) -> blob
Uses bincode serialization to serialize the data (name, attributes, value, etc.), turning it into a blob.
```rust
const func: fn = self.hi;
const bin = func.data().blob(); // entire function as a blob
```"#.into(),
        params: vector![
            Param { name: "data".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(TO_BLOB.clone());
            Ok(instructions)
        })
    }
}

/// From blob.
pub fn data_from_blob() -> LibFunc {
    LibFunc {
        library: DATA_LIB.clone(),
        name: "load_blob".into(),
        is_async: false,
        docs: r#"# Data.load_blob(bytes: blob, context: obj | str = self) -> data
Uses bincode to deserialize the data blob (name, attributes, value, etc.), adding it to the desired context object.
```rust
const func: fn = self.hi;
const bin = func.data().blob(); // entire function as a blob

const other = new {};
const dref = Data.load_blob(bin, other); // copy of the function is now on "other"
```"#.into(),
        params: vector![
            Param { name: "bytes".into(), param_type: Type::Blob, default: None },
            Param { name: "context".into(), param_type: Type::Void, default: Some(Arc::new(Base::Literal(Val::Null))) }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(FROM_BLOB.clone());
            Ok(instructions)
        })
    }
}

/// Drop.
pub fn data_drop() -> LibFunc {
    LibFunc {
        library: DATA_LIB.clone(),
        name: "drop".into(),
        is_async: false,
        docs: r#"# Data.drop(ptr: data) -> bool
Drop this data from the document, returning true if the data existed and was removed.
```rust
const func: fn = self.hi;
assert(func.data().drop());
```"#.into(),
        params: vector![
            Param { name: "data".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(DROP.clone());
            Ok(instructions)
        })
    }
}

/// Drop from.
pub fn data_drop_from() -> LibFunc {
    LibFunc {
        library: DATA_LIB.clone(),
        name: "drop_from".into(),
        is_async: false,
        docs: r#"# Data.drop_from(ptr: data, obj: obj) -> bool
Drop this data from a specific object. If the given object was the only reference, the data will be dropped completely from the document.
```rust
const func: fn = self.hi;
assert(func.data().drop_from(self));
```"#.into(),
        params: vector![
            Param { name: "data".into(), param_type: Type::Void, default: None },
            Param { name: "obj".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(DROP_FROM.clone());
            Ok(instructions)
        })
    }
}

/// Attach.
pub fn data_attach() -> LibFunc {
    LibFunc {
        library: DATA_LIB.clone(),
        name: "attach".into(),
        is_async: false,
        docs: r#"# Data.attach(ptr: data, obj: obj) -> bool
Attach this data to an additional object. This data will now be accessible using the same name from the object.
```rust
const func: fn = self.hi;
const other = new {};
assert(func.data().attach(other));
assert_eq(other.hi, func);
```"#.into(),
        params: vector![
            Param { name: "data".into(), param_type: Type::Void, default: None },
            Param { name: "obj".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(ATTACH.clone());
            Ok(instructions)
        })
    }
}

/// Move.
pub fn data_move() -> LibFunc {
    LibFunc {
        library: DATA_LIB.clone(),
        name: "move".into(),
        is_async: false,
        docs: r#"# Data.move(ptr: data, from: obj, to: obj) -> bool
Combines a drop and attach, removing this data from an object and placing it on another.
```rust
const func: fn = self.hi;
const other = new {};
assert(func.data().move(self, other));
assert_not(self.hi); // func is now on other
```"#.into(),
        params: vector![
            Param { name: "data".into(), param_type: Type::Void, default: None },
            Param { name: "from".into(), param_type: Type::Void, default: None },
            Param { name: "to".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(MOVE.clone());
            Ok(instructions)
        })
    }
}

/// From ID.
pub fn data_from_id() -> LibFunc {
    LibFunc {
        library: DATA_LIB.clone(),
        name: "from_id".into(),
        is_async: false,
        docs: r#"# Data.from_id(id: str) -> data
Create a new data pointer with a string ID.
```rust
const func: fn = self.hi;
const id = func.data().id();
assert_eq(Data.from_id(id), func.data());
```"#.into(),
        params: vector![
            Param { name: "id".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(FROM_ID.clone());
            Ok(instructions)
        })
    }
}

/// From Field Path.
pub fn data_from_field() -> LibFunc {
    LibFunc {
        library: DATA_LIB.clone(),
        name: "field".into(),
        is_async: false,
        docs: r#"# Data.field(path: str) -> data
Create a data pointer to a field, using a path/name from the current object context.
```rust
const ptr = Data.field('myfield'); // self.myfield
assert(ptr.exists());
```"#.into(),
        params: vector![
            Param { name: "path".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(FIELD.clone());
            Ok(instructions)
        })
    }
}
