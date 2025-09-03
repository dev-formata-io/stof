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

use std::{fmt::Debug, ops::Deref, str::FromStr, sync::Arc};
use age::secrecy::ExposeSecret;
use arcstr::{literal, ArcStr};
use bytes::Bytes;
use imbl::vector;
use serde::{Deserialize, Serialize};
use crate::{model::{Field, Graph, LibFunc, Param, SId, SPath, StofData, SELF_STR_KEYWORD, SUPER_STR_KEYWORD}, runtime::{instruction::{Instruction, Instructions}, instructions::Base, proc::ProcEnv, Error, Type, Val, Variable}};


/// Library name.
pub(self) const AGE_LIB: ArcStr = literal!("Age");
pub fn insert_age_encrypt_library(graph: &mut Graph) {
    // Age.parse(age: Data<Age>, bin: blob, context: obj = self, format: str = "stof") -> bool
    graph.insert_libfunc(LibFunc {
        library: AGE_LIB.clone(),
        name: "parse".into(),
        is_async: false,
        docs: r#"# Age.parse(age: Data<Age>, bin: blob, context: obj = self, format: str = "stof") -> bool
Parse an age-encrypted binary. Similar to Std.parse, but requires an Age identity (secret private key).
"#.into(),
        params: vector![
            Param { name: "age".into(), param_type: Type::Data(AGE_LIB.clone()), default: None, },
            Param { name: "bin".into(), param_type: Type::Blob, default: None, },
            Param { name: "context".into(), param_type: Type::Void, default: Some(Arc::new(Base::Literal(Val::Null))), },
            Param { name: "format".into(), param_type: Type::Str, default: Some(Arc::new(Base::Literal(Val::Null))), },
        ],
        unbounded_args: false,
        return_type: None,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(AgeIns::Parse));
            Ok(instructions)
        })
    });

    // Age.blobify(recipients: str | list | Data<Age>, format: str = 'stof', context?: obj) -> blob
    graph.insert_libfunc(LibFunc {
        library: AGE_LIB.clone(),
        name: "blobify".into(),
        is_async: false,
        docs: r#"# Age.blobify(recipients: str | list | Data<Age>, format: str = 'stof', context?: obj) -> blob
Std.blobify, but with age public-key recipients. The resulting blob can only be parsed by a recipient's private key.
"#.into(),
        params: vector![
            Param { name: "recipients".into(), param_type: Type::Union(vector![Type::Str, Type::List, Type::Data(AGE_LIB.clone())]), default: None, },
            Param { name: "format".into(), param_type: Type::Str, default: Some(Arc::new(Base::Literal(Val::Null))), },
            Param { name: "context".into(), param_type: Type::Void, default: Some(Arc::new(Base::Literal(Val::Null))), },
        ],
        unbounded_args: false,
        return_type: None,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(AgeIns::Blobify));
            Ok(instructions)
        })
    });

    // Age.generate(context: obj = self) -> Data<Age>
    graph.insert_libfunc(LibFunc {
        library: AGE_LIB.clone(),
        name: "generate".into(),
        is_async: false,
        docs: r#"# Age.generate(context: obj = self) -> Data<Age>
Generate a new Age Identity (Data<Age>) on the given context object (default is self).
"#.into(),
        params: vector![
            Param { name: "context".into(), param_type: Type::Void, default: Some(Arc::new(Base::Literal(Val::Null))), },
        ],
        unbounded_args: false,
        return_type: None,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(AgeIns::Generate));
            Ok(instructions)
        })
    });

    // Age.public(age: Data<Age>) -> str
    graph.insert_libfunc(LibFunc {
        library: AGE_LIB.clone(),
        name: "public".into(),
        is_async: false,
        docs: r#"# Age.public(age: Data<Age>) -> str
Get the public key for a given age identity.
"#.into(),
        params: vector![
            Param { name: "age".into(), param_type: Type::Data(AGE_LIB.clone()), default: None, },
        ],
        unbounded_args: false,
        return_type: None,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(AgeIns::PubKey));
            Ok(instructions)
        })
    });
}


#[derive(Clone, Serialize, Deserialize)]
/// Age identity data.
pub struct AgeIdentity {
    #[serde(deserialize_with = "deserialize_identity")]
    #[serde(serialize_with = "serialize_identity")]
    pub id: age::x25519::Identity,
}
fn serialize_identity<S>(identity: &age::x25519::Identity, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
    identity.to_string().expose_secret().serialize(serializer)
}
fn deserialize_identity<'de, D>(deserializer: D) -> Result<age::x25519::Identity, D::Error>
    where
        D: serde::Deserializer<'de> {
    let secret: String = Deserialize::deserialize(deserializer)?;
    match age::x25519::Identity::from_str(&secret) {
        Ok(identity) => Ok(identity),
        Err(error) => Err(serde::de::Error::custom(error))
    }
}

impl Debug for AgeIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AgeIdentity[REDACTED]")
    }
}

#[typetag::serde(name = "Age")] // also the libname (Data<Age>)!
impl StofData for AgeIdentity {}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Age instructions.
pub enum AgeIns {
    /// Parse an age encrypted binary (graph.age_decrypt_import) with an identity.
    Parse,
    /// Export an age encrypted binary to a set of identities (public keys).
    Blobify,
    /// Generate a new identity in the graph.
    Generate,
    /// Get the public key of an identity to share with others.
    PubKey,
}
#[typetag::serde(name = "AgeIns")]
impl Instruction for AgeIns {
    fn exec(&self, env: &mut ProcEnv, graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        match self {
            Self::Parse => {
                // Age.parse(age_identity: Data<Age>, bin: blob, context: obj = self, format: str = "stof") -> bool
                if let Some(format_var) = env.stack.pop() {
                    if let Some(context_var) = env.stack.pop() {
                        if let Some(encrypted_bin_var) = env.stack.pop() {
                            if let Some(age_data_var) = env.stack.pop() {
                                if let Some(dref) = age_data_var.try_data_or_func() {
                                    let mut id = None;
                                    if let Some(age_identity) = graph.get_stof_data::<AgeIdentity>(&dref) {
                                        id = Some(age_identity.id.clone());
                                    }
                                    if let Some(id) = id {
                                        let mut context = env.self_ptr();
                                        match context_var.val.read().deref() {
                                            Val::Str(path) => {
                                                let mut ctx = None;
                                                if path.starts_with(SELF_STR_KEYWORD.as_str()) || path.starts_with(SUPER_STR_KEYWORD.as_str()) {
                                                    ctx = Some(env.self_ptr());
                                                }
                                                if let Some(field_ref) = Field::field_from_path(graph, path.as_str(), ctx.clone()) {
                                                    if let Some(field) = graph.get_stof_data::<Field>(&field_ref) {
                                                        if let Some(obj) = field.value.try_obj() {
                                                            context = obj;
                                                        } else {
                                                            // Context was not an object
                                                            env.stack.push(Variable::val(Val::Bool(false)));
                                                            return Ok(None);
                                                        }
                                                    }
                                                } else if let Some(node) = SPath::node(&graph, path.as_str(), ctx) {
                                                    context = node;
                                                } else {
                                                    // context given, but not found (return false)
                                                    env.stack.push(Variable::val(Val::Bool(false)));
                                                    return Ok(None);
                                                }
                                            },
                                            Val::Obj(nref) => {
                                                context = nref.clone();
                                            },
                                            _ => {}
                                        }
                                        
                                        let mut format = "stof".to_string();
                                        match format_var.val.read().deref() {
                                            Val::Str(fmt) => {
                                                format = fmt.to_string();
                                            },
                                            Val::Void |
                                            Val::Null => {}, // keep as stof
                                            _ => {
                                                return Err(Error::StdParse("format must be a string content type or stof format identifier".to_string()));
                                            }
                                        }

                                        match encrypted_bin_var.val.read().deref() {
                                            Val::Blob(bytes) => {
                                                if let Err(error) = graph.age_decrypt_import(&format, Bytes::from(bytes.clone()), Some(context), &id) {
                                                    if error == Error::AgeNoMatchingKeys {
                                                        env.stack.push(Variable::val(Val::Bool(false)));
                                                    } else {
                                                        return Err(error);
                                                    }
                                                } else {
                                                    env.stack.push(Variable::val(Val::Bool(true)));
                                                }
                                                return Ok(None);
                                            },
                                            _ => {
                                                return Err(Error::StdParse("age parse source data must be a blob".to_string()));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                return Err(Error::Custom("AgeParseError".into()));
            },
            Self::Blobify => {
                // Age.blobify(recipients: str | list | Data<Age>, format: str = 'stof', context?: obj) -> blob
                if let Some(context_var) = env.stack.pop() {
                    if let Some(format_var) = env.stack.pop() {
                        if let Some(recipients_var) = env.stack.pop() {
                            let mut format = "stof".to_string();
                            match format_var.val.read().deref() {
                                Val::Str(fmt) => {
                                    format = fmt.to_string();
                                },
                                Val::Void |
                                Val::Null => {},
                                _ => {
                                    return Err(Error::StdBlobify("format must be a string content type or format identifier and must be made available to the graph explicitely by each runtime".to_string()))
                                }
                            }

                            let mut ctx = None;
                            match context_var.val.read().deref() {
                                Val::Obj(nref) => {
                                    ctx = Some(nref.clone());
                                },
                                Val::Void |
                                Val::Null => {},
                                _ => {
                                    return Err(Error::StdBlobify("context must be an object".to_string()));
                                },
                            }

                            let mut recipients = vec![];
                            match recipients_var.val.read().deref() {
                                Val::Str(pubkey) => {
                                    recipients.push(age::x25519::Recipient::from_str(pubkey.as_str()).expect("could not parse age public key string"));
                                },
                                Val::Data(dref) => {
                                    if let Some(identity) = graph.get_stof_data::<AgeIdentity>(dref) {
                                        recipients.push(identity.id.to_public());
                                    }
                                },
                                Val::List(vals) => {
                                    for val in vals {
                                        match val.read().deref() {
                                            Val::Str(pubkey) => {
                                                recipients.push(age::x25519::Recipient::from_str(pubkey.as_str()).expect("could not parse age public key string"));
                                            },
                                            Val::Data(dref) => {
                                                if let Some(identity) = graph.get_stof_data::<AgeIdentity>(dref) {
                                                    recipients.push(identity.id.to_public());
                                                }
                                            },
                                            _ => {}
                                        }
                                    }
                                },
                                _ => {}
                            }
                            if recipients.len() < 1 {
                                return Err(Error::Custom("age blobify empty recipients".into()));
                            }

                            let iter = recipients.iter().map(|r| r as _);
                            let bytes = graph.age_encrypt_export(&format, ctx, iter)?;
                            env.stack.push(Variable::val(Val::Blob(bytes.to_vec())));
                            return Ok(None);
                        }
                    }
                }
                return Err(Error::Custom("AgeBlobifyError".into()));
            },
            Self::Generate => {
                // Age.generate(context: obj = self) -> Data<Age>
                let mut context = env.self_ptr();
                if let Some(context_var) = env.stack.pop() {
                    if let Some(ctx) = context_var.try_obj() {
                        context = ctx;
                    }
                }
                
                let identity = AgeIdentity { id: age::x25519::Identity::generate() };
                let name = SId::default();
                let id = name.clone();
                if let Some(dref) = graph.insert_stof_data(&context, name, Box::new(identity), Some(id)) {
                    env.stack.push(Variable::val(Val::Data(dref)));
                    return Ok(None);
                }
                return Err(Error::Custom("AgeGenerateError".into()));
            },
            Self::PubKey => {
                if let Some(age_identity_var) = env.stack.pop() {
                    if let Some(dref) = age_identity_var.try_data_or_func() {
                        if let Some(identity) = graph.get_stof_data::<AgeIdentity>(&dref) {
                            env.stack.push(Variable::val(Val::Str(identity.id.to_public().to_string().into())));
                            return Ok(None);
                        }
                    }
                }
                return Err(Error::Custom("AgePubKeyError".into()));
            },
        }
    }
}
