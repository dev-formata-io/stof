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

use std::collections::{BTreeMap, HashSet};
use serde::{Deserialize, Serialize};
use crate::{lang::{CustomType, ErrorType, SError}, IntoNodeRef, SFunc, SGraph, SNodeRef};


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

    /// Drop types for an associated node, removing all types defined at or under it.
    pub fn drop_types_for(&mut self, node: &SNodeRef, graph: &SGraph) {
        let mut children = graph.all_children(node).into_iter().collect::<HashSet<SNodeRef>>();
        children.insert(node.clone());
        let mut to_remove = Vec::new();
        for (name, custom_types) in &mut self.types {
            custom_types.retain(|ctype| !children.contains(&SNodeRef::new(&ctype.decid)));
            if custom_types.len() < 1 {
                to_remove.push(name.clone());
            }
        }
        for name in to_remove {
            self.types.remove(&name);
        }
    }

    /// Get all types associated with a node, either defined by it or under it (all children).
    pub fn declared_types_for(&self, node: &SNodeRef, graph: &SGraph) -> Self {
        let mut children = graph.all_children(node).into_iter().collect::<HashSet<SNodeRef>>();
        children.insert(node.clone());
        let mut new_types = BTreeMap::new();
        for (name, custom_types) in &self.types {
            let mut new_custom_types = Vec::new();

            for ctype in custom_types {
                if children.contains(&SNodeRef::new(&ctype.decid)) {
                    new_custom_types.push(ctype.clone());
                }
            }

            if new_custom_types.len() > 0 {
                new_types.insert(name.clone(), new_custom_types);
            }
        }
        Self {
            types: new_types,
        }
    }

    /// Declare a new type (not inserted into graph yet).
    pub fn declare(&mut self, mut custom_type: CustomType, graph: &mut SGraph, extends: &str, functions: Vec<SFunc>) -> Result<(), SError> {
        // Insert path for this new custom type
        let mut insert_path = format!("__stof__/prototypes/{}", &custom_type.locid);

        // Handle extends if any - adds fields and functions from an extends type
        if extends.len() > 0 {
            let mut extends_path: Vec<&str> = extends.split('.').collect();
            let extends_name = extends_path.pop().unwrap();
            let mut extends_scope = SNodeRef::from(&custom_type.decid);
            if extends_path.len() > 0 {
                let extends_node_path = extends_path.join("/");
                if extends_node_path.starts_with("self") || extends_node_path.starts_with("super") {
                    if let Some(nref) = graph.node_ref(&extends_node_path, Some(&SNodeRef::from(&custom_type.decid))) {
                        extends_scope = nref;
                    }
                } else {
                    if let Some(nref) = graph.node_ref(&extends_node_path, None) {
                        extends_scope = nref;
                    }
                }
            }

            if let Some(extend_type) = self.find(&graph, extends_name, &extends_scope) {
                let custom_field_names = custom_type.fieldnames();
                for ef in &extend_type.fields {
                    if !custom_field_names.contains(&ef.name) {
                        custom_type.fields.push(ef.clone());
                    }
                }

                // Set insert path as a child of the extends type
                insert_path = format!("{}/{}", SNodeRef::new(&extend_type.locid).path(&graph), &custom_type.locid);
            } else {
                let error = SError {
                    pid: "main".to_string(),
                    error_type: ErrorType::ParseError,
                    message: format!("attempting to extend a type that does not exist: {}", extends),
                    call_stack: Default::default(),
                };
                return Err(error);
            }
        }

        // Insert the custom type into the graph
        custom_type.insert(graph, &insert_path, functions);

        // Insert into types by name
        if let Some(types) = self.types.get_mut(&custom_type.name) {
            types.push(custom_type);
        } else {
            self.types.insert(custom_type.name.clone(), vec![custom_type]);
        }
        Ok(())
    }

    /// Find a type by name from within a scope.
    /// Gets all types with the name, then returns the one closest to scope (has to be above or equal to scope).
    pub fn find(&self, graph: &SGraph, name: &str, scope: impl IntoNodeRef) -> Option<&CustomType> {
        if let Some(type_name_matches) = self.types.get(name) {
            if type_name_matches.len() < 1  { return None; }
            if type_name_matches.len() == 1 { return Some(&type_name_matches[0]); }

            let scope_ref = scope.node_ref();
            let mut best = None;
            let mut best_dist = i32::MAX;
            for ctype in type_name_matches {
                let nref = SNodeRef::from(&ctype.decid);
                let dist = scope_ref.is_child_of_distance(graph, &nref);
                if dist > -1 { // if -1, scope_ref is not a child (or equal) of nref
                    if dist < best_dist {
                        best_dist = dist;
                        best = Some(ctype);
                    }
                }
            }
            if best.is_none() {
                return type_name_matches.last(); // we know we have some types... so just return the last one declared
            }
            return best;
        }
        None
    }
}
