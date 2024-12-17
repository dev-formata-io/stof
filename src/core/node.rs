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

use std::collections::HashSet;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use trie_rs::{Trie, TrieBuilder};
use crate::{SField, SFunc, FKIND, FUNC_KIND};
use super::{SDataRef, SDataSelection, SGraph, SNodeRef, SRef};


/// Stof node.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SNode {
    pub id: String,
    pub name: String,
    pub parent: Option<SNodeRef>,
    pub children: Vec<SNodeRef>,
    pub data: Vec<SDataRef>,
    
    #[serde(skip)]
    pub dirty: HashSet<String>,

    #[serde(skip)]
    pub trie: Option<Trie<u8>>,
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
    pub fn has_child(&self, child: &impl SRef) -> bool {
        let id = child.get();
        for nref in &self.children {
            if nref.id == id { return true; }
        }
        false
    }

    /// Put a child node onto this node.
    /// Marks this node as invalid all.
    pub(crate) fn put_child(&mut self, child: &impl SRef) {
        let mut exists = false;
        let id = child.get();
        for chd in &self.children {
            if chd.id == id { exists = true; }
        }
        if !exists {
            self.children.push(SNodeRef::from(child.get()));
            self.invalidate_all();
        }
    }

    /// Remove a child node.
    pub(crate) fn remove_child(&mut self, child: &impl SRef) -> bool {
        let id = child.get();
        let mut ct = 0;
        self.children.retain(|nref| -> bool {
            let keep = nref.id != id;
            if !keep { ct += 1; }
            keep
        });
        if ct > 0 { self.invalidate_all(); }
        ct > 0
    }

    /// Has data?
    pub fn has_data(&self, data: &impl SRef) -> bool {
        let id = data.get();
        if let Some(trie) = &self.trie {
            return trie.exact_match(&id);
        }
        for dref in &self.data {
            if dref.id == id { return true; }
        }
        false
    }

    /// Put data onto this node.
    pub(crate) fn put_data(&mut self, data: &impl SRef, check: bool) -> bool {
        if check && self.has_data(data) { return false; }
        self.data.push(SDataRef::from(data.get()));
        self.invalidate_all();
        self.build_trie();
        true
    }

    /// Remove data.
    pub(crate) fn remove_data(&mut self, data: &impl SRef) -> bool {
        let id = data.get();
        let mut ct = 0;
        self.data.retain(|dref| -> bool {
            let keep = dref.id != id;
            if !keep { ct += 1; }
            keep
        });
        if ct > 0 {
            self.invalidate_all();
            self.build_trie();
        }
        ct > 0
    }


    /*****************************************************************************
     * Trie.
     *****************************************************************************/

    /// Build trie.
    /// Creates a trie out of all the data ids on this node.
    pub fn build_trie(&mut self) {
        let mut builder = TrieBuilder::new();
        for dref in &self.data {
            builder.push(&dref.id);
        }
        self.trie = Some(builder.build());
    }
    
    /// Exact match?
    pub fn exact_match(&self, id: &str) -> bool {
        if let Some(trie) = &self.trie {
            return trie.exact_match(id);
        }
        false
    }

    /// Return all data ID with the prefix.
    pub fn prefix_matches(&self, prefix: &str) -> Vec<SDataRef> {
        if let Some(trie) = &self.trie {
            let results: Vec<String> = trie.predictive_search(prefix).collect();
            let mut data = Vec::new();
            for res in results {
                data.push(SDataRef::from(res));
            }
            return data;
        }
        vec![]
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
                selection.merge(&child.recursive_selection(graph), false);
            }
        }
        selection.build_trie();
        selection
    }

    /// Prefix selection.
    pub fn prefix_selection(&self, prefix: &str) -> SDataSelection {
        SDataSelection::from(self.prefix_matches(prefix))
    }

    /// Recursive prefix selection.
    pub fn recursive_prefix_selection(&self, graph: &SGraph, prefix: &str) -> SDataSelection {
        let mut selection = self.prefix_selection(prefix);
        for child_ref in &self.children {
            if let Some(child) = child_ref.node(graph) {
                selection.merge(&child.recursive_prefix_selection(graph, prefix), false);
            }
        }
        selection.build_trie();
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
