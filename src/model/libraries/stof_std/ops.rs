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
use crate::{model::{stof_std::{BLOBIFY, PARSE, STD_LIB, STRINGIFY}, LibFunc, Param}, runtime::{instruction::Instructions, instructions::Base, Type, Val}};


/// Parse.
pub fn std_parse() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "parse".into(),
        is_async: false,
        docs: "# Parse\nParse additional data into the document, using any format available to the graph (stof, json, images, pdfs, etc.). The default format used is Stof (.stof) and the default context is 'self' (the calling object).".into(),
        params: vector![
            Param { name: "source".into(), param_type: Type::Void, default: None, },
            Param { name: "context".into(), param_type: Type::Void, default: Some(Arc::new(Base::Literal(Val::Null))), },
            Param { name: "format".into(), param_type: Type::Str, default: Some(Arc::new(Base::Literal(Val::Null))), }, // default is stof
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(PARSE.clone());
            Ok(instructions)
        })
    }
}

/// Stringify.
pub fn std_stringify() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "stringify".into(),
        is_async: false,
        docs: "# Stringify\nExport a portion of this graph as a string in the desired format. The default format is JSON.".into(),
        params: vector![
            Param { name: "format".into(), param_type: Type::Str, default: Some(Arc::new(Base::Literal(Val::Null))), },
            Param { name: "context".into(), param_type: Type::Void, default: Some(Arc::new(Base::Literal(Val::Null))), },
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(STRINGIFY.clone());
            Ok(instructions)
        })
    }
}

/// Blobify.
pub fn std_blobify() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "blobify".into(),
        is_async: false,
        docs: "# Blobify\nExport a portion of this graph as a binary blob in the desired format. The default format is JSON (as UTF-8 bytes).".into(),
        params: vector![
            Param { name: "format".into(), param_type: Type::Str, default: Some(Arc::new(Base::Literal(Val::Null))), },
            Param { name: "context".into(), param_type: Type::Void, default: Some(Arc::new(Base::Literal(Val::Null))), },
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(BLOBIFY.clone());
            Ok(instructions)
        })
    }
}
