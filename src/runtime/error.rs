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

use std::fmt::Display;
use arcstr::ArcStr;
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Error.
/// TODO.
pub enum Error {
    ParseFailure(String),

    Custom(ArcStr),
    NotImplemented,
    Thrown,

    DeclareExisting,
    DeclareInvalidName,
    AssignConst,
    VariableSet,
    FieldPrivateSet,
    AssignSelf,
    AssignSuper,
    AssignRootNonObj,
    AssignExistingRoot,

    JumpTable,
    StackError,
    SelfStackError,
    NewStackError,
    CallStackError,
    CastStackError,
    
    CastVal,

    NewObjParentDne,

    // Function calling errors
    FuncDne,
    FuncDefaultArg(Box<Self>),
    FuncArgs,

    // Value errors
    Truthy,
    NotTruthy,
    GreaterThan,
    GreaterOrEq,
    LessThan,
    LessOrEq,
    Eq,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    AND,
    OR,
    XOR,
    SHL,
    SHR,
}
impl Display for Error { // maps ToString and print to Debug
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
