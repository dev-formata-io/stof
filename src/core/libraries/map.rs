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
use crate::{Library, SData, SDoc, SFunc, SNum, SVal};


/// Map library.
#[derive(Default, Debug)]
pub struct MapLibrary;
impl MapLibrary {
    /// Call map operation.
    pub fn operate(&self, pid: &str, doc: &mut SDoc, name: &str, map: &mut BTreeMap<SVal, SVal>, parameters: &mut Vec<SVal>) -> Result<SVal> {
        match name {
            // Move all elements from another map onto this map, leaving other map empty.
            // Signature: Map.append(map, other: map): void
            "append" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("Map.append(map, other: map) requires another map to move values from"));
                }
                match &mut parameters[0] {
                    SVal::Map(other) => {
                        map.append(other);
                        Ok(SVal::Void)
                    },
                    SVal::Boxed(other) => {
                        let mut other = other.lock().unwrap();
                        let other = other.deref_mut();
                        match other {
                            SVal::Map(other) => {
                                map.append(other);
                                Ok(SVal::Void)
                            },
                            _ => {
                                Err(anyhow!("Map.append(map, other: map) requires a map parameter to move values from"))
                            }
                        }
                    },
                    _ => {
                        Err(anyhow!("Map.append(map, other: map) requires a map parameter to move values from"))
                    }
                }
            },
            // Clear the map, removing all elements.
            // Signature: Map.clear(map): void
            "clear" => {
                map.clear();
                Ok(SVal::Void)
            },
            // Contains a key?
            // Signature: Map.contains(map, key: unknown): bool
            "contains" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("Map.contains(map, key: unknown) requires a key parameter to look for"));
                }
                Ok(SVal::Bool(map.contains_key(&parameters[0])))
            },
            // Get the first key and value pair in this ordered map.
            // Signature: Map.first(map): null | (key: unknown, value: unknown)
            "first" => {
                if let Some((key, value)) = map.first_key_value() {
                    return Ok(SVal::Tuple(vec![key.clone(), value.clone()]));
                }
                Ok(SVal::Null)
            },
            // Get the last key and value pair in this ordered map.
            // Signature: Map.last(map): null | (key: unknown, value: unknown)
            "last" => {
                if let Some((key, value)) = map.last_key_value() {
                    return Ok(SVal::Tuple(vec![key.clone(), value.clone()]));
                }
                Ok(SVal::Null)
            },
            // Get a value in this map.
            // Signature: Map.get(map, key: unknown): null | unknown
            "get" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("Map.get(map, key: unknown) requires a key parameter to search with"));
                }
                if let Some(value) = map.get(&parameters[0]) {
                    return Ok(value.clone());
                }
                Ok(SVal::Null)
            },
            // Insert a key-value pair into this map, returning the old value if already in map
            // Signature: Map.insert(map, key: unknown, value: unknown): null | unknown
            "insert" => {
                if parameters.len() < 2 {
                    return Err(anyhow!("Map.insert(map, key: unknown, value: unknown) requires a key and value to insert with"));
                }
                let value = parameters.pop().unwrap();
                let key = parameters.pop().unwrap();
                if let Some(old) = map.insert(key, value) {
                    Ok(old)
                } else {
                    Ok(SVal::Null)
                }
            },
            // Is this map empty?
            // Signature: Map.empty(map): bool
            "empty" => {
                Ok(SVal::Bool(map.is_empty()))
            },
            // Does this map contain any values?
            // Signature: Map.any(map): bool
            "any" => {
                Ok(SVal::Bool(!map.is_empty()))
            },
            // Return the keys of this map in a vec.
            // Signature: Map.keys(map): vec
            "keys" => {
                Ok(SVal::Array(map.keys().cloned().collect()))
            },
            // Return the values of this map in a vec.
            // Signature: Map.values(map): vec
            "values" => {
                Ok(SVal::Array(map.values().cloned().collect()))
            },
            // Length of this map. Enables iteration as well... (for loops)
            // Signature: Map.len(map): int
            "len" => {
                Ok(SVal::Number(SNum::I64(map.len() as i64)))
            },
            // Get an item at a specific index in this map. Enables iteration as well... (for loops)
            // Signature: Map.at(map, index): (key: unknown, value: unknown)
            "at" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("Map.at(map, index: unknown) requires an index parameter"));
                }
                match &parameters[0] {
                    SVal::Number(index) => {
                        let index = index.int() as usize;
                        if index >= map.len() {
                            return Err(anyhow!("Map.at(map, index: int) index out of bounds"));
                        }
                        if let Some((key, value)) = map.iter().nth(index) {
                            Ok(SVal::Tuple(vec![key.clone(), value.clone()]))
                        } else {
                            Ok(SVal::Null)
                        }
                    },
                    _ => {
                        if let Some((key, value)) = map.get_key_value(&parameters[0]) {
                            Ok(SVal::Tuple(vec![key.clone(), value.clone()]))
                        } else {
                            Ok(SVal::Null)
                        }
                    }
                }
            },
            // Pop first value in this map.
            // Signature: Map.popFirst(map): null | (key: unknown, value: unknown)
            "popFirst" => {
                if let Some((key, value)) = map.pop_first() {
                    Ok(SVal::Tuple(vec![key, value]))
                } else {
                    Ok(SVal::Null)
                }
            },
            // Pop last value in this map.
            // Signature: Map.popLast(map): null | (key: unknown, value: unknown)
            "popLast" => {
                if let Some((key, value)) = map.pop_last() {
                    Ok(SVal::Tuple(vec![key, value]))
                } else {
                    Ok(SVal::Null)
                }
            },
            // Remove an entry in this map.
            // Signature: Map.remove(map, key: unknown): null | unknown
            "remove" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("Map.remove(map, key: unknown) requires a key parameter to remove with"))
                }
                if let Some(value) = map.remove(&parameters[0]) {
                    Ok(value)
                } else {
                    Ok(SVal::Null)
                }
            },
            // Retain only the elements specified by the predicate.
            // Signagure: Map.retain(map, pred: fn): void
            "retain" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("Map.retain(map, predicate: fn) requires a function parameter"));
                }
                match &parameters[0] {
                    SVal::FnPtr(dref) => {
                        if let Ok(func) = SData::data::<SFunc>(&doc.graph, dref) {
                            map.retain(|k, v| {
                                if let Ok(res) = func.call(pid, doc, vec![k.clone(), v.clone()], true) {
                                    res.truthy()
                                } else {
                                    false
                                }
                            });
                            Ok(SVal::Void)
                        } else {
                            Err(anyhow!("Map.retain(map, predicate: fn) function does not exist"))
                        }
                    },
                    _ => {
                        Err(anyhow!("Map.retain(map, predicate: fn) requires a function parameter"))
                    }
                }
            },
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
