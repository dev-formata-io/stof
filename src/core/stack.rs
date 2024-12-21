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

use crate::SVal;
use super::{SNodeRef, Symbol, SymbolTable};


/// SDoc Runtime Stack.
#[derive(Debug, Clone, Default)]
pub struct SStack {
    pub self_stack: Vec<SNodeRef>,
    pub stack: Vec<SVal>,
    pub table: SymbolTable,
    pub bubble_control_flow: u8,
}
impl SStack {
    /// Self pointer.
    pub(crate) fn self_ptr(&self) -> Option<SNodeRef> {
        if let Some(last) = self.self_stack.last() {
            return Some(last.clone());
        }
        None
    }

    /// New table.
    /// Returns the current table, replacing it with a new one.
    /// This happens for function calls.
    pub(crate) fn new_table(&mut self) -> SymbolTable {
        let current = self.table.clone();
        self.table = SymbolTable::default();
        return current;
    }

    /// Set table.
    pub(crate) fn set_table(&mut self, table: SymbolTable) {
        self.table = table;
    }

    /// Add a variable to the current scope.
    pub(crate) fn add_variable<T>(&mut self, name: &str, value: T) where T: Into<SVal> {
        let symbol = Symbol::Variable(value.into());
        self.table.insert(name, symbol);
    }

    /// Set a variable.
    /// Will not add the variable if not already present.
    /// Sets current scope or above variables!
    pub(crate) fn set_variable<T>(&mut self, name: &str, value: T) -> bool where T: Into<SVal> {
        self.table.set_variable(name, &value.into())
    }

    /// Drop a symbol from the current scope.
    pub(crate) fn drop(&mut self, name: &str) -> Option<Symbol> {
        self.table.remove(name)
    }

    /// Get a symbol from the current scope or above.
    pub(crate) fn get_symbol(&mut self, name: &str) -> Option<&Symbol> {
        self.table.get(name)
    }

    /// Has a symbol from the current scope or above.
    pub(crate) fn has_symbol(&mut self, name: &str) -> bool {
        self.table.get(name).is_some()
    }

    /// Push a value onto the stack.
    pub(crate) fn push<T>(&mut self, value: T) where T: Into<SVal> {
        let val: SVal = value.into();
        if !val.is_void() { // Prevent void from being pushed to the stack!
            self.stack.push(val);
        }
    }

    /// Pop a value from the stack.
    pub(crate) fn pop(&mut self) -> Option<SVal> {
        self.stack.pop()
    }

    /// Clean for scripting.
    pub(crate) fn clean(&mut self) {
        self.stack.clear();
        self.table = Default::default();
        self.self_stack.clear();
        self.bubble_control_flow = 0;
    }
}
