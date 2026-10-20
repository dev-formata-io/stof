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

//! Element formats: html and xml.
//!
//! An object is an element. Its fields, in order, become:
//! - `#[attr]` (or `#[attr('name')]`): an attribute. A function marked `#[attr]`, or a field holding a function, is
//!   called for its value (a computed value), and so are `#[attr]` functions from the object's prototypes.
//! - `#[text]`: text inside the element (escaped). A computed value too, when it's a function. An object (or a list
//!   of objects) is written as child elements, so a function can return part of the tree.
//! - an object: a child element. A list: one element per item.
//! - any other value: a child element holding the value as text (`<name>value</name>`).
//! - `#[no-export]`: skipped. Functions without `#[attr]` or `#[text]` aren't written.
//!
//! The element's name is `#[tag('name')]` on its field, on the object, or on one of its prototypes (Ex. a
//! `#[type]`), and otherwise the field's name. A null value is not written.
//!
//! HTML is written to be safe to put in a page, whatever the document holds:
//! - text and attribute values are always escaped
//! - elements that run or load code (script, style, iframe, object, embed, ...) are never written, with their content
//! - `on*` attributes, `srcdoc`, and `javascript:`/`vbscript:`/`data:` URLs are dropped (data: images are kept)
//! - only allowed elements are written: the standard HTML elements and custom elements (with a `-` in the name) by
//!   default, or exactly the host's list (`Graph::allow_html_tags`), which Stof code can't change
//! - an html export of an object without a tag is a fragment: its content without an element around it
//!
//! XML is data: every name must be a valid XML name (an error otherwise), empty elements close themselves, booleans
//! are `true`/`false`, and the document starts with an XML declaration.

use std::sync::Arc;
use arcstr::ArcStr;
use rustc_hash::FxHashSet;
use crate::{model::{Computed, ComputedCall, DataRef, Field, Format, Func, Graph, NodeRef, NOEXPORT_FIELD_ATTR, Prototype, json::export::json_string}, runtime::{Error, Val}};

#[cfg(feature = "xml")]
mod import;



/// The element name attribute: `#[tag('lx-panel')]`.
pub const TAG_ATTR: &str = "tag";
/// An attribute of the element: `#[attr]` or `#[attr('data-id')]`.
pub const ATTR_ATTR: &str = "attr";
/// Text inside the element: `#[text]`.
pub const TEXT_ATTR: &str = "text";

/// Elements html never writes (they run or load code, or change the page around them), with their content.
const HTML_NEVER: &[&str] = &["script", "style", "iframe", "frame", "frameset", "object", "embed", "applet", "base", "link", "meta", "noscript", "template"];

/// Standard HTML elements allowed by default (custom elements, with a `-` in the name, are allowed too).
const HTML_STANDARD: &[&str] = &[
    "a", "abbr", "address", "area", "article", "aside", "audio", "b", "bdi", "bdo", "blockquote", "body", "br",
    "button", "canvas", "caption", "cite", "code", "col", "colgroup", "data", "datalist", "dd", "del", "details",
    "dfn", "dialog", "div", "dl", "dt", "em", "fieldset", "figcaption", "figure", "footer", "form", "h1", "h2",
    "h3", "h4", "h5", "h6", "head", "header", "hgroup", "hr", "html", "i", "img", "input", "ins", "kbd", "label",
    "legend", "li", "main", "map", "mark", "menu", "meter", "nav", "ol", "optgroup", "option", "output", "p",
    "picture", "pre", "progress", "q", "rp", "rt", "ruby", "s", "samp", "search", "section", "select", "slot",
    "small", "source", "span", "strong", "sub", "summary", "sup", "table", "tbody", "td", "textarea", "tfoot",
    "th", "thead", "time", "title", "tr", "track", "u", "ul", "var", "video", "wbr",
    // svg drawing (scripts and event attributes are dropped like everywhere else)
    "svg", "g", "path", "circle", "ellipse", "line", "polyline", "polygon", "rect", "text", "tspan", "defs",
];

/// HTML elements with no content or end tag.
const HTML_VOID: &[&str] = &["area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source", "track", "wbr"];


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Markup {
    Html,
    Xml,
}


#[derive(Debug)]
/// HTML (export only).
pub struct HtmlFormat;
impl Format for HtmlFormat {
    fn identifiers(&self) -> Vec<String> {
        vec!["html".into()]
    }
    fn content_type(&self) -> String {
        "text/html".into()
    }
    fn string_export(&self, graph: &Graph, _format: &str, node: Option<NodeRef>) -> Result<String, Error> {
        export(graph, node, Markup::Html, &Computed::default())
    }
    fn computed(&self, graph: &Graph, _format: &str, node: Option<NodeRef>, known: &Computed) -> Vec<ComputedCall> {
        computed(graph, node, Markup::Html, known)
    }
    fn string_export_computed(&self, graph: &Graph, _format: &str, node: Option<NodeRef>, computed: &Computed) -> Result<String, Error> {
        export(graph, node, Markup::Html, computed)
    }
}


#[derive(Debug)]
/// XML (import with the "xml" feature).
pub struct XmlFormat;
impl Format for XmlFormat {
    fn identifiers(&self) -> Vec<String> {
        vec!["xml".into()]
    }
    fn content_type(&self) -> String {
        "application/xml".into()
    }
    fn string_export(&self, graph: &Graph, _format: &str, node: Option<NodeRef>) -> Result<String, Error> {
        export(graph, node, Markup::Xml, &Computed::default())
    }
    fn computed(&self, graph: &Graph, _format: &str, node: Option<NodeRef>, known: &Computed) -> Vec<ComputedCall> {
        computed(graph, node, Markup::Xml, known)
    }
    fn string_export_computed(&self, graph: &Graph, _format: &str, node: Option<NodeRef>, computed: &Computed) -> Result<String, Error> {
        export(graph, node, Markup::Xml, computed)
    }

    #[cfg(feature = "xml")]
    fn string_import(&self, graph: &mut Graph, _format: &str, src: &str, node: Option<NodeRef>, _profile: &crate::model::Profile) -> Result<(), Error> {
        import::xml_import(graph, src, node)
    }
}


/// Export a node as html or xml, with the computed values known so far.
fn export(graph: &Graph, node: Option<NodeRef>, markup: Markup, known: &Computed) -> Result<String, Error> {
    let Some(root) = node.or_else(|| graph.main_root()) else { return Ok(String::new()); };
    let mut writer = Writer::new(graph, markup, known, false);
    writer.root(&root)?;
    Ok(writer.out)
}

/// The computed values an export still needs (functions not called yet).
fn computed(graph: &Graph, node: Option<NodeRef>, markup: Markup, known: &Computed) -> Vec<ComputedCall> {
    let Some(root) = node.or_else(|| graph.main_root()) else { return vec![]; };
    let mut writer = Writer::new(graph, markup, known, true);
    let _ = writer.root(&root); // collecting: errors are reported by the real export
    writer.missing
}


/// One field (or function) of an object, as the element writer sees it.
struct Entry<'a> {
    name: &'a str,
    attrs: &'a rustc_hash::FxHashMap<String, Val>,
    value: Value,
}

enum Value {
    /// A field's value.
    Val(Val),
    /// A function marked #[attr] or #[text] (called as a method of the object).
    Method(DataRef),
}


struct Writer<'a> {
    graph: &'a Graph,
    markup: Markup,
    known: &'a Computed,

    /// Collecting the computed values that are missing instead of writing (nothing is written).
    collect: bool,
    missing: Vec<ComputedCall>,
    missing_keys: FxHashSet<(NodeRef, ArcStr)>,

    /// Objects being written (an object that holds one of its ancestors isn't written again).
    path: FxHashSet<NodeRef>,
    out: String,
}

impl<'a> Writer<'a> {
    fn new(graph: &'a Graph, markup: Markup, known: &'a Computed, collect: bool) -> Self {
        Self { graph, markup, known, collect, missing: vec![], missing_keys: Default::default(), path: Default::default(), out: String::new() }
    }

    fn root(&mut self, root: &NodeRef) -> Result<(), Error> {
        if self.markup == Markup::Xml {
            self.out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
            let name = self.tag_of(None, root).unwrap_or_else(|| root.node(self.graph).map(|n| n.name.as_ref().to_string()).unwrap_or_default());
            return self.element(&name, root);
        }
        match self.tag_of(None, root) {
            Some(tag) => self.element(&tag, root),
            None => {
                // a fragment: the object's content, without an element around it
                self.path.insert(root.clone());
                let res = self.content(root);
                self.path.remove(root);
                res
            },
        }
    }

    /// An element's name: #[tag] on its field, on the object, or on one of its prototypes.
    fn tag_of(&self, field_attrs: Option<&rustc_hash::FxHashMap<String, Val>>, node: &NodeRef) -> Option<String> {
        if let Some(Val::Str(tag)) = field_attrs.and_then(|attrs| attrs.get(TAG_ATTR)) {
            return Some(tag.to_string());
        }
        if let Some(Val::Str(tag)) = node.node(self.graph).and_then(|n| n.attributes.get(TAG_ATTR)) {
            return Some(tag.to_string());
        }
        for proto in Prototype::prototype_nodes(self.graph, node, true) {
            if let Some(Val::Str(tag)) = proto.node(self.graph).and_then(|n| n.attributes.get(TAG_ATTR)) {
                return Some(tag.to_string());
            }
        }
        None
    }

    /// The object's fields, then #[attr]/#[text] functions (its own, then its prototypes', nearest first).
    fn entries(&self, node: &NodeRef) -> Vec<Entry<'a>> {
        let graph = self.graph;
        let mut entries = Vec::new();
        let mut seen: FxHashSet<&str> = FxHashSet::default();
        let Some(obj) = node.node(graph) else { return entries; };
        for (name, dref) in &obj.data {
            if let Some(field) = graph.get_stof_data::<Field>(dref) {
                seen.insert(name.as_str());
                if hidden(graph, node, field) { continue; }
                entries.push(Entry { name: name.as_str(), attrs: &field.attributes, value: Value::Val(field.value.get()) });
            } else if let Some(func) = graph.get_stof_data::<Func>(dref) {
                seen.insert(name.as_str());
                if marked(&func.attributes) {
                    entries.push(Entry { name: name.as_str(), attrs: &func.attributes, value: Value::Method(dref.clone()) });
                }
            }
        }
        for proto in Prototype::prototype_nodes(graph, node, true) {
            let Some(proto) = proto.node(graph) else { continue; };
            for (name, dref) in &proto.data {
                if seen.contains(name.as_str()) { continue; }
                if let Some(func) = graph.get_stof_data::<Func>(dref) {
                    seen.insert(name.as_str());
                    if marked(&func.attributes) {
                        entries.push(Entry { name: name.as_str(), attrs: &func.attributes, value: Value::Method(dref.clone()) });
                    }
                }
            }
        }
        entries
    }

    /// The value to write for an entry: a field's value, or a computed value (None when it isn't known yet).
    fn resolve(&mut self, node: &NodeRef, entry: &Entry) -> Option<Val> {
        let (func, method) = match &entry.value {
            Value::Val(Val::Fn(dref)) => (dref.clone(), false),
            Value::Val(val) => return Some(val.clone()),
            Value::Method(dref) => (dref.clone(), true),
        };
        if let Some(val) = self.known.get(node, entry.name) {
            return Some(val.clone());
        }
        if self.collect {
            let key = (node.clone(), ArcStr::from(entry.name));
            if self.missing_keys.insert(key.clone()) {
                self.missing.push(ComputedCall { obj: key.0, name: key.1, func, method });
            }
        }
        None
    }

    fn element(&mut self, name: &str, node: &NodeRef) -> Result<(), Error> {
        if self.path.contains(node) { return Ok(()); } // holds one of its ancestors
        if !valid_name(name) {
            if self.markup == Markup::Xml {
                return Err(Error::XMLStringExport(format!("'{}' is not a valid XML element name", name)));
            }
            return Ok(()); // html: not an element name, so not written
        }
        let lower = name.to_ascii_lowercase();
        if self.markup == Markup::Html && !html_tag_allowed(self.graph, &lower) {
            return Ok(()); // with its content
        }

        self.path.insert(node.clone());
        let mut open = String::new();
        open.push('<');
        open.push_str(name);
        let res = self.attributes(node, &mut open).and_then(|_| {
            let outer = std::mem::take(&mut self.out);
            let res = if self.markup == Markup::Html && HTML_VOID.contains(&lower.as_str()) { Ok(()) } else { self.content(node) };
            let inner = std::mem::replace(&mut self.out, outer);
            res.map(|_| inner)
        });
        self.path.remove(node);
        let inner = res?;

        self.out.push_str(&open);
        if self.markup == Markup::Html {
            self.out.push('>');
            if !HTML_VOID.contains(&lower.as_str()) {
                self.out.push_str(&inner);
                self.out.push_str("</");
                self.out.push_str(name);
                self.out.push('>');
            }
        } else if inner.is_empty() {
            self.out.push_str("/>");
        } else {
            self.out.push('>');
            self.out.push_str(&inner);
            self.out.push_str("</");
            self.out.push_str(name);
            self.out.push('>');
        }
        Ok(())
    }

    fn attributes(&mut self, node: &NodeRef, open: &mut String) -> Result<(), Error> {
        for entry in self.entries(node) {
            let Some(attr) = entry.attrs.get(ATTR_ATTR) else { continue; };
            let name = match attr {
                Val::Str(name) => name.to_string(),
                _ => entry.name.to_string(),
            };
            let Some(value) = self.resolve(node, &entry) else { continue; };
            if !valid_name(&name) {
                if self.markup == Markup::Xml {
                    return Err(Error::XMLStringExport(format!("'{}' is not a valid XML attribute name", name)));
                }
                continue;
            }
            let text = match value {
                Val::Void | Val::Null | Val::Promise(..) | Val::Fn(_) => continue,
                Val::Bool(true) if self.markup == Markup::Html => {
                    if html_attr_allowed(&name, "") {
                        open.push(' ');
                        open.push_str(&name);
                    }
                    continue;
                },
                Val::Bool(false) if self.markup == Markup::Html => continue,
                Val::Bool(b) => b.to_string(),
                Val::Str(s) => s.to_string(),
                Val::Prompt(p) => p.to_string(),
                Val::Num(_) | Val::Ver(..) => value.to_string(),
                other => json_string(self.graph, other),
            };
            if self.markup == Markup::Html && !html_attr_allowed(&name, &text) { continue; }
            open.push(' ');
            open.push_str(&name);
            open.push_str("=\"");
            escape_into(open, &text, true, self.markup);
            open.push('"');
        }
        Ok(())
    }

    fn content(&mut self, node: &NodeRef) -> Result<(), Error> {
        for entry in self.entries(node) {
            if entry.attrs.contains_key(ATTR_ATTR) { continue; }
            let text = entry.attrs.contains_key(TEXT_ATTR);
            if !text && matches!(entry.value, Value::Method(_)) { continue; }
            if !text && matches!(entry.value, Value::Val(Val::Fn(_))) { continue; } // functions aren't content
            let Some(value) = self.resolve(node, &entry) else { continue; };
            if text {
                self.text(value)?;
            } else {
                self.value(entry.name, entry.attrs, value)?;
            }
        }
        Ok(())
    }

    /// #[text]: escaped text, or child elements for objects.
    fn text(&mut self, value: Val) -> Result<(), Error> {
        match value {
            Val::Void | Val::Null | Val::Promise(..) | Val::Fn(_) => Ok(()),
            Val::Obj(obj) => {
                match self.tag_of(None, &obj) {
                    Some(tag) => self.element(&tag, &obj),
                    None if self.markup == Markup::Html => {
                        // no tag: its content, like a fragment
                        if self.path.contains(&obj) { return Ok(()); }
                        self.path.insert(obj.clone());
                        let res = self.content(&obj);
                        self.path.remove(&obj);
                        res
                    },
                    None => {
                        let name = obj.node(self.graph).map(|n| n.name.as_ref().to_string()).unwrap_or_default();
                        self.element(&name, &obj)
                    },
                }
            },
            Val::List(items) | Val::Tup(items) => {
                for item in items { self.text(item.read().clone())?; }
                Ok(())
            },
            Val::Set(items) => {
                for item in items { self.text(item.read().clone())?; }
                Ok(())
            },
            Val::Str(s) => { escape_into(&mut self.out, &s, false, self.markup); Ok(()) },
            Val::Map(_) | Val::Blob(_) | Val::Data(_) => {
                let json = json_string(self.graph, value);
                escape_into(&mut self.out, &json, false, self.markup);
                Ok(())
            },
            other => { escape_into(&mut self.out, &other.to_string(), false, self.markup); Ok(()) },
        }
    }

    /// A field without #[attr] or #[text]: a child element (or one per list item).
    fn value(&mut self, name: &str, attrs: &rustc_hash::FxHashMap<String, Val>, value: Val) -> Result<(), Error> {
        match value {
            Val::Void | Val::Null | Val::Promise(..) | Val::Fn(_) => Ok(()),
            Val::Obj(obj) => {
                if obj.node(self.graph).is_some_and(|n| n.attributes.contains_key(NOEXPORT_FIELD_ATTR.as_str())) { return Ok(()); }
                let tag = self.tag_of(Some(attrs), &obj).unwrap_or_else(|| name.to_string());
                self.element(&tag, &obj)
            },
            Val::List(items) | Val::Tup(items) => {
                for item in items { self.value(name, attrs, item.read().clone())?; }
                Ok(())
            },
            Val::Set(items) => {
                for item in items { self.value(name, attrs, item.read().clone())?; }
                Ok(())
            },
            other => {
                let tag = match attrs.get(TAG_ATTR) { Some(Val::Str(tag)) => tag.to_string(), _ => name.to_string() };
                if !valid_name(&tag) {
                    if self.markup == Markup::Xml {
                        return Err(Error::XMLStringExport(format!("'{}' is not a valid XML element name", tag)));
                    }
                    return Ok(());
                }
                if self.markup == Markup::Html && !html_tag_allowed(self.graph, &tag.to_ascii_lowercase()) { return Ok(()); }
                let text = match other {
                    Val::Str(s) => s.to_string(),
                    Val::Map(_) | Val::Blob(_) | Val::Data(_) => json_string(self.graph, other),
                    other => other.to_string(),
                };
                if self.markup == Markup::Xml && text.is_empty() {
                    self.out.push('<');
                    self.out.push_str(&tag);
                    self.out.push_str("/>");
                    return Ok(());
                }
                self.out.push('<');
                self.out.push_str(&tag);
                self.out.push('>');
                if !(self.markup == Markup::Html && HTML_VOID.contains(&tag.to_ascii_lowercase().as_str())) {
                    escape_into(&mut self.out, &text, false, self.markup);
                    self.out.push_str("</");
                    self.out.push_str(&tag);
                    self.out.push('>');
                }
                Ok(())
            },
        }
    }
}


/// A #[no-export] field. For a child object, the object's own attributes decide, like the json format (imports
/// mark object fields #[no-export] while their objects are still exported).
fn hidden(graph: &Graph, node: &NodeRef, field: &Field) -> bool {
    if let Val::Obj(child) = field.value.get() {
        if let Some(child) = child.node(graph) {
            if child.parent.as_ref() == Some(node) {
                return child.attributes.contains_key(NOEXPORT_FIELD_ATTR.as_str());
            }
        }
    }
    field.attributes.contains_key(NOEXPORT_FIELD_ATTR.as_str())
}

/// Marked for an element format (#[attr] or #[text])?
fn marked(attrs: &rustc_hash::FxHashMap<String, Val>) -> bool {
    !attrs.contains_key(NOEXPORT_FIELD_ATTR.as_str()) && (attrs.contains_key(ATTR_ATTR) || attrs.contains_key(TEXT_ATTR))
}

/// A name that's safe to write as an element or attribute name (a valid XML name, without the rarer characters).
fn valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else { return false; };
    (first.is_alphabetic() || first == '_') && chars.all(|c| c.is_alphanumeric() || matches!(c, '-' | '_' | '.' | ':'))
}

/// May html write this element (lowercase name)?
fn html_tag_allowed(graph: &Graph, tag: &str) -> bool {
    if let Some(tags) = &graph.html_tags {
        return tags.contains(tag);
    }
    !HTML_NEVER.contains(&tag) && (HTML_STANDARD.contains(&tag) || tag.contains('-'))
}

/// May html write this attribute with this value?
fn html_attr_allowed(name: &str, value: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    if lower.starts_with("on") || lower == "srcdoc" { return false; }
    // URLs that run code: checked on every attribute (cheap, and covers href, src, action, xlink:href, srcset, ...)
    let compact: String = value.chars().filter(|c| !c.is_whitespace() && !c.is_control()).collect::<String>().to_ascii_lowercase();
    if compact.contains("javascript:") || compact.contains("vbscript:") { return false; }
    if compact.starts_with("data:") {
        let image = ["data:image/png", "data:image/jpeg", "data:image/jpg", "data:image/gif", "data:image/webp", "data:image/avif"];
        return image.iter().any(|prefix| compact.starts_with(prefix));
    }
    true
}

/// Escape text (or an attribute value) for markup. XML also drops characters it can't hold.
fn escape_into(out: &mut String, text: &str, attribute: bool, markup: Markup) {
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' if attribute => out.push_str("&quot;"),
            // xml normalizes whitespace in attribute values when it reads them: keep it as character references
            '\n' if attribute && markup == Markup::Xml => out.push_str("&#10;"),
            '\r' if attribute && markup == Markup::Xml => out.push_str("&#13;"),
            '\t' if attribute && markup == Markup::Xml => out.push_str("&#9;"),
            c if markup == Markup::Xml && c.is_control() && !matches!(c, '\t' | '\n' | '\r') => {},
            c => out.push(c),
        }
    }
}

impl Graph {
    /// Elements html may write (Ex. the components a host renders): only these, replacing the default (the
    /// standard HTML elements and custom elements). Set by the host, so documents can't change it. Calling it again
    /// adds elements. Names are compared without case.
    pub fn allow_html_tags(&mut self, tags: impl IntoIterator<Item = impl AsRef<str>>) {
        let mut allowed = self.html_tags.as_deref().cloned().unwrap_or_default();
        for tag in tags {
            allowed.insert(tag.as_ref().trim().to_ascii_lowercase());
        }
        self.html_tags = Some(Arc::new(allowed));
    }
}


#[cfg(test)]
mod tests {
    use crate::model::{Graph, Profile};

    const VIEW: &str = r#"
        #[type]
        #[tag('ui-card')]
        UiCard: {
            #[attr] str label: '';
            #[attr] float | fn value: 0;
        }
        #[tag('div')]
        view: {
            UiCard card: {
                label: 'Revenue';
                value: (): float => 40 + 2;
            }
            #[tag('span')] note: 'hi';
            #[tag('x-other')] other: {}
        }
    "#;

    fn view_graph() -> Graph {
        let mut graph = Graph::default();
        graph.parse_stof_src(VIEW, None, Profile::default()).unwrap();
        graph
    }

    #[test]
    fn host_stringify_calls_computed_values() {
        let mut graph = view_graph();
        let view = graph.find_node_named("root.view", None).unwrap();
        // the plain export can't call functions: computed values are left out
        assert_eq!(graph.string_export("html", Some(view.clone())).unwrap(), r#"<div><ui-card label="Revenue"></ui-card><span>hi</span><x-other></x-other></div>"#);
        assert_eq!(graph.stringify("html", Some(view.clone())).unwrap(), r#"<div><ui-card label="Revenue" value="42"></ui-card><span>hi</span><x-other></x-other></div>"#);
        assert_eq!(graph.blobify("html", Some(view)).unwrap().as_ref(), br#"<div><ui-card label="Revenue" value="42"></ui-card><span>hi</span><x-other></x-other></div>"#);
    }

    #[test]
    fn host_tag_allowlist_replaces_the_defaults() {
        let mut graph = view_graph();
        graph.allow_html_tags(["DIV", "ui-card"]);
        let view = graph.find_node_named("root.view", None).unwrap();
        assert_eq!(graph.stringify("html", Some(view)).unwrap(), r#"<div><ui-card label="Revenue" value="42"></ui-card></div>"#);

        // a document can't widen it
        let mut graph = view_graph();
        graph.allow_html_tags(["div"]);
        graph.parse_stof_src("#[main] fn main() { allow_html_tags(['span']); }", None, Profile::default()).unwrap();
        let _ = graph.run(None, true);
        let view = graph.find_node_named("root.view", None).unwrap();
        assert_eq!(graph.stringify("html", Some(view)).unwrap(), "<div></div>");
    }

    #[test]
    fn names_that_arent_names_are_never_written() {
        let mut graph = Graph::default();
        graph.parse_stof_src(r#"
            #[tag('div')]
            view: {
                #[attr('x onload=alert(1)')] a: 1;
                #[tag('img src=x onerror=alert(1)')] b: 'x';
                #[attr] ok: 'yes';
            }
        "#, None, Profile::default()).unwrap();
        let view = graph.find_node_named("root.view", None).unwrap();
        assert_eq!(graph.stringify("html", Some(view.clone())).unwrap(), r#"<div ok="yes"></div>"#);
        assert!(graph.stringify("xml", Some(view)).is_err());
    }

    #[test]
    fn an_object_holding_its_ancestor_is_written_once() {
        let mut graph = Graph::default();
        graph.parse_stof_src(r#"
            #[tag('div')]
            view: {
                #[tag('section')]
                inner: {
                    #[text] label: 'x';
                }
            }
            #[main] fn main() { self.view.inner.back = self.view; }
        "#, None, Profile::default()).unwrap();
        graph.run(None, true).unwrap();
        let view = graph.find_node_named("root.view", None).unwrap();
        assert_eq!(graph.stringify("html", Some(view)).unwrap(), "<div><section>x</section></div>");
    }
}
