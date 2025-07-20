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
use crate::{model::{libraries::function::{FuncIns, ATTRIBUTES, DATA, FUNC_LIB, HAS_ATTR, ID, IS_ASYNC, NAME, OBJ, OBJS, PARAMS, RETURN_TYPE}, LibFunc, Param}, runtime::{instruction::Instructions, Type}};


/// Id.
pub fn fn_id() -> LibFunc {
    LibFunc {
        library: FUNC_LIB.clone(),
        name: "id".into(),
        is_async: false,
        docs: "# ID\nReturns the ID of a function reference.".into(),
        params: vector![
            Param { name: "fn".into(), param_type: Type::Fn, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(ID.clone());
            Ok(instructions)
        })
    }
}

/// Data.
pub fn fn_data() -> LibFunc {
    LibFunc {
        library: FUNC_LIB.clone(),
        name: "data".into(),
        is_async: false,
        docs: "# Data\nConverts a function reference into a generic data reference.".into(),
        params: vector![
            Param { name: "fn".into(), param_type: Type::Fn, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(DATA.clone());
            Ok(instructions)
        })
    }
}

/// Name.
pub fn fn_name() -> LibFunc {
    LibFunc {
        library: FUNC_LIB.clone(),
        name: "name".into(),
        is_async: false,
        docs: "# Name\nReturns the name of a function.".into(),
        params: vector![
            Param { name: "fn".into(), param_type: Type::Fn, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(NAME.clone());
            Ok(instructions)
        })
    }
}

/// Params.
pub fn fn_params() -> LibFunc {
    LibFunc {
        library: FUNC_LIB.clone(),
        name: "params".into(),
        is_async: false,
        docs: "# Parameters\nReturns a list of tuples containing the name and type of each expected parameter.".into(),
        params: vector![
            Param { name: "fn".into(), param_type: Type::Fn, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(PARAMS.clone());
            Ok(instructions)
        })
    }
}

/// Return type.
pub fn fn_return_type() -> LibFunc {
    LibFunc {
        library: FUNC_LIB.clone(),
        name: "return_type".into(),
        is_async: false,
        docs: "# Return Type\nReturns a string (typeof) for this functions return type.".into(),
        params: vector![
            Param { name: "fn".into(), param_type: Type::Fn, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(RETURN_TYPE.clone());
            Ok(instructions)
        })
    }
}

/// Has attribute?
pub fn fn_has_attr() -> LibFunc {
    LibFunc {
        library: FUNC_LIB.clone(),
        name: "has_attribute".into(),
        is_async: false,
        docs: "# Has Attribute?\nReturns true if this function has an attribute with the requested name.".into(),
        params: vector![
            Param { name: "fn".into(), param_type: Type::Fn, default: None },
            Param { name: "name".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(HAS_ATTR.clone());
            Ok(instructions)
        })
    }
}

/// Attributes.
pub fn fn_attributes() -> LibFunc {
    LibFunc {
        library: FUNC_LIB.clone(),
        name: "attributes".into(),
        is_async: false,
        docs: "# Attributes\nReturns a map of this functions attributes (with values).".into(),
        params: vector![
            Param { name: "fn".into(), param_type: Type::Fn, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(ATTRIBUTES.clone());
            Ok(instructions)
        })
    }
}

/// Obj.
pub fn fn_obj() -> LibFunc {
    LibFunc {
        library: FUNC_LIB.clone(),
        name: "obj".into(),
        is_async: false,
        docs: "# Object\nReturns the first object reference found that this function is attached to.".into(),
        params: vector![
            Param { name: "fn".into(), param_type: Type::Fn, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(OBJ.clone());
            Ok(instructions)
        })
    }
}

/// Objs.
pub fn fn_objs() -> LibFunc {
    LibFunc {
        library: FUNC_LIB.clone(),
        name: "objs".into(),
        is_async: false,
        docs: "# Objects\nReturns a list of object references that this function is attached to.".into(),
        params: vector![
            Param { name: "fn".into(), param_type: Type::Fn, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(OBJS.clone());
            Ok(instructions)
        })
    }
}

/// Is Async?
pub fn fn_is_async() -> LibFunc {
    LibFunc {
        library: FUNC_LIB.clone(),
        name: "is_async".into(),
        is_async: false,
        docs: "# Is Async?\nReturns true if this function is an async function (has an 'async' attribute).".into(),
        params: vector![
            Param { name: "fn".into(), param_type: Type::Fn, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(IS_ASYNC.clone());
            Ok(instructions)
        })
    }
}

/// Call.
pub fn fn_call() -> LibFunc {
    LibFunc {
        library: FUNC_LIB.clone(),
        name: "call".into(),
        is_async: false,
        docs: "# Call Function\nWill call this function with the provided arguments.".into(),
        params: vector![
            Param { name: "fn".into(), param_type: Type::Fn, default: None }
            // Unbounded parameters after the first function reference
        ],
        return_type: None,
        unbounded_args: true,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(FuncIns::Call(arg_count)));
            Ok(instructions)
        })
    }
}

/// Expanded Call.
pub fn fn_exp_call() -> LibFunc {
    LibFunc {
        library: FUNC_LIB.clone(),
        name: "call_expanded".into(),
        is_async: false,
        docs: "# Expanded Call Function\nWill call this function but will expand the arguments out if they are containers. For example, providing a list of values here will result in each individual list value as a separate function argument.".into(),
        params: vector![
            Param { name: "fn".into(), param_type: Type::Fn, default: None }
            // Unbounded parameters after the first function reference
        ],
        return_type: None,
        unbounded_args: true,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(FuncIns::ExpandCall(arg_count)));
            Ok(instructions)
        })
    }
}
