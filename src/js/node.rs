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
use super::{StofData, StofDoc, DOCS};


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
        unsafe {
            if let Some(doc) = DOCS.get(&doc.id()) {
                if let Some(nref) = doc.graph.node_ref(path, None) {
                    return Some(Self::new(&nref.id));
                }
            }
        }
        None
    }

    /// Path from a starting point constructor.
    #[wasm_bindgen(js_name = fromPathStart)]
    pub fn path_from_start(doc: &StofDoc, path: &str, start: &Self) -> Option<Self> {
        unsafe {
            if let Some(doc) = DOCS.get(&doc.id()) {
                if let Some(nref) = doc.graph.node_ref(path, Some(&start.node_ref())) {
                    return Some(Self::new(&nref.id));
                }
            }
        }
        None
    }

    /// Get the ID of this node reference.
    pub fn id(&self) -> String {
        self.id.clone()
    }

    /// Invalidate this node with a symbol.
    pub fn invalidate(&self, doc: &StofDoc, symbol: &str) -> bool {
        unsafe {
            if let Some(doc) = DOCS.get_mut(&doc.id()) {
                if let Some(node) = self.node_ref().node_mut(&mut doc.graph) {
                    node.invalidate(symbol);
                    return true;
                }
            }
        }
        false
    }

    /// Invalidate all on this node.
    #[wasm_bindgen(js_name = invalidateAll)]
    pub fn invalidate_all(&self, doc: &StofDoc) -> bool {
        unsafe {
            if let Some(doc) = DOCS.get_mut(&doc.id()) {
                if let Some(node) = self.node_ref().node_mut(&mut doc.graph) {
                    node.invalidate_all();
                    return true;
                }
            }
        }
        false
    }

    /// Dirty?
    pub fn dirty(&self, doc: &StofDoc, symbol: &str) -> bool {
        unsafe {
            if let Some(doc) = DOCS.get(&doc.id()) {
                if let Some(node) = self.node_ref().node(&doc.graph) {
                    return node.dirty(symbol);
                }
            }
        }
        false
    }

    /// Any dirty symbols?
    #[wasm_bindgen(js_name = anyDirty)]
    pub fn any_dirty(&self, doc: &StofDoc) -> bool {
        unsafe {
            if let Some(doc) = DOCS.get(&doc.id()) {
                if let Some(node) = self.node_ref().node(&doc.graph) {
                    return node.has_dirty();
                }
            }
        }
        false
    }

    /// Validate this node with the symbol.
    pub fn validate(&self, doc: &StofDoc, symbol: &str) -> bool {
        unsafe {
            if let Some(doc) = DOCS.get_mut(&doc.id()) {
                if let Some(node) = self.node_ref().node_mut(&mut doc.graph) {
                    return node.validate(symbol);
                }
            }
        }
        false
    }

    /// Validate all for this node.
    #[wasm_bindgen(js_name = validateAll)]
    pub fn validate_all(&self, doc: &StofDoc) -> bool {
        unsafe {
            if let Some(doc) = DOCS.get_mut(&doc.id()) {
                if let Some(node) = self.node_ref().node_mut(&mut doc.graph) {
                    return node.validate_all();
                }
            }
        }
        false
    }

    /// Root node for this reference.
    pub fn root(&self, doc: &StofDoc) -> Option<Self> {
        unsafe {
            if let Some(doc) = DOCS.get(&doc.id()) {
                if let Some(nref) = self.node_ref().root(&doc.graph) {
                    return Some(Self::new(&nref.id));
                }
            }
        }
        None
    }

    /// Exists within the document?
    pub fn exists(&self, doc: &StofDoc) -> bool {
        unsafe {
            if let Some(doc) = DOCS.get(&doc.id()) {
                return self.node_ref().exists(&doc.graph);
            }
        }
        false
    }

    /// Is a child of the 'parent' node?
    /// Returns true if this and parent are equal.
    /// Returns true if this node is a granchild or below.
    #[wasm_bindgen(js_name = isChildOf)]
    pub fn is_child_of(&self, doc: &StofDoc, parent: &Self) -> bool {
        unsafe {
            if let Some(doc) = DOCS.get(&doc.id()) {
                return self.node_ref().is_child_of(&doc.graph, &parent.node_ref());
            }
        }
        false
    }

    /// Is an immediate child of 'parent'?
    /// Will return false if this node is a grandchild or below.
    #[wasm_bindgen(js_name = isImmediateChildOf)]
    pub fn is_immediate_child_of(&self, doc: &StofDoc, parent: &Self) -> bool {
        unsafe {
            if let Some(doc) = DOCS.get(&doc.id()) {
                if let Some(parent) = parent.node_ref().node(&doc.graph) {
                    return parent.has_child(&self.node_ref());
                }
            }
        }
        false
    }

    /// Return the named path of this node.
    /// Path is '/' separated and starts at this nodes root.
    pub fn path(&self, doc: &StofDoc) -> String {
        unsafe {
            if let Some(doc) = DOCS.get(&doc.id()) {
                return self.node_ref().path(&doc.graph);
            }
        }
        String::default()
    }

    /// Return the ID path of this node.
    #[wasm_bindgen(js_name = idPath)]
    pub fn id_path(&self, doc: &StofDoc) -> Vec<String> {
        unsafe {
            if let Some(doc) = DOCS.get(&doc.id()) {
                return self.node_ref().id_path(&doc.graph);
            }
        }
        vec![]
    }

    /// Distance to another node in the document.
    #[wasm_bindgen(js_name = distanceTo)]
    pub fn distance_to(&self, doc: &StofDoc, other: &Self) -> i32 {
        unsafe {
            if let Some(doc) = DOCS.get(&doc.id()) {
                return self.node_ref().distance_to(&doc.graph, &other.node_ref());
            }
        }
        -1
    }

    /// Name of this node.
    pub fn name(&self, doc: &StofDoc) -> Option<String> {
        unsafe {
            if let Some(doc) = DOCS.get(&doc.id()) {
                if let Some(node) = self.node_ref().node(&doc.graph) {
                    return Some(node.name.clone());
                }
            }
        }
        None
    }

    /// Parent of this node.
    pub fn parent(&self, doc: &StofDoc) -> Option<Self> {
        unsafe {
            if let Some(doc) = DOCS.get(&doc.id()) {
                if let Some(node) = self.node_ref().node(&doc.graph) {
                    if let Some(parent) = &node.parent {
                        return Some(Self::new(&parent.id));
                    }
                }
            }
        }
        None
    }

    /// Children of this node.
    pub fn children(&self, doc: &StofDoc) -> Vec<Self> {
        let mut children = Vec::new();
        unsafe {
            if let Some(doc) = DOCS.get(&doc.id()) {
                if let Some(node) = self.node_ref().node(&doc.graph) {
                    for child in &node.children {
                        children.push(Self::new(&child.id));
                    }
                }
            }
        }
        children
    }

    /// Data on this node.
    pub fn data(&self, doc: &StofDoc) -> Vec<StofData> {
        let mut data = Vec::new();
        unsafe {
            if let Some(doc) = DOCS.get(&doc.id()) {
                if let Some(node) = self.node_ref().node(&doc.graph) {
                    for dref in &node.data {
                        data.push(StofData::new(&dref.id));
                    }
                }
            }
        }
        data
    }

    /// Has data?
    #[wasm_bindgen(js_name = hasData)]
    pub fn has_data(&self, doc: &StofDoc, data: &StofData) -> bool {
        unsafe {
            if let Some(doc) = DOCS.get(&doc.id()) {
                if let Some(node) = self.node_ref().node(&doc.graph) {
                    return node.has_data(&data.data_ref());
                }
            }
        }
        false
    }

    /// Create some abstract data on this node.
    #[wasm_bindgen(js_name = createData)]
    pub fn create_data(&self, doc: &StofDoc, value: JsValue) -> Result<StofData, String> {
        StofData::construct(doc, self, value)
    }

    /// JSON value of this node as a whole.
    /// Can use this to store this value in an external place.
    pub fn to_json(&self, doc: &StofDoc) -> JsValue {
        unsafe {
            if let Some(doc) = DOCS.get(&doc.id()) {
                if let Some(node) = self.node_ref().node(&doc.graph) {
                    if let Ok(val) = serde_wasm_bindgen::to_value(node) {
                        return val;
                    }
                }
            }
        }
        JsValue::null()
    }

    /// Loat a JSON representation of a node into a document.
    /// Can use this to load nodes from an external place.
    pub fn from_json(doc: &StofDoc, json: JsValue) -> bool {
        if let Ok(node) = serde_wasm_bindgen::from_value::<SNode>(json) {
            unsafe {
                if let Some(doc) = DOCS.get_mut(&doc.id()) {
                    doc.graph.nodes.set(&node.id.clone(), node);
                    return true;
                }
            }
        }
        false
    }
}
