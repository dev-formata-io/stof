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

use std::collections::BTreeSet;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use crate::{SField, SFunc, FKIND, FUNC_KIND};
use super::{IntoDataRef, IntoNodeRef, SDataRef, SDataSelection, SGraph, SNodeRef};


/// Stof node.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SNode {
    pub id: String,
    pub name: String,
    pub parent: Option<SNodeRef>,
    pub children: BTreeSet<SNodeRef>,
    pub data: BTreeSet<SDataRef>,
    
    #[serde(skip)]
    pub dirty: BTreeSet<String>,
}
impl SNode {
    /// Create a new SNode with an ID.
    pub fn new_id(name: &str, id: &str) -> Self {
        Self {
            id: id.to_owned(),
            name: name.to_owned(),
            ..Default::default()
        }
    }

    /// Create a new SNode without an ID.
    pub fn new(name: &str) -> Self {
        Self {
            id: format!("obj{}", nanoid!()),
            name: name.to_owned(),
            ..Default::default()
        }
    }

    /// Invalidate this node with a symbol.
    pub fn invalidate(&mut self, symbol: &str) {
        self.dirty.insert(symbol.to_owned());
    }

    /// Invalidate all.
    pub fn invalidate_all(&mut self) {
        self.invalidate("all");
    }

    /// Validate all.
    pub fn validate_all(&mut self) -> bool {
        self.validate("all")
    }

    /// Has the dirty symbol?
    pub fn dirty(&self, symbol: &str) -> bool {
        self.dirty.contains(symbol)
    }

    /// Validate.
    pub fn validate(&mut self, symbol: &str) -> bool {
        self.dirty.remove(symbol)
    }

    /// Has dirty tags?
    pub fn has_dirty(&self) -> bool {
        self.dirty.len() > 0
    }

    /// Has immediate child?
    /// If looking for general child of, use SNodeRef is_child_of.
    pub fn has_child(&self, child: &impl IntoNodeRef) -> bool {
        let nref = child.node_ref();
        self.children.contains(&nref)
    }

    /// Put a child node onto this node.
    /// Marks this node as invalid all.
    pub(crate) fn put_child(&mut self, child: &impl IntoNodeRef) {
        let new_child = self.children.insert(child.node_ref());
        if new_child {
            self.invalidate_all();
        }
    }

    /// Remove a child node.
    pub(crate) fn remove_child(&mut self, child: &impl IntoNodeRef) -> bool {
        let removed = self.children.remove(&child.node_ref());
        if removed { self.invalidate_all(); }
        removed
    }

    /// Has data?
    pub fn has_data(&self, data: &impl IntoDataRef) -> bool {
        self.data.contains(&data.data_ref())
    }

    /// Put data onto this node.
    pub(crate) fn put_data(&mut self, data: &impl IntoDataRef) -> bool {
        let new_data = self.data.insert(data.data_ref());
        if new_data {
            self.invalidate_all();
        }
        new_data
    }

    /// Remove data.
    pub(crate) fn remove_data(&mut self, data: &impl IntoDataRef) -> bool {
        let removed = self.data.remove(&data.data_ref());
        if removed { self.invalidate_all(); }
        removed
    }


    /*****************************************************************************
     * Selection.
     *****************************************************************************/

    /// Selection.
    pub fn selection(&self) -> SDataSelection {
        SDataSelection::from(self)
    }

    /// Recursive selection.
    pub fn recursive_selection(&self, graph: &SGraph) -> SDataSelection {
        let mut selection = self.selection();
        for child_ref in &self.children {
            if let Some(child) = child_ref.node(graph) {
                selection.merge(&child.recursive_selection(graph));
            }
        }
        selection
    }

    /// Prefix selection.
    pub fn prefix_selection(&self, prefix: &str) -> SDataSelection {
        let mut selection = SDataSelection::default();
        for dref in &self.data {
            if dref.id.starts_with(prefix) {
                selection.insert(&dref.id);
            }
        }
        selection
    }

    /// Recursive prefix selection.
    pub fn recursive_prefix_selection(&self, graph: &SGraph, prefix: &str) -> SDataSelection {
        let mut selection = self.prefix_selection(prefix);
        for child_ref in &self.children {
            if let Some(child) = child_ref.node(graph) {
                selection.merge(&child.recursive_prefix_selection(graph, prefix));
            }
        }
        selection
    }


    /*****************************************************************************
     * Dump.
     *****************************************************************************/
    
    /// Dump this node.
    pub fn dump(&self, graph: &SGraph, level: i32, data: bool) -> String {
        let mut res = String::new();
        
        let mut ident = String::from("\n");
        for _ in 0..level { ident.push('\t'); }

        // Open the braces for this node
        res.push_str(&format!("{}{} ({}) {{", &ident, &self.name, &self.id));
        if level < 1 { res = res.replace('\n', ""); }

        // Dump data?
        if data {
            let mut ident = String::from("\n");
            for _ in 0..(level + 1) { ident.push('\t'); }

            let mut iident = String::from("\n");
            for _ in 0..(level + 2) { iident.push('\t'); }

            for data_ref in &self.data {
                if let Some(data) = data_ref.data(graph) {
                    res.push_str(&format!("{}data ({}) {{", &ident, &data.id));
                    if data.id.starts_with(FUNC_KIND) {
                        res.push_str(&format!("{}{:?}", &iident, data.get_value::<SFunc>().unwrap()));
                    } else if data.id.starts_with(FKIND) {
                        res.push_str(&format!("{}{:?}", &iident, data.get_value::<SField>().unwrap()));
                    }
                    res.push_str(&format!("{}}}", &ident));
                }
            }

            res.push('\n');
        }

        // Do all children
        for child_ref in &self.children {
            if let Some(child) = child_ref.node(graph) {
                res.push_str(&child.dump(graph, level + 1, data));
            }
        }

        // Close the braces for this node
        res.push_str(&format!("{}}}", &ident));
        res
    }
}
