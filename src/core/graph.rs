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

use std::{collections::{HashMap, HashSet}, ops::{Index, IndexMut}};
use anyhow::Result;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use crate::{lang::SError, SField};
use super::{IntoDataRef, IntoNodeRef, SData, SDataRef, SDataStore, SNode, SNodeRef, SNodeStore, SRef, Store, DATA_DIRTY_NODES};


/// Stof graph versions.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum SVersion {
    #[default]
    V1 = 1,
}


/// Stof Graph.
/// Holds and structures data in a Stof document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SGraph {
    pub id: String,
    pub name: String,
    pub version: SVersion,
    pub roots: Vec<SNodeRef>,
    pub nodes: Box<SNodeStore>,
    pub data: Box<SDataStore>,
}
impl Default for SGraph {
    fn default() -> Self {
        let id = nanoid!();
        Self {
            id: id.clone(),
            name: id,
            version: Default::default(),
            roots: Default::default(),
            nodes: Default::default(),
            data: Default::default(),
        }
    }
}
impl Index<&str> for SGraph {
    type Output = SNode;
    fn index(&self, index: &str) -> &Self::Output {
        self.node_from(index, None).expect("Node not found for path")
    }
}
impl IndexMut<&str> for SGraph {
    fn index_mut(&mut self, index: &str) -> &mut Self::Output {
        self.node_from_mut(index, None).expect("Node not found for path")
    }
}
impl SGraph {
    /// Create a new Stof Graph.
    pub fn new(name: &str) -> Self {
        Self {
            id: nanoid!(),
            name: name.to_owned(),
            ..Default::default()
        }
    }

    /// Create a new Stof Graph with an ID.
    pub fn new_id(name: &str, id: &str) -> Self {
        Self {
            id: id.to_owned(),
            name: name.to_owned(),
            ..Default::default()
        }
    }


    /*****************************************************************************
     * Node Helpers.
     *****************************************************************************/

    /// Get a node from within this graph.
    pub fn node(&self, nref: impl SRef) -> Option<&SNode> {
        self.nodes.get(&nref.get())
    }

    /// Get a mutable node from within this graph.
    pub fn node_mut(&mut self, nref: impl SRef) -> Option<&mut SNode> {
        self.nodes.get_mut(&nref.get())
    }

    /// Node ref from a path.
    pub fn node_ref(&self, path: &str, start: Option<&SNodeRef>) -> Option<SNodeRef> {
        SNodeRef::node_from(self, path, start)
    }

    /// Node refs from a path.
    /// All nodes that collide in path.
    pub fn node_refs(&self, path: &str) -> Vec<SNodeRef> {
        SNodeRef::nodes_from(self, path)
    }

    /// Node from a path.
    pub fn node_from(&self, path: &str, start: Option<&SNodeRef>) -> Option<&SNode> {
        if let Some(node_ref) = self.node_ref(path, start) {
            return node_ref.node(self);
        }
        None
    }

    /// Mutable node from a path.
    pub fn node_from_mut(&mut self, path: &str, start: Option<&SNodeRef>) -> Option<&mut SNode> {
        if let Some(node_ref) = self.node_ref(path, start) {
            return node_ref.node_mut(self);
        }
        None
    }


    /*****************************************************************************
     * Root Helpers.
     *****************************************************************************/
    
    /// Root count.
    /// The number of roots this graph has.
    pub fn root_count(&self) -> usize {
        self.roots.len()
    }

    /// Main root.
    /// Defined as the first root in the graph.
    pub fn main_root(&self) -> Option<SNodeRef> {
        if self.roots.len() > 0 {
            Some(self.roots.first().unwrap().clone())
        } else {
            None
        }
    }

    /// Get a root by its name.
    pub fn root_by_name(&self, name: &str) -> Option<SNodeRef> {
        for root in &self.roots {
            if let Some(node) = root.node(self) {
                if node.name == name { return Some(root.clone()); }
            }
        }
        None
    }

    /// Is root ID?
    pub fn is_root_id(&self, id: &str) -> bool {
        for root in &self.roots {
            if root.id == id { return true; }
        }
        false
    }

    /// Push a root.
    pub fn push_root(&mut self, id: &str) -> bool {
        if !self.is_root_id(id) {
            self.roots.push(SNodeRef::from(id));
            return true;
        }
        false
    }

    /// Remove a root.
    pub fn remove_root(&mut self, id: &str) -> bool {
        let mut count = 0;
        self.roots.retain(|x| -> bool {
            let keep = x.id != id;
            if !keep { count += 1; }
            keep
        });
        count > 0
    }


    /*****************************************************************************
     * Data Helpers.
     *****************************************************************************/
    
    /// Data reference from an ID.
    /// Checks whether the data exists.
    pub fn data_ref(&self, id: &str) -> Option<SDataRef> {
        if self.data.contains(id) {
            return Some(SDataRef::from(id));
        }
        None
    }

    /// Data from ref.
    pub fn data_from_ref(&self, to_ref: impl IntoDataRef) -> Option<&SData> {
        to_ref.data_ref().data(self)
    }

    /// Data from an ID.
    pub fn data_from_id(&self, id: &str) -> Option<&SData> {
        self.data.get(id)
    }

    /// Mutable data from an ID.
    pub fn data_from_id_mut(&mut self, id: &str) -> Option<&mut SData> {
        self.data.get_mut(id)
    }


    /*****************************************************************************
     * Nodes.
     *****************************************************************************/
    
    /// Insert a root node.
    pub fn insert_root(&mut self, name: &str) -> SNodeRef {
        let mut node = SNode::new(name);
        let node_ref = node.node_ref();
        node.invalidate_all();
        self.nodes.set(&node.id.clone(), node);
        self.push_root(&node_ref.id);
        node_ref
    }

    /// Insert a node.
    pub fn insert_node(&mut self, name: &str, parent: Option<&SNodeRef>) -> SNodeRef {
        let node = SNode::new(name);
        self.insert_node_raw(node, parent)
    }

    /// Insert a node with an ID.
    pub fn insert_node_with_id(&mut self, name: &str, id: &str, parent: Option<&SNodeRef>) -> SNodeRef {
        let node = SNode::new_id(name, id);
        self.insert_node_raw(node, parent)
    }

    /// Insert a raw node.
    pub fn insert_node_raw(&mut self, mut node: SNode, parent: Option<&SNodeRef>) -> SNodeRef {
        if let Some(parent) = parent {
            if parent.exists(self) {
                node.parent = Some(parent.clone());
            } else {
                node.parent = None;
            }
        } else {
            node.parent = None;
        }
        let res = node.node_ref();
        node.invalidate_all();
        self.nodes.set(&node.id.clone(), node);

        if let Some(parent) = parent {
            if let Some(parent_node) = parent.node_mut(self) {
                parent_node.put_child(&res);
            }
        } else {
            self.push_root(&res.id);
        }
        res
    }

    /// Ensure node path exists.
    /// Returns the last node ref created in the chain.
    pub fn ensure_nodes(&mut self, path: &str, sep: char, fields: bool, start: Option<SNodeRef>) -> SNodeRef {
        let mut current: Option<SNodeRef> = start;
        for segment in path.split(sep).collect::<Vec<&str>>() {
            if let Some(node) = self.node_ref(segment, current.as_ref()) {
                current = Some(node);
            } else {
                if fields && current.is_some() {
                    current = Some(crate::SField::new_object(self, segment, &current.unwrap()));
                } else {
                    current = Some(self.insert_node(segment, current.as_ref()));
                }
            }
        }
        if let Some(nref) = current {
            return nref;
        }
        Default::default()
    }

    /// Rename a node.
    pub fn rename_node(&mut self, node: impl IntoNodeRef, new_name: &str) -> bool {
        let node_ref = node.node_ref();
        if let Some(node) = node_ref.node_mut(self) {
            node.name = new_name.to_owned();
            return true;
        }
        false
    }

    /// Remove node from this graph.
    /// Removes all data on this node, and potentially from the graph also.
    pub fn remove_node(&mut self, node: impl IntoNodeRef) -> bool {
        let node_ref = node.node_ref();
        if !node_ref.exists(self) { return false; }

        // Remove all data from this node!
        // May or may not remove data completely, depending on if there are other references
        let mut data_to_remove: Vec<SDataRef> = vec![];
        let mut nodes_to_remove: Vec<SNodeRef> = vec![];
        let mut parent: Option<SNodeRef> = None;
        if let Some(node) = node_ref.node(self) {
            data_to_remove = node.data.iter().cloned().collect();
            nodes_to_remove = node.children.iter().cloned().collect();
            parent = node.parent.clone();
        }
        for dref in &data_to_remove {
            self.remove_data(dref, Some(&node_ref));
        }

        // Remove node from the graph before child nodes (children won't find this as parent)
        self.nodes.remove(&node_ref.id);

        // Remove all children of the node from the graph also
        for nref in &nodes_to_remove {
            self.remove_node(nref);
        }

        // Remove node from parent if possible
        if parent.is_some() {
            if let Some(parent) = parent.unwrap().node_mut(self) {
                parent.remove_child(&node_ref);
            }
        }

        // Remove node as a root if needed
        self.remove_root(&node_ref.id);

        true
    }

    /// All children nodes.
    /// Recursively get all children of a node.
    pub fn all_children(&self, node: impl IntoNodeRef) -> Vec<SNodeRef> {
        let node_ref = node.node_ref();
        let mut map = HashMap::new();
        if let Some(node) = node_ref.node(self) {
            for child_ref in &node.children {
                map.insert(child_ref.id.clone(), self.all_children(child_ref));
            }
        }
        let mut res = Vec::new();
        for (id, mut children) in map {
            res.push(id.into());
            res.append(&mut children);
        }
        res
    }

    /// Move node up.
    /// Make 'node_ref' node a child of its parent's parent (now a sibling of its parent).
    /// Returns all nodes that have been affected by this action.
    pub fn move_node_up(&mut self, node: impl IntoNodeRef, allow_root: bool) -> Vec<SNodeRef> {
        let node_ref = node.node_ref();
        let mut push_child = Vec::new();
        let mut remove_child = Vec::new();

        if let Some(node) = node_ref.node(self) {
            if let Some(parent) = &node.parent {
                if let Some(parent) = parent.node(self) {
                    if let Some(grandparent) = &parent.parent {
                        if let Some(grandparent) = grandparent.node(self) {
                            push_child.push(grandparent.node_ref());
                            remove_child.push(parent.node_ref());
                        }
                    } else if allow_root {
                        // Add 'node' to roots of this graph
                        remove_child.push(parent.node_ref());
                    }
                }
            }
        }

        let mut res = vec![];
        if remove_child.len() > 0 {
            // Remove child will only ever be one element, the parent
            if let Some(parent) = remove_child[0].node_mut(self) {
                parent.remove_child(&node_ref);
            }

            // If push_child len > 0, we have a grandparent
            if push_child.len() > 0 {
                if let Some(grandparent) = push_child[0].node_mut(self) {
                    grandparent.put_child(&node_ref);
                }
                if let Some(node) = node_ref.node_mut(self) {
                    node.parent = Some(push_child[0].clone());
                }
            } else {
                // The node should be added to roots and no longer has a parent
                self.push_root(&node_ref.id);
                if let Some(node) = node_ref.node_mut(self) {
                    node.parent = None;
                }
            }

            // Append the nodes that were modified
            res.append(&mut push_child);
            res.append(&mut remove_child);
            res.push(node_ref.clone());
        }

        // Invalidate the affected nodes
        for nref in &res {
            if let Some(node) = nref.node_mut(self) {
                node.invalidate_all();
            }
        }
        res
    }

    /// Move node to another node 'destination'.
    /// Since this is a DAG, destination cannot be a descendant of the node (branch loss).
    /// This method does NOT check this, so don't be dumb.
    pub fn move_node(&mut self, source: impl IntoNodeRef, destination: impl IntoNodeRef) -> Vec<SNodeRef> {
        let node_ref = source.node_ref();
        let destination = destination.node_ref();
        if !node_ref.exists(self) || !destination.exists(self) { return vec![]; }
        let mut res: Vec<SNodeRef> = vec![];

        // Add node_ref to destination node
        res.push(destination.clone());
        res.push(node_ref.clone());
        if let Some(new_parent) = destination.node_mut(self) {
            new_parent.put_child(&node_ref);
        }

        // Make node_ref parent the new destination
        let mut existing_parent: Option<SNodeRef> = None;
        if let Some(node) = node_ref.node_mut(self) {
            if node.parent.is_some() {
                existing_parent = node.parent.clone();
            }
            node.parent = Some(destination.clone());
        }

        // Remove the node from the old parent
        if let Some(old_parent) = existing_parent {
            if let Some(old_parent) = old_parent.node_mut(self) {
                old_parent.remove_child(&node_ref);
            }
            res.push(old_parent);
        } else {
            // Remove from the roots
            self.remove_root(&node_ref.id);
        }

        // Invalidate the affected nodes
        for nref in &res {
            if let Some(node) = nref.node_mut(self) {
                node.invalidate_all();
            }
        }
        res
    }

    /// Absorb the data and children of "node" into "onto".
    pub fn absorb_external_node(&mut self, graph: &Self, node: &SNode, onto: &SNodeRef) {
        for dref in &node.data {
            if let Some(data) = dref.data(graph) {
                let mut data = data.clone();
                data.invalidate(DATA_DIRTY_NODES);

                // Remove node references that don't exist on this graph
                let mut nref_remove = Vec::new();
                for i in 0..data.nodes.len() { if !data.nodes[i].exists(&self) { nref_remove.push(i); } }
                nref_remove.reverse();
                for i in nref_remove { data.nodes.remove(i); }

                // Put the cloned data onto the node
                self.put_data(onto, data);
            }
        }
        for child_ref in &node.children {
            if let Some(child) = child_ref.node(graph) {
                self.insert_external_node(graph, child, Some(onto), None);
            }
        }
    }

    /// Add a unique node (clone) to the current node from another graph.
    pub fn insert_external_node(&mut self, graph: &Self, node: &SNode, parent: Option<&SNodeRef>, rename: Option<String>) {
        let mut cloned = node.clone();
        if let Some(new_name) = rename {
            cloned.name = new_name;
        }
        let clone = self.insert_node_raw(cloned, parent);
        
        // Add all data from node to clone
        let mut to_remove = Vec::new();
        for dref in &node.data {
            if let Some(data) = dref.data(graph) {
                let mut data = data.clone();
                data.invalidate(DATA_DIRTY_NODES);
                
                // Remove node references that don't exist on this graph
                let mut nref_remove = Vec::new();
                for i in 0..data.nodes.len() { if !data.nodes[i].exists(&self) { nref_remove.push(i); } }
                nref_remove.reverse();
                for i in nref_remove { data.nodes.remove(i); }

                // Put the cloned data onto the cloned node
                self.put_data(&clone, data);
            } else {
                to_remove.push(dref.clone());
            }
        }

        // Get rid of any data references that don't exist for clone
        if to_remove.len() > 0 {
            if let Some(clone_node) = clone.node_mut(self) {
                for data_ref in &to_remove {
                    clone_node.remove_data(data_ref);
                }
            }
        }

        // Add all children of node to clone
        for child_ref in &node.children {
            if let Some(child) = child_ref.node(graph) {
                self.insert_external_node(graph, child, Some(&clone), None);
            }
        }
    }


    /*****************************************************************************
     * Data.
     *****************************************************************************/
    
    /// Put data into the graph and onto a node.
    /// Will overwrite data with the same ID if already in this graph.
    pub fn put_data(&mut self, node: impl IntoNodeRef, mut data: SData) -> Option<SDataRef> {
        let node_ref = node.node_ref();
        if let Some(node) = node_ref.node_mut(self) {
            let res = data.data_ref();
            // Add data to this node - no check needed as it is new data
            if node.put_data(&res) {
                data.new_reference(node_ref.clone());
            }
            // Insert the data into this graph (overridding if ID already exists)
            self.data.set(&data.id.clone(), data);
            return Some(res);
        }
        None
    }

    /// Put data ref onto a node.
    pub fn put_data_ref(&mut self, node: impl IntoNodeRef, data: impl IntoDataRef) -> bool {
        let node_ref = node.node_ref();
        let data_ref = data.data_ref();
        if !data_ref.exists(self) { return false; }
        let mut added = false;
        if let Some(node) = node_ref.node_mut(self) {
            if node.put_data(&data_ref) {
                added = true;
            }
        }
        if added {
            if let Some(data) = data_ref.data_mut(self) {
                data.new_reference(node_ref.clone());
            }
            return true;
        }
        false
    }

    /// Remove data from this graph.
    /// If a node is not given, the data will be removed completely.
    /// If a node is specified, the data will be removed only from that node.
    /// If the data only exists on that node, the data is removed completely.
    pub fn remove_data(&mut self, to_ref: impl IntoDataRef, node: Option<&SNodeRef>) -> bool {
        let data_ref = to_ref.data_ref();
        if !data_ref.exists(self) { return false; }
        let mut res = false;

        // Remove completely if no node specified, or node has the only ref to data
        let mut remove_completely = true;

        // If a node was specified, remove the data from the node first, deincrement the refs, and evaluate completeness
        if let Some(node) = node {
            remove_completely = false; // Data might not exist on this node, in that case, don't assume no refs

            let mut removed = false;
            if let Some(node) = node.node_mut(self) {
                if node.remove_data(&data_ref) {
                    removed = true;
                }
            }
            if removed {
                res = true;
                if let Some(data) = data_ref.data_mut(self) {
                    data.ref_removed(node);
                    remove_completely = data.ref_count() < 1;
                }
            }
        }

        // If remove completely, remove from this data and all nodes
        if remove_completely {
            // If we haven't contemplated nodes yet, remove data from all of them
            if node.is_none() {
                let mut removed_from: Vec<SNodeRef> = Vec::new();
                for (_, node) in &mut self.nodes.store {
                    if node.remove_data(&data_ref) {
                        removed_from.push(node.node_ref());
                    }
                }
                if let Some(data) = data_ref.data_mut(self) {
                    for nref in &removed_from {
                        data.ref_removed(nref)
                    }
                }
            }

            // Remove from the data manager
            res = self.data.remove(&data_ref.id);
        }
        res
    }


    /*****************************************************************************
     * Flush.
     *****************************************************************************/
    
    /// Flush node deadpool.
    /// Returns an array of node IDs that have been removed from the graph.
    pub fn flush_node_deadpool(&mut self) -> Vec<SNode> {
        self.nodes.flush_deadpool()
    }

    /// Flush data deadpool.
    /// Returns an array of data IDs that have been removed from the graph.
    pub fn flush_data_deadpool(&mut self) -> Vec<SData> {
        self.data.flush_deadpool()
    }

    /// Flush nodes.
    /// Collect dirty nodes for validation.
    /// For no limit, pass -1.
    pub fn flush_nodes(&mut self, limit: i32) -> Vec<&mut SNode> {
        self.nodes.flush(limit)
    }

    /// Flush data.
    /// Collect dirty data for validation.
    /// For no limit, pass -1.
    pub fn flush_data(&mut self, limit: i32) -> Vec<&mut SData> {
        self.data.flush(limit)
    }


    /*****************************************************************************
     * Absorb.
     *****************************************************************************/
    
    /// Absorb another graph completely.
    /// Up to you to make sure collisions in names, etc.. are handled beforehand.
    /// This is meant to be efficient.
    pub fn absorb_graph(&mut self, other: Self) {
        for root_node in other.roots {
            self.push_root(&root_node.id);
        }
        for (id, node) in other.nodes.store {
            self.nodes.set(&id, node);
        }
        for (id, data) in other.data.store {
            self.data.set(&id, data);
        }
    }

    /// Get node collisions.
    /// These two nodes collide in path, so get all collisions under them, recursively!
    fn get_node_collisions(&self, node: &SNodeRef, other: &Self, other_node: &SNodeRef) -> (Vec<SNodeRef>, Vec<SNodeRef>) {
        let mut node_collisions = Vec::new();
        let mut other_collisions = Vec::new();

        if let Some(node) = node.node(&self) {
            if let Some(other_node) = other_node.node(&other) {
                for child_ref in &node.children {
                    if let Some(child) = child_ref.node(&self) {
                        for other_child_ref in &other_node.children {
                            if let Some(other_child) = other_child_ref.node(&other) {
                                if child.name == other_child.name {
                                    node_collisions.push(child_ref.clone());
                                    other_collisions.push(other_child_ref.clone());
                                    
                                    let (mut nodes, mut others) = self.get_node_collisions(child_ref, other, other_child_ref);
                                    node_collisions.append(&mut nodes);
                                    other_collisions.append(&mut others);
                                }
                            }
                        }
                    }
                }
            }
        }
        return (node_collisions, other_collisions);
    }

    /// Get collisions between this graph and another graph.
    /// Returns all of the nodes that collided, and the nodes on other that collided in a separate set.
    pub fn get_collisions(&self, other: &Self) -> (Vec<SNodeRef>, Vec<SNodeRef>) {
        let mut node_collisions = Vec::new();
        let mut other_collisions = Vec::new();
        for root_ref in &self.roots {
            if let Some(root) = root_ref.node(&self) {
                for other_root_ref in &other.roots {
                    if let Some(other_root) = other_root_ref.node(&other) {
                        if root.name == other_root.name {
                            node_collisions.push(root_ref.clone());
                            other_collisions.push(other_root_ref.clone());
                            
                            let (mut nodes, mut others) = self.get_node_collisions(root_ref, other, other_root_ref);
                            node_collisions.append(&mut nodes);
                            other_collisions.append(&mut others);
                        }
                    }
                }
            }
        }
        return (node_collisions, other_collisions);
    }

    /// Absorb another graph and merge it with this one.
    pub fn absorb_merge(&mut self, mut other: Self, add_unique_other: bool,
        mut collision_handler: impl FnMut(&mut Self, &mut Self, &mut (SNodeRef, SNodeRef)) -> Result<(), SError>,
        mut unique_self_handler: impl FnMut(&mut Self, &SNodeRef) -> Result<(), SError>) -> Result<(), SError> {
        let collisions = self.get_collisions(&other);

        for index in 0..collisions.0.len() {
            if index < collisions.1.len() {
                let mut collide = (collisions.0[index].clone(), collisions.1[index].clone());
                collision_handler(self, &mut other, &mut collide)?;
            }
        }

        if add_unique_other {
            let other_collided: HashSet<SNodeRef> = collisions.1.iter().cloned().collect();
            for (_, node) in other.nodes.store {
                if !other_collided.contains(&node.node_ref()) {
                    // Transfer data over
                    for dref in &node.data {
                        if let Some(data) = other.data.store.remove(&dref.id) {
                            self.data.set(&data.id.clone(), data);
                        }
                    }
                    // Transfer node over
                    if node.parent.is_none() {
                        self.roots.push(node.node_ref());
                    }
                    self.nodes.set(&node.id.clone(), node);
                }
            }
        }

        let self_collided: HashSet<SNodeRef> = collisions.0.iter().cloned().collect();
        let mut unique_self = Vec::new();
        for (_, node) in &self.nodes.store {
            if !self_collided.contains(&node.node_ref()) {
                unique_self.push(node.node_ref());
            }
        }
        for unique_ref in unique_self {
            unique_self_handler(self, &unique_ref)?;
        }
        Ok(())
    }

    /// Default version of absorb merge.
    /// Used in imports of additional graphs.
    pub fn default_absorb_merge(&mut self, other: Self) -> Result<(), SError> {    
        self.absorb_merge(other, true,
        |graph, other, nodes| {
            // Move all data from these nodes to the first node on 'graph' at this path
            let mut data = HashSet::new();

            let mut children: Vec<SNodeRef> = Vec::new(); // children nodes of all others
            let mut other_fields = Vec::new();
            if let Some(node) = nodes.1.node(other) {
                for dref in &node.data {
                    data.insert(dref.clone());

                    if SData::type_of::<SField>(&other, dref) {
                        other_fields.push(dref.clone());
                    }
                }
                children.append(&mut node.children.iter().cloned().collect());
            }
            
            // Merge both sets of fields, then insert/set them on this graph node
            SField::merge_fields(graph, &nodes.0, &other, &other_fields)?;

            // Add children to this node
            if let Some(node) = nodes.0.node_mut(graph) {
                for child in &children {
                    node.put_child(child);
                }
            }
            for child in &children {
                if let Some(child) = child.node_mut(graph) {
                    child.parent = Some(nodes.0.clone());
                }
            }

            // Add all data onto this graph
            for dref in &data {
                if !SData::type_of::<SField>(&other, dref) {
                    if let Some(data) = dref.data(other) {
                        graph.put_data(&nodes.0, data.clone());
                    }
                }
            }
            Ok(())
        },
        |_graph, _node| {
            // We like unique selfs
            Ok(())
        })?;
        Ok(())
    }


    /*****************************************************************************
     * Dump.
     *****************************************************************************/
    
    /// Dump this graph for debugging.
    pub fn dump(&self, data: bool) {
        println!("Dump SGraph: {} (ver: {:?})", &self.name, &self.version);
        for root_ref in &self.roots {
            if let Some(root) = root_ref.node(self) {
                println!("{}", root.dump(self, 0, data));
            }
        }
        println!("END DUMP");
    }
}


#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use crate::{Data, SData, SVersion};
    use super::SGraph;

    #[derive(Deserialize, Serialize, Debug, Clone)]
    struct MyData {
        name: String,
    }
    #[typetag::serde(name = "Test::MyData")]
    impl Data for MyData {}

    #[test]
    fn default_constructor() {
        let graph = SGraph::default();
        assert!(graph.id.len() > 0);
        assert!(graph.name.len() > 0);
        assert_eq!(graph.root_count(), 0);
        assert_eq!(graph.nodes.store.len(), 0);
        assert_eq!(graph.data.store.len(), 0);
        assert_eq!(graph.version, SVersion::default());
    }

    #[test]
    fn construct_graph() {
        let mut graph = SGraph::new("graph");
        let cj;
        let amelia;
        let root = graph.insert_node("root", None);
        {
            let base= graph.insert_node("base", Some(&root));
            {
                cj = graph.put_data(&base, SData::new(Box::new(MyData { name: "CJ".to_owned() })));
            }
            let top = graph.insert_node("top", Some(&root));
            {
                amelia = graph.put_data(&top, SData::new(Box::new(MyData { name: "Amelia".to_owned() })));
            }
        }

        let binary = bincode::serialize(&graph).unwrap();
        let gph = bincode::deserialize::<SGraph>(&binary).unwrap();

        let cj = SData::get::<MyData>(&gph, &cj).unwrap();
        let amelia = SData::get::<MyData>(&gph, &amelia).unwrap();
        assert_eq!(cj.name, "CJ");
        assert_eq!(amelia.name, "Amelia");
    }

    #[test]
    fn insert_root() {
        let mut graph = SGraph::new("graph");
        let cj;
        let amelia;
        let root = graph.insert_root("root");
        {
            let base= graph.insert_node("base", Some(&root));
            {
                cj = graph.put_data(&base, SData::new(Box::new(MyData { name: "CJ".to_owned() })));
            }
            let top = graph.insert_node("top", Some(&root));
            {
                amelia = graph.put_data(&top, SData::new(Box::new(MyData { name: "Amelia".to_owned() })));
            }
        }

        let binary = bincode::serialize(&graph).unwrap();
        let mut gph = bincode::deserialize::<SGraph>(&binary).unwrap();

        {
            let cj = SData::get::<MyData>(&gph, &cj).unwrap();
            let amelia = SData::get::<MyData>(&gph, &amelia).unwrap();
            assert_eq!(cj.name, "CJ");
            assert_eq!(amelia.name, "Amelia");
        }

        if let Some(mut_cj) = SData::get_mut::<MyData>(&mut gph, &cj) {
            mut_cj.name = "DUDE".to_string();
        }

        {
            let cj = SData::get::<MyData>(&gph, &cj).unwrap();
            let amelia = SData::get::<MyData>(&gph, &amelia).unwrap();
            assert_eq!(cj.name, "DUDE");
            assert_eq!(amelia.name, "Amelia");
        }
    }

    #[test]
    fn node_path() {
        let mut graph = SGraph::default();
        let root = graph.insert_root("root");
        let base;
        let top;
        {
            base = graph.insert_node("base", Some(&root));
            {
                graph.insert_node("a", Some(&base));
                graph.insert_node("b", Some(&base));
                graph.insert_node("b", Some(&base));
            }
            top = graph.insert_node("top", Some(&root));
            {
                graph.insert_node("a", Some(&top));
                graph.insert_node("b", Some(&top));
            }
        }
        assert!(graph.node_ref("root/base/b", Some(&root)).is_some());
        assert!(graph.node_ref("root/top/b", Some(&root)).is_some());
        assert!(graph.node_ref("base/b", Some(&root)).is_some());
        assert!(graph.node_ref("top/b", Some(&root)).is_some());
        assert!(graph.node_ref("b", Some(&root)).is_none());
        assert!(graph.node_ref("self/base/self/super/base/a/self", Some(&root)).is_some());
        assert!(graph.node_ref("./top/../top/a", Some(&root)).is_some());
        assert!(graph.node_ref("base/a", Some(&root)).is_some());
        assert!(graph.node_ref("top/a/.", Some(&root)).is_some());
        assert!(graph.node_ref("a", Some(&root)).is_none());
        assert!(graph.node_ref("root/top/super/top", Some(&root)).is_some());
        assert!(graph.node_ref("top", Some(&root)).is_some());
        assert!(graph.node_ref("self/self/self/base", Some(&root)).is_some());
        assert!(graph.node_ref("base", Some(&root)).is_some());

        assert!(graph.node_ref("root/base/b", None).is_some());
        assert!(graph.node_ref("root/top/b", None).is_some());
        assert!(graph.node_ref("base/b", None).is_some());
        assert!(graph.node_ref("top/b", None).is_some());
        assert!(graph.node_ref("b", None).is_some());
        assert!(graph.node_ref("root/base/a", None).is_some());
        assert!(graph.node_ref("root/top/a", None).is_some());
        assert!(graph.node_ref("base/a", None).is_some());
        assert!(graph.node_ref("top/a", None).is_some());
        assert!(graph.node_ref("a", None).is_some());
        assert!(graph.node_ref("root/top", None).is_some());
        assert!(graph.node_ref("top", None).is_some());
        assert!(graph.node_ref("root/base", None).is_some());
        assert!(graph.node_ref("base", None).is_some());

        assert!(graph.node_ref("a", Some(&base)).is_some());
        assert!(graph.node_ref("a", Some(&top)).is_some());
        assert!(graph.node_ref("b", Some(&base)).is_some());
        assert!(graph.node_ref("b", Some(&top)).is_some());

        assert_eq!(graph.node_refs("base/b").len(), 2);
        assert_eq!(graph.node_refs("root/base/b").len(), 2);
        assert_eq!(graph.node_refs("base/a").len(), 1);
        assert_eq!(graph.node_refs("top/a").len(), 1);
        assert_eq!(graph.node_refs("top/b").len(), 1);
        assert_eq!(graph.node_refs("root/top/a").len(), 1);
        assert_eq!(graph.node_refs("root/top/b").len(), 1);
        assert_eq!(graph.node_refs("top").len(), 1);
        assert_eq!(graph.node_refs("base").len(), 1);
        assert_eq!(graph.node_refs("root/top").len(), 1);
        assert_eq!(graph.node_refs("root/base").len(), 1);
    }
}
