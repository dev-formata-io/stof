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

use std::{ops::Deref, time::{Duration, SystemTime, UNIX_EPOCH}};
use chrono::{DateTime, Utc};
use crate::{lang::SError, Library, SDoc, SNum, SUnits, SVal};


/// Time library.
#[derive(Default, Debug)]
pub struct TimeLibrary;
impl Library for TimeLibrary {
    fn scope(&self) -> String {
        "Time".to_string()
    }

    fn call(&self, pid: &str, doc: &mut SDoc, name: &str, parameters: &mut Vec<SVal>) -> Result<SVal, SError> {
        match name {
            // Number of milliseconds since the unix epoch.
            "now" => {
                let now = SystemTime::now();
                let dur = now.duration_since(UNIX_EPOCH).unwrap();
                Ok(SVal::Number(SNum::Units(dur.as_millis() as f64, SUnits::Milliseconds)))
            },
            // Number of nanoseconds since the unix epoch.
            "nowNano" => {
                let now = SystemTime::now();
                let dur = now.duration_since(UNIX_EPOCH).unwrap();
                Ok(SVal::Number(SNum::Units(dur.as_nanos() as f64, SUnits::Nanoseconds)))
            },
            // Diff in milliseconds from another time instant (now) since the unix epoch.
            // If no units, it's assumed to be milliseconds.
            "diff" => {
                if parameters.len() < 1 {
                    return Err(SError::time(pid, &doc, "diff", "expecting a number to diff with"));
                }
                match &parameters[0] {
                    SVal::Number(num) => {
                        let prev = num.float_with_units(SUnits::Milliseconds);
                        let now = SystemTime::now();
                        let dur = now.duration_since(UNIX_EPOCH).unwrap();
                        return Ok(SVal::Number(SNum::Units((dur.as_millis() as f64) - prev, SUnits::Milliseconds)));
                    },
                    SVal::Boxed(val) => {
                        let val = val.lock().unwrap();
                        let val = val.deref();
                        match val {
                            SVal::Number(num) => {
                                let prev = num.float_with_units(SUnits::Milliseconds);
                                let now = SystemTime::now();
                                let dur = now.duration_since(UNIX_EPOCH).unwrap();
                                return Ok(SVal::Number(SNum::Units((dur.as_millis() as f64) - prev, SUnits::Milliseconds)));
                            },
                            _ => {}
                        }
                    },
                    _ => {}
                }
                Err(SError::time(pid, &doc, "diff", "expecting a number to diff with"))
            },
            // Diff in nanoseconds from another time instant (nowNano) since the unix epoch.
            // If no units, it's assumed to be nanoseconds.
            "diffNano" => {
                if parameters.len() < 1 {
                    return Err(SError::time(pid, &doc, "diffNano", "expecting a number to diff with"));
                }
                match &parameters[0] {
                    SVal::Number(num) => {
                        let prev = num.float_with_units(SUnits::Nanoseconds);
                        let now = SystemTime::now();
                        let dur = now.duration_since(UNIX_EPOCH).unwrap();
                        return Ok(SVal::Number(SNum::Units((dur.as_nanos() as f64) - prev, SUnits::Nanoseconds)));
                    },
                    SVal::Boxed(val) => {
                        let val = val.lock().unwrap();
                        let val = val.deref();
                        match val {
                            SVal::Number(num) => {
                                let prev = num.float_with_units(SUnits::Nanoseconds);
                                let now = SystemTime::now();
                                let dur = now.duration_since(UNIX_EPOCH).unwrap();
                                return Ok(SVal::Number(SNum::Units((dur.as_nanos() as f64) - prev, SUnits::Nanoseconds)));
                            },
                            _ => {}
                        }
                    },
                    _ => {}
                }
                Err(SError::time(pid, &doc, "diffNano", "expecting a number to diff with"))
            },
            // Sleep for a certain amount of time.
            // This is blocking and should be used carefully, especially if/when we introduce an async runtime.
            // If no units, then it's assumed to be sleeping in milliseconds.
            "sleep" => {
                if parameters.len() < 1 {
                    return Err(SError::time(pid, &doc, "sleep", "expecting a number to use for sleep"));
                }
                match &parameters[0] {
                    SVal::Number(num) => {
                        let mut dur = num.float_with_units(SUnits::Milliseconds);
                        if dur < 0. { dur *= -1.; }
                        let dur = Duration::from_millis(dur as u64);
                        std::thread::sleep(dur);
                        return Ok(SVal::Void);
                    },
                    SVal::Boxed(val) => {
                        let val = val.lock().unwrap();
                        let val = val.deref();
                        match val {
                            SVal::Number(num) => {
                                let mut dur = num.float_with_units(SUnits::Milliseconds);
                                if dur < 0. { dur *= -1.; }
                                let dur = Duration::from_millis(dur as u64);
                                std::thread::sleep(dur);
                                return Ok(SVal::Void);
                            },
                            _ => {}
                        }
                    },
                    _ => {}
                }
                Err(SError::time(pid, &doc, "sleep", "expecting a number to use for sleep"))
            },
            // Sleep for a certain amount of time.
            // This is blocking and should be used carefully, especially if/when we introduce an async runtime.
            // If no units, then it's assumed to be sleeping in nanoseconds.
            "sleepNano" => {
                if parameters.len() < 1 {
                    return Err(SError::time(pid, &doc, "sleepNano", "expecting a number to use for sleep"));
                }
                match &parameters[0] {
                    SVal::Number(num) => {
                        let mut dur = num.float_with_units(SUnits::Nanoseconds);
                        if dur < 0. { dur *= -1.; }
                        let dur = Duration::from_nanos(dur as u64);
                        std::thread::sleep(dur);
                        return Ok(SVal::Void);
                    },
                    SVal::Boxed(val) => {
                        let val = val.lock().unwrap();
                        let val = val.deref();
                        match val {
                            SVal::Number(num) => {
                                let mut dur = num.float_with_units(SUnits::Nanoseconds);
                                if dur < 0. { dur *= -1.; }
                                let dur = Duration::from_nanos(dur as u64);
                                std::thread::sleep(dur);
                                return Ok(SVal::Void);
                            },
                            _ => {}
                        }
                    },
                    _ => {}
                }
                Err(SError::time(pid, &doc, "sleepNano", "expecting a number to use for sleep"))
            },

            /*****************************************************************************
             * RFC 3339.
             *****************************************************************************/
            // Return an RFC 3339 string representing the current time.
            "nowRFC3339" => {
                let now: DateTime<Utc> = DateTime::from(SystemTime::now());
                Ok(SVal::String(now.to_rfc3339()))
            },
            // Return an RFC 2822 string representing the current time.
            "nowRFC2822" => {
                let now: DateTime<Utc> = DateTime::from(SystemTime::now());
                Ok(SVal::String(now.to_rfc2822()))
            },
            // Return an RFC 3339 string representing the given Unix timestamp (time since unix epoch, millisecond accuracy).
            "toRFC3339" => {
                if parameters.len() > 0 {
                    match &parameters[0] {
                        SVal::Number(num) => {
                            let ms = num.float_with_units(SUnits::Milliseconds).abs() as i64;
                            if let Some(time) = DateTime::from_timestamp_millis(ms) {
                                return Ok(SVal::String(time.to_rfc3339()));
                            }
                        },
                        SVal::Boxed(val) => {
                            let val = val.lock().unwrap();
                            let val = val.deref();
                            match val {
                                SVal::Number(num) => {
                                    let ms = num.float_with_units(SUnits::Milliseconds).abs() as i64;
                                    if let Some(time) = DateTime::from_timestamp_millis(ms) {
                                        return Ok(SVal::String(time.to_rfc3339()));
                                    }
                                },
                                _ => {}
                            }
                        },
                        _ => {}
                    }
                }
                Err(SError::time(pid, &doc, "toRFC3339", "expecting a timestamp to convert"))
            },
            // Return an RFC 2822 string representing the given Unix timestamp (time since unix epoch, millisecond accuracy).
            "toRFC2822" => {
                if parameters.len() > 0 {
                    match &parameters[0] {
                        SVal::Number(num) => {
                            let ms = num.float_with_units(SUnits::Milliseconds).abs() as i64;
                            if let Some(time) = DateTime::from_timestamp_millis(ms) {
                                return Ok(SVal::String(time.to_rfc2822()));
                            }
                        },
                        SVal::Boxed(val) => {
                            let val = val.lock().unwrap();
                            let val = val.deref();
                            match val {
                                SVal::Number(num) => {
                                    let ms = num.float_with_units(SUnits::Milliseconds).abs() as i64;
                                    if let Some(time) = DateTime::from_timestamp_millis(ms) {
                                        return Ok(SVal::String(time.to_rfc2822()));
                                    }
                                },
                                _ => {}
                            }
                        },
                        _ => {}
                    }
                }
                Err(SError::time(pid, &doc, "toRFC2822", "expecting a timestamp to convert"))
            },
            // Parse an RFC 3339 string into a unix timestamp (milliseconds since unix epoch).
            "parseRFC3339" => {
                if parameters.len() > 0 {
                    let str = parameters[0].to_string();
                    if let Ok(res) = DateTime::parse_from_rfc3339(&str) {
                        let milli = res.timestamp_millis();
                        return Ok(SVal::Number(SNum::Units(milli as f64, SUnits::Milliseconds)));
                    }
                }
                Err(SError::time(pid, &doc, "parseRFC3339", "expecting a valid RFC 3339 string to parse"))
            },
            // Parse an RFC 2822 string into a unix timestamp (milliseconds since unix epoch).
            "parseRFC2822" => {
                if parameters.len() > 0 {
                    let str = parameters[0].to_string();
                    if let Ok(res) = DateTime::parse_from_rfc2822(&str) {
                        let milli = res.timestamp_millis();
                        return Ok(SVal::Number(SNum::Units(milli as f64, SUnits::Milliseconds)));
                    }
                }
                Err(SError::time(pid, &doc, "parseRFC2822", "expecting a valid RFC 2822 string to parse"))
            },
            _ => {
                Err(SError::time(pid, &doc, "NotFound", &format!("{} is not a function in the Time Library", name)))
            }
        }
    }
}
