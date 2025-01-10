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

use std::{collections::{BTreeMap, BTreeSet, HashSet}, sync::Arc};
use anyhow::{anyhow, Result};
use bytes::Bytes;
use crate::{SDoc, SField, SType, SVal};


/// Stof Libraries.
#[derive(Default, Clone)]
pub struct SLibraries {
    pub libraries: BTreeMap<String, Arc<dyn Library>>,
}
impl SLibraries {
    /// Insert a library.
    pub fn insert(&mut self, library: Arc<dyn Library>) {
        let scope = library.scope();
        if scope.len() > 0 {
            self.libraries.insert(scope, library);
        }
    }

    /// Get a library.
    pub fn get(&self, scope: &str) -> Option<Arc<dyn Library>> {
        self.libraries.get(scope).cloned()
    }

    /// Available libraries.
    pub fn available(&self) -> HashSet<String> {
        let mut scopes = HashSet::new();
        for (scope, _) in &self.libraries {
            scopes.insert(scope.clone());
        }
        scopes
    }
}


/// Stof library.
pub trait Library: Sync + Send {
    /// Library name/scope.
    fn scope(&self) -> String;

    /// Call a library function with a set of parameters.
    fn call(&self, pid: &str, doc: &mut SDoc, name: &str, parameters: &mut Vec<SVal>) -> Result<SVal>;
}


/// Stof standard library.
#[derive(Default, Debug)]
pub struct StdLibrary;
impl Library for StdLibrary {
    /// Library name/scope.
    fn scope(&self) -> String {
        String::from("std")
    }

    /// Call a standard library function.
    fn call(&self, pid: &str, doc: &mut SDoc, name: &str, parameters: &mut Vec<SVal>) -> Result<SVal> {
        match name {
            "parse" => {
                if parameters.len() > 0 {
                    let mut format = "stof".to_string();
                    if parameters.len() > 1 {
                        format = parameters[1].to_string();
                    }
                    let mut as_name = "root".to_string();
                    if let Some(self_ptr) = doc.self_ptr(pid) {
                        as_name = self_ptr.path(&doc.graph); // Absolute path to current location
                    }
                    if parameters.len() > 2 {
                        let req_name = parameters[2].to_string();
                        if req_name.starts_with("self") || req_name.starts_with("super") {
                            // relative path from current location
                            as_name.push_str(&format!(".{}", req_name));
                        } else {
                            // absolute path
                            as_name = req_name;
                        }
                    }
                    match &parameters[0] {
                        SVal::String(src) => {
                            doc.string_import(pid, &format, &src, &as_name)?;
                            return Ok(SVal::Bool(true));
                        },
                        SVal::Blob(bytes) => {
                            let mut bytes = Bytes::from(bytes.clone());
                            doc.header_import(pid, &format, &format, &mut bytes, &as_name)?;
                            return Ok(SVal::Bool(true));
                        },
                        _ => {}
                    }
                }
                Ok(SVal::Bool(false))
            },
            "blobify" => {
                if parameters.len() > 0 { // Must have an object/value - will use STOF as the default format
                    let mut format = "stof".to_string();
                    if parameters.len() > 1 {
                        format = parameters[1].to_string();
                    }
                    match &parameters[0] {
                        SVal::Object(nref) => {
                            let res = doc.export_bytes(pid, &format, Some(nref))?;
                            return Ok(SVal::Blob(res.to_vec()));
                        },
                        _ => {
                            let blob = parameters[0].cast(SType::Blob, pid, doc)?;
                            return Ok(blob);
                        }
                    }
                }
                Ok(SVal::Null)
            },
            "stringify" => {
                if parameters.len() > 0 { // Must have an object/value - will use STOF as the default format
                    let mut format = "stof".to_string();
                    if parameters.len() > 1 {
                        format = parameters[1].to_string();
                    }
                    let mut min = true;
                    if parameters.len() > 2 { // Add another truthy value to make it human readable
                        min = !parameters[2].truthy();
                    }
                    match &parameters[0] {
                        SVal::Object(nref) => {
                            let res;
                            if min {
                                res = doc.export_min_string(pid, &format, Some(nref))?;
                            } else {
                                res = doc.export_string(pid, &format, Some(nref))?;
                            }
                            return Ok(SVal::String(res));
                        },
                        _ => {
                            let str = parameters[0].cast(SType::String, pid, doc)?;
                            return Ok(str);
                        }
                    }
                }
                Ok(SVal::Null)
            },
            "hasFormat" => {
                if parameters.len() > 0 {
                    let format = parameters[0].to_string();
                    let available = doc.available_formats();
                    return Ok(SVal::Bool(available.contains(&format)));
                }
                Err(anyhow!("Must provide a format string to look for"))
            },
            "formats" => {
                let formats: Vec<SVal> = doc.available_formats().into_iter().map(|fmt| SVal::String(fmt)).collect();
                Ok(SVal::Array(formats))
            },
            "formatContentType" => {
                if parameters.len() > 0 {
                    let format = parameters[0].to_string();
                    return Ok(SVal::String(doc.format_content_type(&format).unwrap_or("text/plain".to_string())));
                }
                Err(anyhow!("Must provide a format to get a content type for"))
            },
            "hasLib" |
            "hasLibrary" => {
                if parameters.len() > 0 {
                    let lib = parameters[0].to_string();
                    let available = doc.available_libraries();
                    return Ok(SVal::Bool(available.contains(&lib)));
                }
                Err(anyhow!("Must provide a library name to look for"))
            },
            "libs" |
            "libraries" => {
                let libs: Vec<SVal> = doc.available_libraries().into_iter().map(|fmt| SVal::String(fmt)).collect();
                Ok(SVal::Array(libs))
            },
            "pln" => {
                let mut res = String::default();
                for i in 0..parameters.len() {
                    let param = &parameters[i];
                    let print = param.print(doc);
                    
                    match param.stype(&doc.graph) {
                        SType::String => {
                            // Don't do any gaps for strings!
                            res.push_str(&format!("{}", print));
                        },
                        _ => {
                            if i > 0 {
                                res.push_str(&format!(", {}", print));
                            } else {
                                res.push_str(&format!("{}", print));
                            }
                        }
                    }
                }
                println!("{}", res); // Print to console
                Ok(SVal::Void)
            },
            "dbg" => {
                let mut res = String::default();
                for i in 0..parameters.len() {
                    let param = &parameters[i];
                    let print = param.debug(doc);
                    
                    match param.stype(&doc.graph) {
                        SType::String => {
                            // Don't do any gaps for strings!
                            res.push_str(&format!("{}", print));
                        },
                        _ => {
                            if i > 0 {
                                res.push_str(&format!(", {}", print));
                            } else {
                                res.push_str(&format!("{}", print));
                            }
                        }
                    }
                }
                println!("{}", res); // Print to console
                Ok(SVal::Void)
            },
            // Print to error
            "err" => {
                let mut res = String::default();
                for i in 0..parameters.len() {
                    let param = &parameters[i];
                    let print = param.print(doc);
                    
                    match param.stype(&doc.graph) {
                        SType::String => {
                            // Don't do any gaps for strings!
                            res.push_str(&format!("{}", print));
                        },
                        _ => {
                            if i > 0 {
                                res.push_str(&format!(", {}", print));
                            } else {
                                res.push_str(&format!("{}", print));
                            }
                        }
                    }
                }
                eprintln!("{}", res); // Print to error
                Ok(SVal::Void)
            },
            "throw" => {
                let mut res = String::default();
                for i in 0..parameters.len() {
                    let param = &parameters[i];
                    let print = param.print(doc);
                    
                    match param.stype(&doc.graph) {
                        SType::String => {
                            // Don't do any gaps for strings!
                            res.push_str(&format!("{}", print));
                        },
                        _ => {
                            if i > 0 {
                                res.push_str(&format!(", {}", print));
                            } else {
                                res.push_str(&format!("{}", print));
                            }
                        }
                    }
                }
                Err(anyhow!("{}", res))
            },
            "assert" => {
                if parameters.len() == 1 {
                    let truthy = parameters[0].truthy();
                    if !truthy {
                        return Err(anyhow!("Assert failed - {:?} is not truthy", parameters[0]));
                    }
                    return Ok(SVal::Void);
                }
                Err(anyhow!("Assert must have 1 parameter"))
            },
            "assertNot" => {
                if parameters.len() == 1 {
                    let truthy = parameters[0].truthy();
                    if truthy {
                        return Err(anyhow!("Assert failed - {:?} is truthy", parameters[0]));
                    }
                    return Ok(SVal::Void);
                }
                Err(anyhow!("Assert must have 1 parameter"))
            },
            "assertNull" => {
                if parameters.len() == 1 {
                    if !parameters[0].is_null() {
                        return Err(anyhow!("Assert null failed"));
                    }
                    return Ok(SVal::Void);
                }
                Err(anyhow!("Assert null must have 1 parameter"))
            },
            "assertObject" => {
                if parameters.len() == 1 {
                    if !parameters[0].is_object() {
                        return Err(anyhow!("Assert object failed"));
                    }
                    return Ok(SVal::Void);
                }
                Err(anyhow!("Assert object must have 1 parameter"))
            },
            "assertArray" => {
                if parameters.len() == 1 {
                    if !parameters[0].is_array() {
                        return Err(anyhow!("Assert array failed"));
                    }
                    return Ok(SVal::Void);
                }
                Err(anyhow!("Assert array must have 1 parameter"))
            },
            "assertTuple" => {
                if parameters.len() == 1 {
                    if !parameters[0].is_tuple() {
                        return Err(anyhow!("Assert tuple failed"));
                    }
                    return Ok(SVal::Void);
                }
                Err(anyhow!("Assert tuple must have 1 parameter"))
            },
            "assertNumber" => {
                if parameters.len() == 1 {
                    if !parameters[0].is_number() {
                        return Err(anyhow!("Assert number failed"));
                    }
                    return Ok(SVal::Void);
                }
                Err(anyhow!("Assert number must have 1 parameter"))
            },
            "assertEq" => {
                if parameters.len() == 2 {
                    let equals = parameters[0].equal(&parameters[1]);
                    match equals {
                        Ok(val) => {
                            let truthy = val.truthy();
                            if !truthy {
                                return Err(anyhow!("Assert equals failed - {:?} != {:?}", parameters[0], parameters[1]));
                            }
                            return Ok(SVal::Void);
                        },
                        Err(msg) => {
                            return Err(anyhow!("Assert equals failed: {}", msg))
                        }
                    }
                }
                Err(anyhow!("Assert equals must have 2 parameters"))
            },
            "assertNeq" => {
                if parameters.len() == 2 {
                    let nequals = parameters[0].neq(&parameters[1]);
                    match nequals {
                        Ok(val) => {
                            let truthy = val.truthy();
                            if !truthy {
                                return Err(anyhow!("Assert not equals failed - {:?} == {:?}", parameters[0], parameters[1]));
                            }
                            return Ok(SVal::Void);
                        },
                        Err(msg) => {
                            return Err(anyhow!("Assert not equals failed: {}", msg))
                        }
                    }
                }
                Err(anyhow!("Assert not equals must have 2 parameters"))
            },
            /*"dump" => {
                doc.graph.dump(true);
                Ok(SVal::Void)
            },*/

            /*****************************************************************************
             * Or helpers.
             *****************************************************************************/
            // Return the first non-falsy value.
            "or" => {
                for param in parameters.drain(..) {
                    if param.truthy() {
                        return Ok(param);
                    }
                }
                Ok(SVal::Null)
            },

            /*****************************************************************************
             * Box and Unbox helper functions.
             *****************************************************************************/
            "box" => {
                if parameters.len() > 0 {
                    return Ok(parameters.pop().unwrap().to_box());
                }
                Err(anyhow!("std.box(..) requires at least one parameter"))
            },
            "unbox" => {
                if parameters.len() > 0 {
                    return Ok(parameters.pop().unwrap().unbox());
                }
                Err(anyhow!("std.unbox(..) requires at least one parameter"))
            },

            /*****************************************************************************
             * STD Lib Constructors.
             *****************************************************************************/
            "vec" => {
                let mut array = Vec::new();
                if parameters.len() > 0 {
                    for param in parameters.drain(..) {
                        match param {
                            SVal::Array(vals) => {
                                for val in vals {
                                    array.push(val);
                                }
                            },
                            SVal::Set(oset) => {
                                for val in oset {
                                    array.push(val);
                                }
                            },
                            _ => {
                                array.push(param);
                            }
                        }
                    }
                }
                Ok(SVal::Array(array))
            },
            "set" => {
                let mut set = BTreeSet::new();
                if parameters.len() > 0 {
                    for param in parameters.drain(..) {
                        match param {
                            SVal::Array(vals) => {
                                for val in vals {
                                    set.insert(val);
                                }
                            },
                            SVal::Set(oset) => {
                                for val in oset {
                                    set.insert(val);
                                }
                            },
                            _ => {
                                set.insert(param);
                            }
                        }
                    }
                }
                Ok(SVal::Set(set))
            },
            "map" => {
                let mut map = BTreeMap::new();
                if parameters.len() > 0 {
                    for param in 0..parameters.len() {
                        match &parameters[param] {
                            SVal::Array(vals) => {
                                for val in vals {
                                    match val {
                                        SVal::Tuple(tup) => {
                                            if tup.len() == 2 {
                                                map.insert(tup[0].clone(), tup[1].clone());
                                            }
                                        },
                                        _ => {
                                            return Err(anyhow!("Cannot initialize a Map with any value other than a tuple (key, value)"));
                                        }
                                    }
                                }
                            },
                            SVal::Map(omap) => {
                                for (k, v) in omap {
                                    if let Some(existing_val) = map.get_mut(k) {
                                        existing_val.union(v);
                                    } else {
                                        map.insert(k.clone(), v.clone());
                                    }
                                }
                            },
                            SVal::Object(nref) => {
                                // this one is kinda cool...
                                for field in SField::fields(&doc.graph, nref) {
                                    let k = SVal::String(field.name);
                                    let mut v = field.value;

                                    // If this field is an object, go deeper!
                                    if v.is_object() {
                                        v = self.call(pid, doc, "map", &mut vec![v])?;
                                    }

                                    if let Some(existing_val) = map.get_mut(&k) {
                                        existing_val.union(&v);
                                    } else {
                                        map.insert(k.clone(), v);
                                    }
                                }
                            },
                            _ => {}
                        }
                    }
                }
                Ok(SVal::Map(map))
            },
            _ => {
                Err(anyhow!("Did not find a function named '{}' in the standard library", name))
            }
        }
    }
}
