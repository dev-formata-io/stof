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
use crate::{model::{Graph, LibFunc, Param}, runtime::{instruction::Instructions, instructions::map::{ANY_MAP, APPEND_MAP, AT_MAP, AT_REF_MAP, CLEAR_MAP, CONTAINS_MAP, EMPTY_MAP, FIRST_MAP, FIRST_REF_MAP, GET_MAP, GET_REF_MAP, INSERT_MAP, KEYS_MAP, LAST_MAP, LAST_REF_MAP, LEN_MAP, POP_FIRST_MAP, POP_LAST_MAP, REMOVE_MAP, VALUES_MAP, VALUES_REF_MAP}, NumT, Type}};


/// Library name.
pub(self) const MAP_LIB: ArcStr = literal!("Map");


/// Add the map library to a graph.
pub fn insert_map_lib(graph: &mut Graph) {
    graph.insert_libfunc(map_append());
    graph.insert_libfunc(map_clear());
    graph.insert_libfunc(map_contains());
    graph.insert_libfunc(map_first());
    graph.insert_libfunc(map_last());
    graph.insert_libfunc(map_get());
    graph.insert_libfunc(map_insert());
    graph.insert_libfunc(map_empty());
    graph.insert_libfunc(map_any());
    graph.insert_libfunc(map_keys());
    graph.insert_libfunc(map_values());
    graph.insert_libfunc(map_len());
    graph.insert_libfunc(map_at());
    graph.insert_libfunc(map_pop_first());
    graph.insert_libfunc(map_pop_last());
    graph.insert_libfunc(map_remove());
}


/// Append another map.
fn map_append() -> LibFunc {
    LibFunc {
        library: MAP_LIB.clone(),
        name: "append".into(),
        is_async: false,
        docs: "# Append\nAppends another map to this one (returns nothing).".into(),
        params: vector![
            Param { name: "map".into(), param_type: Type::Map, default: None },
            Param { name: "other".into(), param_type: Type::Map, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(APPEND_MAP.clone());
            Ok(instructions)
        })
    }
}

/// Clear.
fn map_clear() -> LibFunc {
    LibFunc {
        library: MAP_LIB.clone(),
        name: "clear".into(),
        is_async: false,
        docs: "# Clear\nClears a map of all keys and values (returns nothing).".into(),
        params: vector![
            Param { name: "map".into(), param_type: Type::Map, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(CLEAR_MAP.clone());
            Ok(instructions)
        })
    }
}

/// Contains?
fn map_contains() -> LibFunc {
    LibFunc {
        library: MAP_LIB.clone(),
        name: "contains".into(),
        is_async: false,
        docs: "# Contains Key?\nReturns true if a map contains a key.".into(),
        params: vector![
            Param { name: "map".into(), param_type: Type::Map, default: None },
            Param { name: "key".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(CONTAINS_MAP.clone());
            Ok(instructions)
        })
    }
}

/// First.
fn map_first() -> LibFunc {
    LibFunc {
        library: MAP_LIB.clone(),
        name: "first".into(),
        is_async: false,
        docs: "# First\nReturns the minimum key-value pair in this map.".into(),
        params: vector![
            Param { name: "map".into(), param_type: Type::Map, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            if as_ref {
                instructions.push(FIRST_REF_MAP.clone());
            } else {
                instructions.push(FIRST_MAP.clone());
            }
            Ok(instructions)
        })
    }
}

/// Last.
fn map_last() -> LibFunc {
    LibFunc {
        library: MAP_LIB.clone(),
        name: "last".into(),
        is_async: false,
        docs: "# Last\nReturns the maximum key-value pair in this map.".into(),
        params: vector![
            Param { name: "map".into(), param_type: Type::Map, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            if as_ref {
                instructions.push(LAST_REF_MAP.clone());
            } else {
                instructions.push(LAST_MAP.clone());
            }
            Ok(instructions)
        })
    }
}

/// Get.
fn map_get() -> LibFunc {
    LibFunc {
        library: MAP_LIB.clone(),
        name: "get".into(),
        is_async: false,
        docs: "# Get\nGet a value in this map by key.".into(),
        params: vector![
            Param { name: "map".into(), param_type: Type::Map, default: None },
            Param { name: "key".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            if as_ref {
                instructions.push(GET_REF_MAP.clone());
            } else {
                instructions.push(GET_MAP.clone());
            }
            Ok(instructions)
        })
    }
}

/// Insert.
fn map_insert() -> LibFunc {
    LibFunc {
        library: MAP_LIB.clone(),
        name: "insert".into(),
        is_async: false,
        docs: "# Insert\nInsert a key-value pair into this map.".into(),
        params: vector![
            Param { name: "map".into(), param_type: Type::Map, default: None },
            Param { name: "key".into(), param_type: Type::Void, default: None },
            Param { name: "value".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(INSERT_MAP.clone());
            Ok(instructions)
        })
    }
}

/// Empty?
fn map_empty() -> LibFunc {
    LibFunc {
        library: MAP_LIB.clone(),
        name: "empty".into(),
        is_async: false,
        docs: "# Empty?\nReturns true if this map is empty.".into(),
        params: vector![
            Param { name: "map".into(), param_type: Type::Map, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(EMPTY_MAP.clone());
            Ok(instructions)
        })
    }
}

/// Any?
fn map_any() -> LibFunc {
    LibFunc {
        library: MAP_LIB.clone(),
        name: "any".into(),
        is_async: false,
        docs: "# Any?\nReturns true if this map is not empty.".into(),
        params: vector![
            Param { name: "map".into(), param_type: Type::Map, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(ANY_MAP.clone());
            Ok(instructions)
        })
    }
}

/// Keys.
fn map_keys() -> LibFunc {
    LibFunc {
        library: MAP_LIB.clone(),
        name: "keys".into(),
        is_async: false,
        docs: "# Keys\nReturns a set of keys in this map.".into(),
        params: vector![
            Param { name: "map".into(), param_type: Type::Map, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(KEYS_MAP.clone());
            Ok(instructions)
        })
    }
}

/// Values.
fn map_values() -> LibFunc {
    LibFunc {
        library: MAP_LIB.clone(),
        name: "values".into(),
        is_async: false,
        docs: "# Values\nReturns a list of values in this map.".into(),
        params: vector![
            Param { name: "map".into(), param_type: Type::Map, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            if as_ref {
                instructions.push(VALUES_REF_MAP.clone());
            } else {
                instructions.push(VALUES_MAP.clone());
            }
            Ok(instructions)
        })
    }
}

/// Len.
fn map_len() -> LibFunc {
    LibFunc {
        library: MAP_LIB.clone(),
        name: "len".into(),
        is_async: false,
        docs: "# Length (size)\nReturns the size of this map.".into(),
        params: vector![
            Param { name: "map".into(), param_type: Type::Map, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(LEN_MAP.clone());
            Ok(instructions)
        })
    }
}

/// At.
fn map_at() -> LibFunc {
    LibFunc {
        library: MAP_LIB.clone(),
        name: "at".into(),
        is_async: false,
        docs: "# At\nReturns a key-value pair at an index within this map.".into(),
        params: vector![
            Param { name: "map".into(), param_type: Type::Map, default: None },
            Param { name: "index".into(), param_type: Type::Num(NumT::Int), default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            if as_ref {
                instructions.push(AT_REF_MAP.clone());
            } else {
                instructions.push(AT_MAP.clone());
            }
            Ok(instructions)
        })
    }
}

/// Pop first.
fn map_pop_first() -> LibFunc {
    LibFunc {
        library: MAP_LIB.clone(),
        name: "pop_first".into(),
        is_async: false,
        docs: "# Pop First (min)\nRemoves and returns the first key-pair in this map (min) or null if the map is empty.".into(),
        params: vector![
            Param { name: "map".into(), param_type: Type::Map, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(POP_FIRST_MAP.clone());
            Ok(instructions)
        })
    }
}

/// Pop last.
fn map_pop_last() -> LibFunc {
    LibFunc {
        library: MAP_LIB.clone(),
        name: "pop_last".into(),
        is_async: false,
        docs: "# Pop Last (max)\nRemoves and returns the last key-pair in this map (max) or null if the map is empty.".into(),
        params: vector![
            Param { name: "map".into(), param_type: Type::Map, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(POP_LAST_MAP.clone());
            Ok(instructions)
        })
    }
}

/// Remove.
fn map_remove() -> LibFunc {
    LibFunc {
        library: MAP_LIB.clone(),
        name: "remove".into(),
        is_async: false,
        docs: "# Remove\nRemoves a value in this map by key or returns null if the key isn't found.".into(),
        params: vector![
            Param { name: "map".into(), param_type: Type::Map, default: None },
            Param { name: "key".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(REMOVE_MAP.clone());
            Ok(instructions)
        })
    }
}
