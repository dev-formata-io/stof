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
use crate::{model::{stof_std::{SLEEP, STD_LIB}, LibFunc, Param}, runtime::{instruction::Instructions, instructions::Base, Num, NumT, Type, Val}};


/// Standard process sleep function.
pub fn stof_sleep() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "sleep".into(),
        is_async: false,
        docs: "# Put Process to Sleep\nInstruct this process to sleep for an amount of time. Use time units for specificity (Ex. sleep(200ms)).".into(),
        params: vector![
            Param { name: "time".into(), param_type: Type::Num(NumT::Float), default: Some(Arc::new(Base::Literal(Val::Num(Num::Float(1000.))))) }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(SLEEP.clone());
            Ok(instructions)
        })
    }
}
