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

use std::{collections::{BTreeMap, HashMap}, ops::Deref};
use serde::{Deserialize, Serialize};
use crate::{SData, SDoc, SField, SFunc, SType, SVal};
use super::{Expr, SError};


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
    pub fn exec(&self, pid: &str, doc: &mut SDoc) -> Result<StatementsRes, SError> {
        for statement in &self.statements {
            match statement {
                Statement::Declare(name, rhs) => {
                    // Check to see if the symbol already exists in the current scope
                    if doc.has_var_with_name_in_current(pid, name) {
                        let error = SError::custom(pid, &doc, "VarDeclare", &format!("cannot declare a variable twice within the same scope: {}", &name));
                        return Err(error);
                    }

                    // Eval rhs, which is the value of the new variable
                    let val = rhs.exec(pid, doc)?;
                    if val.is_void() {
                        let error = SError::custom(pid, &doc, "VarDeclare", "cannot declare a void variable");
                        return Err(error);
                    }

                    // Do not allow fields to be set on declare!
                    if name.contains('.') {
                        let error = SError::custom(pid, &doc, "VarDeclare", "cannot declare variables that are paths, use an assignment operation if intending to create/assign fields");
                        return Err(error);
                    } else {
                        doc.add_variable(pid, &name, val);
                    }
                },
                Statement::Assign(name, rhs) => {
                    // Eval rhs, which is the value of the variable!
                    let val = rhs.exec(pid, doc)?;
                    if val.is_void() {
                        let error = SError::custom(pid, &doc, "AssignError", "cannot assign a void value");
                        return Err(error);
                    }

                    // Try setting a variable in the symbol table
                    let mut set_var = false;
                    if doc.set_variable(pid, &name, val.clone()) {
                        set_var = true;
                    }

                    if !set_var && name.contains('.') && !name.ends_with('.') && !name.starts_with('.') {
                        // Try assigning via an absolute path to a field first
                        if let Some(field_ref) = SField::field_ref(&doc.graph, &name, '.', None) {
                            if doc.perms.can_write_field(&doc.graph, &field_ref, doc.self_ptr(pid).as_ref()) {
                                if let Some(field) = SData::get_mut::<SField>(&mut doc.graph, &field_ref) {
                                    field.value = val;
                                }
                            }
                        } else {
                            // Look for a variable that is the first token in the path next
                            let mut path = name.split('.').collect::<Vec<&str>>();
                            let mut context = None;
                            let mut context_path = name.clone();
                            if let Some(symbol) = doc.get_symbol(pid, path.remove(0)) {
                                match symbol.var() {
                                    SVal::Object(nref) => {
                                        context = Some(nref);
                                        context_path = path.join(".");
                                    },
                                    _ => {}
                                }
                            }
                            // If no variable matches, try setting self scope
                            else if name.starts_with("self") || name.starts_with("super") {
                                context = doc.self_ptr(pid);
                            }

                            if let Some(mut context) = context {
                                // Already defined field?
                                if let Some(field_ref) = SField::field_ref(&doc.graph, &context_path, '.', Some(&context)) {
                                    if doc.perms.can_write_field(&doc.graph, &field_ref, doc.self_ptr(pid).as_ref()) {
                                        if let Some(field) = SData::get_mut::<SField>(&mut doc.graph, &field_ref) {
                                            field.value = val;
                                        }
                                    }
                                }
                                // Creating a new field
                                else {
                                    if doc.perms.can_write_scope(&doc.graph, &context, doc.self_ptr(pid).as_ref()) {
                                        let mut new_field_path = context_path.split('.').collect::<Vec<&str>>();

                                        let field_name = new_field_path.pop().unwrap();
                                        if new_field_path.len() > 0 {
                                            context = doc.graph.ensure_nodes(&new_field_path.join("/"), '/', true, Some(context.clone()));
                                        }

                                        let field = SField::new(field_name, val);
                                        SData::insert_new(&mut doc.graph, &context, Box::new(field));
                                    }
                                }
                            } else {
                                // Create a new object and add a field to it
                                let mut obj_path = name.split(".").collect::<Vec<&str>>();
                                let backup_name = obj_path.pop().unwrap();
                                if obj_path.len() > 0 {
                                    let nref = doc.graph.ensure_nodes(&obj_path.join("/"), '/', true, None);
                                    if doc.perms.can_write_scope(&doc.graph, &nref, doc.self_ptr(pid).as_ref()) {
                                        let field = SField::new(backup_name, val);
                                        SData::insert_new(&mut doc.graph, &nref, Box::new(field)); // attach new field to self
                                    }
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
                            _ => {
                                let error = SError::custom(pid, &doc, "AssignError", "variable does not exist and cannot create/assign a root object with a non-object value");
                                return Err(error)
                            }
                        }
                    }
                },
                Statement::Drop(name) => {
                    // Try dropping a variable in the symbol table by name
                    let mut dropped_var = false;
                    if let Some(symbol) = doc.drop(pid, &name) {
                        let dropped_val = symbol.var();
                        match &dropped_val {
                            SVal::Object(nref) => {
                                if !doc.perms.can_write_scope(&doc.graph, &nref, doc.self_ptr(pid).as_ref()) {
                                    doc.add_variable(pid, &name, dropped_val);
                                } else {
                                    doc.types.drop_types_for(&nref, &doc.graph);
                                    doc.graph.remove_node(nref);
                                }
                            },
                            SVal::FnPtr(dref) => {
                                if doc.perms.can_write_func(&doc.graph, dref, doc.self_ptr(pid).as_ref()) {
                                    doc.graph.remove_data(dref, None);
                                } else {
                                    doc.add_variable(pid, &name, dropped_val);
                                }
                            },
                            _ => {}
                        }
                        dropped_var = true;
                    }

                    if !dropped_var {
                        if let Some(field_ref) = SField::field_ref(&doc.graph, &name, '.', None) {
                            let mut remove = true;
                            let mut remove_objects = Vec::new();
                            if let Some(field) = SData::get::<SField>(&doc.graph, &field_ref) {
                                if !doc.perms.can_write_field(&doc.graph, &field_ref, doc.self_ptr(pid).as_ref()) {
                                    remove = false;
                                } else {
                                    match &field.value {
                                        SVal::Object(nref) => {
                                            remove_objects.push(nref.clone());
                                        },
                                        SVal::Array(vec) => {
                                            for val in vec {
                                                match val {
                                                    SVal::Object(nref) => {
                                                        remove_objects.push(nref.clone());
                                                    },
                                                    _ => {}
                                                }
                                            }
                                        },
                                        SVal::Boxed(val) => {
                                            let val = val.lock().unwrap();
                                            let val = val.deref();
                                            match val {
                                                SVal::Object(nref) => {
                                                    remove_objects.push(nref.clone());
                                                },
                                                SVal::Array(vec) => {
                                                    for val in vec {
                                                        match val {
                                                            SVal::Object(nref) => {
                                                                remove_objects.push(nref.clone());
                                                            },
                                                            _ => {}
                                                        }
                                                    }
                                                },
                                                _ => {}
                                            }
                                        },
                                        _ => {}
                                    }
                                }
                            }
                            if remove {
                                doc.graph.remove_data(&field_ref, None);
                                for obj in remove_objects {
                                    doc.types.drop_types_for(&obj, &doc.graph);
                                    doc.graph.remove_node(&obj);
                                }
                            }
                        } else if let Some(func_ref) = SFunc::func_ref(&doc.graph, &name, '.', None) {
                            if doc.perms.can_write_func(&doc.graph, &func_ref, doc.self_ptr(pid).as_ref()) {
                                doc.graph.remove_data(&func_ref, None);
                            }
                        } else if let Some(node) = doc.graph.node_ref(&name.replace(".", "/"), None) {
                            if doc.perms.can_write_scope(&doc.graph, &node, doc.self_ptr(pid).as_ref()) {
                                doc.types.drop_types_for(&node, &doc.graph);
                                doc.graph.remove_node(&node);
                            }
                        } else {
                            // Get the object we are operating on, if any
                            let mut context_ptr = None;
                            let mut context_path = name.clone();
                            if name == "self" || name == "super" {
                                context_ptr = doc.self_ptr(pid);
                                context_path = String::default();
                            } else if name.starts_with("self") || name.starts_with("super") {
                                context_ptr = doc.self_ptr(pid);
                            } else {
                                let mut path = name.split(".").collect::<Vec<&str>>();
                                if path.len() > 0 {
                                    let var = path.remove(0);
                                    if let Some(symbol) = doc.get_symbol(pid, var) {
                                        let var = symbol.var();
                                        match var {
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
                                    if doc.perms.can_write_scope(&doc.graph, &context, doc.self_ptr(pid).as_ref()) {
                                        let mut object_name = String::default();
                                        if let Some(context_node) = context.node(&doc.graph) {
                                            object_name = context_node.name.clone();
                                        }
                                        if object_name.len() > 0 {
                                            if let Some(field) = SField::field_ref(&doc.graph, &format!("super.{}", object_name), '.', Some(&context)) {
                                                doc.graph.remove_data(&field, None);
                                            }
                                        }
                                        doc.types.drop_types_for(&context, &doc.graph);
                                        doc.graph.remove_node(&context);
                                    }
                                } else if let Some(field_ref) = SField::field_ref(&doc.graph, &context_path, '.', Some(&context)) {
                                    let mut remove = true;
                                    let mut remove_objects = Vec::new();
                                    if let Some(field) = SData::get::<SField>(&doc.graph, &field_ref) {
                                        if !doc.perms.can_write_field(&doc.graph, &field_ref, doc.self_ptr(pid).as_ref()) {
                                            remove = false;
                                        } else {
                                            match &field.value {
                                                SVal::Object(nref) => {
                                                    remove_objects.push(nref.clone());
                                                },
                                                SVal::Array(vec) => {
                                                    for val in vec {
                                                        match val {
                                                            SVal::Object(nref) => {
                                                                remove_objects.push(nref.clone());
                                                            },
                                                            _ => {}
                                                        }
                                                    }
                                                },
                                                SVal::Boxed(val) => {
                                                    let val = val.lock().unwrap();
                                                    let val = val.deref();
                                                    match val {
                                                        SVal::Object(nref) => {
                                                            remove_objects.push(nref.clone());
                                                        },
                                                        SVal::Array(vec) => {
                                                            for val in vec {
                                                                match val {
                                                                    SVal::Object(nref) => {
                                                                        remove_objects.push(nref.clone());
                                                                    },
                                                                    _ => {}
                                                                }
                                                            }
                                                        },
                                                        _ => {}
                                                    }
                                                },
                                                _ => {}
                                            }
                                        }
                                    }
                                    if remove {
                                        doc.graph.remove_data(&field_ref, None);
                                        for obj in remove_objects {
                                            doc.types.drop_types_for(&obj, &doc.graph);
                                            doc.graph.remove_node(&obj);
                                        }
                                    }
                                } else if let Some(func_ref) = SFunc::func_ref(&doc.graph, &context_path, '.', Some(&context)) {
                                    if doc.perms.can_write_func(&doc.graph, &func_ref, doc.self_ptr(pid).as_ref()) {
                                        doc.graph.remove_data(&func_ref, None);
                                    }
                                }
                            }
                        }
                    }
                },
                Statement::If { if_expr, elif_exprs, else_expr } => {
                    let if_res = if_expr.0.exec(pid, doc)?;
                    if if_res.truthy() {
                        doc.new_scope(pid);
                        let res = if_expr.1.exec(pid, doc)?;
                        doc.end_scope(pid);
                        
                        match res {
                            StatementsRes::Break => {
                                // If bubble, need to propogate break upwards, out of if block
                                if doc.bubble_control_flow(pid) {
                                    return Ok(res);
                                }
                            },
                            StatementsRes::Continue => {
                                // If bubble, need to propogate continue upwards, out of if block
                                if doc.bubble_control_flow(pid) {
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
                            let if_res = if_expr.0.exec(pid, doc)?;
                            if if_res.truthy() {
                                doc.new_scope(pid);
                                let res = if_expr.1.exec(pid, doc)?;
                                doc.end_scope(pid);
                                
                                match res {
                                    StatementsRes::Break => {
                                        // If bubble, need to propogate break upwards, out of if block
                                        if doc.bubble_control_flow(pid) {
                                            return Ok(res);
                                        }
                                    },
                                    StatementsRes::Continue => {
                                        // If bubble, need to propogate continue upwards, out of if block
                                        if doc.bubble_control_flow(pid) {
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
                                doc.new_scope(pid);
                                let res = else_statements.exec(pid, doc)?;
                                doc.end_scope(pid);
                                
                                match res {
                                    StatementsRes::Break => {
                                        // If bubble, need to propogate break upwards, out of if block
                                        if doc.bubble_control_flow(pid) {
                                            return Ok(res);
                                        }
                                    },
                                    StatementsRes::Continue => {
                                        // If bubble, need to propogate continue upwards, out of if block
                                        if doc.bubble_control_flow(pid) {
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
                Statement::Switch(expr, map, default) => {
                    let val = expr.exec(pid, doc)?;
                    if let Some(statements) = map.get(&val) {
                        doc.new_scope(pid);
                        let res = statements.exec(pid, doc)?;
                        doc.end_scope(pid);
                        
                        match res {
                            StatementsRes::Break => {
                                // If bubble, need to propogate break upwards
                                if doc.bubble_control_flow(pid) {
                                    return Ok(res);
                                }
                            },
                            StatementsRes::Continue => {
                                // If bubble, need to propogate continue upwards
                                if doc.bubble_control_flow(pid) {
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
                    } else if let Some(default) = default {
                        doc.new_scope(pid);
                        let res = default.exec(pid, doc)?;
                        doc.end_scope(pid);
                        
                        match res {
                            StatementsRes::Break => {
                                // If bubble, need to propogate break upwards
                                if doc.bubble_control_flow(pid) {
                                    return Ok(res);
                                }
                            },
                            StatementsRes::Continue => {
                                // If bubble, need to propogate continue upwards
                                if doc.bubble_control_flow(pid) {
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
                },
                Statement::TryCatch(try_statements, catch_statements, catch_type, catch_var) => {
                    doc.new_scope(pid);
                    let err_res = try_statements.exec(pid, doc);
                    doc.end_scope(pid);
                    
                    // If we saw an error, do catch statements
                    let res;
                    match err_res {
                        Ok(ok) => {
                            res = ok;
                        },
                        Err(error) => {
                            doc.new_scope(pid);
                            if !catch_type.is_empty() && catch_var.len() > 0 {
                                match catch_type {
                                    SType::String => {
                                        doc.add_variable(pid, catch_var, SVal::String(error.message));
                                    },
                                    SType::Tuple(types) => {
                                        if types.len() == 2 && types[0].is_string() && types[1].is_string() {
                                            doc.add_variable(pid, catch_var, SVal::Tuple(vec![SVal::String(error.error_type.to_string()), SVal::String(error.message)]));
                                        } else {
                                            return Err(SError::type_error(pid, &doc, "try-catch block tuple error must be in the form (type: str, message: str)"));
                                        }
                                    },
                                    SType::Map => {
                                        let mut map = BTreeMap::new();
                                        map.insert(SVal::String("type".into()), SVal::String(error.error_type.to_string()));
                                        map.insert(SVal::String("message".into()), SVal::String(error.message));
                                        map.insert(SVal::String("stack".into()), SVal::Array(error.call_stack.into_iter().map(|dref| SVal::FnPtr(dref)).collect::<Vec<SVal>>()));
                                        doc.add_variable(pid, catch_var, SVal::Map(map));
                                    },
                                    _ => {
                                        return Err(SError::type_error(pid, &doc, "try-catch block has an incompatable type for catching an error"));
                                    }
                                }
                            }
                            res = catch_statements.exec(pid, doc)?;
                            doc.end_scope(pid);
                        }
                    }

                    match res {
                        StatementsRes::Break => {
                            // Break should propogate too if bubbling
                            if doc.bubble_control_flow(pid) {
                                return Ok(res);
                            }
                        },
                        StatementsRes::Continue => {
                            // Continue needs to continue propogating upwards if bubbling
                            if doc.bubble_control_flow(pid) {
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
                Statement::Return(expr) => {
                    // Push result of expr onto the stack
                    let val = expr.exec(pid, doc)?;
                    doc.push(pid, val);
                    return Ok(StatementsRes::Return(true));
                },
                Statement::EmptyReturn => {
                    // Return without putting anything on the stack
                    return Ok(StatementsRes::Return(false));
                },
                Statement::While(expr, statements) => {
                    while expr.exec(pid, doc)?.truthy() {
                        doc.new_scope(pid);
                        doc.inc_bubble_control(pid);
                        let res;
                        let sres = statements.exec(pid, doc);
                        doc.dinc_bubble_control(pid);
                        match sres {
                            Ok(sres) => res = sres,
                            Err(_) => return sres,
                        }
                        doc.end_scope(pid);

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
                    expr.exec(pid, doc)?;
                },
                Statement::Block(statements, finally) => {
                    doc.new_scope(pid);
                    let res = statements.exec(pid, doc)?;
                    if finally.statements.len() > 0 {
                        finally.exec(pid, doc)?; // We don't care about a result here
                    }
                    doc.end_scope(pid);

                    match res {
                        StatementsRes::Break => {
                            // Break should propogate too if bubbling
                            if doc.bubble_control_flow(pid) {
                                return Ok(res);
                            }
                        },
                        StatementsRes::Continue => {
                            // Continue needs to continue propogating upwards if bubbling
                            if doc.bubble_control_flow(pid) {
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
    Switch(Expr, HashMap<SVal, Statements>, Option<Statements>),
    TryCatch(Statements, Statements, SType, String),
    While(Expr, Statements),
    Break,
    Continue,
}
