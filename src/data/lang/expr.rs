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

use std::{ops::Deref, sync::{Arc, RwLock}};
use anyhow::{anyhow, Result};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use crate::{IntoDataRef, SDoc, SField, SFunc, SType, SVal};
use super::{Statement, Statements, StatementsRes};


/// Stof expression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expr {
    Literal(SVal),
    Tuple(Vec<Expr>),
    Array(Vec<Expr>),

    /// Variable expression.
    /// Use a variable from the symbol table.
    /// Get a field from an ID ('.' separated path)
    /// Get a function from an ID ('.' separated path)
    Variable(String),

    Ref(Box<Expr>),
    DeRef(Box<Expr>),

    Cast(SType, Box<Expr>),
    TypeOf(Box<Expr>),
    TypeName(Box<Expr>),

    Call {
        scope: String,
        name: String,
        params: Vec<Expr>,
    },

    Block(Statements),
    NewObject(Statements),

    Add(Vec<Expr>),
    Sub(Vec<Expr>),
    Mul(Vec<Expr>),
    Div(Vec<Expr>),
    Rem(Vec<Expr>),

    And(Vec<Expr>),
    Or(Vec<Expr>),
    Not(Box<Expr>),

    Eq(Box<Expr>, Box<Expr>),
    Neq(Box<Expr>, Box<Expr>),
    Gte(Box<Expr>, Box<Expr>),
    Lte(Box<Expr>, Box<Expr>),
    Gt(Box<Expr>, Box<Expr>),
    Lt(Box<Expr>, Box<Expr>),
}
impl<T> From<T> for Expr where T: Into<SVal> {
    fn from(value: T) -> Self {
        Self::Literal(value.into())
    }
}
impl Expr {
    /// Execute this expression.
    pub fn exec(&self, doc: &mut SDoc) -> Result<SVal> {
        match self {
            Expr::Variable(id) => {
                // Look for a symbol first!
                if let Some(symbol) = doc.get_symbol(&id) {
                    let val = symbol.var();
                    return Ok(val);
                }

                // See if we are referencing self or super only
                if id == "self" {
                    if let Some(self_ref) = doc.self_ptr() {
                        if doc.perms.can_read_scope(&doc.graph, &self_ref, Some(&self_ref)) {
                            return Ok(SVal::Object(self_ref));
                        }
                        return Ok(SVal::Null);
                    } else {
                        return Ok(SVal::Null);
                    }
                } else if id == "super" {
                    if let Some(self_ref) = doc.self_ptr() {
                        if let Some(node) = self_ref.node(&doc.graph) {
                            if let Some(parent) = &node.parent {
                                if doc.perms.can_read_scope(&doc.graph, parent, Some(&self_ref)) {
                                    return Ok(SVal::Object(parent.clone()));
                                }
                            }
                        }
                        return Ok(SVal::Null);
                    } else {
                        return Ok(SVal::Null);
                    }
                }

                // Get the context object we are working with!
                let mut context = None;
                if id.starts_with("self") || id.starts_with("super") {
                    context = doc.self_ptr();
                }
                let mut context_path = id.clone();
                {
                    let mut path: Vec<&str> = id.split('.').collect();
                    if path.len() > 1 {
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
                                    context = Some(nref.clone());
                                    context_path = path.join(".");
                                },
                                _ => {}
                            }
                        }
                    }
                }

                // Look for a field first
                if let Some(field) = SField::field(&doc.graph, &context_path, '.', context.as_ref()) {
                    if doc.perms.can_read_field(&doc.graph, &field, doc.self_ptr().as_ref()) {
                        return Ok(field.value);
                    }
                    return Ok(SVal::Null);
                }
                
                // Look for an object in the graph next
                let obj_path = context_path.replace('.', "/");
                if let Some(node) = doc.graph.node_ref(&obj_path, context.as_ref()) {
                    if doc.perms.can_read_scope(&doc.graph, &node, doc.self_ptr().as_ref()) {
                        return Ok(SVal::Object(node));
                    }
                    return Ok(SVal::Null);
                }

                // Look for a function in the graph
                if let Some(func) = SFunc::func(&doc.graph, &context_path, '.', context.as_ref()) {
                    if doc.perms.can_read_func(&doc.graph, &func, doc.self_ptr().as_ref()) {
                        return Ok(SVal::FnPtr(func.data_ref()));
                    }
                    return Ok(SVal::Null);
                }

                // Not able to find a variable for this symbol, so return null
                Ok(SVal::Null)
            },
            Expr::Ref(expr) => {
                let value = expr.exec(doc)?;
                match value {
                    SVal::Ref(_) => Ok(value),
                    _ => {
                        let new_value = SVal::Ref(Arc::new(RwLock::new(value)));
                        // Special case when expr is a variable symbol - need to set the variable too
                        match expr.as_ref() {
                            Expr::Variable(id) => {
                                if doc.has_symbol(&id) {
                                    doc.set_variable(&id, new_value.clone());
                                }
                            },
                            _ => {}
                        }
                        Ok(new_value)
                    }
                }
            },
            Expr::DeRef(expr) => {
                let value = expr.exec(doc)?;
                match value {
                    SVal::Ref(rf) => {
                        let clone = rf.read().unwrap().clone();
                        Ok(clone)
                    },
                    _ => Ok(value)
                }
            },
            Expr::Literal(val) => {
                Ok(val.clone())
            },
            Expr::Tuple(vals) => {
                let mut vec: Vec<SVal> = Vec::new();
                for val in vals {
                    vec.push(val.exec(doc)?);
                }
                Ok(SVal::Tuple(vec))
            },
            Expr::Array(vals) => {
                let mut vec: Vec<SVal> = Vec::new();
                for val in vals {
                    vec.push(val.exec(doc)?);
                }
                Ok(SVal::Array(vec))
            },
            Expr::Block(statements) => {
                doc.table.new_scope();
                let res = statements.exec(doc)?;
                doc.table.end_scope();

                match res {
                    StatementsRes::Break |
                    StatementsRes::Continue |
                    StatementsRes::None => {
                        // Nothing to do here
                    },
                    StatementsRes::Return(v) => {
                        if v {
                            // block returned something to the stack!
                            return Ok(doc.pop().unwrap());
                        }
                    }
                }
                // Block did not return anything!
                Ok(SVal::Void)
            },
            Expr::NewObject(statements) => {
                let stof_object;
                if let Some(parent) = doc.self_ptr() {
                    stof_object = doc.graph.insert_node(&format!("obj{}", nanoid!(7)), Some(&parent));
                } else {
                    stof_object = doc.graph.insert_node(&format!("obj{}", nanoid!(7)), None);
                }

                // Parse initialization statements and execute them
                let mut init_statements = Vec::new();
                for statement in &statements.statements {
                    match statement {
                        Statement::Assign(name, expr) => {
                            let init_statement = Statement::Assign(format!("self.{}", &name).into(), expr.clone());
                            init_statements.push(init_statement);
                        },
                        Statement::Declare(name, expr) => {
                            let init_statement = Statement::Declare(format!("self.{}", &name).into(), expr.clone());
                            init_statements.push(init_statement);
                        }
                        _ => {}
                    }
                }

                // Execute initialization statements!
                // Make sure to set new object as self_ptr for new sub-objects!
                doc.self_stack.push(stof_object.clone());
                let init_statements = Statements::from(init_statements);
                let _ = init_statements.exec(doc);
                doc.self_stack.pop();

                return Ok(SVal::Object(stof_object));
            },
            Expr::Cast(stype, expr) => {
                let value = expr.exec(doc)?;
                let target = stype.clone();

                if value.stype(&doc.graph) == target {
                    return Ok(value);
                }
                return Ok(value.cast(target, doc)?);
            },
            Expr::TypeOf(expr) => {
                let value = expr.exec(doc)?;
                let value_type = value.stype(&doc.graph);
                if value_type.is_object() { // No custom object types here
                    return Ok(SVal::String("obj".to_string()));
                }
                let type_of = value_type.type_of();
                Ok(SVal::String(type_of))
            },
            Expr::TypeName(expr) => {
                let value = expr.exec(doc)?;
                Ok(SVal::String(value.type_name(&doc.graph)))
            },
            Expr::Not(expr) => {
                let value = expr.exec(doc)?;
                Ok(SVal::Bool(!value.truthy()))
            },
            Expr::Call { scope, name, params } => {
                // Scope can be a symbol, library name, or path to a field, object, or function
                let variable = Self::Variable(scope.replace('/', "."));
                let mut variable_value = variable.exec(doc)?;

                // Deref the variable if needed...
                match variable_value {
                    SVal::Ref(rf) => {
                        let val = rf.read().unwrap();
                        variable_value = val.deref().clone();
                    },
                    _ => {}
                }

                let mut library_name = String::default();
                if !variable_value.is_empty() {
                    let stype = variable_value.stype(&doc.graph);
                    library_name = match stype {
                        SType::Null |
                        SType::Void => String::default(),
                        SType::Array => "Array".to_owned(),
                        SType::FnPtr => "Function".to_owned(),
                        SType::String => "String".to_owned(),
                        SType::Number(_) => "Number".to_owned(),
                        SType::Bool => "Bool".to_owned(),
                        SType::Tuple(_) => "Tuple".to_owned(),
                        SType::Blob => "Blob".to_owned(),
                        SType::Object(_typename) => {
                            "Object".to_owned()
                        },
                    };
                }
                if let Some(lib) = doc.library(&library_name) {
                    let stype = variable_value.stype(&doc.graph);

                    // If the type is an object, try getting the function from that objects scope
                    match &variable_value {
                        SVal::Object(nref) => {
                            // Look for a function on the object itself first! Always higher priority than a prototype
                            if let Some(func) = SFunc::func(&doc.graph, name, '.', Some(&nref)) {
                                let mut func_params = Vec::new();
                                for expr in params {
                                    let val = expr.exec(doc)?;
                                    if !val.is_void() {
                                        func_params.push(val);
                                    }
                                }
                                let current_symbol_table = doc.new_table();
                                let res = func.call(doc, func_params, true)?;
                                doc.set_table(current_symbol_table);
                                return Ok(res);
                            }

                            // Look for a prototype on this object next
                            if let Some(prototype_field) = SField::field(&doc.graph, "__prototype__", '.', Some(nref)) {
                                if let Some(prototype) = doc.graph.node_ref(&prototype_field.to_string(), None) {
                                    // prototype is the exact type we are referencing... we need to check typestack here!
                                    let mut current = Some(prototype);

                                    let mut func_name = name.clone();
                                    let mut type_scope_resolution: Vec<&str> = name.split("::").collect();
                                    if type_scope_resolution.len() == 2 {
                                        func_name = type_scope_resolution.pop().unwrap().to_string();

                                        let scope_type = type_scope_resolution.pop().unwrap();
                                        let mut found = false;
                                        while let Some(typename_field) = SField::field(&doc.graph, "typename", '.', current.as_ref()) {
                                            if typename_field.to_string() == scope_type {
                                                found = true;
                                                break;
                                            }
                                            if let Some(node) = current.clone().unwrap().node(&doc.graph) {
                                                if let Some(parent_ref) = &node.parent {
                                                    current = Some(parent_ref.clone());
                                                } else {
                                                    break;
                                                }
                                            } else {
                                                break;
                                            }
                                        }
                                        if !found {
                                            return Err(anyhow!("Cannot find the requested type scope in the extends stack of this object for the requested function call"));
                                        }
                                    } else if type_scope_resolution.len() > 1 {
                                        return Err(anyhow!("Cannot specify more than one type scope for a function call"));
                                    }

                                    while current.is_some() {
                                        if let Some(func) = SFunc::func(&doc.graph, &func_name, '.', current.as_ref()) {
                                            let mut func_params = Vec::new();
                                            for expr in params {
                                                let val = expr.exec(doc)?;
                                                if !val.is_void() {
                                                    func_params.push(val);
                                                }
                                            }
                                            let current_symbol_table = doc.new_table();
                                            // Set self to the object still...
                                            doc.self_stack.push(nref.clone());
                                            let res = func.call(doc, func_params, false)?;
                                            doc.self_stack.pop();
                                            doc.set_table(current_symbol_table);
                                            return Ok(res);
                                        }
                                        if let Some(node) = current.unwrap().node(&doc.graph) {
                                            if let Some(parent_ref) = &node.parent {
                                                current = Some(parent_ref.clone());
                                            } else {
                                                break;
                                            }
                                        } else {
                                            break;
                                        }
                                    }
                                }
                            }
                        },
                        _ => {}
                    }

                    let mut func_params = vec![variable_value.clone()];
                    for expr in params {
                        let val = expr.exec(doc)?;
                        if !val.is_void() {
                            func_params.push(val);
                        }
                    }
                    let current_symbol_table = doc.new_table();
                    let mut library = lib.write().unwrap();
                    let res = library.call(doc, name, &mut func_params)?;
                    doc.set_table(current_symbol_table);

                    // Update the symbol with the mutated parameter if it's the right type
                    let new_symbol_val = func_params.first().unwrap().clone();
                    if new_symbol_val.stype(&doc.graph) == stype {
                        doc.set_variable(&scope, new_symbol_val);
                    }

                    return Ok(res);
                }

                // If here, scope is not a field, func, object, or symbol
                // Check to see if scope is a library itself before falling back to std lib
                if let Some(lib) = doc.library(&scope) {
                    let mut func_params = Vec::new();
                    for expr in params {
                        let val = expr.exec(doc)?;
                        if !val.is_void() {
                            func_params.push(val);
                        }
                    }
                    let current_symbol_table = doc.new_table();

                    let mut library = lib.write().unwrap();
                    let res = library.call(doc, name, &mut func_params)?;

                    doc.set_table(current_symbol_table);
                    return Ok(res);
                } else if let Some(lib) = doc.library("std") {
                    let mut func_params = Vec::new();
                    for expr in params {
                        let val = expr.exec(doc)?;
                        if !val.is_void() {
                            func_params.push(val);
                        }
                    }
                    let current_symbol_table = doc.new_table();

                    let mut library = lib.write().unwrap();
                    let res = library.call(doc, name, &mut func_params)?;
                    
                    doc.set_table(current_symbol_table);
                    return Ok(res);
                }
                Err(anyhow!("Function/Call does not exist: {}({:?})", name, params))
            },
            Expr::And(exprs) => {
                let mut res = SVal::Void;
                let mut first = true;
                for expr in exprs {
                    let val = expr.exec(doc)?;
                    if first {
                        res = val;
                        first = false;
                    } else {
                        res = res.and(&val, doc)?;
                    }
                }
                Ok(res)
            },
            Expr::Or(exprs) => {
                let mut res = SVal::Void;
                let mut first = true;
                for expr in exprs {
                    let val = expr.exec(doc)?;
                    if first {
                        res = val;
                        first = false;
                    } else {
                        res = res.or(&val, doc)?;
                    }
                }
                Ok(res)
            },
            Expr::Add(exprs) => {
                let mut res = SVal::Void;
                let mut first = true;
                for expr in exprs {
                    let val = expr.exec(doc)?;
                    if first {
                        res = val;
                        first = false;
                    } else {
                        res = res.add(&val, doc)?;
                    }
                }
                Ok(res)
            },
            Expr::Sub(exprs) => {
                let mut res = SVal::Void;
                let mut first = true;
                for expr in exprs {
                    let val = expr.exec(doc)?;
                    if first {
                        res = val;
                        first = false;
                    } else {
                        res = res.sub(&val, doc)?;
                    }
                }
                Ok(res)
            },
            Expr::Mul(exprs) => {
                let mut res = SVal::Void;
                let mut first = true;
                for expr in exprs {
                    let val = expr.exec(doc)?;
                    if first {
                        res = val;
                        first = false;
                    } else {
                        res = res.mul(&val, doc)?;
                    }
                }
                Ok(res)
            },
            Expr::Div(exprs) => {
                let mut res = SVal::Void;
                let mut first = true;
                for expr in exprs {
                    let val = expr.exec(doc)?;
                    if first {
                        res = val;
                        first = false;
                    } else {
                        res = res.div(&val, doc)?;
                    }
                }
                Ok(res)
            },
            Expr::Rem(exprs) => {
                let mut res = SVal::Void;
                let mut first = true;
                for expr in exprs {
                    let val = expr.exec(doc)?;
                    if first {
                        res = val;
                        first = false;
                    } else {
                        res = res.rem(&val, doc)?;
                    }
                }
                Ok(res)
            },
            Expr::Eq(lhs, rhs) => {
                let lhs = lhs.exec(doc)?;
                let rhs = rhs.exec(doc)?;
                Ok(lhs.equal(&rhs, doc)?)
            },
            Expr::Neq(lhs, rhs) => {
                let lhs = lhs.exec(doc)?;
                let rhs = rhs.exec(doc)?;
                Ok(lhs.neq(&rhs, doc)?)
            },
            Expr::Gte(lhs, rhs) => {
                let lhs = lhs.exec(doc)?;
                let rhs = rhs.exec(doc)?;
                Ok(lhs.gte(&rhs, doc)?)
            },
            Expr::Lte(lhs, rhs) => {
                let lhs = lhs.exec(doc)?;
                let rhs = rhs.exec(doc)?;
                Ok(lhs.lte(&rhs, doc)?)
            },
            Expr::Gt(lhs, rhs) => {
                let lhs = lhs.exec(doc)?;
                let rhs = rhs.exec(doc)?;
                Ok(lhs.gt(&rhs, doc)?)
            },
            Expr::Lt(lhs, rhs) => {
                let lhs = lhs.exec(doc)?;
                let rhs = rhs.exec(doc)?;
                Ok(lhs.lt(&rhs, doc)?)
            },
        }
    }

    /// Is variable expression?
    pub fn is_variable(&self) -> bool {
        match self {
            Expr::Variable(_) => true,
            _ => false,
        }
    }

    /// Is literal expression?
    pub fn is_literal(&self) -> bool {
        match self {
            Expr::Literal(_) => true,
            _ => false,
        }
    }
}
