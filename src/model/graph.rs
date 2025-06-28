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

use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use crate::model::{Data, DataRef, Node, NodeRef, SId, SPath, INVALID_NODE_NEW};


#[derive(Debug, Clone, Serialize, Deserialize, Default)]
/// Graph.
/// This is the data store for stof.
pub struct Graph {
    pub id: SId,
    pub roots: FxHashSet<NodeRef>,

    pub nodes: FxHashMap<NodeRef, Node>,
    pub data: FxHashMap<DataRef, Data>,

    #[serde(skip)]
    pub node_deadpool: FxHashMap<NodeRef, Node>,

    #[serde(skip)]
    pub data_deadpool: FxHashMap<DataRef, Data>,
}
impl Graph {
    /// Create a new graph with an ID.
    pub fn new(id: impl Into<SId>) -> Self {
        Self {
            id: id.into(),
            ..Default::default()
        }
    }

    #[inline]
    /// Find a node with a named path, optionally starting from an existing node.
    pub fn find_node_named(&self, path: &str, sep: &str, start: Option<NodeRef>) -> Option<NodeRef> {
        SPath::find(self, path, true, sep, start)
    }

    #[inline(always)]
    /// Main root.
    /// A root node named "root".
    pub fn main_root(&self) -> Option<NodeRef> {
        self.find_root_named("root")
    }

    /// Find a root node with a given name.
    pub fn find_root_named(&self, name: &str) -> Option<NodeRef> {
        for root in &self.roots {
            if let Some(node) = self.nodes.get(root) {
                if node.name.as_ref() == name {
                    return Some(root.clone());
                }
            }
        }
        None
    }


    /*****************************************************************************
     * Nodes.
     *****************************************************************************/
    
    /// Insert a root node directly.
    pub fn insert_root(&mut self, name: impl Into<SId>) -> NodeRef {
        let mut node = Node::new(name.into(), SId::default());
        node.invalidate(INVALID_NODE_NEW);

        let nref = node.id.clone();
        self.nodes.insert(node.id.clone(), node);
        self.roots.insert(nref.clone());
        nref
    }

    /// Insert a node.
    /// If a parent is not provided, the behavior is the same as insert root.
    pub fn insert_node(&mut self, name: impl Into<SId>, parent: Option<NodeRef>) -> NodeRef {
        let node = Node::new(name.into(), SId::default());
        self.insert_stof_node(node, parent)
    }

    /// Insert a node with an ID.
    pub fn insert_node_id(&mut self, name: impl Into<SId>, id: impl Into<SId>, parent: Option<NodeRef>) -> NodeRef {
        let node = Node::new(name.into(), id.into());
        self.insert_stof_node(node, parent)
    }

    /// Insert stof node.
    /// Don't call this with nodes that already exist in the graph (have a valid ID already).
    pub fn insert_stof_node(&mut self, mut node: Node, parent: Option<NodeRef>) -> NodeRef {
        if let Some(parent) = &parent {
            if parent.node_exists(&self) {
                node.parent = Some(parent.clone());
            } else {
                node.parent = None;
            }
        } else {
            node.parent = None;
        }

        let nref = node.id.clone();
        node.invalidate(INVALID_NODE_NEW);
        self.nodes.insert(nref.clone(), node);

        if let Some(parent) = parent {
            if let Some(parent) = parent.node_mut(self) {
                parent.add_child(nref.clone());
            } else {
                self.roots.insert(nref.clone());
            }
        } else {
            self.roots.insert(nref.clone());
        }
        nref
    }

    /// Create nodes from a named path.
    pub fn create_named_path_nodes(&mut self, path: SPath, start: Option<NodeRef>, custom_insert: Option<fn(&mut Self, &SId, &SId)->NodeRef>) -> Option<NodeRef> {
        if path.path.len() < 1 { return None; }
        
        let mut current = start;
        let path = path.path;
        let mut start = 0;
        if current.is_none() {
            start += 1;
            let first = &path[0];

            // needs to be a root, so look before inserting
            for root in &self.roots {
                if let Some(node) = root.node(&self) {
                    if &node.name == first {
                        current = Some(node.id.clone());
                        break;
                    }
                }
            }
            if current.is_none() {
                current = Some(self.insert_root(first));
            }
        }
        
        while current.is_some() && start < path.len() {
            let name = &path[start];
            let nref = current.unwrap();
            current = None;

            if let Some(node) = nref.node(&self) {
                for child in &node.children {
                    if let Some(child_node) = child.node(&self) {
                        if &child_node.name == name {
                            current = Some(child.clone());
                            break;
                        }
                    }
                }
            }
            if current.is_none() {
                // create a new node with the given name here
                if let Some(custom) = &custom_insert {
                    current = Some(custom(self, name, &nref));
                } else {
                    current = Some(self.insert_node(name, Some(nref)));
                }
            }

            start += 1;
        }

        current
    }

    /// Remove a node from the graph.
    /// May or may not remove data completely, depending on where the data is referenced.
    pub fn remove_node(&mut self, nref: &NodeRef) -> bool {
        if let Some(node) = self.nodes.remove(nref) {
            // Remove all data
            for dref in &node.data {
                let mut remove_all = false;
                if let Some(data) = dref.data_mut(self) {
                    if data.node_removed(&node.id) {
                        remove_all = data.ref_count() < 1;
                    }
                }
                if remove_all {
                    if let Some(data) = self.data.remove(&dref) {
                        self.data_deadpool.insert(dref.clone(), data);
                    }
                }
            }

            // Remove from parent if any
            if let Some(parent) = &node.parent {
                if let Some(parent) = parent.node_mut(self) {
                    parent.remove_child(&node.id);
                }
            }

            // Make sure its not in the roots..
            self.roots.remove(&node.id);

            // Remove all children
            for child in &node.children {
                self.remove_node(child);
            }

            // Insert into the deadpool
            self.node_deadpool.insert(node.id.clone(), node);

            return true;
        }
        false
    }

    /// All child nodes for a given node.
    pub fn all_child_nodes(&self, nref: &NodeRef, include_self: bool) -> FxHashSet<NodeRef> {
        let mut set = FxHashSet::default();
        if include_self { set.insert(nref.clone()); }
        if let Some(node) = nref.node(self) {
            for child in &node.children {
                set.insert(child.clone());
                for id in self.all_child_nodes(child, false) {
                    set.insert(id);
                }
            }
        }
        set
    }

    
}
