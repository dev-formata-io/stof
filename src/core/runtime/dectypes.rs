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

use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use crate::{lang::CustomType, IntoNodeRef, SGraph, SNodeRef};


/// Stof custom types declared in a document.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CustomTypes {
    pub types: BTreeMap<String, Vec<CustomType>>,
}
impl CustomTypes {
    /// Merge custom types.
    pub fn merge(&mut self, other: &Self) {
        for (name, types) in &other.types {
            if !self.types.contains_key(name) {
                self.types.insert(name.clone(), types.clone());
            } else if let Some(type_vec) = self.types.get_mut(name) {
                for ty in types {
                    type_vec.push(ty.clone());
                }
            }
        }
    }

    /// Find a type by name from within a scope.
    /// Gets all types with the name, then returns the one closest to scope (has to be above or equal to scope).
    pub fn find(&self, graph: &SGraph, name: &str, scope: impl IntoNodeRef) -> Option<&CustomType> {
        if let Some(type_name_matches) = self.types.get(name) {
            let scope_ref = scope.node_ref();
            let mut best = None;
            let mut best_dist = i32::MAX;
            for ctype in type_name_matches {
                let nref = SNodeRef::from(&ctype.decid);
                if scope_ref.is_child_of(graph, &nref) {
                    let dist = scope_ref.distance_to(graph, &nref);
                    if dist > -1 { // if -1, they are in different roots
                        if dist < best_dist {
                            best_dist = dist;
                            best = Some(ctype);
                        }
                    }
                }
            }
            return best;
        }
        None
    }
}
