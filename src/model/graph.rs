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

use std::{any::Any, i32, sync::Arc};
use arcstr::ArcStr;
use bytes::Bytes;
use colored::Colorize;
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use crate::{model::{blob::insert_blob_lib, filesys::fs_library, http::insert_http_lib, libraries::{data::insert_data_lib, function::insert_fn_lib}, list::insert_list_lib, map::insert_map_lib, md::insert_md_lib, num::insert_number_lib, obj::insert_obj_lib, set::insert_set_lib, stof_std::stof_std_lib, string::insert_string_lib, time::insert_time_lib, tup::insert_tup_lib, ver::insert_semver_lib, BytesFormat, Data, DataRef, Format, JsonFormat, LibFunc, MdFormat, Node, NodeRef, SId, SPath, StofData, StofFormat, TextFormat, TomlFormat, YamlFormat, INVALID_NODE_NEW}, parser::context::ParseContext, runtime::{table::SymbolTable, Error, Runtime, Val}};


/// Root node name.
pub const ROOT_NODE_NAME: SId = SId(Bytes::from_static(b"root"));


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Graph.
/// This is the data store for stof.
pub struct Graph {
    pub id: SId,
    pub roots: FxHashSet<NodeRef>,
    pub nodes: FxHashMap<NodeRef, Node>,
    pub data: FxHashMap<DataRef, Data>,
    pub typemap: FxHashMap<String, FxHashSet<NodeRef>>,

    #[serde(skip)]
    pub node_deadpool: FxHashMap<NodeRef, Node>,
    #[serde(skip)]
    pub data_deadpool: FxHashMap<DataRef, Data>,

    #[serde(skip)]
    pub formats: FxHashMap<String, Arc<dyn Format>>,

    #[serde(skip)]
    pub libfuncs: FxHashMap<ArcStr, FxHashMap<String, LibFunc>>,
}
impl Default for Graph {
    fn default() -> Self {
        let mut graph = Self {
            id: Default::default(),
            roots: Default::default(),
            nodes: Default::default(),
            data: Default::default(),
            typemap: Default::default(),
            node_deadpool: Default::default(),
            data_deadpool: Default::default(),
            formats: Default::default(),
            libfuncs: Default::default(),
        };
        graph.load_std_formats();
        graph.insert_std_lib();
        graph
    }
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
    pub fn find_node_named(&self, path: impl Into<SPath>, start: Option<NodeRef>) -> Option<NodeRef> {
        SPath::node(self, path, start)
    }

    #[inline(always)]
    /// Main root.
    /// A root node named "root".
    pub fn main_root(&self) -> Option<NodeRef> {
        self.find_root_named(ROOT_NODE_NAME)
    }

    /// Find a root node with a given name.
    pub fn find_root_named(&self, name: impl Into<SId>) -> Option<NodeRef> {
        let name = name.into();
        for root in &self.roots {
            if let Some(node) = self.nodes.get(root) {
                if node.name == name {
                    return Some(root.clone());
                }
            }
        }
        None
    }

    #[inline]
    /// Ensure main root.
    /// Make sure the main root exists in this graph.
    /// Will create a root node named "root" if not found.
    pub fn ensure_main_root(&mut self) -> NodeRef {
        if let Some(nref) = self.main_root() {
            nref
        } else {
            self.insert_root(ROOT_NODE_NAME)
        }
    }

    
    /*****************************************************************************
     * Types.
     *****************************************************************************/
    
    /// Insert a type by typename.
    pub fn insert_type(&mut self, name: &str, node: &NodeRef) {
        if let Some(types) = self.typemap.get_mut(name) {
            types.insert(node.clone());
        } else {
            let mut types = FxHashSet::default();
            types.insert(node.clone());
            self.typemap.insert(name.to_string(), types);
        }
    }

    /// Remove an object from types.
    pub fn remove_type(&mut self, node: &NodeRef) {
        let mut to_remove = Vec::new();
        for (id, types) in &mut self.typemap {
            if types.remove(node) && types.is_empty() {
                to_remove.push(id.clone());
            }
        }
        for id in to_remove {
            self.typemap.remove(&id);
        }
    }

    /// Find a type by name, resolving to the closest to the context if collisions.
    pub fn find_type(&self, name: &str, context: Option<NodeRef>) -> Option<NodeRef> {
        if let Some(types) = self.typemap.get(name) {
            if types.len() == 1 || context.is_none() {
                for ty in types.iter() { return Some(ty.clone()); }
            } else if types.len() > 1 {
                let context = context.unwrap();
                let mut best = None;
                let mut closest = i32::MAX;
                for ty in types.iter() {
                    let dist = context.distance_to(self, ty);
                    if dist >= 0 && dist < closest {
                        closest = dist;
                        best = Some(ty.clone());
                    }
                }
                return best;
            }
        }
        None
    }


    /*****************************************************************************
     * Library Functions.
     *****************************************************************************/
    
    /// Insert standard library functions.
    pub fn insert_std_lib(&mut self) {
        // Std libs
        stof_std_lib(self);
        insert_number_lib(self);
        insert_string_lib(self);
        insert_semver_lib(self);
        insert_blob_lib(self);
        insert_fn_lib(self);
        insert_obj_lib(self);
        insert_data_lib(self);
        insert_list_lib(self);
        insert_set_lib(self);
        insert_map_lib(self);
        insert_tup_lib(self);

        // Markdown lib
        insert_md_lib(self);
        
        // System libs
        insert_time_lib(self);
        fs_library(self);
        insert_http_lib(self);
    }
    
    /// Insert a library function to this graph.
    /// Will replace one with the same name and lib if it already exists.
    pub fn insert_libfunc(&mut self, libfunc: LibFunc) {
        if let Some(lib) = self.libfuncs.get_mut(&libfunc.library) {
            lib.insert(libfunc.name.clone(), libfunc);
        } else {
            let lib = libfunc.library.clone();
            let mut funcs = FxHashMap::default();
            funcs.insert(libfunc.name.clone(), libfunc);
            self.libfuncs.insert(lib, funcs);
        }
    }

    /// Get a library function.
    pub fn libfunc(&self, library: &ArcStr, name: &str) -> Option<LibFunc> {
        if let Some(lib) = self.libfuncs.get(library) {
            if let Some(func) = lib.get(name) {
                return Some(func.clone());
            }
        }
        None
    }

    #[inline]
    /// Remove a library function.
    pub fn remove_libfunc(&mut self, library: &ArcStr, name: &str) -> Option<LibFunc> {
        if let Some(lib) = self.libfuncs.get_mut(library) {
            lib.remove(name)
        } else {
            None
        }
    }

    #[inline]
    /// Remove a library.
    pub fn remove_lib(&mut self, library: &ArcStr) -> Option<FxHashMap<String, LibFunc>> {
        self.libfuncs.remove(library)
    }


    /*****************************************************************************
     * Nodes.
     *****************************************************************************/
    
    /// Insert a root node directly.
    pub fn insert_root(&mut self, name: impl Into<SId>) -> NodeRef {
        let mut node = Node::new(name.into(), SId::default(), false);
        node.invalidate(INVALID_NODE_NEW);

        let nref = node.id.clone();
        self.nodes.insert(node.id.clone(), node);
        self.roots.insert(nref.clone());
        nref
    }

    #[inline(always)]
    /// Insert a child node directly.
    pub fn insert_child(&mut self, name: impl Into<SId>, parent: impl Into<NodeRef>, field: bool) -> NodeRef {
        self.insert_node(name, Some(parent.into()), field)
    }
    
    /// Insert a node.
    /// If a parent is not provided, the behavior is the same as insert root.
    pub fn insert_node(&mut self, name: impl Into<SId>, parent: Option<NodeRef>, field: bool) -> NodeRef {
        let node;
        if field && parent.is_some() {
            if let Some(nref) = &parent {
                if !nref.node_exists(&self) {
                    node = Node::new(name.into(), SId::default(), false);
                } else {
                    node = Node::new(name.into(), SId::default(), true);
                }
            } else {
                unreachable!();
            }
        } else {
            node = Node::new(name.into(), SId::default(), false);
        }
        self.insert_stof_node(node, parent)
    }

    /// Insert a node with an ID.
    pub fn insert_node_id(&mut self, name: impl Into<SId>, id: impl Into<SId>, parent: Option<NodeRef>, field: bool) -> NodeRef {
        let node;
        if field && parent.is_some() {
            if let Some(nref) = &parent {
                if !nref.node_exists(&self) {
                    node = Node::new(name.into(), id.into(), false);
                } else {
                    node = Node::new(name.into(), id.into(), true);
                }
            } else {
                unreachable!();
            }
        } else {
            node = Node::new(name.into(), id.into(), false);
        }
        self.insert_stof_node(node, parent)
    }

    /// Insert stof node.
    /// Don't call this with nodes that already exist in the graph (have a valid ID already).
    pub fn insert_stof_node(&mut self, mut node: Node, parent: Option<NodeRef>) -> NodeRef {
        if let Some(parent) = &parent {
            if parent.node_exists(&self) {
                node.parent = Some(parent.clone());
                node.invalidate_parent();
            } else {
                if node.parent.is_some() { node.invalidate_parent(); }
                node.parent = None;
            }
        } else {
            if node.parent.is_some() { node.invalidate_parent(); }
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
    /// Param fields - if creating a new object to match the path, should it be a field (only applies to nodes that don't exist yet)?
    pub fn ensure_named_nodes(&mut self, path: impl Into<SPath>, start: Option<NodeRef>, fields: bool, custom_insert: Option<fn(&mut Self, &SId, Option<NodeRef>)->NodeRef>) -> Option<NodeRef> {
        let path: SPath = path.into();
        if path.path.is_empty() { return None; }
        if !path.names {
            return None;
        }

        let mut current = start;
        for segment in path.path {
            if let Some(node) = SPath::node(&self, (segment.clone(), true), current.clone()) {
                current = Some(node);
            } else {
                if let Some(custom) = &custom_insert {
                    current = Some(custom(self, &segment, current));
                } else {
                    current = Some(self.insert_node(&segment, current, fields));
                }
            }
        }
        current
    }

    /// Remove a node from the graph.
    /// May or may not remove data completely, depending on where the data is referenced.
    /// Note: if you pass gc and also are managing a symbol table, you have to do gc on that table as well.
    pub fn remove_node(&mut self, nref: &NodeRef, gc: bool) -> bool {
        if let Some(node) = self.nodes.remove(nref) {
            // Remove all data on this node
            for (_, dref) in &node.data {
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
                self.remove_node(child, gc);
            }

            // Remove all data in this graph that has a hard reference to this node
            // Kind of expensive, so only do this when you know fields reference this node (data has hard reference to this node)
            if gc {
                let mut to_remove = Vec::new();
                for (id, data) in &self.data {
                    if data.data.hard_node_ref(&node.id) {
                        to_remove.push(id.clone());
                    }
                }
                for id in to_remove {
                    // Have to take the long way here as this data might have other valid nodes that reference it
                    self.remove_data(&id, None);
                }
            }

            // Insert into the deadpool and remove types
            self.remove_type(&node.id);
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

    /// Move a node to another node.
    /// Since this is a DAG, destination cannot be a descendant of the source (branch loss) - this function checks for this.
    pub fn move_node(&mut self, source: &NodeRef, dest: &NodeRef) -> bool {
        if !source.node_exists(&self) || !dest.node_exists(&self) || dest.child_of(&self, source) {
            return false;
        }

        // Add source as a child of dest
        if let Some(dest) = dest.node_mut(self) {
            dest.add_child(source.clone());
        }

        // Change parent on the source to the new destination
        let mut old_parent = None;
        if let Some(node) = source.node_mut(self) {
            old_parent = node.parent.clone();
            node.parent = Some(dest.clone());
            node.invalidate_parent();
        }

        // Remove the source from the old parent if any
        if let Some(old) = old_parent {
            if let Some(old) = old.node_mut(self) {
                old.remove_child(source);
            }
        } else {
            // Remove from the roots of the graph if no parent
            self.roots.remove(source);
        }

        true
    }

    /// Absorb the data and optionally, the children of an external node within another graph.
    pub fn absorb_external_node(&mut self, other: &Self, node: &Node, on: &NodeRef, children_too: bool) {
        for (_, dref) in &node.data {
            if let Some(data) = dref.data(other) {
                let mut data_clone = data.clone();
                data_clone.invalidate_nodes();

                // Make sure to only bring over the nodes that exist on this graph
                let mut new_nodes = FxHashSet::default();
                for nref in &data.nodes {
                    if nref.node_exists(&self) {
                        new_nodes.insert(nref.clone());
                    }
                }
                data_clone.nodes = new_nodes;
                
                self.insert_data(on, data_clone);
            }
        }
        if children_too {
            for nref in &node.children {
                if let Some(child) = nref.node(other) {
                    self.insert_external_node(other, child, Some(on.clone()), None, None);
                }
            }
        }
    }
    
    /// Insert an external node (cloned), contained within another graph.
    pub fn insert_external_node(&mut self, other: &Self, node: &Node, parent: Option<NodeRef>, rename: Option<SId>, field: Option<bool>) -> bool {
        if let Some(parent) = &parent {
            if !parent.node_exists(&self) {
                return false; // specified a parent that doesn't exist (instead of creating a root)
            }
        }
        
        // Clone the node, rename, insert, and insert all children
        // All nodes will be inserted before data gets inserted, for contains checks
        let mut cloned = node.clone();
        if let Some(new_name) = rename {
            cloned.name = new_name;
        }
        if let Some(field) = field {
            if field {
                cloned.make_field();
            } else {
                cloned.not_field();
            }
        }
        let inserted = self.insert_stof_node(cloned, parent);
        for nref in &node.children {
            if let Some(child) = nref.node(other) {
                self.insert_external_node(other, child, Some(inserted.clone()), None, field);
            }
        }

        // Add all data from node to the inserted node
        for (_, dref) in &node.data {
            if let Some(data) = dref.data(other) {
                let mut data_clone = data.clone();
                data_clone.invalidate_nodes();

                // Make sure to only bring over the nodes that exist on this graph
                let mut new_nodes = FxHashSet::default();
                for nref in &data.nodes {
                    if nref.node_exists(&self) {
                        new_nodes.insert(nref.clone());
                    }
                }
                data_clone.nodes = new_nodes;
                
                self.insert_data(&inserted, data_clone);
            } else if let Some(ins) = inserted.node_mut(self) {
                ins.remove_data(dref);
            }
        }

        true
    }


    /*****************************************************************************
     * Data.
     *****************************************************************************/
    
    /// Insert data into the graph and onto a node.
    /// Data in the graph must be associated with a node.
    /// Will overwrite data with the same ID if already in the graph.
    pub fn insert_data(&mut self, node: &NodeRef, mut data: Data) -> Option<DataRef> {
        let mut res = None;
        let mut replaced = None;
        if let Some(node) = node.node_mut(self) {
            let dref = data.id.clone();
            if let Some(old) = node.add_data(data.name.to_string(), dref.clone()) {
                if old != dref {
                    replaced = Some(old);
                }
            }

            data.node_added(node.id.clone());
            self.data.insert(dref.clone(), data);
            res = Some(dref);
        }
        if let Some(old) = replaced {
            let mut remove_all = false;
            if let Some(data) = old.data_mut(self) {
                if data.node_removed(&node) {
                    remove_all = data.ref_count() < 1;
                }
            }
            if remove_all {
                if let Some(data) = self.data.remove(&old) {
                    self.data_deadpool.insert(data.id.clone(), data);
                }
            }
        }
        res
    }

    /// Attach an existing data ref to a node in this graph.
    pub fn attach_data(&mut self, node: &NodeRef, data: &DataRef) -> bool {
        if !node.node_exists(&self) { return false; }

        let name;
        if let Some(data) = data.data_mut(self) {
            name = data.name.clone();
            data.node_added(node.clone());
        } else {
            return false;
        }

        let mut replaced = None;
        if let Some(node) = node.node_mut(self) {
            if let Some(old) = node.add_data(name.to_string(), data.clone()) {
                if &old != data {
                    replaced = Some(old);
                }
            }
        }
        if let Some(old) = replaced {
            let mut remove_all = false;
            if let Some(data) = old.data_mut(self) {
                if data.node_removed(&node) {
                    remove_all = data.ref_count() < 1;
                }
            }
            if remove_all {
                if let Some(data) = self.data.remove(&old) {
                    self.data_deadpool.insert(data.id.clone(), data);
                }
            }
        }
        true
    }

    /// Remove data from this graph.
    /// If given a node to remove from, the data will only be removed from that node.
    /// Otherwise, it will be removed from the entire graph.
    /// If a node is given and it is the only one referencing the data, the data will be removed completely.
    pub fn remove_data(&mut self, data: &DataRef, node: Option<NodeRef>) -> bool {
        let mut remove_all = true;
        let mut res = false;

        if let Some(node) = node {
            remove_all = false;
            if let Some(node) = node.node_mut(self) {
                remove_all = node.remove_data(data);
            }
            if remove_all {
                res = true;
                remove_all = false;
                if let Some(data) = data.data_mut(self) {
                    if data.node_removed(&node) {
                        remove_all = data.ref_count() < 1;
                    }
                }
            }
        }

        if remove_all {
            // remove from all nodes that reference this data
            let mut nodes = Default::default();
            if let Some(data) = data.data(&self) {
                nodes = data.nodes.clone();
            }
            for node in nodes {
                if let Some(node) = node.node_mut(self) {
                    node.remove_data(data);
                }
            }

            if let Some(data) = self.data.remove(data) {
                res = true;
                self.data_deadpool.insert(data.id.clone(), data);
            } else {
                res = false;
            }
        }
        res
    }

    /// Rename data.
    /// Make sure anytime you change the name of data, it's through this function.
    /// Will change the name of the data, but also all of the names in each node (for fast search by name).
    pub fn rename_data(&mut self, data: &DataRef, name: impl Into<SId>) -> bool {
        let new_name: SId = name.into();
        let mut old_name = ROOT_NODE_NAME;
        let mut nodes = FxHashSet::default();
        if let Some(data) = data.data_mut(self) {
            old_name = data.name.clone();
            if !data.set_name(new_name.clone()) {
                return false;
            }
            nodes = data.nodes.clone();
        }
        for nref in nodes {
            let mut replaced = None;
            if let Some(node) = nref.node_mut(self) {
                node.data.remove(old_name.as_ref());
                if let Some(old) = node.add_data(new_name.to_string(), data.clone()) {
                    if &old != data {
                        replaced = Some(old);
                    }
                }
            }
            if let Some(old) = replaced {
                let mut remove_all = false;
                if let Some(data) = old.data_mut(self) {
                    if data.node_removed(&nref) {
                        remove_all = data.ref_count() < 1;
                    }
                }
                if remove_all {
                    if let Some(data) = self.data.remove(&old) {
                        self.data_deadpool.insert(data.id.clone(), data);
                    }
                }
            }
        }
        true
    }

    /// Insert Stof data.
    /// Will create a Data wrapper (optionally provide ID/ref).
    /// Name needs to be unique for the node. For an anonymous option, create an ID and use it for both the name and ID.
    pub fn insert_stof_data(&mut self, node: &NodeRef, name: impl Into<SId>, stof_data: Box<dyn StofData>, id: Option<DataRef>) -> Option<DataRef> {
        let mut rf = DataRef::default();
        if let Some(aid) = id {
            rf = aid;
        }
        let data = Data::new(rf, name.into(), stof_data);
        self.insert_data(node, data)
    }

    #[inline]
    /// Set Stof data.
    pub fn set_stof_data(&mut self, data: &DataRef, stof_data: Box<dyn StofData>) -> bool {
        if let Some(data) = data.data_mut(self) {
            data.set(stof_data);
            true
        } else {
            false
        }
    }

    #[inline]
    /// Get Stof data.
    pub fn get_stof_data<T: Any>(&self, data: &DataRef) -> Option<&T> {
        if let Some(data) = data.data(self) {
            data.get::<T>()
        } else {
            None
        }
    }

    #[inline]
    /// Get mutable Stof data.
    pub fn get_mut_stof_data<T: Any>(&mut self, data: &DataRef) -> Option<&mut T> {
        if let Some(data) = data.data_mut(self) {
            data.get_mut::<T>()
        } else {
            None
        }
    }


    /*****************************************************************************
     * Dump this graph (debugging).
     *****************************************************************************/
    
    /// Dump this graph for debugging.
    pub fn dump(&self, data: bool) {
        println!("Dump Graph: {}", &self.id.as_ref().red());
        for root_ref in &self.roots {
            if let Some(root) = root_ref.node(self) {
                println!("{}", root.dump(self, 0, data));
            }
        }
        println!("End Dump");
    }


    /*****************************************************************************
     * Flush & Validate.
     *****************************************************************************/
    
    /// Flush node deadpool.
    /// These are nodes that have been removed from this graph.
    /// This empties the deadpool and returns all removed nodes as a completely detached vector.
    pub fn flush_node_deadpool(&mut self) -> Vec<Node> {
        let mut nodes = Vec::with_capacity(self.node_deadpool.len());
        for (_, node) in self.node_deadpool.drain() { nodes.push(node); }
        nodes
    }

    /// Clear the node deadpool.
    /// This is a flush, but without the vector allocation.
    pub fn clear_node_deadpool(&mut self) {
        self.node_deadpool.clear();
    }

    /// Flush data deadpool.
    /// These are the individual data elements that have been removed from this graph.
    /// This empties the deadpool and returns all removed data as a completely detached vector.
    pub fn flush_data_deadpool(&mut self) -> Vec<Data> {
        let mut datas = Vec::with_capacity(self.data_deadpool.len());
        for (_, data) in self.data_deadpool.drain() { datas.push(data); }
        datas
    }

    /// Clear data deadpool.
    /// This is a flush, but without the vector allocation.
    pub fn clear_data_deadpool(&mut self) {
        self.data_deadpool.clear();
    }

    /// Collects dirty nodes for validation as a group.
    /// Optionally provide a set of symbols to filter "how" that node is dirty.
    pub fn dirty_nodes(&self, symbols: Option<FxHashSet<SId>>) -> FxHashSet<NodeRef> {
        let mut dirty = FxHashSet::default();
        for (_, node) in &self.nodes {
            if node.any_dirty() {
                if let Some(sym) = &symbols {
                    for sy in sym {
                        if node.dirty(sy) {
                            dirty.insert(node.id.clone());
                            break;
                        }
                    }
                } else {
                    dirty.insert(node.id.clone());
                }
            }
        }
        dirty
    }

    /// Collects dirty data for validation as a group.
    /// Optionally provide a set of symbols to filter "how" that data is dirty.
    pub fn dirty_data(&self, symbols: Option<FxHashSet<SId>>) -> FxHashSet<NodeRef> {
        let mut dirty = FxHashSet::default();
        for (_, data) in &self.data {
            if data.any_dirty() {
                if let Some(sym) = &symbols {
                    for sy in sym {
                        if data.dirty(sy) {
                            dirty.insert(data.id.clone());
                            break;
                        }
                    }
                } else {
                    dirty.insert(data.id.clone());
                }
            }
        }
        dirty
    }

    /// Flush this graph.
    /// This operation clears both deadpools, validates all nodes, and validates all data.
    pub fn flush(&mut self) {
        self.clear_node_deadpool();
        self.clear_data_deadpool();
        for nref in self.dirty_nodes(None) {
            if let Some(node) = nref.node_mut(self) {
                node.validate_clear();
            }
        }
        for dref in self.dirty_data(None) {
            if let Some(data) = dref.data_mut(self) {
                data.validate_clear();
            }
        }
    }

    /// Flush join.
    /// Joins another graph with this one via flushed nodes and data.
    pub fn flush_join(&mut self, other: &Self, gc_removed_nodes: bool) {
        // Delete nodes that have been deleted in other first
        for (id, _) in &other.node_deadpool {
            self.remove_node(id, gc_removed_nodes);
        }

        // Delete data that has been deleted in other next
        for (id, _) in &other.data_deadpool {
            self.remove_data(id, None);
        }

        // Update nodes in this graph that have been modified or inserted in other
        for nref in other.dirty_nodes(None) {
            if let Some(changed_node) = nref.node(other) {
                if let Some(existing) = nref.node_mut(self) {
                    if changed_node.name != ROOT_NODE_NAME {
                        existing.name = changed_node.name.clone();
                    }
                    existing.parent = changed_node.parent.clone();
                    existing.children = changed_node.children.clone();
                    existing.data = changed_node.data.clone();
                    existing.attributes = changed_node.attributes.clone();
                } else {
                    self.nodes.insert(changed_node.id.clone(), changed_node.clone());
                }
            }
        }

        // Update data in this graph that have been modified or inserted in other
        for dref in other.dirty_data(None) {
            if let Some(changed_data) = dref.data(other) {
                if let Some(existing) = dref.data_mut(self) {
                    existing.name = changed_data.name.clone();
                    existing.nodes = changed_data.nodes.clone();
                    existing.data = changed_data.data.clone();
                } else {
                    self.data.insert(changed_data.id.clone(), changed_data.clone());
                }
            }
        }
    }

    /// Clone this graph with a given context.
    pub fn context_clone(&self, context: FxHashSet<NodeRef>) -> Self {
        let mut clone = Self::default();
        
        // Get a high-level snapshot of the nodes to add
        // This removes any children from within the context, because insert external adds child nodes
        let mut snapshot = FxHashSet::default();
        for a in &context {
            let mut is_child = false;
            for b in &context {
                if a != b && a.child_of(self, b) {
                    is_child = true;
                    break;
                }
            }
            if !is_child {
                snapshot.insert(a);
            }
        }

        // Add nodes from self that meet the snapshot into the cloned graph
        for nref in &snapshot {
            if let Some(node) = nref.node(self) {
                let mut parent = None;
                if let Some(prnt) = &node.parent {
                    if snapshot.contains(prnt) {
                        parent = Some(prnt.clone());
                    }
                }
                clone.insert_external_node(self, node, parent, None, None);
            }
        }

        // Make sure the main root has the name "root"
        let mut found_root = false;
        let mut new_roots = FxHashSet::default();
        let mut stof_node = None;
        for nref in &clone.roots {
            if let Some(node) = nref.node(&clone) {
                if node.name == ROOT_NODE_NAME {
                    new_roots.insert(nref.clone());
                    found_root = true;
                } else if node.name.as_ref() == "__stof__" {
                    stof_node = Some(nref.clone());
                } else {
                    new_roots.insert(nref.clone());
                }
            }
        }
        if !found_root {
            for rt in &new_roots {
                rt.rename_node(&mut clone, ROOT_NODE_NAME);
                break;
            }
        }
        clone.roots = new_roots;
        
        if let Some(srt) = stof_node {
            clone.roots.insert(srt);
        }

        clone
    }

    /// Garbage collect for the graph and table, optionally flushing the deadpools.
    /// Remove variables in the symbol table that no longer reference a valid object or data
    /// Remove data that has any hard references to a dead node
    ///
    /// Ex. gc_table(table, true) - will clear the deadpools
    pub fn gc_table(&mut self, table: &mut SymbolTable, flush_deadpools: bool) {
        // For each node that has been removed, remove all data that has a
        // hard reference to those nodes.
        if !self.node_deadpool.is_empty() {
            let mut to_remove = Vec::new(); // data IDs that have hard refs to dead nodes
            for (dref, data) in &self.data {
                for (dnref, _) in &self.node_deadpool {
                    if data.data.hard_node_ref(dnref) {
                        to_remove.push(dref.clone());
                    }
                }
            }
            for id in to_remove {
                // Have to take the long way here as other valid nodes might be referencing this data
                self.remove_data(&id, None);
            }

            for (dnref, _) in &self.node_deadpool {
                table.gc_node(dnref);
            }
        }

        // For each data that has been removed, remove all references in the table
        for (did, _data) in &self.data_deadpool {
            table.gc_data(did);
        }

        if flush_deadpools {
            self.clear_node_deadpool();
            self.clear_data_deadpool();
        }
    }


    /*****************************************************************************
     * Formats.
     *****************************************************************************/
    
    /// Load standard (included) formats.
    pub fn load_std_formats(&mut self) {
        self.load_format(Arc::new(StofFormat{}));
        self.load_format(Arc::new(JsonFormat{}));
        self.load_format(Arc::new(TomlFormat{}));
        self.load_format(Arc::new(YamlFormat{}));
        self.load_format(Arc::new(TextFormat{}));
        self.load_format(Arc::new(MdFormat{}));
        self.load_format(Arc::new(BytesFormat{}));
    }
    
    /// Load a format.
    pub fn load_format(&mut self, format: Arc<dyn Format>) {
        for id in format.identifiers() {
            self.formats.insert(id, format.clone());
        }
    }

    #[inline(always)]
    /// Get a format.
    pub fn get_format(&self, id: &str) -> Option<Arc<dyn Format>> {
        self.formats.get(id).cloned()
    }

    /// Get a format by content type.
    pub fn get_format_by_content_type(&self, id: &str) -> Option<Arc<dyn Format>> {
        for (_, fmt) in &self.formats {
            if fmt.content_type() == id {
                return Some(fmt.clone());
            }
        }
        None
    }

    /// Remove this format completely, even if it has other identifiers.
    /// Returns true if the format was found and removed completely.
    pub fn remove_format(&mut self, id: &str) -> bool {
        if let Some(format) = self.formats.remove(id) {
            for id in format.identifiers() {
                self.formats.remove(&id);
            }
            true
        } else {
            false
        }
    }

    /// Remove a single format ID.
    /// Returns true if the format was completely removed in all of its identifiers.
    pub fn remove_format_id(&mut self, id: &str) -> bool {
        if let Some(format) = self.formats.remove(id) {
            for id in format.identifiers() {
                if self.formats.contains_key(&id) {
                    return false;
                }
            }
            return true;
        }
        false
    }

    /// Available format identifiers.
    pub fn available_formats(&self) -> FxHashSet<String> {
        let mut formats = FxHashSet::default();
        for (id, _) in &self.formats {
            formats.insert(id.clone());
        }
        formats
    }

    /// Binary import into this graph, using a loaded format.
    pub fn binary_import(&mut self, format: &str, bytes: Bytes, node: Option<NodeRef>) -> Result<(), Error> {
        let id = format;
        if let Some(format) = self.get_format(id) {
            format.binary_import(self, id, bytes, node)
        } else if let Some(format) = self.get_format_by_content_type(id) {
            format.binary_import(self, id, bytes, node)
        } else if let Some(format) = self.get_format("bytes") {
            format.binary_import(self, id, bytes, node)
        } else {
            Err(Error::GraphFormatNotFound)
        }
    }

    /// Import a string into this graph, using a loaded format.
    pub fn string_import(&mut self, format: &str, src: &str, node: Option<NodeRef>) -> Result<(), Error> {
        let id = format;
        if let Some(format) = self.get_format(id) {
            format.string_import(self, id, src, node)
        } else if let Some(format) = self.get_format_by_content_type(id) {
            format.string_import(self, id, src, node)
        } else if let Some(format) = self.get_format("text") {
            format.string_import(self, id, src, node)
        } else {
            Err(Error::GraphFormatNotFound)
        }
    }

    /// File import into this graph, using a loaded format.
    pub fn file_import(&mut self, format: &str, path: &str, node: Option<NodeRef>) -> Result<(), Error> {
        let id = format;
        if let Some(format) = self.get_format(id) {
            format.file_import(self, id, path, node)
        } else {
            Err(Error::GraphFormatNotFound)
        }
    }

    /// String export.
    pub fn string_export(&self, format: &str, node: Option<NodeRef>) -> Result<String, Error> {
        let id = format;
        if let Some(format) = self.get_format(id) {
            format.string_export(self, id, node)
        } else if let Some(format) = self.get_format_by_content_type(id) {
            format.string_export(self, id, node)
        } else {
            Err(Error::GraphFormatNotFound)
        }
    }

    /// Binary export.
    pub fn binary_export(&self, format: &str, node: Option<NodeRef>) -> Result<Bytes, Error> {
        let id = format;
        if let Some(format) = self.get_format(id) {
            format.binary_export(self, id, node)
        } else if let Some(format) = self.get_format_by_content_type(id) {
            format.binary_export(self, id, node)
        } else {
            Err(Error::GraphFormatNotFound)
        }
    }


    /*****************************************************************************
     * Stof Language.
     *****************************************************************************/
    
    #[inline]
    /// Parse stof into this graph, optionally into a specific node.
    /// Use file_import for files...
    pub fn parse_stof_src(&mut self, stof: &str, node: Option<NodeRef>) -> Result<(), Error> {
        // stof format creates a new context
        self.string_import("stof", stof, node)
    }

    /// Parse a stof file into this graph, optionally into a specific node.
    /// This serves as an entrypoint for parsing Stof into a graph.
    pub fn parse_stof_file(&mut self, format: &str, path: &str, node: Option<NodeRef>, docs: bool) -> Result<(), Error> {
        let mut context = ParseContext::new(self);
        context.docs = docs;
        context.parse_from_file(format, path, node)
    }

    /// Call a function (by named '.' separated path) in this graph.
    pub fn call(&mut self, func_path: &str, start: Option<NodeRef>, args: Vec<Val>) -> Result<Val, Error> {
        let search;
        if let Some(start) = start {
            if let Some(nodepath) = start.node_path(&self, true) {
                search = format!("{}.{}", nodepath.join("."), func_path);
            } else {
                return Err(Error::Custom("Graph call start node doesn't exist".into()));
            }
        } else if func_path.split('.').collect::<Vec<_>>().len() < 2 {
            search = format!("root.{func_path}");
        } else {
            search = func_path.to_string();
        }

        Runtime::call(self, &search, args)
    }

    #[inline]
    /// Test this graph, calling all #[test] functions, optionally resulting in an Err or always Ok.
    pub fn test(&mut self, context: Option<String>, throw: bool) -> Result<String, String> {
        Runtime::test(self, context, throw)
    }
}


#[cfg(test)]
mod tests {
    use crate::model::{Data, Graph, SPath, StofData, ROOT_NODE_NAME};

    #[test]
    fn new_with_id() {
        let graph = Graph::new("hello");
        assert_eq!(graph.id.as_ref(), "hello");
    }

    #[test]
    fn default_graph() {
        let graph = Graph::default();
        assert_eq!(graph.id.len(), 20);
        assert_eq!(graph.roots.len(), 0);
        assert_eq!(graph.nodes.len(), 0);
        assert_eq!(graph.data.len(), 0);
    }

    #[test]
    fn ensure_main_root() {
        let mut graph = Graph::default();
        graph.ensure_main_root();

        assert!(graph.main_root().is_some());
        assert_eq!(graph.roots.len(), 1);
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.data.len(), 0);
        assert!(graph.find_root_named(ROOT_NODE_NAME).is_some());
    }

    #[test]
    fn insert_node_as_root() {
        let mut graph = Graph::default();
        let nref = graph.insert_node("root", None, false);

        assert!(nref.node_exists(&graph));
        assert_eq!(graph.roots.len(), 1);
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.data.len(), 0);
        assert!(graph.find_node_named("root", None).is_some());
    }

    #[test]
    fn paths() {
        let mut graph = Graph::default();
        let root = graph.ensure_main_root();
        let base;
        let top;

        let self_test;
        let super_test;
        {
            base = graph.insert_node("base", Some(root.clone()), false);
            {
                graph.insert_child("a", &base, false);
                graph.insert_child("b", &base, false);
            }
            top = graph.insert_child("top", &root, false);
            {
                graph.insert_child("a", &top, false);
                graph.insert_child("b", &top, false);

                self_test = graph.insert_child("self", &top, false);
                super_test = graph.insert_child("super", &top, false);
            }
        }
        assert_eq!(graph.find_node_named("root.base", None).unwrap(), base);
        assert_eq!(graph.find_node_named("root.base", Some(root.clone())).unwrap(), base);

        assert_eq!(graph.find_node_named("base", Some(root.clone())).unwrap(), base);
        assert_eq!(graph.find_node_named("base", Some(base.clone())).unwrap(), base);
        assert_eq!(graph.find_node_named("self", Some(base.clone())).unwrap(), base);
        assert_eq!(graph.find_node_named("super", Some(base.clone())).unwrap(), root);

        assert_eq!(graph.find_node_named("self.self", Some(top.clone())).unwrap(), self_test);
        assert_eq!(graph.find_node_named("super.super", Some(top.clone())).unwrap(), top);
        assert_eq!(graph.find_node_named("super", Some(top.clone())).unwrap(), super_test);
    }

    #[test]
    fn create_named_path() {
        let mut graph = Graph::default();
        let a = graph.ensure_named_nodes("root.base.a", None, false, None).unwrap();
        assert_eq!(a.node_name(&graph).unwrap().as_ref(), "a");

        let b = graph.ensure_named_nodes("root.base.b", None, false, None).unwrap();
        assert_eq!(b.node_name(&graph).unwrap().as_ref(), "b");

        assert!(graph.main_root().is_some());
        assert_eq!(graph.nodes.len(), 4);
        assert_eq!(graph.roots.len(), 1);
        assert_eq!(graph.data.len(), 0);
    }

    #[test]
    fn remove_node() {
        let mut graph = Graph::default();
        graph.ensure_named_nodes(SPath::from("root.base.a"), None, false, None);
        graph.ensure_named_nodes(SPath::from("root.base.b"), None, false, None);
        graph.ensure_named_nodes(SPath::from("root.top.a"), None, false, None);
        graph.ensure_named_nodes(SPath::from("root.top.b"), None, false, None);

        assert_eq!(graph.nodes.len(), 7);
        assert_eq!(graph.roots.len(), 1);

        let base = graph.find_node_named("root.base", None).unwrap();
        graph.remove_node(&base, false);

        assert!(!base.node_exists(&graph));
        assert_eq!(graph.nodes.len(), 4);
        assert_eq!(graph.roots.len(), 1);
        assert_eq!(graph.node_deadpool.len(), 3);

        let top = graph.find_node_named("top", None).unwrap();
        assert!(top.node_exists(&graph));

        assert_eq!(graph.all_child_nodes(&graph.main_root().unwrap(), true).len(), 4);
    }

    #[test]
    fn move_node_up() {
        let mut graph = Graph::default();
        graph.ensure_named_nodes(SPath::from("root.base.a"), None, false, None);
        graph.ensure_named_nodes(SPath::from("root.base.b"), None, false, None);
        graph.ensure_named_nodes(SPath::from("root.top.a"), None, false, None);
        graph.ensure_named_nodes(SPath::from("root.top.b"), None, false, None);

        let b = graph.find_node_named("root.base.b", None).unwrap();
        let root = graph.ensure_main_root();
        assert!(graph.move_node(&b, &root));
        assert_eq!(b.node_path(&graph, true).unwrap().join("."), "root.b");

        assert_eq!(root.node(&graph).unwrap().children.len(), 3);
        assert_eq!(b.node_parent(&graph).unwrap(), root);
    }

    #[test]
    fn insert_external() {
        let mut graph = Graph::default();
        graph.ensure_named_nodes("Hello.dude.another.Hi", None, false, None);

        let mut other = Graph::default();
        other.ensure_named_nodes("Dude.dude.created", None, false, None);
        let external = other.find_node_named("Dude.dude", None).unwrap();
        graph.insert_external_node(&other, external.node(&other).unwrap(), None, Some(ROOT_NODE_NAME), None);
    
        assert!(graph.find_node_named("root.created", None).is_some());
        assert_eq!(graph.nodes.len(), 6);
        assert_eq!(graph.roots.len(), 2);
    }

    #[test]
    fn insert_attach_data() {
        let mut graph = Graph::default();
        let root = graph.ensure_main_root();
        let child = graph.insert_child("child", &root, false);

        let dref = graph.insert_data(&root, Data::from(Box::new("value".to_owned()) as Box<dyn StofData>)).unwrap();
        assert!(graph.attach_data(&child, &dref));

        assert_eq!(dref.data_nodes(&graph).len(), 2);
        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.data.len(), 1);
        assert_eq!(graph.roots.len(), 1);
        assert_eq!(graph.get_stof_data::<String>(&dref).unwrap(), "value");
        assert_eq!(root.node(&graph).unwrap().data.len(), 1);
        assert_eq!(child.node(&graph).unwrap().data.len(), 1);

        graph.remove_data(&dref, Some(child.clone()));
        assert_eq!(dref.data_nodes(&graph).len(), 1);
        assert_eq!(root.node(&graph).unwrap().data.len(), 1);
        assert_eq!(child.node(&graph).unwrap().data.len(), 0);

        graph.remove_data(&dref, Some(root.clone()));
        assert!(!dref.data_exists(&graph));
        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.data.len(), 0);
        assert_eq!(graph.data_deadpool.len(), 1);
        assert_eq!(root.node(&graph).unwrap().data.len(), 0);
        assert_eq!(child.node(&graph).unwrap().data.len(), 0);
    }

    #[test]
    fn named_data() {
        let mut graph = Graph::default();
        let root = graph.ensure_main_root();

        let dref = graph.insert_stof_data(&root, "test", Box::new("value".to_owned()), None).unwrap();
        assert_eq!(dref.data_name(&graph).unwrap().as_ref(), "test");
        assert_eq!(dref.tagname(&graph).unwrap(), "_String");

        assert_eq!(root.node_data_named(&graph, "test").unwrap(), &dref);
        assert_eq!(graph.data.len(), 1);
        assert_eq!(graph.get_stof_data::<String>(&dref).unwrap(), "value");

        assert!(graph.rename_data(&dref, "renamed"));
        assert!(root.node_data_named(&graph, "test").is_none());
        assert_eq!(root.node_data_named(&graph, "renamed").unwrap(), &dref);
        assert_eq!(dref.data_name(&graph).unwrap().as_ref(), "renamed");
    }
}
