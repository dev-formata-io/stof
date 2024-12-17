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

use std::collections::{HashMap, HashSet};
use crate::{lang::{Expr, Statement, Statements}, IntoNodeRef, SData, SDoc, SField, SFunc, SGraph, SNode, SNodeRef, SVal, FKIND, FUNC_KIND};


/// Stof export env
#[derive(Default, Debug)]
pub struct StofExportEnv {
    /// Export context stack
    pub context_stack: Vec<SNodeRef>,

    /// Object ID to export name lookup table
    pub object_id_export_name: HashMap<String, String>,
}
impl StofExportEnv {
    /// Relative path for an object ID.
    /// Dot separated.
    pub fn relative_obj_path(&self, id: &str, graph: &SGraph) -> Option<String> {
        let mut path = SNodeRef::from(id).path(graph);
        if let Some(context) = &self.context_stack.last() {
            // Relative path against context
            // path:    root.child.other.parent
            // context: root.child
            // result:  self.other.parent
            let context_path = context.path(graph);
            if context_path == path {
                path = "self".to_string();
            } else {
                let pth: Vec<&str> = path.split('/').collect();
                let context_pth: Vec<&str> = context_path.split('/').collect();
                if pth[0] != context_pth[0] || pth.len() < context_pth.len() {
                    // Not in the same roots or within the same export, so no common path with context
                    return None;
                }
                let mut real_path = Vec::new();
                for i in 0..pth.len() {
                    if i < context_pth.len() && pth[i] == context_pth[i] && real_path.len() < 1 {
                        // nada..
                    } else {
                        real_path.push(pth[i]);
                    }
                }
                path = format!("self/{}", real_path.join("/"));
            }
        }
        if let Some(export_name) = self.object_id_export_name.get(id) {
            let mut pth: Vec<&str> = path.split('/').collect();
            pth.pop();
            pth.push(&export_name);
            path = pth.join(".");
        } else {
            path = path.replace('/', ".");
        }
        Some(path)
    }
}


/// To Stof document trait.
pub trait Stof {
    /// Stof string.
    fn stof(&self, ident: &str, doc: Option<&SDoc>, env: &mut StofExportEnv) -> String;

    /// Optional minified stof.
    fn min_stof(&self, ident: &str, doc: Option<&SDoc>, env: &mut StofExportEnv) -> String {
        self.stof(ident, doc, env)
    }
}


/// Implement Stof for a SDoc
impl Stof for SDoc {
    fn stof(&self, ident: &str, _doc: Option<&SDoc>, env: &mut StofExportEnv) -> String {
        let mut document = String::default();
        for root in self.graph.roots.iter().rev() {
            if let Some(node) = root.node(&self.graph) {
                env.object_id_export_name.insert(node.id.clone(), node.name.clone());
                if node.name == "root" {
                    document.push_str(&node.stof(ident, Some(&self), env));
                } else if node.name != "__stof__" { // we don't export stof meta - types, etc...
                    let ntype = "root".to_string();
                    document.push_str(&format!("{}{} {}: {{\n", ident, ntype, &node.name));
                    document.push_str(&node.stof(&format!("{}\t", ident), Some(&self), env));
                    document.push_str(&format!("{}}}\n", ident));
                }
            }
        }
        document.trim().to_string()
    }
}


/// Implement Stof for an SNode
impl Stof for SNode {
    fn stof(&self, ident: &str, doc: Option<&SDoc>, env: &mut StofExportEnv) -> String {
        if let Some(doc) = doc {
            env.context_stack.push(self.node_ref());

            let mut stof = String::default();
            
            // First, types associated with this Node on the doc
            /* Nope, no exporting of type information, just fields
            for (_, dectypes) in &doc.types.types {
                for dectype in dectypes {
                    if dectype.decid == self.id {
                        // Need to export this custom type, because it was defined on this node
                        stof.push_str(&dectype.stof(ident, Some(doc), env));
                    }
                }
            }
            */

            // Then, all children of this node
            let mut seen_names: HashSet<String> = HashSet::new();
            for child_ref in &self.children {
                if let Some(child) = child_ref.node(&doc.graph) {
                    let mut export_name = child.name.clone();
                    let mut count = 1;
                    while seen_names.contains(&export_name) {
                        export_name = format!("{}{}", child.name, count);
                        count += 1;
                    }
                    seen_names.insert(export_name.clone());
                    env.object_id_export_name.insert(child.id.clone(), export_name.clone());

                    /*let mut ntype = SType::Object.type_of();
                    if let Some(prototype) = SField::field(&doc.graph, "__prototype__", '.', Some(child_ref)) {
                        if let Some(node) = doc.graph.node_ref(&prototype.to_string(), None) {
                            if let Some(typename) = SField::field(&doc.graph, "typename", '.', Some(&node)) {
                                let prototype_path = node.path(&doc.graph).replace('/', ".");
                                ntype = format!("{}.{}", prototype_path, typename.to_string()); // Make sure to include path for correct type reference
                            }
                        }
                    }*/

                    stof.push_str(&format!("{}{}: {{\n", ident, &export_name));
                    stof.push_str(&child.stof(&format!("{}\t", ident), Some(doc), env));
                    stof.push_str(&format!("{}}}\n", ident));
                }
            }

            // Then, all of the data on this node
            for data_ref in &self.data {
                if let Some(data) = data_ref.data(&doc.graph) {
                    // If this data is a field and the field points to a child object, then don't export it!
                    let mut export = true;
                    if data.id.starts_with(FKIND) {
                        if let Ok(field) = data.get_value::<SField>() {
                            match field.value {
                                SVal::Object(nref) => {
                                    if self.has_child(&nref) {
                                        export = false;
                                    }
                                },
                                _ => {}
                            }
                        }
                    }

                    if export {
                        stof.push_str(&data.stof(ident, Some(doc), env));
                    }
                }
            }
            env.context_stack.pop();
            return stof;
        }
        String::default()
    }
}


/// Implement Stof for a CustomType
/*
impl Stof for CustomType {
    fn stof(&self, ident: &str, doc: Option<&SDoc>, env: &mut StofExportEnv) -> String {
        // Get all type attributes!
        let mut attributes = String::default();
        if self.attributes.len() > 0 {
            for (key, val) in &self.attributes {
                attributes.push_str(&format!("#[{}({})] ", key, val.stof("", doc, env)));
            }
            attributes.push('\n');
        }

        let mut custom_type;
        if attributes.len() > 0 {
            custom_type = format!("{}{}{}type {} {{\n", ident, attributes, ident, self.name);
        } else {
            custom_type = format!("{}type {} {{\n", ident, self.name);
        }

        // Add fields to the type
        let subdent = format!("{}\t", ident);
        for param in &self.fields {
            if let Some(def) = &param.default {
                custom_type.push_str(&format!("{}{}: {} = {};\n", subdent, param.name, param.ptype.type_of(), def.stof("", doc, env)));
            } else {
                custom_type.push_str(&format!("{}{}: {};\n", subdent, param.name, param.ptype.type_of()));
            }
        }

        // Add functions to the type
        if let Some(doc) = doc {
            for func in &self.get_functions(&doc.graph) {
                custom_type.push_str(&func.stof(&subdent, Some(doc), env));
            }
        }
        
        custom_type.push_str(&format!("{}}}\n", ident));
        custom_type
    }
}
*/


/// Implement Stof for an SData
impl Stof for SData {
    fn stof(&self, ident: &str, doc: Option<&SDoc>, env: &mut StofExportEnv) -> String {
        if let Some(doc) = doc {
            if self.id.starts_with(FKIND) {
                if let Ok(field) = self.get_value::<SField>() {
                    if field.name != "__prototype__" { // no type info
                        return field.stof(ident, Some(doc), env);
                    }
                }
            } else if self.id.starts_with(FUNC_KIND) {
                /*if let Ok(func) = self.get_value::<SFunc>() {
                    return func.stof(ident, Some(doc), env);
                }*/
            }
        }
        String::default()
    }
}


/// Implement Stof for an SField
/// #[attributes]
/// type name: value
impl Stof for SField {
    fn stof(&self, ident: &str, doc: Option<&SDoc>, env: &mut StofExportEnv) -> String {
        if let Some(export) = self.attributes.get("export") {
            if !export.truthy() { return String::default(); }
        }

        // Get all field attributes!
        let mut attributes = String::default();
        if self.attributes.len() > 0 {
            for (key, val) in &self.attributes {
                attributes.push_str(&format!("#[{}({})] ", key, val.stof("", doc.clone(), env)));
            }
            attributes.push('\n');
        }

        //let typename = self.value.stype().type_of();
        let name = &self.name;
        let value = self.value.stof(ident, doc, env);

        if attributes.len() > 0 {
            format!("{}{}{}{}: {}\n", ident, attributes, ident, name, value)
        } else {
            format!("{}{}: {}\n", ident, name, value)
        }
    }
}


/// Implement Stof for an SFunc
/// #[attributes]
/// fn name(params): rtype { statements }
impl Stof for SFunc {
    fn stof(&self, ident: &str, doc: Option<&SDoc>, env: &mut StofExportEnv) -> String {
        // Get all function attributes!
        let mut attributes = String::default();
        if self.attributes.len() > 0 {
            for (key, val) in &self.attributes {
                attributes.push_str(&format!("#[{}({})] ", key, val.stof("", doc.clone(), env)));
            }
            attributes.push('\n');
        }

        let mut function = format!("{}fn {}(", ident, self.name);
        if attributes.len() > 0 {
            function = format!("{}{}{}fn {}(", ident, attributes, ident, self.name);
        }

        // Add parameters
        let mut first = true;
        for param in &self.params {
            if first {
                first = false;
                if let Some(def) = &param.default {
                    function.push_str(&format!("{}: {} = {}", param.name, param.ptype.type_of(), def.stof("", doc, env)));
                } else {
                    function.push_str(&format!("{}: {}", param.name, param.ptype.type_of()));
                }
            } else {
                if let Some(def) = &param.default {
                    function.push_str(&format!(", {}: {} = {}", param.name, param.ptype.type_of(), def.stof("", doc, env)));
                } else {
                    function.push_str(&format!(", {}: {}", param.name, param.ptype.type_of()));
                }
            }
        }
        function.push(')');

        // Add rtype if needed
        if !self.rtype.is_void() {
            function.push_str(&format!(": {}", self.rtype.type_of()));
        }

        // Statements
        function.push_str(" {\n");
        function.push_str(&self.statements.stof(&format!("{}\t", ident), doc, env));
        
        function.push_str(&format!("{}}}\n", ident));
        function
    }
}
impl Stof for Statements {
    fn stof(&self, ident: &str, doc: Option<&SDoc>, env: &mut StofExportEnv) -> String {
        let mut statements = String::default();
        for statement in &self.statements {
            statements.push_str(&statement.stof(ident, doc, env));
        }
        statements
    }
}
impl Stof for Statement {
    fn stof(&self, ident: &str, doc: Option<&SDoc>, env: &mut StofExportEnv) -> String {
        match self {
            Statement::Assign(name, expr) => {
                format!("{}{} = {};\n", ident, name, expr.stof("", doc, env))
            },
            Statement::Declare(name, expr) => {
                format!("{}let {} = {};\n", ident, name, expr.stof("", doc, env))
            },
            Statement::Drop(name) => {
                format!("{}drop {};\n", ident, name)
            },
            Statement::Move(source, destination) => {
                format!("{}move {} -> {};\n", ident, source, destination)
            },
            Statement::Rename(source, expr) => {
                format!("{}rename {} -> {};\n", ident, source, expr.stof("", doc, env))
            },
            Statement::Expr(expr) => {
                format!("{}{};\n", ident, expr.stof("", doc, env))
            },
            Statement::Return(expr) => {
                format!("{}return {};\n", ident, expr.stof("", doc, env))
            },
            Statement::EmptyReturn => {
                format!("{}return;\n", ident)
            },
            Statement::Break => {
                format!("{}break;\n", ident)
            },
            Statement::Continue => {
                format!("{}continue;\n", ident)
            },
            Statement::If { if_expr, elif_exprs, else_expr } => {
                let subdent = format!("{}\t", ident);
                let if_expr_stof = if_expr.0.stof("", doc, env);
                let if_statements = if_expr.1.stof(&subdent, doc, env);
                let mut res = format!("{}if ({}) {{\n{}\n{}}}", ident, if_expr_stof, if_statements, ident);

                if elif_exprs.len() > 0 {
                    let mut else_if_statements = String::default();
                    for (expr, statements) in elif_exprs {
                        let expr_stof = expr.stof("", doc, env);
                        let statements_stof = statements.stof(&subdent, doc, env);
                        else_if_statements.push_str(&format!(" else if ({}) {{\n{}\n{}}}", expr_stof, statements_stof, ident));
                    }
                    res.push_str(&else_if_statements);
                }

                if let Some(else_statements) = &else_expr {
                    let statements_stof = else_statements.stof(&subdent, doc, env);
                    res.push_str(&format!(" else {{\n{}\n{}}}", statements_stof, ident));
                }

                res.push('\n');
                res
            },
            Statement::While(expr, statements) => {
                let subdent = format!("{}\t", ident);
                let statements_stof = statements.stof(&subdent, doc, env);
                format!("{}while ({}) {{\n{}\n{}}}\n", ident, &expr.stof("", doc, env), statements_stof, ident)
            },
            Statement::Block(statements, finally) => {
                let subdent = format!("{}\t", ident);
                let statements_stof = statements.stof(&subdent, doc, env);
                let finally_stof = finally.stof(&subdent, doc, env);
                format!("{{\n{}\n{}{}}}\n", ident, statements_stof, finally_stof)
            },
        }
    }
}
impl Stof for Expr {
    fn stof(&self, ident: &str, doc: Option<&SDoc>, env: &mut StofExportEnv) -> String {
        match &self {
            Expr::Literal(value) => {
                value.stof(ident, doc, env)
            },
            Expr::Tuple(vals) => {
                let mut res = String::from("(");
                let mut first = true;
                for val in vals {
                    if first {
                        res.push_str(&val.stof("", doc, env));
                        first = false;
                    } else {
                        res.push_str(&format!(", {}", val.stof("", doc, env)));
                    }
                }
                res.push(')');
                res
            },
            Expr::Array(vals) => {
                let mut res = String::from("[");
                let mut first = true;
                for val in vals {
                    if first {
                        res.push_str(&val.stof("", doc, env));
                        first = false;
                    } else {
                        res.push_str(&format!(", {}", val.stof("", doc, env)));
                    }
                }
                res.push(']');
                res
            },
            Expr::Variable(name) => {
                name.clone()
            },
            Expr::Ref(expr) => {
                format!("(ref {})", expr.stof(ident, doc, env))
            },
            Expr::DeRef(expr) => {
                format!("(deref {})", expr.stof(ident, doc, env))
            },
            Expr::Cast(stype, expr) => {
                format!("{} as {}", expr.stof(ident, doc, env), stype.type_of())
            },
            Expr::TypeOf(expr) => {
                format!("(typeof {})", expr.stof(ident, doc, env))
            },
            Expr::TypeName(expr) => {
                format!("(typename {})", expr.stof(ident, doc, env))
            },
            Expr::Not(expr) => {
                format!("!({})", expr.stof(ident, doc, env))
            },
            Expr::Call { scope, name, params } => {
                let mut scope = scope.replace("/", ".");
                if scope.len() > 0 { scope.push_str("."); }

                let mut parameters = String::default();
                let mut first = true;
                for param in params {
                    if first {
                        parameters.push_str(&param.stof("", doc, env));
                        first = false;
                    } else {
                        parameters.push_str(&format!(", {}", &param.stof("", doc, env)));
                    }
                }

                format!("{}{}({})", scope, name, parameters)
            },
            Expr::Block(statements) => {
                let subdent = format!("{}\t", ident);
                format!("{{\n{}\n{}}}", statements.stof(&subdent, doc, env), ident)
            },
            Expr::NewObject(statements) => {
                // NewObjects only have Assign statements in them for initializing the object!
                let subdent = format!("{}\t", ident);
                let mut res = "new {".to_string();
                for statement in &statements.statements {
                    match statement {
                        Statement::Assign(name, expr) => {
                            res.push_str(&format!("\n{}{}: {},", &subdent, &name, &expr.stof("", doc, env)));
                        },
                        Statement::Declare(name, expr) => {
                            res.push_str(&format!("\n{}{}: {},", &subdent, &name, &expr.stof("", doc, env)));
                        },
                        _ => {}
                    }
                }
                res.push_str(&format!("\n{}}}", ident));
                res
            },
            Expr::Add(exprs) => {
                let mut expr_strings = Vec::new();
                for expr in exprs { expr_strings.push(expr.stof(ident, doc, env)); }
                format!("({})", expr_strings.join(" + "))
            },
            Expr::Sub(exprs) => {
                let mut expr_strings = Vec::new();
                for expr in exprs { expr_strings.push(expr.stof(ident, doc, env)); }
                format!("({})", expr_strings.join(" - "))
            },
            Expr::Mul(exprs) => {
                let mut expr_strings = Vec::new();
                for expr in exprs { expr_strings.push(expr.stof(ident, doc, env)); }
                format!("({})", expr_strings.join(" * "))
            },
            Expr::Div(exprs) => {
                let mut expr_strings = Vec::new();
                for expr in exprs { expr_strings.push(expr.stof(ident, doc, env)); }
                format!("({})", expr_strings.join(" / "))
            },
            Expr::Rem(exprs) => {
                let mut expr_strings = Vec::new();
                for expr in exprs { expr_strings.push(expr.stof(ident, doc, env)); }
                format!("({})", expr_strings.join(" % "))
            },
            Expr::And(exprs) => {
                let mut expr_strings = Vec::new();
                for expr in exprs { expr_strings.push(expr.stof(ident, doc, env)); }
                format!("({})", expr_strings.join(" && "))
            },
            Expr::Or(exprs) => {
                let mut expr_strings = Vec::new();
                for expr in exprs { expr_strings.push(expr.stof(ident, doc, env)); }
                expr_strings.join(" || ")
            },
            Expr::Eq(lhs, rhs) => {
                format!("({} == {})", &lhs.stof(ident, doc, env), &rhs.stof(ident, doc, env))
            },
            Expr::Neq(lhs, rhs) => {
                format!("({} != {})", &lhs.stof(ident, doc, env), &rhs.stof(ident, doc, env))
            },
            Expr::Gte(lhs, rhs) => {
                format!("({} >= {})", &lhs.stof(ident, doc, env), &rhs.stof(ident, doc, env))
            },
            Expr::Lte(lhs, rhs) => {
                format!("({} <= {})", &lhs.stof(ident, doc, env), &rhs.stof(ident, doc, env))
            },
            Expr::Gt(lhs, rhs) => {
                format!("({} > {})", &lhs.stof(ident, doc, env), &rhs.stof(ident, doc, env))
            },
            Expr::Lt(lhs, rhs) => {
                format!("({} < {})", &lhs.stof(ident, doc, env), &rhs.stof(ident, doc, env))
            },
        }
    }
}


/// Implement Stof for SVal
impl Stof for SVal {
    fn stof(&self, ident: &str, doc: Option<&SDoc>, env: &mut StofExportEnv) -> String {
        if let Some(doc) = doc {
            match self {
                Self::String(val) => { format!("\"{}\"", val) },
                Self::Bool(val) => { val.to_string() },
                Self::Number(val) => { val.to_string() },
                Self::Array(vals) => {
                    let mut res = String::from("[");
                    let mut first = true;
                    for val in vals {
                        if first {
                            first = false;
                            res.push_str(&val.stof(ident, Some(doc), env));
                        } else {
                            res.push_str(&format!(", {}", val.stof(ident, Some(doc), env)));
                        }
                    }
                    res.push(']');
                    res
                },
                Self::Object(nref) => {
                    if let Some(val) = env.relative_obj_path(&nref.id, &doc.graph) {
                        return val;
                    }
                    nref.path(&doc.graph).replace('/', ".")
                },
                Self::FnPtr(dref) => {
                    if let Some(data) = dref.data(&doc.graph) {
                        for node in &data.nodes {
                            if let Ok(func) = data.get_value::<SFunc>() {
                                if let Some(val) = env.relative_obj_path(&node.id, &doc.graph) {
                                    return format!("{}.{}", val, func.name);
                                }
                            }
                        }
                    }
                    String::default()
                },
                Self::Null => { "null".to_string() },
                Self::Void => { "void".to_string() },
                Self::Ref(rf) => {
                    let val = rf.read().unwrap();
                    val.stof(ident, Some(doc), env)
                },
                Self::Tuple(tup) => {
                    let mut res = String::from("(");
                    let mut first = true;
                    for val in tup {
                        if first {
                            first = false;
                            res.push_str(&val.stof(ident, Some(doc), env));
                        } else {
                            res.push_str(&format!(", {}", val.stof(ident, Some(doc), env)));
                        }
                    }
                    res.push(')');
                    res
                },
                Self::Blob(blob) => {
                    format!("({:?} as blob)", blob)
                },
            }
        } else {
            String::default()
        }
    }
}
