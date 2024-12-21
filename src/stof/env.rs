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

use std::collections::{HashMap, HashSet};
use crate::{Data, IntoNodeRef, SData, SField, SFunc, SGraph, SNodeRef, SType, SVal};


/// Stof parse environment.
/// Used in controling the parsing of a stof document.
#[derive(Clone, Debug)]
pub struct StofEnv {
    /// The "root" node we are parsing Stof source into.
    pub main: SNodeRef,

    /// Paths that have been parsed already.
    pub compiled_paths: HashSet<String>,

    /// Assign type stack (keep track of variable types for casting when declared with type).
    pub assign_type_stack: Vec<HashMap<String, SType>>,

    /// Init functions to execute (in order) after parse is complete.
    pub init_funcs: Vec<(SFunc, Vec<SVal>)>,

    /// "Compile" time field names per node.
    /// Used for collision handling.
    /// NodeRef->(FieldName->FieldId)
    node_field_collisions: HashMap<String, HashMap<String, String>>,
}
impl StofEnv {
    /// Construct a new Stof env from a document.
    pub fn new(graph: &mut SGraph) -> Self {
        let main;
        if graph.roots.len() < 1 {
            main = graph.insert_root("root");
        } else {
            main = graph.main_root().unwrap();
        }
        Self {
            main,
            compiled_paths: Default::default(),
            assign_type_stack: vec![Default::default()],
            init_funcs: Default::default(),
            node_field_collisions: Default::default(),
        }
    }

    /// Construct a new Stof env from a document and main node.
    pub fn new_at_node(graph: &mut SGraph, node: impl IntoNodeRef) -> Option<Self> {
        let nref = node.node_ref();
        if nref.exists(&graph) {
            return Some(Self {
                main: nref,
                init_funcs: Default::default(),
                assign_type_stack: vec![Default::default()],
                compiled_paths: Default::default(),
                node_field_collisions: Default::default(),
            });
        }
        None
    }

    /// Insert a field onto a node.
    /// Check for field collisions on the node, merging fields if necessary.
    pub(crate) fn insert_field(&mut self, graph: &mut SGraph, node: impl IntoNodeRef, field: &mut SField) {
        let node_ref = node.node_ref();
        if !self.node_field_collisions.contains_key(&node_ref.id) {
            let existing_fields = SField::fields(&graph, &node_ref);
            let mut map = HashMap::new();
            for field in existing_fields {
                map.insert(field.name, field.id);
            }
            self.node_field_collisions.insert(node_ref.id.clone(), map);
        }
        let mut merged = false;
        if let Some(existing) = self.node_field_collisions.get_mut(&node_ref.id) {
            if existing.contains_key(&field.name) {
                // This field collides with an existing one on this node!
                // Union the existing field with the new field, and set the existing back into the graph
                if let Ok(mut existing_field) = SData::data::<SField>(&graph, existing.get(&field.name).unwrap()) {
                    existing_field.union(field);
                    existing_field.set(graph);
                    merged = true;
                }
            } else {
                // We have not collided with any field names on this node, so insert the field into the collisions
                existing.insert(field.name.clone(), field.id.clone());
            }
        }
        if !merged {
            field.attach(&node_ref, graph);
        }
    }

    /// Before parse.
    pub fn before_parse(&mut self, graph: &mut SGraph) {
        graph.stack.self_stack.push(self.main.clone());
    }

    /// After parse.
    pub fn after_parse(&mut self, graph: &mut SGraph) {
        self.call_init_functions(graph);
        graph.stack.clean();
    }

    /// Already compiled this file?
    pub fn compiled_path(&self, path: &str) -> bool {
        self.compiled_paths.contains(path)
    }

    /// Add file path to compiled files.
    pub fn add_compiled_path(&mut self, path: &str) {
        self.compiled_paths.insert(path.to_owned());
    }

    /// Current scope.
    pub fn scope(&self, graph: &SGraph) -> SNodeRef {
        if let Some(nref) = graph.stack.self_ptr() {
            nref
        } else {
            self.main.clone()
        }
    }

    /// Set scope of this graph!
    /// Adds every node in the path if needed and sets the current scope.
    pub fn push_scope(&mut self, graph: &mut SGraph, path: &str, sep: char, fields: bool) -> SNodeRef {
        let nref = graph.ensure_nodes(path, sep, fields, None);
        self.push_scope_ref(graph, nref.clone());
        nref
    }

    /// Push scope ref.
    pub fn push_scope_ref(&mut self, graph: &mut SGraph, nref: SNodeRef) {
        graph.stack.self_stack.push(nref);
        self.assign_type_stack.push(HashMap::default());
    }

    /// Pop scope.
    pub fn pop_scope(&mut self, graph: &mut SGraph) {
        graph.stack.self_stack.pop();
        self.assign_type_stack.pop();
    }

    /// Call init functions with the document.
    pub fn call_init_functions(&self, graph: &mut SGraph) {
        for (func, params) in &self.init_funcs {
            func.call(graph, params.clone(), true).expect(&format!("Failed to call init function: {:?}", func));
        }
    }
}
