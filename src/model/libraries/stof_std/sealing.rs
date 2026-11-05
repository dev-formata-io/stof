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

//! Sealing from Stof (`Std::seal`, `Std::digest`). A document can seal parts of itself, so code that runs later
//! (an agent's function, a rule added at runtime) can call them but not change them. It's one-way: only the host
//! can unseal. See `model::seal`.

use std::sync::Arc;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use crate::{model::{Graph, LibFunc, Param, stof_std::STD_LIB}, runtime::{Error, Type, Val, Variable, instruction::{Instruction, Instructions}, instructions::Base, proc::ProcEnv}};


lazy_static! {
    static ref SEAL: Arc<dyn Instruction> = Arc::new(SealIns::Seal);
    static ref DIGEST: Arc<dyn Instruction> = Arc::new(SealIns::Digest);
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Seal instructions (their own type, so BSTF's variant indexes for the other libraries never shift).
pub enum SealIns {
    /// (obj, fields) -> digest, sealing it.
    Seal,
    /// (obj, fields) -> digest.
    Digest,
}
#[typetag::serde(name = "SealIns")]
impl Instruction for SealIns {
    fn exec(&self, env: &mut ProcEnv, graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        let fields = env.stack.pop().map(|var| var.val.read().truthy()).unwrap_or(false);
        let target = env.stack.pop().and_then(|var| var.try_obj());
        let Some(node) = target else {
            return Err(Error::Sealed(format!("{} needs an object (Ex. self.rules, or a type: <Engine>)", if matches!(self, Self::Seal) { "seal" } else { "digest" })));
        };
        let digest = match self {
            Self::Seal => graph.seal(&node, fields)?,
            Self::Digest => graph.digest(&node, fields)?,
        };
        env.stack.push(Variable::val(Val::Str(digest.into())));
        Ok(None)
    }
}


/// Std::seal.
pub fn std_seal() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "seal".into(),
        is_async: false,
        docs: r#"# Std::seal(obj: obj, fields: bool = false) -> str
Seal an object (Ex. `self.rules`, or a type: `<Engine>`) and everything under it: from now on its functions can be called but not added, removed, renamed, replaced, or moved, and its attributes can't change, by this document's code or by the host. Its fields still change unless `fields` is true. Returns its digest (see `digest`). It's one-way: a document can't unseal, only the host can. A change a seal refuses fails with a `sealed: ...` error and changes nothing.
```rust
#[type] Engine: { fn price(units: int) -> int { units * 50 } }

#[init]
fn lock() { seal(<Engine>); }

fn later() {
    parse("fn price(units: int) -> int { 0 }", <Engine>); // error: sealed
}
```
"#.into(),
        params: crate::params![
            Param { name: "obj".into(), param_type: Type::Void, default: None },
            Param { name: "fields".into(), param_type: Type::Bool, default: Some(Arc::new(Base::Literal(Val::Bool(false)))) },
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(SEAL.clone());
            Ok(instructions)
        })
    }
}

/// Std::digest.
pub fn std_digest() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "digest".into(),
        is_async: false,
        docs: r#"# Std::digest(obj: obj, fields: bool = false) -> str
A SHA-256 (hex) over an object's functions and structure (its functions' names, parameters, return types, attributes, and instructions, and the objects, types, and fields that hold functions or objects under it), plus every field's value when `fields` is true. The same for the same document, whether it was parsed or loaded from BSTF, so it identifies exactly the logic it covers. Hosts check it when they seal (`doc.seal(path, { digest })`) or verify a signature over it.
```rust
const before = digest(self.rules);
assert_eq(digest(self.rules), before);
```
"#.into(),
        params: crate::params![
            Param { name: "obj".into(), param_type: Type::Void, default: None },
            Param { name: "fields".into(), param_type: Type::Bool, default: Some(Arc::new(Base::Literal(Val::Bool(false)))) },
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(DIGEST.clone());
            Ok(instructions)
        })
    }
}
