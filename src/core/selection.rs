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
use serde::{Deserialize, Serialize};
use super::{IntoDataRef, SData, SDataRef, SNode};


/// Data selection.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SDataSelection {
    pub data: BTreeSet<SDataRef>,
}
impl From<&SNode> for SDataSelection {
    fn from(value: &SNode) -> Self {
        let mut selection = Self::default();
        selection.data = value.data.clone();
        selection
    }
}
impl From<&SDataSelection> for SDataSelection {
    fn from(value: &SDataSelection) -> Self {
        value.clone()
    }
}
impl From<&SDataRef> for SDataSelection {
    fn from(value: &SDataRef) -> Self {
        let mut selection = Self::default();
        selection.insert(value);
        selection
    }
}
impl From<SDataRef> for SDataSelection {
    fn from(value: SDataRef) -> Self {
        let mut selection = Self::default();
        selection.data.insert(value);
        selection
    }
}
impl From<&SData> for SDataSelection {
    fn from(value: &SData) -> Self {
        let mut selection = Self::default();
        selection.insert(value);
        selection
    }
}
impl From<Vec<&SData>> for SDataSelection {
    fn from(value: Vec<&SData>) -> Self {
        let mut selection = Self::default();
        for val in value { selection.insert(val); }
        selection
    }
}
impl From<&Vec<&SData>> for SDataSelection {
    fn from(value: &Vec<&SData>) -> Self {
        let mut selection = Self::default();
        for val in value { selection.insert(*val); }
        selection
    }
}
impl From<Vec<&str>> for SDataSelection {
    fn from(value: Vec<&str>) -> Self {
        let mut selection = Self::default();
        for val in value { selection.insert(val); }
        selection
    }
}
impl From<Vec<String>> for SDataSelection {
    fn from(value: Vec<String>) -> Self {
        let mut selection = Self::default();
        for val in value { selection.insert(val); }
        selection
    }
}
impl From<Vec<&String>> for SDataSelection {
    fn from(value: Vec<&String>) -> Self {
        let mut selection = Self::default();
        for val in value { selection.insert(val); }
        selection
    }
}
impl From<Vec<&SDataRef>> for SDataSelection {
    fn from(value: Vec<&SDataRef>) -> Self {
        let mut selection = Self::default();
        for val in value { selection.insert(val); }
        selection
    }
}
impl From<&Vec<&SDataRef>> for SDataSelection {
    fn from(value: &Vec<&SDataRef>) -> Self {
        let mut selection = Self::default();
        for val in value { selection.insert(*val); }
        selection
    }
}
impl From<Vec<SDataRef>> for SDataSelection {
    fn from(value: Vec<SDataRef>) -> Self {
        let mut selection = Self::default();
        for val in value { selection.data.insert(val); }
        selection
    }
}
impl From<&Vec<SDataRef>> for SDataSelection {
    fn from(value: &Vec<SDataRef>) -> Self {
        let mut selection = Self::default();
        for val in value { selection.insert(val); }
        selection
    }
}
impl SDataSelection {
    /// Size of this selection.
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Contains an ID?
    pub fn contains(&self, toref: impl IntoDataRef) -> bool {
        self.data.contains(&toref.data_ref())
    }

    /// Add data to this selection.
    pub fn insert(&mut self, toref: impl IntoDataRef) -> bool {
        self.data.insert(toref.data_ref())
    }

    /// Remove data from this selection.
    pub fn remove(&mut self, toref: impl IntoDataRef) -> bool {
        self.data.remove(&toref.data_ref())
    }

    /// Merge this selection with another.
    /// Other selection gets added to this selection and is unmodified.
    pub fn merge(&mut self, other: &Self) {
        for dref in &other.data {
            self.data.insert(dref.clone());
        }
    }

    /// Prefix selection.
    pub fn prefix_selection(&self, prefix: &str) -> Self {
        let mut selection = Self::default();
        for dref in &self.data {
            if dref.id.starts_with(prefix) {
                selection.data.insert(dref.clone());
            }
        }
        selection
    }
}


/// Into iterator for DataSelection.
impl IntoIterator for SDataSelection {
    type Item = SDataRef;
    type IntoIter = std::collections::btree_set::IntoIter<Self::Item>;
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
