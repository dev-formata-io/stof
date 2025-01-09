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

use std::{collections::BTreeMap, ops::DerefMut};
use anyhow::{anyhow, Result};
use crate::{Library, SDoc, SVal};


/// Map library.
#[derive(Default, Debug)]
pub struct MapLibrary;
impl MapLibrary {
    /// Call map operation.
    pub fn operate(&self, pid: &str, doc: &mut SDoc, name: &str, map: &mut BTreeMap<SVal, SVal>, parameters: &mut Vec<SVal>) -> Result<SVal> {
        match name {
            _ => {
                Err(anyhow!("Did not find the requested Map library function '{}'", name))
            }
        }
    }
}
impl Library for MapLibrary {
    fn scope(&self) -> String {
        "Map".to_string()
    }

    fn call(&self, pid: &str, doc: &mut SDoc, name: &str, parameters: &mut Vec<SVal>) -> Result<SVal> {
        if parameters.len() > 0 {
            match name {
                "toString" => {
                    return Ok(SVal::String(parameters[0].print(doc)));
                },
                _ => {}
            }

            let mut params;
            if parameters.len() > 1 {
                params = parameters.drain(1..).collect();
            } else {
                params = Vec::new();
            }
            match &mut parameters[0] {
                SVal::Map(map) => {
                    return self.operate(pid, doc, name, map, &mut params);
                },
                SVal::Boxed(val) => {
                    let mut val = val.lock().unwrap();
                    let val = val.deref_mut();
                    match val {
                        SVal::Map(map) => {
                            return self.operate(pid, doc, name, map, &mut params);
                        },
                        _ => {
                            return Err(anyhow!("Map library requires the first parameter to be a map"));
                        }
                    }
                },
                _ => {
                    return Err(anyhow!("Map library requires the first parameter to be a map"));
                }
            }
        } else {
            return Err(anyhow!("Map library requires a 'map' parameter to work with"));
        }
    }
}
