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
use arcstr::{literal, ArcStr};
use imbl::vector;
use crate::{model::{Graph, LibFunc, Param}, runtime::{instruction::Instructions, instructions::set::{ANY_SET, APPEND_SET, AT_REF_SET, AT_SET, CLEAR_SET, CONTAINS_SET, DIFF_SET, DISJOINT_SET, EMPTY_SET, FIRST_REF_SET, FIRST_SET, INSERT_SET, INTERSECTION_SET, IS_UNIFORM_SET, LAST_REF_SET, LAST_SET, LEN_SET, POP_FIRST_SET, POP_LAST_SET, REMOVE_SET, SPLIT_SET, SUBSET_SET, SUPERSET_SET, SYMMETRIC_DIFF_SET, TO_UNIFORM_SET, UNION_SET}, NumT, Type}};


/// Library name.
pub(self) const SET_LIB: ArcStr = literal!("Set");


/// Add the set library to a graph.
pub fn insert_set_lib(graph: &mut Graph) {
    graph.insert_libfunc(set_append());
    graph.insert_libfunc(set_clear());
    graph.insert_libfunc(set_contains());
    graph.insert_libfunc(set_first());
    graph.insert_libfunc(set_last());
    graph.insert_libfunc(set_insert());
    graph.insert_libfunc(set_split());
    graph.insert_libfunc(set_empty());
    graph.insert_libfunc(set_any());
    graph.insert_libfunc(set_len());
    graph.insert_libfunc(set_at());
    graph.insert_libfunc(set_pop_first());
    graph.insert_libfunc(set_pop_last());
    graph.insert_libfunc(set_remove());
    graph.insert_libfunc(set_union());
    graph.insert_libfunc(set_diff());
    graph.insert_libfunc(set_intersection());
    graph.insert_libfunc(set_symmetric_diff());
    graph.insert_libfunc(set_disjoint());
    graph.insert_libfunc(set_subset());
    graph.insert_libfunc(set_superset());
    graph.insert_libfunc(set_uniform());
    graph.insert_libfunc(set_to_uniform());
}


/// Append another set.
fn set_append() -> LibFunc {
    LibFunc {
        library: SET_LIB.clone(),
        name: "append".into(),
        is_async: false,
        docs: r#"# Set.append(set: set, other: set) -> void
Append another set to this one.
```rust
const set = {1, 2, 3};
set.append({3, 4});
assert_eq(set, {1, 2, 3, 4});
```
"#.into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None },
            Param { name: "other".into(), param_type: Type::Set, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(APPEND_SET.clone());
            Ok(instructions)
        })
    }
}

/// Clear.
fn set_clear() -> LibFunc {
    LibFunc {
        library: SET_LIB.clone(),
        name: "clear".into(),
        is_async: false,
        docs: r#"# Set.clear(set: set) -> void
Clear all values from the set.
```rust
const set = {1, 2, 3};
set.clear();
assert_eq(set, {});
```
"#.into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(CLEAR_SET.clone());
            Ok(instructions)
        })
    }
}

/// Contains?
fn set_contains() -> LibFunc {
    LibFunc {
        library: SET_LIB.clone(),
        name: "contains".into(),
        is_async: false,
        docs: r#"# Set.contains(set: set, val: unknown) -> bool
Returns true if the set contains the value.
```rust
const set = {1, 2, 3};
assert(set.contains(3));
```
"#.into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None },
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(CONTAINS_SET.clone());
            Ok(instructions)
        })
    }
}

/// First.
fn set_first() -> LibFunc {
    LibFunc {
        library: SET_LIB.clone(),
        name: "first".into(),
        is_async: false,
        docs: r#"# Set.first(set: set) -> unknown
Return the first (minimum) value in the set, or null if the set is empty.
```rust
const set = {1, 2, 3};
assert_eq(set.first(), 1);
```
"#.into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            if as_ref {
                instructions.push(FIRST_REF_SET.clone());
            } else {
                instructions.push(FIRST_SET.clone());
            }
            Ok(instructions)
        })
    }
}

/// Last.
fn set_last() -> LibFunc {
    LibFunc {
        library: SET_LIB.clone(),
        name: "last".into(),
        is_async: false,
        docs: r#"# Set.last(set: set) -> unknown
Return the last (maximum) value in the set, or null if the set is empty.
```rust
const set = {1, 2, 3};
assert_eq(set.last(), 3);
```
"#.into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            if as_ref {
                instructions.push(LAST_REF_SET.clone());
            } else {
                instructions.push(LAST_SET.clone());
            }
            Ok(instructions)
        })
    }
}

/// Insert.
fn set_insert() -> LibFunc {
    LibFunc {
        library: SET_LIB.clone(),
        name: "insert".into(),
        is_async: false,
        docs: r#"# Set.insert(set: set, val: unknown) -> bool
Insert the value into the set, returning true if the value was not previously in the set (newly inserted).
```rust
const set = {1, 2};
assert(set.insert(3));
```
"#.into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None },
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(INSERT_SET.clone());
            Ok(instructions)
        })
    }
}

/// Split.
fn set_split() -> LibFunc {
    LibFunc {
        library: SET_LIB.clone(),
        name: "split".into(),
        is_async: false,
        docs: r#"# Set.split(set: set, val: unknown) -> (set, set)
Split the set into a smaller set (left) and larger set (right) at the given value (not included in resulting sets).
```rust
const set = {1, 2, 3};
assert_eq(set.split(2), ({1}, {3}));
```
"#.into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None },
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(SPLIT_SET.clone());
            Ok(instructions)
        })
    }
}

/// Empty?
fn set_empty() -> LibFunc {
    LibFunc {
        library: SET_LIB.clone(),
        name: "empty".into(),
        is_async: false,
        docs: r#"# Set.empty(set: set) -> bool
Is this set empty?
```rust
const set = {1, 2, 3};
assert_not(set.empty());
```
"#.into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(EMPTY_SET.clone());
            Ok(instructions)
        })
    }
}

/// Any?
fn set_any() -> LibFunc {
    LibFunc {
        library: SET_LIB.clone(),
        name: "any".into(),
        is_async: false,
        docs: r#"# Set.any(set: set) -> bool
Does this set contain any values?
```rust
const set = {1, 2, 3};
assert(set.any());
```
"#.into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(ANY_SET.clone());
            Ok(instructions)
        })
    }
}

/// Length.
fn set_len() -> LibFunc {
    LibFunc {
        library: SET_LIB.clone(),
        name: "len".into(),
        is_async: false,
        docs: r#"# Set.len(set: set) -> int
Return the size of this set (cardinality).
```rust
const set = {1, 2, 3};
assert_eq(set.len(), 3);
```
"#.into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(LEN_SET.clone());
            Ok(instructions)
        })
    }
}

/// At.
fn set_at() -> LibFunc {
    LibFunc {
        library: SET_LIB.clone(),
        name: "at".into(),
        is_async: false,
        docs: r#"# Set.at(set: set, index: int) -> unknown
Return the Nth (index) element in this ordered set, or null if the index is out of bounds.
```rust
const set = {1, 2, 3};
assert_eq(set[1], 2);
```
"#.into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None },
            Param { name: "index".into(), param_type: Type::Num(NumT::Int), default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            if as_ref {
                instructions.push(AT_REF_SET.clone());
            } else {
                instructions.push(AT_SET.clone());
            }
            Ok(instructions)
        })
    }
}

/// Pop first.
fn set_pop_first() -> LibFunc {
    LibFunc {
        library: SET_LIB.clone(),
        name: "pop_first".into(),
        is_async: false,
        docs: r#"# Set.pop_first(set: set) -> unknown
Remove and return the first (minimum) value in the set.
```rust
const set = {1, 2, 3};
assert_eq(set.pop_first(), 1);
assert_eq(set, {2, 3});
```
"#.into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(POP_FIRST_SET.clone());
            Ok(instructions)
        })
    }
}

/// Pop last.
fn set_pop_last() -> LibFunc {
    LibFunc {
        library: SET_LIB.clone(),
        name: "pop_last".into(),
        is_async: false,
        docs: r#"# Set.pop_last(set: set) -> unknown
Remove and return the last (maxiumum) value in the set.
```rust
const set = {1, 2, 3};
assert_eq(set.pop_last(), 3);
assert_eq(set, {1, 2});
```
"#.into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(POP_LAST_SET.clone());
            Ok(instructions)
        })
    }
}

/// Remove.
fn set_remove() -> LibFunc {
    LibFunc {
        library: SET_LIB.clone(),
        name: "remove".into(),
        is_async: false,
        docs: r#"# Set.remove(set: set, val: unknown) -> unknown
Remove and return the value if found in the set, otherwise null.
```rust
const set = {1, 2, 3};
assert_eq(set.remove(2), 2);
assert_eq(set, {1, 3});
```
"#.into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None },
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(REMOVE_SET.clone());
            Ok(instructions)
        })
    }
}

/// Union.
fn set_union() -> LibFunc {
    LibFunc {
        library: SET_LIB.clone(),
        name: "union".into(),
        is_async: false,
        docs: r#"# Set.union(set: set, other: set) -> set
Union two sets, returning a new set.
```rust
const set = {1, 2, 3};
const other = {4, 5};
assert_eq(set.union(other), {1, 2, 3, 4, 5});
```
"#.into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None },
            Param { name: "other".into(), param_type: Type::Set, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(UNION_SET.clone());
            Ok(instructions)
        })
    }
}

/// Difference.
fn set_diff() -> LibFunc {
    LibFunc {
        library: SET_LIB.clone(),
        name: "difference".into(),
        is_async: false,
        docs: r#"# Set.difference(set: set, other: set) -> set
Perform a difference between two sets, returning a new set (everything in this set that is not in other).
```rust
const set = {1, 2, 3};
assert_eq(set.difference({2, 3}), {1});
```
"#.into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None },
            Param { name: "other".into(), param_type: Type::Set, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(DIFF_SET.clone());
            Ok(instructions)
        })
    }
}

/// Intersection.
fn set_intersection() -> LibFunc {
    LibFunc {
        library: SET_LIB.clone(),
        name: "intersection".into(),
        is_async: false,
        docs: r#"# Set.intersection(set: set, other: set) -> set
Perform an intersection between two sets, returning a new set (only elements found in both sets).
```rust
const set = {1, 2, 3};
assert_eq(set.intersection({3, 4}), {3});
```
"#.into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None },
            Param { name: "other".into(), param_type: Type::Set, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(INTERSECTION_SET.clone());
            Ok(instructions)
        })
    }
}

/// Symmetric difference.
fn set_symmetric_diff() -> LibFunc {
    LibFunc {
        library: SET_LIB.clone(),
        name: "symmetric_difference".into(),
        is_async: false,
        docs: r#"# Set.symmetric_difference(set: set, other: set) -> set
Perform a symmetric difference between two sets, returning a new set (values in this set that do not exist in other unioned with the values in other that do not exist in this set).
```rust
const set = {1, 2, 3};
assert_eq(set.symmetric_difference({2, 3, 4}), {1, 4});
```
"#.into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None },
            Param { name: "other".into(), param_type: Type::Set, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(SYMMETRIC_DIFF_SET.clone());
            Ok(instructions)
        })
    }
}

/// Disjoint?
fn set_disjoint() -> LibFunc {
    LibFunc {
        library: SET_LIB.clone(),
        name: "disjoint".into(),
        is_async: false,
        docs: r#"# Set.disjoint(set: set, other: set) -> bool
Returns true if there is no overlap between the two sets (empty intersection).
```rust
const set = {1, 2, 3};
const other = {4, 5};
assert(set.disjoint(other));
```
"#.into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None },
            Param { name: "other".into(), param_type: Type::Set, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(DISJOINT_SET.clone());
            Ok(instructions)
        })
    }
}

/// Subset?
fn set_subset() -> LibFunc {
    LibFunc {
        library: SET_LIB.clone(),
        name: "subset".into(),
        is_async: false,
        docs: r#"# Set.subset(set: set, other: set) -> bool
Returns true if all values in this set exist within another set.
```rust
const set = {2, 3};
const other = {2, 3, 4};
assert(set.subset(other));
```
"#.into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None },
            Param { name: "other".into(), param_type: Type::Set, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(SUBSET_SET.clone());
            Ok(instructions)
        })
    }
}

/// Superset?
fn set_superset() -> LibFunc {
    LibFunc {
        library: SET_LIB.clone(),
        name: "superset".into(),
        is_async: false,
        docs: r#"# Set.superset(set: set, other: set) -> bool
Returns true if all values in another set exist within this set.
```rust
const set = {2, 3};
const other = {2, 3, 4};
assert(other.superset(set));
```
"#.into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None },
            Param { name: "other".into(), param_type: Type::Set, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(SUPERSET_SET.clone());
            Ok(instructions)
        })
    }
}

/// Uniform type?
fn set_uniform() -> LibFunc {
    LibFunc {
        library: SET_LIB.clone(),
        name: "is_uniform".into(),
        is_async: false,
        docs: r#"# Set.is_uniform(set: set) -> bool
Returns true if all values in this set are of the same specific type.
```rust
const set = {2, 3};
assert(set.is_uniform());
```
"#.into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(IS_UNIFORM_SET.clone());
            Ok(instructions)
        })
    }
}

/// To uniform type.
fn set_to_uniform() -> LibFunc {
    LibFunc {
        library: SET_LIB.clone(),
        name: "to_uniform".into(),
        is_async: false,
        docs: r#"# Set.to_uniform(set: set, type: str) -> void
Try casting all set values to a single type. Type parameter is a string, just like you'd specify a type in Stof.
```rust
const set = {2000m, 3km};
set.to_uniform("km");
assert_eq(set, {2km, 3km});
```
"#.into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None },
            Param { name: "type".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(TO_UNIFORM_SET.clone());
            Ok(instructions)
        })
    }
}
