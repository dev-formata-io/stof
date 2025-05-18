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
use crate::{lang::SError, Library, SData, SDoc, SFunc, SNum, SVal};


/// Set library.
#[derive(Default, Debug)]
pub struct SetLibrary;
impl SetLibrary {
    /// Call set operation.
    pub fn operate(&self, pid: &str, doc: &mut SDoc, name: &str, set: &mut BTreeSet<SVal>, parameters: &mut Vec<SVal>) -> Result<SVal, SError> {
        match name {
            // Move all elements from another set onto this set, leaving other set empty.
            // Signature: Set.append(set, other: set): void
            "append" => {
                if parameters.len() < 1 {
                    return Err(SError::set(pid, &doc, "append", "set to append not found"));
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
                                Err(SError::set(pid, &doc, "append", "set to append not found"))
                            }
                        }
                    },
                    _ => {
                        Err(SError::set(pid, &doc, "append", "set to append not found"))
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
                    return Err(SError::set(pid, &doc, "contains", "contains value argument not found"));
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
                    return Err(SError::set(pid, &doc, "insert", "value to insert not found"));
                }
                let value = parameters.pop().unwrap();
                Ok(SVal::Bool(set.insert(value)))
            },
            // Take a value from this set if the set contains it. Removes this value if it exists.
            // Signature: Set.take(set, value: unknown): null | unknown
            "take" => {
                if parameters.len() < 1 {
                    return Err(SError::set(pid, &doc, "take", "take value argument not found"));
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
                    return Err(SError::set(pid, &doc, "split", "split value not found"));
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
                    return Err(SError::set(pid, &doc, "at", "index argument not found"));
                }
                match &parameters[0] {
                    SVal::Number(index) => {
                        let index = index.int() as usize;
                        if index >= set.len() {
                            return Err(SError::set(pid, &doc, "at", "index out of bounds"));
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
                    return Err(SError::set(pid, &doc, "remove", "value argument to remove not found"));
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
                    return Err(SError::set(pid, &doc, "retain", "predicate not found"));
                }
                match &parameters[0] {
                    SVal::FnPtr(dref) => {
                        if let Some(func) = SData::get::<SFunc>(&doc.graph, dref) {
                            let rtype = func.rtype.clone();
                            let statements = func.statements.clone();
                            let params = func.params.clone();
                            set.retain(|v| {
                                if let Ok(res) = SFunc::call_internal(dref, pid, doc, vec![v.clone()], true, &params, &statements, &rtype, false) {
                                    res.truthy()
                                } else {
                                    false
                                }
                            });
                        }
                        Ok(SVal::Void)
                    },
                    _ => {
                        Err(SError::set(pid, &doc, "retain", "predicate not found"))
                    }
                }
            },
            // set + other (union).
            // Signature: Set.union(set, other: set): set
            "union" => {
                if parameters.len() < 1 {
                    return Err(SError::set(pid, &doc, "union", "set argument not found"));
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
                                Err(SError::set(pid, &doc, "union", "set argument not found"))
                            }
                        }
                    },
                    _ => {
                        Err(SError::set(pid, &doc, "union", "set argument not found"))
                    }
                }
            },
            // set - other (difference).
            // Signature: Set.difference(set, other: set): set
            "difference" => {
                if parameters.len() < 1 {
                    return Err(SError::set(pid, &doc, "difference", "set argument not found"));
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
                                Err(SError::set(pid, &doc, "difference", "set argument not found"))
                            }
                        }
                    },
                    _ => {
                        Err(SError::set(pid, &doc, "difference", "set argument not found"))
                    }
                }
            },
            // set * other (intersection).
            // Signature: Set.intersection(set, other: set): set
            "intersection" => {
                if parameters.len() < 1 {
                    return Err(SError::set(pid, &doc, "intersection", "set argument not found"));
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
                                Err(SError::set(pid, &doc, "intersection", "set argument not found"))
                            }
                        }
                    },
                    _ => {
                        Err(SError::set(pid, &doc, "intersection", "set argument not found"))
                    }
                }
            },
            // set % other (symmetric difference).
            // Signature: Set.symmetricDifference(set, other: set): set
            "symmetricDifference" => {
                if parameters.len() < 1 {
                    return Err(SError::set(pid, &doc, "symmetricDifference", "set argument not found"));
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
                                Err(SError::set(pid, &doc, "symmetricDifference", "set argument not found"))
                            }
                        }
                    },
                    _ => {
                        Err(SError::set(pid, &doc, "symmetricDifference", "set argument not found"))
                    }
                }
            },
            // Returns true if this set has no elements in common with another.
            // Signature: Set.disjoint(set, other: set): bool
            "disjoint" => {
                if parameters.len() < 1 {
                    return Err(SError::set(pid, &doc, "disjoint", "set argument not found"));
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
                                Err(SError::set(pid, &doc, "disjoint", "set argument not found"))
                            }
                        }
                    },
                    _ => {
                        Err(SError::set(pid, &doc, "disjoint", "set argument not found"))
                    }
                }
            },
            // Returns true if this set is a subset of another.
            // Signature: Set.subset(set, other: set): bool
            "subset" => {
                if parameters.len() < 1 {
                    return Err(SError::set(pid, &doc, "subset", "set argument not found"));
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
                                Err(SError::set(pid, &doc, "subset", "set argument not found"))
                            }
                        }
                    },
                    _ => {
                        Err(SError::set(pid, &doc, "subset", "set argument not found"))
                    }
                }
            },
            // Returns true if this set is a superset of another.
            // Signature: Set.superset(set, other: set): bool
            "superset" => {
                if parameters.len() < 1 {
                    return Err(SError::set(pid, &doc, "superset", "set argument not found"));
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
                                Err(SError::set(pid, &doc, "superset", "set argument not found"))
                            }
                        }
                    },
                    _ => {
                        Err(SError::set(pid, &doc, "superset", "set argument not found"))
                    }
                }
            },
            _ => {
                Err(SError::set(pid, &doc, "NotFound", &format!("{} is not a function in the Set Library", name)))
            }
        }
    }
}
impl Library for SetLibrary {
    fn scope(&self) -> String {
        "Set".to_string()
    }

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
                            return Err(SError::set(pid, &doc, "InvalidArgument", "set argument not found"));
                        }
                    }
                },
                _ => {
                    return Err(SError::set(pid, &doc, "InvalidArgument", "set argument not found"));
                }
            }
        } else {
            return Err(SError::set(pid, &doc, "InvalidArgument", "set argument not found"));
        }
    }
}
