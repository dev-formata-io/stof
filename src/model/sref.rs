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

use std::mem::swap;
use rustc_hash::FxHashSet;
use crate::model::{Data, Graph, Node, SId, SPath};


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

    /// Is this node a child of (or the same as) another node?
    pub fn child_of(&self, graph: &Graph, other: &NodeRef) -> bool {
        if self == other { return true; }
        if let Some(node) = self.node(graph) {
            if let Some(parent) = &node.parent {
                return parent.child_of(graph, other);
            }
        }
        false
    }

    /// Child of, but return the distance.
    /// Returns distance if a child, -1 otherwise.
    pub fn child_of_distance(&self, graph: &Graph, other: &NodeRef) -> i32 {
        if self == other { return 0; }

        let mut node_parent = None;
        if let Some(node) = self.node(graph) {
            node_parent = node.parent.clone();
        }

        let mut dist = 0;
        while node_parent.is_some() {
            dist += 1;
            if let Some(np) = &node_parent {
                if np == other {
                    return dist;
                } else if let Some(node) = np.node(graph) {
                    node_parent = node.parent.clone();
                } else {
                    node_parent = None;
                }
            }
        }
        -1
    }

    /// Node path.
    pub fn node_path(&self, graph: &Graph, names: bool) -> Option<SPath> {
        let mut node = self.node(graph);
        if node.is_some() {
            let mut res = Vec::new();
            let mut seen = FxHashSet::default();
            while node.is_some() {
                let inner = node.unwrap();
                if seen.contains(&inner.id) { break; }

                if names {
                    res.push(inner.name.clone());
                } else {
                    res.push(inner.id.clone());
                }

                seen.insert(inner.id.clone());
                if let Some(parent) = &inner.parent {
                    node = parent.node(graph);
                } else {
                    node = None;
                }
            }
            res.reverse();
            return Some(SPath {
                names,
                path: res,
            });
        }
        None
    }

    /// Distance to another node in the graph.
    /// If a node doesn't exist, -2.
    /// If same node, distance is 0.
    /// If nodes are not in the same graph or are in different roots, distance is -1.
    /// Otherwise, distance is the path length from this node to other node.
    pub fn distance_to(&self, graph: &Graph, other: &Self) -> i32 {
        if !self.node_exists(graph) { return -2; }
        if !other.node_exists(graph) { return -2; }
        if self == other { return 0; }

        let mut node_a_id_path = self.node_path(graph, false).unwrap().path;
        let mut node_b_id_path = other.node_path(graph, false).unwrap().path;
        if node_a_id_path.len() < 1 || node_b_id_path.len() < 1 || &node_a_id_path[0] != &node_b_id_path[0] {
            return -1;
        }

        if node_a_id_path.len() > node_b_id_path.len() {
            swap(&mut node_a_id_path, &mut node_b_id_path);
        }

        let mut to_remove = FxHashSet::default();
        let mut last = SId::default();
        for i in 0..node_a_id_path.len() {
            let aid = &node_a_id_path[i];
            let bid = &node_b_id_path[i];
            if aid == bid {
                to_remove.insert(aid.clone());
                last = aid.clone();
            } else {
                break;
            }
        }
        to_remove.remove(&last);

        // Remove the shared ids from each vector
        node_a_id_path.retain(|x| !to_remove.contains(x));
        node_b_id_path.retain(|x| !to_remove.contains(x));

        (node_a_id_path.len() as i32 - 1) + (node_b_id_path.len() as i32 - 1)
    }
}


/// Type alias for SId for readability when referencing data.
pub type DataRef = SId;
impl DataRef {
    #[inline(always)]
    /// Data exists at this ref?
    pub fn data_exists(&self, graph: &Graph) -> bool {
        graph.data.contains_key(self)
    }

    #[inline(always)]
    /// Get data.
    pub fn data<'a>(&self, graph: &'a Graph) -> Option<&'a Data> {
        graph.data.get(self)
    }

    #[inline(always)]
    /// Get a mutable data.
    pub fn data_mut<'a>(&self, graph: &'a mut Graph) -> Option<&'a mut Data> {
        graph.data.get_mut(self)
    }
}
