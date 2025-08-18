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
use crate::{model::{blob::{AT_BLOB, BASE64_BLOB, BLOB_LIB, FROM_BASE64_BLOB, FROM_URL_SAFE_BLOB, FROM_UTF8_BLOB, LEN_BLOB, SIZE_BLOB, URL_SAFE_BLOB, UTF8_BLOB}, LibFunc, Param}, runtime::{instruction::Instructions, NumT, Type}};


/// Len.
pub fn blob_len() -> LibFunc {
    LibFunc {
        library: BLOB_LIB.clone(),
        name: "len".into(),
        is_async: false,
        docs: r#"# Blob.len(bytes: blob) -> int
Size of this binary blob (integer number of bytes).
```rust
const bytes: blob = "hello";
assert_eq(bytes.len(), 5);
```"#.into(),
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

/// Size (bytes).
pub fn blob_size() -> LibFunc {
    LibFunc {
        library: BLOB_LIB.clone(),
        name: "size".into(),
        is_async: false,
        docs: r#"# Blob.size(bytes: blob) -> bytes
Size of this binary blob (in units of bytes).
```rust
const bytes: blob = "hello";
assert_eq(bytes.size(), 5bytes);
```"#.into(),
        params: vector![
            Param { name: "blob".into(), param_type: Type::Blob, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(SIZE_BLOB.clone());
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
        docs: r#"# Blob.at(bytes: blob, index: int) -> int
Byte at a specific index within this blob.
```rust
const bytes: blob = "hello";
assert_eq(bytes[1], 101); // or '.at(1)' or 'Blob.at(bytes, 1)'
```"#.into(),
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
        docs: r#"# Blob.utf8(bytes: blob) -> str
Transform this blob into a string using UTF-8 (default conversion for casts also).
```rust
const bytes: blob = "hello";
assert_eq(bytes.utf8(), "hello");
assert_eq(bytes as str, "hello");
```"#.into(),
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
        docs: r#"# Blob.base64(bytes: blob) -> str
Transform this blob into a string using Base64 encoding.
```rust
const bytes: blob = "hello";
assert_eq(bytes.base64(), "aGVsbG8=");
```"#.into(),
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
        docs: r#"# Blob.url_base64(bytes: blob) -> str
Transform this blob into a string using URL-safe Base64 encoding.
```rust
const bytes: blob = "hello";
assert_eq(bytes.url_base64(), "aGVsbG8=");
```"#.into(),
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
        docs: r#"# Blob.from_utf8(val: str) -> blob
Transform a string into a blob, using standard UTF-8 encoding (default for normal casts too).
```rust
const bytes: blob = "hello";
assert_eq(bytes, Blob.from_utf8("hello"));
```"#.into(),
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
        docs: r#"# Blob.from_base64(val: str) -> blob
Transform a string into a blob, using Base64 encoding.
```rust
const bytes: blob = Blob.from_base64("aGVsbG8=");
assert_eq(bytes as str, "hello");
```"#.into(),
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
        docs: r#"# Blob.from_url_base64(val: str) -> blob
Transform a string into a blob, using URL-safe Base64 encoding.
```rust
const bytes: blob = Blob.from_url_base64("aGVsbG8=");
assert_eq(bytes as str, "hello");
```"#.into(),
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
