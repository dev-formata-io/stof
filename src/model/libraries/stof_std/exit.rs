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


/// Standard process exit function.
pub fn stof_exit() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "exit".into(),
        is_async: false,
        docs: "# Exit a process\nImmediately terminates this (or another) process. Pass a promise into this function to terminate that process.".into(),
        params: vector![],
        return_type: None,
        unbounded_args: true,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(StdIns::Exit(arg_count)));
            Ok(instructions)
        })
    }
}
