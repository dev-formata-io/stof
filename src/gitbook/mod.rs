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
use crate::{lang::{SCustomTypeFieldDoc, SError, SInnerDoc}, CustomTypes, Format, IntoNodeRef, SData, SDoc, SExternDoc, SExternFuncDoc, SField, SFieldDoc, SFunc, SFuncDoc, SNodeRef};


/// Gitbook export format.
pub struct Gitbook;
impl Gitbook {
    /// Parse a document into docs.
    pub fn export_doc_markdown(doc: &SDoc) -> String {
        let mut docs = String::default();
        for root in &doc.graph.roots {
            if root.node(&doc.graph).unwrap().name == "__stof__" { continue; }
            docs.push_str(&Self::export_node_markdown(doc, root));
        }
        docs
    }

    /// Parse a node into docs.
    pub fn export_node_markdown(doc: &SDoc, node: &SNodeRef) -> String {
        let mut docs = String::default();

        // Node inner docs first (SInnerDoc)
        docs.push_str(&Self::inner_docs(doc, node, false));

        // Node fields & field structure next (Stof fields overview from this nodes perspective (all sub nodes))
        let field_docs = Self::field_docs(doc, node, "");
        if field_docs.len() > 0 {
            docs.push_str(&format!("\n# Fields\n```javascript\n{field_docs}\n```\n"));
        }

        // Node functions & func overviews next (Stof func overview from this nodes perspective (all sub nodes))
        let func_docs = Self::func_docs(doc, node, false);
        if func_docs.len() > 0 {
            docs.push_str(&format!("\n# Functions\n{func_docs}\n"));
        }

        // Node types next (type fields, type inner docs, and type functions (all sub nodes))
        let custom_types = doc.types.declared_types_for(node, &doc.graph);
        if custom_types.types.len() > 0 {
            let ctypes = Self::custom_type_docs(doc, node, custom_types);
            docs.push_str(&format!("\n# Types\n{ctypes}"));
        }

        // Node extern blocks next (SExternDoc & SExternFuncDoc & SInnerDoc)
        let external = Self::external_docs(doc, node);
        if external.len() > 0 {
            docs.push_str(&format!("\n{external}"));
        }

        docs
    }

    /// Parse external libs.
    fn external_docs(doc: &SDoc, node: &SNodeRef) -> String {
        let mut docs = String::default();

        let extern_funcs = SExternFuncDoc::extern_func_docs(&doc.graph, node, true);
        for extern_block in SExternDoc::extern_docs(&doc.graph, node, true) {
            let mut extern_docs;
            if extern_block.libname.len() > 0 {
                extern_docs = format!("# {}\n", extern_block.libname);
            } else {
                extern_docs = format!("# Extern Lib\n");
            }

            if let Some(comments) = &extern_block.docs {
                extern_docs.push_str(&comments);
                extern_docs.push('\n');
            }

            if let Some((link, expr)) = &extern_block.link_expr {
                extern_docs.push_str(&format!("## Link Attribute\n"));
                extern_docs.push_str(&format!("Name: {link}\n"));
                if let Some(expr) = expr {
                    extern_docs.push_str(&format!("Expr: {expr:?}\n"));
                }
                extern_docs.push('\n');
            }
            
            let mut funcs = Vec::new();
            for func in &extern_funcs {
                if func.extern_id == extern_block.extern_id {
                    funcs.push(func);
                }
            }
            funcs.sort_by(|a, b| a.name.cmp(&b.name));

            if funcs.len() > 0 {
                for func in funcs {
                    let mut params = String::default();
                    let mut first = true;
                    for param in &func.params {
                        if first {
                            first = false;
                            params.push_str(&format!("{}:{}", param.name, param.ptype.type_of()));
                            if param.default.is_some() {
                                params.push_str(" = (default expr)");
                            }
                        } else {
                            params.push_str(&format!(", {}:{}", param.name, param.ptype.type_of()));
                            if param.default.is_some() {
                                params.push_str(" = (default expr)");
                            }
                        }
                    }

                    extern_docs.push_str(&format!("## {}({params}) -> {}\n", func.name, func.rtype.type_of()));

                    if func.attributes.len() > 0 {
                        extern_docs.push_str("\n**Attributes**\n```\n");
                        let mut attributes = String::default();
                        for attr in &func.attributes {
                            let stype = attr.1.stype(&doc.graph);
                            if stype.is_empty() {
                                attributes.push_str(&format!("{}: {}\n", attr.0, attr.1.print(doc)));
                            } else {
                                attributes.push_str(&format!("{} {}: {}\n", stype.type_of(), attr.0, attr.1.print(doc)));
                            }
                        }
                        extern_docs.push_str(&format!("{attributes}```\n\n"));
                    }

                    if let Some(docs) = &func.docs {
                        extern_docs.push_str(docs);
                    }
                }
            }

            docs.push_str(&extern_docs);
        }

        docs
    }

    /// Parse custom types.
    fn custom_type_docs(doc: &SDoc, node: &SNodeRef, custom_types: CustomTypes) -> String {
        let mut docs = String::default();
        let node_path = node.path(&doc.graph);

        let mut named_type_docs: Vec<(String, String)> = Vec::new();
        for (name, types) in custom_types.types {
            let mut named_docs = String::default();
            for ctype in types {
                let mut type_docs = String::default();

                let mut declared_path = SNodeRef::from(ctype.decid).path(&doc.graph).replace('/', ".");
                if let Some(fp) = declared_path.strip_prefix(&node_path) { declared_path = fp.to_owned(); }
                declared_path = declared_path.trim_start_matches('.').to_owned();

                if declared_path.len() > 0 {
                    type_docs.push_str(&format!("## {declared_path}.{}\n", ctype.name));
                } else {
                    type_docs.push_str(&format!("## {}\n", ctype.name));
                }

                let loc_ref = SNodeRef::from(ctype.locid);
                for inner in SInnerDoc::inner_docs(&doc.graph, &loc_ref, false) {
                    type_docs.push_str(&inner.docs);
                }

                if ctype.attributes.len() > 0 {
                    type_docs.push_str("\n### Attributes\n```\n");
                    let mut attributes = String::default();
                    for attr in &ctype.attributes {
                        let stype = attr.1.stype(&doc.graph);
                        if stype.is_empty() {
                            attributes.push_str(&format!("{}: {}\n", attr.0, attr.1.print(doc)));
                        } else {
                            attributes.push_str(&format!("{} {}: {}\n", stype.type_of(), attr.0, attr.1.print(doc)));
                        }
                    }
                    type_docs.push_str(&format!("{attributes}```\n"));
                }

                if ctype.fields.len() > 0 {
                    type_docs.push_str("\n```javascript\n");
                    let mut field_docs = FxHashMap::default();
                    for field_doc in SCustomTypeFieldDoc::ct_field_docs(&doc.graph, &loc_ref) {
                        field_docs.insert(&field_doc.field, &field_doc.docs);
                    }
                    for field in &ctype.fields {
                        let doc_comment = field_docs.get(&field.name);
                        if let Some(doc) = doc_comment {
                            for line in doc.trim_end_matches('\n').split('\n') {
                                type_docs.push_str(&format!("// {line}\n"));
                            }
                        }

                        if field.attributes.len() > 0 {
                            for attr in &field.attributes {
                                let stype = attr.1.stype(&doc.graph);
                                if stype.is_empty() {
                                    type_docs.push_str(&format!("[{}({})]\n", attr.0, attr.1.print(doc)));
                                } else {
                                    type_docs.push_str(&format!("[{}({}:{})]\n", attr.0, attr.1.print(doc), stype.type_of()));
                                }
                            }
                        }

                        if field.optional {
                            if field.default.is_some() {
                                type_docs.push_str(&format!("{}?: {} = (default expr)\n", field.name, field.ptype.type_of()));
                            } else {
                                type_docs.push_str(&format!("{}?: {}\n", field.name, field.ptype.type_of()));
                            }
                        } else {
                            if field.default.is_some() {
                                type_docs.push_str(&format!("{}: {} = (default expr)\n", field.name, field.ptype.type_of()));
                            } else {
                                type_docs.push_str(&format!("{}: {}\n", field.name, field.ptype.type_of()));
                            }
                        }
                        type_docs.push('\n');
                    }
                    type_docs.push_str(&format!("```\n"));
                }
                
                let func_doc = Self::func_docs(doc, &loc_ref, true);
                if func_doc.len() > 0 {
                    type_docs.push_str(&func_doc);
                    type_docs.push('\n');
                }
                
                named_docs.push_str(&type_docs);
            }
            named_type_docs.push((name, named_docs));
        }
        named_type_docs.sort_by(|a, b| a.0.cmp(&b.0));
        for (_, d) in named_type_docs { docs.push_str(&d); }

        docs
    }

    /// Parse inner docs.
    fn inner_docs(doc: &SDoc, node: &SNodeRef, recursive: bool) -> String {
        let mut docs = String::default();
        for inner in SInnerDoc::inner_docs(&doc.graph, node, recursive) {
            docs.push_str(&inner.docs);
        }
        docs
    }

    /// Parse node functions.
    fn func_docs(doc: &SDoc, node: &SNodeRef, types: bool) -> String {
        let mut docs = String::default();

        let mut start_node = node.clone();
        if types {
            while let Some(node) = start_node.node(&doc.graph) {
                if let Some(parent) = &node.parent {
                    if let Some(parent) = parent.node(&doc.graph) {
                        if parent.name != "prototypes" && parent.name != "__stof__" {
                            start_node = parent.node_ref();
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
        
        let mut func_docs = FxHashMap::default();
        for func_doc in SFuncDoc::func_docs(&doc.graph, &start_node, true) {
            func_docs.insert(&func_doc.func, &func_doc.docs);
        }

        let node_path = node.path(&doc.graph);
        let mut func_blocks = Vec::new(); // (name, docs)
        for func_ref in SFunc::recursive_func_refs(&doc.graph, &start_node) {
            let mut func_doc = String::default();

            let mut func_path = func_ref.first_path(&doc.graph);
            if types && func_path.len() > node_path.len() { continue; }

            if let Some(fp) = func_path.strip_prefix(&node_path) {
                func_path = fp.to_owned();
            }
            func_path = func_path.replace('/', ".").trim_start_matches('.').to_owned();

            if let Some(func) = SData::get::<SFunc>(&doc.graph, &func_ref) {
                let mut params = String::default();
                let mut first = true;
                for param in &func.params {
                    if first {
                        first = false;
                        params.push_str(&format!("{}:{}", param.name, param.ptype.type_of()));
                        if param.default.is_some() {
                            params.push_str(" = (default expr)");
                        }
                    } else {
                        params.push_str(&format!(", {}:{}", param.name, param.ptype.type_of()));
                        if param.default.is_some() {
                            params.push_str(" = (default expr)");
                        }
                    }
                }

                if types {
                    if func_path.len() > 0 {
                        if func_path.starts_with("__stof__") {
                            func_doc.push_str(&format!("### super.{}({params}) -> {}\n", &func.name, &func.rtype.type_of()));
                        } else {
                            func_doc.push_str(&format!("### {func_path}.{}({params}) -> {}\n", &func.name, &func.rtype.type_of()));
                        }
                    } else {
                        func_doc.push_str(&format!("### {}({params}) -> {}\n", &func.name, &func.rtype.type_of()));
                    }
                } else {
                    if func_path.len() > 0 {
                        func_doc.push_str(&format!("## {func_path}.{}({params}) -> {}\n", &func.name, &func.rtype.type_of()));
                    } else {
                        func_doc.push_str(&format!("## {}({params}) -> {}\n", &func.name, &func.rtype.type_of()));
                    }
                }

                if func.attributes.len() > 0 {
                    if types {
                        func_doc.push_str("**Attributes**\n```\n");
                    } else {
                        func_doc.push_str("### Attributes\n```\n");
                    }
                    let mut attributes = String::default();
                    for attr in &func.attributes {
                        let stype = attr.1.stype(&doc.graph);
                        if stype.is_empty() {
                            attributes.push_str(&format!("{}: {}\n", attr.0, attr.1.print(doc)));
                        } else {
                            attributes.push_str(&format!("{} {}: {}\n", stype.type_of(), attr.0, attr.1.print(doc)));
                        }
                    }
                    func_doc.push_str(&format!("{attributes}```\n"));
                }

                func_path.push_str(&func.name);
            }

            let doc_comment = func_docs.get(&func_ref);
            if let Some(doc) = doc_comment {
                func_doc.push_str(doc);
            }

            func_blocks.push((func_path, func_doc));
        }
        func_blocks.sort_by(|a, b| {
            a.0.cmp(&b.0)
        });
        for (_, d) in func_blocks {
            docs.push_str(&d);
        }

        docs.trim_end_matches('\n').to_owned()
    }

    /// Parse node fields.
    fn field_docs(doc: &SDoc, node: &SNodeRef, indent: &str) -> String {
        let mut docs = String::default();
        
        let mut field_docs = FxHashMap::default();
        for field_doc in SFieldDoc::field_docs(&doc.graph, node) {
            field_docs.insert(&field_doc.field, &field_doc.docs);
        }

        for field_ref in SField::field_refs(&doc.graph, node) {
            let doc_comment = field_docs.get(&field_ref);
            if let Some(doc) = doc_comment {
                for line in doc.trim_end_matches('\n').split('\n') {
                    docs.push_str(&format!("{indent}// {line}\n"));
                }
            }
            if let Some(field) = SData::get::<SField>(&doc.graph, &field_ref) {
                let field_type = field.value.stype(&doc.graph).type_of();

                if let Some(nref) = field.value.try_object() {
                    docs.push_str(&format!("{indent}{} {}: {{\n", field_type, &field.name));
                    docs.push_str(&Self::field_docs(doc, &nref, &format!("{indent}\t")));
                    docs.push_str(&format!("\n{indent}}}\n"));
                } else {
                    let field_value = field.value.print(doc);
                    docs.push_str(&format!("{indent}{} {}: {}\n", field_type, &field.name, field_value));
                }
            }
        }
        docs.trim_end_matches('\n').to_owned()
    }
}
impl Format for Gitbook {
    fn format(&self) -> String {
        "gitbook".into()
    }

    fn content_type(&self) -> String {
        "application/stof+gitbook".into()
    }

    fn export_string(&self, _pid: &str, doc: &SDoc, node: Option<&SNodeRef>) -> Result<String, SError> {
        if let Some(node) = node {
            Ok(Self::export_node_markdown(doc, node))
        } else {
            Ok(Self::export_doc_markdown(doc))
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::SDoc;

    #[test]
    fn gitbook_docs() {
        let gitbook = SDoc::gitbook_md_from_file("src/gitbook/docs.stof").unwrap();
        //println!("{gitbook}");
        assert!(gitbook.len() > 0);
    }
}
