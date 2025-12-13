//
// Copyright 2025 Formata, Inc. All rights reserved.
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

use core::str;
use std::{ops::Deref, sync::Arc};
use arcstr::{literal, ArcStr};
use base64::{engine::general_purpose::{STANDARD, URL_SAFE}, Engine as _};
use bytes::Bytes;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use crate::{model::{blob::ops::{blob_at, blob_base64, blob_from_base64, blob_from_url_base64, blob_from_utf8, blob_len, blob_size, blob_url_base64, blob_utf8}, Graph}, runtime::{instruction::{Instruction, Instructions}, proc::ProcEnv, Error, Num, Units, Val, Variable}};
mod ops;

/// Library name.
pub(self) const BLOB_LIB: ArcStr = literal!("Blob");


/// Add blob library to a graph.
pub fn insert_blob_lib(graph: &mut Graph) {
    graph.insert_libfunc(blob_len());
    graph.insert_libfunc(blob_at());
    graph.insert_libfunc(blob_size());
    graph.insert_libfunc(blob_utf8());
    graph.insert_libfunc(blob_base64());
    graph.insert_libfunc(blob_url_base64());

    graph.insert_libfunc(blob_from_utf8());
    graph.insert_libfunc(blob_from_base64());
    graph.insert_libfunc(blob_from_url_base64());
}


lazy_static! {
    pub(self) static ref LEN_BLOB: Arc<dyn Instruction> = Arc::new(BlobIns::Len);
    pub(self) static ref SIZE_BLOB: Arc<dyn Instruction> = Arc::new(BlobIns::Size);
    pub(self) static ref AT_BLOB: Arc<dyn Instruction> = Arc::new(BlobIns::At);
    pub(self) static ref UTF8_BLOB: Arc<dyn Instruction> = Arc::new(BlobIns::Utf8Str);
    pub(self) static ref BASE64_BLOB: Arc<dyn Instruction> = Arc::new(BlobIns::Base64Str);
    pub(self) static ref URL_SAFE_BLOB: Arc<dyn Instruction> = Arc::new(BlobIns::UrlSafeBase64Str);
    pub(self) static ref FROM_UTF8_BLOB: Arc<dyn Instruction> = Arc::new(BlobIns::FromUtf8Str);
    pub(self) static ref FROM_BASE64_BLOB: Arc<dyn Instruction> = Arc::new(BlobIns::FromBase64Str);
    pub(self) static ref FROM_URL_SAFE_BLOB: Arc<dyn Instruction> = Arc::new(BlobIns::FromUrlSafeBase64Str);
}


#[derive(Clone, Debug, Serialize, Deserialize)]
/// Blob ins.
pub enum BlobIns {
    Len,
    At,
    Size,

    Utf8Str,
    Base64Str,
    UrlSafeBase64Str,

    FromUtf8Str,
    FromBase64Str,
    FromUrlSafeBase64Str,
}
#[typetag::serde(name = "BlobIns")]
impl Instruction for BlobIns {
    fn exec(&self, env: &mut ProcEnv, _graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        match self {
            Self::Len => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Blob(blob) => {
                            env.stack.push(Variable::val(Val::Num(Num::Int(blob.len() as i64))));
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::BlobLen)
            },
            Self::Size => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Blob(blob) => {
                            env.stack.push(Variable::val(Val::Num(Num::Units(blob.len() as f64, Units::Bytes))));
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::BlobLen)
            },
            Self::At => {
                if let Some(index_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        match index_var.val.read().deref() {
                            Val::Num(num) => {
                                match var.val.read().deref() {
                                    Val::Blob(blob) => {
                                        let index = num.int() as usize;
                                        if index < blob.len() {
                                            env.stack.push(Variable::val(Val::Num(Num::Int(blob[index] as i64))));
                                            return Ok(None);
                                        }
                                    },
                                    _ => {}
                                }
                            },
                            _ => {}
                        }
                    }
                }
                Err(Error::BlobAt)
            },
            Self::Utf8Str => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Blob(blob) => {
                            if let Ok(res) = str::from_utf8(&blob) {
                                env.stack.push(Variable::val(Val::Str(res.into())));
                                return Ok(None);
                            }
                        },
                        _ => {}
                    }
                }
                Err(Error::BlobUtf8Str)
            },
            Self::Base64Str => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Blob(blob) => {
                            let res = STANDARD.encode(blob);
                            env.stack.push(Variable::val(Val::Str(res.into())));
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::BlobBase64Str)
            },
            Self::UrlSafeBase64Str => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Blob(blob) => {
                            let res = URL_SAFE.encode(blob);
                            env.stack.push(Variable::val(Val::Str(res.into())));
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::BlobUrlSafeBase64Str)
            },
            Self::FromUtf8Str => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Str(utf8) => {
                            let blob = str::as_bytes(utf8.as_str());
                            env.stack.push(Variable::val(Val::Blob(Bytes::copy_from_slice(blob))));
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::BlobFromUtf8Str)
            },
            Self::FromBase64Str => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Str(base64) => {
                            if let Ok(bytes) = STANDARD.decode(base64.as_str()) {
                                env.stack.push(Variable::val(Val::Blob(bytes.into())));
                                return Ok(None);
                            }
                        },
                        _ => {}
                    }
                }
                Err(Error::BlobFromBase64Str)
            },
            Self::FromUrlSafeBase64Str => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Str(base64) => {
                            if let Ok(bytes) = URL_SAFE.decode(base64.as_str()) {
                                env.stack.push(Variable::val(Val::Blob(bytes.into())));
                                return Ok(None);
                            }
                        },
                        _ => {}
                    }
                }
                Err(Error::BlobFromUrlSafeBase64Str)
            },
        }
    }
}
