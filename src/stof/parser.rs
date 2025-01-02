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
use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use nanoid::nanoid;
use pest_derive::Parser;
use pest::{iterators::{Pair, Pairs}, pratt_parser::PrattParser, Parser};
use crate::{lang::{CustomType, Expr, Statement, Statements}, Data, IntoDataRef, SData, SDoc, SField, SFunc, SNum, SNumType, SParam, SType, SUnits, SVal};
use super::StofEnv;


/// Pest Parser for Stof
#[derive(Parser)]
#[grammar = "src/stof/stof.pest"]
struct StofParser;


lazy_static! {
    static ref MATH_OPS_PARSER: PrattParser<Rule> = {
        use pest::pratt_parser::{Assoc::*, Op};
        use Rule::*;
        PrattParser::new()
            .op(Op::infix(and, Left) | Op::infix(or, Left))
            .op(Op::infix(eq, Left) | Op::infix(neq, Left) | Op::infix(gte, Left) | Op::infix(lte, Left) | Op::infix(gt, Left) | Op::infix(lt, Left))
            .op(Op::infix(add, Left) | Op::infix(sub, Left))
            .op(Op::infix(mul, Left) | Op::infix(div, Left) | Op::infix(rem, Left))
            .op(Op::prefix(unary_minus))
    };
}
enum MathExpr {
    Expr(Expr),
    UnaryMinus(Box<MathExpr>),
    Not(Box<MathExpr>),
    Op {
        lhs: Box<MathExpr>,
        op: MathOp,
        rhs: Box<MathExpr>,
    }
}
impl MathExpr {
    pub fn to_expr(self) -> Expr {
        match self {
            Self::Expr(expr) => expr,
            Self::Not(val) => Expr::Not(Box::new(val.to_expr())),
            Self::UnaryMinus(val) => Expr::Mul(vec![Expr::Literal(SVal::Number(SNum::I64(-1))), val.to_expr()]),
            Self::Op { lhs, op, rhs } => {
                let lhs = lhs.to_expr();
                let rhs = rhs.to_expr();
                match op {
                    MathOp::And => Expr::And(vec![lhs, rhs]),
                    MathOp::Or => Expr::Or(vec![lhs, rhs]),
                    MathOp::Add => Expr::Add(vec![lhs, rhs]),
                    MathOp::Sub => Expr::Sub(vec![lhs, rhs]),
                    MathOp::Mul => Expr::Mul(vec![lhs, rhs]),
                    MathOp::Div => Expr::Div(vec![lhs, rhs]),
                    MathOp::Rem => Expr::Rem(vec![lhs, rhs]),
                    MathOp::Eq => Expr::Eq(Box::new(lhs), Box::new(rhs)),
                    MathOp::Neq => Expr::Neq(Box::new(lhs), Box::new(rhs)),
                    MathOp::Gte => Expr::Gte(Box::new(lhs), Box::new(rhs)),
                    MathOp::Lte => Expr::Lte(Box::new(lhs), Box::new(rhs)),
                    MathOp::Gt => Expr::Gt(Box::new(lhs), Box::new(rhs)),
                    MathOp::Lt => Expr::Lt(Box::new(lhs), Box::new(rhs)),
                }
            }
        }
    }
}
enum MathOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Neq,
    Gte,
    Lte,
    Gt,
    Lt,
    And,
    Or,
}


/// Parse internal.
pub fn parse_internal(src: &str, doc: &mut SDoc, env: &mut StofEnv) -> Result<()> {
    let pairs = StofParser::parse(Rule::document, src)?;
    for pair in pairs {
        match pair.as_rule() {
            Rule::document => {
                parse_statements(doc, env, pair.into_inner())?;
            },
            _ => {

            }
        }
    }
    Ok(())
}


/// Parse document statement.
fn parse_statements(doc: &mut SDoc, env: &mut StofEnv, pairs: Pairs<Rule>) -> Result<()> {
    for pair in pairs {
        let span = pair.as_span();
        match pair.as_rule() {
            Rule::import => {
                let mut import_path = String::default();
                let mut import_ext = String::default();
                let mut as_name = "root".to_owned();
                let mut set_as_name = false;
                let mut format = String::default();
                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::import_format => {
                            for pair in pair.into_inner() {
                                match pair.as_rule() {
                                    Rule::ident => {
                                        format = pair.as_str().to_owned();
                                    },
                                    _ => {}
                                }
                            }
                        },
                        Rule::path => {
                            for pair in pair.into_inner() {
                                match pair.as_rule() {
                                    Rule::inner_path => {
                                        import_path = pair.as_str().to_owned();
                                        import_path = import_path.trim_start_matches("\"").trim_end_matches("\"").to_string();
                                        import_path = import_path.trim_start_matches("'").trim_end_matches("'").to_string();
                                        
                                        let pth: Vec<&str> = import_path.split('.').collect();
                                        import_ext = pth.last().unwrap().to_string();
                                    },
                                    _ => return Err(anyhow!("Unrecognized inner path rule"))
                                }
                            }
                        },
                        Rule::ident => {
                            as_name = pair.as_str().to_owned();
                            set_as_name = true;
                        },
                        rule => return Err(anyhow!("Unrecognized import rule: {:?}", rule))
                    }
                }
                if format.len() < 1 {
                    format = import_ext.clone();
                }

                // Perform the file import
                if import_path.len() > 0 {
                    if !set_as_name {
                        let scope = env.scope(doc);
                        as_name = scope.path(&doc.graph).replace('/', ".");
                    } else if as_name.starts_with("self") || as_name.starts_with("super") {
                        let scope = env.scope(doc);
                        let path = scope.path(&doc.graph).replace('/', ".");
                        as_name = format!("{}.{}", path, as_name);
                    }

                    let compiled_path = format!("{}{}{}{}", &format, &import_path, &import_ext, &as_name);
                    if !env.compiled_path(&compiled_path) { // Don't import the same thing more than once!
                        doc.file_import(&env.pid, &format, &import_path, &import_ext, &as_name)?;
                        env.add_compiled_path(&compiled_path);
                    }
                }
            },
            Rule::function => {
                let mut func = parse_function(doc, env, pair)?;
                let scope = env.scope(doc);

                // Function decorators - before func gets attached to the graph
                let mut dec_val = func.attributes.get("@");
                if dec_val.is_none() { dec_val = func.attributes.get("decorator") }
                if let Some(dec_val) = dec_val {
                    let mut success = false;
                    match dec_val {
                        SVal::FnPtr(dref) => {
                            if let Ok(decorator) = SData::data::<SFunc>(&doc.graph, dref) {
                                if decorator.params.len() == 1 && decorator.params[0].ptype == SType::FnPtr && decorator.rtype == SType::FnPtr {
                                    // Make func a random name and attach to the graph
                                    let funcparamname = decorator.params[0].name.clone();
                                    let funcname = func.name.clone();
                                    func.name = format!("decfn_{}", nanoid!(7));
                                    func.attach(&scope, &mut doc.graph);

                                    // Call the decorator function with the func as the parameter
                                    if let Ok(res_val) = decorator.call(&env.pid, doc, vec![SVal::FnPtr(func.data_ref())], true) {
                                        match res_val {
                                            SVal::FnPtr(res_ref) => {
                                                if let Ok(mut res_func) = SData::data::<SFunc>(&doc.graph, res_ref) {
                                                    res_func.name = funcname;
                                                    
                                                    let old_statments = res_func.statements.clone();
                                                    res_func.statements = Statements::from(vec![Statement::Declare(funcparamname, Expr::Literal(SVal::FnPtr(func.data_ref())))]);
                                                    res_func.statements.absorb(old_statments);

                                                    res_func.set(&mut doc.graph);
                                                    res_func.attach(&scope, &mut doc.graph); // Make sure it's in the current scope too
                                                    success = true;
                                                }
                                            },
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        },
                        _ => {
                            success = false;
                        }
                    }
                    if !success {
                        return Err(anyhow!("Cannot decorate a function with any value other than another function"));
                    }
                } else {
                    func.attach(&scope, &mut doc.graph);

                    // Is an init function?
                    if let Some(init_param_val) = func.attributes.get("init") {
                        if init_param_val.is_empty() { // null or void
                            env.init_funcs.push((func.clone(), vec![]));
                        } else {
                            env.init_funcs.push((func.clone(), vec![init_param_val.clone()]));
                        }
                    }

                    // Is a field also?
                    if let Some(field_val) = func.attributes.get("field") {
                        let add = field_val.is_empty() || field_val.truthy();
                        if add {
                            let mut field = SField::new(&func.name, SVal::FnPtr(func.data_ref()));
                            field.attributes.insert("export".to_owned(), SVal::Bool(false));
                            field.attach(&env.scope(doc), &mut doc.graph);
                        }
                    }
                }
            },
            Rule::ref_field => {
                let mut field_path = String::default();
                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::string => {
                            field_path = pair.as_str().to_owned();
                            field_path = field_path.trim_start_matches('"').to_owned();
                            field_path = field_path.trim_end_matches('"').to_owned();
                            field_path = field_path.trim_start_matches('\'').to_owned();
                            field_path = field_path.trim_end_matches('\'').to_owned();
                        },
                        Rule::ident => {
                            field_path = pair.as_str().to_owned();
                        },
                        rule => return Err(anyhow!("Unrecognized rule for field reference: {:?}", rule))
                    }
                }
                if let Some(mut field) = SField::field(&doc.graph, &field_path, '.', Some(&env.scope(doc))) {
                    field.attach(&env.scope(doc), &mut doc.graph);
                } else if let Some(mut field) = SField::field(&doc.graph, &field_path, '.', None) {
                    field.attach(&env.scope(doc), &mut doc.graph);
                } else if let Some(mut func) = SFunc::func(&doc.graph, &field_path, '.', Some(&env.scope(doc))) {
                    func.attach(&env.scope(doc), &mut doc.graph);
                } else if let Some(mut func) = SFunc::func(&doc.graph, &field_path, '.', None) {
                    func.attach(&env.scope(doc), &mut doc.graph);
                }
            },
            Rule::typed_field |
            Rule::field => {
                let mut attributes = BTreeMap::new();
                parse_field(doc, env, pair, "", &mut attributes)?;
            },
            Rule::json_fields => {
                parse_statements(doc, env, pair.into_inner())?;
            },
            Rule::stof_type_declaration => {
                let mut ident = String::default();
                let mut extends = String::default();
                let mut params = Vec::new();
                let mut functions = Vec::new();
                let mut attributes = BTreeMap::new();
                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::stof_type_attribute => {
                            let mut key = String::default();
                            let mut value = SVal::Null;
                            for pair in pair.into_inner() {
                                match pair.as_rule() {
                                    Rule::ident => {
                                        key = pair.as_str().to_string();
                                    },
                                    Rule::expr => {
                                        let value_expr = parse_expression(doc, env, pair)?;
                                        let result = value_expr.exec(&env.pid, doc);
                                        match result {
                                            Ok(sval) => {
                                                value = sval;
                                            },
                                            Err(message) => {
                                                return Err(anyhow!("Unable to execute attribute expression {}", message));
                                            }
                                        }
                                    },
                                    _ => {
                                        return Err(anyhow!("Unrecognized rule for function attribute"));
                                    }
                                }
                            }
                            attributes.insert(key, value);
                        },
                        Rule::ident => {
                            if ident.len() < 1 {
                                ident = pair.as_str().to_owned();
                            } else {
                                extends = pair.as_str().to_owned();
                            }
                        },
                        Rule::stof_type_field => {
                            let mut field_name = String::default();
                            let mut stype = SType::Void;
                            let mut default = None;
                            for pair in pair.into_inner() {
                                match pair.as_rule() {
                                    Rule::ident => {
                                        field_name = pair.as_str().to_owned();
                                    },
                                    Rule::atype => {
                                        stype = SType::from(pair.as_str());
                                    },
                                    Rule::expr => {
                                        default = Some(parse_expression(doc, env, pair)?);
                                    },
                                    _ => {}
                                }
                            }
                            params.push(SParam::new(&field_name, stype, default));
                        },
                        Rule::function => {
                            functions.push(parse_function(doc, env, pair)?);
                        },
                        _ => {}
                    }
                }
                if ident.len() > 0 {
                    let mut ctype = CustomType::new(&env.scope(doc).id, &ident, params, functions);
                    ctype.attributes = attributes;
                    doc.types.declare(ctype, &mut doc.graph, &extends)?;
                }
            },
            Rule::EOI => {
                // nada...
            },
            rule => {
                return Err(anyhow!("Unrecognized document level rule for input: \"{}\", {:?}", span.as_str(), rule));
            }
        }
    }
    Ok(())
}


/// Parse a field.
fn parse_field(doc: &mut SDoc, env: &mut StofEnv, pair: Pair<Rule>, field_type: &str, attributes: &mut BTreeMap<String, SVal>) -> Result<()> {
    match pair.as_rule() {
        Rule::typed_field => {
            let mut stype = String::default();
            for pair in pair.into_inner() {
                match pair.as_rule() {
                    Rule::field_attribute => {
                        let mut key = String::default();
                        let mut value = SVal::Null;
                        for pair in pair.into_inner() {
                            match pair.as_rule() {
                                Rule::ident => {
                                    key = pair.as_str().to_string();
                                },
                                Rule::expr => {
                                    let value_expr = parse_expression(doc, env, pair)?;
                                    let result = value_expr.exec(&env.pid, doc);
                                    match result {
                                        Ok(sval) => {
                                            value = sval;
                                        },
                                        Err(message) => {
                                            return Err(anyhow!("Unable to execute attribute expression {}", message));
                                        }
                                    }
                                },
                                _ => {
                                    return Err(anyhow!("Unrecognized rule for function attribute"));
                                }
                            }
                        }
                        attributes.insert(key, value);
                    },
                    Rule::atype => {
                        stype = pair.as_str().to_owned();
                    },
                    Rule::field => {
                        parse_field(doc, env, pair, &stype, attributes)?;
                    },
                    _ => {}
                }
            }
        },
        Rule::field => {
            let mut field_name = String::default();
            let mut field_value = SVal::Null;
            let mut object_declaration = false;
            for pair in pair.into_inner() {
                match pair.as_rule() {
                    Rule::field_attribute => {
                        let mut key = String::default();
                        let mut value = SVal::Null;
                        for pair in pair.into_inner() {
                            match pair.as_rule() {
                                Rule::ident => {
                                    key = pair.as_str().to_string();
                                },
                                Rule::expr => {
                                    let value_expr = parse_expression(doc, env, pair)?;
                                    let result = value_expr.exec(&env.pid, doc);
                                    match result {
                                        Ok(sval) => {
                                            value = sval;
                                        },
                                        Err(message) => {
                                            return Err(anyhow!("Unable to execute attribute expression {}", message));
                                        }
                                    }
                                },
                                _ => {
                                    return Err(anyhow!("Unrecognized rule for function attribute"));
                                }
                            }
                        }
                        attributes.insert(key, value);
                    },
                    Rule::string => {
                        field_name = pair.as_str().to_owned();
                        field_name = field_name.trim_start_matches('"').to_owned();
                        field_name = field_name.trim_end_matches('"').to_owned();
                        field_name = field_name.trim_start_matches('\'').to_owned();
                        field_name = field_name.trim_end_matches('\'').to_owned();
                    },
                    Rule::ident => {
                        field_name = pair.as_str().to_owned();
                    },
                    Rule::value => {
                        (field_value, object_declaration) = parse_value(field_type, &field_name, doc, env, pair)?;
                    },
                    rule => return Err(anyhow!("Unrecognized rule for field: {:?}", rule))
                }
            }
            if field_name.len() > 0 && !object_declaration { // parse_value takes care of object declarations!
                let list: Vec<&str> = field_name.split('.').collect();
                let last = list.last().unwrap().to_string();

                let mut field = SField::new(&last, field_value);
                field.attributes = attributes.clone();
                env.insert_field(doc, &env.scope(&doc), &mut field);
            }
        },
        rule => return Err(anyhow!("Unrecognized inline json import rule: {:?}", rule))
    }
    Ok(())
}


/// Parse value.
fn parse_value(field_type: &str, field_name: &str, doc: &mut SDoc, env: &mut StofEnv, pair: Pair<Rule>) -> Result<(SVal, bool)> {
    let mut field_value = SVal::Null;
    let mut object_declaration = false;
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::object_value => {
                let list: Vec<&str> = field_name.split('.').collect();
                let mut path;
                let mut fields = true;
                if field_type.len() > 0 && field_type.to_lowercase() == "root" {
                    path = list.join("/");
                    fields = false; // no fields for root nodes
                } else {
                    let current_scope = &env.scope(doc);
                    let current_path = current_scope.path(&doc.graph);
                    path = format!("{}/{}", current_path, list.join("/"));
                }

                // For array objects, don't create fields...
                if field_name.starts_with("_a_obj") {
                    fields = false;
                }

                // Check to see if this object collides with an existing field in the current scope
                // If so, it will be added to an array... so the name should be unique and fields shouldn't be created for it
                let collision_field = SField::field(&doc.graph, &path, '/', None);
                if collision_field.is_some() {
                    fields = false;
                    let name = format!("_a_obj{}", nanoid!(7));
                    let mut tmp: Vec<&str> = path.split('/').collect();
                    tmp.pop();
                    tmp.push(&name);
                    path = tmp.join("/");
                }

                // Create the fields needed and add the scope to the table
                let created = env.push_scope(doc, &path, '/', fields);

                // If there was a collision field, union that field with the newly created object
                if let Some(mut collision_field) = collision_field {
                    let new_field = SField::new(&collision_field.name, SVal::Object(created));
                    collision_field.union(&new_field);
                    collision_field.set(&mut doc.graph);
                }

                // Set the field value to the newly created scope
                field_value = SVal::Object(env.scope(doc));
                object_declaration = true;

                // Now keep parsing statements in this object
                parse_statements(doc, env, pair.into_inner())?;

                // Cast this expression to another type (if possible)!
                if field_type.len() > 0 {
                    let stype = SType::from(field_type);
                    if stype.is_object() && field_type != "obj" && field_type != "root" {
                        field_value = field_value.cast(stype, &env.pid, doc)?;
                    } else if !stype.is_object() {
                        return Err(anyhow!("Cannot cast an object to a non-object type"));
                    }
                }

                // Pop the scope
                env.pop_scope(doc);
            },
            Rule::array_value => {
                let mut array = Vec::new();
                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::value => {
                            let name = format!("_a_obj{}", nanoid!(7));
                            array.push(parse_value("",&name, doc, env, pair)?.0);
                        },
                        _ => {}
                    }
                }
                field_value = SVal::Array(array);
            },
            Rule::expr => {
                let mut expr = parse_expression(doc, env, pair)?;

                if field_type.len() > 0 {
                    let stype = SType::from(field_type);
                    expr = Expr::Cast(stype, Box::new(expr));
                }

                let result = expr.exec(&env.pid, doc);
                match result {
                    Ok(sval) => {
                        field_value = sval;
                    },
                    Err(message) => {
                        return Err(anyhow!("{}", message));
                    }
                }
            },
            rule => return Err(anyhow!("Unrecognized inline json value rule: {:?}", rule))
        }
    }
    Ok((field_value, object_declaration))
}


/// Parse a function to declare in the current scope.
fn parse_function(doc: &mut SDoc, env: &mut StofEnv, pair: Pair<Rule>) -> Result<SFunc> {
    let mut name = String::from("arrow");
    let mut params = Vec::new();
    let mut rtype = SType::Void;
    let mut statements = Statements::default();
    let mut attributes = BTreeMap::new();
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::func_attribute => {
                let mut key = String::default();
                let mut value = SVal::Null;
                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::ident => {
                            key = pair.as_str().to_string();
                        },
                        Rule::expr => {
                            let value_expr = parse_expression(doc, env, pair)?;
                            let result = value_expr.exec(&env.pid, doc);
                            match result {
                                Ok(sval) => {
                                    value = sval;
                                },
                                Err(message) => {
                                    return Err(anyhow!("Unable to execute attribute expression {}", message));
                                }
                            }
                        },
                        _ => {
                            return Err(anyhow!("Unrecognized rule for function attribute"));
                        }
                    }
                }
                attributes.insert(key, value);
            },
            Rule::ident => {
                name = pair.as_str().to_owned();
            },
            Rule::func_param => {
                let mut id = String::default();
                let mut atype = SType::Void;
                let mut default = None;
                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::ident => {
                            id = pair.as_str().to_owned();
                        },
                        Rule::atype => {
                            atype = pair.as_str().into();
                        },
                        Rule::expr => {
                            default = Some(parse_expression(doc, env, pair)?);
                        },
                        _ => {
                            return Err(anyhow!("Unrecognized rule for function parameter"));
                        }
                    }
                }
                params.push(SParam::new(&id, atype, default));
            },
            Rule::atype => {
                rtype = pair.as_str().into();
            },
            Rule::block => {
                statements = parse_block(doc, env, pair)?;
            },
            Rule::expr => {
                statements.statements.push(Statement::Return(parse_expression(doc, env, pair)?));
            },
            Rule::EOI => {},
            _ => {
                return Err(anyhow!("Unrecognized rule for function"));
            }
        }
    }

    let mut func = SFunc::new(&name, params, rtype, statements);
    func.attributes = attributes.clone();
    Ok(func)
}


/// Parse a block of statements.
fn parse_block(doc: &mut SDoc, env: &mut StofEnv, pair: Pair<Rule>) -> Result<Statements> {
    let mut statements = Vec::new();
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::return_statement => {
                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::expr => {
                            statements.push(Statement::Return(parse_expression(doc, env, pair)?));
                        },
                        _ => {}
                    }
                }
            },
            Rule::empty_return => {
                statements.push(Statement::EmptyReturn);
            },
            Rule::while_loop => {
                let mut expr = Expr::Literal(SVal::Void);
                let mut while_statements = Statements::default();
                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::expr => {
                            expr = parse_expression(doc, env, pair)?;
                        },
                        Rule::single_block |
                        Rule::block => {
                            while_statements = parse_block(doc, env, pair)?;
                        },
                        rule => return Err(anyhow!("Unrecognized rule in while loop: {:?}", rule))
                    }
                }
                statements.push(Statement::While(expr, while_statements));
            },
            Rule::for_in_loop => { // iterable must have a "len" lib function and an "at" lib function
                let mut inner_statements = Statements::default();
                let mut iterable_expr = Expr::Literal(SVal::Null);
                let mut atype_str = String::default();
                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::atype => {
                            atype_str = pair.as_str().to_owned();
                        },
                        Rule::ident => {
                            // Set the expression to declare with
                            let mut dec_expr = Expr::Call {
                                scope: "iterable".to_string(),
                                name: "at".to_string(),
                                params: vec![Expr::Variable("index".into())],
                            };

                            // Cast this expression to another type (if possible and required)!
                            if atype_str.len() > 0 {
                                let stype = SType::from(atype_str.as_str());
                                dec_expr = Expr::Cast(stype, Box::new(dec_expr));
                            }

                            let var_name = pair.as_str().to_owned();
                            inner_statements.push(Statement::Declare(var_name.into(), dec_expr));
                        },
                        Rule::expr => {
                            // Array (or iterable) expression!
                            iterable_expr = parse_expression(doc, env, pair)?;
                        },
                        Rule::single_block |
                        Rule::block => {
                            inner_statements.absorb(parse_block(doc, env, pair)?);
                        },
                        rule => return Err(anyhow!("Unrecognized rule in for 'in' loop: {:?}", rule))
                    }
                }
                let mut outer_statements = vec![
                    Statement::Declare("iterable".into(), iterable_expr),
                    Statement::Declare("length".into(), Expr::Call {
                        scope: "iterable".to_string(),
                        name: "len".to_string(),
                        params: vec![],
                    }),
                    Statement::Declare("index".into(), Expr::Literal(0.into())),
                    Statement::Declare("first".into(), Expr::Literal(true.into())),
                    Statement::Declare("last".into(), Expr::Literal(false.into())),
                ];
                // Put finally statements together that increases "index" by one on the end of the inner statements
                let finally_statements = Statements::from(vec![
                    Statement::Assign("index".into(), Expr::Add(vec![Expr::Variable("index".into()), Expr::Literal(1.into())])),
                    Statement::Assign("first".into(), Expr::Literal(false.into())),
                    Statement::Assign("last".into(), Expr::Eq(Box::new(Expr::Variable("index".into())), Box::new(Expr::Sub(vec![Expr::Variable("length".into()), Expr::Literal(1.into())])))),
                ]);
                // Create the while loop in the outer statements
                let block_statements = Statements::from(vec![Statement::Block(inner_statements, finally_statements)]);
                outer_statements.push(Statement::While(Expr::Lt(Box::new(Expr::Variable("index".into())), Box::new(Expr::Variable("length".into()))), block_statements));
                // Add the block of statements (new scope for declaration)
                statements.push(Statement::Block(outer_statements.into(), Statements::default()));
            },
            Rule::for_loop => {
                let mut for_statements = Vec::new();
                let mut expr = Expr::Literal(SVal::Bool(true)); // infinite loop by default
                let mut while_statements = Statements::default();
                let mut end_while_statement = Statement::Continue;
                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::declare => {
                            for_statements.push(parse_declare(doc, env, pair)?);
                        },
                        Rule::expr => {
                            expr = parse_expression(doc, env, pair)?;
                        },
                        Rule::single_block |
                        Rule::block => {
                            while_statements = parse_block(doc, env, pair)?;
                        },
                        Rule::assign |
                        Rule::add_assign |
                        Rule::sub_assign |
                        Rule::mul_assign |
                        Rule::div_assign |
                        Rule::rem_assign => {
                            end_while_statement = parse_assignment(doc, env, pair)?;
                        },
                        rule => return Err(anyhow!("Unrecognized rule in for loop: {:?}", rule))
                    }
                }
                // Put finally statements together
                let finally_statements = Statements::from(vec![end_while_statement]);
                // Create the while loop
                for_statements.push(Statement::While(expr, Statements::from(vec![Statement::Block(while_statements, finally_statements)])));
                // Add the block of statements (new scope for declaration)
                statements.push(Statement::Block(for_statements.into(), Statements::default()));
            },
            Rule::break_stat => {
                statements.push(Statement::Break);
            },
            Rule::continue_stat => {
                statements.push(Statement::Continue);
            },
            Rule::if_statement => {
                let mut set_first_expr = false;
                let mut if_expr: (Expr, Statements) = (Expr::Literal(SVal::Void), Default::default());
                let mut elif_exprs: Vec<(Expr, Statements)> = Vec::new();
                let mut else_expr: Option<Statements> = None;

                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::expr => {
                            if set_first_expr {
                                let expr = parse_expression(doc, env, pair)?;
                                if_expr.1 = Statements::from(vec![Statement::Expr(expr)]);
                            } else {
                                if_expr.0 = parse_expression(doc, env, pair)?;
                                set_first_expr = true;
                            }
                        },
                        Rule::single_block |
                        Rule::block => {
                            if_expr.1 = parse_block(doc, env, pair)?;
                        },
                        Rule::else_if_statement => {
                            let mut set_elif_first_expr = false;
                            let mut elif_expr: (Expr, Statements) = (Expr::Literal(SVal::Void), Default::default());
                            for pair in pair.into_inner() {
                                match pair.as_rule() {
                                    Rule::expr => {
                                        if set_elif_first_expr {
                                            let expr = parse_expression(doc, env, pair)?;
                                            elif_expr.1 = Statements::from(vec![Statement::Expr(expr)]);
                                        } else {
                                            elif_expr.0 = parse_expression(doc, env, pair)?;
                                            set_elif_first_expr = true;
                                        }
                                    },
                                    Rule::single_block |
                                    Rule::block => {
                                        elif_expr.1 = parse_block(doc, env, pair)?;
                                    },
                                    rule => {
                                        return Err(anyhow!("Unrecognized rule in else-if-statement: {:?}", rule));
                                    }
                                }
                            }
                            elif_exprs.push(elif_expr);
                        },
                        Rule::else_statement => {
                            for pair in pair.into_inner() {
                                match pair.as_rule() {
                                    Rule::expr => {
                                        else_expr = Some(Statements::from(vec![Statement::Expr(parse_expression(doc, env, pair)?)]));
                                    },
                                    Rule::single_block |
                                    Rule::block => {
                                        else_expr = Some(parse_block(doc, env, pair)?);
                                    },
                                    rule => {
                                        return Err(anyhow!("Unrecognized rule in else statement: {:?}", rule));
                                    }
                                }
                            }
                        },
                        rule => {
                            return Err(anyhow!("Unrecognized rule in if-statement: {:?}", rule));
                        }
                    }
                }

                statements.push(Statement::If {
                    if_expr,
                    elif_exprs,
                    else_expr
                });
            },
            Rule::declare => {
                statements.push(parse_declare(doc, env, pair)?);
            },
            Rule::drop => {
                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::ident => {
                            statements.push(Statement::Drop(pair.as_str().into()));
                        },
                        _ => {
                            return Err(anyhow!("Unrecognized rule in drop statement: \"{}\"", pair.as_span().as_str()));
                        }
                    }
                }
            },
            Rule::move_stat => {
                let mut ident = String::default();
                let mut dest = String::default();
                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::ident => {
                            if ident.is_empty() {
                                ident = pair.as_str().to_owned();
                            } else {
                                dest = pair.as_str().to_owned();
                            }
                        },
                        _ => {
                            return Err(anyhow!("Unrecognized rule in move statement: \"{}\"", pair.as_span().as_str()));
                        }
                    }
                }
                if ident.len() > 0 {
                    statements.push(Statement::Move(ident, dest));
                }
            },
            Rule::rename => {
                let mut ident = String::default();
                let mut expr = Expr::Literal(SVal::Void);
                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::ident => {
                            ident = pair.as_str().to_owned();
                        },
                        Rule::expr => {
                            expr = parse_expression(doc, env, pair)?;
                        },
                        _ => {
                            return Err(anyhow!("Unrecognized rule in move statement: \"{}\"", pair.as_span().as_str()));
                        }
                    }
                }
                if ident.len() > 0 {
                    statements.push(Statement::Rename(ident, expr));
                }
            },
            Rule::assign |
            Rule::add_assign |
            Rule::sub_assign |
            Rule::mul_assign |
            Rule::div_assign |
            Rule::rem_assign => {
                statements.push(parse_assignment(doc, env, pair)?);
            },
            Rule::expr => {
                statements.push(Statement::Expr(parse_expression(doc, env, pair)?));
            },
            Rule::block => {
                let block_statements = parse_block(doc, env, pair)?;
                statements.push(Statement::Block(block_statements, Statements::default()));
            },
            _ => {
                return Err(anyhow!("Unrecognized rule for parsing block of statements"))
            }
        }
    }
    Ok(statements.into())
}


/// Parse declare statement.
fn parse_declare(doc: &mut SDoc, env: &mut StofEnv, pair: Pair<Rule>) -> Result<Statement> {
    let mut ident = String::default();
    let mut expr = Expr::Literal("".into());
    let mut atype = SType::Void;
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::ident => {
                ident = pair.as_str().to_owned();
            },
            Rule::atype => {
                atype = SType::from(pair.as_str());
            },
            Rule::expr => {
                expr = parse_expression(doc, env, pair)?;
            },
            _ => {
                return Err(anyhow!("Unrecognized rule in declare statement (block): \"{}\"", pair.as_span().as_str()));
            }
        }
    }
    if ident.len() > 0 {
        if !atype.is_void() {
            // Assigned a variable with a type... so future assignments must cast to this type
            env.assign_type_stack.last_mut().unwrap().insert(ident.clone(), atype.clone());
            expr = Expr::Cast(atype, Box::new(expr));
        }
        return Ok(Statement::Declare(ident, expr));
    }
    Err(anyhow!("Could not parse declare statement"))
}


/// Parse assignment.
fn parse_assignment(doc: &mut SDoc, env: &mut StofEnv, pair: Pair<Rule>) -> Result<Statement> {
    match pair.as_rule() {
        Rule::assign => {
            let mut ident = String::default();
            let mut expr = Expr::Literal("".into());
            for pair in pair.into_inner() {
                match pair.as_rule() {
                    Rule::ident => {
                        ident = pair.as_str().to_owned();
                    },
                    Rule::expr => {
                        expr = parse_expression(doc, env, pair)?;
                        if let Some(cast_type) = env.assign_type_stack.last().unwrap().get(&ident) {
                            expr = Expr::Cast(cast_type.clone(), Box::new(expr));
                        }
                    },
                    _ => {
                        return Err(anyhow!("Unrecognized rule in assign statement (block): \"{}\"", pair.as_span().as_str()));
                    }
                }
            }
            return Ok(Statement::Assign(ident, expr));
        },
        Rule::add_assign => {
            let mut ident = String::default();
            let mut expr = Expr::Literal("".into());
            for pair in pair.into_inner() {
                match pair.as_rule() {
                    Rule::ident => {
                        ident = pair.as_str().to_owned();
                    },
                    Rule::expr => {
                        expr = parse_expression(doc, env, pair)?;
                        if let Some(cast_type) = env.assign_type_stack.last().unwrap().get(&ident) {
                            expr = Expr::Cast(cast_type.clone(), Box::new(expr));
                        }
                    },
                    _ => {
                        return Err(anyhow!("Unrecognized rule in assign statement (block): \"{}\"", pair.as_span().as_str()));
                    }
                }
            }
            if ident.len() > 0 {
                let var_use = ident.clone();
                return Ok(Statement::Assign(ident, Expr::Add(vec![Expr::Variable(var_use), expr])));
            }
            Err(anyhow!("Not able to parse assignment"))
        },
        Rule::sub_assign => {
            let mut ident = String::default();
            let mut expr = Expr::Literal("".into());
            for pair in pair.into_inner() {
                match pair.as_rule() {
                    Rule::ident => {
                        ident = pair.as_str().to_owned();
                    },
                    Rule::expr => {
                        expr = parse_expression(doc, env, pair)?;
                        if let Some(cast_type) = env.assign_type_stack.last().unwrap().get(&ident) {
                            expr = Expr::Cast(cast_type.clone(), Box::new(expr));
                        }
                    },
                    _ => {
                        return Err(anyhow!("Unrecognized rule in assign statement (block): \"{}\"", pair.as_span().as_str()));
                    }
                }
            }
            if ident.len() > 0 {
                let var_use = ident.clone();
                return Ok(Statement::Assign(ident, Expr::Sub(vec![Expr::Variable(var_use), expr])));
            }
            Err(anyhow!("Not able to parse assignment"))
        },
        Rule::mul_assign => {
            let mut ident = String::default();
            let mut expr = Expr::Literal("".into());
            for pair in pair.into_inner() {
                match pair.as_rule() {
                    Rule::ident => {
                        ident = pair.as_str().to_owned();
                    },
                    Rule::expr => {
                        expr = parse_expression(doc, env, pair)?;
                        if let Some(cast_type) = env.assign_type_stack.last().unwrap().get(&ident) {
                            expr = Expr::Cast(cast_type.clone(), Box::new(expr));
                        }
                    },
                    _ => {
                        return Err(anyhow!("Unrecognized rule in assign statement (block): \"{}\"", pair.as_span().as_str()));
                    }
                }
            }
            if ident.len() > 0 {
                let var_use = ident.clone();
                return Ok(Statement::Assign(ident, Expr::Mul(vec![Expr::Variable(var_use), expr])));
            }
            Err(anyhow!("Not able to parse assignment"))
        },
        Rule::div_assign => {
            let mut ident = String::default();
            let mut expr = Expr::Literal("".into());
            for pair in pair.into_inner() {
                match pair.as_rule() {
                    Rule::ident => {
                        ident = pair.as_str().to_owned();
                    },
                    Rule::expr => {
                        expr = parse_expression(doc, env, pair)?;
                        if let Some(cast_type) = env.assign_type_stack.last().unwrap().get(&ident) {
                            expr = Expr::Cast(cast_type.clone(), Box::new(expr));
                        }
                    },
                    _ => {
                        return Err(anyhow!("Unrecognized rule in assign statement (block): \"{}\"", pair.as_span().as_str()));
                    }
                }
            }
            if ident.len() > 0 {
                let var_use = ident.clone();
                return Ok(Statement::Assign(ident, Expr::Div(vec![Expr::Variable(var_use), expr])));
            }
            Err(anyhow!("Not able to parse assignment"))
        },
        Rule::rem_assign => {
            let mut ident = String::default();
            let mut expr = Expr::Literal("".into());
            for pair in pair.into_inner() {
                match pair.as_rule() {
                    Rule::ident => {
                        ident = pair.as_str().to_owned();
                    },
                    Rule::expr => {
                        expr = parse_expression(doc, env, pair)?;
                        if let Some(cast_type) = env.assign_type_stack.last().unwrap().get(&ident) {
                            expr = Expr::Cast(cast_type.clone(), Box::new(expr));
                        }
                    },
                    _ => {
                        return Err(anyhow!("Unrecognized rule in assign statement (block): \"{}\"", pair.as_span().as_str()));
                    }
                }
            }
            if ident.len() > 0 {
                let var_use = ident.clone();
                return Ok(Statement::Assign(ident, Expr::Rem(vec![Expr::Variable(var_use), expr])));
            }
            Err(anyhow!("Not able to parse assignment"))
        },
        rule => Err(anyhow!("Unrecognized assignment rule: {:?}", rule))
    }
}


/// Parse expressions (expr rule).
/// SType is available for parsing if able.
fn parse_expression(doc: &mut SDoc, env: &mut StofEnv, pair: Pair<Rule>) -> Result<Expr> {
    let mut res: Expr = Expr::Literal(SVal::Null);
    for pair in pair.into_inner() {
        match pair.as_rule() {
            Rule::atype => {
                // Cast this expression to another type (if possible)!
                let stype = SType::from(pair.as_str());
                res = Expr::Cast(stype, Box::new(res));
            },
            Rule::if_expr => { // Ternary expression
                let mut set_first = false;
                let mut if_expr = Expr::Literal(SVal::Bool(true));
                let mut first = Expr::Literal(SVal::Void);
                let mut second = Expr::Literal(SVal::Void);
                for pair in pair.into_inner() {
                    match pair.as_rule() {
                        Rule::expr => {
                            if set_first {
                                second = parse_expression(doc, env, pair)?;
                            } else {
                                first = parse_expression(doc, env, pair)?;
                                set_first = true;
                            }
                        },
                        _ => {
                            if_expr = parse_expr_pair(doc, env, pair)?;
                        }
                    }
                }
                let if_statement = Statement::If {
                    if_expr: (if_expr, Statements::from(vec![Statement::Return(first)])),
                    elif_exprs: vec![],
                    else_expr: Some(Statements::from(vec![Statement::Return(second)])),
                };
                res = Expr::Block(Statements::from(vec![if_statement]));
            },
            _ => {
                res = parse_expr_pair(doc, env, pair)?;
            }
        }
    }
    Ok(res)
}


/// Parse expression pair.
fn parse_expr_pair(doc: &mut SDoc, env: &mut StofEnv, pair: Pair<Rule>) -> Result<Expr> {
    let mut res: Expr = Expr::Literal(SVal::Null);
    match pair.as_rule() {
        Rule::type_of_expr => {
            for pair in pair.into_inner() {
                match pair.as_rule() {
                    Rule::expr => {
                        res = Expr::TypeOf(Box::new(parse_expression(doc, env, pair)?));
                    },
                    _ => {}
                }
            }
        },
        Rule::type_name_expr => {
            for pair in pair.into_inner() {
                match pair.as_rule() {
                    Rule::expr => {
                        res = Expr::TypeName(Box::new(parse_expression(doc, env, pair)?));
                    },
                    _ => {}
                }
            }
        },
        Rule::not_expr => {
            for pair in pair.into_inner() {
                match pair.as_rule() {
                    Rule::expr => {
                        res = Expr::Not(Box::new(parse_expression(doc, env, pair)?));
                    },
                    _ => {}
                }
            }
        },
        Rule::math_expr => {
            res = parse_math_pairs(doc, env, pair.into_inner()).to_expr();
        },
        Rule::tuple_expr => {
            let mut vec: Vec<Expr> = Vec::new();
            for pair in pair.into_inner() {
                match pair.as_rule() {
                    Rule::expr => {
                        vec.push(parse_expression(doc, env, pair)?);
                    },
                    rule => {
                        return Err(anyhow!("Unrecognized rule in tuple constructor: {:?}", rule));
                    }
                }
            }
            res = Expr::Tuple(vec);
        },
        Rule::array_expr => {
            let mut vec: Vec<Expr> = Vec::new();
            for pair in pair.into_inner() {
                match pair.as_rule() {
                    Rule::expr => {
                        vec.push(parse_expression(doc, env, pair)?);
                    },
                    rule => {
                        return Err(anyhow!("Unrecognized rule in tuple constructor: {:?}", rule));
                    }
                }
            }
            res = Expr::Array(vec);
        },
        Rule::range_expr => {
            let mut seen_first = false;
            let mut first = 0;
            let mut seen_second = false;
            let mut second = 0;
            let mut step = 1;
            for pair in pair.into_inner() {
                match pair.as_rule() {
                    Rule::number => {
                        for pair in pair.into_inner() {
                            match pair.as_rule() {
                                Rule::decimal => {
                                    let val_str = pair.as_str().replace('+', "");
                                    let val = val_str.parse::<f64>().unwrap();
                                    let val = val as usize;
                                    if seen_first && seen_second {
                                        step = val;
                                    } else if seen_first {
                                        second = val;
                                        seen_second = true;
                                    } else {
                                        first = val;
                                        seen_first = true;
                                    }
                                },
                                Rule::integer => {
                                    let val_str = pair.as_str().replace('+', "");
                                    let val = val_str.parse::<i64>().unwrap();
                                    let val = val as usize;
                                    if seen_first && seen_second {
                                        step = val;
                                    } else if seen_first {
                                        second = val;
                                        seen_second = true;
                                    } else {
                                        first = val;
                                        seen_first = true;
                                    }
                                },
                                Rule::units => {
                                    // Do nothing with units here...
                                },
                                _ => {
                                    return Err(anyhow!("Unrecognized rule for number literal"));
                                }
                            }
                        }
                    },
                    _ => {}
                }
            }
            let mut vec: Vec<Expr> = Vec::new();
            for x in (first..second).step_by(step) {
                vec.push(Expr::Literal(SVal::Number((x as i64).into())));
            }
            res = Expr::Array(vec);
        },
        Rule::index_expr => {
            let mut scope = String::default();
            let mut params = Vec::new();
            for pair in pair.into_inner() {
                match pair.as_rule() {
                    Rule::ident => {
                        let path: Vec<&str> = pair.as_str().split(".").collect();
                        if path.len() > 0 {
                            scope = path.join("/");
                        } else {
                            return Err(anyhow!("Did not find a scope and name for index expr"));
                        }
                    },
                    Rule::expr => {
                        params.push(parse_expression(doc, env, pair)?);
                    },
                    _ => {}
                }
            }
            if scope != String::default() {
                res = Expr::Call { scope, name: "at".into(), params };
            } else {
                return Err(anyhow!("Unable to parse index expression into 'at' call expr"));
            }
        },
        Rule::chain_index => {
            let mut scope = String::default();
            let mut block_statements = Vec::new();
            let mut declared = false;
            for pair in pair.into_inner() {
                match pair.as_rule() {
                    Rule::ident => {
                        let path: Vec<&str> = pair.as_str().split(".").collect();
                        if path.len() > 0 {
                            scope = path.join("/");
                        } else {
                            return Err(anyhow!("Did not find a scope and name for index expr"));
                        }
                    },
                    Rule::chain_index_inner => {
                        let mut params = Vec::new();
                        for pair in pair.into_inner() {
                            match pair.as_rule() {
                                Rule::expr => {
                                    params.push(parse_expression(doc, env, pair)?);
                                },
                                _ => {}
                            }
                        }
                        if !declared {
                            declared = true;
                            block_statements.push(Statement::Declare("tmp".into(), Expr::Call { scope: scope.clone(), name: "at".into(), params }));
                        } else {
                            block_statements.push(Statement::Assign("tmp".into(), Expr::Call { scope: scope.clone(), name: "at".into(), params }));
                        }
                        scope = "tmp".to_owned();
                    },
                    _ => {}
                }
            }
            block_statements.push(Statement::Return(Expr::Variable("tmp".into())));
            res = Expr::Block(Statements::from(block_statements));
        },
        Rule::expr_call => {
            let mut block_statements = Vec::new();
            for pair in pair.into_inner() {
                match pair.as_rule() {
                    Rule::call => {
                        let mut expr = parse_expr_pair(doc, env, pair)?;
                        match &mut expr {
                            Expr::Call { scope, name: _, params: _ } => {
                                *scope = "tmp".to_string();
                            },
                            _ => unreachable!()
                        }
                        block_statements.push(Statement::Return(expr));
                    },
                    _ => {
                        block_statements.push(Statement::Declare("tmp".into(), parse_expr_pair(doc, env, pair)?));
                    }
                }
            }
            res = Expr::Block(block_statements.into());
        },
        Rule::chain_call => {
            let mut block_statements = Vec::new();
            let mut declared = false;
            for pair in pair.into_inner() {
                match pair.as_rule() {
                    Rule::expr_call |
                    Rule::call => {
                        let mut expr = parse_expr_pair(doc, env, pair)?;
                        if declared {
                            match &mut expr {
                                Expr::Call { scope, name: _, params: _ } => {
                                    *scope = "tmp".to_string();
                                },
                                _ => unreachable!()
                            }
                        }
                        if !declared {
                            declared = true;
                            block_statements.push(Statement::Declare("tmp".into(), expr));
                        } else {
                            block_statements.push(Statement::Assign("tmp".into(), expr));
                        }
                    },
                    _ => {}
                }
            }
            block_statements.push(Statement::Return(Expr::Variable("tmp".into())));
            res = Expr::Block(block_statements.into());
        },
        Rule::call => {
            let mut scope = String::default();
            let mut ident = String::default();
            let mut params = Vec::new();
            for pair in pair.into_inner() {
                match pair.as_rule() {
                    Rule::ident => {
                        let mut path: Vec<&str> = pair.as_str().split(".").collect();
                        if path.len() > 1 {
                            ident = path.pop().unwrap().into();
                            scope = path.join("/");
                        } else if path.len() > 0 {
                            scope = "std".into();
                            ident = path.pop().unwrap().into();
                        } else {
                            return Err(anyhow!("Did not find a scope and name for call expr"));
                        }
                    },
                    Rule::call_params => {
                        for pair in pair.into_inner() {
                            match pair.as_rule() {
                                Rule::expr => {
                                    let expr = parse_expression(doc, env, pair)?;
                                    params.push(expr);
                                },
                                _ => {
                                    return Err(anyhow!("Cannot have a non-expression call parameter"));
                                }
                            }
                        }
                    },
                    _ => {

                    }
                }
            }
            if ident != String::default() {
                res = Expr::Call { scope, name: ident, params };
            } else {
                return Err(anyhow!("Unable to parse call expression"));
            }
        },
        Rule::block => {
            let statements = parse_block(doc, env, pair)?;
            res = Expr::Block(statements);
        },
        Rule::fmt_expr => {
            for pair in pair.into_inner() {
                match pair.as_rule() {
                    Rule::fmt_expr_i => {
                        let format = pair.as_str();
                        let mut statements = Vec::new();
                        statements.push(Statement::Declare("tmp".into(), Expr::Literal(SVal::String(format.to_owned()))));
                        statements.push(Statement::Declare("res".into(), Expr::Literal(SVal::String(format.to_owned()))));
                        statements.push(Statement::Declare("sub".into(), Expr::Literal(SVal::Null)));

                        let fmt_span = pair.as_span();
                        let fmt_start = fmt_span.start();

                        for pair in pair.into_inner() {
                            match pair.as_rule() {
                                Rule::fmt_inner => {
                                    let span = pair.as_span();
                                    for pair in pair.into_inner() {
                                        match pair.as_rule() {
                                            Rule::expr => {
                                                // Parse the val expression
                                                let val = parse_expression(doc, env, pair)?;

                                                // Assign the substring
                                                let start = span.start() - fmt_start;
                                                let end = span.end() - fmt_start;
                                                statements.push(Statement::Assign("sub".into(), Expr::Call {
                                                    scope: "tmp".into(),
                                                    name: "substring".into(),
                                                    params: vec![Expr::Literal(SVal::Number(SNum::I64(start as i64))), Expr::Literal(SVal::Number(SNum::I64(end as i64)))],
                                                }));

                                                // Do the replacement
                                                statements.push(Statement::Assign("res".into(), Expr::Call {
                                                    scope: "res".into(),
                                                    name: "replace".into(),
                                                    params: vec![Expr::Variable("sub".into()), Expr::Cast(SType::String, Box::new(val))],
                                                }));
                                            },
                                            _ => {}
                                        }
                                    }
                                },
                                _ => {}
                            }
                        }

                        statements.push(Statement::Return(Expr::Variable("res".into())));
                        res = Expr::Block(Statements::from(statements));
                    },
                    _ => {}
                }
            }
        },
        Rule::literal => {
            for pair in pair.into_inner() {
                match pair.as_rule() {
                    Rule::bool => {
                        let val = pair.as_str().parse::<bool>().unwrap();
                        res = Expr::Literal(SVal::Bool(val));
                    },
                    Rule::null => {
                        res = Expr::Literal(SVal::Null);
                    },
                    Rule::string => {
                        let val = pair.as_str()
                            .trim_start_matches("\"")
                            .trim_end_matches("\"")
                            .trim_start_matches("'")
                            .trim_end_matches("'")
                            .replace("\\n", "\n")
                            .replace("\\t", "\t")
                            .replace("\\\\", "\\");
                        res = Expr::Literal(SVal::String(val));
                    },
                    Rule::number => {
                        let mut number = SNum::I64(0);
                        for pair in pair.into_inner() {
                            match pair.as_rule() {
                                Rule::decimal => {
                                    let val_str = pair.as_str().replace('+', "");
                                    let val = val_str.parse::<f64>().unwrap();
                                    number = SNum::F64(val);
                                },
                                Rule::integer => {
                                    let val_str = pair.as_str().replace('+', "");
                                    let val = val_str.parse::<i64>().unwrap();
                                    number = SNum::I64(val);
                                },
                                Rule::units => {
                                    let units = SUnits::from(pair.as_str());
                                    if units.has_units() && !units.is_undefined() {
                                        number = number.cast(SNumType::Units(units));
                                    }
                                },
                                _ => {
                                    return Err(anyhow!("Unrecognized rule for number literal"));
                                }
                            }
                        }
                        res = Expr::Literal(SVal::Number(number));
                    },
                    Rule::ident => {
                        // This one is special for now, its actually a variable usage!
                        res = Expr::Variable(pair.as_str().into());
                    },
                    _ => {
                        return Err(anyhow!("Unknown expression literal: {}", pair.as_span().as_str()));
                    }
                }
            }
        },
        Rule::wrapped_expr => {
            let mut unary = false;
            let mut not = false;
            for pair in pair.into_inner() {
                match pair.as_rule() {
                    Rule::unary_minus => {
                        unary = true;
                    },
                    Rule::bang => {
                        not = true;
                    },
                    Rule::expr => {
                        res = parse_expression(doc, env, pair)?;
                    },
                    _ => {}
                }
            }
            if unary {
                res = Expr::Mul(vec![Expr::Literal(SVal::Number(SNum::I64(-1))), res]);
            }
            if not {
                res = Expr::Not(Box::new(res));
            }
        },
        Rule::arrow_function => {
            let mut function = parse_function(doc, env, pair)?;
            // Set anonymous name for function
            let func_name = format!("func{}", nanoid!(7));
            function.name = func_name;
            
            // Declare the function in the current scope
            let scope = env.scope(doc);
            function.attach(&scope, &mut doc.graph);

            // Return a function pointer literal
            res = Expr::Literal(SVal::FnPtr(function.data_ref()));
        },
        Rule::stof_type_constructor => {
            let mut type_name = String::from("<anonymous>");
            let mut prototype_location = String::default();
            let mut prototype_fields = Vec::new();
            let mut block_statements = Vec::new();
            for pair in pair.into_inner() {
                match pair.as_rule() {
                    Rule::ident => {
                        let mut type_path: Vec<&str> = pair.as_str().split('.').collect();
                        let custom_type_name = type_path.pop().unwrap();

                        // Find a scope to use other than our own?
                        let scope = env.scope(doc);
                        let mut type_scope = scope.clone();
                        if type_path.len() > 0 {
                            let path = type_path.join("/");
                            if path.starts_with("self") || path.starts_with("super") {
                                if let Some(nref) = doc.graph.node_ref(&path, Some(&type_scope)) {
                                    type_scope = nref;
                                } else {
                                    return Err(anyhow!("Cannot find referenced type scope for constructing an object"));
                                }
                            } else {
                                if let Some(nref) = doc.graph.node_ref(&path, None) {
                                    type_scope = nref;
                                } else {
                                    return Err(anyhow!("Cannot find referenced type scope for constructing an object"));
                                }
                            }
                        }

                        if let Some(custom_type) = doc.types.find(&doc.graph, custom_type_name, &type_scope) {
                            if custom_type.is_private() && !scope.is_child_of(&doc.graph, &type_scope) {
                                // Custom type is private and the current scope is not equal or a child of the type's scope
                                return Err(anyhow!("Cannot cast object expr to private type: {}", pair.as_str()));
                            }
                            type_name = custom_type.name.clone();
                            prototype_location = custom_type.path(&doc.graph);
                            prototype_fields = custom_type.fields.clone();
                        } else {
                            return Err(anyhow!("Attempting to construct a type that has not been defined: {}", pair.as_str()));
                        }
                    },
                    Rule::stof_type_field_create => {
                        let mut field_name = String::default();
                        let mut expr = Expr::Literal(SVal::Null);
                        for pair in pair.into_inner() {
                            match pair.as_rule() {
                                Rule::ident => {
                                    field_name = pair.as_str().to_owned();
                                },
                                Rule::string => {
                                    field_name = pair.as_str().to_owned();
                                    field_name = field_name.trim_start_matches('"').trim_end_matches('"').to_owned();
                                    field_name = field_name.trim_start_matches("'").trim_end_matches("'").to_owned();
                                },
                                Rule::expr => {
                                    expr = parse_expression(doc, env, pair)?;
                                },
                                _ => {}
                            }
                        }
                        if type_name == "<anonymous>" {
                            // No prototype for this object, so just assign the field
                            block_statements.push(Statement::Assign(field_name.into(), expr));
                        } else {
                            let mut found = false;
                            for i in 0..prototype_fields.len() {
                                if prototype_fields[i].name == field_name {
                                    found = true;
                                    let param = prototype_fields.remove(i);
                                    // Cast the expression to the correct (defined) type
                                    let assign_expr = Expr::Cast(param.ptype, Box::new(expr));
                                    block_statements.push(Statement::Assign(field_name.clone().into(), assign_expr));
                                    break; // We found the field
                                }
                            }
                            if !found {
                                return Err(anyhow!("Attempting to set a field '{}' that does not exist on the type: {}", field_name, type_name));
                            }
                        }
                    },
                    _ => {}
                }
            }

            // Any prototype fields left must have defaults defined, or err
            if prototype_fields.len() > 0 {
                for param in prototype_fields {
                    if let Some(default) = param.default {
                        let assign_expr = Expr::Cast(param.ptype, Box::new(default));
                        block_statements.push(Statement::Assign(param.name, assign_expr));
                    } else {
                        return Err(anyhow!("Unconstructed field '{}' on type '{}' does not have a default value given", &param.name, &type_name));
                    }
                }
            }

            // Add the prototype field to the anonymous object for function lookups
            if prototype_location.len() > 0 {
                block_statements.push(Statement::Assign("__prototype__".into(), Expr::Literal(SVal::String(prototype_location))));
            }

            // New object expression - creates the object at runtime under self
            res = Expr::NewObject(Statements::from(block_statements));
        },
        _ => {
            return Err(anyhow!("Unrecognized rule for expression: {}", pair.as_span().as_str()));
        }
    }
    Ok(res)
}


/// Parse math expression pairs.
fn parse_math_pairs(doc: &mut SDoc, env: &mut StofEnv, pairs: Pairs<Rule>) -> MathExpr {
    MATH_OPS_PARSER
        .map_primary(|primary| match primary.as_rule() {
            _ => {
                MathExpr::Expr(parse_expr_pair(doc, env, primary).expect("Expr::parse_math creating expr"))
            },
        })
        .map_infix(|lhs, op, rhs| {
            let op = match op.as_rule() {
                Rule::and => MathOp::And,
                Rule::or => MathOp::Or,
                Rule::add => MathOp::Add,
                Rule::sub => MathOp::Sub,
                Rule::mul => MathOp::Mul,
                Rule::div => MathOp::Div,
                Rule::rem => MathOp::Rem,
                Rule::eq => MathOp::Eq,
                Rule::neq => MathOp::Neq,
                Rule::gte => MathOp::Gte,
                Rule::lte => MathOp::Lte,
                Rule::gt => MathOp::Gt,
                Rule::lt => MathOp::Lt,
                rule => unreachable!("Expr::parse_math expected infix operation, found {:?}", rule)
            };
            MathExpr::Op { lhs: Box::new(lhs), op, rhs: Box::new(rhs) }
        })
        .map_prefix(|op, rhs| match op.as_rule() {
            Rule::unary_minus => MathExpr::UnaryMinus(Box::new(rhs)),
            Rule::bang => MathExpr::Not(Box::new(rhs)),
            _ => unreachable!()
        })
        .parse(pairs)
}
