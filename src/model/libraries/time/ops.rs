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
        docs: r#"# Time.now() -> ms
Return the current time in milliseconds since the Unix Epoch (unix timestamp).
```rust
const ts = Time.now();
assert(Time.now() >= ts);
```
"#.into(),
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
        docs: r#"# Time.now_ns() -> ns
Return the current time in nanoseconds since the Unix Epoch (unix timestamp).
```rust
const ts = Time.now_ns();
assert(Time.now_ns() >= ts);
```
"#.into(),
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
        docs: r#"# Time.diff(prev: float) -> ms
Convenience function for getting the difference in milliseconds between a previous timestamp (takes any units, default ms) and the current time. Shorthand for (Time.now() - prev).
```rust
const ts = Time.now();
sleep(50ms);
const diff = Time.diff(ts);
assert(diff >= 50ms);
```
"#.into(),
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
        docs: r#"# Time.diff_ns(prev: float) -> ns
Convenience function for getting the difference in nanoseconds between a previous timestamp (takes any units, default ns) and the current time. Shorthand for (Time.now_ns() - prev).
```rust
const ts = Time.now_ns();
sleep(50ms);
const diff = Time.diff_ns(ts);
assert(diff >= 50ms);
```
"#.into(),
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
        docs: r#"# Time.sleep(time: float = 1000ms) -> void
Alias for Std.sleep, instructing this process to sleep for a given amount of time (default units are milliseconds).
```rust
const ts = Time.now();
Time.sleep(50ms); // units make life better here
const diff = Time.diff(ts);
assert(diff >= 50ms);
```
"#.into(),
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
        docs: r#"# Time.now_rfc3339() -> str
Returns a string representing the current time according to the RFC-3339 specefication.
```rust
const now = Time.now_rfc3339();
pln(now); // "2025-08-13T16:22:43.028375200+00:00" when these docs were written
```
"#.into(),
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
        docs: r#"# Time.now_rfc2822() -> str
Returns a string representing the current time according to the RFC-2822 specefication.
```rust
const now = Time.now_rfc2822();
pln(now); // "Wed, 13 Aug 2025 16:24:12 +0000" when these docs were written
```
"#.into(),
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
        docs: r#"# Time.to_rfc3339(time: float) -> str
Returns a string representing the given timestamp according to the RFC-3339 specefication.
```rust
const now = Time.to_rfc3339(Time.now());
pln(now); // "2025-08-13T16:22:43.028375200+00:00" when these docs were written
```
"#.into(),
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
        docs: r#"# Time.to_rfc2822(time: float) -> str
Returns a string representing the given timestamp according to the RFC-2822 specefication.
```rust
const now = Time.to_rfc2822(Time.now());
pln(now); // "Wed, 13 Aug 2025 16:24:12 +0000" when these docs were written
```
"#.into(),
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
        docs: r#"# Time.from_rfc3339(time: str) -> ms
Returns a unix timestamp (milliseconds since Epoch) representing the given RFC-3339 string.
```rust
const ts = Time.from_rfc3339("2025-08-13T16:22:43.028375200+00:00");
assert(ts < Time.now());
```
"#.into(),
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
        docs: r#"# Time.from_rfc2822(time: str) -> ms
Returns a unix timestamp (milliseconds since Epoch) representing the given RFC-2822 string.
```rust
const ts = Time.from_rfc2822("Wed, 13 Aug 2025 16:24:12 +0000");
assert(ts < Time.now());
```
"#.into(),
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
