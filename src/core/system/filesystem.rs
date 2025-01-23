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

use std::fs;
use crate::{lang::SError, Library, SDoc, SVal};


/// File system library.
#[derive(Default, Debug)]
pub struct FileSystemLibrary;
impl Library for FileSystemLibrary {
    fn scope(&self) -> String {
        "fs".to_string()
    }

    fn call(&self, pid: &str, doc: &mut SDoc, name: &str, parameters: &mut Vec<SVal>) -> Result<SVal, SError> {
        match name {
            "write" => {
                if parameters.len() == 2 {
                    // write(path, contents)
                    let contents = parameters.pop().unwrap().owned_to_string();
                    let path = parameters.pop().unwrap().owned_to_string();
                    let res = fs::write(&path, &contents);
                    return match res {
                        Ok(_) => {
                            Ok(SVal::Void)
                        },
                        Err(error) => {
                            Err(SError::filesys(pid, &doc, "write", &error.to_string()))
                        }
                    };
                }
                Err(SError::filesys(pid, &doc, "write", "invalid arguments - path and contents not found"))
            },
            "write_blob" => {
                if parameters.len() == 2 {
                    // write(path, blob contents)
                    let contents = parameters.pop().unwrap();
                    let path = parameters.pop().unwrap().owned_to_string();
                    match contents {
                        SVal::Blob(blob) => {
                            let res = fs::write(&path, blob);
                            return match res {
                                Ok(_) => {
                                    Ok(SVal::Void)
                                },
                                Err(error) => {
                                    Err(SError::filesys(pid, &doc, "write_blob", &error.to_string()))
                                }
                            };
                        },
                        _ => {}
                    }
                }
                Err(SError::filesys(pid, &doc, "write_blob", "invalid arguments - path and blob contents not found"))
            },
            "read" => {
                if parameters.len() == 1 {
                    let path = parameters.pop().unwrap().owned_to_string();
                    let res = fs::read_to_string(&path);
                    return match res {
                        Ok(contents) => {
                            Ok(SVal::String(contents))
                        },
                        Err(error) => {
                            Err(SError::filesys(pid, &doc, "read", &error.to_string()))
                        }
                    };
                }
                Err(SError::filesys(pid, &doc, "read", "invalid arguments - file path not found"))
            },
            "read_blob" => {
                if parameters.len() == 1 {
                    let path = parameters.pop().unwrap().owned_to_string();
                    let res = fs::read(&path);
                    return match res {
                        Ok(blob) => {
                            Ok(SVal::Blob(blob))
                        },
                        Err(error) => {
                            Err(SError::filesys(pid, &doc, "read_blob", &error.to_string()))
                        }
                    };
                }
                Err(SError::filesys(pid, &doc, "read_blob", "invalid arguments - file path not found"))
            },
            _ => {
                Err(SError::filesys(pid, &doc, "NotFound", &format!("{} is not a function in the FileSystem Library", name)))
            }
        }
    }
}
