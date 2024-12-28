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
use anyhow::{anyhow, Result};
use crate::{Library, SDoc, SVal};


/// File system library.
#[derive(Default, Debug)]
pub struct FileSystemLibrary;
impl Library for FileSystemLibrary {
    fn scope(&self) -> String {
        "fs".to_string()
    }

    fn call(&self, _pid: &str, _doc: &mut SDoc, name: &str, parameters: &mut Vec<SVal>) -> Result<SVal> {
        match name {
            "write" => {
                if parameters.len() == 2 {
                    // write(path, contents)
                    let contents = parameters.pop().unwrap().owned_to_string();
                    let path = parameters.pop().unwrap().owned_to_string();
                    fs::write(&path, &contents)?;
                    return Ok(SVal::Void);
                }
                Err(anyhow!("Could not write '{:?}' using the filesystem library", parameters))
            },
            "write_blob" => {
                if parameters.len() == 2 {
                    // write(path, blob contents)
                    let contents = parameters.pop().unwrap();
                    let path = parameters.pop().unwrap().owned_to_string();
                    match contents {
                        SVal::Blob(blob) => {
                            fs::write(&path, blob)?;
                            return Ok(SVal::Void);
                        },
                        _ => {}
                    }
                }
                Err(anyhow!("Could not write blob '{:?}' using the filesystem library", parameters))
            },
            "read" |
            "read_to_string" => {
                if parameters.len() == 1 {
                    let path = parameters.pop().unwrap().owned_to_string();
                    let contents = fs::read_to_string(&path)?;
                    return Ok(SVal::String(contents));
                }
                Err(anyhow!("Could not read '{:?}' to a string using the filesystem library", parameters))
            },
            "read_to_blob" => {
                if parameters.len() == 1 {
                    let path = parameters.pop().unwrap().owned_to_string();
                    let blob = fs::read(&path)?;
                    return Ok(SVal::Blob(blob));
                }
                Err(anyhow!("Could not read '{:?}' to a blob using the filesystem library", parameters))
            },
            _ => {
                Err(anyhow!("Did not find the function '{}' in the filesystem library", name))
            }
        }
    }
}
