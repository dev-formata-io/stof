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

use std::{fmt::{Debug, Display}, sync::Arc};
use arcstr::ArcStr;
use imbl::Vector;
use crate::{model::{Graph, Param}, runtime::{instruction::Instructions, proc::ProcEnv, Error, Type}};

pub mod filesys;
pub mod http;

pub mod stof_std;
pub mod num;
pub mod string;
pub mod ver;
pub mod time;
pub mod function;
pub mod list;
pub mod set;
pub mod map;
pub mod tup;
pub mod blob;
pub mod data;
pub mod obj;


#[derive(Clone)]
/// Library function.
pub struct LibFunc {
    /// What library does this function belong to?
    /// This is how this func will be referenced from within Stof.
    /// Ex. Num.abs(v), where the library is "Num".
    pub library: ArcStr,

    /// Name of this function within the library.
    pub name: String,

    /// Should this function be executed in its own process?
    pub is_async: bool,

    /// Docs for this lib func.
    pub docs: String,

    /// Parameters for this function.
    /// Arguments will be placed in this order, according to name.
    pub params: Vector<Param>,

    /// Allow unbounded arguments? Ex. pln(a, b, c, d, ...)
    pub unbounded_args: bool,

    /// Return type of this func.
    /// If None, no cast will be performed at the end of instructions.
    pub return_type: Option<Type>,

    /// When adding arguments before this call, should they go to the symbol table
    /// under the parameter name? If false, they will be left on the stack and the
    /// func below will be required to pop values according to the arg_count.
    pub args_to_symbol_table: bool,

    /// fn(arg_count, env, graph) -> Instructions
    /// What instructions will this library function execute?
    pub func: Arc<dyn Fn(bool, usize, &mut ProcEnv, &mut Graph)->Result<Instructions, Error>>
}
impl Display for LibFunc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut rtype = String::from("Void");
        if let Some(ty) = &self.return_type { rtype = ty.type_of().to_string(); }
        write!(f, "{}.{}({:?}) -> {};", self.library, self.name, self.params, &rtype)
    }
}
impl Debug for LibFunc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut rtype = String::from("Void");
        if let Some(ty) = &self.return_type { rtype = ty.type_of().to_string(); }
        write!(f, "{}.{}({:?}) -> {};", self.library, self.name, self.params, &rtype)
    }
}
