//
// Copyright 2025 Formata, Inc. All rights reserved.
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
use crate::{model::{time::{ADD_DAYS, ADD_MONTHS, DAY_OF_MONTH, DAY_OF_WEEK, DAYS_IN_MONTH, DIFF, DIFF_NANO, FROM_RFC2822, FROM_RFC3339, HOUR, MINUTE, MONTH, NOW, NOW_NANO, NOW_RFC2822, NOW_RFC3339, SECOND, SLEEP, START_OF_DAY, START_OF_MONTH, START_OF_PERIOD, START_OF_WEEK, TIME_LIB, TO_RFC2822, TO_RFC3339, YEAR}, LibFunc, Param}, runtime::{instruction::Instructions, instructions::Base, Num, NumT, Type, Val}};


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
/// Start of day (UTC midnight).
pub fn time_start_of_day() -> LibFunc {
    LibFunc {
        library: TIME_LIB.clone(),
        name: "start_of_day".into(),
        is_async: false,
        docs: r#"# Time.start_of_day(ts: ms) -> ms
Returns the UTC midnight timestamp for the day containing the given timestamp.
```rust
const today = Time.start_of_day(Time.now());
assert(today <= Time.now());
```
"#.into(),
        params: vector![
            Param { name: "ts".into(), param_type: Type::Num(NumT::Float), default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(START_OF_DAY.clone());
            Ok(instructions)
        })
    }
}

/// Start of month (UTC midnight on the 1st).
pub fn time_start_of_month() -> LibFunc {
    LibFunc {
        library: TIME_LIB.clone(),
        name: "start_of_month".into(),
        is_async: false,
        docs: r#"# Time.start_of_month(ts: ms) -> ms
Returns the UTC midnight timestamp for the first day of the month containing the given timestamp.
```rust
const som = Time.start_of_month(Time.now());
assert(som <= Time.now());
assert(Time.day_of_month(som) == 1);
```
"#.into(),
        params: vector![
            Param { name: "ts".into(), param_type: Type::Num(NumT::Float), default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(START_OF_MONTH.clone());
            Ok(instructions)
        })
    }
}

/// Day of month (1–31).
pub fn time_day_of_month() -> LibFunc {
    LibFunc {
        library: TIME_LIB.clone(),
        name: "day_of_month".into(),
        is_async: false,
        docs: r#"# Time.day_of_month(ts: ms) -> int
Returns the day of the month (1–31) for the given UTC timestamp.
```rust
const dom = Time.day_of_month(Time.now());
assert(dom >= 1 && dom <= 31);
```
"#.into(),
        params: vector![
            Param { name: "ts".into(), param_type: Type::Num(NumT::Float), default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(DAY_OF_MONTH.clone());
            Ok(instructions)
        })
    }
}

/// Day of week (0=Mon..6=Sun, ISO).
pub fn time_day_of_week() -> LibFunc {
    LibFunc {
        library: TIME_LIB.clone(),
        name: "day_of_week".into(),
        is_async: false,
        docs: r#"# Time.day_of_week(ts: ms) -> int
Returns the ISO day of the week for the given UTC timestamp.
0 = Monday, 1 = Tuesday, ..., 6 = Sunday.
```rust
const dow = Time.day_of_week(Time.now());
assert(dow >= 0 && dow <= 6);
```
"#.into(),
        params: vector![
            Param { name: "ts".into(), param_type: Type::Num(NumT::Float), default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(DAY_OF_WEEK.clone());
            Ok(instructions)
        })
    }
}

/// Days in month.
pub fn time_days_in_month() -> LibFunc {
    LibFunc {
        library: TIME_LIB.clone(),
        name: "days_in_month".into(),
        is_async: false,
        docs: r#"# Time.days_in_month(ts: ms) -> int
Returns the number of days in the month containing the given UTC timestamp.
Correctly handles leap years for February.
```rust
const dim = Time.days_in_month(Time.now());
assert(dim >= 28 && dim <= 31);
```
"#.into(),
        params: vector![
            Param { name: "ts".into(), param_type: Type::Num(NumT::Float), default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(DAYS_IN_MONTH.clone());
            Ok(instructions)
        })
    }
}

/// Add months (calendar-aware).
pub fn time_add_months() -> LibFunc {
    LibFunc {
        library: TIME_LIB.clone(),
        name: "add_months".into(),
        is_async: false,
        docs: r#"# Time.add_months(ts: ms, n: int) -> ms
Adds n calendar months to the given UTC timestamp.
The day of month is clamped to the last day of the target month if necessary
(e.g. Jan 31 + 1 month = Feb 28/29).
```rust
const next = Time.add_months(Time.now(), 1);
assert(next > Time.now());
```
"#.into(),
        params: vector![
            Param { name: "ts".into(), param_type: Type::Num(NumT::Float), default: None },
            Param { name: "n".into(), param_type: Type::Num(NumT::Float), default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(ADD_MONTHS.clone());
            Ok(instructions)
        })
    }
}

/// Add days.
pub fn time_add_days() -> LibFunc {
    LibFunc {
        library: TIME_LIB.clone(),
        name: "add_days".into(),
        is_async: false,
        docs: r#"# Time.add_days(ts: ms, n: int) -> ms
Adds n calendar days to the given UTC timestamp. n may be negative.
```rust
const tomorrow = Time.add_days(Time.now(), 1);
assert(tomorrow > Time.now());
const yesterday = Time.add_days(Time.now(), -1);
assert(yesterday < Time.now());
```
"#.into(),
        params: vector![
            Param { name: "ts".into(), param_type: Type::Num(NumT::Float), default: None },
            Param { name: "n".into(), param_type: Type::Num(NumT::Float), default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(ADD_DAYS.clone());
            Ok(instructions)
        })
    }
}

/// Start of period for a calendar schedule.
pub fn time_start_of_period() -> LibFunc {
    LibFunc {
        library: TIME_LIB.clone(),
        name: "start_of_period".into(),
        is_async: false,
        docs: r#"# Time.start_of_period(ts: ms, schedule: str) -> ms
Returns the start of the current billing/reset period for the given UTC timestamp
and schedule expression. Returns null if the schedule is invalid.

Schedule formats:
  "monthly:N"             — Nth day of every month (1–31, clamped to month length)
  "monthly:last"          — Last day of every month
  "weekly:mon"            — Every Monday (mon|tue|wed|thu|fri|sat|sun)
  "nth_weekday:N:mon"     — Nth occurrence of weekday in the month (N = 1–4)

All times are UTC. The returned timestamp is always midnight UTC on the period start day.

```rust
// Resets on the 1st of every month
const period = Time.start_of_period(Time.now(), 'monthly:1');
assert(period <= Time.now());

// Resets every Monday
const week_start = Time.start_of_period(Time.now(), 'weekly:mon');
assert(week_start <= Time.now());

// Resets on the first Tuesday of every month
const billing = Time.start_of_period(Time.now(), 'nth_weekday:1:tue');
assert(billing <= Time.now());
```
"#.into(),
        params: vector![
            Param { name: "ts".into(), param_type: Type::Num(NumT::Float), default: None },
            Param { name: "schedule".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(START_OF_PERIOD.clone());
            Ok(instructions)
        })
    }
}

/// Year component.
pub fn time_year() -> LibFunc {
    LibFunc {
        library: TIME_LIB.clone(),
        name: "year".into(),
        is_async: false,
        docs: r#"# Time.year(ts: ms) -> int
Returns the UTC year for the given timestamp.
```rust
const y = Time.year(Time.now());
assert(y >= 2025);
```
"#.into(),
        params: vector![
            Param { name: "ts".into(), param_type: Type::Num(NumT::Float), default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(YEAR.clone());
            Ok(instructions)
        })
    }
}

/// Month component (1–12).
pub fn time_month() -> LibFunc {
    LibFunc {
        library: TIME_LIB.clone(),
        name: "month".into(),
        is_async: false,
        docs: r#"# Time.month(ts: ms) -> int
Returns the UTC month (1–12) for the given timestamp.
```rust
const m = Time.month(Time.now());
assert(m >= 1 && m <= 12);
```
"#.into(),
        params: vector![
            Param { name: "ts".into(), param_type: Type::Num(NumT::Float), default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(MONTH.clone());
            Ok(instructions)
        })
    }
}

/// Hour component (0–23).
pub fn time_hour() -> LibFunc {
    LibFunc {
        library: TIME_LIB.clone(),
        name: "hour".into(),
        is_async: false,
        docs: r#"# Time.hour(ts: ms) -> int
Returns the UTC hour (0–23) for the given timestamp.
```rust
const h = Time.hour(Time.now());
assert(h >= 0 && h <= 23);
```
"#.into(),
        params: vector![
            Param { name: "ts".into(), param_type: Type::Num(NumT::Float), default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(HOUR.clone());
            Ok(instructions)
        })
    }
}

/// Minute component (0–59).
pub fn time_minute() -> LibFunc {
    LibFunc {
        library: TIME_LIB.clone(),
        name: "minute".into(),
        is_async: false,
        docs: r#"# Time.minute(ts: ms) -> int
Returns the UTC minute (0–59) for the given timestamp.
```rust
const m = Time.minute(Time.now());
assert(m >= 0 && m <= 59);
```
"#.into(),
        params: vector![
            Param { name: "ts".into(), param_type: Type::Num(NumT::Float), default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(MINUTE.clone());
            Ok(instructions)
        })
    }
}

/// Second component (0–59).
pub fn time_second() -> LibFunc {
    LibFunc {
        library: TIME_LIB.clone(),
        name: "second".into(),
        is_async: false,
        docs: r#"# Time.second(ts: ms) -> int
Returns the UTC second (0–59) for the given timestamp.
```rust
const s = Time.second(Time.now());
assert(s >= 0 && s <= 59);
```
"#.into(),
        params: vector![
            Param { name: "ts".into(), param_type: Type::Num(NumT::Float), default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(SECOND.clone());
            Ok(instructions)
        })
    }
}

/// Start of week.
pub fn time_start_of_week() -> LibFunc {
    LibFunc {
        library: TIME_LIB.clone(),
        name: "start_of_week".into(),
        is_async: false,
        docs: r#"# Time.start_of_week(ts: ms, start_day: int = 0) -> ms
Returns the UTC midnight timestamp for the start of the week containing the given timestamp.
start_day follows ISO convention: 0 = Monday (default), 6 = Sunday.
```rust
const sow = Time.start_of_week(Time.now());       // week starting Monday
assert(sow <= Time.now());

const sun = Time.start_of_week(Time.now(), 6);    // week starting Sunday
assert(sun <= Time.now());
```
"#.into(),
        params: vector![
            Param { name: "ts".into(), param_type: Type::Num(NumT::Float), default: None },
            Param { name: "start_day".into(), param_type: Type::Num(NumT::Float), default: Some(Arc::new(Base::Literal(Val::Num(Num::Int(0))))) }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(START_OF_WEEK.clone());
            Ok(instructions)
        })
    }
}
