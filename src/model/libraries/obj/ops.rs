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
use crate::{model::{obj::{ANY, AT, ATTRIBUTES, AT_REF, CHILDREN, CONTAINS, CREATE_TYPE, DISTANCE, DUMP, EMPTY, EXISTS, FIELDS, FROM_ID, FROM_MAP, FUNCS, GET, GET_REF, ID, INSERT, INSTANCE_OF, IS_PARENT, IS_ROOT, LEN, MOVE, MOVE_FIELD, NAME, OBJ_LIB, PARENT, PATH, PROTO, REMOVE, REMOVE_PROTO, ROOT, RUN, SCHEMAFY, SET_PROTO, TO_MAP, TO_MAP_REF, UPCAST}, LibFunc, Param}, runtime::{instruction::Instructions, instructions::Base, NumT, Type, Val}};


/// Name.
pub fn obj_name() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "name".into(),
        is_async: false,
        docs: r#"# Obj.name(obj: obj) -> str
Return the name of this object.
```rust
const obj = new {};
assert(obj.name().len() > 0);
```
"#.into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None }
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

/// Id.
pub fn obj_id() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "id".into(),
        is_async: false,
        docs: r#"# Obj.id(obj: obj) -> str
Return the ID of this object.
```rust
const obj = new {};
assert(obj.id().len() > 0);
```
"#.into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None }
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

/// Path.
pub fn obj_path() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "path".into(),
        is_async: false,
        docs: r#"# Obj.path(obj: obj) -> str
Return the path of this object as a dot '.' separated string.
```rust
assert_eq(self.path(), "root.TestObject"); // if self is "TestObject" and it's parent is "root"
```
"#.into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(PATH.clone());
            Ok(instructions)
        })
    }
}

/// Parent.
pub fn obj_parent() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "parent".into(),
        is_async: false,
        docs: r#"# Obj.parent(obj: obj) -> obj
Return the parent of this object, or null if this object is a root.
```rust
const obj = new {};
assert_eq(obj.parent(), self);
```
"#.into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(PARENT.clone());
            Ok(instructions)
        })
    }
}

/// Is Parent?
pub fn obj_is_parent() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "is_parent".into(),
        is_async: false,
        docs: r#"# Obj.is_parent(obj: obj, other: obj) -> bool
Returns true if this object is a parent of another.
```rust
const obj = new {};
assert(self.is_parent(obj));
```
"#.into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None },
            Param { name: "other".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(IS_PARENT.clone());
            Ok(instructions)
        })
    }
}

/// Exists?
pub fn obj_exists() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "exists".into(),
        is_async: false,
        docs: r#"# Obj.exists(obj: obj) -> bool
Returns true if this object reference points to an existing object. This is false if the object has been dropped from the document.
```rust
const obj = new {};
assert(obj.exists());
```
"#.into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None }
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

/// Children.
pub fn obj_children() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "children".into(),
        is_async: false,
        docs: r#"# Obj.children(obj: obj) -> list
Returns a list containing this objects children.
```rust
const obj = new {};
assert_eq(self.children(), [obj]);
```
"#.into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(CHILDREN.clone());
            Ok(instructions)
        })
    }
}

/// Root.
pub fn obj_root() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "root".into(),
        is_async: false,
        docs: r#"# Obj.root(obj: obj) -> obj
Returns the root object that contains this object (or self if this object is a root).
```rust
const obj = new {};
assert_eq(obj.root(), self); // if self is a root
```
"#.into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(ROOT.clone());
            Ok(instructions)
        })
    }
}

/// Is root?
pub fn obj_is_root() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "is_root".into(),
        is_async: false,
        docs: r#"# Obj.is_root(obj: obj) -> bool
Returns true if this object is a root.
```rust
assert(self.is_root()); // if self is a root
```
"#.into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(IS_ROOT.clone());
            Ok(instructions)
        })
    }
}

/// Prototype.
pub fn obj_proto() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "prototype".into(),
        is_async: false,
        docs: r#"# Obj.prototype(obj: obj) -> obj
Returns the prototype object for this object or null if this object doesn't have one.
```rust
assert_not(self.prototype()); // no prototype
```
"#.into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(PROTO.clone());
            Ok(instructions)
        })
    }
}

/// Create a type.
pub fn obj_create_type() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "create_type".into(),
        is_async: false,
        docs: r#"# Obj.create_type(obj: obj, typename: str) -> void
Add a typename reference to the graph, pointing to this object. Programmatic version of #[type("typename")] attribute.
```rust
const obj = new { float x: 0, float y: 0 };
obj.create_type("MyType");

const ins = new MyType {};
assert_eq(ins.x, 0);
assert_eq(ins.y, 0);
assert_eq(typename ins, "MyType");
assert_eq(ins.prototype(), obj);
```
"#.into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None },
            Param { name: "typename".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(CREATE_TYPE.clone());
            Ok(instructions)
        })
    }
}

/// Upcast.
pub fn obj_upcast() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "upcast".into(),
        is_async: false,
        docs: r#"# Obj.upcast(obj: obj) -> bool
Set the prototype of this object to the prototype of this objects existing prototype.
```rust
const obj = new SubType {};
assert(obj.upcast());
assert_eq(typename obj, "SuperType");
```
"#.into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(UPCAST.clone());
            Ok(instructions)
        })
    }
}

/// Set prototype.
pub fn obj_set_proto() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "set_prototype".into(),
        is_async: false,
        docs: r#"# Obj.set_prototype(obj: obj, proto: obj | str) -> void
Set the prototype of this object.
```rust
const proto = new {};
const obj = new {};
obj.set_prototype(proto);
assert_eq(obj.prototype(), proto);
```
"#.into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None },
            Param { name: "proto".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(SET_PROTO.clone());
            Ok(instructions)
        })
    }
}

/// Remove prototype.
pub fn obj_remove_proto() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "remove_prototype".into(),
        is_async: false,
        docs: r#"# Obj.remove_prototype(obj: obj) -> void
Remove an object's prototype.
```rust
const obj = new MyType {};
obj.remove_prototype();
assert_eq(typename obj, "obj");
```
"#.into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(REMOVE_PROTO.clone());
            Ok(instructions)
        })
    }
}

/// Instance of prototype?
pub fn obj_instance_of_proto() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "instance_of".into(),
        is_async: false,
        docs: r#"# Obj.instance_of(obj: obj, proto: str | obj) -> bool
Returns true if this object is an instance of a prototype.
```rust
const obj = new MyType {};
assert(obj.instance_of("MyType"));
```
"#.into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None },
            Param { name: "proto".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(INSTANCE_OF.clone());
            Ok(instructions)
        })
    }
}

/// Length.
pub fn obj_len() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "len".into(),
        is_async: false,
        docs: r#"# Obj.len(obj: obj) -> int
Number of fields on this object.
```rust
const obj = new { x: 0, y: 0 };
assert_eq(obj.len(), 2);
```
"#.into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(LEN.clone());
            Ok(instructions)
        })
    }
}

/// At.
pub fn obj_at() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "at".into(),
        is_async: false,
        docs: r#"# Obj.at(obj: obj, index: int) -> (str, unknown)
Field (name, value) on this object at the given index, or null if the index is out of bounds.
```rust
const obj = new { x: 0, y: 0 };
assert_eq(obj[1], ("y", 0));
```
"#.into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None },
            Param { name: "index".into(), param_type: Type::Num(NumT::Int), default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            if as_ref {
                instructions.push(AT_REF.clone());
            } else {
                instructions.push(AT.clone());
            }
            Ok(instructions)
        })
    }
}

/// Get.
pub fn obj_get() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "get".into(),
        is_async: false,
        docs: r#"# Obj.get(obj: obj, name: str) -> unknown
Get data on this object by name (field value, fn, or data pointer).
```rust
const obj = new { x: 0, y: 0 };
assert_eq(obj.get("x"), 0);
```
"#.into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None },
            Param { name: "name".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            if as_ref {
                instructions.push(GET_REF.clone());
            } else {
                instructions.push(GET.clone());
            }
            Ok(instructions)
        })
    }
}

/// Contains?
pub fn obj_contains() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "contains".into(),
        is_async: false,
        docs: r#"# Obj.contains(obj: obj, name: str) -> bool
Return true if this object contains data with the given name.
```rust
const obj = new { x: 0, y: 0 };
assert(obj.contains("y"));
```
"#.into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None },
            Param { name: "name".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(CONTAINS.clone());
            Ok(instructions)
        })
    }
}

/// Insert.
pub fn obj_insert() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "insert".into(),
        is_async: false,
        docs: r#"# Obj.insert(obj: obj, path: str, value: unknown) -> void
Either creates or assigns to a field, just like a normal field assignment, using this object as a starting context.
```rust
const obj = new { x: 0, y: 0 };
obj.insert("z", 9);
assert_eq(obj.z, 9);
```
"#.into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None },
            Param { name: "path".into(), param_type: Type::Str, default: None },
            Param { name: "value".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(INSERT.clone());
            Ok(instructions)
        })
    }
}

/// Remove.
pub fn obj_remove() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "remove".into(),
        is_async: false,
        docs: r#"# Obj.remove(obj: obj, path: str, shallow: bool = false) -> bool
Performs a "drop" operation, just like the Std.drop(..) function, using this object as a starting context. Use this to remove fields, functions, data, etc.

## Shallow
If shallow is true and the path references an object field, drop the field, but don't drop the object from the graph. Default behavior is to drop objects.

```rust
const obj = new { x: 0, y: 0 };
assert(obj.remove("x"));
assert_not(obj.x);
```
"#.into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None },
            Param { name: "path".into(), param_type: Type::Str, default: None },
            Param { name: "shallow".into(), param_type: Type::Bool, default: Some(Arc::new(Base::Literal(Val::Bool(false)))) }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(REMOVE.clone());
            Ok(instructions)
        })
    }
}

/// Move field.
pub fn obj_move_field() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "move_field".into(),
        is_async: false,
        docs: r#"# Obj.move_field(obj: obj, source: str, dest: str) -> bool
Move or rename a field from a source path/name to a destination path/name (like "mv" in bash), returning true if successfully moved.
```rust
const obj = new { x: 0, y: 0 };
obj.move_field("x", "dude");
assert_eq(obj.dude, 0);
assert_not(obj.x);
```
"#.into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None },
            Param { name: "source".into(), param_type: Type::Str, default: None },
            Param { name: "dest".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(MOVE_FIELD.clone());
            Ok(instructions)
        })
    }
}

/// Fields.
pub fn obj_fields() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "fields".into(),
        is_async: false,
        docs: r#"# Obj.fields(obj: obj) -> list
Returns a list of fields (tuples with name and value each) on this object.
```rust
const obj = new { x: 0, y: 0 };
assert_eq(obj.fields(), [("x", 0), ("y", 0)]);
```
"#.into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(FIELDS.clone());
            Ok(instructions)
        })
    }
}

/// Funcs.
pub fn obj_funcs() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "funcs".into(),
        is_async: false,
        docs: r#"# Obj.funcs(obj: obj, attributes: str | list | set = null) -> list
Returns a list of functions on this object, optionally filtering by attributes (str, list of str, set of str, tuple of str).
```rust
// #[myfunc] fn func() {}
assert_eq(self.funcs("myfunc"), [self.func]);
```
"#.into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None },
            Param { name: "attributes".into(), param_type: Type::Void, default: Some(Arc::new(Base::Literal(Val::Null))) },
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(FUNCS.clone());
            Ok(instructions)
        })
    }
}

/// Empty?
pub fn obj_empty() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "empty".into(),
        is_async: false,
        docs: r#"# Obj.empty(obj: obj) -> bool
Returns true if this object doesn't have any data attached to it.
```rust
const obj = new { x: 0, y: 0 };
assert_not(obj.empty());
```
"#.into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(EMPTY.clone());
            Ok(instructions)
        })
    }
}

/// Any?
pub fn obj_any() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "any".into(),
        is_async: false,
        docs: r#"# Obj.any(obj: obj) -> bool
Returns true if this object has any data attached to it.
```rust
const obj = new { x: 0, y: 0 };
assert(obj.any());
```
"#.into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(ANY.clone());
            Ok(instructions)
        })
    }
}

/// Attributes.
pub fn obj_attributes() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "attributes".into(),
        is_async: false,
        docs: r#"# Obj.attributes(obj: obj, path: str = null) -> map
Returns a map of attributes, either for this object if the path is null, or for the field/func/obj at the given path.
```rust
assert_eq(self.attributes(), {"a": null}); // if self was defined as a field with the attribute #[a]
```
"#.into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None },
            Param { name: "path".into(), param_type: Type::Str, default: Some(Arc::new(Base::Literal(Val::Null))) }
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

/// Move.
pub fn obj_move() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "move".into(),
        is_async: false,
        docs: r#"# Obj.move(obj: obj, dest: obj) -> bool
Move this object to a new parent destination. Parent destination cannot be a child of this object (node detachment).
```rust
const obj = new { x: 0, y: 0 };
const other = new {};
obj.move(other);
assert_eq(obj.parent(), other);
```
"#.into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None },
            Param { name: "dest".into(), param_type: Type::Void, default: None }
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

/// Distance.
pub fn obj_dist() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "dist".into(),
        is_async: false,
        docs: r#"# Obj.dist(obj: obj, other: obj) -> int
Get the distance between two objects (number of edges that separate them).
```rust
const obj = new { x: 0, y: 0 };
assert_eq(obj.dist(self), 1);
```
"#.into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None },
            Param { name: "other".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(DISTANCE.clone());
            Ok(instructions)
        })
    }
}

/// Run.
pub fn obj_run() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "run".into(),
        is_async: false,
        docs: r#"# Obj.run(obj: obj) -> void
Run an object (like calling a function, but for the entire object as a task). This will execute all fields and functions with a #[run] attribute, optionally with an order #[run(3)]. Any sub objects encountered will also get ran recursively. Arrays act like pipelines, unlocking serious functionality.

## Motivation
This concept enables data-driven abstractions above function calls. An example would be setting some fields on an object that already has some #[run] functions defined, ready to utilize the values in those fields. With prototypes, you can probably see how this is a powerful tool.

### Concrete Example
```rust
#[type]
Request: {
    str name: "europe"

    #[run]
    fn execute() {
        self.result = await Http.fetch("https://myawesomeendpoint/" + self.name);
    }
}

#[main]
fn example() {
    const req = new Request { name: "usa" };
    req.run();
    // now work with req.result as needed
}
```
"#.into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None },
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(RUN.clone());
            Ok(instructions)
        })
    }
}

/// Schemafy.
pub fn obj_schemafy() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "schemafy".into(),
        is_async: false,
        docs: r#"# Obj.schemafy(schema: obj, target: obj, remove_invalid: bool = false, remove_undefined: bool = false) -> bool
Applies all #[schema] fields from a schema object onto a target object, manipulating the target's fields accordingly and returning true if the target is determined to be valid (matches the schema).

## Use Cases
- filtering & renaming fields as a batch
- validation
- structured transformations (to/from APIs, etc.)
- access control

```rust
schema: {
    #[schema((target_value: str): bool => target_value.len() > 2)]
    first: 'John'

    #[schema(( // pipelines are big AND filters, applied in order and short circuited like &&
        (target_value: unknown): bool => (typeof target_value) == 'str',
        (target_value: str): bool => target_value.contains('Dude'),
    ))]
    last: 'Doe'
}

target: {
    first: 'aj'
    last: 'Dude'
    undefined: 'blah'
}

#[test]
fn schemafy_obj() {
    assert(self.schema.schemafy(self.target, remove_invalid = true, remove_undefined = true));
    assert_eq(str(self.target), "{\"last\":\"Dude\"}");
}
```
"#.into(),
        params: vector![
            Param { name: "schema".into(), param_type: Type::Void, default: None },
            Param { name: "target".into(), param_type: Type::Void, default: None },
            Param { name: "remove_invalid".into(), param_type: Type::Bool, default: Some(Arc::new(Base::Literal(Val::Bool(false)))) },
            Param { name: "remove_undefined".into(), param_type: Type::Bool, default: Some(Arc::new(Base::Literal(Val::Bool(false)))) },
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(SCHEMAFY.clone());
            Ok(instructions)
        })
    }
}

/// To Map.
pub fn obj_to_map() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "to_map".into(),
        is_async: false,
        docs: r#"# Obj.to_map(obj: obj) -> map
Create a new map out of this object's fields.
```rust
const obj = new { x: 3km, y: 5.5m };
const map = obj.to_map();
assert_eq(map.get("x"), 3km);
```
"#.into(),
        params: vector![
            Param { name: "obj".into(), param_type: Type::Void, default: None },
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            if as_ref {
                instructions.push(TO_MAP_REF.clone());
            } else {
                instructions.push(TO_MAP.clone());
            }
            Ok(instructions)
        })
    }
}

/// From Map.
pub fn obj_from_map() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "from_map".into(),
        is_async: false,
        docs: r#"# Obj.from_map(map: map) -> obj
Get the distance between two objects (number of edges that separate them).
```rust
const map = { "x": 0, "y": 0 };
const obj = Obj.from_map(map);
assert_eq(obj.x, 0);
```
"#.into(),
        params: vector![
            Param { name: "map".into(), param_type: Type::Map, default: None },
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(FROM_MAP.clone());
            Ok(instructions)
        })
    }
}

/// From ID.
pub fn obj_from_id() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "from_id".into(),
        is_async: false,
        docs: r#"# Obj.from_id(id: str) -> obj
Create a new object reference from an ID. Objects in Stof are references just like data.
```rust
const obj = new { x: 0, y: 0 };
const ptr = Obj.from_id(obj.id());
assert_eq(ptr, obj);
```
"#.into(),
        params: vector![
            Param { name: "id".into(), param_type: Type::Str, default: None },
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

/// Dump graph info.
pub fn obj_dump_graph() -> LibFunc {
    LibFunc {
        library: OBJ_LIB.clone(),
        name: "dbg_graph".into(),
        is_async: false,
        docs: r#"# Obj.dbg_graph() -> void
Utility function for dumping the complete graph, helpful for some debugging cases. To dump a specific node, use Std.dbg(..) with the desired object(s).
"#.into(),
        params: vector![],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(DUMP.clone());
            Ok(instructions)
        })
    }
}
