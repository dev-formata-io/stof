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

//! XML import: the reverse of the xml export, so a document comes back as it was exported.
//! - The root element's content goes into the object it's parsed into (like json). When the root's name isn't the
//!   object's name, the object gets `#[tag('root name')]`, so it exports under the same name.
//! - An element with only text (or nothing) is a string field: `<total>42.5</total>` is `total: '42.5'`.
//! - Any other element is an object field. Its attributes are `#[attr]` string fields, and text mixed in with child
//!   elements is a `#[text]` field named `text`.
//! - Repeated elements with the same name are a list, in order.
//! - Values stay strings: XML has no types, and guessing would make results depend on the data.
//! - Comments, processing instructions, and the declaration are dropped. Namespace prefixes stay in names.
//!
//! Elements of different names that alternate (`<a/><b/><a/>`) come back grouped by name: each name keeps its order.

use imbl::Vector;
use quick_xml::{Reader, events::{BytesStart, Event}};
use rustc_hash::FxHashMap;
use crate::{model::{Field, Graph, NodeRef, NOEXPORT_FIELD_ATTR, SId}, runtime::{Error, Val, ValRef, Variable}};
use super::{ATTR_ATTR, TAG_ATTR, TEXT_ATTR};


/// A parsed element.
#[derive(Debug, Default)]
struct Elem {
    name: String,
    attrs: Vec<(String, String)>,
    children: Vec<Elem>,
    text: String,
}

impl Elem {
    fn simple(&self) -> bool {
        self.attrs.is_empty() && self.children.is_empty()
    }
}


/// Parse an XML document into a node (the main root when none is given).
pub(super) fn xml_import(graph: &mut Graph, src: &str, node: Option<NodeRef>) -> Result<(), Error> {
    let root = parse(src)?;
    let Some(root) = root else { return Ok(()); };
    let node = match node {
        Some(node) => node,
        None => graph.ensure_main_root(),
    };

    let node_name = node.node(graph).map(|n| n.name.as_ref().to_string()).unwrap_or_default();
    if root.name != node_name {
        if let Some(obj) = node.node_mut(graph) {
            obj.insert_attribute(TAG_ATTR.to_string(), Val::Str(root.name.as_str().into()));
        }
    }
    content(graph, &node, root);
    Ok(())
}


/// Parse the source into its root element.
fn parse(src: &str) -> Result<Option<Elem>, Error> {
    let mut reader = Reader::from_str(src);
    let mut stack: Vec<Elem> = Vec::new();
    let mut root: Option<Elem> = None;
    loop {
        let event = reader.read_event().map_err(|e| Error::XMLStringImport(format!("{} (at byte {})", e, reader.error_position())))?;
        match event {
            Event::Start(start) => stack.push(element(&start)?),
            Event::Empty(start) => {
                let elem = element(&start)?;
                close(&mut stack, &mut root, elem)?;
            },
            Event::End(_) => {
                let Some(elem) = stack.pop() else { return Err(Error::XMLStringImport("an end tag without a start".into())); };
                close(&mut stack, &mut root, elem)?;
            },
            Event::Text(text) => {
                let text = text.xml10_content().map_err(|e| Error::XMLStringImport(e.to_string()))?;
                if let Some(elem) = stack.last_mut() { elem.text.push_str(&text); }
            },
            Event::CData(data) => {
                let text = data.decode().map_err(|e| Error::XMLStringImport(e.to_string()))?;
                if let Some(elem) = stack.last_mut() { elem.text.push_str(&text); }
            },
            Event::GeneralRef(reference) => {
                let resolved = match reference.resolve_char_ref().map_err(|e| Error::XMLStringImport(e.to_string()))? {
                    Some(c) => c.to_string(),
                    None => {
                        let name = reference.decode().map_err(|e| Error::XMLStringImport(e.to_string()))?;
                        match name.as_ref() {
                            "lt" => "<".into(),
                            "gt" => ">".into(),
                            "amp" => "&".into(),
                            "apos" => "'".into(),
                            "quot" => "\"".into(),
                            other => format!("&{};", other), // an entity the document defines: left as it is
                        }
                    },
                };
                if let Some(elem) = stack.last_mut() { elem.text.push_str(&resolved); }
            },
            Event::Eof => break,
            Event::Comment(_) | Event::Decl(_) | Event::PI(_) | Event::DocType(_) => {},
        }
    }
    if !stack.is_empty() {
        return Err(Error::XMLStringImport(format!("'{}' is never closed", stack.last().map(|e| e.name.as_str()).unwrap_or_default())));
    }
    Ok(root)
}

fn element(start: &BytesStart) -> Result<Elem, Error> {
    let name = String::from_utf8_lossy(start.name().as_ref()).to_string();
    let mut attrs = Vec::new();
    for attr in start.attributes() {
        let attr = attr.map_err(|e| Error::XMLStringImport(e.to_string()))?;
        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
        let value = attr.normalized_value(quick_xml::XmlVersion::Implicit1_0).map_err(|e| Error::XMLStringImport(e.to_string()))?;
        attrs.push((key, value.to_string()));
    }
    Ok(Elem { name, attrs, ..Default::default() })
}

fn close(stack: &mut Vec<Elem>, root: &mut Option<Elem>, elem: Elem) -> Result<(), Error> {
    match stack.last_mut() {
        Some(parent) => parent.children.push(elem),
        None => {
            if root.is_some() {
                return Err(Error::XMLStringImport("more than one root element".into()));
            }
            *root = Some(elem);
        },
    }
    Ok(())
}


/// An element's content into an object: attributes, then child elements, then text.
fn content(graph: &mut Graph, node: &NodeRef, elem: Elem) {
    let element_names: Vec<String> = elem.children.iter().map(|c| c.name.clone()).collect();
    let text = elem.text.trim().to_string();
    for (name, value) in elem.attrs.iter().cloned() {
        let mut attrs = FxHashMap::default();
        // an attribute with the name of a child element is stored as '@name' (still written as the attribute 'name')
        let key = if element_names.contains(&name) {
            attrs.insert(ATTR_ATTR.to_string(), Val::Str(name.as_str().into()));
            format!("@{}", name)
        } else {
            attrs.insert(ATTR_ATTR.to_string(), Val::Null);
            name
        };
        insert(graph, node, &key, Val::Str(value.into()), attrs);
    }

    // group repeated elements into lists, in order
    let mut groups: Vec<(String, Vec<Elem>)> = Vec::new();
    for child in elem.children {
        match groups.iter_mut().find(|(name, _)| *name == child.name) {
            Some((_, items)) => items.push(child),
            None => groups.push((child.name.clone(), vec![child])),
        }
    }
    for (name, mut items) in groups {
        if items.len() == 1 {
            let item = items.pop().unwrap();
            if item.simple() {
                insert(graph, node, &name, Val::Str(item.text.into()), FxHashMap::default());
            } else {
                let child = graph.insert_node(name.as_str(), Some(node.clone()), true);
                content(graph, &child, item);
                let mut attrs = FxHashMap::default();
                attrs.insert(NOEXPORT_FIELD_ATTR.to_string(), Val::Null); // like json: the object is exported as a child
                insert(graph, node, &name, Val::Obj(child), attrs);
            }
        } else {
            let mut list = Vector::default();
            for item in items {
                if item.simple() {
                    list.push_back(ValRef::new(Val::Str(item.text.into())));
                } else {
                    let id = SId::default();
                    let child = graph.insert_node_id(&id, &id, Some(node.clone()), false);
                    content(graph, &child, item);
                    list.push_back(ValRef::new(Val::Obj(child)));
                }
            }
            insert(graph, node, &name, Val::List(list), FxHashMap::default());
        }
    }

    // text next to attributes or child elements (an element with only text is a string field in its parent):
    // a #[text] field named 'text', or '_text' when an attribute or element has that name
    if !text.is_empty() {
        let mut attrs = FxHashMap::default();
        attrs.insert(TEXT_ATTR.to_string(), Val::Null);
        let taken = element_names.iter().any(|n| n == TEXT_ATTR) || elem.attrs.iter().any(|(n, _)| n == TEXT_ATTR);
        insert(graph, node, if taken { "_text" } else { TEXT_ATTR }, Val::Str(text.into()), attrs);
    }
}

fn insert(graph: &mut Graph, node: &NodeRef, name: &str, val: Val, attrs: FxHashMap<String, Val>) {
    let field = Field::new(Variable::new(graph, true, val, false), Some(attrs));
    graph.insert_stof_data(node, name, Box::new(field), None);
}
