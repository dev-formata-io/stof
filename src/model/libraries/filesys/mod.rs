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


use std::{fs, ops::Deref, sync::Arc};
use imbl::vector;
use serde::{Deserialize, Serialize};
use crate::{model::{Graph, LibFunc, Param, FS_LIB}, runtime::{instruction::{Instruction, Instructions}, proc::ProcEnv, Error, Type, Val, Variable}};


/// Add fs library to a graph.
pub fn fs_library(graph: &mut Graph) {
    graph.insert_libfunc(read());
    graph.insert_libfunc(write());
    graph.insert_libfunc(read_string());
}


/// Filesystem read a file into a blob.
pub(self) fn read() -> LibFunc {
    LibFunc {
        library: FS_LIB.clone(),
        name: "read".into(),
        is_async: false,
        docs: r#"# fs.read(path: str) -> blob
If available, will read a file from a path into a binary blob.
```rust
const bytes = fs.read("src/lib.rs");
```"#.into(),
        params: vector![
            Param { name: "path".into(), param_type: Type::Str, default: None }
        ],
        return_type: Some(Type::Blob),
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(FsIns::Read));
            Ok(instructions)
        })
    }
}


/// Filesystem read a file into a string.
pub(self) fn read_string() -> LibFunc {
    LibFunc {
        library: FS_LIB.clone(),
        name: "read_string".into(),
        is_async: false,
        docs: r#"# fs.read_string(path: str) -> str
If available, will read a file from a path into a string.
```rust
const content = fs.read_string("src/lib.rs");
```"#.into(),
        params: vector![
            Param { name: "path".into(), param_type: Type::Str, default: None }
        ],
        return_type: Some(Type::Blob),
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(FsIns::ReadString));
            Ok(instructions)
        })
    }
}


/// Filesystem write to a file.
pub(self) fn write() -> LibFunc {
    LibFunc {
        library: FS_LIB.clone(),
        name: "write".into(),
        is_async: false,
        docs: r#"# fs.write(path: str, content: str | blob) -> void
If available, will write content into a file at the given path. Will throw an error if the directory doesn't exist and will overwrite the file if it already exists.
```rust
fs.write("src/text.txt", "testing");
```"#.into(),
        params: vector![
            Param { name: "path".into(), param_type: Type::Str, default: None },
            Param { name: "content".into(), param_type: Type::Union(vector![Type::Str, Type::Blob]), default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(FsIns::Write));
            Ok(instructions)
        })
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FsIns {
    Read,
    ReadString,
    Write,
}
#[typetag::serde(name = "FsIns")]
impl Instruction for FsIns {
    fn exec(&self, env: &mut ProcEnv, _graph: &mut Graph) -> Result<Option<Instructions> , Error> {
        match self {
            Self::Read => {
                if let Some(var) = env.stack.pop() {
                    let path = var.val.read().to_string();
                    match fs::read(path) {
                        Ok(contents) => {
                            env.stack.push(Variable::val(Val::Blob(contents)));
                        },
                        Err(error) => {
                            return Err(Error::FsReadError(error.to_string()));
                        }
                    }
                } else {
                    return Err(Error::FsReadStackError);
                }
            },
            Self::ReadString => {
                if let Some(var) = env.stack.pop() {
                    let path = var.val.read().to_string();
                    match fs::read_to_string(path) {
                        Ok(contents) => {
                            env.stack.push(Variable::val(Val::Str(contents.into())));
                        },
                        Err(error) => {
                            return Err(Error::FsReadStringError(error.to_string()));
                        }
                    }
                } else {
                    return Err(Error::FsReadStringStackError);
                }
            },
            Self::Write => { // path, then content
                if let Some(content) = env.stack.pop() {
                    if let Some(path) = env.stack.pop() {
                        let path = path.val.read().to_string();
                        match content.val.read().deref() {
                            Val::Blob(content) => {
                                match fs::write(path, content) {
                                    Ok(_) => {
                                        return Ok(None);
                                    },
                                    Err(error) => {
                                        return Err(Error::FsWriteError(error.to_string()));
                                    }
                                }
                            },
                            val => {
                                let content = val.to_string();
                                match fs::write(path, content) {
                                    Ok(_) => {
                                        return Ok(None);
                                    },
                                    Err(error) => {
                                        return Err(Error::FsWriteError(error.to_string()));
                                    }
                                }
                            }
                        }
                    }
                }
                return Err(Error::FsWriteStackError);
            },
        }
        Ok(None)   
    }
}
