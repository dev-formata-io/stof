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
use crate::SVal;


/// Scope of symbols.
#[derive(Debug, Default, Clone)]
pub struct SymbolScope {
    symbols: FxHashMap<String, Symbol>,
}
impl SymbolScope {
    /// Insert a symbol.
    pub fn insert(&mut self, name: &str, symbol: Symbol) {
        self.symbols.insert(name.to_owned(), symbol);
    }

    /// Remove a symbol.
    pub fn remove(&mut self, name: &str) -> Option<Symbol> {
        self.symbols.remove(name)
    }

    /// Get a symbol.
    pub fn get(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    /// Get a mutable symbol.
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        self.symbols.get_mut(name)
    }

    /// Set a variable by name.
    /// Will not insert if not present!
    pub fn set_variable(&mut self, name: &str, value: &SVal, force: bool) -> Result<bool, String> {
        if let Some(var) = self.get_mut(name) {
            var.set(value.clone(), force)?;
            return Ok(true);
        }
        Ok(false)
    }
}


/// Symbol.
#[derive(Debug, Clone)]
pub enum Symbol {
    Variable((bool, SVal)), // const, value
}
impl Symbol {
    /// Get variable value.
    pub fn var(&self) -> SVal {
        match self {
            Symbol::Variable(val) => val.1.clone(),
        }
    }

    /// Set variable.
    pub fn set(&mut self, val: SVal, force: bool) -> Result<(), String> {
        match self {
            Symbol::Variable((is_const, var)) => {
                if !force && *is_const {
                    return Err(format!("cannot set const variable"));
                }
                
                if var.is_boxed() && !val.is_boxed() { // set for everyone!
                    match var {
                        SVal::Boxed(var) => {
                            *var.lock().unwrap() = val;
                        },
                        _ => {}
                    }
                } else { // if val is boxed, we are setting just this var
                    *var = val;
                }
            }
        }
        Ok(())
    }
}

/// Symbol table.
/// This is where the current call scope exists.
#[derive(Debug, Clone)]
pub struct SymbolTable {
    scopes: FxHashMap<i32, SymbolScope>,
    scope: i32,
}
impl Default for SymbolTable {
    fn default() -> Self {
        let mut table = Self {
            scope: 0,
            scopes: Default::default(),
        };
        table.scopes.insert(0, SymbolScope::default());
        table
    }
}
impl SymbolTable {
    /// Add a new scope!
    pub fn new_scope(&mut self) {
        self.scope += 1;
        self.scopes.insert(self.scope, SymbolScope::default());
    }

    /// Current scope.
    pub fn current(&mut self) -> &mut SymbolScope {
        self.scopes.get_mut(&self.scope).expect("No current scope!")
    }

    /// End scope.
    pub fn end_scope(&mut self) {
        if self.scope > 0 {
            self.scopes.remove(&self.scope);
            self.scope -= 1;
        }
    }

    /// Insert a symbol into the current scope.
    pub fn insert(&mut self, name: &str, symbol: Symbol) {
        self.current().insert(name, symbol);
    }

    /// Remove a symbol from the current scope.
    pub fn remove(&mut self, name: &str) -> Option<Symbol> {
        self.current().remove(name)
    }

    /// Has a symbol with this name in the current scope?
    pub fn has_in_current(&mut self, name: &str) -> bool {
        self.current().get(name).is_some()
    }

    /// Get a symbol from the current scope or above.
    pub fn get(&mut self, name: &str) -> Option<&Symbol> {
        let mut curr = self.scope;
        while curr >= 0 {
            let scope = self.scopes.get(&curr).expect("No scope!");
            if let Some(symbol) = scope.get(name) {
                return Some(symbol);
            }
            curr -= 1;
        }
        None
    }

    /// Set a variable by name.
    /// Will not insert if not present!
    pub fn set_variable(&mut self, name: &str, value: &SVal, force: bool) -> Result<bool, String> {
        for i in (0..self.scope + 1).rev() {
            if let Some(scope) = self.scopes.get_mut(&i) {
                if scope.set_variable(name, value, force)? {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}
