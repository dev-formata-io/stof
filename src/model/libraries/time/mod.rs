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

use std::{ops::Deref, sync::Arc};
use web_time::{Duration, SystemTime, UNIX_EPOCH};
use arcstr::{literal, ArcStr};
use chrono::{DateTime, Datelike, Days, NaiveDate, Timelike, Utc, Weekday};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use crate::{model::{time::ops::{time_add_days, time_add_months, time_day_of_month, time_day_of_week, time_days_in_month, time_diff, time_diff_ns, time_from_rfc2822, time_from_rfc3339, time_hour, time_minute, time_month, time_now, time_now_ns, time_now_rfc2822, time_now_rfc3339, time_second, time_sleep, time_start_of_day, time_start_of_month, time_start_of_period, time_start_of_week, time_start_of_year, time_to_rfc2822, time_to_rfc3339, time_year}, Graph}, runtime::{instruction::{Instruction, Instructions}, instructions::Base, proc::ProcEnv, Error, Num, Units, Val, Variable}};
mod ops;


/// Library name.
pub(self) const TIME_LIB: ArcStr = literal!("Time");


/// Add the time library to a graph.
pub fn insert_time_lib(graph: &mut Graph) {
    graph.insert_libfunc(time_now());
    graph.insert_libfunc(time_now_ns());
    graph.insert_libfunc(time_diff());
    graph.insert_libfunc(time_diff_ns());
    graph.insert_libfunc(time_sleep());
    graph.insert_libfunc(time_now_rfc3339());
    graph.insert_libfunc(time_now_rfc2822());
    graph.insert_libfunc(time_to_rfc3339());
    graph.insert_libfunc(time_to_rfc2822());
    graph.insert_libfunc(time_from_rfc3339());
    graph.insert_libfunc(time_from_rfc2822());

    // Calendar functions
    graph.insert_libfunc(time_start_of_day());
    graph.insert_libfunc(time_start_of_month());
    graph.insert_libfunc(time_day_of_month());
    graph.insert_libfunc(time_day_of_week());
    graph.insert_libfunc(time_days_in_month());
    graph.insert_libfunc(time_add_months());
    graph.insert_libfunc(time_add_days());
    graph.insert_libfunc(time_start_of_period());

    // Component extraction
    graph.insert_libfunc(time_year());
    graph.insert_libfunc(time_month());
    graph.insert_libfunc(time_hour());
    graph.insert_libfunc(time_minute());
    graph.insert_libfunc(time_second());

    // Start of week
    graph.insert_libfunc(time_start_of_week());
    graph.insert_libfunc(time_start_of_year());
}


lazy_static! {
    pub(self) static ref NOW: Arc<dyn Instruction> = Arc::new(TimeIns::Now);
    pub(self) static ref NOW_NANO: Arc<dyn Instruction> = Arc::new(TimeIns::NowNano);
    pub(self) static ref DIFF: Arc<dyn Instruction> = Arc::new(TimeIns::Diff);
    pub(self) static ref DIFF_NANO: Arc<dyn Instruction> = Arc::new(TimeIns::DiffNano);
    pub(self) static ref SLEEP: Arc<dyn Instruction> = Arc::new(TimeIns::Sleep);
    pub(self) static ref NOW_RFC3339: Arc<dyn Instruction> = Arc::new(TimeIns::NowRFC3339);
    pub(self) static ref NOW_RFC2822: Arc<dyn Instruction> = Arc::new(TimeIns::NowRFC2822);
    pub(self) static ref TO_RFC3339: Arc<dyn Instruction> = Arc::new(TimeIns::ToRFC3339);
    pub(self) static ref TO_RFC2822: Arc<dyn Instruction> = Arc::new(TimeIns::ToRFC2822);
    pub(self) static ref FROM_RFC3339: Arc<dyn Instruction> = Arc::new(TimeIns::FromRFC3339);
    pub(self) static ref FROM_RFC2822: Arc<dyn Instruction> = Arc::new(TimeIns::FromRFC2822);

    // Calendar instructions
    pub(self) static ref START_OF_DAY: Arc<dyn Instruction> = Arc::new(TimeIns::StartOfDay);
    pub(self) static ref START_OF_MONTH: Arc<dyn Instruction> = Arc::new(TimeIns::StartOfMonth);
    pub(self) static ref DAY_OF_MONTH: Arc<dyn Instruction> = Arc::new(TimeIns::DayOfMonth);
    pub(self) static ref DAY_OF_WEEK: Arc<dyn Instruction> = Arc::new(TimeIns::DayOfWeek);
    pub(self) static ref DAYS_IN_MONTH: Arc<dyn Instruction> = Arc::new(TimeIns::DaysInMonth);
    pub(self) static ref ADD_MONTHS: Arc<dyn Instruction> = Arc::new(TimeIns::AddMonths);
    pub(self) static ref ADD_DAYS: Arc<dyn Instruction> = Arc::new(TimeIns::AddDays);
    pub(self) static ref START_OF_PERIOD: Arc<dyn Instruction> = Arc::new(TimeIns::StartOfPeriod);
    pub(self) static ref START_OF_WEEK: Arc<dyn Instruction> = Arc::new(TimeIns::StartOfWeek);
    pub(self) static ref START_OF_YEAR: Arc<dyn Instruction> = Arc::new(TimeIns::StartOfYear);
    pub(self) static ref YEAR: Arc<dyn Instruction> = Arc::new(TimeIns::Year);
    pub(self) static ref MONTH: Arc<dyn Instruction> = Arc::new(TimeIns::Month);
    pub(self) static ref HOUR: Arc<dyn Instruction> = Arc::new(TimeIns::Hour);
    pub(self) static ref MINUTE: Arc<dyn Instruction> = Arc::new(TimeIns::Minute);
    pub(self) static ref SECOND: Arc<dyn Instruction> = Arc::new(TimeIns::Second);
}


/// Parse a ms timestamp from the stack into a chrono UTC DateTime.
fn pop_ms_as_datetime(env: &mut ProcEnv) -> Option<DateTime<Utc>> {
    if let Some(var) = env.stack.pop() {
        match var.val.read().deref() {
            Val::Num(num) => {
                let ms = num.float(Some(Units::Milliseconds)).abs() as i64;
                return DateTime::from_timestamp_millis(ms);
            },
            _ => {}
        }
    }
    None
}

/// Push a DateTime<Utc> back onto the stack as a ms unit value.
fn push_datetime_as_ms(env: &mut ProcEnv, dt: DateTime<Utc>) {
    env.stack.push(Variable::val(Val::Num(Num::Units(dt.timestamp_millis() as f64, Units::Milliseconds))));
}

/// Parse a schedule string and return the start of the current period for a given timestamp.
/// Schedule formats:
///   "monthly:N"          — Nth day of every month (1-28/29/30/31, clamped to month length)
///   "monthly:last"       — Last day of every month
///   "weekly:mon|tue|wed|thu|fri|sat|sun" — Every given weekday
///   "nth_weekday:N:mon|tue|..." — Nth occurrence of weekday in the month (1-4)
///   "yearly:M-D"         — Month M, day D of every year (e.g. "yearly:1-1" for Jan 1)
///   "quarterly:D"        — Day D of the first month of each quarter (Jan/Apr/Jul/Oct)
pub fn start_of_period_for(ts: DateTime<Utc>, schedule: &str) -> Option<DateTime<Utc>> {
    let parts: Vec<&str> = schedule.splitn(3, ':').collect();
    if parts.is_empty() { return None; }

    match parts[0] {
        "monthly" => {
            if parts.len() < 2 { return None; }
            let dim = days_in_month(ts.year(), ts.month());
            let target_day: u32 = if parts[1] == "last" {
                dim
            } else {
                parts[1].parse::<u32>().ok()?.min(dim).max(1)
            };
            // Period start: target_day of this month at midnight UTC
            let candidate = NaiveDate::from_ymd_opt(ts.year(), ts.month(), target_day)?
                .and_hms_opt(0, 0, 0)?
                .and_utc();
            // If we haven't reached the target day yet this month, period started last month
            if ts < candidate {
                let (prev_year, prev_month) = prev_month(ts.year(), ts.month());
                let prev_dim = days_in_month(prev_year, prev_month);
                let prev_day = target_day.min(prev_dim);
                Some(NaiveDate::from_ymd_opt(prev_year, prev_month, prev_day)?
                    .and_hms_opt(0, 0, 0)?
                    .and_utc())
            } else {
                Some(candidate)
            }
        },
        "weekly" => {
            if parts.len() < 2 { return None; }
            let target_wd = parse_weekday(parts[1])?;
            // Walk back from ts to find the most recent occurrence of target_wd
            let mut candidate = ts.date_naive();
            for _ in 0..7 {
                if candidate.weekday() == target_wd {
                    break;
                }
                candidate = candidate.pred_opt()?;
            }
            Some(candidate.and_hms_opt(0, 0, 0)?.and_utc())
        },
        "nth_weekday" => {
            if parts.len() < 3 { return None; }
            let n: u32 = parts[1].parse::<u32>().ok()?.max(1).min(4);
            let target_wd = parse_weekday(parts[2])?;
            // Find the Nth target_wd of this month
            let candidate = nth_weekday_of_month(ts.year(), ts.month(), n, target_wd)?;
            if ts < candidate {
                // Nth weekday this month hasn't arrived yet — use previous month
                let (prev_year, prev_month) = prev_month(ts.year(), ts.month());
                nth_weekday_of_month(prev_year, prev_month, n, target_wd)
            } else {
                Some(candidate)
            }
        },
        "yearly" => {
            // Format: "yearly:M-D" — resets on month M, day D each year
            if parts.len() < 2 { return None; }
            let date_parts: Vec<&str> = parts[1].splitn(2, '-').collect();
            if date_parts.len() < 2 { return None; }
            let target_month: u32 = date_parts[0].parse::<u32>().ok()?.max(1).min(12);
            let target_day: u32 = date_parts[1].parse::<u32>().ok()?.max(1).min(31);
            // Period start: target_month/target_day of this year at midnight UTC, clamped to month length
            let dim = days_in_month(ts.year(), target_month);
            let candidate = NaiveDate::from_ymd_opt(ts.year(), target_month, target_day.min(dim))?
                .and_hms_opt(0, 0, 0)?
                .and_utc();
            if ts < candidate {
                // Anniversary hasn't arrived yet this year — period started last year
                let prev_dim = days_in_month(ts.year() - 1, target_month);
                Some(NaiveDate::from_ymd_opt(ts.year() - 1, target_month, target_day.min(prev_dim))?
                    .and_hms_opt(0, 0, 0)?
                    .and_utc())
            } else {
                Some(candidate)
            }
        },
        "quarterly" => {
            // Format: "quarterly:D" — resets on day D of the first month of each quarter
            // Quarters start: Jan (1), Apr (4), Jul (7), Oct (10)
            if parts.len() < 2 { return None; }
            let target_day: u32 = parts[1].parse::<u32>().ok()?.max(1).min(31);
            let current_quarter_month: u32 = ((ts.month() - 1) / 3) * 3 + 1; // 1, 4, 7, or 10
            let dim = days_in_month(ts.year(), current_quarter_month);
            let day = target_day.min(dim);
            let candidate = NaiveDate::from_ymd_opt(ts.year(), current_quarter_month, day)?
                .and_hms_opt(0, 0, 0)?
                .and_utc();
            if ts < candidate {
                // Haven't reached the start day in this quarter yet — use previous quarter
                let (prev_year, prev_quarter_month) = if current_quarter_month == 1 {
                    (ts.year() - 1, 10u32)
                } else {
                    (ts.year(), current_quarter_month - 3)
                };
                let prev_dim = days_in_month(prev_year, prev_quarter_month);
                Some(NaiveDate::from_ymd_opt(prev_year, prev_quarter_month, target_day.min(prev_dim))?
                    .and_hms_opt(0, 0, 0)?
                    .and_utc())
            } else {
                Some(candidate)
            }
        },
        _ => None,
    }
}

/// Number of days in a given year/month.
fn days_in_month(year: i32, month: u32) -> u32 {
    let next = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    };
    let first = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    (next.unwrap() - first).num_days() as u32
}

/// Previous (year, month).
fn prev_month(year: i32, month: u32) -> (i32, u32) {
    if month == 1 { (year - 1, 12) } else { (year, month - 1) }
}

/// Parse a weekday name to chrono Weekday.
fn parse_weekday(s: &str) -> Option<Weekday> {
    match s.to_lowercase().as_str() {
        "mon" | "monday"    => Some(Weekday::Mon),
        "tue" | "tuesday"   => Some(Weekday::Tue),
        "wed" | "wednesday" => Some(Weekday::Wed),
        "thu" | "thursday"  => Some(Weekday::Thu),
        "fri" | "friday"    => Some(Weekday::Fri),
        "sat" | "saturday"  => Some(Weekday::Sat),
        "sun" | "sunday"    => Some(Weekday::Sun),
        _ => None,
    }
}

/// Find the Nth occurrence of a weekday in a given month, at midnight UTC.
fn nth_weekday_of_month(year: i32, month: u32, n: u32, wd: Weekday) -> Option<DateTime<Utc>> {
    let mut count = 0u32;
    let dim = days_in_month(year, month);
    for day in 1..=dim {
        let d = NaiveDate::from_ymd_opt(year, month, day)?;
        if d.weekday() == wd {
            count += 1;
            if count == n {
                return Some(d.and_hms_opt(0, 0, 0)?.and_utc());
            }
        }
    }
    None
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Time instructions.
pub enum TimeIns {
    Now,
    NowNano,
    Diff,
    DiffNano,
    Sleep,

    NowRFC3339,
    NowRFC2822,
    
    ToRFC3339,
    ToRFC2822,

    FromRFC3339,
    FromRFC2822,

    // Calendar
    StartOfDay,
    StartOfMonth,
    DayOfMonth,
    DayOfWeek,
    DaysInMonth,
    AddMonths,
    AddDays,
    StartOfPeriod,

    // Component extraction
    Year,
    Month,
    Hour,
    Minute,
    Second,

    // Start of week
    StartOfWeek,

    // Start of year
    StartOfYear,
}
#[typetag::serde(name = "TimeIns")]
impl Instruction for TimeIns {
    fn exec(&self, env: &mut ProcEnv, _graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        match self {
            Self::Now => {
                let now = SystemTime::now();
                let dur = now.duration_since(UNIX_EPOCH).unwrap();
                env.stack.push(Variable::val(Val::Num(Num::Units(dur.as_millis() as f64, Units::Milliseconds))));
                Ok(None)
            },
            Self::NowNano => {
                let now = SystemTime::now();
                let dur = now.duration_since(UNIX_EPOCH).unwrap();
                env.stack.push(Variable::val(Val::Num(Num::Units(dur.as_nanos() as f64, Units::Nanoseconds))));
                Ok(None)
            },
            Self::Diff => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Num(num) => {
                            let millis = num.float(Some(Units::Milliseconds));
                            let now = SystemTime::now();
                            let dur = now.duration_since(UNIX_EPOCH).unwrap();
                            env.stack.push(Variable::val(Val::Num(Num::Units((dur.as_millis() as f64) - millis, Units::Milliseconds))));
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::TimeDiff)
            },
            Self::DiffNano => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Num(num) => {
                            let nanos = num.float(Some(Units::Nanoseconds));
                            let now = SystemTime::now();
                            let dur = now.duration_since(UNIX_EPOCH).unwrap();
                            env.stack.push(Variable::val(Val::Num(Num::Units((dur.as_nanos() as f64) - nanos, Units::Nanoseconds))));
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::TimeDiffNano)
            },
            Self::Sleep => {
                let duration;
                if let Some(val) = env.stack.pop() {
                    if let Some(num) = val.val.write().try_num() {
                        duration = num.float(Some(Units::Milliseconds));
                    } else {
                        return Err(Error::TimeSleep);
                    }
                } else {
                    return Err(Error::TimeSleep);
                }

                let mut instructions = Instructions::default();
                instructions.push(Arc::new(Base::CtrlSleepFor(Duration::from_millis(duration.abs() as u64))));
                return Ok(Some(instructions));
            },
            Self::NowRFC3339 => {
                let now: DateTime<Utc> = DateTime::from_timestamp_millis(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64).unwrap();
                env.stack.push(Variable::val(Val::Str(now.to_rfc3339().into())));
                Ok(None)
            },
            Self::NowRFC2822 => {
                let now: DateTime<Utc> = DateTime::from_timestamp_millis(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64).unwrap();
                env.stack.push(Variable::val(Val::Str(now.to_rfc2822().into())));
                Ok(None)
            },
            Self::ToRFC3339 => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Num(num) => {
                            let ms = num.float(Some(Units::Milliseconds)).abs() as i64;
                            if let Some(time) = DateTime::from_timestamp_millis(ms) {
                                env.stack.push(Variable::val(Val::Str(time.to_rfc3339().into())));
                                return Ok(None);
                            }
                        },
                        _ => {}
                    }
                }
                Err(Error::TimeToRFC3339)
            },
            Self::ToRFC2822 => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Num(num) => {
                            let ms = num.float(Some(Units::Milliseconds)).abs() as i64;
                            if let Some(time) = DateTime::from_timestamp_millis(ms) {
                                env.stack.push(Variable::val(Val::Str(time.to_rfc2822().into())));
                                return Ok(None);
                            }
                        },
                        _ => {}
                    }
                }
                Err(Error::TimeToRFC2822)
            },
            Self::FromRFC3339 => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Str(val) => {
                            if let Ok(res) = DateTime::parse_from_rfc3339(val.as_str()) {
                                let milli = res.timestamp_millis();
                                env.stack.push(Variable::val(Val::Num(Num::Units(milli as f64, Units::Milliseconds))));
                                return Ok(None);
                            }
                        },
                        _ => {}
                    }
                }
                Err(Error::TimeFromRFC3339)
            },
            Self::FromRFC2822 => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Str(val) => {
                            if let Ok(res) = DateTime::parse_from_rfc2822(val.as_str()) {
                                let milli = res.timestamp_millis();
                                env.stack.push(Variable::val(Val::Num(Num::Units(milli as f64, Units::Milliseconds))));
                                return Ok(None);
                            }
                        },
                        _ => {}
                    }
                }
                Err(Error::TimeFromRFC2822)
            },

            // ── Calendar instructions ──────────────────────────────────────

            Self::StartOfDay => {
                if let Some(dt) = pop_ms_as_datetime(env) {
                    let sod = dt.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
                    push_datetime_as_ms(env, sod);
                    return Ok(None);
                }
                Err(Error::Custom("Time.start_of_day: invalid timestamp".into()))
            },
            Self::StartOfMonth => {
                if let Some(dt) = pop_ms_as_datetime(env) {
                    if let Some(som) = NaiveDate::from_ymd_opt(dt.year(), dt.month(), 1)
                        .and_then(|d| d.and_hms_opt(0, 0, 0))
                        .map(|ndt| ndt.and_utc())
                    {
                        push_datetime_as_ms(env, som);
                        return Ok(None);
                    }
                }
                Err(Error::Custom("Time.start_of_month: invalid timestamp".into()))
            },
            Self::DayOfMonth => {
                if let Some(dt) = pop_ms_as_datetime(env) {
                    env.stack.push(Variable::val(Val::Num(Num::Int(dt.day() as i64))));
                    return Ok(None);
                }
                Err(Error::Custom("Time.day_of_month: invalid timestamp".into()))
            },
            Self::DayOfWeek => {
                // Returns 0=Mon..6=Sun (ISO weekday - 1)
                if let Some(dt) = pop_ms_as_datetime(env) {
                    let dow = dt.weekday().num_days_from_monday() as i64;
                    env.stack.push(Variable::val(Val::Num(Num::Int(dow))));
                    return Ok(None);
                }
                Err(Error::Custom("Time.day_of_week: invalid timestamp".into()))
            },
            Self::DaysInMonth => {
                if let Some(dt) = pop_ms_as_datetime(env) {
                    let dim = days_in_month(dt.year(), dt.month());
                    env.stack.push(Variable::val(Val::Num(Num::Int(dim as i64))));
                    return Ok(None);
                }
                Err(Error::Custom("Time.days_in_month: invalid timestamp".into()))
            },
            Self::AddMonths => {
                // Stack: ts (bottom), n (top)
                let n = if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Num(num) => num.float(None) as i32,
                        _ => return Err(Error::Custom("Time.add_months: n must be a number".into())),
                    }
                } else {
                    return Err(Error::Custom("Time.add_months: missing n".into()));
                };
                if let Some(dt) = pop_ms_as_datetime(env) {
                    let mut year = dt.year();
                    let mut month = dt.month() as i32 + n;
                    while month > 12 { month -= 12; year += 1; }
                    while month < 1  { month += 12; year -= 1; }
                    let dim = days_in_month(year, month as u32);
                    let day = dt.day().min(dim);
                    if let Some(result) = NaiveDate::from_ymd_opt(year, month as u32, day)
                        .and_then(|d| d.and_hms_opt(dt.hour(), dt.minute(), dt.second()))
                        .map(|ndt| ndt.and_utc())
                    {
                        push_datetime_as_ms(env, result);
                        return Ok(None);
                    }
                }
                Err(Error::Custom("Time.add_months: invalid arguments".into()))
            },
            Self::AddDays => {
                // Stack: ts (bottom), n (top)
                let n = if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Num(num) => num.float(None) as i64,
                        _ => return Err(Error::Custom("Time.add_days: n must be a number".into())),
                    }
                } else {
                    return Err(Error::Custom("Time.add_days: missing n".into()));
                };
                if let Some(dt) = pop_ms_as_datetime(env) {
                    let result = if n >= 0 {
                        dt.checked_add_days(Days::new(n as u64))
                    } else {
                        dt.checked_sub_days(Days::new((-n) as u64))
                    };
                    if let Some(r) = result {
                        push_datetime_as_ms(env, r);
                        return Ok(None);
                    }
                }
                Err(Error::Custom("Time.add_days: invalid arguments".into()))
            },
            Self::StartOfPeriod => {
                // Stack: ts (bottom), schedule str (top)
                let schedule = if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Str(s) => s.to_string(),
                        _ => return Err(Error::Custom("Time.start_of_period: schedule must be a string".into())),
                    }
                } else {
                    return Err(Error::Custom("Time.start_of_period: missing schedule".into()));
                };
                if let Some(dt) = pop_ms_as_datetime(env) {
                    if let Some(period_start) = start_of_period_for(dt, &schedule) {
                        push_datetime_as_ms(env, period_start);
                        return Ok(None);
                    }
                    return Err(Error::Custom(format!("Time.start_of_period: could not compute period for schedule '{}'", schedule).into()));
                }
                Err(Error::Custom("Time.start_of_period: invalid timestamp".into()))
            },

            // ── Component extraction ───────────────────────────────────────

            Self::Year => {
                if let Some(dt) = pop_ms_as_datetime(env) {
                    env.stack.push(Variable::val(Val::Num(Num::Int(dt.year() as i64))));
                    return Ok(None);
                }
                Err(Error::Custom("Time.year: invalid timestamp".into()))
            },
            Self::Month => {
                // Returns 1–12
                if let Some(dt) = pop_ms_as_datetime(env) {
                    env.stack.push(Variable::val(Val::Num(Num::Int(dt.month() as i64))));
                    return Ok(None);
                }
                Err(Error::Custom("Time.month: invalid timestamp".into()))
            },
            Self::Hour => {
                if let Some(dt) = pop_ms_as_datetime(env) {
                    env.stack.push(Variable::val(Val::Num(Num::Int(dt.hour() as i64))));
                    return Ok(None);
                }
                Err(Error::Custom("Time.hour: invalid timestamp".into()))
            },
            Self::Minute => {
                if let Some(dt) = pop_ms_as_datetime(env) {
                    env.stack.push(Variable::val(Val::Num(Num::Int(dt.minute() as i64))));
                    return Ok(None);
                }
                Err(Error::Custom("Time.minute: invalid timestamp".into()))
            },
            Self::Second => {
                if let Some(dt) = pop_ms_as_datetime(env) {
                    env.stack.push(Variable::val(Val::Num(Num::Int(dt.second() as i64))));
                    return Ok(None);
                }
                Err(Error::Custom("Time.second: invalid timestamp".into()))
            },

            // ── Start of week ──────────────────────────────────────────────

            Self::StartOfWeek => {
                // Stack: ts (bottom), start_day int (top) — 0=Mon (default), 6=Sun
                let start_day = if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Num(num) => (num.float(None) as u32).min(6),
                        _ => return Err(Error::Custom("Time.start_of_week: start_day must be a number".into())),
                    }
                } else {
                    return Err(Error::Custom("Time.start_of_week: missing start_day".into()));
                };
                if let Some(dt) = pop_ms_as_datetime(env) {
                    // ISO: Mon=0..Sun=6 via num_days_from_monday
                    let current_dow = dt.weekday().num_days_from_monday();
                    // How many days back to reach start_day?
                    let days_back = (current_dow + 7 - start_day) % 7;
                    let sow = dt.date_naive()
                        .checked_sub_days(Days::new(days_back as u64))
                        .and_then(|d| d.and_hms_opt(0, 0, 0))
                        .map(|ndt| ndt.and_utc());
                    if let Some(result) = sow {
                        push_datetime_as_ms(env, result);
                        return Ok(None);
                    }
                }
                Err(Error::Custom("Time.start_of_week: invalid arguments".into()))
            },

            // ── Start of year ──────────────────────────────────────────────

            Self::StartOfYear => {
                if let Some(dt) = pop_ms_as_datetime(env) {
                    if let Some(soy) = NaiveDate::from_ymd_opt(dt.year(), 1, 1)
                        .and_then(|d| d.and_hms_opt(0, 0, 0))
                        .map(|ndt| ndt.and_utc())
                    {
                        push_datetime_as_ms(env, soy);
                        return Ok(None);
                    }
                }
                Err(Error::Custom("Time.start_of_year: invalid timestamp".into()))
            },
        }
    }
}
