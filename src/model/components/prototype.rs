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

use arcstr::{literal, ArcStr};
use serde::{Deserialize, Serialize};
use crate::model::{DataRef, Graph, NodeRef, SPath, StofData};


/// Const prototype "type" literal.
pub const PROTOTYPE_TYPE_ATTR: ArcStr = literal!("type");

/// Const prototype "extends" literal.
pub const PROTOTYPE_EXTENDS_ATTR: ArcStr = literal!("extends");


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Prototype.
pub struct Prototype {
    pub node: NodeRef,
}

#[typetag::serde(name = "Proto")]
impl StofData for Prototype {
    fn core_data(&self) -> bool {
        true
    }
}

impl Prototype {
    /// Get a prototype from a dot separated name path string.
    pub fn from_path(graph: &Graph, path: &str, start: Option<NodeRef>) -> Vec<DataRef> {
        let spath = SPath::from(path);
        if spath.path.is_empty() { return vec![]; }
        if let Some(node) = SPath::node(&graph, spath, start) {
            return Self::prototype_refs(graph, &node);
        }
        vec![]
    }
    
    /// Prototype references on a node.
    pub fn prototype_refs(graph: &Graph, node: &NodeRef) -> Vec<DataRef> {
        let mut protos = Vec::new();
        if let Some(node) = node.node(graph) {
            for (_, dref) in &node.data {
                if dref.type_of::<Self>(&graph) {
                    protos.push(dref.clone());
                }
            }
        }
        protos
    }

    /// Prototype nodes referenced by a node.
    pub fn prototype_nodes(graph: &Graph, node: &NodeRef, recursive: bool) -> Vec<NodeRef> {
        let mut protos = Vec::new();
        if let Some(node) = node.node(graph) {
            for (_, dref) in &node.data {
                if let Some(proto) = graph.get_stof_data::<Self>(dref) {
                    protos.push(proto.node.clone());

                    if recursive {
                        protos.append(&mut Self::prototype_nodes(graph, &proto.node, true));
                    }
                }
            }
        }
        protos
    }
}
