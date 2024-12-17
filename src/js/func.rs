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
use crate::{Data, SData, SFunc, SVal};
use super::{StofData, StofDoc, StofNode};


/// JS Stof Func.
#[wasm_bindgen]
pub struct StofFunc {
    func: SFunc,
}
impl StofFunc {
    /// From an existing func.
    pub fn from_func(func: SFunc) -> Self {
        Self { func }
    }

    /// Get func.
    pub fn func(&self) -> &SFunc {
        &self.func
    }

    /// Get mutable func.
    pub fn func_mut(&mut self) -> &mut SFunc {
        &mut self.func
    }
}
#[wasm_bindgen]
impl StofFunc {
    /// From a data reference.
    #[wasm_bindgen(js_name = fromData)]
    pub fn from_data(doc: &StofDoc, data: &StofData) -> Result<Self, String> {
        if let Ok(func) = SData::data::<SFunc>(&doc.doc().graph, &data.data_ref()) {
            return Ok(Self::from_func(func));
        }
        Err(format!("Could not construct a func from this data reference"))
    }

    /// Data reference.
    #[wasm_bindgen(js_name = data)]
    pub fn data_ref(&self) -> StofData {
        StofData::new(&self.func.id)
    }

    /// Name of this func.
    pub fn name(&self) -> String {
        self.func.name.clone()
    }

    /// Return type of this func.
    #[wasm_bindgen(js_name = returnType)]
    pub fn return_type(&self) -> String {
        self.func.rtype.type_of()
    }

    /// Parameters of this func.
    pub fn parameters(&self) -> Vec<StofFuncParam> {
        let mut params = Vec::new();
        for param in &self.func.params {
            let name = param.name.clone();
            let ptype = param.ptype.type_of();
            let has_default = param.default.is_some();
            params.push(StofFuncParam { name, ptype, has_default });
        }
        params
    }

    /// Attach this func to a node within the document.
    pub fn attach(&mut self, doc: &mut StofDoc, node: &StofNode) {
        self.func.attach(&node.node_ref(), &mut doc.doc_mut().graph);
    }

    /// Remove this func from the document everywhere.
    pub fn remove(&self, doc: &mut StofDoc) {
        self.func.remove(&mut doc.doc_mut().graph, None);
    }

    /// Remove this func from a specific node.
    /// If this node is the only one that references the func, the func will be removed from the doc.
    #[wasm_bindgen(js_name = removeFrom)]
    pub fn remove_from(&self, doc: &mut StofDoc, node: &StofNode) {
        self.func.remove(&mut doc.doc_mut().graph, Some(&node.node_ref()));
    }

    /// Get all funcs on a node.
    pub fn funcs(doc: &StofDoc, node: &StofNode) -> Vec<Self> {
        let funcs = SFunc::funcs(&doc.doc().graph, &node.node_ref());
        funcs.into_iter().map(|f| Self::from_func(f)).collect()
    }

    /// Get an adjacent func to this func in the document from a dot separated path.
    pub fn adjacent(&self, doc: &StofDoc, path: &str) -> Option<Self> {
        if let Some(func) = self.func.adjacent(&doc.doc().graph, path, '.') {
            return Some(Self::from_func(func));
        }
        None
    }

    /// Get a specific func from a dot separated path, starting at the root.
    #[wasm_bindgen(js_name = funcFromRoot)]
    pub fn func_from_root(doc: &StofDoc, path: &str) -> Option<Self> {
        if let Some(func) = SFunc::func(&doc.doc().graph, path, '.', None) {
            return Some(Self::from_func(func));
        }
        None
    }

    /// Get a specific func from a dot separated path, starting at a node.
    #[wasm_bindgen(js_name = func)]
    pub fn get_func(doc: &StofDoc, path: &str, node: &StofNode) -> Option<Self> {
        if let Some(func) = SFunc::func(&doc.doc().graph, path, '.', Some(&node.node_ref())) {
            return Some(Self::from_func(func));
        }
        None
    }

    /// Call this function.
    pub fn call(&self, doc: &mut StofDoc, params: Vec<JsValue>) -> Result<JsValue, String> {
        let params = params.into_iter().map(|p| SVal::from((p, doc.doc()))).collect();
        match self.func.call(doc.doc_mut(), params, true) {
            Ok(result) => {
                Ok(JsValue::from(result))
            },
            Err(e) => {
                Err(format!("Error calling function: {:?}", e))
            }
        }
    }
}


/// Stof Func param interface.
#[wasm_bindgen]
pub struct StofFuncParam {
    name: String,
    ptype: String,
    has_default: bool,
}
#[wasm_bindgen]
impl StofFuncParam {
    /// Name.
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Type.
    #[wasm_bindgen(js_name = type)]
    pub fn ptype(&self) -> String {
        self.ptype.clone()
    }

    /// Has a default value?
    #[wasm_bindgen(js_name = hasDefault)]
    pub fn has_default(&self) -> bool {
        self.has_default
    }
}
