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

use crate::{Graph, Node, SId};


/// Type alias for SId for readability when referencing a node.
pub type NodeRef = SId;
impl NodeRef {
    #[inline(always)]
    /// Node exists at this ref?
    pub fn node_exists(&self, graph: &Graph) -> bool {
        graph.nodes.contains_key(self)
    }

    #[inline(always)]
    /// Get a node.
    pub fn node<'a>(&self, graph: &'a Graph) -> Option<&'a Node> {
        graph.nodes.get(self)
    }

    #[inline(always)]
    /// Get a mutable node.
    pub fn node_mut<'a>(&self, graph: &'a mut Graph) -> Option<&'a mut Node> {
        graph.nodes.get_mut(self)
    }

    /// Root node ref for this ref.
    pub fn root(&self, graph: &Graph) -> Option<NodeRef> {
        if let Some(node) = self.node(graph) {
            if let Some(parent) = &node.parent {
                return parent.root(graph);
            }
            return Some(node.id.clone());
        }
        None
    }
}


/// Type alias for SId for readability when referencing data.
pub type DataRef = SId;
impl DataRef {

}
