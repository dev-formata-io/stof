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
use crate::{model::{num::{ABS, NUM_LIB}, LibFunc, Param}, runtime::{instruction::Instructions, instructions::Base, NumT, Type}};


/// Absolute value library function (float output version (or units)).
pub fn num_abs() -> LibFunc {
    LibFunc {
        library: NUM_LIB.clone(),
        name: "abs".into(),
        is_async: false,
        docs: r#""#.into(),
        params: vector![
            // Param is void type to prevent a need to cast to unknown or something
            Param { name: "val".into(), param_type: Type::Void, default: None }
        ],
        return_type: Some(Type::Num(NumT::Float)),
        func: Arc::new(|_env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(Base::LoadVariable("val".into(), false, false)));
            instructions.push(ABS.clone());
            Ok(instructions)
        })
    }
}
