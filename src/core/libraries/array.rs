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
use crate::{Library, SData, SDoc, SFunc, SNum, SType, SVal};
use super::object::Object;


/// Array library.
#[derive(Default, Debug)]
pub struct ArrayLibrary;
impl Object for ArrayLibrary {}
impl Library for ArrayLibrary {
    /// Scope.
    fn scope(&self) -> String {
        "Array".to_string()
    }
    
    /// Call into the Array library.
    fn call(&self, pid: &str, doc: &mut SDoc, name: &str, parameters: &mut Vec<SVal>) -> Result<SVal> {
        if parameters.len() > 0 {
            match name {
                "push" => {
                    if parameters.len() > 1 {
                        let mut values = Vec::new();
                        for i in 1..parameters.len() {
                            values.push(parameters[i].clone());
                        }
                        Self::push(&mut parameters[0], values);
                        return Ok(SVal::Void);
                    }
                    return Err(anyhow!("Must provide value parameters to push into array."));
                },
                "pop" => {
                    if parameters.len() == 1 {
                        // Removes an element from the array if possible, returning it
                        return Self::pop(&mut parameters[0]);
                    } else if parameters.len() == 2 {
                        let find_val = parameters[1].clone();
                        let index;
                        if find_val.is_number() {
                            match &find_val {
                                SVal::Number(nval) => {
                                    index = nval.int() as usize;
                                },
                                _ => unreachable!()
                            }
                        } else {
                            let ival = self.call(pid, doc, "find", parameters)?;
                            match ival {
                                SVal::Number(nval) => {
                                    index = nval.int() as usize;
                                },
                                _ => unreachable!()
                            }
                        }

                        match &mut parameters[0] {
                            SVal::Array(vals) => {
                                return Ok(vals.remove(index));
                            },
                            _ => return Err(anyhow!("Cannot remove anything in the Array library on non-array"))
                        }
                    }
                },
                "reverse" => {
                    // Reverses the array in place
                    if parameters.len() == 1 {
                        match &mut parameters[0] {
                            SVal::Array(vals) => {
                                vals.reverse();
                                return Ok(SVal::Void);
                            },
                            _ => {}
                        }
                    }
                    return Err(anyhow!("Array.reverse(array) requires one array parameter"));
                },
                "reversed" => {
                    // Clones the array in reverse order
                    if parameters.len() == 1 {
                        match &parameters[0] {
                            SVal::Array(vals) => {
                                let mut new_vals = Vec::new();
                                for i in (0..vals.len()).rev() {
                                    new_vals.push(vals[i].clone());
                                }
                                return Ok(SVal::Array(new_vals));
                            },
                            _ => {}
                        }
                    }
                    return Err(anyhow!("Array.reversed(array) requires one array parameter"));
                },
                "len" => {
                    if parameters.len() == 1 {
                        // Return the length of the array
                        match &parameters[0] {
                            SVal::Array(vals) => {
                                return Ok(SVal::Number(SNum::I64(vals.len() as i64)));
                            },
                            _ => {}
                        }
                    }
                },
                "empty" => {
                    if parameters.len() == 1 {
                        // Return the length of the array
                        match &parameters[0] {
                            SVal::Array(vals) => {
                                return Ok(SVal::Bool(vals.len() < 1));
                            },
                            _ => {}
                        }
                    }
                },
                "any" => {
                    if parameters.len() == 1 {
                        // Return the length of the array
                        match &parameters[0] {
                            SVal::Array(vals) => {
                                return Ok(SVal::Bool(vals.len() > 0));
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
                            SVal::Array(vals) => {
                                if let Some(val) = vals.get(index) {
                                    return Ok(val.clone());
                                }
                                return Err(anyhow!("Index out of range"));
                            },
                            _ => return Err(anyhow!("Cannot index into anything but an array here"))
                        }
                    }
                },
                "first" => {
                    if parameters.len() == 1 {
                        match &parameters[0] {
                            SVal::Array(vals) => {
                                if let Some(val) = vals.first() {
                                    return Ok(val.clone());
                                }
                                return Ok(SVal::Null);
                            },
                            _ => return Err(anyhow!("Cannot index into anything but an array here"))
                        }
                    }
                },
                "last" => {
                    if parameters.len() == 1 {
                        match &parameters[0] {
                            SVal::Array(vals) => {
                                if let Some(val) = vals.last() {
                                    return Ok(val.clone());
                                }
                                return Ok(SVal::Null);
                            },
                            _ => return Err(anyhow!("Cannot index into anything but an array here"))
                        }
                    }
                },
                "join" => {
                    if parameters.len() == 2 {
                        let separator = parameters[1].cast(SType::String, pid, doc)?;
                        match separator {
                            SVal::String(separator) => {
                                match &mut parameters[0] {
                                    SVal::Array(vals) => {
                                        let mut str_vals = Vec::new();
                                        for v in vals {
                                            let str_val = v.cast(SType::String, pid, doc)?;
                                            match str_val {
                                                SVal::String(str_val) => str_vals.push(str_val),
                                                _ => {}
                                            }
                                        }
                                        return Ok(SVal::String(str_vals.join(&separator)));
                                    },
                                    _ => return Err(anyhow!("Cannot join array into a string"))
                                }
                            },
                            _ => {}
                        }
                    }
                },
                "has" |
                "contains" => {
                    if parameters.len() == 2 {
                        let find_val = parameters[1].clone();
                        match &mut parameters[0] {
                            SVal::Array(vals) => {
                                for i in 0..vals.len() {
                                    let res = vals[i].equal(&find_val);
                                    match res {
                                        Ok(val) => {
                                            if val.truthy() {
                                                return Ok(SVal::Bool(true));
                                            }
                                        },
                                        _ => {}
                                    }
                                }
                                return Ok(SVal::Bool(false));
                            },
                            _ => return Err(anyhow!("Cannot find on anything but an array in this library"))
                        }
                    }
                },
                "find" => {
                    if parameters.len() == 2 {
                        let find_val = parameters[1].clone();
                        match &mut parameters[0] {
                            SVal::Array(vals) => {
                                for i in 0..vals.len() {
                                    let res = vals[i].equal(&find_val);
                                    match res {
                                        Ok(val) => {
                                            if val.truthy() {
                                                return Ok(SVal::Number(SNum::I64(i as i64)));
                                            }
                                        },
                                        _ => {}
                                    }
                                }
                                return Ok(SVal::Number(SNum::I64(-1 as i64)));
                            },
                            _ => return Err(anyhow!("Cannot find on anything but an array in this library"))
                        }
                    }
                },
                "remove" => {
                    if parameters.len() == 2 {
                        let index;
                        let ival = self.call(pid, doc, "find", parameters);
                        if ival.is_err() {
                            return Ok(SVal::Null); // nothing removed...
                        }
                        match ival.unwrap() {
                            SVal::Number(nval) => {
                                let int = nval.int();
                                if int < 0 {
                                    return Ok(SVal::Null); // nothing found...
                                }
                                index = nval.int() as usize;
                            },
                            _ => return Err(anyhow!("Not able to find value"))
                        }

                        match &mut parameters[0] {
                            SVal::Array(vals) => {
                                return Ok(vals.remove(index));
                            },
                            _ => return Err(anyhow!("Cannot remove anything in the Array library on non-array"))
                        }
                    }
                },
                "removeAll" => {
                    if parameters.len() == 2 {
                        let find_val = parameters[1].clone();
                        match &mut parameters[0] {
                            SVal::Array(vals) => {
                                let mut to_remove = Vec::new();
                                for i in 0..vals.len() {
                                    let res = vals[i].equal(&find_val);
                                    match res {
                                        Ok(val) => {
                                            if val.truthy() {
                                                to_remove.push(i);
                                            }
                                        },
                                        _ => {}
                                    }
                                }
                                to_remove.reverse();
                                for index in &to_remove {
                                    vals.remove(*index);
                                }
                                return Ok(SVal::Bool(to_remove.len() > 0));
                            },
                            _ => return Err(anyhow!("Cannot find on anything but an array in this library"))
                        }
                    }
                },
                "insert" => {
                    if parameters.len() > 2 {
                        // Need at least 3 (array, index, ...to insert values)
                        let mut index;
                        {
                            let index_val = parameters[1].clone();
                            match index_val {
                                SVal::Number(nval) => {
                                    index = nval.int() as usize;
                                },
                                _ => return Err(anyhow!("Cannot call at with anything but a number index"))
                            }
                        }
                        let mut to_insert = Vec::new();
                        for i in 2..parameters.len() {
                            to_insert.push(parameters[i].clone());
                        }
                        match &mut parameters[0] {
                            SVal::Array(vals) => {
                                if index >= vals.len() {
                                    return Err(anyhow!("Out of bounds error"));
                                }
                                for v in to_insert {
                                    vals.insert(index, v);
                                    index += 1;
                                }
                                return Ok(SVal::Void);
                            },
                            _ => return Err(anyhow!("Cannot insert anything in the Array library on non-array"))
                        }
                    }
                },
                "set" |
                "replace" => {
                    if parameters.len() > 2 {
                        // Need at least 3 (array, index, ...to insert values)
                        let mut index;
                        {
                            let index_val = parameters[1].clone();
                            match index_val {
                                SVal::Number(nval) => {
                                    index = nval.int() as usize;
                                },
                                _ => return Err(anyhow!("Cannot call at with anything but a number index"))
                            }
                        }
                        let mut to_insert = Vec::new();
                        for i in 2..parameters.len() {
                            to_insert.push(parameters[i].clone());
                        }
                        match &mut parameters[0] {
                            SVal::Array(vals) => {
                                if index >= vals.len() {
                                    return Err(anyhow!("Out of bounds error"));
                                }
                                vals.remove(index);
                                for v in to_insert {
                                    vals.insert(index, v);
                                    index += 1;
                                }
                                return Ok(SVal::Void);
                            },
                            _ => return Err(anyhow!("Cannot insert anything in the Array library on non-array"))
                        }
                    }
                },
                "iter" => {
                    // Iterate through this array, calling the given function on each value
                    if parameters.len() == 2 {
                        match &parameters[1] {
                            SVal::FnPtr(dref) => {
                                if let Ok(func) = SData::data::<SFunc>(&doc.graph, dref) {
                                    match &mut parameters[0] {
                                        SVal::Array(vals) => {
                                            for val in vals {
                                                let res = func.call(pid, doc, vec![val.clone()], true)?;
                                                if !res.is_empty() {
                                                    *val = res;
                                                }
                                            }
                                            return Ok(SVal::Void);
                                        },
                                        _ => {}
                                    }
                                }
                            },
                            _ => return Err(anyhow!("Array.iter(array, fn) requires a function parameter"))
                        }
                    }
                    return Err(anyhow!("Array.iter(array, fn) requires an array and function parameters"))
                },
                "retain" => {
                    // Iterate through this array, calling the given function on each value and retaining the value if the return is truthy
                    if parameters.len() == 2 {
                        match &parameters[1] {
                            SVal::FnPtr(dref) => {
                                if let Ok(func) = SData::data::<SFunc>(&doc.graph, dref) {
                                    match &mut parameters[0] {
                                        SVal::Array(vals) => {
                                            vals.retain(|val| -> bool {
                                                let res = func.call(pid, doc, vec![val.clone()], true).unwrap_or(SVal::Null);
                                                res.truthy()
                                            });
                                            return Ok(SVal::Void);
                                        },
                                        _ => {}
                                    }
                                }
                            },
                            _ => return Err(anyhow!("Array.retain(array, fn) requires a function parameter"))
                        }
                    }
                    return Err(anyhow!("Array.retain(array, fn) requires an array and function parameters"))
                },
                _ => {}
            }
        }

        // try object scope
        if let Ok(val) = Self::object_call(pid, doc, name, parameters) {
            return Ok(val);
        }
        Err(anyhow!("Failed to find an Array library method."))
    }
}
impl ArrayLibrary {
    ///
    /// Push values into an array.
    ///
    fn push(array: &mut SVal, values: Vec<SVal>) {
        match array {
            SVal::Array(vals) => {
                for v in values { vals.push(v); }
            },
            _ => {}
        }
    }


    ///
    /// Pop values from an array.
    ///
    fn pop(array: &mut SVal) -> Result<SVal> {
        match array {
            SVal::Array(vals) => {
                if let Some(val) = vals.pop() {
                    return Ok(val);
                }
                return Ok(SVal::Null);
            },
            _ => {}
        }
        Err(anyhow!("Not able to pop this array"))
    }
}
