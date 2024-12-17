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

use serde::{Deserialize, Serialize};
use trie_rs::{Trie, TrieBuilder};
use super::{IntoDataRef, SData, SDataRef, SNode};


/// Data selection.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SDataSelection {
    pub data: Vec<SDataRef>,

    #[serde(skip)]
    pub trie: Option<Trie<u8>>,
}
impl From<&SNode> for SDataSelection {
    fn from(value: &SNode) -> Self {
        Self::from(&value.data)
    }
}
impl From<&SDataSelection> for SDataSelection {
    fn from(value: &SDataSelection) -> Self {
        Self::from(&value.data)
    }
}
impl From<&SDataRef> for SDataSelection {
    fn from(value: &SDataRef) -> Self {
        Self::from(vec![value])
    }
}
impl From<SDataRef> for SDataSelection {
    fn from(value: SDataRef) -> Self {
        Self::from(vec![value])
    }
}
impl From<&SData> for SDataSelection {
    fn from(value: &SData) -> Self {
        Self::from(vec![value.data_ref()])
    }
}
impl From<Vec<&SData>> for SDataSelection {
    fn from(value: Vec<&SData>) -> Self {
        let mut data = Vec::new();
        for val in value { data.push(val.data_ref()); }
        Self::from(data)
    }
}
impl From<&Vec<&SData>> for SDataSelection {
    fn from(value: &Vec<&SData>) -> Self {
        let mut data = Vec::new();
        for val in value { data.push(val.data_ref()); }
        Self::from(data)
    }
}
impl From<Vec<&str>> for SDataSelection {
    fn from(value: Vec<&str>) -> Self {
        let mut data: Vec<SDataRef> = Vec::new();
        for val in &value {
            data.push(SDataRef::from(*val));
        }
        let mut selection = Self::default();
        selection.data = data;
        selection.build_trie();
        selection
    }
}
impl From<Vec<String>> for SDataSelection {
    fn from(value: Vec<String>) -> Self {
        let mut data: Vec<SDataRef> = Vec::new();
        for val in value {
            data.push(SDataRef::from(val));
        }
        let mut selection = Self::default();
        selection.data = data;
        selection.build_trie();
        selection
    }
}
impl From<Vec<&String>> for SDataSelection {
    fn from(value: Vec<&String>) -> Self {
        let mut data: Vec<SDataRef> = Vec::new();
        for val in &value {
            data.push(SDataRef::from(*val));
        }
        let mut selection = Self::default();
        selection.data = data;
        selection.build_trie();
        selection
    }
}
impl From<Vec<&SDataRef>> for SDataSelection {
    fn from(value: Vec<&SDataRef>) -> Self {
        let mut data: Vec<SDataRef> = Vec::new();
        for val in &value {
            data.push(SDataRef::from(&val.id));
        }
        let mut selection = Self::default();
        selection.data = data;
        selection.build_trie();
        selection
    }
}
impl From<&Vec<&SDataRef>> for SDataSelection {
    fn from(value: &Vec<&SDataRef>) -> Self {
        let mut data: Vec<SDataRef> = Vec::new();
        for val in value {
            data.push(SDataRef::from(&val.id));
        }
        let mut selection = Self::default();
        selection.data = data;
        selection.build_trie();
        selection
    }
}
impl From<Vec<SDataRef>> for SDataSelection {
    fn from(value: Vec<SDataRef>) -> Self {
        let mut selection = Self::default();
        selection.data = value;
        selection.build_trie();
        selection
    }
}
impl From<&Vec<SDataRef>> for SDataSelection {
    fn from(value: &Vec<SDataRef>) -> Self {
        let mut data: Vec<SDataRef> = Vec::new();
        for val in value {
            data.push(SDataRef::from(&val.id));
        }
        let mut selection = Self::default();
        selection.data = data;
        selection.build_trie();
        selection
    }
}
impl SDataSelection {
    /// Build trie.
    pub fn build_trie(&mut self) {
        let mut builder = TrieBuilder::new();
        for dref in &self.data {
            builder.push(&dref.id);
        }
        self.trie = Some(builder.build());
    }

    /// Size of this selection.
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Contains an ID?
    pub fn contains(&self, toref: impl IntoDataRef) -> bool {
        let id = toref.data_ref().id;
        if let Some(trie) = &self.trie {
            return trie.exact_match(&id);
        }
        for dref in &self.data {
            if dref.id == id { return true; }
        }
        false
    }

    /// Add data to this selection.
    pub fn push(&mut self, toref: impl IntoDataRef, build: bool) -> bool {
        let dref = toref.data_ref();
        if !self.contains(&dref) {
            self.data.push(dref);
            if build {
                self.build_trie();
            }
            return true;
        }
        false
    }

    /// Remove data from this selection.
    pub fn remove(&mut self, toref: impl IntoDataRef) -> bool {
        let id = toref.data_ref().id;
        let mut count = 0;
        self.data.retain(|x| {
            let keep = x.id != id;
            if !keep { count += 1; }
            keep
        });
        if count > 0 {
            self.build_trie();
        }
        count > 0
    }

    /// Merge this selection with another.
    /// Other selection gets added to this selection and is unmodified.
    pub fn merge(&mut self, other: &Self, build: bool) {
        let mut added = false;
        for dref in &other.data {
            added = self.push(dref, false) || added;
        }
        if added && build {
            self.build_trie();
        }
    }

    /// Exact match?
    pub fn exact_match(&self, toref: impl IntoDataRef) -> bool {
        let id = toref.data_ref().id;
        if let Some(trie) = &self.trie {
            return trie.exact_match(&id);
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

    /// Prefix selection.
    pub fn prefix_selection(&self, prefix: &str) -> Self {
        Self::from(self.prefix_matches(prefix))
    }
}


/// Into iterator for DataSelection.
impl IntoIterator for SDataSelection {
    type Item = SDataRef;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

/// Into iterator for DataSelection.
impl<'a> IntoIterator for &'a SDataSelection {
    type Item = &'a SDataRef;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        let mut vec = Vec::new();
        for dref in &self.data {
            vec.push(dref);
        }
        vec.into_iter()
    }
}

/// Into iterator for DataSelection.
impl<'a> IntoIterator for &'a mut SDataSelection {
    type Item = &'a mut SDataRef;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        let mut vec = Vec::new();
        for dref in &mut self.data {
            vec.push(dref);
        }
        vec.into_iter()
    }
}
