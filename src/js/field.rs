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
use crate::{SData, SDataRef, SField, SVal};
use super::{StofDoc, StofNode, DOCS};


/// Stof Field.
#[wasm_bindgen]
pub struct StofField {
    id: String,
}
impl StofField {
    /// Return the SDataRef for this field.
    pub fn data_ref(&self) -> SDataRef {
        SDataRef::from(&self.id)
    }
}
#[wasm_bindgen]
impl StofField {
    /// Field constructor with a JS Value.
    /// Creates a new field with this value on the node.
    #[wasm_bindgen(constructor)]
    pub fn construct(doc: &StofDoc, node: &StofNode, name: &str, value: JsValue) -> Result<Self, String> {
        unsafe {
            if let Some(doc) = DOCS.get_mut(&doc.id()) {
                let sval = SVal::from((value, &*doc));
                let field = SField::new(&name, sval);
                if let Some(dref) = SData::insert_new(&mut doc.graph, node.node_ref(), Box::new(field)) {
                    return Ok(Self {
                        id: dref.id,
                    });
                }
            }
        }
        Err(format!("Could not create Stof field"))
    }

    /// Field from a dot separated path.
    pub fn field(doc: &StofDoc, path: &str) -> Option<Self> {
        unsafe {
            if let Some(doc) = DOCS.get(&doc.id()) {
                if let Some(dref) = SField::field_ref(&doc.graph, path, '.', None) {
                    return Some(Self {
                        id: dref.id,
                    });
                }
            }
        }
        None
    }

    /// Field from a dot separated path and a starting node.
    #[wasm_bindgen(js_name = fieldFrom)]
    pub fn field_from(doc: &StofDoc, path: &str, start: &StofNode) -> Option<Self> {
        unsafe {
            if let Some(doc) = DOCS.get(&doc.id()) {
                if let Some(dref) = SField::field_ref(&doc.graph, path, '.', Some(&start.node_ref())) {
                    return Some(Self {
                        id: dref.id,
                    });
                }
            }
        }
        None
    }

    /// Field value getter.
    pub fn value(&self, doc: &StofDoc) -> JsValue {
        unsafe {
            if let Some(doc) = DOCS.get(&doc.id()) {
                if let Some(field) = SData::get::<SField>(&doc.graph, &self.id) {
                    return JsValue::from(field.value.clone());
                }
            }
        }
        JsValue::undefined()
    }

    /// Field value setter.
    pub fn set(&self, doc: &StofDoc, value: JsValue) -> bool {
        unsafe {
            if let Some(doc) = DOCS.get_mut(&doc.id()) {
                let sval = SVal::from((value, &mut *doc));
                if let Some(field) = SData::get_mut::<SField>(&mut doc.graph, &self.id) {
                    field.value = sval;
                    return true;
                }
            }
        }
        false
    }
}
