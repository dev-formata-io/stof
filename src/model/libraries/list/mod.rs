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
use crate::{model::{Graph, LibFunc, Param}, runtime::{instruction::Instructions, instructions::{list::{ListIns, ANY_LIST, APPEND_LIST, AT_LIST, AT_REF_LIST, CLEAR_LIST, CONTAINS_LIST, EMPTY_LIST, FIRST_LIST, FIRST_REF_LIST, INDEX_OF_LIST, INSERT_LIST, IS_UNIFORM_LIST, JOIN_LIST, LAST_LIST, LAST_REF_LIST, LEN_LIST, POP_BACK_LIST, POP_FRONT_LIST, REMOVE_ALL_LIST, REMOVE_FIRST_LIST, REMOVE_LAST_LIST, REMOVE_LIST, REPLACE_LIST, REVERSED_LIST, REVERSE_LIST, SORT_LIST, TO_UNIFORM_LIST}, Base}, NumT, Type, Val}};


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
    graph.insert_libfunc(list_is_uniform());
    graph.insert_libfunc(list_to_uniform());
}


/// Append another list.
fn list_append() -> LibFunc {
    LibFunc {
        library: LIST_LIB.clone(),
        name: "append".into(),
        is_async: false,
        docs: "# Append\nAppends another list to this one (returns nothing).".into(),
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
        docs: "# Push Back\nPushes arguments to the back of this list.".into(),
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
        docs: "# Push Front\nPushes arguments to the front of this list (in order).".into(),
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
        docs: "# Pop Front\nRemoves a value from the front of this list, returning that value or null if the list is empty.".into(),
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
        docs: "# Pop Back\nRemoves a value from the back of this list, returning that value or null if the list is empty.".into(),
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
        docs: "# Clear\nClears the list of all values.".into(),
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
        docs: "# Reverse\nReverses the list in place.".into(),
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
        docs: "# Reversed\nReturns a new list in the reverse order.".into(),
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
        docs: "# Length\nReturns the length of this list.".into(),
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
        docs: "# At\nReturn an element at the given index (or null if out of bounds).".into(),
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
        docs: "# Empty\nReturns true if this list is empty.".into(),
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
        docs: "# Any\nReturns true if this list is not empty.".into(),
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
        docs: "# Front\nReturns the value at the front of this list or null if the list is empty.".into(),
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
        docs: "# Back\nReturns the value at the back of this list or null if the list is empty.".into(),
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
        docs: "# Join\nJoins the values in this list into a string, separated by a given separator (default is an empty space char).".into(),
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
        docs: "# Contains\nReturns true if this list contains the given value.".into(),
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
        docs: "# Index Of\nReturns the first index of the given value if found or -1 if not found.".into(),
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
        docs: "# Remove\nRemove a value at the given index, returning it if found or null if the index is out of bounds.".into(),
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
        docs: "# Remove First\nRemove the first occurrance of a value, returning it if found or null otherwise.".into(),
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
        docs: "# Remove Last\nRemove the last occurrance of a value, returning it if found or null otherwise.".into(),
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
        docs: "# Remove All\nRemove all occurrances of a value, returning true if any were found and removed.".into(),
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
        docs: "# Insert\nInsert a value into this list at the given index, pushing values right.".into(),
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
        docs: "# Replace\nReplace a value into this list at the given index, returning the old value.".into(),
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
        docs: "# Sort\nSorts this list given the default value ordering.".into(),
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

/// Is uniform type?
fn list_is_uniform() -> LibFunc {
    LibFunc {
        library: LIST_LIB.clone(),
        name: "is_uniform".into(),
        is_async: false,
        docs: "# Is Uniform?\nReturns true if the list contains only a singular type of value.".into(),
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
        docs: "# To Uniform Type\nCasts each value in this list to a given type.".into(),
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
