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
use super::{StofDoc, StofNode};


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

    /// Get the data.
    pub fn data<'a>(&self, doc: &'a StofDoc) -> Option<&'a SData> {
        self.data_ref().data(&doc.doc().graph)
    }

    /// Get a mutable reference to the data.
    pub fn data_mut<'a>(&self, doc: &'a mut StofDoc) -> Option<&'a mut SData> {
        self.data_ref().data_mut(&mut doc.doc_mut().graph)
    }
}
#[wasm_bindgen]
impl StofData {
    /// JSON data constructor.
    /// Will turn value into serde_json::Value, then create an SData, putting it into the document at 'node'.
    /// Only works if node is already in the document.
    #[wasm_bindgen(constructor)]
    pub fn construct(doc: &mut StofDoc, node: &StofNode, value: JsValue) -> Result<Self, String> {
        if let Ok(value) = serde_wasm_bindgen::from_value::<serde_json::Value>(value) {
            if let Ok(json) = serde_json::to_string(&value) {
                let data = SData::new(json);
                if let Some(dref) = doc.doc_mut().graph.put_data(&node.node_ref(), data) {
                    return Ok(Self::new(&dref.id));
                }
            }
        }
        Err(format!("Could not create the Stof data"))
    }

    /// Construct a new StofData with an ID and a value.
    #[wasm_bindgen(js_name = newWithId)]
    pub fn construct_with_id(doc: &mut StofDoc, node: &StofNode, id: &str, value: JsValue) -> Result<Self, String> {
        if let Ok(value) = serde_wasm_bindgen::from_value::<serde_json::Value>(value) {
            if let Ok(json) = serde_json::to_string(&value) {
                let data = SData::new_id(id, json);
                if let Some(dref) = doc.doc_mut().graph.put_data(&node.node_ref(), data) {
                    return Ok(Self::new(&dref.id));
                }
            }
        }
        Err(format!("Could not create the Stof data"))
    }

    /// Remove this data from every place within the document.
    pub fn remove(&self, doc: &mut StofDoc) -> bool {
        doc.doc_mut().graph.remove_data(&self.data_ref(), None)
    }

    /// Remove this data from a specific node in the document.
    #[wasm_bindgen(js_name = removeFrom)]
    pub fn remove_from(&self, doc: &mut StofDoc, node: &StofNode) -> bool {
        doc.doc_mut().graph.remove_data(&self.data_ref(), Some(&node.node_ref()))
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
    pub fn invalidate(&self, doc: &mut StofDoc, symbol: &str) -> bool {
        if let Some(data) = self.data_mut(doc) {
            data.invalidate(symbol);
            return true;
        }
        false
    }

    /// Invalidate value on this data.
    #[wasm_bindgen(js_name = invalidateValue)]
    pub fn invalidate_value(&self, doc: &mut StofDoc) -> bool {
        if let Some(data) = self.data_mut(doc) {
            data.invalidate_val();
            return true;
        }
        false
    }

    /// Dirty?
    pub fn dirty(&self, doc: &StofDoc, symbol: &str) -> bool {
        if let Some(data) = self.data(doc) {
            return data.dirty(symbol);
        }
        false
    }

    /// Any dirty symbols?
    #[wasm_bindgen(js_name = anyDirty)]
    pub fn any_dirty(&self, doc: &StofDoc) -> bool {
        if let Some(data) = self.data(doc) {
            return data.has_dirty();
        }
        false
    }

    /// Validate this data with the symbol.
    pub fn validate(&self, doc: &mut StofDoc, symbol: &str) -> bool {
        if let Some(data) = self.data_mut(doc) {
            return data.validate(symbol);
        }
        false
    }

    /// Validate value for this data.
    pub fn validate_all(&self, doc: &mut StofDoc) -> bool {
        if let Some(data) = self.data_mut(doc) {
            return data.validate_val();
        }
        false
    }

    /// Exists?
    pub fn exists(&self, doc: &StofDoc) -> bool {
        self.data_ref().exists(&doc.doc().graph)
    }

    /// Nodes that contain this data.
    /// Data can exist on several nodes at once.
    pub fn nodes(&self, doc: &StofDoc) -> Vec<StofNode> {
        let mut nodes = Vec::new();
        if let Some(data) = self.data(doc) {
            for nref in &data.nodes {
                nodes.push(StofNode::new(&nref.id));
            }
        }
        nodes
    }

    /// Try getting the JSON value of this data.
    /// Will only work if the value of this data can be deserialized into serde_json::Value.
    #[wasm_bindgen(js_name = getValue)]
    pub fn get_value(&self, doc: &StofDoc) -> Result<JsValue, String> {
        if let Some(data) = self.data(doc) {
            if let Ok(json) = data.get_value::<String>() {
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(&json) {
                    if let Ok(jsval) = serde_wasm_bindgen::to_value(&value) {
                        return Ok(jsval);
                    }
                }
            }
        }
        Err(format!("Could not get a JSON value for this data"))
    }

    /// Try setting a JSON value for this data.
    #[wasm_bindgen(js_name = setValue)]
    pub fn set_value(&self, doc: &mut StofDoc, value: JsValue) -> bool {
        if let Some(data) = self.data_mut(doc) {
            if let Ok(value) = serde_wasm_bindgen::from_value::<serde_json::Value>(value) {
                if let Ok(json) = serde_json::to_string(&value) {
                    data.set_value(json);
                    return true;
                }
            }
        }
        false
    }

    /// JSON value of this data as a whole.
    /// Can use this to store this value in an external place.
    pub fn to_json(&self, doc: &StofDoc) -> JsValue {
        if let Some(data) = self.data(doc) {
            if let Ok(val) = serde_wasm_bindgen::to_value(data) {
                return val;
            }
        }
        JsValue::null()
    }

    /// Loat a JSON representation of a data into a document.
    /// Can use this to load data from an external place.
    pub fn from_json(doc: &mut StofDoc, json: JsValue) -> bool {
        if let Ok(data) = serde_wasm_bindgen::from_value::<SData>(json) {
            doc.doc_mut().graph.data.set(&data.id.clone(), data);
            return true;
        }
        false
    }
}
