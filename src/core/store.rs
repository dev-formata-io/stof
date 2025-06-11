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

use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use super::{SData, SNode};


/// Stof store trait.
pub trait Store<T> {
    /// Get an item by ID.
    fn get(&self, id: &str) -> Option<&T>;

    /// Get a mutable item by ID.
    fn get_mut(&mut self, id: &str) -> Option<&mut T>;

    /// Set an item in the store.
    fn set(&mut self, id: &str, item: T);

    /// Drop from the store.
    /// Skips any deadpools.
    fn drop_item(&mut self, id: &str) -> bool;

    /// Remove from the store.
    /// Adds the object to a deadpool.
    fn remove(&mut self, id: &str) -> bool;

    /// Contains?
    fn contains(&self, id: &str) -> bool;

    /// Flush deadpool.
    fn flush_deadpool(&mut self) -> Vec<T>;

    /// Flush for validation.
    /// Collect all dirty items (with limit).
    fn flush(&mut self, limit: i32) -> Vec<&mut T>;
}


/// Stof Node store.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct SNodeStore {
    /// Nodes.
    pub store: FxHashMap<String, SNode>,

    /// Deadpool.
    #[serde(skip)]
    pub deadpool: FxHashMap<String, SNode>,
}
impl Store<SNode> for SNodeStore {
    /// Get by ID.
    fn get(&self, id: &str) -> Option<&SNode> {
        self.store.get(id)
    }

    /// Get mutable node by ID.
    fn get_mut(&mut self, id: &str) -> Option<&mut SNode> {
        self.store.get_mut(id)
    }

    /// Set a node.
    fn set(&mut self, id: &str, item: SNode) {
        self.store.insert(id.to_owned(), item);
    }

    /// Drop a node.
    fn drop_item(&mut self, id: &str) -> bool {
        self.store.remove(id).is_some()
    }

    /// Remove a node.
    /// Adds to the deadpool.
    fn remove(&mut self, id: &str) -> bool {
        if let Some(node) = self.store.remove(id) {
            self.deadpool.insert(node.id.clone(), node);
            return true;
        }
        false
    }

    /// Contains node?
    fn contains(&self, id: &str) -> bool {
        self.store.contains_key(id)
    }

    /// Flush deadpool.
    fn flush_deadpool(&mut self) -> Vec<SNode> {
        let mut res = Vec::new();
        for (_, node) in self.deadpool.drain() { res.push(node); }
        res
    }

    /// Flush nodes.
    /// Collect any dirty nodes with a limit.
    /// For no limit, set "limit" to -1.
    fn flush(&mut self, limit: i32) -> Vec<&mut SNode> {
        let mut res = Vec::new();
        let mut n = 0;
        for (_, node) in &mut self.store {
            if node.has_dirty() {
                n += 1;
                res.push(node);

                if limit > -1 && n >= limit { break; }
            }
        }
        res
    }
}


/// Stof Data store.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct SDataStore {
    /// Data.
    pub store: FxHashMap<String, SData>,

    /// Deadpool.
    #[serde(skip)]
    pub deadpool: FxHashMap<String, SData>,
}
impl Store<SData> for SDataStore {
    /// Get by ID.
    fn get(&self, id: &str) -> Option<&SData> {
        self.store.get(id)
    }

    /// Get mutable data by ID.
    fn get_mut(&mut self, id: &str) -> Option<&mut SData> {
        self.store.get_mut(id)
    }

    /// Set data.
    fn set(&mut self, id: &str, item: SData) {
        self.store.insert(id.to_owned(), item);
    }

    /// Drop data.
    fn drop_item(&mut self, id: &str) -> bool {
        self.store.remove(id).is_some()
    }

    /// Remove data.
    /// Adds to the deadpool.
    fn remove(&mut self, id: &str) -> bool {
        if let Some(data) = self.store.remove(id) {
            self.deadpool.insert(data.id.clone(), data);
            return true;
        }
        false
    }

    /// Contains data?
    fn contains(&self, id: &str) -> bool {
        self.store.contains_key(id)
    }

    /// Flush deadpool.
    fn flush_deadpool(&mut self) -> Vec<SData> {
        let mut res = Vec::new();
        for (_, data) in self.deadpool.drain() { res.push(data); }
        res
    }

    /// Flush data.
    /// Collect any dirty data with a limit.
    /// For no limit, set "limit" to -1.
    fn flush(&mut self, limit: i32) -> Vec<&mut SData> {
        let mut res = Vec::new();
        let mut n = 0;
        for (_, data) in &mut self.store {
            if data.has_dirty() {
                n += 1;
                res.push(data);
                
                if limit > -1 && n >= limit { break; }
            }
        }
        res
    }
}
