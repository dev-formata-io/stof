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


pub mod func;
pub use func::*;

pub mod field;
pub use field::*;

pub mod prototype;
pub use prototype::*;

use serde::{Deserialize, Serialize};
use crate::model::{Graph, NodeRef, StofData};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InnerDoc {
    pub docs: String,
}
#[typetag::serde(name = "_InnerDoc")]
impl StofData for InnerDoc {
    fn core_data(&self) -> bool {
        return true;
    }
}
impl InnerDoc {
    /// Inner docs on a node.
    pub fn docs(graph: &Graph, node: &NodeRef) -> String {
        let mut docs = Vec::new();
        if let Some(node) = node.node(graph) {
            for (_, dref) in &node.data {
                if let Some(doc) = graph.get_stof_data::<Self>(dref) {
                    docs.push(doc.docs.clone());
                }
            }
        }
        docs.join("\n\n")
    }
}
