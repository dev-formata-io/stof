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

use std::{collections::HashSet, ops::Deref};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use crate::{Data, SDoc, SField, SVal, SFunc};
use super::Expr;


/// Statements result enum.
pub enum StatementsRes {
    None,
    Return(bool),
    Break,
    Continue,
}

/// Statements.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Statements {
    pub statements: Vec<Statement>,
}
impl Statements {
    /// Push a statement.
    pub fn push(&mut self, statement: Statement) {
        self.statements.push(statement);
    }

    /// Absorb statements from another.
    pub fn absorb(&mut self, mut statements: Self) {
        self.statements.append(&mut statements.statements);
    }

    /// Execute these statements with the given doc.
    pub fn exec(&self, doc: &mut SDoc) -> Result<StatementsRes> {
        for statement in &self.statements {
            match statement {
                Statement::Declare(name, rhs) => {
                    // Check to see if the symbol already exists in the current scope
                    if let Some(_) = doc.table.current().get(&name) {
                        return Err(anyhow!("Attempting to declare a variable that already exists in the same scope: {}", &name));
                    }

                    // Eval rhs, which is the value of the new variable
                    let val = rhs.exec(doc)?;
                    if val.is_void() {
                        return Err(anyhow!("Cannot declare void variable"));
                    }

                    // Do not allow fields to be set on declare!
                    if name.contains('.') {
                        return Err(anyhow!("Cannot declare variables that are paths. If you're setting fields, drop the 'let' keyword."));
                    } else {
                        doc.add_variable(&name, val);
                    }
                },
                Statement::Assign(name, rhs) => {
                    // Eval rhs, which is the value of the variable!
                    let val = rhs.exec(doc)?;
                    if val.is_void() {
                        return Err(anyhow!("Cannot assign void"));
                    }

                    // Try setting a variable in the symbol table
                    let mut set_var = false;
                    if doc.set_variable(&name, val.clone()) {
                        set_var = true;
                    }

                    if !set_var && name.contains('.') && !name.ends_with('.') && !name.starts_with('.') {
                        // Try assigning via an absolute path to a field first
                        if let Some(mut field) = SField::field(&doc.graph, &name, '.', None) {
                            if doc.perms.can_write_field(&doc.graph, &field, doc.self_ptr().as_ref()) {
                                field.value = val;
                                field.set(&mut doc.graph);
                            }
                            return Ok(StatementsRes::None); // Go no further!
                        }

                        // Look for a variable that is the first token in the path next
                        let mut path = name.split('.').collect::<Vec<&str>>();
                        let mut context = None;
                        let mut context_path = name.clone();
                        if let Some(symbol) = doc.get_symbol(path.remove(0)) {
                            match symbol.var() {
                                SVal::Ref(rf) => {
                                    let refval = rf.read().unwrap();
                                    let val = refval.deref();
                                    match val {
                                        SVal::Object(nref) => {
                                            context = Some(nref.clone());
                                            context_path = path.join(".");
                                        },
                                        _ => {}
                                    }
                                },
                                SVal::Object(nref) => {
                                    context = Some(nref);
                                    context_path = path.join(".");
                                },
                                _ => {}
                            }
                        }
                        // If no variable matches, try setting self scope
                        else if name.starts_with("self") || name.starts_with("super") {
                            context = doc.self_ptr();
                        }

                        if let Some(mut context) = context {
                            let mut set = false;

                            // Already defined field?
                            if let Some(mut field) = SField::field(&doc.graph, &context_path, '.', Some(&context)) {
                                if doc.perms.can_write_field(&doc.graph, &field, doc.self_ptr().as_ref()) {
                                    field.value = val.clone();
                                    field.set(&mut doc.graph);
                                    set = true;
                                }
                            }
                            // Creating a new field
                            else {
                                if doc.perms.can_write_scope(&doc.graph, &context, doc.self_ptr().as_ref()) {
                                    let mut new_field_path = context_path.split('.').collect::<Vec<&str>>();

                                    let field_name = new_field_path.pop().unwrap();
                                    if new_field_path.len() > 0 {
                                        context = doc.graph.ensure_nodes(&new_field_path.join("/"), '/', true, Some(context.clone()));
                                    }

                                    let mut field = SField::new(field_name, val.clone());
                                    field.attach(&context, &mut doc.graph);
                                    set = true;
                                }
                            }

                            // If val is an object, move it to context destination by default and rename to field name (usually, this is what folks want)
                            if set {
                                match &val {
                                    SVal::Object(source) => {
                                        // New name for the source node
                                        let new_name = context_path.split('.').collect::<Vec<&str>>().pop().unwrap();
                                        let mut old_name = String::default();
                                        if let Some(source_node) = source.node(&doc.graph) {
                                            old_name = source_node.name.clone();
                                        }

                                        let mut source_ident = source.path(&doc.graph);
                                        source_ident = source_ident.replace('/', "."); // Make a variable

                                        let mut destination_ident = context.path(&doc.graph);
                                        destination_ident = destination_ident.replace('/', ".");

                                        let mut statements_vec = vec![Statement::Move(source_ident.into(), destination_ident.clone().into())];
                                        if old_name.len() > 0 {
                                            let new_location = format!("{}.{}", destination_ident, old_name);
                                            statements_vec.push(Statement::Rename(new_location.into(), Expr::Literal(SVal::String(new_name.to_owned()))));
                                        }

                                        let statements = Statements::from(statements_vec);
                                        statements.exec(doc)?;
                                    },
                                    SVal::Array(vals) => {
                                        let mut statement_vec = Vec::new();

                                        for val in vals {
                                            match val {
                                                SVal::Object(source) => {
                                                    let mut source_ident = source.path(&doc.graph);
                                                    source_ident = source_ident.replace('/', ".");

                                                    let mut destination_ident = context.path(&doc.graph);
                                                    destination_ident = destination_ident.replace('/', ".");

                                                    statement_vec.push(Statement::Move(source_ident.into(), destination_ident.into()));
                                                },
                                                _ => {}
                                            }
                                        }

                                        if statement_vec.len() > 0 {
                                            let statements = Statements::from(statement_vec);
                                            statements.exec(doc)?;
                                        }
                                    },
                                    _ => {}
                                }
                            }
                        } else {
                            // Create a new object and add a field to it
                            let mut obj_path = name.split(".").collect::<Vec<&str>>();
                            let backup_name = obj_path.pop().unwrap();
                            if obj_path.len() > 0 {
                                let nref = doc.graph.ensure_nodes(&obj_path.join("/"), '/', true, None);
                                if doc.perms.can_write_scope(&doc.graph, &nref, doc.self_ptr().as_ref()) {
                                    let mut field = SField::new(backup_name, val);
                                    field.attach(&nref, &mut doc.graph); // attach new field to self
                                }
                            }
                        }
                    } else if !set_var && !name.contains('.') && !name.ends_with('.') && !name.starts_with('.') {
                        // Assigning a root object!
                        match val {
                            SVal::Object(nref) => {
                                if let Some(existing) = doc.graph.root_by_name(&name) {
                                    doc.graph.remove_root(&existing.id);
                                }
                                if let Some(node) = doc.graph.node_mut(nref.clone()) {
                                    node.name = name.clone();
                                }
                                doc.graph.roots.push(nref);
                            },
                            _ => return Err(anyhow!("Cannot assign anything but an object as a root"))
                        }
                    }
                },
                Statement::Drop(name) => {
                    // Try dropping a variable in the symbol table by name
                    let mut dropped_var = false;
                    if let Some(symbol) = doc.drop(&name) {
                        let dropped_val = symbol.var();
                        match &dropped_val {
                            SVal::Object(nref) => {
                                if !doc.perms.can_write_scope(&doc.graph, &nref, doc.self_ptr().as_ref()) {
                                    doc.add_variable(&name, dropped_val);
                                } else {
                                    doc.graph.remove_node(nref);
                                }
                            },
                            SVal::FnPtr(dref) => {
                                let func: SFunc = dref.data(&doc.graph).unwrap().get_value().unwrap();
                                if !doc.perms.can_write_func(&doc.graph, &func, doc.self_ptr().as_ref()) {
                                    doc.add_variable(&name, dropped_val);
                                } else {
                                    func.remove(&mut doc.graph, None);
                                }
                            },
                            _ => {}
                        }
                        dropped_var = true;
                    }

                    if !dropped_var {
                        if let Some(field) = SField::field(&doc.graph, &name, '.', None) {
                            if doc.perms.can_write_field(&doc.graph, &field, doc.self_ptr().as_ref()) {
                                field.remove(&mut doc.graph, None);
                                match &field.value {
                                    SVal::Object(nref) => {
                                        doc.graph.remove_node(nref);
                                    },
                                    SVal::Array(vec) => {
                                        for val in vec {
                                            match val {
                                                SVal::Object(nref) => {
                                                    doc.graph.remove_node(nref);
                                                },
                                                _ => {}
                                            }
                                        }
                                    },
                                    _ => {}
                                }
                            }
                        } else if let Some(func) = SFunc::func(&doc.graph, &name, '.', None) {
                            if doc.perms.can_write_func(&doc.graph, &func, doc.self_ptr().as_ref()) {
                                func.remove(&mut doc.graph, None);
                            }
                        } else if let Some(node) = doc.graph.node_ref(&name.replace(".", "/"), None) {
                            if doc.perms.can_write_scope(&doc.graph, &node, doc.self_ptr().as_ref()) {
                                doc.graph.remove_node(&node);
                            }
                        } else {
                            // Get the object we are operating on, if any
                            let mut context_ptr = None;
                            let mut context_path = name.clone();
                            if name == "self" || name == "super" {
                                context_ptr = doc.self_ptr();
                                context_path = String::default();
                            } else if name.starts_with("self") || name.starts_with("super") {
                                context_ptr = doc.self_ptr();
                            } else {
                                let mut path = name.split(".").collect::<Vec<&str>>();
                                if path.len() > 0 {
                                    let var = path.remove(0);
                                    if let Some(symbol) = doc.get_symbol(var) {
                                        let var = symbol.var();
                                        match var {
                                            SVal::Ref(rf) => {
                                                let refval = rf.read().unwrap();
                                                let val = refval.deref();
                                                match val {
                                                    SVal::Object(nref) => {
                                                        context_ptr = Some(nref.clone());
                                                        context_path = path.join(".");
                                                    },
                                                    _ => {}
                                                }
                                            },
                                            SVal::Object(nref) => {
                                                context_ptr = Some(nref);
                                                context_path = path.join(".");
                                            },
                                            _ => {}
                                        }
                                    }
                                }
                            }

                            if let Some(context) = context_ptr {
                                if context_path.len() < 1 {
                                    // We are deleting the context itself!
                                    // Have to delete the object field on parent if any and the context object
                                    if doc.perms.can_write_scope(&doc.graph, &context, doc.self_ptr().as_ref()) {
                                        let mut object_name = String::default();
                                        if let Some(context_node) = context.node(&doc.graph) {
                                            object_name = context_node.name.clone();
                                        }
                                        if object_name.len() > 0 {
                                            if let Some(field) = SField::field(&doc.graph, &format!("super.{}", object_name), '.', Some(&context)) {
                                                field.remove(&mut doc.graph, None);
                                            }
                                        }
                                        doc.graph.remove_node(&context);
                                    }
                                } else if let Some(field) = SField::field(&doc.graph, &context_path, '.', Some(&context)) {
                                    if doc.perms.can_write_field(&doc.graph, &field, doc.self_ptr().as_ref()) {
                                        field.remove(&mut doc.graph, None);
                                        match &field.value {
                                            SVal::Object(nref) => {
                                                doc.graph.remove_node(nref);
                                            },
                                            SVal::Array(vec) => {
                                                for val in vec {
                                                    match val {
                                                        SVal::Object(nref) => {
                                                            doc.graph.remove_node(nref);
                                                        },
                                                        _ => {}
                                                    }
                                                }
                                            },
                                            _ => {}
                                        }
                                    }
                                } else if let Some(func) = SFunc::func(&doc.graph, &context_path, '.', Some(&context)) {
                                    if doc.perms.can_write_func(&doc.graph, &func, doc.self_ptr().as_ref()) {
                                        func.remove(&mut doc.graph, None);
                                    }
                                }
                            }
                        }
                    }
                },
                Statement::Move(name, dest) => {
                    let mut destination = doc.self_ptr().expect("Failed to find a self pointer on the stack for move");
                    if dest.len() > 0 {
                        let dest_val = Expr::Variable(dest.clone()).exec(doc)?;
                        match dest_val {
                            SVal::Ref(rf) => {
                                let refval = rf.read().unwrap();
                                let val = refval.deref();
                                match val {
                                    SVal::Object(node) => {
                                        destination = node.clone();
                                    },
                                    _ => {
                                        return Err(anyhow!("Cannot move into anything but an object (from ref)"));
                                    }
                                }
                            },
                            SVal::Object(node) => {
                                destination = node;
                            },
                            SVal::Null => {
                                // Create the destination nodes!
                                if dest.starts_with("self.") {
                                    let path = destination.path(&doc.graph);
                                    let id = dest.replace("self", "").replace("..", ".").replace(".", "/");
                                    destination = doc.graph.ensure_nodes(&format!("{}{}", path, id), '.', true, None);
                                } else {
                                    let id = dest.replace(".", "/");
                                    destination = doc.graph.ensure_nodes(&id, '.', true, None);
                                }
                            },
                            _ => {
                                return Err(anyhow!("Cannot move into anything but an object"));
                            }
                        }
                    }

                    if let Some(mut field) = SField::field(&doc.graph, &name, '.', None) {
                        if doc.perms.can_write_field(&doc.graph, &field, doc.self_ptr().as_ref()) {
                            // Move the field
                            field.remove(&mut doc.graph, None);
                            field.attach(&destination, &mut doc.graph);

                            // If its an object, move the object too
                            match &field.value {
                                SVal::Object(node) => {
                                    let id_path: HashSet<String> = HashSet::from_iter(node.id_path(&doc.graph).into_iter());
                                    if !id_path.contains(&destination.id) && !destination.is_child_of(&doc.graph, &node) {
                                        doc.graph.move_node(node, &destination);
                                    }
                                },
                                // TODO: arrays in arrays
                                SVal::Array(values) => {
                                    for value in values {
                                        match value {
                                            SVal::Object(node) => {
                                                let id_path: HashSet<String> = HashSet::from_iter(node.id_path(&doc.graph).into_iter());
                                                if !id_path.contains(&destination.id) && !destination.is_child_of(&doc.graph, &node) {
                                                    doc.graph.move_node(node, &destination);
                                                }
                                            },
                                            _ => {}
                                        }
                                    }
                                },
                                _ => {}
                            }
                        }
                    } else if let Some(mut func) = SFunc::func(&doc.graph, &name, '.', None) {
                        if doc.perms.can_write_func(&doc.graph, &func, doc.self_ptr().as_ref()) {
                            func.remove(&mut doc.graph, None);
                            func.attach(&destination, &mut doc.graph);
                        }
                    } else if let Some(node) = doc.graph.node_ref(&name.replace(".", "/"), None) {
                        if doc.perms.can_write_scope(&doc.graph, &node, doc.self_ptr().as_ref()) {
                            let id_path: HashSet<String> = HashSet::from_iter(node.id_path(&doc.graph).into_iter());
                            if !id_path.contains(&destination.id) && !destination.is_child_of(&doc.graph, &node) {
                                doc.graph.move_node(&node, &destination);
                            }
                        }
                    } else {
                        // Get the object we are operating on, if any
                        let mut context_ptr = None;
                        let mut context_path = name.clone();
                        if name == "self" || name == "super" {
                            context_ptr = doc.self_ptr();
                            context_path = String::default();
                        } else if name.starts_with("self") || name.starts_with("super") {
                            context_ptr = doc.self_ptr();
                        } else {
                            let mut path = name.split(".").collect::<Vec<&str>>();
                            if path.len() > 0 {
                                let var = path.remove(0);
                                if let Some(symbol) = doc.get_symbol(var) {
                                    match symbol.var() {
                                        SVal::Ref(rf) => {
                                            let refval = rf.read().unwrap();
                                            let val = refval.deref();
                                            match val {
                                                SVal::Object(nref) => {
                                                    context_ptr = Some(nref.clone());
                                                    context_path = path.join(".");
                                                },
                                                _ => {}
                                            }
                                        },
                                        SVal::Object(nref) => {
                                            context_ptr = Some(nref);
                                            context_path = path.join(".");
                                        },
                                        _ => {}
                                    }
                                }
                            }
                        }

                        if let Some(context) = context_ptr {
                            if context_path.len() < 1 {
                                // We are moving the context itself!
                                // Have to move the object field on parent if any and the context object
                                if doc.perms.can_write_scope(&doc.graph, &context, doc.self_ptr().as_ref()) {
                                    let mut object_name = String::default();
                                    if let Some(context_node) = context.node(&doc.graph) {
                                        object_name = context_node.name.clone();
                                    }
                                    if object_name.len() > 0 {
                                        if let Some(mut field) = SField::field(&doc.graph, &format!("super.{}", object_name), '.', Some(&context)) {
                                            field.remove(&mut doc.graph, None);
                                            field.attach(&destination, &mut doc.graph);
                                        }
                                    }

                                    let id_path: HashSet<String> = HashSet::from_iter(context.id_path(&doc.graph).into_iter());
                                    if !id_path.contains(&destination.id) && !destination.is_child_of(&doc.graph, &context) {
                                        doc.graph.move_node(&context, &destination);
                                    }
                                }
                            } else if let Some(mut field) = SField::field(&doc.graph, &context_path, '.', Some(&context)) {
                                if doc.perms.can_write_field(&doc.graph, &field, doc.self_ptr().as_ref()) {
                                    field.remove(&mut doc.graph, None);
                                    field.attach(&destination, &mut doc.graph);

                                    match &field.value {
                                        SVal::Object(node) => {
                                            let id_path: HashSet<String> = HashSet::from_iter(node.id_path(&doc.graph).into_iter());
                                            if !id_path.contains(&destination.id) && !destination.is_child_of(&doc.graph, &node) {
                                                doc.graph.move_node(node, &destination);
                                            }
                                        },
                                        // TODO: arrays in arrays
                                        SVal::Array(values) => {
                                            for value in values {
                                                match value {
                                                    SVal::Object(node) => {
                                                        let id_path: HashSet<String> = HashSet::from_iter(node.id_path(&doc.graph).into_iter());
                                                        if !id_path.contains(&destination.id) && !destination.is_child_of(&doc.graph, &node) {
                                                            doc.graph.move_node(node, &destination);
                                                        }
                                                    },
                                                    _ => {}
                                                }
                                            }
                                        },
                                        _ => {}
                                    }
                                }
                            } else if let Some(mut func) = SFunc::func(&doc.graph, &context_path, '.', Some(&context)) {
                                if doc.perms.can_write_func(&doc.graph, &func, doc.self_ptr().as_ref()) {
                                    func.remove(&mut doc.graph, None);
                                    func.attach(&destination, &mut doc.graph);
                                }
                            }
                        }
                    }
                },
                Statement::Rename(name, new_name) => {
                    let name_res = new_name.exec(doc)?;
                    let new_name_val;
                    match name_res {
                        SVal::String(v) => {
                            new_name_val = v;
                        },
                        SVal::Ref(rf) => {
                            let refval = rf.read().unwrap();
                            let val = refval.deref();
                            match val {
                                SVal::String(v) => {
                                    new_name_val = v.clone();
                                },
                                _ => return Err(anyhow!("Cannot rename to a non-string value"))
                            }
                        },
                        _ => return Err(anyhow!("Cannot rename to a non-string value"))
                    }

                    if name.contains('.') && !name.ends_with('.') && !name.starts_with('.') {
                        if let Some(mut field) = SField::field(&doc.graph, &name, '.', None) {
                            if doc.perms.can_write_field(&doc.graph, &field, doc.self_ptr().as_ref()) {
                                match &field.value {
                                    SVal::Object(node) => {
                                        doc.graph.rename_node(node, &new_name_val);
                                    },
                                    _ => {}
                                }
                                
                                field.name = new_name_val.clone();
                                field.set(&mut doc.graph);
                            }
                        } else if let Some(mut func) = SFunc::func(&doc.graph, &name, '.', None) {
                            if doc.perms.can_write_func(&doc.graph, &func, doc.self_ptr().as_ref()) {
                                func.name = new_name_val.clone();
                                func.set(&mut doc.graph);
                            }
                        } else if let Some(node) = doc.graph.node_ref(&name.replace(".", "/"), None) {
                            if doc.perms.can_write_scope(&doc.graph, &node, doc.self_ptr().as_ref()) {
                                doc.graph.rename_node(&node, &new_name_val);
                            }
                        } else {
                            // Get the object we are operating on, if any
                            let mut context_ptr = None;
                            let mut context_path = name.clone();
                            if name == "self" || name == "super" {
                                context_ptr = doc.self_ptr();
                                context_path = String::default();
                            } else if name.starts_with("self") || name.starts_with("super") {
                                context_ptr = doc.self_ptr();
                            } else {
                                let mut path = name.split(".").collect::<Vec<&str>>();
                                if path.len() > 0 {
                                    let var = path.remove(0);
                                    if let Some(symbol) = doc.get_symbol(var) {
                                        match symbol.var() {
                                            SVal::Ref(rf) => {
                                                let refval = rf.read().unwrap();
                                                let val = refval.deref();
                                                match val {
                                                    SVal::Object(nref) => {
                                                        context_ptr = Some(nref.clone());
                                                        context_path = path.join(".");
                                                    },
                                                    _ => {}
                                                }
                                            },
                                            SVal::Object(nref) => {
                                                context_ptr = Some(nref);
                                                context_path = path.join(".");
                                            },
                                            _ => {}
                                        }
                                    }
                                }
                            }

                            if let Some(context) = context_ptr {
                                if context_path.len() < 1 {
                                    // We are renaming the context itself!
                                    // Have to rename the object field on parent if any
                                    if doc.perms.can_write_scope(&doc.graph, &context, doc.self_ptr().as_ref()) {
                                        let mut object_name = String::default();
                                        if let Some(context_node) = context.node(&doc.graph) {
                                            object_name = context_node.name.clone();
                                        }
                                        if object_name.len() > 0 {
                                            if let Some(mut field) = SField::field(&doc.graph, &format!("super.{}", object_name), '.', Some(&context)) {
                                                match &field.value {
                                                    SVal::Object(node) => {
                                                        doc.graph.rename_node(node, &new_name_val);
                                                    },
                                                    _ => {}
                                                }
                                                
                                                field.name = new_name_val.clone();
                                                field.set(&mut doc.graph);
                                            }
                                        }
                                    }
                                } else if let Some(mut field) = SField::field(&doc.graph, &context_path, '.', Some(&context)) {
                                    if doc.perms.can_write_field(&doc.graph, &field, doc.self_ptr().as_ref()) {
                                        match &field.value {
                                            SVal::Object(node) => {
                                                doc.graph.rename_node(node, &new_name_val);
                                            },
                                            _ => {}
                                        }
                                        
                                        field.name = new_name_val.clone();
                                        field.set(&mut doc.graph);
                                    }
                                } else if let Some(mut func) = SFunc::func(&doc.graph, &context_path, '.', Some(&context)) {
                                    if doc.perms.can_write_func(&doc.graph, &func, doc.self_ptr().as_ref()) {
                                        func.name = new_name_val.clone();
                                        func.set(&mut doc.graph);
                                    }
                                }
                            }
                        }
                    }
                },
                Statement::If { if_expr, elif_exprs, else_expr } => {
                    let if_res = if_expr.0.exec(doc)?;
                    if if_res.truthy() {
                        doc.table.new_scope();
                        let res = if_expr.1.exec(doc)?;
                        doc.table.end_scope();
                        
                        match res {
                            StatementsRes::Break => {
                                // If bubble, need to propogate break upwards, out of if block
                                if doc.bubble_control_flow > 0 {
                                    return Ok(res);
                                }
                            },
                            StatementsRes::Continue => {
                                // If bubble, need to propogate continue upwards, out of if block
                                if doc.bubble_control_flow > 0 {
                                    return Ok(res);
                                }
                            },
                            StatementsRes::Return(_) => {
                                // Return statements always go all the way back up
                                return Ok(res);
                            },
                            StatementsRes::None => {
                                // Nothing to do here
                            }
                        }
                    } else {
                        // If statement was not able to execute, so drop into elif exprs
                        let mut matched = false;
                        for if_expr in elif_exprs {
                            let if_res = if_expr.0.exec(doc)?;
                            if if_res.truthy() {
                                doc.table.new_scope();
                                let res = if_expr.1.exec(doc)?;
                                doc.table.end_scope();
                                
                                match res {
                                    StatementsRes::Break => {
                                        // If bubble, need to propogate break upwards, out of if block
                                        if doc.bubble_control_flow > 0 {
                                            return Ok(res);
                                        }
                                    },
                                    StatementsRes::Continue => {
                                        // If bubble, need to propogate continue upwards, out of if block
                                        if doc.bubble_control_flow > 0 {
                                            return Ok(res);
                                        }
                                    },
                                    StatementsRes::Return(_) => {
                                        // Return statements always go all the way back up
                                        return Ok(res);
                                    },
                                    StatementsRes::None => {
                                        // Nothing to do here
                                    }
                                }
                                matched = true;
                                break;
                            }
                        }
                        if !matched {
                            // Didn't find an else if statement match, so look for an else statement
                            if let Some(else_statements) = else_expr {
                                doc.table.new_scope();
                                let res = else_statements.exec(doc)?;
                                doc.table.end_scope();
                                
                                match res {
                                    StatementsRes::Break => {
                                        // If bubble, need to propogate break upwards, out of if block
                                        if doc.bubble_control_flow > 0 {
                                            return Ok(res);
                                        }
                                    },
                                    StatementsRes::Continue => {
                                        // If bubble, need to propogate continue upwards, out of if block
                                        if doc.bubble_control_flow > 0 {
                                            return Ok(res);
                                        }
                                    },
                                    StatementsRes::Return(_) => {
                                        // Return statements always go all the way back up
                                        return Ok(res);
                                    },
                                    StatementsRes::None => {
                                        // Nothing to do here
                                    }
                                }
                            }
                        }
                    }
                },
                Statement::Return(expr) => {
                    // Push result of expr onto the stack
                    let val = expr.exec(doc)?;
                    doc.push(val);
                    return Ok(StatementsRes::Return(true));
                },
                Statement::EmptyReturn => {
                    // Return without putting anything on the stack
                    return Ok(StatementsRes::Return(false));
                },
                Statement::While(expr, statements) => {
                    while expr.exec(doc)?.truthy() {
                        doc.table.new_scope();
                        doc.bubble_control_flow += 1;
                        let res;
                        let sres = statements.exec(doc);
                        doc.bubble_control_flow -= 1;
                        match sres {
                            Ok(sres) => res = sres,
                            Err(_) => return sres,
                        }
                        doc.table.end_scope();

                        match res {
                            StatementsRes::Break => {
                                // Exit the while loop!
                                break;
                            },
                            StatementsRes::Continue => {
                                // Continue the while loop!
                            },
                            StatementsRes::None => {
                                // Do nothing here
                            },
                            StatementsRes::Return(_) => {
                                // Propagate returns all the way back up
                                return Ok(res);
                            }
                        }
                    }
                },
                Statement::Break => {
                    // No more statements in this block!
                    return Ok(StatementsRes::Break);
                },
                Statement::Continue => {
                    // No more statements in this block, but keep evaluating expr in while
                    return Ok(StatementsRes::Continue);
                },
                Statement::Expr(expr) => {
                    expr.exec(doc)?;
                },
                Statement::Block(statements, finally) => {
                    doc.table.new_scope();
                    let res = statements.exec(doc)?;
                    if finally.statements.len() > 0 {
                        finally.exec(doc)?; // We don't care about a result here
                    }
                    doc.table.end_scope();

                    match res {
                        StatementsRes::Break => {
                            // Break should propogate too if bubbling
                            if doc.bubble_control_flow > 0 {
                                return Ok(res);
                            }
                        },
                        StatementsRes::Continue => {
                            // Continue needs to continue propogating upwards if bubbling
                            if doc.bubble_control_flow > 0 {
                                return Ok(res);
                            }
                        },
                        StatementsRes::None => {
                            // Do nothing here
                        },
                        StatementsRes::Return(_) => {
                            // Propagate returns all the way back up
                            return Ok(res);
                        }
                    }
                },
            }
        }
        Ok(StatementsRes::None)
    }
}
impl From<Vec<Statement>> for Statements {
    fn from(value: Vec<Statement>) -> Self {
        Self {
            statements: value,
        }
    }
}


/// Statement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Statement {
    // Variables.
    Declare(String, Expr),
    Assign(String, Expr),

    // Field & Variable ops
    Drop(String),
    Move(String, String),
    Rename(String, Expr),

    // Expression statement.
    // Function calls, etc...
    Expr(Expr),

    // Block statement (different that block expr).
    // Second statements are 'finally' statements that are in the same scope, but always run.
    // They are helpful when constructing more complex statements, like for-in loops.
    Block(Statements, Statements),

    // Return statements.
    Return(Expr),
    EmptyReturn,
    
    // Control flow
    If {
        if_expr: (Expr, Statements),
        elif_exprs: Vec<(Expr, Statements)>,
        else_expr: Option<Statements>
    },
    While(Expr, Statements),
    Break,
    Continue,
}
