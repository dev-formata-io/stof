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

use std::{cmp::Ordering, ops::DerefMut};
use anyhow::{anyhow, Result};
use crate::{Library, SData, SDoc, SFunc, SNum, SVal};


/// Array library.
#[derive(Default, Debug)]
pub struct ArrayLibrary;
impl ArrayLibrary {
    ///
    /// Call array operation with an array values.
    ///
    pub fn operate(&self, pid: &str, doc: &mut SDoc, name: &str, array: &mut Vec<SVal>, parameters: &mut Vec<SVal>) -> Result<SVal> {
        match name {
            // Push all parameters onto the array.
            "push" => {
                for v in parameters.drain(..) { array.push(v); }
                Ok(SVal::Void)
            },
            // Pop values from the array.
            "pop" => {
                if parameters.len() > 0 {
                    let index = parameters.pop().unwrap();
                    match index {
                        SVal::Number(num) => {
                            // Pop an element at a specific index
                            let index = num.int() as usize;
                            if index >= array.len() {
                                return Err(anyhow!("attempting to pop from an Array with an index that is out of range"));
                            }
                            return Ok(array.remove(index));
                        },
                        _ => {
                            // Try to find the index of this 'index' value
                            let mut i = -1;
                            for idx in 0..array.len() {
                                if array[idx] == index {
                                    i = idx as i32;
                                    break; // found
                                }
                            }
                            if i < 0 {
                                return Ok(SVal::Null); // nothing found to remove
                            }
                            return Ok(array.remove(i as usize));
                        }
                    }
                }
                if let Some(val) = array.pop() {
                    return Ok(val);
                }
                return Ok(SVal::Null);
            },
            // Reverse the array in-place.
            "reverse" => {
                array.reverse();
                Ok(SVal::Void)
            },
            // Clone of this array in the reverse order.
            "reversed" => {
                let mut vals = Vec::new();
                for i in (0..array.len()).rev() {
                    vals.push(array[i].clone());
                }
                Ok(SVal::Array(vals))
            },
            // Length of this array.
            "len" => {
                Ok(SVal::Number(SNum::I64(array.len() as i64)))
            },
            // Is this array empty?
            "empty" => {
                Ok(SVal::Bool(array.len() < 1))
            },
            // Does this array have any values?
            "any" => {
                Ok(SVal::Bool(array.len() > 0))
            },
            // Get the value in the array at a specific index.
            "at" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("Array.at(vec, index) requires at least 2 parameters"));
                }
                let mut results = Vec::new();
                for index in parameters.drain(..) {
                    match index {
                        SVal::Number(index) => {
                            let index = index.int() as usize;
                            if let Some(val) = array.get(index) {
                                results.push(val.clone());
                            } else {
                                return Err(anyhow!("Array.at(vec, index) index out of bounds"));
                            }
                        },
                        _ => {
                            return Err(anyhow!("Array.at(vec, index) cannot have indices that are not numbers"));
                        }
                    }
                }
                if results.len() == 1 {
                    Ok(results.pop().unwrap())
                } else {
                    Ok(SVal::Array(results))
                }
            },
            // Get the first value in the array.
            "first" => {
                if let Some(val) = array.first() {
                    return Ok(val.clone());
                }
                return Ok(SVal::Null);
            },
            // Get the last value in the array.
            "last" => {
                if let Some(val) = array.last() {
                    return Ok(val.clone());
                }
                return Ok(SVal::Null);
            },
            // Join the elements of this array together with a separator.
            "join" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("Array.join(vec, sep) requires a string separator"));
                }
                let separator = parameters[0].to_string();
                let mut str_vals = Vec::new();
                for v in array {
                    str_vals.push(v.to_string());
                }
                Ok(SVal::String(str_vals.join(&separator)))
            },
            // Has or contains a value?
            "has" |
            "contains" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("Array.has(vec, val) requires a value to search this array for"));
                }
                let search = parameters.pop().unwrap();
                let mut found = false;
                for val in array {
                    if val == &search {
                        found = true;
                        break;
                    }
                }
                Ok(SVal::Bool(found))
            },
            // Find the first index of a value in this array.
            // Takes one value parameter and returns -1 if not found or the index if found.
            "find" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("Array.find(vec, val) requires a value to search for"));
                }
                let search = parameters.pop().unwrap();
                let mut index = SVal::Number(SNum::I64(-1));
                for i in 0..array.len() {
                    if &array[i] == &search {
                        index = SVal::Number(SNum::I64(i as i64));
                        break;
                    }
                }
                Ok(index)
            },
            // Remove the first matching value from this array (not by index, but by value).
            // If trying to remove by index, see "pop".
            "remove" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("Array.remove(vec, val) requires a value to search for"))
                }
                let search = parameters.pop().unwrap();
                let mut index = -1;
                for i in 0..array.len() {
                    if &array[i] == &search {
                        index = i as i32;
                        break;
                    }
                }
                if index > -1 {
                    Ok(array.remove(index as usize))
                } else {
                    Ok(SVal::Null)
                }
            },
            // Remove the last matching value from this array (not by index, but by value).
            // If trying to remove by index, see "pop".
            "removeLast" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("Array.removeLast(vec, val) requires a value to search for"))
                }
                let search = parameters.pop().unwrap();
                let mut index = -1;
                for i in (0..array.len()).rev() {
                    if &array[i] == &search {
                        index = i as i32;
                        break;
                    }
                }
                if index > -1 {
                    Ok(array.remove(index as usize))
                } else {
                    Ok(SVal::Null)
                }
            },
            // Remove all matching values from this array (by value).
            "removeAll" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("Array.removeAll(vec, val, ..) requires at least one value to search for"));
                }
                let mut results = Vec::new();
                for value in parameters.drain(..) {
                    let mut to_remove = Vec::new();
                    for i in 0..array.len() {
                        if &array[i] == &value {
                            to_remove.push(i);
                        }
                    }
                    to_remove.reverse();
                    let mut res = Vec::new();
                    for i in to_remove {
                        res.push(array.remove(i));
                    }
                    res.reverse();
                    results.append(&mut res);
                }
                Ok(SVal::Array(results))
            },
            // Insert values into the array at a specific index.
            "insert" => {
                if parameters.len() < 2 {
                    return Err(anyhow!("Array.insert(vec, index, value, ..) requires an index and at least one value to insert"));
                }
                let index = parameters.remove(0);
                match index {
                    SVal::Number(num) => {
                        let mut index = num.int() as usize;
                        if index >= array.len() {
                            return Err(anyhow!("Array.insert(vec, index, value, ..) out of bounds error"));
                        }
                        for val in parameters.drain(..) {
                            array.insert(index, val);
                            index += 1;
                        }
                        Ok(SVal::Void)
                    },
                    _ => {
                        Err(anyhow!("Array.insert(vec, index, value, ..) must have a number as an index"))
                    }
                }
            },
            // Insert values into the array at a specific index, replacing the current value.
            "set" |
            "replace" => {
                if parameters.len() < 2 {
                    return Err(anyhow!("Array.set(vec, index, value, ..) requires an index and at least one value to insert"));
                }
                let index = parameters.remove(0);
                match index {
                    SVal::Number(num) => {
                        let mut index = num.int() as usize;
                        if index >= array.len() {
                            return Err(anyhow!("Array.set(vec, index, value, ..) out of bounds error"));
                        }
                        array.remove(index);
                        for val in parameters.drain(..) {
                            array.insert(index, val);
                            index += 1;
                        }
                        Ok(SVal::Void)
                    },
                    _ => {
                        Err(anyhow!("Array.set(vec, index, value, ..) must have a number as an index"))
                    }
                }
            },
            // Iterate over this array, calling a function for each value.
            "iter" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("Array.iter(vec, fn) requires a function to iterate with"));
                }
                match &parameters[0] {
                    SVal::FnPtr(dref) => {
                        if let Ok(func) = SData::data::<SFunc>(&doc.graph, dref) {
                            for val in array {
                                let res = func.call(pid, doc, vec![val.clone()], true)?;
                                if !res.is_empty() {
                                    *val = res;
                                }
                            }
                        }
                        Ok(SVal::Void)
                    },
                    _ => return Err(anyhow!("Array.iter(array, fn) requires a function parameter"))
                }
            },
            // Retain values in this array, according to a function call.
            "retain" => {
                if parameters.len() < 1 {
                    return Err(anyhow!("Array.retain(vec, fn) requires a function to iterate with"));
                }
                match &parameters[0] {
                    SVal::FnPtr(dref) => {
                        if let Ok(func) = SData::data::<SFunc>(&doc.graph, dref) {
                            array.retain(|val| -> bool {
                                let res = func.call(pid, doc, vec![val.clone()], true).unwrap_or(SVal::Null);
                                res.truthy()
                            });
                        }
                        Ok(SVal::Void)
                    },
                    _ => return Err(anyhow!("Array.retain(array, fn) requires a function parameter"))
                }
            },
            // Sort this array in-place.
            "sort" => {
                if parameters.len() == 0 {
                    array.sort_by(|a, b| {
                        let lt = a.lt(b);
                        if lt.is_err() {
                            return Ordering::Equal;
                        }
                        if lt.unwrap().truthy() {
                            return Ordering::Less;
                        }

                        let gt = a.gt(b);
                        if gt.is_err() {
                            return Ordering::Equal;
                        }
                        if gt.unwrap().truthy() {
                            return Ordering::Greater;
                        }
                        Ordering::Equal
                    });
                    Ok(SVal::Void)
                } else {
                    match &parameters[0] {
                        SVal::FnPtr(dref) => {
                            if let Ok(func) = SData::data::<SFunc>(&doc.graph, dref) {
                                array.sort_by(|a, b| {
                                    let res = func.call(pid, doc, vec![a.clone(), b.clone()], true).unwrap_or(SVal::Number(SNum::I64(0)));
                                    match res {
                                        SVal::Number(num) => {
                                            let int = num.int();
                                            if int < 0 {
                                                Ordering::Less
                                            } else if int > 0 {
                                                Ordering::Greater
                                            } else {
                                                Ordering::Equal
                                            }
                                        },
                                        _ => {
                                            Ordering::Equal
                                        }
                                    }
                                });
                            }
                            Ok(SVal::Void)
                        },
                        _ => return Err(anyhow!("Array.sort(array, fn) requires a function parameter"))
                    }
                }
            },
            _ => {
                Err(anyhow!("Did not find the requested Array library function '{}'", name))
            }
        }
    }
}
impl Library for ArrayLibrary {
    /// Scope.
    fn scope(&self) -> String {
        "Array".to_string()
    }
    
    /// Call into the Array library.
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
                SVal::Array(vals) => {
                    return self.operate(pid, doc, name, vals, &mut params);
                },
                SVal::Boxed(val) => {
                    let mut val = val.lock().unwrap();
                    let val = val.deref_mut();
                    match val {
                        SVal::Array(vals) => {
                            return self.operate(pid, doc, name, vals, &mut params);
                        },
                        _ => {
                            return Err(anyhow!("Array library requires the first parameter to be a vec (Array)"));
                        }
                    }
                },
                _ => {
                    return Err(anyhow!("Array library requires the first parameter to be a vec (Array)"));
                }
            }
        } else {
            return Err(anyhow!("Array library requires a 'vec' parameter to work on"));
        }
    }
}
