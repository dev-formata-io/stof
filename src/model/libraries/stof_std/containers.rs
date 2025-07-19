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
use crate::{model::{stof_std::{StdIns, STD_LIB}, LibFunc}, runtime::instruction::Instructions};


/// List constructor function.
pub fn std_list() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "list".into(),
        is_async: false,
        docs: "# List Constructor\nCreate a new list.".into(),
        params: vector![],
        return_type: None,
        unbounded_args: true,
        args_to_symbol_table: false,
        func: Arc::new(|arg_count, _env, _graph| {
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
        docs: "# Set Constructor\nCreate a new set.".into(),
        params: vector![],
        return_type: None,
        unbounded_args: true,
        args_to_symbol_table: false,
        func: Arc::new(|arg_count, _env, _graph| {
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
        docs: "# Map Constructor\nCreate a new map.".into(),
        params: vector![],
        return_type: None,
        unbounded_args: true,
        args_to_symbol_table: false,
        func: Arc::new(|arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(StdIns::Map(arg_count)));
            Ok(instructions)
        })
    }
}
