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

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::{IntoDataRef, SField, SFunc, SGraph, SNodeRef};


/// Access.
/// Default access is None.
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub enum Access {
    #[default]
    None,
    Read,
    Write,
}
impl Access {
    /// Can read?
    pub fn can_read(&self) -> bool {
        match self {
            Self::Read |
            Self::Write => true,
            _ => false,
        }
    }

    /// Can write?
    pub fn can_write(&self) -> bool {
        match self {
            Self::Write => true,
            _ => false,
        }
    }
}


/// Document permissions.
/// A standard permissions interface for Stof documents.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DocPermissions {
    pub permissions: Permissions,
}
impl DocPermissions {
    /// Merge doc permissions.
    pub fn merge(&mut self, other: &Self) {
        self.permissions.merge(&other.permissions);
    }

    /// Can read field?
    pub fn can_read_field(&mut self, graph: &SGraph, field: &SField, from: Option<&SNodeRef>) -> bool {
        let private_field = field.attributes.contains_key("private");
        if let Some(data) = field.data_ref().data(graph) {
            for nref in &data.nodes {
                if self.can_read_scope(graph, nref, from) {
                    if let Some(from_ref) = from {
                        if private_field && from_ref.id == nref.id {
                            return true;
                        } else if !private_field {
                            return true;
                        }
                    } else {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Can read func?
    pub fn can_read_func(&mut self, graph: &SGraph, func: &SFunc, from: Option<&SNodeRef>) -> bool {
        let private_func = func.attributes.contains_key("private");
        if let Some(data) = func.data_ref().data(graph) {
            for nref in &data.nodes {
                if self.can_read_scope(graph, nref, from) {
                    if let Some(from_ref) = from {
                        if private_func && from_ref.id == nref.id {
                            return true;
                        } else if !private_func {
                            return true;
                        }
                    } else {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Can write field?
    pub fn can_write_field(&mut self, graph: &SGraph, field: &SField, from: Option<&SNodeRef>) -> bool {
        if let Some(read_only_val) = field.attributes.get("readonly") {
            if read_only_val.is_empty() || read_only_val.truthy() {
                return false;
            }
        }
        let private_field = field.attributes.contains_key("private");
        if let Some(data) = field.data_ref().data(graph) {
            for nref in &data.nodes {
                if self.can_write_scope(graph, nref, from) {
                    if let Some(from_ref) = from {
                        if private_field && from_ref.id == nref.id {
                            return true;
                        } else if !private_field {
                            return true;
                        }
                    } else {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Can write function?
    pub fn can_write_func(&mut self, graph: &SGraph, func: &SFunc, from: Option<&SNodeRef>) -> bool {
        let private_func = func.attributes.contains_key("private");
        if let Some(data) = func.data_ref().data(graph) {
            for nref in &data.nodes {
                if self.can_write_scope(graph, nref, from) {
                    if let Some(from_ref) = from {
                        if private_func && from_ref.id == nref.id {
                            return true;
                        } else if !private_func {
                            return true;
                        }
                    } else {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Can read at scope?
    pub fn can_read_scope(&mut self, graph: &SGraph, scope: &SNodeRef, from: Option<&SNodeRef>) -> bool {
        let access = self.permissions.access(graph, scope, from);
        access.can_read()
    }

    /// Can write at scope?
    pub fn can_write_scope(&mut self, graph: &SGraph, scope: &SNodeRef, from: Option<&SNodeRef>) -> bool {
        let access = self.permissions.access(graph, scope, from);
        access.can_write()
    }
}


/// Permissions.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Permissions {
    /// Locked totally?
    locked: bool,

    /// General permissions.
    pub general: ScopePermissions,

    /// Scope permissions.
    /// Each scope can have it's own permissions.
    /// Overrides general permissions.
    pub scope: HashMap<SNodeRef, ScopePermissions>,
}
impl Permissions {
    /// Merge permissions.
    pub fn merge(&mut self, other: &Self) {
        self.general.merge(&other.general);
        for (scope, perms) in &other.scope {
            if !self.scope.contains_key(scope) {
                self.scope.insert(scope.clone(), perms.clone());
            }
        }
    }

    /// Locked down permissions!
    pub fn locked() -> Self {
        let mut permissions = Self::default();
        permissions.locked = true;
        permissions
    }
    
    /// Get access for a specific scope, from a specific scope.
    pub fn access(&self, graph: &SGraph, scope: &SNodeRef, from: Option<&SNodeRef>) -> Access {
        // If locked, no access!
        if self.locked {
            return Access::None;
        }

        // Get specific scope access if able
        if let Some(from) = from {
            if let Some(from_perms) = self.scope.get(from) {
                return from_perms.access(graph, scope);
            }
            let mut id_path = from.id_path(graph);
            id_path.pop();
            id_path.reverse();
            for nref_id in id_path {
                if let Some(perms) = self.scope.get(&SNodeRef::from(nref_id)) {
                    return perms.access(graph, scope);
                }
            }
        }

        // Return general access
        self.general.access(graph, scope)
    }

    /// Set general access to a scope.
    /// This scope and anything under it (unless specified) will have this access in the runtime.
    pub fn set_general_scope_access_path(&mut self, graph: &SGraph, path: &str, start: Option<&SNodeRef>, access: Access) -> bool {
        self.general.set_scope_access_path(graph, path, start, access)
    }

    /// Set general scope access.
    pub fn set_general_scope_access(&mut self, scope: SNodeRef, access: Access) {
        self.general.set_scope_access(scope, access);
    }

    /// Set access to a scope from another scope.
    /// This scope and anything under it (unless specified) will have this access in the runtime.
    pub fn set_scope_access_path_from(&mut self, from: &SNodeRef, graph: &SGraph, path: &str, start: Option<&SNodeRef>, access: Access) -> bool {
        if let Some(from_perms) = self.scope.get_mut(from) {
            return from_perms.set_scope_access_path(graph, path, start, access);
        }
        let mut from_perms = ScopePermissions::default();
        let res = from_perms.set_scope_access_path(graph, path, start, access);
        if res {
            self.scope.insert(from.clone(), from_perms);
        }
        res
    }

    /// Set scope access from another scope.
    pub fn set_scope_access_from(&mut self, from: &SNodeRef, scope: SNodeRef, access: Access) {
        if let Some(from_perms) = self.scope.get_mut(from) {
            from_perms.set_scope_access(scope, access);
        } else {
            let mut from_perms = ScopePermissions::default();
            from_perms.set_scope_access(scope, access);
            self.scope.insert(from.clone(), from_perms);
        }
    }

    /// Set access to a scope from another scope.
    /// This scope and anything under it (unless specified) will have this access in the runtime.
    pub fn set_scope_access_path_from_path(&mut self, graph: &SGraph, from: &str, path: &str, from_start: Option<&SNodeRef>, path_start: Option<&SNodeRef>, access: Access) -> bool {
        if let Some(nref) = graph.node_ref(&from.replace('.', "/"), from_start) {
            return self.set_scope_access_path_from(&nref, graph, path, path_start, access);
        }
        false
    }

    /// Set scope access from another scope.
    pub fn set_scope_access_from_path(&mut self, graph: &SGraph, from: &str, from_start: Option<&SNodeRef>, scope: SNodeRef, access: Access) -> bool {
        if let Some(nref) = graph.node_ref(&from.replace('.', "/"), from_start) {
            self.set_scope_access_from(&nref, scope, access);
            return true;
        }
        false
    }
}


/// Scope Permissions.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ScopePermissions {
    pub scopes: HashMap<SNodeRef, Access>,
}
impl ScopePermissions {
    /// Merge scope permissions.
    pub fn merge(&mut self, other: &Self) {
        for (nref, access) in &other.scopes {
            if !self.scopes.contains_key(nref) {
                self.scopes.insert(nref.clone(), access.clone());
            }
        }
    }

    /// Add access to a scope.
    /// This scope and anything under it (unless specified) will have this access in the runtime.
    pub fn set_scope_access_path(&mut self, graph: &SGraph, path: &str, start: Option<&SNodeRef>, access: Access) -> bool {
        if let Some(nref) = graph.node_ref(&path.replace('.', "/"), start) {
            self.scopes.insert(nref, access);
            return true;
        }
        false
    }

    /// Set scope access.
    pub fn set_scope_access(&mut self, scope: SNodeRef, access: Access) {
        self.scopes.insert(scope, access);
    }

    /// Get access for a specific scope.
    pub fn access(&self, graph: &SGraph, scope: &SNodeRef) -> Access {
        if let Some(access) = self.scopes.get(scope) {
            return *access;
        }
        let mut id_path = scope.id_path(graph);
        id_path.pop(); // Already tested scope
        id_path.reverse();

        // Search for parents with access permissions (going up the graph)
        for nref_id in id_path {
            if let Some(access) = self.scopes.get(&SNodeRef::from(nref_id)) {
                return *access;
            }
        }
        Access::Write
    }
}
