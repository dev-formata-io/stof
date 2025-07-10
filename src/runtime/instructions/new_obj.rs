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

use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use crate::{model::{Graph, Node, NodeRef, SId}, runtime::{instruction::{Instruction, Instructions}, proc::ProcEnv, Error, Type, Val, Variable}};


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// New object instruction.
/// Creates an object and adds a ref on the stack.
pub struct NewObjIns {
    /// Optional parent for this node (root if None)
    pub parent: Option<NodeRef>,
    /// Name of this object (random if None (set to ID))
    pub name: Option<SId>,
    /// ID of this object (random if None)
    pub id: Option<SId>,
    /// Is this object a field?
    pub field: bool,
    /// Attributes on this object
    pub attributes: Option<FxHashMap<String, Val>>,
    /// Cast the object to a type?
    pub cast_type: Option<Type>,
}
#[typetag::serde(name = "NewObjIns")]
impl Instruction for NewObjIns {
    fn exec(&self, env: &mut ProcEnv, graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        let mut id = SId::default();
        if let Some(cid) = &self.id { id = cid.clone(); }

        let mut name = id.clone();
        if let Some(cn) = &self.name { name = cn.clone(); }

        let mut node = Node::new(name, id, self.field);
        if let Some(attr) = &self.attributes {
            for (k, v) in attr {
                node.attributes.insert(k.clone(), v.clone());
            }
        }

        // check for parent
        if let Some(parent) = &self.parent {
            if !parent.node_exists(&graph) {
                return Err(Error::NewObjParentDne);
            }

            // TODO: check for name collisions
        }
        let nref = graph.insert_stof_node(node, self.parent.clone());

        // cast value if needed
        let mut val = Val::Obj(nref);
        if let Some(cast_type) = &self.cast_type {
            val.cast(cast_type, graph, Some(env.self_ptr()))?;
        }
        env.stack.push(Variable::val(val));
        Ok(None)
    }
}
