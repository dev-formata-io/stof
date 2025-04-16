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


use core::str;
use std::ops::DerefMut;
use crate::{lang::SError, Library, SDoc, SNum, SVal};


/// SemVer library.
#[derive(Default, Debug)]
pub struct SemVerLibrary;
impl SemVerLibrary {
    /// Call semver operation.
    pub fn operate(&self, pid: &str, doc: &mut SDoc, name: &str, semver: &mut SVal, parameters: &mut Vec<SVal>) -> Result<SVal, SError> {
        match name {
            "major" => {
                match semver {
                    SVal::SemVer { major, minor: _, patch: _, release: _, build: _ } => {
                        return Ok(SVal::Number(SNum::I64(*major as i64)));
                    },
                    _ => {}
                }
                Err(SError::custom(pid, &doc, "SemVerMajor", &format!("semver not found")))
            },
            "setMajor" => {
                if parameters.len() < 1 {
                    return Err(SError::custom(pid, &doc, "SemVerSetMajor", &format!("version not found")));
                }
                match &parameters[0] {
                    SVal::Number(num) => {
                        match semver {
                            SVal::SemVer { major, minor: _, patch: _, release: _, build: _ } => {
                                *major = num.int() as i32;
                                return Ok(SVal::Void);
                            },
                            _ => {}
                        }
                    },
                    _ => {}
                }
                Err(SError::custom(pid, &doc, "SemVerMajor", &format!("semver not found")))
            },
            "minor" => {
                match semver {
                    SVal::SemVer { major: _, minor, patch: _, release: _, build: _ } => {
                        return Ok(SVal::Number(SNum::I64(*minor as i64)));
                    },
                    _ => {}
                }
                Err(SError::custom(pid, &doc, "SemVerMinor", &format!("semver not found")))
            },
            "setMinor" => {
                if parameters.len() < 1 {
                    return Err(SError::custom(pid, &doc, "SemVerSetMinor", &format!("version not found")));
                }
                match &parameters[0] {
                    SVal::Number(num) => {
                        match semver {
                            SVal::SemVer { major: _, minor, patch: _, release: _, build: _ } => {
                                *minor = num.int() as i32;
                                return Ok(SVal::Void);
                            },
                            _ => {}
                        }
                    },
                    _ => {}
                }
                Err(SError::custom(pid, &doc, "SemVerMinor", &format!("semver not found")))
            },
            "patch" => {
                match semver {
                    SVal::SemVer { major: _, minor: _, patch, release: _, build: _ } => {
                        return Ok(SVal::Number(SNum::I64(*patch as i64)));
                    },
                    _ => {}
                }
                Err(SError::custom(pid, &doc, "SemVerPatch", &format!("semver not found")))
            },
            "setPatch" => {
                if parameters.len() < 1 {
                    return Err(SError::custom(pid, &doc, "SemVerSetPatch", &format!("version not found")));
                }
                match &parameters[0] {
                    SVal::Number(num) => {
                        match semver {
                            SVal::SemVer { major: _, minor: _, patch, release: _, build: _ } => {
                                *patch = num.int() as i32;
                                return Ok(SVal::Void);
                            },
                            _ => {}
                        }
                    },
                    _ => {}
                }
                Err(SError::custom(pid, &doc, "SemVerPatch", &format!("semver not found")))
            },
            "release" => {
                match semver {
                    SVal::SemVer { major: _, minor: _, patch: _, release, build: _ } => {
                        if let Some(release) = release {
                            return Ok(SVal::String(release.clone()));
                        }
                        return Ok(SVal::Null);
                    },
                    _ => {}
                }
                Err(SError::custom(pid, &doc, "SemVerRelease", &format!("semver not found")))
            },
            "setRelease" => {
                if parameters.len() < 1 {
                    return Err(SError::custom(pid, &doc, "SemVerSetRelease", &format!("release string not found")));
                }
                match &parameters[0] {
                    SVal::String(val) => {
                        match semver {
                            SVal::SemVer { major: _, minor: _, patch: _, release, build: _ } => {
                                if val.len() > 0 {
                                    *release = Some(val.clone());
                                } else {
                                    *release = None;
                                }
                                return Ok(SVal::Void);
                            },
                            _ => {}
                        }
                    },
                    _ => {}
                }
                Err(SError::custom(pid, &doc, "SemVerSetRelease", &format!("semver not found")))
            },
            "clearRelease" => {
                match semver {
                    SVal::SemVer { major: _, minor: _, patch: _, release, build: _ } => {
                        *release = None;
                        return Ok(SVal::Void);
                    },
                    _ => {}
                }
                Err(SError::custom(pid, &doc, "SemVerClearRelease", &format!("semver not found")))
            },
            "build" => {
                match semver {
                    SVal::SemVer { major: _, minor: _, patch: _, release: _, build } => {
                        if let Some(build) = build {
                            return Ok(SVal::String(build.clone()));
                        }
                        return Ok(SVal::Null);
                    },
                    _ => {}
                }
                Err(SError::custom(pid, &doc, "SemVerBuild", &format!("semver not found")))
            },
            "setBuild" => {
                if parameters.len() < 1 {
                    return Err(SError::custom(pid, &doc, "SemVerSetBuild", &format!("build string not found")));
                }
                match &parameters[0] {
                    SVal::String(val) => {
                        match semver {
                            SVal::SemVer { major: _, minor: _, patch: _, release: _, build } => {
                                if val.len() > 0 {
                                    *build = Some(val.clone());
                                } else {
                                    *build = None;
                                }
                                return Ok(SVal::Void);
                            },
                            _ => {}
                        }
                    },
                    SVal::Number(num) => {
                        match semver {
                            SVal::SemVer { major: _, minor: _, patch: _, release: _, build } => {
                                *build = Some(format!("{}", num.int()));
                                return Ok(SVal::Void);
                            },
                            _ => {}
                        }
                    },
                    _ => {}
                }
                Err(SError::custom(pid, &doc, "SemVerSetBuild", &format!("semver not found")))
            },
            "clearBuild" => {
                match semver {
                    SVal::SemVer { major: _, minor: _, patch: _, release: _, build } => {
                        *build = None;
                        return Ok(SVal::Void);
                    },
                    _ => {}
                }
                Err(SError::custom(pid, &doc, "SemVerClearBuild", &format!("semver not found")))
            },
            _ => {
                Err(SError::custom(pid, &doc, "SemVerNotFound", &format!("{} is not a function in the SemVer Library", name)))
            }
        }
    }
}
impl Library for SemVerLibrary {
    /// Scope.
    fn scope(&self) -> String {
        "SemVer".to_string()
    }
    
    /// Call into the SemVer library.
    fn call(&self, pid: &str, doc: &mut SDoc, name: &str, parameters: &mut Vec<SVal>) -> Result<SVal, SError> {
        if parameters.len() > 0 {
            match name {
                "toString" => {
                    return Ok(SVal::String(parameters[0].print(doc)));
                },
                "or" => {
                    for param in parameters.drain(..) {
                        if !param.is_empty() {
                            return Ok(param);
                        }
                    }
                    return Ok(SVal::Null);
                },
                _ => {}
            }

            let mut params;
            if parameters.len() > 1 {
                params = parameters.drain(1..).collect();
            } else {
                params = Vec::new();
            }
            let val = &mut parameters[0];
            match val {
                SVal::SemVer { major: _, minor: _, patch: _, release: _, build: _ } => {
                    return self.operate(pid, doc, name, val, &mut params);
                },
                SVal::Boxed(val) => {
                    let mut val = val.lock().unwrap();
                    let val = val.deref_mut();
                    match val {
                        SVal::SemVer { major: _, minor: _, patch: _, release: _, build: _ } => {
                            return self.operate(pid, doc, name, val, &mut params);
                        },
                        _ => {
                            return Err(SError::custom(pid, &doc, "SemVerInvalidArgument", "semver argument not found"));
                        }
                    }
                },
                _ => {
                    return Err(SError::custom(pid, &doc, "SemVerInvalidArgument", "semver argument not found"));
                }
            }
        } else {
            return Err(SError::custom(pid, &doc, "SemVerInvalidArgument", "semver argument not found"));
        }
    }
}
