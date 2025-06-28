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

use bytes::Bytes;
use serde::{Deserialize, Serialize};

use crate::model::{Graph, NodeRef, SId};


/// Const super keyword for paths.
pub const SUPER_KEYWORD: SId = SId(Bytes::from_static(b"super"));

/// Const self keyword for paths.
pub const SELF_KEYWORD: SId = SId(Bytes::from_static(b"self"));


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
/// Path to a Node in a Graph.
pub struct SPath {
    /// Is this a path of names (or IDs)?
    /// If IDs, set to false.
    pub names: bool,
    pub path: Vec<SId>,
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

    /// ID path for this named path.
    pub fn to_id_path(mut self, graph: &Graph) -> Option<Self> {
        if !self.names {
            Some(self)
        } else {
            if self.path.len() < 1 {
                return Some(Self {
                    names: false,
                    path: self.path,
                })
            }
            self.path.reverse();
            
            let mut current = None;
            let first = self.path.pop().unwrap();

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

            'node_loop: while current.is_some() && self.path.len() > 0 {
                let current_node = current.unwrap();
                let next_name = self.path.pop().unwrap();

                // Look in current node's children
                for child in &current_node.children {
                    if let Some(child) = child.node(graph) {
                        if child.name == next_name {
                            current = Some(child);
                            continue 'node_loop;
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
                                break 'node_loop;
                            }
                        }
                    }
                }

                // Handle self (or duplicate) next
                if next_name == SELF_KEYWORD || current_node.name == next_name {
                    current = Some(current_node);
                    continue 'node_loop;
                }

                // TODO: Look for a field in the current node that is an object here

                current = None;
            }

            if let Some(node) = current {
                node.id.node_path(graph, false)
            } else {
                None
            }
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

    /// Node ID that this path points to (ref).
    /// More efficient to convert to ID path first.
    pub fn node(&self, graph: &Graph) -> Option<NodeRef> {
        if self.path.len() < 1 { return None; }
        if !self.names {
            Some(self.path[self.path.len() - 1].clone())
        } else if let Some(mut cpy) = self.clone().to_id_path(graph) {
            if cpy.path.len() < 1 {
                None
            } else {
                Some(cpy.path.pop().unwrap())
            }
        } else {
            None
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
