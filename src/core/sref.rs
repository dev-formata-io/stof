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

use std::{collections::HashSet, fmt::Display, mem::swap};
use bincode::{Error, ErrorKind};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use crate::{IntoSVal, SField, SUnits, SVal};

use super::{Data, SData, SGraph, SNode, Store};


/// Stof Ref Trait.
pub trait SRef {
    /// Get the ID for this ref.
    fn get(&self) -> String;

    /// Set the ID for this ref.
    fn set(&mut self, id: &str);
}


/// Into SNodeRef.
pub trait IntoNodeRef {
    fn node_ref(&self) -> SNodeRef;
}


/// Into SDataRef.
pub trait IntoDataRef {
    fn data_ref(&self) -> SDataRef;
}


/// SNode Ref.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct SNodeRef {
    pub id: String,
}
impl SRef for SNodeRef {
    fn get(&self) -> String {
        self.id.clone()
    }
    fn set(&mut self, id: &str) {
        self.id = id.to_owned();
    }
}
impl IntoNodeRef for SNodeRef {
    fn node_ref(&self) -> SNodeRef {
        SNodeRef::from(&self.id)
    }
}
impl IntoNodeRef for &SNodeRef {
    fn node_ref(&self) -> SNodeRef {
        SNodeRef::from(&self.id)
    }
}
impl IntoNodeRef for SNode {
    fn node_ref(&self) -> SNodeRef {
        SNodeRef::from(&self.id)
    }
}
impl IntoNodeRef for &SNode {
    fn node_ref(&self) -> SNodeRef {
        SNodeRef::from(&self.id)
    }
}
impl IntoNodeRef for &str {
    fn node_ref(&self) -> SNodeRef {
        SNodeRef::from(*self)
    }
}
impl IntoNodeRef for String {
    fn node_ref(&self) -> SNodeRef {
        SNodeRef::from(self)
    }
}
impl IntoNodeRef for &String {
    fn node_ref(&self) -> SNodeRef {
        SNodeRef::from(self.as_str())
    }
}
impl IntoNodeRef for Option<SNodeRef> {
    fn node_ref(&self) -> SNodeRef {
        if let Some(rf) = &self {
            return SNodeRef::from(&rf.id);
        }
        SNodeRef::default()
    }
}
impl IntoNodeRef for &Option<SNodeRef> {
    fn node_ref(&self) -> SNodeRef {
        if let Some(rf) = &self {
            return SNodeRef::from(&rf.id);
        }
        SNodeRef::default()
    }
}
impl IntoNodeRef for Option<&SNodeRef> {
    fn node_ref(&self) -> SNodeRef {
        if let Some(rf) = self {
            return SNodeRef::from(&rf.id);
        }
        SNodeRef::default()
    }
}
impl IntoNodeRef for &Option<&SNodeRef> {
    fn node_ref(&self) -> SNodeRef {
        if let Some(rf) = self {
            return SNodeRef::from(&rf.id);
        }
        SNodeRef::default()
    }
}
impl From<&str> for SNodeRef {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}
impl From<String> for SNodeRef {
    fn from(value: String) -> Self {
        Self {
            id: value,
        }
    }
}
impl From<&String> for SNodeRef {
    fn from(value: &String) -> Self {
        Self::new(value)
    }
}
impl Display for SNodeRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}
impl SNodeRef {
    /// New node ref.
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_owned(),
        }
    }

    /// Get the node.
    pub fn node<'a>(&self, graph: &'a SGraph) -> Option<&'a SNode> {
        graph.nodes.get(&self.id)
    }

    /// Get mutable node.
    pub fn node_mut<'a>(&self, graph: &'a mut SGraph) -> Option<&'a mut SNode> {
        graph.nodes.get_mut(&self.id)
    }

    /// Exists?
    pub fn exists(&self, graph: &SGraph) -> bool {
        graph.nodes.contains(&self.id)
    }

    /// Root node for this ref.
    pub fn root<'a>(&self, graph: &'a SGraph) -> Option<&'a SNode> {
        if let Some(node) = self.node(graph) {
            if let Some(parent) = &node.parent {
                return parent.root(graph);
            }
            return Some(node);
        }
        None
    }

    /// Is a child of 'parent'?
    /// Counts the same node as true.
    pub fn is_child_of(&self, graph: &SGraph, parent: &SNodeRef) -> bool {
        if self.id == parent.id { return true; }
        if let Some(node) = self.node(graph) {
            if let Some(par) = &node.parent {
                return par.is_child_of(graph, parent);
            }
        }
        false
    }

    /// Path.
    pub fn path(&self, graph: &SGraph) -> String {
        let mut node = self.node(graph);
        if node.is_some() {
            let mut res: Vec<String> = vec![];
            let mut seen: HashSet<String> = HashSet::new();
            while node.is_some() {
                let node_inner = node.unwrap();
                if seen.contains(&node_inner.id) { break; }

                res.push(node_inner.name.clone());
                seen.insert(node_inner.id.clone());
                if let Some(par) = &node_inner.parent {
                    node = par.node(graph);
                } else {
                    node = None;
                }
            }
            res.reverse();
            return res.join("/");
        }
        String::new()
    }

    /// ID path.
    pub fn id_path(&self, graph: &SGraph) -> Vec<String> {
        let mut node = self.node(graph);
        if node.is_some() {
            let mut res: Vec<String> = vec![];
            let mut seen: HashSet<String> = HashSet::new();
            while node.is_some() {
                let node_inner = node.unwrap();
                if seen.contains(&node_inner.id) { break; }

                res.push(node_inner.id.clone());
                seen.insert(node_inner.id.clone());
                if let Some(par) = &node_inner.parent {
                    node = par.node(graph);
                } else {
                    node = None;
                }
            }
            res.reverse();
            return res;
        }
        Vec::new()
    }

    /// Get a field on this node if it exists (dot separated path).
    pub fn dot_field(&self, graph: &SGraph, path: &str) -> Option<SField> {
        self.field(graph, path, '.')
    }

    /// Get a field on this node if it exists (slash separated path).
    pub fn slash_field(&self, graph: &SGraph, path: &str) -> Option<SField> {
        self.field(graph, path, '/')
    }
    
    /// Get a field on this node if it exists.
    pub fn field(&self, graph: &SGraph, path: &str, sep: char) -> Option<SField> {
        SField::field(graph, path, sep, Some(&self))
    }

    /// Get the first field that matches a given path.
    pub fn first_field_match(&self, graph: &SGraph, paths: Vec<&str>, sep: char) -> Option<SField> {
        SField::first_match(graph, paths, sep, Some(&self))
    }

    /// Get all fields on this node.
    pub fn fields(&self, graph: &SGraph) -> Vec<SField> {
        SField::fields(graph, self)
    }

    /// Create a new field on this node.
    pub fn new_field(&self, graph: &mut SGraph, name: &str, value: impl IntoSVal) -> SField {
        let mut field = SField::new(name, value);
        field.attach(&self, graph);
        field
    }
    
    /// Distance to another node in the graph.
    /// If same node, distance is 0.
    /// If nodes are not in the same graph or are in different roots, distance is -1.
    /// Otherwise, distance is the path length from this node to other node.
    pub fn distance_to(&self, graph: &SGraph, other: &Self) -> i32 {
        if self.id == other.id { return 0; }

        let mut node_a_id_path = self.id_path(graph);
        let mut node_b_id_path = other.id_path(graph);
        if node_a_id_path.len() < 1 || node_b_id_path.len() < 1 || &node_a_id_path[0] != &node_b_id_path[0] {
            return -1;
        }

        if node_a_id_path.len() > node_b_id_path.len() {
            swap(&mut node_a_id_path, &mut node_b_id_path);
        }

        let mut to_remove: HashSet<String> = HashSet::new();
        let mut last: String = String::new();
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

    /// Construct all node refs from a string path (full path) of node names.
    /// Path is '/' separated.
    pub fn nodes_from(graph: &SGraph, path: &str) -> Vec<Self> {
        if path.len() < 1 { return vec![]; }

        let lower = path.trim();
        let mut names: Vec<&str> = lower.split('/').collect();

        let start_path;
        let mut first_node = None;
        let first = names.remove(0);
        for (_, node) in &graph.nodes.store {
            if node.name == first {
                // To ensure we get all nodes returned, we need to go all the way up to the root
                let nref = node.node_ref();
                if let Some(root) = nref.root(graph) {
                    // Add names to the front of names
                    start_path = nref.path(graph);
                    let mut start_names: Vec<&str> = start_path.split('/').collect();
                    
                    start_names.append(&mut names);
                    names = start_names;
                    names.remove(0);

                    first_node = Some(root);
                } else {
                    first_node = Some(node);
                }
                break;
            }
        }

        if let Some(first_node) = first_node {
            names.reverse();
            let mut seen = HashSet::new();
            let mut res = Vec::new();
            loop {
                let mut new_names = names.clone();
                if let Some(r) = Self::path_constructor(graph, first_node, &mut new_names, &mut seen) {
                    res.push(r);
                } else {
                    break;
                }
            }
            return res;
        }
        vec![]
    }

    /// Construct a node ref from a string path of node names.
    /// Path is '/' separated.
    pub fn node_from(graph: &SGraph, path: &str, start: Option<&Self>) -> Option<Self> {
        if path.len() < 1 { return None; }

        let lower = path.trim();
        let mut names: Vec<&str> = lower.split('/').collect();
        names.reverse();

        let mut first_node = None;
        if let Some(start) = start {
            if let Some(node) = start.node(graph) {
                first_node = Some(node);
            }
        }
        if first_node.is_none() {
            let first = names.pop().unwrap();
            for (_, node) in &graph.nodes.store {
                if node.name == first {
                    first_node = Some(node);
                    break;
                }
            }
        }

        if let Some(first_node) = first_node {
            return Self::path_constructor(graph, first_node, &mut names, &mut HashSet::new());
        }
        None
    }

    /// Internal path constructor.
    fn path_constructor(graph: &SGraph, current: &SNode, names: &mut Vec<&str>, seen: &mut HashSet<String>) -> Option<Self> {
        let next = names.pop();
        if let Some(next) = next {
            // Look in current node's children
            for child_ref in &current.children {
                if let Some(child) = child_ref.node(graph) {
                    if child.name == next {
                        let mut new_names = names.clone();
                        let res = Self::path_constructor(graph, child, &mut new_names, seen);
                        if res.is_some() { return res; }
                    }
                }
            }

            // If not found in children, look at the current nodes parent
            if let Some(parent) = &current.parent {
                if let Some(node) = parent.node(graph) {
                    if next == ".." || next == "super" || node.name == next {
                        return Self::path_constructor(graph, node, names, seen);
                    }
                }
            } else if let Some(root) = graph.root_by_name(next) {
                if let Some(node) = root.node(graph) {
                    return Self::path_constructor(graph, node, names, seen);
                }
            }

            // If its a duplicate, handle that
            if next == "." || next == "self" || current.name == next {
                return Self::path_constructor(graph, current, names, seen);
            }

            // Is it a field in the current node that is an object?
            if let Some(field) = SField::field(graph, next, '/', Some(&current.node_ref())) {
                match field.value {
                    SVal::Object(nref) => {
                        if let Some(node) = nref.node(graph) {
                            return Self::path_constructor(graph, node, names, seen);
                        }
                    },
                    _ => {}
                }
            }

            // Make sure to return None as user expects this to be a specific node...
            // Do not just go as far into the path as possible - might alter data that isn't meant to be altered
            return None;
        }

        if seen.contains(&current.id) {
            return None;
        }
        seen.insert(current.id.clone());
        Some(current.node_ref())
    }


    /*****************************************************************************
     * Field creation helpers.
     *****************************************************************************/
    
    /// New object field on this object.
    #[inline]
    pub fn new_object(&self, graph: &mut SGraph, name: &str) -> Self {
        SField::new_object(graph, name, &self)
    }

    /// Insert an array field on this object.
    #[inline]
    pub fn new_array(&self, graph: &mut SGraph, name: &str, vals: Vec<SVal>) -> SField {
        SField::new_array(graph, name, vals, &self)
    }

    /// Insert a new string field on this object.
    #[inline]
    pub fn new_string(&self, graph: &mut SGraph, name: &str, value: &str) -> SField {
        SField::new_string(graph, name, value, &self)
    }

    /// Insert a new boolean field on this object.
    #[inline]
    pub fn new_bool(&self, graph: &mut SGraph, name: &str, value: bool) -> SField {
        SField::new_bool(graph, name, value, &self)
    }

    /// New integer field on this object.
    #[inline]
    pub fn new_int(&self, graph: &mut SGraph, name: &str, value: i64) -> SField {
        SField::new_int(graph, name, value, &self)
    }

    /// New float field on this object.
    #[inline]
    pub fn new_float(&self, graph: &mut SGraph, name: &str, value: f64) -> SField {
        SField::new_float(graph, name, value, &self)
    }

    /// New float units field on this object.
    #[inline]
    pub fn new_units(&self, graph: &mut SGraph, name: &str, value: f64, units: SUnits) -> SField {
        SField::new_units(graph, name, value, units, &self)
    }
}


/// SData Ref.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct SDataRef {
    pub id: String,
}
impl SRef for SDataRef {
    fn get(&self) -> String {
        self.id.clone()
    }
    fn set(&mut self, id: &str) {
        self.id = id.to_owned();
    }
}
impl IntoDataRef for SDataRef {
    fn data_ref(&self) -> SDataRef {
        SDataRef::from(&self.id)
    }
}
impl IntoDataRef for &SDataRef {
    fn data_ref(&self) -> SDataRef {
        SDataRef::from(&self.id)
    }
}
impl IntoDataRef for SData {
    fn data_ref(&self) -> SDataRef {
        SDataRef::from(&self.id)
    }
}
impl IntoDataRef for &SData {
    fn data_ref(&self) -> SDataRef {
        SDataRef::from(&self.id)
    }
}
impl IntoDataRef for &str {
    fn data_ref(&self) -> SDataRef {
        SDataRef::from(*self)
    }
}
impl IntoDataRef for String {
    fn data_ref(&self) -> SDataRef {
        SDataRef::from(self)
    }
}
impl IntoDataRef for &String {
    fn data_ref(&self) -> SDataRef {
        SDataRef::from(self.as_str())
    }
}
impl IntoDataRef for Option<SDataRef> {
    fn data_ref(&self) -> SDataRef {
        if let Some(rf) = &self {
            return SDataRef::from(&rf.id);
        }
        SDataRef::default()
    }
}
impl IntoDataRef for &Option<SDataRef> {
    fn data_ref(&self) -> SDataRef {
        if let Some(rf) = &self {
            return SDataRef::from(&rf.id);
        }
        SDataRef::default()
    }
}
impl IntoDataRef for Option<&SDataRef> {
    fn data_ref(&self) -> SDataRef {
        if let Some(rf) = self {
            return SDataRef::from(&rf.id);
        }
        SDataRef::default()
    }
}
impl IntoDataRef for &Option<&SDataRef> {
    fn data_ref(&self) -> SDataRef {
        if let Some(rf) = self {
            return SDataRef::from(&rf.id);
        }
        SDataRef::default()
    }
}
impl From<&str> for SDataRef {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}
impl From<String> for SDataRef {
    fn from(value: String) -> Self {
        Self {
            id: value,
        }
    }
}
impl From<&String> for SDataRef {
    fn from(value: &String) -> Self {
        Self::new(value)
    }
}
impl Display for SDataRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}
impl SDataRef {
    /// New data ref.
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_owned(),
        }
    }

    /// Data.
    pub fn data<'a>(&self, graph: &'a SGraph) -> Option<&'a SData> {
        graph.data.get(&self.id)
    }

    /// Data mut.
    pub fn data_mut<'a>(&self, graph: &'a mut SGraph) -> Option<&'a mut SData> {
        graph.data.get_mut(&self.id)
    }

    /// Exists?
    pub fn exists(&self, graph: &SGraph) -> bool {
        graph.data.contains(&self.id)
    }

    /// Nodes on this data.
    pub fn nodes(&self, graph: &SGraph) -> Vec<SNodeRef> {
        if let Some(data) = self.data(graph) {
            return data.nodes.clone();
        }
        vec![]
    }

    /// Validate value.
    /// Returns whether the data was invalid.
    pub fn validate_val(&self, graph: &mut SGraph) -> bool {
        if let Some(data) = self.data_mut(graph) {
            return data.validate_val();
        }
        false
    }

    /// Set the value of this data.
    pub fn set_value(&self, graph: &mut SGraph, value: impl Serialize) -> bool {
        if let Some(data) = self.data_mut(graph) {
            data.set_value(value);
            return true;
        }
        false
    }

    /// Get the value of this data.
    pub fn get_value<T>(&self, graph: &SGraph) -> Result<T, Error> where T: DeserializeOwned {
        if let Some(data) = self.data(graph) {
            return data.get_value();
        }
        Err(Box::new(ErrorKind::Custom("SDataRef Error: Data not found while attempting to get value".to_owned())))
    }
}
