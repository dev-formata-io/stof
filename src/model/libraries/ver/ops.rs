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

use std::sync::Arc;
use crate::{model::{ver::{BUILD, BUMP_MAJOR, BUMP_MINOR, BUMP_PATCH, CLEAR_BUILD, CLEAR_RELEASE, COMPATIBLE, IS_PRERELEASE, MAJOR, MATCHES, MINOR, PATCH, RELEASE, SET_BUILD, SET_MAJOR, SET_MINOR, SET_PATCH, SET_RELEASE, VER_LIB}, LibFunc, Param}, runtime::{instruction::Instructions, NumT, Type}};


/// Major.
pub fn ver_major() -> LibFunc {
    LibFunc {
        library: VER_LIB.clone(),
        name: "major".into(),
        is_async: false,
        docs: r#"# Ver::major(ver: ver) -> int
Return the major portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
assert_eq(ver.major(), 1);
```
"#.into(),
        params: crate::params![
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
        docs: r#"# Ver::minor(ver: ver) -> int
Return the minor portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
assert_eq(ver.minor(), 2);
```
"#.into(),
        params: crate::params![
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
        docs: r#"# Ver::patch(ver: ver) -> int
Return the patch portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
assert_eq(ver.patch(), 3);
```
"#.into(),
        params: crate::params![
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
        docs: r#"# Ver::release(ver: ver) -> str
Return the release portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
assert_eq(ver.release(), "release");
```
"#.into(),
        params: crate::params![
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
        docs: r#"# Ver::build(ver: ver) -> str
Return the build portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
assert_eq(ver.build(), "build");
```
"#.into(),
        params: crate::params![
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
        docs: r#"# Ver::set_major(ver: ver, val: int) -> void
Set the major portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
ver.set_major(4);
assert_eq(ver, 4.2.3-release+build);
```
"#.into(),
        params: crate::params![
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
        docs: r#"# Ver::set_minor(ver: ver, val: int) -> void
Set the minor portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
ver.set_minor(4);
assert_eq(ver, 1.4.3-release+build);
```
"#.into(),
        params: crate::params![
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
        docs: r#"# Ver::set_patch(ver: ver, val: int) -> void
Set the patch portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
ver.set_patch(4);
assert_eq(ver, 1.2.4-release+build);
```
"#.into(),
        params: crate::params![
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
        docs: r#"# Ver::set_release(ver: ver, val: str) -> void
Set the release portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
ver.set_release("modified");
assert_eq(ver, 1.2.3-modified+build);
```
"#.into(),
        params: crate::params![
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
        docs: r#"# Ver::set_build(ver: ver, val: str) -> void
Set the build portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
ver.set_build("modified");
assert_eq(ver, 1.2.3-release+modified);
```
"#.into(),
        params: crate::params![
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
        docs: r#"# Ver::clear_release(ver: ver) -> void
Clear the release portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
ver.clear_release();
assert_eq(ver, 1.2.3+build);
```
"#.into(),
        params: crate::params![
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
        docs: r#"# Ver::clear_build(ver: ver) -> void
Clear the build portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
ver.clear_build();
assert_eq(ver, 1.2.3-release);
```
"#.into(),
        params: crate::params![
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


/// Matches a requirement?
pub fn ver_matches() -> LibFunc {
    LibFunc {
        library: VER_LIB.clone(),
        name: "matches".into(),
        is_async: false,
        docs: r#"# Ver::matches(ver: ver, req: str) -> bool
Does this version match a requirement, as Cargo's: `^1.2.3` (or a bare `1.2.3`: compatible versions), `~1.2` (patch updates), `=1.2.3`, `>1.2`, `>=1.2`, `<2`, `<=2.1`, `1.2.*`, or `*`, separated by commas (all must match). A pre-release only matches a requirement with a pre-release of its own version.
```rust
const ver = 1.4.2;
assert(ver.matches('^1.2'));
assert(!ver.matches('>=2.0.0, <3.0.0'));
const pre = 1.5.0-rc.2;
assert(!pre.matches('^1.4'));
assert(pre.matches('>=1.5.0-rc.1'));
```
"#.into(),
        params: crate::params![
            Param { name: "ver".into(), param_type: Type::Ver, default: None },
            Param { name: "req".into(), param_type: Type::Str, default: None }
        ],
        return_type: Some(Type::Bool),
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(MATCHES.clone());
            Ok(instructions)
        })
    }
}

/// Compatible with another version?
pub fn ver_compatible() -> LibFunc {
    LibFunc {
        library: VER_LIB.clone(),
        name: "compatible".into(),
        is_async: false,
        docs: r#"# Ver::compatible(ver: ver, other: ver) -> bool
Can the newer of these versions stand in for the older (as `^older` matches it)? The same major version, or for 0.x the same minor version (0.0.x: the same version). A pre-release goes only with its own version's pre-releases and the releases from that version on.
```rust
const ver = 1.2.0;
assert(ver.compatible(1.9.3));
assert(!ver.compatible(2.0.0));
```
"#.into(),
        params: crate::params![
            Param { name: "ver".into(), param_type: Type::Ver, default: None },
            Param { name: "other".into(), param_type: Type::Ver, default: None }
        ],
        return_type: Some(Type::Bool),
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(COMPATIBLE.clone());
            Ok(instructions)
        })
    }
}

/// Is a pre-release?
pub fn ver_is_prerelease() -> LibFunc {
    LibFunc {
        library: VER_LIB.clone(),
        name: "is_prerelease".into(),
        is_async: false,
        docs: r#"# Ver::is_prerelease(ver: ver) -> bool
Is this version a pre-release (it has a release part, Ex. `1.0.0-rc.1`)?
```rust
const pre = 1.0.0-rc.1;
assert(pre.is_prerelease());
const release = 1.0.0+build;
assert(!release.is_prerelease());
```
"#.into(),
        params: crate::params![
            Param { name: "ver".into(), param_type: Type::Ver, default: None }
        ],
        return_type: Some(Type::Bool),
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(IS_PRERELEASE.clone());
            Ok(instructions)
        })
    }
}

/// Bump the major version.
pub fn ver_bump_major() -> LibFunc {
    LibFunc {
        library: VER_LIB.clone(),
        name: "bump_major".into(),
        is_async: false,
        docs: r#"# Ver::bump_major(ver: ver) -> void
Bump this version to its next major version (minor and patch to 0; the release and build cleared). A pre-release of a major version becomes that version.
```rust
const ver = 1.2.3-rc.1+build;
ver.bump_major();
assert_eq(ver, 2.0.0);
const pre = 2.0.0-rc.1;
pre.bump_major();
assert_eq(pre, 2.0.0);
```
"#.into(),
        params: crate::params![
            Param { name: "ver".into(), param_type: Type::Ver, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(BUMP_MAJOR.clone());
            Ok(instructions)
        })
    }
}

/// Bump the minor version.
pub fn ver_bump_minor() -> LibFunc {
    LibFunc {
        library: VER_LIB.clone(),
        name: "bump_minor".into(),
        is_async: false,
        docs: r#"# Ver::bump_minor(ver: ver) -> void
Bump this version to its next minor version (patch to 0; the release and build cleared). A pre-release of a minor version becomes that version.
```rust
const ver = 1.2.3;
ver.bump_minor();
assert_eq(ver, 1.3.0);
const pre = 1.3.0-rc.1;
pre.bump_minor();
assert_eq(pre, 1.3.0);
```
"#.into(),
        params: crate::params![
            Param { name: "ver".into(), param_type: Type::Ver, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(BUMP_MINOR.clone());
            Ok(instructions)
        })
    }
}

/// Bump the patch version.
pub fn ver_bump_patch() -> LibFunc {
    LibFunc {
        library: VER_LIB.clone(),
        name: "bump_patch".into(),
        is_async: false,
        docs: r#"# Ver::bump_patch(ver: ver) -> void
Bump this version to its next patch version (the release and build cleared). A pre-release becomes its version.
```rust
const ver = 1.2.3;
ver.bump_patch();
assert_eq(ver, 1.2.4);
const pre = 1.2.4-rc.1;
pre.bump_patch();
assert_eq(pre, 1.2.4);
```
"#.into(),
        params: crate::params![
            Param { name: "ver".into(), param_type: Type::Ver, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(BUMP_PATCH.clone());
            Ok(instructions)
        })
    }
}
