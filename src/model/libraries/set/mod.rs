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
use crate::{model::{Graph, LibFunc, Param}, runtime::{instruction::Instructions, instructions::set::{ANY_SET, APPEND_SET, AT_SET, CLEAR_SET, CONTAINS_SET, DIFF_SET, DISJOINT_SET, EMPTY_SET, FIRST_SET, INSERT_SET, INTERSECTION_SET, IS_UNIFORM_SET, LAST_SET, LEN_SET, POP_FIRST_SET, POP_LAST_SET, REMOVE_SET, SPLIT_SET, SUBSET_SET, SUPERSET_SET, SYMMETRIC_DIFF_SET, TO_UNIFORM_SET, UNION_SET}, NumT, Type}};


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
        docs: "# Append\nAppends another set to this one (returns nothing).".into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None },
            Param { name: "other".into(), param_type: Type::Set, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
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
        docs: "# Clear\nClear all values from a set (returns nothing).".into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
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
        docs: "# Contains?\nReturns true if the set contains a given value.".into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None },
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
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
        docs: "# First\nReturns the first (minimum) value in the set.".into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(FIRST_SET.clone());
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
        docs: "# Last\nReturns the last (maximum) value in the set.".into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(LAST_SET.clone());
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
        docs: "# Insert\nInsert a value into this set, returning true if the value was newly inserted.".into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None },
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
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
        docs: "# Split\nSplit this set into two sets - one that contains all smaller values and one that contains all larger values. Will return a tuple containing the two sets, with the smaller values at index 0 (Tup(smaller set, larger set)).".into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None },
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
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
        docs: "# Empty?\nReturns true if the set is empty.".into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
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
        docs: "# Any?\nReturns true if the set contains at least one value.".into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
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
        docs: "# Length\nReturns the size of the set.".into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
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
        docs: "# At\nReturns the value at the given index or null if the index if out of bounds.".into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None },
            Param { name: "index".into(), param_type: Type::Num(NumT::Int), default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(AT_SET.clone());
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
        docs: "# Pop First\nRemoves and returns the first (minimum) value in the set.".into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
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
        docs: "# Pop Last\nRemoves and returns the last (maximum) value in the set.".into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
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
        docs: "# Remove\nRemoves and returns a value from the set or null if the value doesn't exist.".into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None },
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
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
        docs: "# Union\nPerforms a union between two sets, returning a new set.".into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None },
            Param { name: "other".into(), param_type: Type::Set, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
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
        docs: "# Difference\nPerforms a difference between two sets, returning a new set (everything in this set that is not in other).".into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None },
            Param { name: "other".into(), param_type: Type::Set, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
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
        docs: "# Intersection\nPerforms an intersection between two sets, returning a new set.".into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None },
            Param { name: "other".into(), param_type: Type::Set, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
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
        docs: "# Symmetric Difference\nPerforms a symmetric difference between two sets, returning a new set (values in this set that do not exist in other unioned with the values in other that do not exist in this set).".into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None },
            Param { name: "other".into(), param_type: Type::Set, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
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
        docs: "# Disjoint?\nReturns true if there is no overlap between two sets (empty intersection).".into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None },
            Param { name: "other".into(), param_type: Type::Set, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
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
        docs: "# Subset?\nReturns true if all values in this set exist within another set.".into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None },
            Param { name: "other".into(), param_type: Type::Set, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
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
        docs: "# Superset?\nReturns true if all values in another set exist within this set.".into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None },
            Param { name: "other".into(), param_type: Type::Set, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
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
        docs: "# Uniform Types?\nReturns true if all values in this set are of the same type.".into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
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
        docs: "# To Uniform\nCasts all values in the set to the same type.".into(),
        params: vector![
            Param { name: "set".into(), param_type: Type::Set, default: None },
            Param { name: "type".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(TO_UNIFORM_SET.clone());
            Ok(instructions)
        })
    }
}
