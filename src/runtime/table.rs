//
// Copyright 2025 Formata, Inc. All rights reserved.
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
use crate::{model::{DataRef, Graph, NodeRef}, runtime::{Error, Val, Variable}};


#[derive(Debug, Default, Clone, Serialize, Deserialize)]
/// Symbol table for a process.
pub struct SymbolTable {
    pub scopes: Vec<Scope>,
}
impl SymbolTable {
    #[inline(always)]
    /// Push a new scope to this table.
    pub fn push(&mut self) {
        self.scopes.push(Scope::default());
    }

    #[inline(always)]
    /// Push a function scope: a boundary that lookups don't continue past (a function never sees its
    /// caller's variables).
    pub fn push_boundary(&mut self) {
        self.scopes.push(Scope { boundary: true, ..Default::default() });
    }

    /// Scopes visible from the current one, innermost first: up to and including the nearest function scope.
    fn visible_mut(&mut self) -> impl Iterator<Item = &mut Scope> {
        let start = self.scopes.iter().rposition(|scope| scope.boundary).unwrap_or(0);
        self.scopes[start..].iter_mut().rev()
    }

    #[inline(always)]
    /// Clear this table.
    pub fn clear(&mut self) {
        self.scopes.clear();
    }

    #[inline(always)]
    /// Pop a scope from this table.
    /// Returns the values of its `using` variables that still need dropping (objects, functions, data),
    /// last declared first. The caller drops them (see Base::using_drops).
    pub fn pop(&mut self) -> Vec<Val> {
        match self.scopes.pop() {
            Some(scope) => scope.owned(),
            None => Vec::new(),
        }
    }

    /// Mark a variable in the current scope as `using`: dropped when this scope ends.
    pub fn mark_using(&mut self, name: impl AsRef<str>) {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.has(name.as_ref()) {
                scope.using.push(name.as_ref().to_string());
            }
        }
    }

    /// Take every `using` value in every scope (innermost first), leaving the variables in place.
    /// Used when a process fails with an uncaught error, so its scopes are never popped.
    pub fn take_all_using(&mut self) -> Vec<Val> {
        let mut vals = Vec::new();
        for scope in self.scopes.iter_mut().rev() {
            vals.append(&mut scope.owned());
            scope.using.clear();
        }
        vals
    }

    /// Forget `using` marks without dropping anything (Ex. a spawned process's copy of its parent's table:
    /// the parent still owns those values).
    pub fn disown_using(&mut self) {
        for scope in &mut self.scopes {
            scope.using.clear();
        }
    }

    #[inline(always)]
    /// Can declare variable in the current scope?
    /// True if the name doesn't collide with a current var in the scope.
    pub fn can_declare(&self, name: impl AsRef<str>) -> bool {
        self.scopes.is_empty() || !self.scopes.last().unwrap().has(name)
    }

    #[inline]
    /// Insert a variable into this scope.
    /// Will create a new scope if one doesn't exist.
    pub fn insert(&mut self, name: impl ToString, var: Variable) {
        if self.scopes.is_empty() { self.push(); }
        self.scopes.last_mut().unwrap().insert(name, var);
    }

    /// Remove a variable from this symbol table.
    /// Will only drop one if multiple exist (closest).
    pub fn drop_var(&mut self, name: impl AsRef<str>) -> Option<Variable> {
        let name = name.as_ref();
        for scope in self.visible_mut() {
            if let Some(var) = scope.remove(name) {
                return Some(var);
            }
        }
        None
    }

    /// Get a variable from this symbol table.
    /// Will find the closest if it exists.
    pub fn get(&self, name: impl AsRef<str>) -> Option<&Variable> {
        let name = name.as_ref();
        let start = self.scopes.iter().rposition(|scope| scope.boundary).unwrap_or(0);
        for scope in self.scopes[start..].iter().rev() {
            if let Some(var) = scope.get(name) {
                return Some(var);
            }
        }
        None
    }

    /// Set an existing variable in this symbol table.
    /// Will return an error if the var exists but is const.
    pub fn set(&mut self, name: impl AsRef<str>, var: &Variable, graph: &mut Graph, context: Option<NodeRef>) -> Result<bool, Error> {
        let name = name.as_ref();
        for scope in self.visible_mut() {
            if scope.set(name, var, graph, context.clone())? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    #[inline]
    /// Garbage collect variables in this symbol table that reference a node.
    pub fn gc_node(&mut self, node: &NodeRef) {
        for scope in &mut self.scopes {
            scope.gc_node(node);
        }
    }

    #[inline]
    /// Garbage collect variables in this symbol table that reference data.
    pub fn gc_data(&mut self, data: &DataRef) {
        for scope in &mut self.scopes {
            scope.gc_data(data);
        }
    }
}


#[derive(Debug, Default, Clone, Serialize, Deserialize)]
/// Symbol table scope.
pub struct Scope {
    variables: FxHashMap<String, Variable>,

    /// Function scope: lookups stop here (no dynamic scoping into the caller).
    #[serde(default)]
    pub boundary: bool,

    /// Names of `using` variables in this scope, in declaration order: dropped when the scope ends.
    #[serde(default)]
    pub using: Vec<String>,
}
impl Scope {
    /// Values of this scope's `using` variables that hold something droppable, last declared first.
    /// A variable that was already dropped (or removed) is skipped.
    fn owned(&self) -> Vec<Val> {
        let mut vals = Vec::new();
        for name in self.using.iter().rev() {
            if let Some(var) = self.variables.get(name) {
                let val = var.val.read().clone();
                match &val {
                    Val::Obj(_) | Val::Fn(_) | Val::Data(_) => vals.push(val),
                    _ => {}
                }
            }
        }
        vals
    }

    #[inline(always)]
    /// Has a variable with this name?
    pub fn has(&self, name: impl AsRef<str>) -> bool {
        self.variables.contains_key(name.as_ref())
    }
    
    #[inline(always)]
    /// Insert a variable.
    pub fn insert(&mut self, name: impl ToString, var: Variable) {
        self.variables.insert(name.to_string(), var);
    }

    #[inline(always)]
    /// Remove a variable.
    pub fn remove(&mut self, name: impl AsRef<str>) -> Option<Variable> {
        self.variables.remove(name.as_ref())
    }

    #[inline(always)]
    /// Get a variable.
    pub fn get(&self, name: impl AsRef<str>) -> Option<&Variable> {
        self.variables.get(name.as_ref())
    }

    #[inline(always)]
    /// Get a mutable variable.
    pub fn get_mut(&mut self, name: impl AsRef<str>) -> Option<&mut Variable> {
        self.variables.get_mut(name.as_ref())
    }

    #[inline]
    /// Set a variable.
    pub fn set(&mut self, name: impl AsRef<str>, var: &Variable, graph: &mut Graph, context: Option<NodeRef>) -> Result<bool, Error> {
        if let Some(svar) = self.get_mut(name) {
            svar.set(var, graph, context)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Garbage collect variables in this symbol table that reference a node.
    pub fn gc_node(&mut self, node: &NodeRef) {
        let mut to_remove = Vec::new();
        for (name, var) in &self.variables {
            if let Some(nref) = var.try_obj() {
                if &nref == node {
                    to_remove.push(name.clone());
                }
            }
        }
        for name in to_remove {
            self.variables.remove(&name);
        }
    }

    /// Garbage collect variables in this symbol table that reference data.
    pub fn gc_data(&mut self, data: &DataRef) {
        let mut to_remove = Vec::new();
        for (name, var) in &self.variables {
            if var.is_data_ref(data) {
                to_remove.push(name.clone());
            }
        }
        for name in to_remove {
            self.variables.remove(&name);
        }
    }
}
