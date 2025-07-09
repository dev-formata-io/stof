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

pub mod num;


#[derive(Clone)]
/// Library function.
pub struct LibFunc {
    pub library: ArcStr,
    pub name: String,
    pub is_async: bool,
    pub docs: String,
    pub params: Vector<Param>,
    pub return_type: Option<Type>,
    pub func: Arc<dyn Fn(&mut ProcEnv, &mut Graph)->Result<Instructions, Error>>
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
