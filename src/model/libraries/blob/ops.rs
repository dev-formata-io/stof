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

use std::sync::Arc;
use imbl::vector;
use crate::{model::{blob::{AT_BLOB, BASE64_BLOB, BLOB_LIB, FROM_BASE64_BLOB, FROM_URL_SAFE_BLOB, FROM_UTF8_BLOB, LEN_BLOB, URL_SAFE_BLOB, UTF8_BLOB}, LibFunc, Param}, runtime::{instruction::Instructions, NumT, Type}};


/// Len.
pub fn blob_len() -> LibFunc {
    LibFunc {
        library: BLOB_LIB.clone(),
        name: "len".into(),
        is_async: false,
        docs: "# Length (size)\nSize of this binary blob (number of bytes).".into(),
        params: vector![
            Param { name: "blob".into(), param_type: Type::Blob, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(LEN_BLOB.clone());
            Ok(instructions)
        })
    }
}

/// At.
pub fn blob_at() -> LibFunc {
    LibFunc {
        library: BLOB_LIB.clone(),
        name: "at".into(),
        is_async: false,
        docs: "# At\nIndex into this blob at a specific byte.".into(),
        params: vector![
            Param { name: "blob".into(), param_type: Type::Blob, default: None },
            Param { name: "index".into(), param_type: Type::Num(NumT::Int), default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(AT_BLOB.clone());
            Ok(instructions)
        })
    }
}

/// Utf8.
pub fn blob_utf8() -> LibFunc {
    LibFunc {
        library: BLOB_LIB.clone(),
        name: "utf8".into(),
        is_async: false,
        docs: "# UTF-8 String\nGet the string version of this blob if its encoded as UTF-8.".into(),
        params: vector![
            Param { name: "blob".into(), param_type: Type::Blob, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(UTF8_BLOB.clone());
            Ok(instructions)
        })
    }
}

/// Base64.
pub fn blob_base64() -> LibFunc {
    LibFunc {
        library: BLOB_LIB.clone(),
        name: "base64".into(),
        is_async: false,
        docs: "# Base64 String\nEncode this blob as a base64 string.".into(),
        params: vector![
            Param { name: "blob".into(), param_type: Type::Blob, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(BASE64_BLOB.clone());
            Ok(instructions)
        })
    }
}

/// URL Safe Base64.
pub fn blob_url_base64() -> LibFunc {
    LibFunc {
        library: BLOB_LIB.clone(),
        name: "url_base64".into(),
        is_async: false,
        docs: "# URL-Safe Base64 String\nEncode this blob as a URL safe base64 string.".into(),
        params: vector![
            Param { name: "blob".into(), param_type: Type::Blob, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(URL_SAFE_BLOB.clone());
            Ok(instructions)
        })
    }
}

/// From utf8.
pub fn blob_from_utf8() -> LibFunc {
    LibFunc {
        library: BLOB_LIB.clone(),
        name: "from_utf8".into(),
        is_async: false,
        docs: "# From UTF-8 String\nUTF-8 string into a blob.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(FROM_UTF8_BLOB.clone());
            Ok(instructions)
        })
    }
}

/// From Base64.
pub fn blob_from_base64() -> LibFunc {
    LibFunc {
        library: BLOB_LIB.clone(),
        name: "from_base64".into(),
        is_async: false,
        docs: "# From Base64 String\nDecode a base 64 string into a blob.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(FROM_BASE64_BLOB.clone());
            Ok(instructions)
        })
    }
}

/// From URL Base64.
pub fn blob_from_url_base64() -> LibFunc {
    LibFunc {
        library: BLOB_LIB.clone(),
        name: "from_url_base64".into(),
        is_async: false,
        docs: "# From URL-Safe Base64 String\nDecode a URL safe base 64 string into a blob.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(FROM_URL_SAFE_BLOB.clone());
            Ok(instructions)
        })
    }
}
