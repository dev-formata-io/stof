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
use crate::{model::{stof_std::{StdIns, STD_LIB}, LibFunc}, runtime::{instruction::Instructions}};


/// Standard printline function.
pub fn pln() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "pln".into(),
        is_async: false,
        docs: "# Print to Standard Output\nWill print N arguments to the standard output stream.".into(),
        params: vector![],
        return_type: None,
        unbounded_args: true, // allow an unbounded number of arguments
        args_to_symbol_table: false, // keep the arg on the stack instead of putting it into st
        func: Arc::new(|_as_ref, arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(StdIns::Pln(arg_count)));
            Ok(instructions)
        })
    }
}

/// Standard debug print function.
pub fn dbg() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "dbg".into(),
        is_async: false,
        docs: "# Print Debug to Standard Output\nWill print N arguments (using a debug format) to the standard output stream.".into(),
        params: vector![],
        return_type: None,
        unbounded_args: true, // allow an unbounded number of arguments
        args_to_symbol_table: false, // keep the arg on the stack instead of putting it into st
        func: Arc::new(|_as_ref, arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(StdIns::Dbg(arg_count)));
            Ok(instructions)
        })
    }
}

/// Standard printline function to error stream.
pub fn err() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "err".into(),
        is_async: false,
        docs: "# Print to Error Output\nWill print N arguments to the error output stream.".into(),
        params: vector![],
        return_type: None,
        unbounded_args: true, // allow an unbounded number of arguments
        args_to_symbol_table: false, // keep the arg on the stack instead of putting it into st
        func: Arc::new(|_as_ref, arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(StdIns::Err(arg_count)));
            Ok(instructions)
        })
    }
}
