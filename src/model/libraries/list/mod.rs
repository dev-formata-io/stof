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
use arcstr::{literal, ArcStr};
use imbl::vector;
use crate::{model::{Graph, LibFunc, Param}, runtime::{NumT, Type, Val, instruction::Instructions, instructions::{Base, list::{ANY_LIST, APPEND_LIST, AT_LIST, AT_REF_LIST, CLEAR_LIST, CONTAINS_LIST, EMPTY_LIST, FIRST_LIST, FIRST_REF_LIST, INDEX_OF_LIST, INSERT_LIST, IS_UNIFORM_LIST, JOIN_LIST, LAST_LIST, LAST_REF_LIST, LEN_LIST, ListIns, POP_BACK_LIST, POP_FRONT_LIST, REMOVE_ALL_LIST, REMOVE_FIRST_LIST, REMOVE_LAST_LIST, REMOVE_LIST, REPLACE_LIST, REVERSE_LIST, REVERSED_LIST, SORT_LIST, SORT_LIST_BY, TO_UNIFORM_LIST}}}};


/// Library name.
pub(self) const LIST_LIB: ArcStr = literal!("List");


/// Add the list library to a graph.
pub fn insert_list_lib(graph: &mut Graph) {
    graph.insert_libfunc(list_append());
    graph.insert_libfunc(list_push_back());
    graph.insert_libfunc(list_push_front());
    graph.insert_libfunc(list_pop_back());
    graph.insert_libfunc(list_pop_front());
    graph.insert_libfunc(list_clear());
    graph.insert_libfunc(list_reverse());
    graph.insert_libfunc(list_reversed());
    graph.insert_libfunc(list_len());
    graph.insert_libfunc(list_at());
    graph.insert_libfunc(list_empty());
    graph.insert_libfunc(list_any());
    graph.insert_libfunc(list_front());
    graph.insert_libfunc(list_back());
    graph.insert_libfunc(list_join());
    graph.insert_libfunc(list_index_of());
    graph.insert_libfunc(list_contains());
    graph.insert_libfunc(list_remove());
    graph.insert_libfunc(list_remove_first());
    graph.insert_libfunc(list_remove_last());
    graph.insert_libfunc(list_remove_all());
    graph.insert_libfunc(list_insert());
    graph.insert_libfunc(list_replace());
    graph.insert_libfunc(list_sort());
    graph.insert_libfunc(list_sort_by());
    graph.insert_libfunc(list_is_uniform());
    graph.insert_libfunc(list_to_uniform());
}


/// Append another list.
fn list_append() -> LibFunc {
    LibFunc {
        library: LIST_LIB.clone(),
        name: "append".into(),
        is_async: false,
        docs: r#"# List.append(array: list, other: list) -> void
Append another list to this list, leaving other unmodified.
```rust
const array = [1, 2, 3];
array.append([4, 5]);
assert_eq(array, [1, 2, 3, 4, 5]);
```"#.into(),
        params: vector![
            Param { name: "list".into(), param_type: Type::List, default: None },
            Param { name: "other".into(), param_type: Type::List, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(APPEND_LIST.clone());
            Ok(instructions)
        })
    }
}

/// Push values onto the back of a list.
fn list_push_back() -> LibFunc {
    LibFunc {
        library: LIST_LIB.clone(),
        name: "push_back".into(),
        is_async: false,
        docs: r#"# List.push_back(array: list, ..) -> void
Push N values to the back of this list.
```rust
const array = [1, 2, 3];
array.push_back(4, 5);
assert_eq(array, [1, 2, 3, 4, 5]);
```"#.into(),
        params: vector![
            Param { name: "list".into(), param_type: Type::List, default: None }
        ],
        return_type: None,
        unbounded_args: true,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(ListIns::PushBack(arg_count)));
            Ok(instructions)
        })
    }
}

/// Push values onto the front of a list.
fn list_push_front() -> LibFunc {
    LibFunc {
        library: LIST_LIB.clone(),
        name: "push_front".into(),
        is_async: false,
        docs: r#"# List.push_front(array: list, ..) -> void
Push N values to the front of this list.
```rust
const array = [1, 2, 3];
array.push_front(4, 5);
assert_eq(array, [5, 4, 1, 2, 3]);
```"#.into(),
        params: vector![
            Param { name: "list".into(), param_type: Type::List, default: None }
        ],
        return_type: None,
        unbounded_args: true,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(ListIns::PushFront(arg_count)));
            Ok(instructions)
        })
    }
}

/// Pop front.
fn list_pop_front() -> LibFunc {
    LibFunc {
        library: LIST_LIB.clone(),
        name: "pop_front".into(),
        is_async: false,
        docs: r#"# List.pop_front(array: list) -> unknown
Remove a single value from the front of this list and return it. 
```rust
const array = [1, 2, 3];
assert_eq(array.pop_front(), 1);
assert_eq(array, [2, 3]);
```"#.into(),
        params: vector![
            Param { name: "list".into(), param_type: Type::List, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(POP_FRONT_LIST.clone());
            Ok(instructions)
        })
    }
}

/// Pop back.
fn list_pop_back() -> LibFunc {
    LibFunc {
        library: LIST_LIB.clone(),
        name: "pop_back".into(),
        is_async: false,
        docs: r#"# List.pop_back(array: list) -> unknown
Remove a single value from the back of this list and return it. 
```rust
const array = [1, 2, 3];
assert_eq(array.pop_back(), 3);
assert_eq(array, [1, 2]);
```"#.into(),
        params: vector![
            Param { name: "list".into(), param_type: Type::List, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(POP_BACK_LIST.clone());
            Ok(instructions)
        })
    }
}

/// Clear.
fn list_clear() -> LibFunc {
    LibFunc {
        library: LIST_LIB.clone(),
        name: "clear".into(),
        is_async: false,
        docs: r#"# List.clear(array: list) -> void
Clear all values from this list.
```rust
const array = [1, 2, 3];
array.clear();
assert_eq(array, []);
```"#.into(),
        params: vector![
            Param { name: "list".into(), param_type: Type::List, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(CLEAR_LIST.clone());
            Ok(instructions)
        })
    }
}

/// Reverse.
fn list_reverse() -> LibFunc {
    LibFunc {
        library: LIST_LIB.clone(),
        name: "reverse".into(),
        is_async: false,
        docs: r#"# List.reverse(array: list) -> void
Reverses this list in-place.
```rust
const array = [1, 2, 3];
array.reverse();
assert_eq(array, [3, 2, 1]);
```"#.into(),
        params: vector![
            Param { name: "list".into(), param_type: Type::List, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(REVERSE_LIST.clone());
            Ok(instructions)
        })
    }
}

/// Reversed.
fn list_reversed() -> LibFunc {
    LibFunc {
        library: LIST_LIB.clone(),
        name: "reversed".into(),
        is_async: false,
        docs: r#"# List.reversed(array: list) -> list
Return a new list that is reversed, leaving this list unmodified.
```rust
const array = [1, 2, 3];
const other = array.reversed();
assert_eq(array, [1, 2, 3]);
assert_eq(other, [3, 2, 1]);
```"#.into(),
        params: vector![
            Param { name: "list".into(), param_type: Type::List, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(REVERSED_LIST.clone());
            Ok(instructions)
        })
    }
}

/// Length.
fn list_len() -> LibFunc {
    LibFunc {
        library: LIST_LIB.clone(),
        name: "len".into(),
        is_async: false,
        docs: r#"# List.len(array: list) -> int
Return the length of this list.
```rust
const array = [1, 2, 3];
assert_eq(array.len(), 3);
```"#.into(),
        params: vector![
            Param { name: "list".into(), param_type: Type::List, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(LEN_LIST.clone());
            Ok(instructions)
        })
    }
}

/// At.
fn list_at() -> LibFunc {
    LibFunc {
        library: LIST_LIB.clone(),
        name: "at".into(),
        is_async: false,
        docs: r#"# List.at(array: list, index: int) -> unknown
Get the value at the given index, optionally by reference. 
```rust
const array = [1, 2, 3];
let v = &array[1]; // &List.at(array, 1);
v = 5;
assert_eq(array, [1, 5, 3]);
```"#.into(),
        params: vector![
            Param { name: "list".into(), param_type: Type::List, default: None },
            Param { name: "index".into(), param_type: Type::Num(NumT::Int), default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            if as_ref {
                instructions.push(AT_REF_LIST.clone());
            } else {
                instructions.push(AT_LIST.clone());
            }
            Ok(instructions)
        })
    }
}

/// Empty?
fn list_empty() -> LibFunc {
    LibFunc {
        library: LIST_LIB.clone(),
        name: "empty".into(),
        is_async: false,
        docs: r#"# List.empty(array: list) -> bool
Is this list empty?
```rust
const array = [1];
assert_not(array.empty());
```"#.into(),
        params: vector![
            Param { name: "list".into(), param_type: Type::List, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(EMPTY_LIST.clone());
            Ok(instructions)
        })
    }
}

/// Any?
fn list_any() -> LibFunc {
    LibFunc {
        library: LIST_LIB.clone(),
        name: "any".into(),
        is_async: false,
        docs: r#"# List.any(array: list) -> bool
Does this list contain any values?
```rust
const array = [1];
assert(array.any());
```"#.into(),
        params: vector![
            Param { name: "list".into(), param_type: Type::List, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(ANY_LIST.clone());
            Ok(instructions)
        })
    }
}

/// Front.
fn list_front() -> LibFunc {
    LibFunc {
        library: LIST_LIB.clone(),
        name: "front".into(),
        is_async: false,
        docs: r#"# List.front(array: list) -> unknown
Get the value at the front of this list, optionally by reference.
```rust
const array = [1];
assert_eq(array.front(), 1);
```"#.into(),
        params: vector![
            Param { name: "list".into(), param_type: Type::List, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            if as_ref {
                instructions.push(FIRST_REF_LIST.clone());
            } else {
                instructions.push(FIRST_LIST.clone());
            }
            Ok(instructions)
        })
    }
}

/// Back.
fn list_back() -> LibFunc {
    LibFunc {
        library: LIST_LIB.clone(),
        name: "back".into(),
        is_async: false,
        docs: r#"# List.back(array: list) -> unknown
Get the value at the back of this list, optionally by reference.
```rust
const array = [1, 2];
assert_eq(array.back(), 2);
```"#.into(),
        params: vector![
            Param { name: "list".into(), param_type: Type::List, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            if as_ref {
                instructions.push(LAST_REF_LIST.clone());
            } else {
                instructions.push(LAST_LIST.clone());
            }
            Ok(instructions)
        })
    }
}

/// Join.
fn list_join() -> LibFunc {
    LibFunc {
        library: LIST_LIB.clone(),
        name: "join".into(),
        is_async: false,
        docs: r#"# List.join(array: list, sep: str) -> str
Join the values in this array together into a single string.
```rust
const array = ["hello", "world"];
assert_eq(array.join(", "), "hello, world");
```"#.into(),
        params: vector![
            Param { name: "list".into(), param_type: Type::List, default: None },
            Param { name: "sep".into(), param_type: Type::Str, default: Some(Arc::new(Base::Literal(Val::Str(literal!(" "))))) }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(JOIN_LIST.clone());
            Ok(instructions)
        })
    }
}

/// Contains?
fn list_contains() -> LibFunc {
    LibFunc {
        library: LIST_LIB.clone(),
        name: "contains".into(),
        is_async: false,
        docs: r#"# List.contains(array: list, value: unknown) -> bool
Does this list contain the given value?
```rust
const array = [1];
assert(array.contains(1));
```"#.into(),
        params: vector![
            Param { name: "list".into(), param_type: Type::List, default: None },
            Param { name: "search".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(CONTAINS_LIST.clone());
            Ok(instructions)
        })
    }
}

/// Index of.
fn list_index_of() -> LibFunc {
    LibFunc {
        library: LIST_LIB.clone(),
        name: "index_of".into(),
        is_async: false,
        docs: r#"# List.index_of(array: list, v: unknown) -> int
If the list contains the given value, return the index of the first matched value. Returns -1 if the list does not contain the given value.
```rust
const array = [1, 2, 3];
assert_eq(array.index_of(2), 1);
```"#.into(),
        params: vector![
            Param { name: "list".into(), param_type: Type::List, default: None },
            Param { name: "search".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(INDEX_OF_LIST.clone());
            Ok(instructions)
        })
    }
}

/// Remove.
fn list_remove() -> LibFunc {
    LibFunc {
        library: LIST_LIB.clone(),
        name: "remove".into(),
        is_async: false,
        docs: r#"# List.remove(array: list, index: int) -> unknown
Remove a value at the given index and return it. Returns null if index is out of bounds.
```rust
const array = [1];
assert_eq(array.remove(0), 1);
assert(array.empty());
```"#.into(),
        params: vector![
            Param { name: "list".into(), param_type: Type::List, default: None },
            Param { name: "index".into(), param_type: Type::Num(NumT::Int), default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(REMOVE_LIST.clone());
            Ok(instructions)
        })
    }
}

/// Remove first.
fn list_remove_first() -> LibFunc {
    LibFunc {
        library: LIST_LIB.clone(),
        name: "remove_first".into(),
        is_async: false,
        docs: r#"# List.remove_first(array: list, val: unknown) -> unknown
Remove the first occurrance of a value in this array (equals) and return it.
```rust
const array = [2, 1, 1, 2];
assert_eq(array.remove_first(2), 2);
assert_eq(array, [1, 1, 2]);
```"#.into(),
        params: vector![
            Param { name: "list".into(), param_type: Type::List, default: None },
            Param { name: "search".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(REMOVE_FIRST_LIST.clone());
            Ok(instructions)
        })
    }
}

/// Remove last.
fn list_remove_last() -> LibFunc {
    LibFunc {
        library: LIST_LIB.clone(),
        name: "remove_last".into(),
        is_async: false,
        docs: r#"# List.remove_last(array: list, val: unknown) -> unknown
Remove the last occurrance of a value in this array (equals) and return it.
```rust
const array = [2, 1, 1, 2];
assert_eq(array.remove_last(2), 2);
assert_eq(array, [2, 1, 1]);
```"#.into(),
        params: vector![
            Param { name: "list".into(), param_type: Type::List, default: None },
            Param { name: "search".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(REMOVE_LAST_LIST.clone());
            Ok(instructions)
        })
    }
}

/// Remove all.
fn list_remove_all() -> LibFunc {
    LibFunc {
        library: LIST_LIB.clone(),
        name: "remove_all".into(),
        is_async: false,
        docs: r#"# List.remove_all(array: list, val: unknown) -> bool
Remove all occurrances of a value in this array (equals) and return true if any were removed.
```rust
const array = [2, 1, 1, 2];
assert(array.remove_all(2));
assert_eq(array, [1, 1]);
```"#.into(),
        params: vector![
            Param { name: "list".into(), param_type: Type::List, default: None },
            Param { name: "search".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(REMOVE_ALL_LIST.clone());
            Ok(instructions)
        })
    }
}

/// Insert.
fn list_insert() -> LibFunc {
    LibFunc {
        library: LIST_LIB.clone(),
        name: "insert".into(),
        is_async: false,
        docs: r#"# List.insert(array: list, index: int, val: unknown) -> void
Insert a value into this list at the given index.
```rust
const array = [2, 1];
array.insert(1, 3);
assert_eq(array, [2, 3, 1]);
```"#.into(),
        params: vector![
            Param { name: "list".into(), param_type: Type::List, default: None },
            Param { name: "index".into(), param_type: Type::Num(NumT::Int), default: None },
            Param { name: "value".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(INSERT_LIST.clone());
            Ok(instructions)
        })
    }
}

/// Replace.
fn list_replace() -> LibFunc {
    LibFunc {
        library: LIST_LIB.clone(),
        name: "replace".into(),
        is_async: false,
        docs: r#"# List.replace(array: list, index: int, val: unknown) -> unknown
Replace/set the value at the given index with a new value, returning the old.
```rust
const array = [2, 1];
assert_eq(array.replace(1, 4), 1);
assert_eq(array, [2, 4]);
```"#.into(),
        params: vector![
            Param { name: "list".into(), param_type: Type::List, default: None },
            Param { name: "index".into(), param_type: Type::Num(NumT::Int), default: None },
            Param { name: "value".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(REPLACE_LIST.clone());
            Ok(instructions)
        })
    }
}

/// Sort.
fn list_sort() -> LibFunc {
    LibFunc {
        library: LIST_LIB.clone(),
        name: "sort".into(),
        is_async: false,
        docs: r#"# List.sort(array: list) -> void
Sort the values in this array according to their already defined ordering.
```rust
const array = [2, 1, 4, 3];
array.sort();
assert_eq(array, [1, 2, 3, 4]);
```"#.into(),
        params: vector![
            Param { name: "list".into(), param_type: Type::List, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(SORT_LIST.clone());
            Ok(instructions)
        })
    }
}

/// Sort by.
fn list_sort_by() -> LibFunc {
    LibFunc {
        library: LIST_LIB.clone(),
        name: "sort_by".into(),
        is_async: false,
        docs: r#"# List.sort_by(array: list, func: fn) -> void
Sort the values in this array according to a function that takes two list arguments and returns an integer (< 0 for less, > 0 for greater, and 0 for equal).
```rust
const array = [2, 1, 4, 3];
array.sort_by((a: int, b: int): int => {
    if (a < b) 1
    if (a > b) -1
    0
});
assert_eq(array, [4, 3, 2, 1]);
```"#.into(),
        params: vector![
            Param { name: "list".into(), param_type: Type::List, default: None },
            Param { name: "func".into(), param_type: Type::Fn, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(SORT_LIST_BY.clone());
            Ok(instructions)
        })
    }
}

/// Is uniform type?
fn list_is_uniform() -> LibFunc {
    LibFunc {
        library: LIST_LIB.clone(),
        name: "is_uniform".into(),
        is_async: false,
        docs: r#"# List.is_uniform(array: list) -> bool
Returns true if every value in this list has the same specific type (does not account for object prototype inheritance).
```rust
const array = ["hi", true];
assert_not(array.is_uniform());
```"#.into(),
        params: vector![
            Param { name: "list".into(), param_type: Type::List, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(IS_UNIFORM_LIST.clone());
            Ok(instructions)
        })
    }
}

/// To uniform type.
fn list_to_uniform() -> LibFunc {
    LibFunc {
        library: LIST_LIB.clone(),
        name: "to_uniform".into(),
        is_async: false,
        docs: r#"# List.to_uniform(array: list, type: str) -> void
Try casting all values in this list to the given type (given as a string like you would in a Stof file). Will throw an error if a value cannot be cast.
```rust
const array = [1, "hi", true];
array.to_uniform("str");
assert_eq(array, ["1", "hi", "true"]);
```"#.into(),
        params: vector![
            Param { name: "list".into(), param_type: Type::List, default: None },
            Param { name: "type".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(TO_UNIFORM_LIST.clone());
            Ok(instructions)
        })
    }
}
