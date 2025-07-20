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
use crate::{model::{num::{NumIns, NUM_LIB}, LibFunc}, runtime::instruction::Instructions};


/// Max value library function.
pub fn num_max() -> LibFunc {
    LibFunc {
        library: NUM_LIB.clone(),
        name: "max".into(),
        is_async: false,
        docs: "# Maximum Value\nReturn the maximum value for all parameters given (unbounded). If a list or set is provided, this will contemplate the max value in that collection. Will consider units if provided as well.".into(),
        params: vector![],
        return_type: None,
        unbounded_args: true,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(NumIns::Max(arg_count)));
            Ok(instructions)
        })
    }
}


/// Min value library function.
pub fn num_min() -> LibFunc {
    LibFunc {
        library: NUM_LIB.clone(),
        name: "min".into(),
        is_async: false,
        docs: "# Minimum Value\nReturn the minimum value for all parameters given (unbounded). If a list or set is provided, this will contemplate the min value in that collection. Will consider units if provided as well.".into(),
        params: vector![],
        return_type: None,
        unbounded_args: true,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(NumIns::Min(arg_count)));
            Ok(instructions)
        })
    }
}
