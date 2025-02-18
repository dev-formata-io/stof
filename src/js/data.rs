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
use crate::{SData, SDataRef, Store};
use super::{StofDoc, StofNode, DOCS};


/// Stof Data.
#[wasm_bindgen]
pub struct StofData {
    id: String,
}
impl StofData {
    /// Return the SDataRef for this data.
    pub fn data_ref(&self) -> SDataRef {
        SDataRef::from(&self.id)
    }
}
#[wasm_bindgen]
impl StofData {
    /// JSON data constructor.
    /// Will turn value into serde_json::Value, then create an SData, putting it into the document at 'node'.
    /// Only works if node is already in the document.
    #[wasm_bindgen(constructor)]
    pub fn construct(doc: &StofDoc, node: &StofNode, value: JsValue) -> Result<Self, String> {
        if let Ok(value) = serde_wasm_bindgen::from_value::<serde_json::Value>(value) {
            if let Ok(json) = serde_json::to_string(&value) {
                let data = SData::new(Box::new(json));
                unsafe {
                    if let Some(doc) = DOCS.get_mut(&doc.id()) {
                        if let Some(dref) = doc.graph.put_data(&node.node_ref(), data) {
                            return Ok(Self::new(&dref.id));
                        }
                    }
                }
            }
        }
        Err(format!("Could not create the Stof data"))
    }

    /// Construct a new StofData with an ID and a value.
    #[wasm_bindgen(js_name = newWithId)]
    pub fn construct_with_id(doc: &StofDoc, node: &StofNode, id: &str, value: JsValue) -> Result<Self, String> {
        if let Ok(value) = serde_wasm_bindgen::from_value::<serde_json::Value>(value) {
            if let Ok(json) = serde_json::to_string(&value) {
                let data = SData::new_id(id, Box::new(json));
                unsafe {
                    if let Some(doc) = DOCS.get_mut(&doc.id()) {
                        if let Some(dref) = doc.graph.put_data(&node.node_ref(), data) {
                            return Ok(Self::new(&dref.id));
                        }
                    }
                }
            }
        }
        Err(format!("Could not create the Stof data"))
    }

    /// Remove this data from every place within the document.
    pub fn remove(&self, doc: &StofDoc) -> bool {
        unsafe {
            if let Some(doc) = DOCS.get_mut(&doc.id()) {
                return doc.graph.remove_data(&self.data_ref(), None);
            }
        }
        false
    }

    /// Remove this data from a specific node in the document.
    #[wasm_bindgen(js_name = removeFrom)]
    pub fn remove_from(&self, doc: &StofDoc, node: &StofNode) -> bool {
        unsafe {
            if let Some(doc) = DOCS.get_mut(&doc.id()) {
                return doc.graph.remove_data(&self.data_ref(), Some(&node.node_ref()));
            }
        }
        false
    }

    /// ID constructor.
    pub fn new(id: &str) -> Self {
        Self { id: id.to_owned() }
    }

    /// Get the ID of this reference.
    pub fn id(&self) -> String {
        self.id.clone()
    }

    /// Invalidate this data with a symbol.
    pub fn invalidate(&self, doc: &StofDoc, symbol: &str) -> bool {
        unsafe {
            if let Some(doc) = DOCS.get_mut(&doc.id()) {
                if let Some(data) = self.data_ref().data_mut(&mut doc.graph) {
                    data.invalidate(symbol);
                    return true;
                }
            }
        }
        false
    }

    /// Invalidate value on this data.
    #[wasm_bindgen(js_name = invalidateValue)]
    pub fn invalidate_value(&self, doc: &StofDoc) -> bool {
        unsafe {
            if let Some(doc) = DOCS.get_mut(&doc.id()) {
                if let Some(data) = self.data_ref().data_mut(&mut doc.graph) {
                    data.invalidate_val();
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
                if let Some(data) = self.data_ref().data(&doc.graph) {
                    return data.dirty(symbol);
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
                if let Some(data) = self.data_ref().data(&doc.graph) {
                    return data.has_dirty();
                }
            }
        }
        false
    }

    /// Validate this data with the symbol.
    pub fn validate(&self, doc: &StofDoc, symbol: &str) -> bool {
        unsafe {
            if let Some(doc) = DOCS.get_mut(&doc.id()) {
                if let Some(data) = self.data_ref().data_mut(&mut doc.graph) {
                    return data.validate(symbol);
                }
            }
        }
        false
    }

    /// Validate value for this data.
    #[wasm_bindgen(js_name = validateValue)]
    pub fn validate_value(&self, doc: &StofDoc) -> bool {
        unsafe {
            if let Some(doc) = DOCS.get_mut(&doc.id()) {
                if let Some(data) = self.data_ref().data_mut(&mut doc.graph) {
                    return data.validate_val();
                }
            }
        }
        false
    }

    /// Exists?
    pub fn exists(&self, doc: &StofDoc) -> bool {
        unsafe {
            if let Some(doc) = DOCS.get(&doc.id()) {
                return self.data_ref().exists(&doc.graph);
            }
        }
        false
    }

    /// Nodes that contain this data.
    /// Data can exist on several nodes at once.
    pub fn nodes(&self, doc: &StofDoc) -> Vec<StofNode> {
        let mut nodes = Vec::new();
        unsafe {
            if let Some(doc) = DOCS.get(&doc.id()) {
                if let Some(data) = self.data_ref().data(&doc.graph) {
                    for nref in &data.nodes {
                        nodes.push(StofNode::new(&nref.id));
                    }
                }
            }
        }
        nodes
    }

    /// Try getting the JSON value of this data.
    /// Will only work if the value of this data can be deserialized into serde_json::Value.
    #[wasm_bindgen(js_name = getValue)]
    pub fn get_value(&self, doc: &StofDoc) -> Result<JsValue, String> {
        unsafe {
            if let Some(doc) = DOCS.get(&doc.id()) {
                if let Some(data) = self.data_ref().data(&doc.graph) {
                    if let Some(json) = data.get_data::<String>() {
                        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&json) {
                            if let Ok(jsval) = serde_wasm_bindgen::to_value(&value) {
                                return Ok(jsval);
                            }
                        }
                    }
                }
            }
        }
        Err(format!("Could not get a JSON value for this data"))
    }

    /// Try setting a JSON value for this data.
    #[wasm_bindgen(js_name = setValue)]
    pub fn set_value(&self, doc: &StofDoc, value: JsValue) -> bool {
        unsafe {
            if let Some(doc) = DOCS.get_mut(&doc.id()) {
                if let Some(data) = self.data_ref().data_mut(&mut doc.graph) {
                    if let Ok(value) = serde_wasm_bindgen::from_value::<serde_json::Value>(value) {
                        if let Ok(json) = serde_json::to_string(&value) {
                            data.set_data(Box::new(json));
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// JSON value of this data as a whole.
    /// Can use this to store this value in an external place.
    pub fn to_json(&self, doc: &StofDoc) -> JsValue {
        unsafe {
            if let Some(doc) = DOCS.get(&doc.id()) {
                if let Some(data) = self.data_ref().data(&doc.graph) {
                    if let Ok(val) = serde_wasm_bindgen::to_value(data) {
                        return val;
                    }
                }
            }
        }
        JsValue::null()
    }

    /// Loat a JSON representation of a data into a document.
    /// Can use this to load data from an external place.
    pub fn from_json(doc: &StofDoc, json: JsValue) -> bool {
        if let Ok(data) = serde_wasm_bindgen::from_value::<SData>(json) {
            unsafe {
                if let Some(doc) = DOCS.get_mut(&doc.id()) {
                    doc.graph.data.set(&data.id.clone(), data);
                    return true;
                }
            }
        }
        false
    }
}
