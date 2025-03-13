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
use bytes::Bytes;
use colored::Colorize;
use nanoid::nanoid;
use crate::{lang::SError, SData, SDoc, SField, SFunc, SType, SVal};


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
    fn call(&self, pid: &str, doc: &mut SDoc, name: &str, parameters: &mut Vec<SVal>) -> Result<SVal, SError>;
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
    fn call(&self, pid: &str, doc: &mut SDoc, name: &str, parameters: &mut Vec<SVal>) -> Result<SVal, SError> {
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
                Err(SError::std(pid, &doc, "hasFormat", "must provide a format string argument"))
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
                Err(SError::std(pid, &doc, "formatContentType", "must provide a format string argument"))
            },
            "hasLib" |
            "hasLibrary" => {
                if parameters.len() > 0 {
                    let lib = parameters[0].to_string();
                    let available = doc.available_libraries();
                    return Ok(SVal::Bool(available.contains(&lib)));
                }
                Err(SError::std(pid, &doc, "hasLibrary", "must provide a library string argument"))
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
                let mut error_type = String::from("Std");
                let mut start = 0;
                if parameters.len() > 1 {
                    error_type = parameters[0].to_string();
                    start += 1;
                }
                for i in start..parameters.len() {
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
                Err(SError::thrown(pid, &doc, &error_type, &res))
            },
            "assert" => {
                if parameters.len() == 1 {
                    let truthy = parameters[0].truthy();
                    if !truthy {
                        return Err(SError::std(pid, &doc, "assert", &format!("{:?} is not truthy", parameters[0])));
                    }
                    return Ok(SVal::Void);
                }
                Err(SError::std(pid, &doc, "assert", "must have one argument to assert"))
            },
            "assertNot" => {
                if parameters.len() == 1 {
                    let truthy = parameters[0].truthy();
                    if truthy {
                        return Err(SError::std(pid, &doc, "assertNot", &format!("{:?} is truthy", parameters[0])));
                    }
                    return Ok(SVal::Void);
                }
                Err(SError::std(pid, &doc, "assertNot", "must have one argument to assert not truthy"))
            },
            "assertNull" => {
                if parameters.len() == 1 {
                    if !parameters[0].is_null() {
                        return Err(SError::std(pid, &doc, "assertNull", &format!("{:?} is not null", parameters[0])));
                    }
                    return Ok(SVal::Void);
                }
                Err(SError::std(pid, &doc, "assertNull", "must give one argument to assert a null value"))
            },
            "assertObject" => {
                if parameters.len() == 1 {
                    if !parameters[0].is_object() {
                        return Err(SError::std(pid, &doc, "assertObject", &format!("{:?} is not an object", parameters[0])));
                    }
                    return Ok(SVal::Void);
                }
                Err(SError::std(pid, &doc, "assertObject", "must give one argument to assert it is an object value"))
            },
            "assertArray" => {
                if parameters.len() == 1 {
                    if !parameters[0].is_array() {
                        return Err(SError::std(pid, &doc, "assertArray", &format!("{:?} is not an array value", parameters[0])));
                    }
                    return Ok(SVal::Void);
                }
                Err(SError::std(pid, &doc, "assertArray", "must give one argument to assert it is an array value type"))
            },
            "assertTuple" => {
                if parameters.len() == 1 {
                    if !parameters[0].is_tuple() {
                        return Err(SError::std(pid, &doc, "assertTuple", &format!("{:?} is not a tuple value", parameters[0])));
                    }
                    return Ok(SVal::Void);
                }
                Err(SError::std(pid, &doc, "assertTuple", "must give one argument to assert it is a tuple value type"))
            },
            "assertNumber" => {
                if parameters.len() == 1 {
                    if !parameters[0].is_number() {
                        return Err(SError::std(pid, &doc, "assertNumber", &format!("{:?} is not a number value", parameters[0])));
                    }
                    return Ok(SVal::Void);
                }
                Err(SError::std(pid, &doc, "assertNumber", "must give one argument to assert it is a number value type"))
            },
            "assertEq" => {
                if parameters.len() == 2 {
                    let equals = parameters[0].equal(&parameters[1]);
                    match equals {
                        Ok(val) => {
                            let truthy = val.truthy();
                            if !truthy {
                                return Err(SError::std(pid, &doc, "assertEq", &format!("{:?} != {:?}", parameters[0], parameters[1])));
                            }
                            return Ok(SVal::Void);
                        },
                        Err(msg) => {
                            return Err(SError::std(pid, &doc, "assertEq", &msg.to_string(&doc.graph)));
                        }
                    }
                }
                Err(SError::std(pid, &doc, "assertEq", "must give 2 parameters to assert they equal each other"))
            },
            "assertNeq" => {
                if parameters.len() == 2 {
                    let nequals = parameters[0].neq(&parameters[1]);
                    match nequals {
                        Ok(val) => {
                            let truthy = val.truthy();
                            if !truthy {
                                return Err(SError::std(pid, &doc, "assertNeq", &format!("{:?} == {:?}", parameters[0], parameters[1])));
                            }
                            return Ok(SVal::Void);
                        },
                        Err(msg) => {
                            return Err(SError::std(pid, &doc, "assertNeq", &msg.to_string(&doc.graph)));
                        }
                    }
                }
                Err(SError::std(pid, &doc, "assertNeq", "must give 2 arguments to assert they do not equal each other"))
            },

            /*****************************************************************************
             * IDs.
             *****************************************************************************/
            // Return a new nanoid with an optional length (default is 21 URL safe chars).
            "nanoid" => {
                let mut length = None;
                if parameters.len() > 0 {
                    match &parameters[0] {
                        SVal::Number(num) => {
                            length = Some(num.int() as usize);
                        },
                        _ => {}
                    }
                }
                if let Some(length) = length {
                    return Ok(SVal::String(nanoid!(length)));
                }
                Ok(SVal::String(nanoid!()))
            },

            /*****************************************************************************
             * Tracing & Debugging.
             *****************************************************************************/
            // Return the current callstack functions.
            "callstack" => {
                let mut callstack = Vec::new();
                if let Some(process) = doc.processes.get(pid) {
                    for dref in &process.call_stack {
                        callstack.push(SVal::FnPtr(dref.clone()));
                    }
                }
                Ok(SVal::Array(callstack))
            },
            // Return a helpful string with the callstack printed out.
            "trace" => {
                let mut res = String::default();
                if let Some(process) = doc.processes.get(pid) {
                    for dref in &process.call_stack {
                        if let Some(func) = SData::get::<SFunc>(&doc.graph, dref) {
                            let func_nodes = dref.nodes(&doc.graph);
                            let func_path;
                            if func_nodes.len() > 0 {
                                func_path = func_nodes.first().unwrap().path(&doc.graph);
                            } else {
                                func_path = String::from("<unknown>");
                            }

                            res.push_str(&format!("{} {} {} {} ...\n", "trace".on_cyan(), func.name.blue(), "@".dimmed(), func_path.italic().bright_cyan()));
                        }
                    }
                }

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

            /*****************************************************************************
             * Value helpers.
             *****************************************************************************/
            // Return the first non-empty value.
            "or" => {
                for param in parameters.drain(..) {
                    if !param.is_empty() {
                        return Ok(param);
                    }
                }
                Ok(SVal::Null)
            },
            // Is the value a number type (boxed or unboxed int, float or units)?
            "isNumber" => {
                if parameters.len() != 1 {
                    return Err(SError::std(pid, &doc, "isNumber", "expecting one value argument to test that it is a number"));
                }
                Ok(SVal::Bool(parameters[0].is_number()))
            },
            // Is the value an object (boxed or unboxed)?
            "isObject" => {
                if parameters.len() != 1 {
                    return Err(SError::std(pid, &doc, "isObject", "expecting one value argument to test that it is an object"));
                }
                Ok(SVal::Bool(parameters[0].is_object()))
            },
            "isEmpty" |
            "isNull" => {
                if parameters.len() != 1 {
                    return Err(SError::std(pid, &doc, "isNull", "expecting one value argument to test that it is null/empty"));
                }
                Ok(SVal::Bool(parameters[0].is_empty()))
            },
            "isString" => {
                if parameters.len() != 1 {
                    return Err(SError::std(pid, &doc, "isString", "expecting one value argument to test that it is a string"));
                }
                Ok(SVal::Bool(parameters[0].is_string()))
            },
            "isBool" => {
                if parameters.len() != 1 {
                    return Err(SError::std(pid, &doc, "isBool", "expecting one value argument to test that it is a bool"));
                }
                Ok(SVal::Bool(parameters[0].is_bool()))
            },
            "isMap" => {
                if parameters.len() != 1 {
                    return Err(SError::std(pid, &doc, "isMap", "expecting one value argument to test that it is a map"));
                }
                Ok(SVal::Bool(parameters[0].is_map()))
            },
            "isSet" => {
                if parameters.len() != 1 {
                    return Err(SError::std(pid, &doc, "isSet", "expecting one value argument to test that it is a set"));
                }
                Ok(SVal::Bool(parameters[0].is_set()))
            },
            "isBlob" => {
                if parameters.len() != 1 {
                    return Err(SError::std(pid, &doc, "isBlob", "expecting one value argument to test that it is a blob"));
                }
                Ok(SVal::Bool(parameters[0].is_blob()))
            },
            "isVec" |
            "isArray" => {
                if parameters.len() != 1 {
                    return Err(SError::std(pid, &doc, "isVec", "expecting one value argument to test that it is a vector"));
                }
                Ok(SVal::Bool(parameters[0].is_array()))
            },
            "isFunc" => {
                if parameters.len() != 1 {
                    return Err(SError::std(pid, &doc, "isFunc", "expecting one value argument to test that it is a function"));
                }
                Ok(SVal::Bool(parameters[0].is_func()))
            },
            "isBoxed" |
            "isBox" => {
                if parameters.len() != 1 {
                    return Err(SError::std(pid, &doc, "isBox", "expecting one value argument to test that it is a boxed value"));
                }
                Ok(SVal::Bool(parameters[0].is_boxed()))
            },

            /*****************************************************************************
             * Box and Unbox helper functions.
             *****************************************************************************/
            "box" => {
                if parameters.len() > 0 {
                    return Ok(parameters.pop().unwrap().to_box());
                }
                Err(SError::std(pid, &doc, "box", "must give one argument value to ensure it is boxed"))
            },
            "unbox" => {
                if parameters.len() > 0 {
                    return Ok(parameters.pop().unwrap().unbox());
                }
                Err(SError::std(pid, &doc, "unbox", "must give one argument value to ensure it is not boxed"))
            },

            /*****************************************************************************
             * STD Lib Constructors.
             *****************************************************************************/
            "data" => {
                if let Some(data_lib) = doc.libraries.get("Data") {
                    return data_lib.call(pid, doc, "from", parameters);
                }
                Ok(SVal::Null)
            },
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
                                            return Err(SError::std(pid, &doc, "map", "cannot initialize a map with any array containing a non-tuple type with anything other than 2 values (key, value)"));
                                        }
                                    }
                                }
                            },
                            SVal::Map(omap) => {
                                for (k, v) in omap {
                                    if let Some(existing_val) = map.get_mut(k) {
                                        existing_val.merge(v);
                                    } else {
                                        map.insert(k.clone(), v.clone());
                                    }
                                }
                            },
                            SVal::Object(nref) => {
                                let mut to_insert = Vec::new();
                                for field in SField::fields(&doc.graph, nref) {
                                    to_insert.push(field.clone());
                                }
                                for field in to_insert {
                                    let k = SVal::String(field.name);
                                    let mut v = field.value;

                                    // If this field is an object, go deeper!
                                    if v.is_object() {
                                        v = self.call(pid, doc, "map", &mut vec![v])?;
                                    }

                                    if let Some(existing_val) = map.get_mut(&k) {
                                        existing_val.merge(&v);
                                    } else {
                                        map.insert(k.clone(), v);
                                    }
                                }
                            },
                            SVal::Tuple(tup) => {
                                if tup.len() == 2 {
                                    map.insert(tup[0].clone(), tup[1].clone());
                                }
                            },
                            _ => {}
                        }
                    }
                }
                Ok(SVal::Map(map))
            },
            _ => {
                Err(SError::std(pid, &doc, "NotFound", &format!("{} is not a function in the Stof Standard Library", name)))
            }
        }
    }
}
