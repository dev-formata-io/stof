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
use arcstr::{literal, ArcStr};
use imbl::vector;
use crate::{model::{Graph, LibFunc, Param}, runtime::{instruction::Instructions, instructions::tup::{AT_REF_TUP, AT_TUP, LEN_TUP}, NumT, Type}};


/// Library name.
pub(self) const TUP_LIB: ArcStr = literal!("Tup");


/// Insert tuple library.
pub fn insert_tup_lib(graph: &mut Graph) {
    graph.insert_libfunc(tup_len());
    graph.insert_libfunc(tup_at());
}


/// Length.
fn tup_len() -> LibFunc {
    LibFunc {
        library: TUP_LIB.clone(),
        name: "len".into(),
        is_async: false,
        docs: r#"# Tup.len(tup: (..)) -> int
Return the length of this tuple.
```rust
const tup = ("hi", 42, true);
assert_eq(tup.len(), 3);
```
"#.into(),
        params: vector![
            Param { name: "tup".into(), param_type: Type::Void, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(LEN_TUP.clone());
            Ok(instructions)
        })
    }
}


/// At.
fn tup_at() -> LibFunc {
    LibFunc {
        library: TUP_LIB.clone(),
        name: "at".into(),
        is_async: false,
        docs: r#"# Tup.at(tup: (..), index: int) -> unknown
Return the value (optionally by reference) at the given index in the tuple.
```rust
const tup = ("hi", 42, true);
assert_eq(&tup[1], 42);
```
"#.into(),
        params: vector![
            Param { name: "tup".into(), param_type: Type::Void, default: None },
            Param { name: "index".into(), param_type: Type::Num(NumT::Int), default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            if as_ref {
                instructions.push(AT_REF_TUP.clone());
            } else {
                instructions.push(AT_TUP.clone());
            }
            Ok(instructions)
        })
    }
}
