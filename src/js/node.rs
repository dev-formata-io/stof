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

use wasm_bindgen::prelude::*;
use crate::{SNode, SNodeRef, Store};
use super::{StofData, StofDoc, StofField};


/// Stof Node.
#[wasm_bindgen]
pub struct StofNode {
    id: String,
}
impl StofNode {
    /// Return the SNodeRef for this node.
    pub fn node_ref(&self) -> SNodeRef {
        SNodeRef::from(&self.id)
    }

    /// Get the node.
    pub fn node<'a>(&self, doc: &'a StofDoc) -> Option<&'a SNode> {
        self.node_ref().node(&doc.doc().graph)
    }

    /// Get a mutable reference to the node.
    pub fn node_mut<'a>(&self, doc: &'a mut StofDoc) -> Option<&'a mut SNode> {
        self.node_ref().node_mut(&mut doc.doc_mut().graph)
    }
}
#[wasm_bindgen]
impl StofNode {
    /// ID constructor.
    #[wasm_bindgen(constructor)]
    pub fn new(id: &str) -> Self {
        Self { id: id.to_owned() }
    }

    /// Path constructor.
    #[wasm_bindgen(js_name = fromPath)]
    pub fn path_from(doc: &StofDoc, path: &str) -> Option<Self> {
        if let Some(nref) = doc.doc().graph.node_ref(path, None) {
            return Some(Self::new(&nref.id));
        }
        None
    }

    /// Path from a starting point constructor.
    #[wasm_bindgen(js_name = fromPathStart)]
    pub fn path_from_start(doc: &StofDoc, path: &str, start: &Self) -> Option<Self> {
        if let Some(nref) = doc.doc().graph.node_ref(path, Some(&start.node_ref())) {
            return Some(Self::new(&nref.id));
        }
        None
    }

    /// Get the ID of this node reference.
    pub fn id(&self) -> String {
        self.id.clone()
    }

    /// Invalidate this node with a symbol.
    pub fn invalidate(&self, doc: &mut StofDoc, symbol: &str) -> bool {
        if let Some(node) = self.node_mut(doc) {
            node.invalidate(symbol);
            return true;
        }
        false
    }

    /// Invalidate all on this node.
    #[wasm_bindgen(js_name = invalidateAll)]
    pub fn invalidate_all(&self, doc: &mut StofDoc) -> bool {
        if let Some(node) = self.node_mut(doc) {
            node.invalidate_all();
            return true;
        }
        false
    }

    /// Dirty?
    pub fn dirty(&self, doc: &StofDoc, symbol: &str) -> bool {
        if let Some(node) = self.node(doc) {
            return node.dirty(symbol);
        }
        false
    }

    /// Any dirty symbols?
    #[wasm_bindgen(js_name = anyDirty)]
    pub fn any_dirty(&self, doc: &StofDoc) -> bool {
        if let Some(node) = self.node(doc) {
            return node.has_dirty();
        }
        false
    }

    /// Validate this node with the symbol.
    pub fn validate(&self, doc: &mut StofDoc, symbol: &str) -> bool {
        if let Some(node) = self.node_mut(doc) {
            return node.validate(symbol);
        }
        false
    }

    /// Validate all for this node.
    pub fn validate_all(&self, doc: &mut StofDoc) -> bool {
        if let Some(node) = self.node_mut(doc) {
            return node.validate_all();
        }
        false
    }

    /// Root node for this reference.
    pub fn root(&self, doc: &StofDoc) -> Option<Self> {
        if let Some(nref) = self.node_ref().root(&doc.doc().graph) {
            return Some(Self::new(&nref.id));
        }
        None
    }

    /// Exists within the document?
    pub fn exists(&self, doc: &StofDoc) -> bool {
        self.node_ref().exists(&doc.doc().graph)
    }

    /// Is a child of the 'parent' node?
    /// Returns true if this and parent are equal.
    /// Returns true if this node is a granchild or below.
    #[wasm_bindgen(js_name = isChildOf)]
    pub fn is_child_of(&self, doc: &StofDoc, parent: &Self) -> bool {
        self.node_ref().is_child_of(&doc.doc().graph, &parent.node_ref())
    }

    /// Is an immediate child of 'parent'?
    /// Will return false if this node is a grandchild or below.
    #[wasm_bindgen(js_name = isImmediateChildOf)]
    pub fn is_immediate_child_of(&self, doc: &StofDoc, parent: &Self) -> bool {
        if let Some(node) = parent.node(doc) {
            return node.has_child(&self.node_ref());
        }
        false
    }

    /// Return the named path of this node.
    /// Path is '/' separated and starts at this nodes root.
    pub fn path(&self, doc: &StofDoc) -> String {
        self.node_ref().path(&doc.doc().graph)
    }

    /// Return the ID path of this node.
    #[wasm_bindgen(js_name = idPath)]
    pub fn id_path(&self, doc: &StofDoc) -> Vec<String> {
        self.node_ref().id_path(&doc.doc().graph)
    }

    /// Distance to another node in the document.
    #[wasm_bindgen(js_name = distanceTo)]
    pub fn distance_to(&self, doc: &StofDoc, other: &Self) -> i32 {
        self.node_ref().distance_to(&doc.doc().graph, &other.node_ref())
    }

    /// Build this nodes trie for searching through data.
    /// Should already be built, but nice to have just in case.
    pub fn build_trie(&self, doc: &mut StofDoc) {
        if let Some(node) = self.node_mut(doc) {
            node.build_trie();
        }
    }

    /// Name of this node.
    pub fn name(&self, doc: &StofDoc) -> Option<String> {
        if let Some(node) = self.node(doc) {
            return Some(node.name.clone());
        }
        None
    }

    /// Parent of this node.
    pub fn parent(&self, doc: &StofDoc) -> Option<Self> {
        if let Some(node) = self.node(doc) {
            if let Some(parent) = &node.parent {
                return Some(Self::new(&parent.id));
            }
        }
        None
    }

    /// Children of this node.
    pub fn children(&self, doc: &StofDoc) -> Vec<Self> {
        let mut children = Vec::new();
        if let Some(node) = self.node(doc) {
            for child in &node.children {
                children.push(Self::new(&child.id));
            }
        }
        children
    }

    /// Data on this node.
    pub fn data(&self, doc: &StofDoc) -> Vec<StofData> {
        let mut data = Vec::new();
        if let Some(node) = self.node(doc) {
            for dref in &node.data {
                data.push(StofData::new(&dref.id));
            }
        }
        data
    }

    /// All data on all children nodes.
    #[wasm_bindgen(js_name = allData)]
    pub fn all_data(&self, doc: &StofDoc) -> Vec<StofData> {
        let mut data = Vec::new();
        if let Some(node) = self.node(doc) {
            for dref in node.recursive_selection(&doc.doc().graph) {
                data.push(StofData::new(&dref.id));
            }
        }
        data
    }

    /// Has data?
    #[wasm_bindgen(js_name = hasData)]
    pub fn has_data(&self, doc: &StofDoc, data: &StofData) -> bool {
        if let Some(node) = self.node(doc) {
            return node.has_data(&data.data_ref());
        }
        false
    }

    /// Data on this node with an ID that has the prefix 'prefix'.
    #[wasm_bindgen(js_name = prefixData)]
    pub fn prefix_data(&self, doc: &StofDoc, prefix: &str) -> Vec<StofData> {
        let mut data = Vec::new();
        if let Some(node) = self.node(doc) {
            for dref in node.prefix_selection(prefix) {
                data.push(StofData::new(&dref.id));
            }
        }
        data
    }

    /// All data on all children nodes with an ID that has the prefix 'prefix'.
    #[wasm_bindgen(js_name = allPrefixData)]
    pub fn all_prefix_data(&self, doc: &StofDoc, prefix: &str) -> Vec<StofData> {
        let mut data = Vec::new();
        if let Some(node) = self.node(doc) {
            for dref in node.recursive_prefix_selection(&doc.doc().graph, prefix) {
                data.push(StofData::new(&dref.id));
            }
        }
        data
    }

    /// Create some abstract data on this node.
    #[wasm_bindgen(js_name = createData)]
    pub fn create_data(&self, doc: &mut StofDoc, value: JsValue) -> Result<StofData, String> {
        StofData::construct(doc, self, value)
    }

    /// Create a new field on this node.
    #[wasm_bindgen(js_name = createField)]
    pub fn create_field(&self, doc: &mut StofDoc, name: &str, value: JsValue) -> StofField {
        let mut field = StofField::new(&doc, name, value);
        field.attach(doc, self);
        field
    }

    /// JSON value of this node as a whole.
    /// Can use this to store this value in an external place.
    pub fn to_json(&self, doc: &StofDoc) -> JsValue {
        if let Some(node) = self.node(doc) {
            if let Ok(val) = serde_wasm_bindgen::to_value(node) {
                return val;
            }
        }
        JsValue::null()
    }

    /// Loat a JSON representation of a node into a document.
    /// Can use this to load nodes from an external place.
    pub fn from_json(doc: &mut StofDoc, json: JsValue) -> bool {
        if let Ok(node) = serde_wasm_bindgen::from_value::<SNode>(json) {
            doc.doc_mut().graph.nodes.set(&node.id.clone(), node);
            return true;
        }
        false
    }
}
