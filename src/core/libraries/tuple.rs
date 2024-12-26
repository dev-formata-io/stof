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

use anyhow::{anyhow, Result};
use crate::{SDoc, Library, SNum, SVal};
use super::Object;


/// Tuple library.
#[derive(Default, Debug)]
pub struct TupleLibrary;
impl Object for TupleLibrary {}
impl Library for TupleLibrary {
    /// Scope.
    fn scope(&self) -> String {
        "Tuple".to_string()
    }

    /// Call into the Tuple library.
    fn call(&self, pid: &str, doc: &mut SDoc, name: &str, parameters: &mut Vec<SVal>) -> Result<SVal> {
        if parameters.len() > 0 {
            if parameters[0].is_tuple() {
                match name {
                    "len" => {
                        if parameters.len() == 1 {
                            // Return the length of the tuple
                            match &parameters[0] {
                                SVal::Tuple(vals) => {
                                    return Ok(SVal::Number(SNum::I64(vals.len() as i64)));
                                },
                                _ => {}
                            }
                        }
                    },
                    "at" => {
                        if parameters.len() == 2 {
                            let index;
                            {
                                let index_val = parameters[1].clone();
                                match index_val {
                                    SVal::Number(nval) => {
                                        index = nval.int() as usize;
                                    },
                                    _ => return Err(anyhow!("Cannot call at with anything but a number index"))
                                }
                            }
                            match &parameters[0] {
                                SVal::Tuple(vals) => {
                                    if let Some(val) = vals.get(index) {
                                        return Ok(val.clone());
                                    }
                                    return Err(anyhow!("Index out of range"));
                                },
                                _ => return Err(anyhow!("Cannot index into anything but a tuple here"))
                            }
                        }
                    },
                    _ => {}
                }
            }
        }
        if let Ok(val) = Self::object_call(pid, doc, name, parameters) {
            return Ok(val);
        }
        Err(anyhow!("Failed to find a Tuple library method."))
    }
}
