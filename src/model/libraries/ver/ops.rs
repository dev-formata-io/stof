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
use crate::{model::{ver::{BUILD, CLEAR_BUILD, CLEAR_RELEASE, MAJOR, MINOR, PATCH, RELEASE, SET_BUILD, SET_MAJOR, SET_MINOR, SET_PATCH, SET_RELEASE, VER_LIB}, LibFunc, Param}, runtime::{instruction::Instructions, NumT, Type}};


/// Major.
pub fn ver_major() -> LibFunc {
    LibFunc {
        library: VER_LIB.clone(),
        name: "major".into(),
        is_async: false,
        docs: "# Major\nReturn the major portion of this version.".into(),
        params: vector![
            Param { name: "ver".into(), param_type: Type::Ver, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(MAJOR.clone());
            Ok(instructions)
        })
    }
}

/// Minor.
pub fn ver_minor() -> LibFunc {
    LibFunc {
        library: VER_LIB.clone(),
        name: "minor".into(),
        is_async: false,
        docs: "# Minor\nReturn the minor portion of this version.".into(),
        params: vector![
            Param { name: "ver".into(), param_type: Type::Ver, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(MINOR.clone());
            Ok(instructions)
        })
    }
}

/// Patch.
pub fn ver_patch() -> LibFunc {
    LibFunc {
        library: VER_LIB.clone(),
        name: "patch".into(),
        is_async: false,
        docs: "# Patch\nReturn the patch portion of this version.".into(),
        params: vector![
            Param { name: "ver".into(), param_type: Type::Ver, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(PATCH.clone());
            Ok(instructions)
        })
    }
}

/// Release.
pub fn ver_release() -> LibFunc {
    LibFunc {
        library: VER_LIB.clone(),
        name: "release".into(),
        is_async: false,
        docs: "# Release\nReturn the release portion of this version.".into(),
        params: vector![
            Param { name: "ver".into(), param_type: Type::Ver, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(RELEASE.clone());
            Ok(instructions)
        })
    }
}

/// Build.
pub fn ver_build() -> LibFunc {
    LibFunc {
        library: VER_LIB.clone(),
        name: "build".into(),
        is_async: false,
        docs: "# Build\nReturn the build portion of this version.".into(),
        params: vector![
            Param { name: "ver".into(), param_type: Type::Ver, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(BUILD.clone());
            Ok(instructions)
        })
    }
}

/// Set major.
pub fn ver_set_major() -> LibFunc {
    LibFunc {
        library: VER_LIB.clone(),
        name: "set_major".into(),
        is_async: false,
        docs: "# Set Major\nSet the major portion of this version (does not return anything).".into(),
        params: vector![
            Param { name: "ver".into(), param_type: Type::Ver, default: None },
            Param { name: "val".into(), param_type: Type::Num(NumT::Int), default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(SET_MAJOR.clone());
            Ok(instructions)
        })
    }
}

/// Set minor.
pub fn ver_set_minor() -> LibFunc {
    LibFunc {
        library: VER_LIB.clone(),
        name: "set_minor".into(),
        is_async: false,
        docs: "# Set Minor\nSet the minor portion of this version (does not return anything).".into(),
        params: vector![
            Param { name: "ver".into(), param_type: Type::Ver, default: None },
            Param { name: "val".into(), param_type: Type::Num(NumT::Int), default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(SET_MINOR.clone());
            Ok(instructions)
        })
    }
}

/// Set patch.
pub fn ver_set_patch() -> LibFunc {
    LibFunc {
        library: VER_LIB.clone(),
        name: "set_patch".into(),
        is_async: false,
        docs: "# Set Patch\nSet the patch portion of this version (does not return anything).".into(),
        params: vector![
            Param { name: "ver".into(), param_type: Type::Ver, default: None },
            Param { name: "val".into(), param_type: Type::Num(NumT::Int), default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(SET_PATCH.clone());
            Ok(instructions)
        })
    }
}

/// Set release.
pub fn ver_set_release() -> LibFunc {
    LibFunc {
        library: VER_LIB.clone(),
        name: "set_release".into(),
        is_async: false,
        docs: "# Set Release\nSet the release portion of this version (does not return anything).".into(),
        params: vector![
            Param { name: "ver".into(), param_type: Type::Ver, default: None },
            Param { name: "val".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(SET_RELEASE.clone());
            Ok(instructions)
        })
    }
}

/// Set build.
pub fn ver_set_build() -> LibFunc {
    LibFunc {
        library: VER_LIB.clone(),
        name: "set_build".into(),
        is_async: false,
        docs: "# Set Build\nSet the build portion of this version (does not return anything).".into(),
        params: vector![
            Param { name: "ver".into(), param_type: Type::Ver, default: None },
            Param { name: "val".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(SET_BUILD.clone());
            Ok(instructions)
        })
    }
}

/// Clear release.
pub fn ver_clear_release() -> LibFunc {
    LibFunc {
        library: VER_LIB.clone(),
        name: "clear_release".into(),
        is_async: false,
        docs: "# Clear Release\nRemoves the release portion of this version (does not return anything).".into(),
        params: vector![
            Param { name: "ver".into(), param_type: Type::Ver, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(CLEAR_RELEASE.clone());
            Ok(instructions)
        })
    }
}

/// Clear build.
pub fn ver_clear_build() -> LibFunc {
    LibFunc {
        library: VER_LIB.clone(),
        name: "clear_build".into(),
        is_async: false,
        docs: "# Clear Build\nRemoves the build portion of this version (does not return anything).".into(),
        params: vector![
            Param { name: "ver".into(), param_type: Type::Ver, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(CLEAR_BUILD.clone());
            Ok(instructions)
        })
    }
}
