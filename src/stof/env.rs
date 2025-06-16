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
use crate::{lang::SError, IntoNodeRef, SData, SDataRef, SDoc, SField, SFieldDoc, SFunc, SNodeRef, SType, SVal};


/// Stof parse environment.
/// Used in controling the parsing of a stof document.
#[derive(Clone, Debug)]
pub struct StofEnv {
    /// The "root" node we are parsing Stof source into.
    pub main: SNodeRef,

    /// Process ID in which we are compiling.
    pub pid: String,

    /// Relative import path. Gets added to the import path when starting with '.'
    pub relative_import_path: String,

    /// Assign type stack (keep track of variable types for casting when declared with type).
    pub assign_type_stack: Vec<FxHashMap<String, SType>>,

    /// Init functions to execute (in order) after parse is complete.
    pub init_funcs: Vec<(SDataRef, Vec<SVal>)>,

    /// Parse documentation data into the document?
    /// If a function has doc comments, will add SFuncDoc to the document as well.
    pub documentation: bool,

    /// "Compile" time field names per node.
    /// Used for collision handling.
    /// NodeRef->(FieldName->FieldDataRef)
    node_field_collisions: FxHashMap<String, FxHashMap<String, SDataRef>>,
}
impl StofEnv {
    /// Construct a new Stof env from a document.
    pub fn new(pid: &str, doc: &mut SDoc, documentation: bool) -> Self {
        let main;
        if doc.graph.roots.len() < 1 {
            main = doc.graph.insert_root("root");
        } else {
            main = doc.graph.main_root().unwrap();
        }
        Self {
            main,
            documentation,
            pid: pid.to_owned(),
            assign_type_stack: vec![Default::default()],
            init_funcs: Default::default(),
            node_field_collisions: Default::default(),
            relative_import_path: String::default(),
        }
    }

    /// Construct a new Stof env from a document and main node.
    pub fn new_at_node(pid: &str, doc: &mut SDoc, node: impl IntoNodeRef, documentation: bool) -> Option<Self> {
        let nref = node.node_ref();
        if nref.exists(&doc.graph) {
            return Some(Self {
                main: nref,
                documentation,
                pid: pid.to_owned(),
                init_funcs: Default::default(),
                assign_type_stack: vec![Default::default()],
                node_field_collisions: Default::default(),
                relative_import_path: String::default(),
            });
        }
        None
    }

    /// Insert a field onto a node.
    /// Check for field collisions on the node, merging fields if necessary.
    pub(crate) fn insert_field(&mut self, doc: &mut SDoc, node: impl IntoNodeRef, field: SField, comments: Option<String>) -> Result<(), SError> {
        let node_ref = node.node_ref();
        if !self.node_field_collisions.contains_key(&node_ref.id) {
            let mut map = FxHashMap::default();
            for existing_ref in SField::field_refs(&doc.graph, &node_ref) {
                if let Some(field) = SData::get::<SField>(&doc.graph, &existing_ref) {
                    map.insert(field.name.clone(), existing_ref);
                }
            }
            self.node_field_collisions.insert(node_ref.id.clone(), map);
        }
        if let Some(existing) = self.node_field_collisions.get_mut(&node_ref.id) {
            if existing.contains_key(&field.name) {
                // This field collides with an existing one on this node!
                // Union the existing field with the new field, and set the existing back into the graph
                if let Some(existing_field) = SData::get_mut::<SField>(&mut doc.graph, existing.get(&field.name).unwrap()) {
                    existing_field.merge(&field)?;
                }
            } else {
                // We have not collided with any field names on this node, so insert the field into the collisions
                let name = field.name.clone();
                if let Some(dref) = SData::insert_new(&mut doc.graph, &node_ref, Box::new(field)) {
                    existing.insert(name, dref.clone());

                    // Insert field comments if we have any
                    if let Some(comments) = comments {
                        SData::insert_new(&mut doc.graph, &node_ref, Box::new(SFieldDoc::new(dref, comments)));
                    }
                }
            }
        }
        Ok(())
    }

    /// Before parse.
    pub fn before_parse(&mut self, doc: &mut SDoc) {
        doc.push_self(&self.pid, self.main.clone());
    }

    /// After parse.
    pub fn after_parse(&mut self, doc: &mut SDoc) {
        self.call_init_functions(doc);
        doc.clean(&self.pid);
    }

    /// Already compiled this file?
    pub fn compiled_path(&self, path: &str, doc: &SDoc) -> bool {
        doc.env_compiled_paths.contains(path)
    }

    /// Add file path to compiled files.
    pub fn add_compiled_path(&mut self, path: &str, doc: &mut SDoc) {
        doc.env_compiled_paths.insert(path.to_owned());
    }

    /// Current scope.
    pub fn scope(&self, doc: &SDoc) -> SNodeRef {
        if let Some(nref) = doc.self_ptr(&self.pid) {
            nref
        } else {
            self.main.clone()
        }
    }

    /// Set scope of this graph!
    /// Adds every node in the path if needed and sets the current scope.
    pub fn push_scope(&mut self, doc: &mut SDoc, path: &str, sep: char, fields: bool) -> SNodeRef {
        let nref = doc.graph.ensure_nodes(path, sep, fields, None);
        self.push_scope_ref(doc, nref.clone());
        nref
    }

    /// Push scope ref.
    pub fn push_scope_ref(&mut self, doc: &mut SDoc, nref: SNodeRef) {
        doc.push_self(&self.pid, nref);
    }

    /// Pop scope.
    pub fn pop_scope(&mut self, doc: &mut SDoc) {
        doc.pop_self(&self.pid);
    }

    /// Get assign type for a variable name.
    pub fn assign_type_for_var(&self, ident: &str) -> Option<SType> {
        for types in self.assign_type_stack.iter().rev() {
            if let Some(tp) = types.get(ident) {
                return Some(tp.clone());
            }
        }
        None
    }

    /// Call init functions with the document.
    pub fn call_init_functions(&self, doc: &mut SDoc) {
        for (dref, params) in &self.init_funcs {
            SFunc::call(dref, &self.pid, doc, params.clone(), true, true).expect(&format!("Failed to call init function: {:?}", SData::get::<SFunc>(&doc.graph, dref)));
        }
    }
}
