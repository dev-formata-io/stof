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

use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use crate::{IntoDataRef, IntoNodeRef, SDoc, SField, SFunc, SPrototype, SType, SVal};
use super::{SError, Statement, Statements, StatementsRes};


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
    pub fn exec(&self, pid: &str, doc: &mut SDoc) -> Result<SVal, SError> {
        match self {
            Expr::Variable(id) => {
                // Look for a symbol first!
                if let Some(symbol) = doc.get_symbol(pid, &id) {
                    let val = symbol.var();
                    return Ok(val);
                }

                // See if we are referencing self or super only
                if id == "self" {
                    if let Some(self_ref) = doc.self_ptr(pid) {
                        if doc.perms.can_read_scope(&doc.graph, &self_ref, Some(&self_ref)) {
                            return Ok(SVal::Object(self_ref));
                        }
                        return Ok(SVal::Null);
                    } else {
                        return Ok(SVal::Null);
                    }
                } else if id == "super" {
                    if let Some(self_ref) = doc.self_ptr(pid) {
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
                    context = doc.self_ptr(pid);
                }
                let mut context_path = id.clone();
                {
                    let mut path: Vec<&str> = id.split('.').collect();
                    if path.len() > 1 {
                        if let Some(symbol) = doc.get_symbol(pid, path.remove(0)) {
                            match symbol.var() {
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
                    if doc.perms.can_read_field(&doc.graph, &field, doc.self_ptr(pid).as_ref()) {
                        return Ok(field.value);
                    }
                    return Ok(SVal::Null);
                }
                
                // Look for an object in the graph next
                let obj_path = context_path.replace('.', "/");
                if let Some(node) = doc.graph.node_ref(&obj_path, context.as_ref()) {
                    if doc.perms.can_read_scope(&doc.graph, &node, doc.self_ptr(pid).as_ref()) {
                        return Ok(SVal::Object(node));
                    }
                    return Ok(SVal::Null);
                }

                // Look for a function in the graph
                if let Some(func) = SFunc::func(&doc.graph, &context_path, '.', context.as_ref()) {
                    if doc.perms.can_read_func(&doc.graph, &func, doc.self_ptr(pid).as_ref()) {
                        return Ok(SVal::FnPtr(func.data_ref()));
                    }
                    return Ok(SVal::Null);
                }

                // Not able to find a variable for this symbol, so return null
                Ok(SVal::Null)
            },
            Expr::Literal(val) => {
                Ok(val.clone())
            },
            Expr::Tuple(vals) => {
                let mut vec: Vec<SVal> = Vec::new();
                for val in vals {
                    vec.push(val.exec(pid, doc)?);
                }
                Ok(SVal::Tuple(vec))
            },
            Expr::Array(vals) => {
                let mut vec: Vec<SVal> = Vec::new();
                for val in vals {
                    vec.push(val.exec(pid, doc)?);
                }
                Ok(SVal::Array(vec))
            },
            Expr::Block(statements) => {
                doc.new_scope(pid);
                let res = statements.exec(pid, doc)?;
                doc.end_scope(pid);

                match res {
                    StatementsRes::Break |
                    StatementsRes::Continue |
                    StatementsRes::None => {
                        // Nothing to do here
                    },
                    StatementsRes::Return(v) => {
                        if v {
                            // block returned something to the stack!
                            return Ok(doc.pop(pid).unwrap());
                        }
                    }
                }
                // Block did not return anything!
                Ok(SVal::Void)
            },
            Expr::NewObject(statements) => {
                let stof_object;
                if let Some(parent) = doc.self_ptr(pid) {
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
                doc.push_self(pid, stof_object.clone());
                let init_statements = Statements::from(init_statements);
                let _ = init_statements.exec(pid, doc);
                doc.pop_self(pid);

                return Ok(SVal::Object(stof_object));
            },
            Expr::Cast(stype, expr) => {
                let value = expr.exec(pid, doc)?;
                let target = stype.clone();

                if value.stype(&doc.graph) == target {
                    return Ok(value);
                }
                return Ok(value.cast(target, pid, doc)?);
            },
            Expr::TypeOf(expr) => {  // always generic type (ex. float, int, obj), but can be boxed
                let value = expr.exec(pid, doc)?;
                
                if value.is_number() {
                    if value.is_int() {
                        if value.is_boxed() {
                            return Ok(SVal::String("Box<int>".to_string()));
                        }
                        return Ok(SVal::String("int".to_string()));
                    } else {
                        if value.is_boxed() {
                            return Ok(SVal::String("Box<float>".to_string()));
                        }
                        return Ok(SVal::String("float".to_string()));
                    }
                }
                if value.is_object() {
                    if value.is_boxed() {
                        return Ok(SVal::String("Box<obj>".to_string()));
                    }
                    return Ok(SVal::String("obj".to_string()));
                }

                let value_type = value.stype(&doc.graph);
                Ok(SVal::String(value_type.type_of()))
            },
            Expr::TypeName(expr) => { // never boxed, always explicit type name (ex. units, custom object type)
                let value = expr.exec(pid, doc)?;
                Ok(SVal::String(value.type_name(&doc.graph)))
            },
            Expr::Not(expr) => {
                let value = expr.exec(pid, doc)?;
                Ok(SVal::Bool(!value.truthy()))
            },
            Expr::Call { scope, name, params } => {
                // Scope can be a symbol, library name, or path to a field, object, or function
                let variable = Self::Variable(scope.replace('/', "."));
                let variable_value = variable.exec(pid, doc)?;

                // If the type is an object, try getting the function from that objects scope
                match &variable_value {
                    SVal::Object(nref) => {
                        // Look for a function on the object itself first! Always higher priority than a prototype
                        if let Some(func) = SFunc::func(&doc.graph, name, '.', Some(&nref)) {
                            let mut func_params = Vec::new();
                            for expr in params {
                                let val = expr.exec(pid, doc)?;
                                if !val.is_void() {
                                    func_params.push(val);
                                }
                            }
                            let current_symbol_table = doc.new_table(pid);
                            let res = func.call(pid, doc, func_params, true)?;
                            doc.set_table(pid, current_symbol_table);
                            return Ok(res);
                        }

                        // Look for a prototype on this object next
                        if let Some(prototype) = SPrototype::get(&doc.graph, nref) {
                            let prototype = prototype.node_ref();
                            
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
                                    let error = SError::type_error(pid, &doc, "cannot find the requested type in the extends stack of this object in resolving function call");
                                    return Err(error);
                                }
                            } else if type_scope_resolution.len() > 1 {
                                let error = SError::type_error(pid, &doc, "cannot specify more than one type to resolve a function call");
                                return Err(error);
                            }

                            while current.is_some() {
                                if let Some(func) = SFunc::func(&doc.graph, &func_name, '.', current.as_ref()) {
                                    let mut func_params = Vec::new();
                                    for expr in params {
                                        let val = expr.exec(pid, doc)?;
                                        if !val.is_void() {
                                            func_params.push(val);
                                        }
                                    }
                                    let current_symbol_table = doc.new_table(pid);
                                    // Set self to the object still...
                                    doc.push_self(pid, nref.clone());
                                    let res = func.call(pid, doc, func_params, false)?;
                                    doc.pop_self(pid);
                                    doc.set_table(pid, current_symbol_table);
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
                    },
                    _ => {}
                }

                let mut library_name = String::default();
                let stype = variable_value.stype(&doc.graph);
                if !variable_value.is_empty() {
                    library_name = stype.std_libname();
                }
                if let Some(lib) = doc.library(&library_name) {
                    let mut func_params = vec![variable_value];
                    for expr in params {
                        let val = expr.exec(pid, doc)?;
                        if !val.is_void() {
                            func_params.push(val);
                        }
                    }

                    // For the standard libraries, allow them to access the current symbol table...
                    // This includes Function.call, allowing arrow functions to capture outer scope when called
                    //let current_symbol_table = doc.new_table(pid);
                    doc.new_scope(pid);
                    let res = lib.call(pid, doc, name, &mut func_params);
                    doc.end_scope(pid);
                    //doc.set_table(pid, current_symbol_table);

                    // If res is an error, check if we have a library with the scope to fall back on
                    if res.is_err() && doc.library(&scope).is_some() {
                        // Allow fall-through to scope library
                    } else if res.is_err() {
                        return Err(res.err().unwrap());
                    } else {
                        let res = res.unwrap();
                        
                        // Update the symbol with the mutated parameter if it's the right type
                        if func_params.len() > 0 {
                            let new_symbol_val = func_params.first().unwrap().clone();
                            if new_symbol_val.stype(&doc.graph) == stype {
                                doc.set_variable(pid, &scope, new_symbol_val);
                            }
                        }

                        return Ok(res);
                    }
                }

                // If here, scope is not a field, func, object, or symbol
                // Check to see if scope is a library itself before falling back to std lib
                if let Some(lib) = doc.library(&scope) {
                    let mut func_params = Vec::new();
                    for expr in params {
                        let val = expr.exec(pid, doc)?;
                        if !val.is_void() {
                            func_params.push(val);
                        }
                    }
                    let current_symbol_table = doc.new_table(pid);
                    let res = lib.call(pid, doc, name, &mut func_params);
                    match res {
                        Ok(val) => {
                            doc.set_table(pid, current_symbol_table);
                            return Ok(val);
                        },
                        Err(error) => {
                            doc.set_table(pid, current_symbol_table);
                            return Err(error);
                        }
                    }
                } else if let Some(lib) = doc.library("std") {
                    let mut func_params = Vec::new();
                    for expr in params {
                        let val = expr.exec(pid, doc)?;
                        if !val.is_void() {
                            func_params.push(val);
                        }
                    }
                    let current_symbol_table = doc.new_table(pid);
                    let res = lib.call(pid, doc, name, &mut func_params);
                    match res {
                        Ok(val) => {
                            doc.set_table(pid, current_symbol_table);
                            return Ok(val);
                        },
                        Err(error) => {
                            doc.set_table(pid, current_symbol_table);
                            return Err(error);
                        }
                    }
                }
                let error = SError::custom(pid, &doc, "ExprCall", &format!("function does not exist: {}({:?})", name, params));
                Err(error)
            },
            Expr::And(exprs) => {
                for expr in exprs {
                    let val = expr.exec(pid, doc)?;
                    if !val.truthy() {
                        return Ok(SVal::Bool(false));
                    }
                }
                Ok(SVal::Bool(true))
            },
            Expr::Or(exprs) => {
                for expr in exprs {
                    let val = expr.exec(pid, doc)?;
                    if val.truthy() {
                        return Ok(SVal::Bool(true));
                    }
                }
                Ok(SVal::Bool(false))
            },
            Expr::Add(exprs) => {
                let mut res = SVal::Void;
                let mut first = true;
                for expr in exprs {
                    let val = expr.exec(pid, doc)?;
                    if first {
                        res = val;
                        first = false;
                    } else {
                        res = res.add(pid, val, doc)?;
                    }
                }
                Ok(res)
            },
            Expr::Sub(exprs) => {
                let mut res = SVal::Void;
                let mut first = true;
                for expr in exprs {
                    let val = expr.exec(pid, doc)?;
                    if first {
                        res = val;
                        first = false;
                    } else {
                        res = res.sub(pid, val, doc)?;
                    }
                }
                Ok(res)
            },
            Expr::Mul(exprs) => {
                let mut res = SVal::Void;
                let mut first = true;
                for expr in exprs {
                    let val = expr.exec(pid, doc)?;
                    if first {
                        res = val;
                        first = false;
                    } else {
                        res = res.mul(pid, val, doc)?;
                    }
                }
                Ok(res)
            },
            Expr::Div(exprs) => {
                let mut res = SVal::Void;
                let mut first = true;
                for expr in exprs {
                    let val = expr.exec(pid, doc)?;
                    if first {
                        res = val;
                        first = false;
                    } else {
                        res = res.div(pid, val, doc)?;
                    }
                }
                Ok(res)
            },
            Expr::Rem(exprs) => {
                let mut res = SVal::Void;
                let mut first = true;
                for expr in exprs {
                    let val = expr.exec(pid, doc)?;
                    if first {
                        res = val;
                        first = false;
                    } else {
                        res = res.rem(pid, val, doc)?;
                    }
                }
                Ok(res)
            },
            Expr::Eq(lhs, rhs) => {
                let lhs = lhs.exec(pid, doc)?;
                let rhs = rhs.exec(pid, doc)?;
                Ok(lhs.equal(&rhs)?)
            },
            Expr::Neq(lhs, rhs) => {
                let lhs = lhs.exec(pid, doc)?;
                let rhs = rhs.exec(pid, doc)?;
                Ok(lhs.neq(&rhs)?)
            },
            Expr::Gte(lhs, rhs) => {
                let lhs = lhs.exec(pid, doc)?;
                let rhs = rhs.exec(pid, doc)?;
                Ok(lhs.gte(&rhs)?)
            },
            Expr::Lte(lhs, rhs) => {
                let lhs = lhs.exec(pid, doc)?;
                let rhs = rhs.exec(pid, doc)?;
                Ok(lhs.lte(&rhs)?)
            },
            Expr::Gt(lhs, rhs) => {
                let lhs = lhs.exec(pid, doc)?;
                let rhs = rhs.exec(pid, doc)?;
                Ok(lhs.gt(&rhs)?)
            },
            Expr::Lt(lhs, rhs) => {
                let lhs = lhs.exec(pid, doc)?;
                let rhs = rhs.exec(pid, doc)?;
                Ok(lhs.lt(&rhs)?)
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
