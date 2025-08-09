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
use crate::{model::{libraries::data::{ATTACH, DATA_LIB, DROP, DROP_FROM, EXISTS, FIELD, FROM_ID, ID, MOVE, OBJS, TAGNAME}, LibFunc, Param}, runtime::{instruction::Instructions, Type}};


/// Id.
pub fn data_id() -> LibFunc {
    LibFunc {
        library: DATA_LIB.clone(),
        name: "id".into(),
        is_async: false,
        docs: "# Data Id\nString ID for this data reference.".into(),
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
        docs: "# Data Library Name\nThe 'tagname' for this data reference. If the data points to a function, this will return 'Fn' for example. For custom data, like a PDF, this would return 'Pdf'.".into(),
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
        docs: "# Data Exists?\nReturns true if this data reference points to valid data in a graph.".into(),
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

/// Objects.
pub fn data_objs() -> LibFunc {
    LibFunc {
        library: DATA_LIB.clone(),
        name: "objs".into(),
        is_async: false,
        docs: "# Data Objects\nList of objects that this data is attached to.".into(),
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

/// Drop.
pub fn data_drop() -> LibFunc {
    LibFunc {
        library: DATA_LIB.clone(),
        name: "drop".into(),
        is_async: false,
        docs: "# Drop\nRemove data completely from the graph.".into(),
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
        docs: "# Drop From\nRemove data from a node in the graph (object). If this node is the only object referencing the data, the data will be removed completely from the graph.".into(),
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
        docs: "# Attach To\nAttach this data to an additional object.".into(),
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
        docs: "# Move\nDrop this data from an object and move it to another object.".into(),
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
        docs: "# From ID\nCreate a data reference from a string ID.".into(),
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
        docs: "# From Field Path\nCreate a data reference from a dot '.' separated field path.".into(),
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
