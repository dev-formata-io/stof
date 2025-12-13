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

use arcstr::{literal, ArcStr};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use crate::model::{Field, Graph, NodeRef, SId};


/// Const super keyword for paths.
pub const SUPER_KEYWORD: SId = SId(Bytes::from_static(b"super"));
pub const SUPER_STR_KEYWORD: ArcStr = literal!("super");

/// Const self keyword for paths.
pub const SELF_KEYWORD: SId = SId(Bytes::from_static(b"self"));
pub const SELF_STR_KEYWORD: ArcStr = literal!("self");


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
/// Path to a Node in a Graph.
pub struct SPath {
    /// Is this a path of names (or IDs)?
    /// If IDs, set to false.
    pub names: bool,
    pub path: Vec<SId>,
}
impl From<(Vec<SId>, bool)> for SPath {
    fn from(value: (Vec<SId>, bool)) -> Self {
        Self {
            names: value.1,
            path: value.0,
        }
    }
}
impl From<(&Vec<SId>, bool)> for SPath {
    fn from(value: (&Vec<SId>, bool)) -> Self {
        Self {
            names: value.1,
            path: value.0.clone(),
        }
    }
}
impl From<(SId, bool)> for SPath {
    fn from(value: (SId, bool)) -> Self {
        Self {
            names: value.1,
            path: vec![value.0],
        }
    }
}
impl SPath {
    /// Create a new path.
    pub fn new(path: Vec<SId>, names: bool) -> Self {
        Self {
            names,
            path
        }
    }

    /// From string path.
    pub fn string(path: &str, names: bool, sep: &str) -> Self {
        let path = path
            .split(sep)
            .into_iter()
            .map(|name| SId::from(name))
            .collect::<Vec<_>>();
        Self {
            names,
            path
        }
    }

    /// Join this path into a single string with a separator.
    pub fn join(&self, sep: &str) -> String {
        self.path.iter()
            .map(|id| id.as_ref())
            .collect::<Vec<&str>>()
            .join(sep)
    }

    /// Named path to a node.
    pub fn node(graph: &Graph, path: impl Into<Self>, start: Option<NodeRef>) -> Option<NodeRef> {
        let mut named_path: Self = path.into();
        if named_path.path.is_empty() {
            return start;
        }
        if !named_path.names {
            return Some(named_path.path[named_path.path.len() - 1].clone());
        }

        named_path.path.reverse();
        let mut current = None;
        if let Some(start) = start {
            if let Some(node) = start.node(graph) {
                current = Some(node);
            }
        }
        if current.is_none() {
            let first = named_path.path.pop().unwrap();

            // common to be a root, so look there first
            for root in &graph.roots {
                if let Some(node) = root.node(graph) {
                    if node.name == first {
                        current = Some(node);
                        break;
                    }
                }
            }
            if current.is_none() {
                for (_, node) in &graph.nodes {
                    if node.name == first {
                        current = Some(node);
                        break;
                    }
                }
            }
        }

        'node_loop: while current.is_some() && !named_path.path.is_empty() {
            let current_node = current.unwrap();
            let next_name = named_path.path.pop().unwrap();

            // Look in current node's children
            for child in &current_node.children {
                if let Some(child) = child.node(graph) {
                    if child.name == next_name {
                        current = Some(child);
                        continue 'node_loop;
                    }
                }
            }

            // Look for a field in the current node with a name that points to another object
            // Children are already handled above, so just look for direct fields
            // This is before parents because it is basically the same as looking for a child
            if let Some(dref) = Field::direct_field(graph, &current_node.id, next_name.as_ref()) {
                if let Some(field) = graph.get_stof_data::<Field>(&dref) {
                    if let Some(nref) = field.value.try_obj() {
                        if let Some(node) = nref.node(graph) {
                            current = Some(node);
                            continue 'node_loop;
                        }
                    }
                }
            }

            // Look at parent
            if let Some(parent) = &current_node.parent {
                if let Some(parent) = parent.node(graph) {
                    if next_name == SUPER_KEYWORD || next_name == parent.name {
                        current = Some(parent);
                        continue 'node_loop;
                    }
                }
            } else {
                // Look at roots
                for root in &graph.roots {
                    if let Some(node) = root.node(graph) {
                        if node.name == next_name {
                            current = Some(node);
                            continue 'node_loop;
                        }
                    }
                }
            }

            // Handle self (or duplicate) next
            if next_name == SELF_KEYWORD || current_node.name == next_name {
                current = Some(current_node);
                continue 'node_loop;
            }

            current = None;
        }

        if let Some(node) = current {
            Some(node.id.clone())
        } else {
            None
        }
    }
    
    /// ID path for this named path.
    pub fn to_id_path(&self, graph: &Graph, start: Option<NodeRef>) -> Option<Self> {
        if !self.names {
            Some(self.clone())
        } else if let Some(node) = Self::node(graph, (&self.path, true), start) {
            node.node_path(graph, false)
        } else {
            None
        }
    }

    /// Named path for this ID path.
    pub fn to_name_path(self, graph: &Graph) -> Self {
        if self.names {
            self
        } else {
            let mut names = Vec::new();
            for id in self.path {
                if let Some(node) = id.node(graph) {
                    names.push(node.name.clone());
                }
            }
            Self {
                names: true,
                path: names
            }
        }
    }

    /// Equals.
    /// Will convert to ID paths to see if they are the same if needed.
    pub fn equals(&self, graph: &Graph, other: &Self, start: Option<NodeRef>) -> bool {
        if self.names {
            if other.names {
                let sf = self.to_id_path(graph, start.clone());
                let ot = other.to_id_path(graph, start);
                sf.is_some() && sf == ot
            } else {
                if let Some(sf) = self.to_id_path(graph, start) {
                    return &sf == other;
                }
                false
            }
        } else {
            if other.names {
                if let Some(ot) = other.to_id_path(graph, start) {
                    return &ot == self;
                }
                false
            } else {
                self == other
            }
        }
    }
}


/// Default path is a dot ('.') separated named path.
impl<T: ?Sized + ToString> From<&T> for SPath {
    fn from(value: &T) -> Self {
        let path = value.to_string()
            .split('.')
            .into_iter()
            .map(|name| SId::from(name))
            .collect::<Vec<_>>();
        Self {
            names: true,
            path
        }
    }
}
