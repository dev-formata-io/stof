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

//! Keys and signatures (`Key`): make a key, share its public key, sign the exact bytes of a message under a
//! context, and verify. Ed25519; keys and signatures are base64 strings. See `model::seal` (the same functions are
//! on every host: `StofDoc.sign`, `Doc.sign`, `stof::model::seal::sign`).

use std::sync::Arc;
use arcstr::{literal, ArcStr};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use crate::{model::{Graph, LibFunc, Param, seal}, runtime::{Error, Type, Val, Variable, instruction::{Instruction, Instructions}, proc::ProcEnv}};


/// Library name.
pub const KEY_LIB: ArcStr = literal!("Key");

lazy_static! {
    static ref GENERATE: Arc<dyn Instruction> = Arc::new(KeyIns::Generate);
    static ref PUBLIC: Arc<dyn Instruction> = Arc::new(KeyIns::Public);
    static ref SIGN: Arc<dyn Instruction> = Arc::new(KeyIns::Sign);
    static ref VERIFY: Arc<dyn Instruction> = Arc::new(KeyIns::Verify);
}


/// Add the key library to a graph.
pub fn insert_key_lib(graph: &mut Graph) {
    graph.insert_libfunc(key_generate());
    graph.insert_libfunc(key_public());
    graph.insert_libfunc(key_sign());
    graph.insert_libfunc(key_verify());
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Key instructions.
pub enum KeyIns {
    /// () -> str (a secret key).
    Generate,
    /// (secret) -> str (its public key).
    Public,
    /// (secret, context, message) -> str (a signature).
    Sign,
    /// (public, context, message, signature) -> bool.
    Verify,
}
#[typetag::serde(name = "KeyIns")]
impl Instruction for KeyIns {
    fn exec(&self, env: &mut ProcEnv, _graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        let mut pop = || env.stack.pop().map(|var| var.val.read().clone()).unwrap_or(Val::Null);
        let res = match self {
            Self::Generate => Val::Str(seal::generate_key().into()),
            Self::Public => {
                let secret = text(pop(), "Key::public", "the secret key")?;
                Val::Str(seal::public_key(&secret)?.into())
            },
            Self::Sign => {
                let message = pop();
                let context = text(pop(), "Key::sign", "the context")?;
                let secret = text(pop(), "Key::sign", "the secret key")?;
                let bytes = message_bytes(message, "Key::sign")?;
                Val::Str(seal::sign(&secret, &context, &bytes)?.into())
            },
            Self::Verify => {
                let signature = pop();
                let message = pop();
                let context = text(pop(), "Key::verify", "the context")?;
                let public = pop();
                let bytes = message_bytes(message, "Key::verify")?;
                // a key or signature that isn't one (Ex. null) is simply not a valid signature
                let (Val::Str(public), Val::Str(signature)) = (public, signature) else { return push(env, Val::Bool(false)) };
                Val::Bool(seal::verify(&public, &context, &bytes, &signature))
            },
        };
        push(env, res)
    }
}

fn push(env: &mut ProcEnv, val: Val) -> Result<Option<Instructions>, Error> {
    env.stack.push(Variable::val(val));
    Ok(None)
}

fn text(val: Val, func: &str, what: &str) -> Result<String, Error> {
    match val {
        Val::Str(text) => Ok(text.to_string()),
        other => Err(Error::Key(format!("{func} needs {what} as a string (got {})", other.gen_type().type_of()))),
    }
}

/// A message's exact bytes: a string's UTF-8, or a blob.
fn message_bytes(message: Val, func: &str) -> Result<Vec<u8>, Error> {
    match message {
        Val::Str(text) => Ok(text.as_bytes().to_vec()),
        Val::Blob(bytes) => Ok(bytes.to_vec()),
        _ => Err(Error::Key(format!("{func} needs the message as a string or blob (the exact bytes that are signed): stringify an object first"))),
    }
}


/// Key::generate.
fn key_generate() -> LibFunc {
    LibFunc {
        library: KEY_LIB.clone(),
        name: "generate".into(),
        is_async: false,
        docs: r#"# Key::generate() -> str
A new secret key: 32 bytes from the system's secure random source, as base64. Keep it secret (pass it in where it's needed, or encrypt it with Age if a document has to hold it), and share only its public key (`Key::public`).
```rust
const secret = Key::generate();
const public = Key::public(secret);
```
"#.into(),
        params: crate::params![],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(GENERATE.clone());
            Ok(instructions)
        })
    }
}

/// Key::public.
fn key_public() -> LibFunc {
    LibFunc {
        library: KEY_LIB.clone(),
        name: "public".into(),
        is_async: false,
        docs: r#"# Key::public(secret: str) -> str
The public key for a secret key (base64). Anyone with it can check your signatures (`Key::verify`), but can't make them.
```rust
const public = Key::public(secret);
```
"#.into(),
        params: crate::params![
            Param { name: "secret".into(), param_type: Type::Str, default: None },
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(PUBLIC.clone());
            Ok(instructions)
        })
    }
}

/// Key::sign.
fn key_sign() -> LibFunc {
    LibFunc {
        library: KEY_LIB.clone(),
        name: "sign".into(),
        is_async: false,
        docs: r#"# Key::sign(secret: str, context: str, message: str | blob) -> str
Sign a message with a secret key: the signature (base64). What's signed is the message's exact bytes (a string's UTF-8, or a blob), so sign the text you send (Ex. `stringify('json', order)`), not an object.

The context names what the signature is for (Ex. "orders/v1"): it only verifies under that same context, so a signature made for one purpose can't be passed off as another. A seal signature is a signature over a part's digest under the context "stof/seal".
```rust
const order = stringify('json', self.order);
const signature = Key::sign(secret, 'orders/v1', order);
assert(Key::verify(Key::public(secret), 'orders/v1', order, signature));

// sign a part of the document, for a host to check when it seals it
const part = Key::sign(secret, 'stof/seal', digest(self.rules));
```
"#.into(),
        params: crate::params![
            Param { name: "secret".into(), param_type: Type::Str, default: None },
            Param { name: "context".into(), param_type: Type::Str, default: None },
            Param { name: "message".into(), param_type: Type::Void, default: None },
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(SIGN.clone());
            Ok(instructions)
        })
    }
}

/// Key::verify.
fn key_verify() -> LibFunc {
    LibFunc {
        library: KEY_LIB.clone(),
        name: "verify".into(),
        is_async: false,
        docs: r#"# Key::verify(public: str, context: str, message: str | blob, signature: str) -> bool
Is `signature` a signature over exactly these bytes, under this context, by the secret key behind `public`? False for anything else: another message (even one byte), another context, another key, or a key or signature that isn't one.
```rust
fn accept(order: str, signature: str) -> bool {
    if (!Key::verify(self.shop_key, 'orders/v1', order, signature)) return false;
    parse(order, self.order, 'json'); // trusted now
    true
}
```
"#.into(),
        params: crate::params![
            Param { name: "public".into(), param_type: Type::Void, default: None },
            Param { name: "context".into(), param_type: Type::Str, default: None },
            Param { name: "message".into(), param_type: Type::Void, default: None },
            Param { name: "signature".into(), param_type: Type::Void, default: None },
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(VERIFY.clone());
            Ok(instructions)
        })
    }
}
