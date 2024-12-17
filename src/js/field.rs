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
use crate::{Data, IntoDataRef, SData, SField, SVal};
use super::{StofData, StofDoc, StofNode};


/// JS Stof Field.
#[wasm_bindgen]
pub struct StofField {
    field: SField,
}
impl StofField {
    /// From an existing field.
    pub fn from_field(field: SField) -> Self {
        Self { field }
    }

    /// Get field.
    pub fn field(&self) -> &SField {
        &self.field
    }

    /// Get mutable field.
    pub fn field_mut(&mut self) -> &mut SField {
        &mut self.field
    }
}
#[wasm_bindgen]
impl StofField {
    /// New field.
    /// Does not insert into the document, but needs the document for JsValue -> SVal.
    #[wasm_bindgen(constructor)]
    pub fn new(doc: &StofDoc, name: &str, value: JsValue) -> Self {
        let value = SVal::from((value, doc.doc()));
        Self::from_field(SField::new(name, value))
    }

    /// From a data reference.
    #[wasm_bindgen(js_name = fromData)]
    pub fn from_data(doc: &StofDoc, data: &StofData) -> Result<Self, String> {
        if let Ok(field) = SData::data::<SField>(&doc.doc().graph, &data.data_ref()) {
            return Ok(Self::from_field(field));
        }
        Err(format!("Could not construct a field from this data reference"))
    }

    /// Data reference.
    #[wasm_bindgen(js_name = data)]
    pub fn data_ref(&self) -> StofData {
        StofData::new(&self.field.id)
    }

    /// Name of this field.
    pub fn name(&self) -> String {
        self.field.name.clone()
    }

    /// Value of this field.
    pub fn value(&self) -> JsValue {
        JsValue::from(self.field.value.clone())
    }

    /// Set the value of this field.
    /// If this field exists within the document, it will set the value in the document as well.
    #[wasm_bindgen(js_name = setValue)]
    pub fn set_value(&mut self, doc: &mut StofDoc, value: JsValue) {
        let value = SVal::from((value, doc.doc()));
        self.field.value = value;
        if self.field.data_ref().exists(&doc.doc().graph) {
            self.field.set(&mut doc.doc_mut().graph);
        }
    }

    /// Attach this field to a node within the document.
    pub fn attach(&mut self, doc: &mut StofDoc, node: &StofNode) {
        self.field.attach(&node.node_ref(), &mut doc.doc_mut().graph);
    }

    /// Remove this field from the document everywhere.
    pub fn remove(&self, doc: &mut StofDoc) {
        self.field.remove(&mut doc.doc_mut().graph, None);
    }

    /// Remove this field from a specific node.
    /// If this node is the only one that references the field, the field will be removed from the doc.
    #[wasm_bindgen(js_name = removeFrom)]
    pub fn remove_from(&self, doc: &mut StofDoc, node: &StofNode) {
        self.field.remove(&mut doc.doc_mut().graph, Some(&node.node_ref()));
    }

    /// Get all fields on a node.
    pub fn fields(doc: &StofDoc, node: &StofNode) -> Vec<Self> {
        let fields = SField::fields(&doc.doc().graph, &node.node_ref());
        fields.into_iter().map(|f| Self::from_field(f)).collect()
    }

    /// Get an adjacent field to this field in the document from a dot separated path.
    pub fn adjacent(&self, doc: &StofDoc, path: &str) -> Option<Self> {
        if let Some(field) = self.field.adjacent(&doc.doc().graph, path, '.') {
            return Some(Self::from_field(field));
        }
        None
    }

    /// Get a specific field from a dot separated path, starting at the root.
    #[wasm_bindgen(js_name = fieldFromRoot)]
    pub fn field_from_root(doc: &StofDoc, path: &str) -> Option<Self> {
        if let Some(field) = SField::field(&doc.doc().graph, path, '.', None) {
            return Some(Self::from_field(field));
        }
        None
    }

    /// Get a specific field from a dot separated path, starting at a node.
    #[wasm_bindgen(js_name = field)]
    pub fn get_field(doc: &StofDoc, path: &str, node: &StofNode) -> Option<Self> {
        if let Some(field) = SField::field(&doc.doc().graph, path, '.', Some(&node.node_ref())) {
            return Some(Self::from_field(field));
        }
        None
    }
}
