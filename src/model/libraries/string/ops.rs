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
use arcstr::literal;
use imbl::vector;
use crate::{model::{string::{AT, CONTAINS, ENDS_WITH, FIRST, INDEX_OF, LAST, LEN, LOWER, PUSH, REPLACE, SPLIT, STARTS_WITH, STR_LIB, SUBSTRING, TRIM, TRIM_END, TRIM_START, UPPER}, LibFunc, Param}, runtime::{instruction::Instructions, instructions::Base, Num, NumT, Type, Val}};


/// Len.
pub fn str_len() -> LibFunc {
    LibFunc {
        library: STR_LIB.clone(),
        name: "len".into(),
        is_async: false,
        docs: "# String Length\nReturn the length of a string.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(LEN.clone());
            Ok(instructions)
        })
    }
}

/// At.
pub fn str_at() -> LibFunc {
    LibFunc {
        library: STR_LIB.clone(),
        name: "at".into(),
        is_async: false,
        docs: "# String At (index op)\nReturn a char (as a string) at the given index.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Str, default: None },
            Param { name: "index".into(), param_type: Type::Num(NumT::Int), default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(AT.clone());
            Ok(instructions)
        })
    }
}

/// First.
pub fn str_first() -> LibFunc {
    LibFunc {
        library: STR_LIB.clone(),
        name: "first".into(),
        is_async: false,
        docs: "# First Char in String\nReturn the first char (as a string) in a string.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(FIRST.clone());
            Ok(instructions)
        })
    }
}

/// Last.
pub fn str_last() -> LibFunc {
    LibFunc {
        library: STR_LIB.clone(),
        name: "last".into(),
        is_async: false,
        docs: "# Last Char in String\nReturn the last char (as a string) in a string.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(LAST.clone());
            Ok(instructions)
        })
    }
}

/// Starts with?
pub fn str_starts_with() -> LibFunc {
    LibFunc {
        library: STR_LIB.clone(),
        name: "starts_with".into(),
        is_async: false,
        docs: "# String Starts With\nReturn true if the string starts with a given sequence.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Str, default: None },
            Param { name: "seq".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(STARTS_WITH.clone());
            Ok(instructions)
        })
    }
}

/// Ends with?
pub fn str_ends_with() -> LibFunc {
    LibFunc {
        library: STR_LIB.clone(),
        name: "ends_with".into(),
        is_async: false,
        docs: "# String Ends With\nReturn true if the string ends with a given sequence.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Str, default: None },
            Param { name: "seq".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(ENDS_WITH.clone());
            Ok(instructions)
        })
    }
}

/// Push.
pub fn str_push() -> LibFunc {
    LibFunc {
        library: STR_LIB.clone(),
        name: "push".into(),
        is_async: false,
        docs: "# Push\nPushes a string to the back of a string (concatination). Does not return anything.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Str, default: None },
            Param { name: "other".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(PUSH.clone());
            Ok(instructions)
        })
    }
}

/// Contains?
pub fn str_contains() -> LibFunc {
    LibFunc {
        library: STR_LIB.clone(),
        name: "contains".into(),
        is_async: false,
        docs: "# String Contains\nReturn true if the string contains a given sequence.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Str, default: None },
            Param { name: "seq".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(CONTAINS.clone());
            Ok(instructions)
        })
    }
}

/// Index Of.
pub fn str_index_of() -> LibFunc {
    LibFunc {
        library: STR_LIB.clone(),
        name: "index_of".into(),
        is_async: false,
        docs: "# Index Of\nReturn the index of a given squence (first char) if found, otherwise -1.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Str, default: None },
            Param { name: "seq".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(INDEX_OF.clone());
            Ok(instructions)
        })
    }
}

/// Replace.
pub fn str_replace() -> LibFunc {
    LibFunc {
        library: STR_LIB.clone(),
        name: "replace".into(),
        is_async: false,
        docs: "# String Replace\nReplace all instances of a find string with a replace string (defaults to an empty replace string, which removes all instances of the find string). Returns a new string without modifying the original.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Str, default: None },
            Param { name: "find".into(), param_type: Type::Str, default: None },
            Param { name: "replace".into(), param_type: Type::Str, default: Some(Arc::new(Base::Literal(Val::Str(literal!(""))))) }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(REPLACE.clone());
            Ok(instructions)
        })
    }
}

/// Split.
pub fn str_split() -> LibFunc {
    LibFunc {
        library: STR_LIB.clone(),
        name: "split".into(),
        is_async: false,
        docs: "# Split\nSplits a string into a list at the given separator. Default separator is a single space.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Str, default: None },
            Param { name: "sep".into(), param_type: Type::Str, default: Some(Arc::new(Base::Literal(Val::Str(" ".into())))) }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(SPLIT.clone());
            Ok(instructions)
        })
    }
}

/// Upper.
pub fn str_upper() -> LibFunc {
    LibFunc {
        library: STR_LIB.clone(),
        name: "upper".into(),
        is_async: false,
        docs: "# To Uppercase\nConverts all chars to uppercase.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(UPPER.clone());
            Ok(instructions)
        })
    }
}

/// Lower.
pub fn str_lower() -> LibFunc {
    LibFunc {
        library: STR_LIB.clone(),
        name: "lower".into(),
        is_async: false,
        docs: "# To Lowercase\nConverts all chars to lowercase.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(LOWER.clone());
            Ok(instructions)
        })
    }
}

/// Trim.
pub fn str_trim() -> LibFunc {
    LibFunc {
        library: STR_LIB.clone(),
        name: "trim".into(),
        is_async: false,
        docs: "# Trim\nTrims whitespace from the front and back of a string.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(TRIM.clone());
            Ok(instructions)
        })
    }
}

/// Trim start.
pub fn str_trim_start() -> LibFunc {
    LibFunc {
        library: STR_LIB.clone(),
        name: "trim_start".into(),
        is_async: false,
        docs: "# Trim Start\nTrims whitespace from the start of a string.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(TRIM_START.clone());
            Ok(instructions)
        })
    }
}

/// Trim end.
pub fn str_trim_end() -> LibFunc {
    LibFunc {
        library: STR_LIB.clone(),
        name: "trim_end".into(),
        is_async: false,
        docs: "# Trim End\nTrims whitespace from the end of a string.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(TRIM_END.clone());
            Ok(instructions)
        })
    }
}

/// Substring.
pub fn str_substr() -> LibFunc {
    LibFunc {
        library: STR_LIB.clone(),
        name: "substring".into(),
        is_async: false,
        docs: "# Substring\nReturn a substring from a start index to an optional end index (up to, but not including). Default end index is the length of the string.".into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Str, default: None },
            Param { name: "start".into(), param_type: Type::Num(NumT::Int), default: Some(Arc::new(Base::Literal(Val::Num(Num::Int(0))))) },
            Param { name: "end".into(), param_type: Type::Num(NumT::Int), default: Some(Arc::new(Base::Literal(Val::Num(Num::Int(-1))))) }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(SUBSTRING.clone());
            Ok(instructions)
        })
    }
}
