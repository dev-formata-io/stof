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
use crate::{model::{stof_std::{StdIns, BLOBIFY, CALLSTACK, FORMATS, FORMAT_CONTENT_TYPE, GRAPH_ID, HAS_FORMAT, HAS_LIB, LIBS, NANO_ID, PARSE, STD_LIB, STRINGIFY}, LibFunc, Param}, runtime::{instruction::Instructions, instructions::Base, Num, NumT, Type, Val}};


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

/// Has format?
pub fn std_has_format() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "has_format".into(),
        is_async: false,
        docs: "# Has Format?\nReturn true if a given format is available in this graph.".into(),
        params: vector![
            Param { name: "format".into(), param_type: Type::Str, default: None, },
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(HAS_FORMAT.clone());
            Ok(instructions)
        })
    }
}

/// Formats.
pub fn std_formats() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "formats".into(),
        is_async: false,
        docs: "# Formats\nReturns a set of all available formats (for parse, stringify, blobify, etc.).".into(),
        params: vector![],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(FORMATS.clone());
            Ok(instructions)
        })
    }
}

/// Format content type.
pub fn std_format_content_type() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "format_content_type".into(),
        is_async: false,
        docs: "# Format Content Type\nReturns the requested format's content type, or null if the format is not available. Ex. assert_eq(format_content_type('json'), 'application/json')".into(),
        params: vector![
            Param { name: "format".into(), param_type: Type::Str, default: None, },
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(FORMAT_CONTENT_TYPE.clone());
            Ok(instructions)
        })
    }
}

/// Has lib?
pub fn std_has_lib() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "has_lib".into(),
        is_async: false,
        docs: "# Has Library?\nReturn true if a given library is available in this graph.".into(),
        params: vector![
            Param { name: "lib".into(), param_type: Type::Str, default: None, },
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(HAS_LIB.clone());
            Ok(instructions)
        })
    }
}

/// Libs.
pub fn std_libs() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "libs".into(),
        is_async: false,
        docs: "# Libs\nReturns a set of all available libraries.".into(),
        params: vector![],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(LIBS.clone());
            Ok(instructions)
        })
    }
}

/// Nanoid
pub fn std_nanoid() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "nanoid".into(),
        is_async: false,
        docs: "# Nano ID\nGenerate a new nanoid string (URL safe). Default lenght is 21 characters.".into(),
        params: vector![
            Param { name: "length".into(), param_type: Type::Num(NumT::Int), default: Some(Arc::new(Base::Literal(Val::Num(Num::Int(21))))), },
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(NANO_ID.clone());
            Ok(instructions)
        })
    }
}

/// Graph ID.
pub fn std_graph_id() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "graph_id".into(),
        is_async: false,
        docs: "# Graph ID\nReturn this graph's unique ID.".into(),
        params: vector![],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(GRAPH_ID.clone());
            Ok(instructions)
        })
    }
}

/// Max value library function.
pub fn std_max() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "max".into(),
        is_async: false,
        docs: "# Maximum Value\nReturn the maximum value for all parameters given (unbounded). If a list or set is provided, this will contemplate the max value in that collection. Will consider units if provided as well.".into(),
        params: vector![],
        return_type: None,
        unbounded_args: true,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(StdIns::Max(arg_count)));
            Ok(instructions)
        })
    }
}


/// Min value library function.
pub fn std_min() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "min".into(),
        is_async: false,
        docs: "# Minimum Value\nReturn the minimum value for all parameters given (unbounded). If a list or set is provided, this will contemplate the min value in that collection. Will consider units if provided as well.".into(),
        params: vector![],
        return_type: None,
        unbounded_args: true,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(StdIns::Min(arg_count)));
            Ok(instructions)
        })
    }
}

/// Callstack.
pub fn std_callstack() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "callstack".into(),
        is_async: false,
        docs: "# Callstack\nReturn the current callstack as a list of functions (last function is 'this').".into(),
        params: vector![],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(CALLSTACK.clone());
            Ok(instructions)
        })
    }
}

/// Trace.
pub fn std_trace() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "trace".into(),
        is_async: false,
        docs: "# Trace\nTrace this location in the current process.".into(),
        params: vector![],
        return_type: None,
        unbounded_args: true,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(StdIns::Trace(arg_count)));
            Ok(instructions)
        })
    }
}
