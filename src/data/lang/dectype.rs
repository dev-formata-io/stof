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
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use crate::{Data, SField, SFunc, SGraph, SNodeRef, SType, SVal};
use super::Expr;


/// Custom type declaration information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomType {
    /// Location ID for this custom type.
    /// This is the SNodeRef ID to this prototype in a SGraph.
    pub locid: String,

    /// Location ID for where this type was declared.
    /// This is the SNodeRef ID associated with this type.
    pub decid: String,

    pub name: String,
    pub fields: Vec<CustomTypeField>,
    pub attributes: BTreeMap<String, SVal>,
}
impl CustomType {
    /// New Type.
    pub fn new(decid: &str, name: &str, fields: Vec<CustomTypeField>) -> Self {
        Self {
            name: name.to_owned(),
            fields,
            locid: format!("ctl{}", nanoid!(10)),
            decid: decid.to_owned(),
            attributes: Default::default(),
        }
    }

    /// Is a private type?
    /// Means this type can only be instantiated/casted to from within it's own scope (or sub-scopes)!
    pub fn is_private(&self) -> bool {
        return self.attributes.contains_key("private");
    }

    /// Location path in the graph for functions.
    /// Also the scope for calling functions on this type.
    pub fn path(&self, graph: &SGraph) -> String {
        SNodeRef::new(&self.locid).path(graph)
    }

    /// Field names.
    pub fn fieldnames(&self) -> HashSet<String> {
        let mut names = HashSet::new();
        for param in &self.fields {
            names.insert(param.name.clone());
        }
        names
    }

    /// Get functions.
    pub fn get_functions(&self, graph: &SGraph) -> Vec<SFunc> {
        SFunc::funcs(graph, &SNodeRef::new(&self.locid))
    }

    /// Typepath for this type.
    pub fn typepath(&self, graph: &SGraph) -> String {
        let typepath = SNodeRef::new(&self.decid).path(&graph).replace('/', ".");
        format!("{}.{}", typepath, self.name)
    }

    /// Insert this custom type into the graph.
    pub fn insert(&mut self, graph: &mut SGraph, location: &str, functions: &mut Vec<SFunc>) {
        let nref = graph.ensure_nodes(location, '/', true, None);
        
        // Set the location of this custom type so it is not lost
        self.locid = nref.id.clone();

        // Insert functions into the graph
        for f in functions {
            f.attach(&nref, graph);
        }

        // Insert typename into the graph
        if let Some(mut typename_field) = SField::field(graph, "typename", '.', Some(&nref)) {
            typename_field.value = SVal::String(self.name.clone());
            typename_field.set(graph);
        } else {
            SField::new_string(graph, "typename", &self.name, &nref);
        }

        // Insert typepath into the graph, which includes the declaration path
        let typepath = SNodeRef::new(&self.decid).path(&graph).replace('/', ".");
        if let Some(mut typepath_field) = SField::field(graph, "typepath", '.', Some(&nref)) {
            typepath_field.value = SVal::String(format!("{}.{}", typepath, self.name));
            typepath_field.set(graph);
        } else {
            SField::new_string(graph, "typepath", &format!("{}.{}", typepath, self.name), &nref);
        }
    }
}


/// Custom type field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTypeField {
    pub name: String,
    pub ptype: SType,
    pub default: Option<Expr>,
    pub attributes: BTreeMap<String, SVal>,
}
impl CustomTypeField {
    /// New parameter.
    pub fn new(name: &str, ptype: SType, default: Option<Expr>, attrs: BTreeMap<String, SVal>) -> Self {
        Self {
            name: name.into(),
            ptype,
            default,
            attributes: attrs,
        }
    }
}
