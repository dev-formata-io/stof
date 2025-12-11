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

use arcstr::ArcStr;
use imbl::vector;
use std::{ops::Deref, sync::Arc};
use serde::{Deserialize, Serialize};
use crate::{model::{Graph, LibFunc, Param, Profile, stof_std::STD_LIB}, runtime::{Error, Type, Val, Variable, instruction::{Instruction, Instructions}, proc::ProcEnv}};


/// Insert the profile library into this graph.
pub fn insert_profile_lib(graph: &mut Graph, profile: &Profile) {
    // Std.prof(name: str) -> bool
    let prof_name: ArcStr = profile.name.clone().into();
    graph.insert_libfunc(LibFunc {
        library: STD_LIB.clone(),
        name: "prof".into(),
        is_async: false,
        docs: r#"# Std.prof(name: str) -> bool
Is/was this graph parsed last with the given profile name?
```rust
// is the current profile named "test"?
const test = prof('test');
```
"#.into(),
        params: vector![
            Param { name: "name".into(), param_type: Type::Str, default: None, },
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(move |_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(ProfIns::NameCheck(prof_name.clone())));
            Ok(instructions)
        })
    });
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Profile instructions.
pub enum ProfIns {
    NameCheck(ArcStr),
}
#[typetag::serde(name = "ProfIns")]
impl Instruction for ProfIns {
    fn exec(&self, env: &mut ProcEnv, _graph: &mut Graph) -> Result<Option<Instructions>, crate::runtime::Error> {
        match self {
            Self::NameCheck(prof_name) => {
                if let Some(name_var) = env.stack.pop() {
                    match name_var.val.read().deref() {
                        Val::Str(name) => {
                            env.stack.push(Variable::val(Val::Bool(name == prof_name)));
                            return Ok(None);
                        },
                        _ => {}
                    }
                }
                Err(Error::Custom("Profile NameCheck Error".into()))
            },
        }
    }
}
