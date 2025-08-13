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
        docs: r#"# Ver.major(ver: ver) -> int
Return the major portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
assert_eq(ver.major(), 1);
```
"#.into(),
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
        docs: r#"# Ver.minor(ver: ver) -> int
Return the minor portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
assert_eq(ver.minor(), 2);
```
"#.into(),
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
        docs: r#"# Ver.patch(ver: ver) -> int
Return the patch portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
assert_eq(ver.patch(), 3);
```
"#.into(),
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
        docs: r#"# Ver.release(ver: ver) -> str
Return the release portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
assert_eq(ver.release(), "release");
```
"#.into(),
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
        docs: r#"# Ver.build(ver: ver) -> str
Return the build portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
assert_eq(ver.build(), "build");
```
"#.into(),
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
        docs: r#"# Ver.set_major(ver: ver, val: int) -> void
Set the major portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
ver.set_major(4);
assert_eq(ver, 4.2.3-release+build);
```
"#.into(),
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
        docs: r#"# Ver.set_minor(ver: ver, val: int) -> void
Set the minor portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
ver.set_minor(4);
assert_eq(ver, 1.4.3-release+build);
```
"#.into(),
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
        docs: r#"# Ver.set_patch(ver: ver, val: int) -> void
Set the patch portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
ver.set_patch(4);
assert_eq(ver, 1.2.4-release+build);
```
"#.into(),
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
        docs: r#"# Ver.set_release(ver: ver, val: str) -> void
Set the release portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
ver.set_release("modified");
assert_eq(ver, 1.2.3-modified+build);
```
"#.into(),
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
        docs: r#"# Ver.set_build(ver: ver, val: str) -> void
Set the build portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
ver.set_build("modified");
assert_eq(ver, 1.2.3-release+modified);
```
"#.into(),
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
        docs: r#"# Ver.clear_release(ver: ver) -> void
Clear the release portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
ver.clear_release();
assert_eq(ver, 1.2.3+build);
```
"#.into(),
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
        docs: r#"# Ver.clear_build(ver: ver) -> void
Clear the build portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
ver.clear_build();
assert_eq(ver, 1.2.3-release);
```
"#.into(),
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
