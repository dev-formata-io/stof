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

use std::{ops::Deref, sync::Arc};
use web_time::{Duration, SystemTime, UNIX_EPOCH};
use arcstr::{literal, ArcStr};
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use crate::{model::{time::ops::{time_diff, time_diff_ns, time_from_rfc2822, time_from_rfc3339, time_now, time_now_ns, time_now_rfc2822, time_now_rfc3339, time_sleep, time_to_rfc2822, time_to_rfc3339}, Graph}, runtime::{instruction::{Instruction, Instructions}, instructions::Base, proc::ProcEnv, Error, Num, Units, Val, Variable}};
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
        }
    }
}
