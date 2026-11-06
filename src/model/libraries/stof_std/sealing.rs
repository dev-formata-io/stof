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

//! Sealing from Stof (`Std::seal`, `Std::digest`, `Std::sealed`). A document can seal parts of itself, so code
//! that runs later (an agent's function, a rule added at runtime) can call them but not change them, and confine
//! them to what they may reach. It's one-way: only the host can unseal. See `model::seal`.

use std::sync::Arc;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use crate::{model::{Graph, LibFunc, Param, seal::SealOptions, stof_std::STD_LIB}, runtime::{Error, Type, Val, Variable, instruction::{Instruction, Instructions}, instructions::Base, proc::ProcEnv}};


lazy_static! {
    static ref SEAL: Arc<dyn Instruction> = Arc::new(SealIns::Seal);
    static ref DIGEST: Arc<dyn Instruction> = Arc::new(SealIns::Digest);
    static ref SEALED: Arc<dyn Instruction> = Arc::new(SealIns::Sealed);
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Seal instructions (their own type, so BSTF's variant indexes for the other libraries never shift).
pub enum SealIns {
    /// (obj, fields, removable, reads, hosts) -> digest, sealing it.
    Seal,
    /// (obj, fields) -> digest.
    Digest,
    /// (obj) -> map | null.
    Sealed,
}
#[typetag::serde(name = "SealIns")]
impl Instruction for SealIns {
    fn exec(&self, env: &mut ProcEnv, graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        if matches!(self, Self::Sealed) {
            let target = env.stack.pop().and_then(|var| var.try_obj());
            let Some(node) = target else { return Err(Error::Sealed("sealed needs an object (Ex. self.tools, or a type: <Engine>)".into())) };
            let info = graph.seal_info(&node).map(|info| info.to_val(graph)).unwrap_or(Val::Null);
            env.stack.push(Variable::val(info));
            return Ok(None);
        }

        let mut options = SealOptions::default();
        if matches!(self, Self::Seal) {
            let hosts = env.stack.pop().map(|var| var.val.read().clone()).unwrap_or(Val::Null);
            let reads = env.stack.pop().map(|var| var.val.read().clone()).unwrap_or(Val::Null);
            options.removable = env.stack.pop().map(|var| var.val.read().truthy()).unwrap_or(false);
            options.hosts = match hosts {
                Val::Null | Val::Void => None,
                Val::List(items) => {
                    let mut hosts = Vec::new();
                    for item in items {
                        match &*item.read() {
                            Val::Str(host) => hosts.push(host.to_string()),
                            _ => return Err(Error::Sealed("seal hosts must be a list of host names (Ex. ['api.example.com'])".into())),
                        }
                    }
                    Some(hosts)
                },
                _ => return Err(Error::Sealed("seal hosts must be a list of host names (Ex. ['api.example.com'])".into())),
            };
            // objects are kept as their paths (looked up each time the part is entered); paths can be given too
            options.reads = match reads {
                Val::Null | Val::Void => None,
                Val::List(items) => {
                    let mut reads = Vec::new();
                    for item in items {
                        match &*item.read() {
                            Val::Obj(nref) => match nref.node_path(graph, true) {
                                Some(path) => reads.push(path.join(".")),
                                None => return Err(Error::Sealed("seal reads: one of the objects doesn't exist".into())),
                            },
                            Val::Str(path) => reads.push(path.to_string()),
                            _ => return Err(Error::Sealed("seal reads must be a list of objects or paths (Ex. [self.inputs, <Helpers>])".into())),
                        }
                    }
                    Some(reads)
                },
                _ => return Err(Error::Sealed("seal reads must be a list of objects or paths (Ex. [self.inputs, <Helpers>])".into())),
            };
        }
        options.fields = env.stack.pop().map(|var| var.val.read().truthy()).unwrap_or(false);
        let target = env.stack.pop().and_then(|var| var.try_obj());
        let Some(node) = target else {
            return Err(Error::Sealed(format!("{} needs an object (Ex. self.rules, or a type: <Engine>)", if matches!(self, Self::Seal) { "seal" } else { "digest" })));
        };
        let digest = match self {
            Self::Seal => {
                if !graph.confined_changes(&node) {
                    return Err(Error::Sealed("this sealed part can only seal objects it may change".into()));
                }
                graph.seal_with(&node, &options)?
            },
            _ => graph.digest(&node, options.fields)?,
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
        docs: r#"# Std::seal(obj: obj, fields: bool = false, removable: bool = false, reads: list = null, hosts: list = null) -> str
Seal an object (Ex. `self.rules`, or a type: `<Engine>`) and everything under it: from now on its functions can be called but not added, removed, renamed, replaced, or moved, and its attributes can't change, by this document's code or by the host. Its fields still change unless `fields` is true. Returns its digest (see `digest`). It's one-way: a document can't unseal, only the host can. A change a seal refuses fails with a `sealed: ...` error and changes nothing.

- `removable`: the object can still be dropped whole (nothing in it can change, and it can't be moved or renamed). For code that comes and goes, Ex. a provider's capabilities.
- `reads` (a list of objects or paths): confine the part's code. While it runs (and everything it calls), it finds only the sealed object and these objects (a path to anything else finds nothing, calling a function elsewhere fails), changes only the sealed object, and can't hand an object elsewhere to a library function (Ex. `stringify`). `[]` means only itself. They're kept as paths and looked up each time the part is entered, so an object replaced at a path is the one it reads.
- `hosts` (a list of host names): `Http.fetch` from the part's code reaches only these hosts (the document's own allowlist still applies). `[]` means no network.

Sealing again only adds to a seal: it never unfreezes fields, makes an object removable that wasn't, or lifts a confinement. Confined code can only seal what it may change.
```rust
#[type] Engine: { fn price(units: int) -> int { units * 50 } }

#[init]
fn lock() { seal(<Engine>); }

fn later() {
    parse("fn price(units: int) -> int { 0 }", <Engine>); // error: sealed
}

fn add_tool(source: str) {
    const tool = new {} on self.tools;
    parse(source, tool);
    seal(tool, removable = true, reads = [self.inputs], hosts = ['api.weather.com']);
}
```
"#.into(),
        params: crate::params![
            Param { name: "obj".into(), param_type: Type::Void, default: None },
            Param { name: "fields".into(), param_type: Type::Bool, default: Some(Arc::new(Base::Literal(Val::Bool(false)))) },
            Param { name: "removable".into(), param_type: Type::Bool, default: Some(Arc::new(Base::Literal(Val::Bool(false)))) },
            Param { name: "reads".into(), param_type: Type::Void, default: Some(Arc::new(Base::Literal(Val::Null))) },
            Param { name: "hosts".into(), param_type: Type::Void, default: Some(Arc::new(Base::Literal(Val::Null))) },
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
A SHA-256 (hex) over an object's functions and structure (its functions' names, parameters, return types, attributes, and instructions, and the objects, types, and fields that hold functions or objects under it), plus every field's value when `fields` is true. The same for the same document, whether it was parsed or loaded from BSTF, and wherever the object is (its own name isn't part of it), so an object sent to another document has the same digest there. It identifies exactly the logic (and with `fields`, the data) it covers. Hosts check it when they seal (`doc.seal(path, { digest })`) or verify a signature over it.
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

/// Std::sealed.
pub fn std_sealed() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "sealed".into(),
        is_async: false,
        docs: r#"# Std::sealed(obj: obj) -> map | null
How an object is sealed, or null when it isn't: a map with `part` (the path of the object `seal` was called on, which may be this one or one around it), `fields` (its fields are frozen), `removable` (it can be dropped whole), and `confined` (a list of `{ reads, hosts }`, one per confinement its code runs under; empty when it isn't confined).
```rust
fn check() {
    seal(self.tool, removable = true, reads = [self.inputs]);
    const info = sealed(self.tool);
    assert(info.get('removable'));
    assert_eq(info.get('confined').len(), 1);
    assert_null(sealed(self.inputs));
}
```
"#.into(),
        params: crate::params![
            Param { name: "obj".into(), param_type: Type::Void, default: None },
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(SEALED.clone());
            Ok(instructions)
        })
    }
}
