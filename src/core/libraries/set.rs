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

use std::{collections::BTreeSet, ops::{Deref, DerefMut}};
use anyhow::{anyhow, Result};
use crate::{Library, SData, SDoc, SFunc, SNum, SVal};


/// Set library.
#[derive(Default, Debug)]
pub struct SetLibrary;
impl SetLibrary {
    /// Call set operation.
    pub fn operate(&self, pid: &str, doc: &mut SDoc, name: &str, set: &mut BTreeSet<SVal>, parameters: &mut Vec<SVal>) -> Result<SVal> {
        match name {
            // Move all elements from another set onto this set, leaving other set empty.
            // Signature: Set.append(set, other: set): void
            "append" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("Set.append(set, other: set) requires another set to move values from"));
                }
                match &mut parameters[0] {
                    SVal::Set(other) => {
                        set.append(other);
                        Ok(SVal::Void)
                    },
                    SVal::Boxed(other) => {
                        let mut other = other.lock().unwrap();
                        let other = other.deref_mut();
                        match other {
                            SVal::Set(other) => {
                                set.append(other);
                                Ok(SVal::Void)
                            },
                            _ => {
                                Err(anyhow!("Set.append(set, other: set) requires a set parameter to move values from"))
                            }
                        }
                    },
                    _ => {
                        Err(anyhow!("Set.append(set, other: set) requires a set parameter to move values from"))
                    }
                }
            },
            // Clear the set, removing all elements.
            // Signature: Set.clear(set): void
            "clear" => {
                set.clear();
                Ok(SVal::Void)
            },
            // Contains a value?
            // Signature: Set.contains(set, value: unknown): bool
            "contains" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("Set.contains(set, value: unknown) requires a value parameter to look for"));
                }
                Ok(SVal::Bool(set.contains(&parameters[0])))
            },
            // Get the first value in this ordered set.
            // Signature: Set.first(set): null | unknown
            "first" => {
                if let Some(value) = set.first() {
                    return Ok(value.clone());
                }
                Ok(SVal::Null)
            },
            // Get the last value in this ordered set.
            // Signature: Set.last(set): null | unknown
            "last" => {
                if let Some(value) = set.last() {
                    return Ok(value.clone());
                }
                Ok(SVal::Null)
            },
            // Insert a value into this set, returning whether this value was newly inserted.
            // Signature: Set.insert(set, value: unknown): bool
            "insert" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("Set.insert(set, value: unknown) requires a value to insert with"));
                }
                let value = parameters.pop().unwrap();
                Ok(SVal::Bool(set.insert(value)))
            },
            // Take a value from this set if the set contains it. Removes this value if it exists.
            // Signature: Set.take(set, value: unknown): null | unknown
            "take" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("Set.take(set, value: unknown) requires a value parameter to take from this set"))
                }
                if let Some(value) = set.take(&parameters[0]) {
                    Ok(value)
                } else {
                    Ok(SVal::Null)
                }
            },
            // Split this set at a value.
            // Signature: Set.split(set, value: unknown): set
            "split" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("Set.split(set, value: unknown) requires a value to split this set with"));
                }
                Ok(SVal::Set(set.split_off(&parameters[0])))
            },
            // Is this set empty?
            // Signature: Set.empty(set): bool
            "empty" => {
                Ok(SVal::Bool(set.is_empty()))
            },
            // Does this set contain any values?
            // Signature: Set.any(set): bool
            "any" => {
                Ok(SVal::Bool(!set.is_empty()))
            },
            // Length of this set. Enables iteration as well... (for loops)
            // Signature: Set.len(set): int
            "len" => {
                Ok(SVal::Number(SNum::I64(set.len() as i64)))
            },
            // Get an item at a specific index in this set. Enables iteration as well... (for loops)
            // Signature: Set.at(set, index): unknown
            "at" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("Set.at(set, index: unknown) requires an index parameter"));
                }
                match &parameters[0] {
                    SVal::Number(index) => {
                        let index = index.int() as usize;
                        if index >= set.len() {
                            return Err(anyhow!("Set.at(set, index: int) index out of bounds"));
                        }
                        if let Some(value) = set.iter().nth(index) {
                            Ok(value.clone())
                        } else {
                            Ok(SVal::Null)
                        }
                    },
                    _ => {
                        if let Some(value) = set.get(&parameters[0]) {
                            Ok(value.clone())
                        } else {
                            Ok(SVal::Null)
                        }
                    }
                }
            },
            // Pop first value in this set.
            // Signature: Set.popFirst(set): null | unknown
            "popFirst" => {
                if let Some(value) = set.pop_first() {
                    Ok(value)
                } else {
                    Ok(SVal::Null)
                }
            },
            // Pop last value in this set.
            // Signature: Set.popLast(set): null | unknown
            "popLast" => {
                if let Some(value) = set.pop_last() {
                    Ok(value)
                } else {
                    Ok(SVal::Null)
                }
            },
            // Remove a value from this set, returning true if the value existed.
            // Signature: Set.remove(set, value: unknown): bool
            "remove" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("Set.remove(set, value: unknown) requires a value parameter to remove"))
                }
                if set.remove(&parameters[0]) {
                    Ok(SVal::Bool(true))
                } else {
                    Ok(SVal::Bool(false))
                }
            },
            // Retain only the elements specified by the predicate.
            // Signature: Set.retain(set, pred: fn): void
            "retain" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("Set.retain(set, predicate: fn) requires a function parameter"));
                }
                match &parameters[0] {
                    SVal::FnPtr(dref) => {
                        if let Ok(func) = SData::data::<SFunc>(&doc.graph, dref) {
                            set.retain(|v| {
                                if let Ok(res) = func.call(pid, doc, vec![v.clone()], true) {
                                    res.truthy()
                                } else {
                                    false
                                }
                            });
                            Ok(SVal::Void)
                        } else {
                            Err(anyhow!("Set.retain(set, predicate: fn) function does not exist"))
                        }
                    },
                    _ => {
                        Err(anyhow!("Set.retain(set, predicate: fn) requires a function parameter"))
                    }
                }
            },
            // set + other (union).
            // Signature: Set.union(set, other: set): set
            "union" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("Set.union(set, other: set) requires a set parameter to work with"));
                }
                match &parameters[0] {
                    SVal::Set(other) => {
                        Ok(SVal::Set(set.union(other).cloned().collect()))
                    },
                    SVal::Boxed(val) => {
                        let val = val.lock().unwrap();
                        let val = val.deref();
                        match val {
                            SVal::Set(other) => {
                                Ok(SVal::Set(set.union(other).cloned().collect()))
                            },
                            _ => {
                                Err(anyhow!("Set.union(set, other: set) requires a set parameter to work with"))
                            }
                        }
                    },
                    _ => {
                        Err(anyhow!("Set.union(set, other: set) requires a set parameter to work with"))
                    }
                }
            },
            // set - other (difference).
            // Signature: Set.difference(set, other: set): set
            "difference" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("Set.difference(set, other: set) requires a set parameter to work with"));
                }
                match &parameters[0] {
                    SVal::Set(other) => {
                        Ok(SVal::Set(set.difference(other).cloned().collect()))
                    },
                    SVal::Boxed(val) => {
                        let val = val.lock().unwrap();
                        let val = val.deref();
                        match val {
                            SVal::Set(other) => {
                                Ok(SVal::Set(set.difference(other).cloned().collect()))
                            },
                            _ => {
                                Err(anyhow!("Set.difference(set, other: set) requires a set parameter to work with"))
                            }
                        }
                    },
                    _ => {
                        Err(anyhow!("Set.difference(set, other: set) requires a set parameter to work with"))
                    }
                }
            },
            // set * other (intersection).
            // Signature: Set.intersection(set, other: set): set
            "intersection" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("Set.intersection(set, other: set) requires a set parameter to work with"));
                }
                match &parameters[0] {
                    SVal::Set(other) => {
                        Ok(SVal::Set(set.intersection(other).cloned().collect()))
                    },
                    SVal::Boxed(val) => {
                        let val = val.lock().unwrap();
                        let val = val.deref();
                        match val {
                            SVal::Set(other) => {
                                Ok(SVal::Set(set.intersection(other).cloned().collect()))
                            },
                            _ => {
                                Err(anyhow!("Set.intersection(set, other: set) requires a set parameter to work with"))
                            }
                        }
                    },
                    _ => {
                        Err(anyhow!("Set.intersection(set, other: set) requires a set parameter to work with"))
                    }
                }
            },
            // set % other (symmetric difference).
            // Signature: Set.symmetricDifference(set, other: set): set
            "symmetricDifference" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("Set.symmetricDifference(set, other: set) requires a set parameter to work with"));
                }
                match &parameters[0] {
                    SVal::Set(other) => {
                        Ok(SVal::Set(set.symmetric_difference(other).cloned().collect()))
                    },
                    SVal::Boxed(val) => {
                        let val = val.lock().unwrap();
                        let val = val.deref();
                        match val {
                            SVal::Set(other) => {
                                Ok(SVal::Set(set.symmetric_difference(other).cloned().collect()))
                            },
                            _ => {
                                Err(anyhow!("Set.symmetricDifference(set, other: set) requires a set parameter to work with"))
                            }
                        }
                    },
                    _ => {
                        Err(anyhow!("Set.symmetricDifference(set, other: set) requires a set parameter to work with"))
                    }
                }
            },
            // Returns true if this set has no elements in common with another.
            // Signature: Set.disjoint(set, other: set): bool
            "disjoint" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("Set.disjoint(set, other: set) requires a set parameter to work with"));
                }
                match &parameters[0] {
                    SVal::Set(other) => {
                        Ok(SVal::Bool(set.is_disjoint(other)))
                    },
                    SVal::Boxed(val) => {
                        let val = val.lock().unwrap();
                        let val = val.deref();
                        match val {
                            SVal::Set(other) => {
                                Ok(SVal::Bool(set.is_disjoint(other)))
                            },
                            _ => {
                                Err(anyhow!("Set.disjoint(set, other: set) requires a set parameter to work with"))
                            }
                        }
                    },
                    _ => {
                        Err(anyhow!("Set.disjoint(set, other: set) requires a set parameter to work with"))
                    }
                }
            },
            // Returns true if this set is a subset of another.
            // Signature: Set.subset(set, other: set): bool
            "subset" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("Set.subset(set, other: set) requires a set parameter to work with"));
                }
                match &parameters[0] {
                    SVal::Set(other) => {
                        Ok(SVal::Bool(set.is_subset(other)))
                    },
                    SVal::Boxed(val) => {
                        let val = val.lock().unwrap();
                        let val = val.deref();
                        match val {
                            SVal::Set(other) => {
                                Ok(SVal::Bool(set.is_subset(other)))
                            },
                            _ => {
                                Err(anyhow!("Set.subset(set, other: set) requires a set parameter to work with"))
                            }
                        }
                    },
                    _ => {
                        Err(anyhow!("Set.subset(set, other: set) requires a set parameter to work with"))
                    }
                }
            },
            // Returns true if this set is a superset of another.
            // Signature: Set.superset(set, other: set): bool
            "superset" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("Set.superset(set, other: set) requires a set parameter to work with"));
                }
                match &parameters[0] {
                    SVal::Set(other) => {
                        Ok(SVal::Bool(set.is_superset(other)))
                    },
                    SVal::Boxed(val) => {
                        let val = val.lock().unwrap();
                        let val = val.deref();
                        match val {
                            SVal::Set(other) => {
                                Ok(SVal::Bool(set.is_superset(other)))
                            },
                            _ => {
                                Err(anyhow!("Set.superset(set, other: set) requires a set parameter to work with"))
                            }
                        }
                    },
                    _ => {
                        Err(anyhow!("Set.superset(set, other: set) requires a set parameter to work with"))
                    }
                }
            },
            _ => {
                Err(anyhow!("Did not find the requested Set library function '{}'", name))
            }
        }
    }
}
impl Library for SetLibrary {
    fn scope(&self) -> String {
        "Set".to_string()
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
                SVal::Set(set) => {
                    return self.operate(pid, doc, name, set, &mut params);
                },
                SVal::Boxed(val) => {
                    let mut val = val.lock().unwrap();
                    let val = val.deref_mut();
                    match val {
                        SVal::Set(set) => {
                            return self.operate(pid, doc, name, set, &mut params);
                        },
                        _ => {
                            return Err(anyhow!("Set library requires the first parameter to be a set"));
                        }
                    }
                },
                _ => {
                    return Err(anyhow!("Set library requires the first parameter to be a set"));
                }
            }
        } else {
            return Err(anyhow!("Set library requires a 'set' parameter to work with"));
        }
    }
}
