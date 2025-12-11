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

use std::{ops::{Deref, DerefMut}, sync::Arc};
use arcstr::{literal, ArcStr};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use crate::{model::{ver::ops::{ver_build, ver_clear_build, ver_clear_release, ver_major, ver_minor, ver_patch, ver_release, ver_set_build, ver_set_major, ver_set_minor, ver_set_patch, ver_set_release}, Graph}, runtime::{instruction::{Instruction, Instructions}, proc::ProcEnv, Error, Num, Val, Variable}};

mod ops;


/// Library name.
pub(self) const VER_LIB: ArcStr = literal!("Ver");


/// Add the version library to a graph.
pub fn insert_semver_lib(graph: &mut Graph) {
    graph.insert_libfunc(ver_major());
    graph.insert_libfunc(ver_minor());
    graph.insert_libfunc(ver_patch());
    graph.insert_libfunc(ver_release());
    graph.insert_libfunc(ver_build());
    graph.insert_libfunc(ver_set_major());
    graph.insert_libfunc(ver_set_minor());
    graph.insert_libfunc(ver_set_patch());
    graph.insert_libfunc(ver_set_release());
    graph.insert_libfunc(ver_set_build());
    graph.insert_libfunc(ver_clear_release());
    graph.insert_libfunc(ver_clear_build());
}


lazy_static! {
    pub(self) static ref MAJOR: Arc<dyn Instruction> = Arc::new(VerIns::Major);
    pub(self) static ref MINOR: Arc<dyn Instruction> = Arc::new(VerIns::Minor);
    pub(self) static ref PATCH: Arc<dyn Instruction> = Arc::new(VerIns::Patch);
    pub(self) static ref RELEASE: Arc<dyn Instruction> = Arc::new(VerIns::Release);
    pub(self) static ref BUILD: Arc<dyn Instruction> = Arc::new(VerIns::Build);
    pub(self) static ref SET_MAJOR: Arc<dyn Instruction> = Arc::new(VerIns::SetMajor);
    pub(self) static ref SET_MINOR: Arc<dyn Instruction> = Arc::new(VerIns::SetMinor);
    pub(self) static ref SET_PATCH: Arc<dyn Instruction> = Arc::new(VerIns::SetPatch);
    pub(self) static ref SET_RELEASE: Arc<dyn Instruction> = Arc::new(VerIns::SetRelease);
    pub(self) static ref SET_BUILD: Arc<dyn Instruction> = Arc::new(VerIns::SetBuild);
    pub(self) static ref CLEAR_RELEASE: Arc<dyn Instruction> = Arc::new(VerIns::ClearRelease);
    pub(self) static ref CLEAR_BUILD: Arc<dyn Instruction> = Arc::new(VerIns::ClearBuild);
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Semver instructions.
pub enum VerIns {
    Major,
    SetMajor,
    
    Minor,
    SetMinor,
    
    Patch,
    SetPatch,
    
    Release,
    SetRelease,
    ClearRelease,
    
    Build,
    SetBuild,
    ClearBuild,
}
#[typetag::serde(name = "VerIns")]
impl Instruction for VerIns {
    fn exec(&self, env: &mut ProcEnv, _graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        match self {
            Self::Major => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Ver(major, ..) => {
                            env.stack.push(Variable::val(Val::Num(Num::Int(*major as i64))));
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::VerMajor)
            },
            Self::SetMajor => {
                if let Some(set) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        match set.val.read().deref() {
                            Val::Num(num) => {
                                match var.val.write().deref_mut() {
                                    Val::Ver(major, ..) => {
                                        *major = num.int() as i32;
                                        return Ok(None);
                                    },
                                    _ => {}
                                }
                            },
                            _ => {},
                        }
                    }
                }
                Err(Error::VerSetMajor)
            },
            Self::Minor => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Ver(_, minor, ..) => {
                            env.stack.push(Variable::val(Val::Num(Num::Int(*minor as i64))));
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::VerMinor)
            },
            Self::SetMinor => {
                if let Some(set) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        match set.val.read().deref() {
                            Val::Num(num) => {
                                match var.val.write().deref_mut() {
                                    Val::Ver(_, minor, ..) => {
                                        *minor = num.int() as i32;
                                        return Ok(None);
                                    },
                                    _ => {}
                                }
                            },
                            _ => {},
                        }
                    }
                }
                Err(Error::VerSetMinor)
            },
            Self::Patch => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Ver(_, _, patch, ..) => {
                            env.stack.push(Variable::val(Val::Num(Num::Int(*patch as i64))));
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::VerPatch)
            },
            Self::SetPatch => {
                if let Some(set) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        match set.val.read().deref() {
                            Val::Num(num) => {
                                match var.val.write().deref_mut() {
                                    Val::Ver(_, _, patch, ..) => {
                                        *patch = num.int() as i32;
                                        return Ok(None);
                                    },
                                    _ => {}
                                }
                            },
                            _ => {},
                        }
                    }
                }
                Err(Error::VerSetPatch)
            },
            Self::Release => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Ver(_, _, _, release, _) => {
                            if let Some(release) = release {
                                env.stack.push(Variable::val(Val::Str(release.clone())));
                            } else {
                                env.stack.push(Variable::val(Val::Null));
                            }
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::VerRelease)
            },
            Self::SetRelease => {
                if let Some(set) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        match set.val.read().deref() {
                            Val::Str(new_release) => {
                                match var.val.write().deref_mut() {
                                    Val::Ver(_, _, _, release, _) => {
                                        *release = Some(new_release.clone());
                                        return Ok(None);
                                    },
                                    _ => {}
                                }
                            },
                            _ => {},
                        }
                    }
                }
                Err(Error::VerSetRelease)
            },
            Self::Build => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Ver(_, _, _, _, build) => {
                            if let Some(build) = build {
                                env.stack.push(Variable::val(Val::Str(build.clone())));
                            } else {
                                env.stack.push(Variable::val(Val::Null));
                            }
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::VerBuild)
            },
            Self::SetBuild => {
                if let Some(set) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        match set.val.read().deref() {
                            Val::Str(new_build) => {
                                match var.val.write().deref_mut() {
                                    Val::Ver(_, _, _, _, build) => {
                                        *build = Some(new_build.clone());
                                        return Ok(None);
                                    },
                                    _ => {}
                                }
                            },
                            _ => {},
                        }
                    }
                }
                Err(Error::VerSetBuild)
            },
            Self::ClearRelease => {
                if let Some(var) = env.stack.pop() {
                    match var.val.write().deref_mut() {
                        Val::Ver(_, _, _, release, _) => {
                            *release = None;
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::VerClearRelease)
            },
            Self::ClearBuild => {
                if let Some(var) = env.stack.pop() {
                    match var.val.write().deref_mut() {
                        Val::Ver(_, _, _, _, build) => {
                            *build = None;
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::VerClearBuild)
            },
        }
    }
}
