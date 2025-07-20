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
use crate::{model::{time::{DIFF, DIFF_NANO, FROM_RFC2822, FROM_RFC3339, NOW, NOW_NANO, NOW_RFC2822, NOW_RFC3339, SLEEP, TIME_LIB, TO_RFC2822, TO_RFC3339}, LibFunc, Param}, runtime::{instruction::Instructions, instructions::Base, Num, NumT, Type, Val}};


/// Now.
pub fn time_now() -> LibFunc {
    LibFunc {
        library: TIME_LIB.clone(),
        name: "now".into(),
        is_async: false,
        docs: "# Now\nReturns the current time in milliseconds since the Unix Epoch.".into(),
        params: vector![],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(NOW.clone());
            Ok(instructions)
        })
    }
}

/// Now nanos.
pub fn time_now_ns() -> LibFunc {
    LibFunc {
        library: TIME_LIB.clone(),
        name: "now_ns".into(),
        is_async: false,
        docs: "# Now (Nanoseconds)\nReturns the current time in nanoseconds since the Unix Epoch.".into(),
        params: vector![],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(NOW_NANO.clone());
            Ok(instructions)
        })
    }
}

/// Diff.
pub fn time_diff() -> LibFunc {
    LibFunc {
        library: TIME_LIB.clone(),
        name: "diff".into(),
        is_async: false,
        docs: "# Difference\nReturns the difference in time (milliseconds) from a previous timestamp (now - prev).".into(),
        params: vector![
            Param { name: "prev".into(), param_type: Type::Num(NumT::Float), default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(DIFF.clone());
            Ok(instructions)
        })
    }
}

/// Diff nanos.
pub fn time_diff_ns() -> LibFunc {
    LibFunc {
        library: TIME_LIB.clone(),
        name: "diff_ns".into(),
        is_async: false,
        docs: "# Difference (Nanoseconds)\nReturns the difference in time (nanoseconds) from a previous timestamp (now - prev).".into(),
        params: vector![
            Param { name: "prev".into(), param_type: Type::Num(NumT::Float), default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(DIFF_NANO.clone());
            Ok(instructions)
        })
    }
}

/// Sleep (same as std).
pub fn time_sleep() -> LibFunc {
    LibFunc {
        library: TIME_LIB.clone(),
        name: "sleep".into(),
        is_async: false,
        docs: "# Put Process to Sleep\nInstruct this process to sleep for an amount of time. Use time units for specificity (Ex. Time.sleep(200ms)).".into(),
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

/// Now RFC3339.
pub fn time_now_rfc3339() -> LibFunc {
    LibFunc {
        library: TIME_LIB.clone(),
        name: "now_rfc3339".into(),
        is_async: false,
        docs: "# Now RFC-3339\nReturns a string representing the current time according the the RFC-3339 specification.".into(),
        params: vector![],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(NOW_RFC3339.clone());
            Ok(instructions)
        })
    }
}

/// Now RFC2822.
pub fn time_now_rfc2822() -> LibFunc {
    LibFunc {
        library: TIME_LIB.clone(),
        name: "now_rfc2822".into(),
        is_async: false,
        docs: "# Now RFC-2822\nReturns a string representing the current time according the the RFC-2822 specification.".into(),
        params: vector![],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(NOW_RFC2822.clone());
            Ok(instructions)
        })
    }
}

/// To RFC3339.
pub fn time_to_rfc3339() -> LibFunc {
    LibFunc {
        library: TIME_LIB.clone(),
        name: "to_rfc3339".into(),
        is_async: false,
        docs: "# To RFC-3339\nReturns a string representing the given time (unix timestamp) according to RFC-3339.".into(),
        params: vector![
            Param { name: "time".into(), param_type: Type::Num(NumT::Float), default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(TO_RFC3339.clone());
            Ok(instructions)
        })
    }
}

/// To RFC2822.
pub fn time_to_rfc2822() -> LibFunc {
    LibFunc {
        library: TIME_LIB.clone(),
        name: "to_rfc2822".into(),
        is_async: false,
        docs: "# To RFC-2822\nReturns a string representing the given time (unix timestamp) according to RFC-2822.".into(),
        params: vector![
            Param { name: "time".into(), param_type: Type::Num(NumT::Float), default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(TO_RFC2822.clone());
            Ok(instructions)
        })
    }
}

/// From RFC3339.
pub fn time_from_rfc3339() -> LibFunc {
    LibFunc {
        library: TIME_LIB.clone(),
        name: "from_rfc3339".into(),
        is_async: false,
        docs: "# From RFC-3339\nReturns a unix timestamp (milliseconds since epoch) representing the given time (string) according to RFC-3339.".into(),
        params: vector![
            Param { name: "time".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(FROM_RFC3339.clone());
            Ok(instructions)
        })
    }
}

/// From RFC2822.
pub fn time_from_rfc2822() -> LibFunc {
    LibFunc {
        library: TIME_LIB.clone(),
        name: "from_rfc2822".into(),
        is_async: false,
        docs: "# From RFC-2822\nReturns a unix timestamp (milliseconds since epoch) representing the given time (string) according to RFC-2822.".into(),
        params: vector![
            Param { name: "time".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(FROM_RFC2822.clone());
            Ok(instructions)
        })
    }
}
