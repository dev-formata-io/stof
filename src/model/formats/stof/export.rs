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

use rustc_hash::FxHashSet;
use crate::{model::{Field, Func, Graph, NodeRef, NOEXPORT_FIELD_ATTR}, runtime::Val};


#[derive(Debug, Clone, Default)]
pub struct StofExportContext {
    pub stof: String,
    pub indent: String,
    pub human: bool,
}
impl StofExportContext {
    /// Export a node into this context.
    pub fn export_node(&mut self, graph: &Graph, node: &NodeRef, seen: &mut FxHashSet<NodeRef>) {
        if seen.contains(node) { return; }
        seen.insert(node.clone());

        let mut root = false;
        if node.is_root(graph) {
            if let Some(main) = graph.main_root() {
                root = &main != node;
            } else {
                root = true;
            }
        }
        let mut started = false;
        if root {
            // root ObjectName { (id) ..datas.. }
            if let Some(name) = node.node_name(graph) {
                let obj_name = name.as_ref().replace("'", "\\'");
                self.push_line(&format!("root '{}' {{ ({})\n", obj_name, node.as_ref()), false);
                self.push_indent();
                started = true;
            }
        }

        if let Some(node) = node.node(graph) {
            for (_name, dref) in &node.data {
                if let Some(field) = graph.get_stof_data::<Field>(dref) {
                    if !field.attributes.contains_key(NOEXPORT_FIELD_ATTR.as_str()) {
                        if let Some(field_node) = field.value.try_obj() {
                            if !field_node.child_of(graph, &node.id) && !seen.contains(&field_node) {
                                if let Some(child) = field_node.node(graph) {
                                    if self.human { self.push_line("", false); }
                                    let mut saw_no_field = false;
                                    for attr in &child.attributes {
                                        if attr.0 == "no-field" { saw_no_field = true; }
                                        if attr.1.empty() {
                                            self.push_line(&format!("#[{}]", attr.0), false);
                                        } else if attr.0 != "extends" { // prototypes are already included... this may error otherwise with Str typenames
                                            self.push_line(&format!("#[{}(", attr.0), false);
                                            self.export_value(graph, attr.1.clone());
                                            self.push_text(")]", false);
                                        }
                                    }
                                    if !saw_no_field {
                                        self.push_line("#[no-field]", false);
                                    }
                                    let obj_name = child.name.as_ref().replace("'", "\\'");
                                    self.push_line(&format!("'{}': {{ ({})", obj_name, child.id.as_ref()), false);
                                    self.push_indent();
                                    self.export_node(graph, &child.id, seen);
                                    self.pop_indent();
                                    self.push_line("}", false);
                                }
                            }
                        }
                        if let Some(data) = dref.data(graph) {
                            if let Ok(bytes) = bincode::serialize(data) {
                                if self.human {
                                    self.push_line("", false);
                                    for attr in &field.attributes {
                                        if attr.1.empty() {
                                            self.push_line(&format!("// #[{}]", attr.0), false);
                                        } else {
                                            self.push_line(&format!("// #[{}({})]", attr.0, attr.1.print(graph)), false);
                                        }
                                    }
                                    self.push_line(&format!("// {}: {};", data.name.as_ref(), field.value.val.read().print(graph)), false);
                                }

                                let str = format!("{bytes:?}");
                                self.push_line(&format!("data@v1 |{}|", str.trim_start_matches('[').trim_end_matches(']')), true);
                            }
                        }
                    }
                } else if let Some(func) = graph.get_stof_data::<Func>(dref) {
                    if !func.attributes.contains_key(NOEXPORT_FIELD_ATTR.as_str()) {
                        if let Some(data) = dref.data(graph) {
                            if let Ok(bytes) = bincode::serialize(data) {
                                if self.human {
                                    self.push_line("", false);
                                    for attr in &func.attributes {
                                        if attr.1.empty() {
                                            self.push_line(&format!("// #[{}]", attr.0), false);
                                        } else {
                                            self.push_line(&format!("// #[{}({})]", attr.0, attr.1.print(graph)), false);
                                        }
                                    }
                                    self.push_line(&format!("// fn {}({:?}) -> {};", data.name.as_ref(), func.params, func.return_type.rt_type_of(graph)), false);
                                }

                                let str = format!("{bytes:?}");
                                self.push_line(&format!("data@v1 |{}|", str.trim_start_matches('[').trim_end_matches(']')), true);
                            }
                        }
                    }
                } else {
                    // prototype or custom data implementation
                    if let Some(data) = dref.data(graph) {
                        if let Ok(bytes) = bincode::serialize(data) {
                            if self.human {
                                self.push_line("", false);
                                self.push_line(&format!("// data named '{}', libname '{}'", data.name.as_ref(), data.tagname()), false);
                            }

                            let str = format!("{bytes:?}");
                            self.push_line(&format!("data@v1 |{}|", str.trim_start_matches('[').trim_end_matches(']')), true);
                        }
                    }
                }
            }
            for child in &node.children {
                if let Some(child) = child.node(graph) {
                    if !seen.contains(&child.id) {
                        if self.human { self.push_line("", false); }
                        let mut saw_no_field = false;
                        for attr in &child.attributes {
                            if attr.0 == "no-field" { saw_no_field = true; }
                            if attr.1.empty() {
                                self.push_line(&format!("#[{}]", attr.0), false);
                            } else if attr.0 != "extends" { // prototypes are already included... this may error otherwise with Str typenames
                                self.push_line(&format!("#[{}(", attr.0), false);
                                self.export_value(graph, attr.1.clone());
                                self.push_text(")]", false);
                            }
                        }
                        if !child.is_field() && !saw_no_field {
                            self.push_line("#[no-field]", false);
                        }
                        let obj_name = child.name.as_ref().replace("'", "\\'");
                        self.push_line(&format!("'{}': {{ ({})", obj_name, child.id.as_ref()), false);
                        self.push_indent();
                        self.export_node(graph, &child.id, seen);
                        self.pop_indent();
                        self.push_line("}", false);
                    }
                }
            }
        }

        if started {
            self.pop_indent();
            self.push_line("}", false);
        }
    }

    /// Export a singular value.
    fn export_value(&mut self, graph: &Graph, val: Val) {
        match val {
            Val::Void |
            Val::Promise(..) |
            Val::Null => self.stof.push_str("null"),
            Val::Bool(_) => self.stof.push_str(&val.to_string()),
            Val::Str(str) => self.stof.push_str(&format!("r#\"{str}\"#")),
            Val::Prompt(v) => self.stof.push_str(&format!("r#\"{}\"#", v.to_string())),
            Val::Num(v) => self.stof.push_str(&v.to_string()),
            Val::Blob(blob) => {
                let str = format!("{blob:?}");
                self.stof.push_str(&format!("|{}|", str.trim_start_matches('[').trim_end_matches(']')));
            },
            Val::Fn(dref) => {
                self.stof.push_str(&format!("Data.from_id('{}') as fn", dref.as_ref()));
            },
            Val::Data(dref) => {
                self.stof.push_str(&format!("Data.from_id('{}')", dref.as_ref()));
            },
            Val::Obj(nref) => {
                self.stof.push_str(&format!("Obj.from_id('{}')", nref.as_ref()));
            },
            Val::List(vals) => {
                let mut context = Self::default();
                context.push_text("[", false);
                let mut first = true;
                for val in vals {
                    if !first { context.push_text(", ", false); }
                    else { first = false; }
                    context.export_value(graph, val.read().clone());
                }
                context.push_text("]", false);
                self.stof.push_str(&context.stof);
            },
            Val::Tup(vals) => {
                let mut context = Self::default();
                context.push_text("(", false);
                let mut first = true;
                for val in vals {
                    if !first { context.push_text(", ", false); }
                    else { first = false; }
                    context.export_value(graph, val.read().clone());
                }
                context.push_text(")", false);
                self.stof.push_str(&context.stof);
            },
            Val::Ver(..) => self.stof.push_str(&val.to_string()),
            Val::Set(set) => {
                let mut context = Self::default();
                context.push_text("{", false);
                let mut first = true;
                for val in set {
                    if !first { context.push_text(", ", false); }
                    else { first = false; }
                    context.export_value(graph, val.read().clone());
                }
                context.push_text("}", false);
                self.stof.push_str(&context.stof);
            },
            Val::Map(map) => {
                let mut context = Self::default();
                context.push_text("{", false);
                let mut first = true;
                for (k, v) in map {
                    if !first { context.push_text(", ", false); }
                    else { first = false; }
                    context.export_value(graph, k.read().clone());
                    context.push_text(": ", false);
                    context.export_value(graph, v.read().clone());
                }
                context.push_text("}", false);
                self.stof.push_str(&context.stof);
            },
        }
    }

    #[inline]
    /// Push a line.
    fn push_line(&mut self, line: &str, semicolon: bool) {
        if semicolon {
            self.stof.push_str(&format!("\n{}{line};", self.indent));
        } else {
            self.stof.push_str(&format!("\n{}{line}", self.indent));    
        }
    }

    #[inline]
    /// Push text.
    fn push_text(&mut self, text: &str, indent: bool) {
        if indent {
            self.stof.push_str(&format!("{}{text}", self.indent));    
        } else {
            self.stof.push_str(text);
        }
    }

    #[inline(always)]
    /// Push indent.
    fn push_indent(&mut self) {
        self.indent.push_str("\t");
    }

    #[inline(always)]
    /// Pop indent.
    fn pop_indent(&mut self) {
        if let Some(indent) = self.indent.strip_suffix('\t') { self.indent = indent.to_owned(); }
    }
}
