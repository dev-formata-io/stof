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
use nanoid::nanoid;
use crate::SVal;
use super::{IntoDataRef, SDataRef, SNodeRef, Symbol, SymbolTable};


/// Stof processes.
#[derive(Debug, Clone)]
pub struct SProcesses {
    pub processes: BTreeMap<String, SProcess>,
}
impl Default for SProcesses {
    fn default() -> Self {
        Self::new()
    }
}
impl SProcesses {
    /// Create a new block of preccesses.
    pub fn new() -> Self {
        let mut processes = BTreeMap::new();
        processes.insert("main".to_string(), SProcess::new("main"));
        Self {
            processes
        }
    }

    /// Spawn a new process.
    pub fn spawn(&mut self) -> String {
        let pid = nanoid!();
        let process = SProcess::new(&pid);
        self.processes.insert(pid.clone(), process);
        pid
    }

    /// Kill a process.
    pub fn kill(&mut self, pid: &str) {
        self.processes.remove(pid);
    }

    /// Get a process.
    pub fn get(&self, pid: &str) -> Option<&SProcess> {
        self.processes.get(pid)
    }

    /// Get a mutable process.
    pub fn get_mut(&mut self, pid: &str) -> Option<&mut SProcess> {
        self.processes.get_mut(pid)
    }

    /// Get the main process.
    pub fn main(&self) -> Option<&SProcess> {
        self.get("main")
    }

    /// Get the mutable main process.
    pub fn main_mut(&mut self) -> Option<&mut SProcess> {
        self.get_mut("main")
    }
}

/// Stof Process.
#[derive(Debug, Clone)]
pub struct SProcess {
    pub pid: String,
    pub self_stack: Vec<SNodeRef>,
    pub stack: Vec<SVal>,
    pub table: SymbolTable,
    pub call_stack: Vec<SDataRef>,
    pub bubble_control_flow: u8,
}
impl SProcess {
    /// Create a new process with an id.
    pub fn new(id: &str) -> Self {
        Self {
            pid: id.to_owned(),
            self_stack: Default::default(),
            stack: Default::default(),
            table: Default::default(),
            call_stack: Default::default(),
            bubble_control_flow: 0,
        }
    }

    /// Self pointer.
    pub fn self_ptr(&self) -> Option<SNodeRef> {
        if let Some(last) = self.self_stack.last() {
            return Some(last.clone());
        }
        None
    }

    /// New table.
    /// Returns the current table, replacing it with a new one.
    /// This happens for function calls.
    pub fn new_table(&mut self) -> SymbolTable {
        let current = self.table.clone();
        self.table = SymbolTable::default();
        return current;
    }

    /// Set table.
    pub fn set_table(&mut self, table: SymbolTable) {
        self.table = table;
    }

    /// Push to call stack.
    /// These are function references that get pushed to the stack.
    pub fn push_call_stack(&mut self, dref: impl IntoDataRef) {
        self.call_stack.push(dref.data_ref());
    }

    /// Pop call stack.
    pub fn pop_call_stack(&mut self) {
        self.call_stack.pop();
    }

    /// Add a variable to the current scope.
    pub fn add_variable<T>(&mut self, name: &str, value: T) where T: Into<SVal> {
        let symbol = Symbol::Variable(value.into());
        self.table.insert(name, symbol);
    }

    /// Set a variable.
    /// Will not add the variable if not already present.
    /// Sets current scope or above variables!
    pub fn set_variable<T>(&mut self, name: &str, value: T) -> bool where T: Into<SVal> {
        self.table.set_variable(name, &value.into())
    }

    /// Drop a symbol from the current scope.
    pub fn drop(&mut self, name: &str) -> Option<Symbol> {
        self.table.remove(name)
    }

    /// Get a symbol from the current scope or above.
    pub fn get_symbol(&mut self, name: &str) -> Option<&Symbol> {
        self.table.get(name)
    }

    /// Has a symbol from the current scope or above.
    pub fn has_symbol(&mut self, name: &str) -> bool {
        self.table.get(name).is_some()
    }

    /// Push a value onto the stack.
    pub fn push<T>(&mut self, value: T) where T: Into<SVal> {
        let val: SVal = value.into();
        if !val.is_void() { // Prevent void from being pushed to the stack!
            self.stack.push(val);
        }
    }

    /// Pop a value from the stack.
    pub fn pop(&mut self) -> Option<SVal> {
        self.stack.pop()
    }

    /// Clean for scripting.
    pub fn clean(&mut self) {
        self.stack.clear();
        self.table = Default::default();
        self.self_stack.clear();
        self.bubble_control_flow = 0;
    }
}
