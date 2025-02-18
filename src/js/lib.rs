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

use js_sys::Function;
use wasm_bindgen::prelude::*;
use crate::{lang::SError, Library, SDoc, SVal};
use super::{StofDoc, DOC_LIBS};


/// JS Doc Lib Func.
pub struct StofLibFunc {
    pub name: String,
    pub func: Function,
}
unsafe impl Send for StofLibFunc {}
unsafe impl Sync for StofLibFunc {}


/// JS Stof Lib.
#[wasm_bindgen]
pub struct StofLib {
    scope: String,
}
impl Library for StofLib {
    fn scope(&self) -> String {
        self.scope.clone()
    }
    fn call(&self, pid: &str, doc: &mut SDoc, name: &str, parameters: &mut Vec<SVal>) -> Result<SVal, SError> {
        let refdoc = StofDoc::from_id(&doc.graph.id);
        let context = JsValue::from(refdoc);
        
        let mut func = None;
        let doc_libs = DOC_LIBS.read().unwrap();
        if let Some(libs) = doc_libs.get(&doc.graph.id) {
            if let Some(lib) = libs.get(&self.scope) {
                if let Some(libfunc) = lib.get(name) {
                    func = Some(libfunc.func.clone());
                }
            }
        }

        let mut res = None;
        if let Some(func) = func {
            let params: Vec<JsValue> = parameters.iter().map(|x| JsValue::from(x.clone())).collect();
            if params.len() == 0 {
                if let Ok(jsval) = func.call0(&context) {
                    res = Some(jsval);
                }
            } else if params.len() == 1 {
                if let Ok(jsval) = func.call1(&context, &params[0]) {
                    res = Some(jsval);
                }
            } else if params.len() == 2 {
                if let Ok(jsval) = func.call2(&context, &params[0], &params[1]) {
                    res = Some(jsval);
                }
            } else if params.len() == 3 {
                if let Ok(jsval) = func.call3(&context, &params[0], &params[1], &params[2]) {
                    res = Some(jsval);
                }
            }
        }
        if let Some(res) = res {
            return Ok(SVal::from((res, doc)));
        }
        Err(SError::custom(pid, &doc, "WasmStofLibError", &format!("failed to execute '{}' in library '{}'", name, &self.scope)))
    }
}
#[wasm_bindgen]
impl StofLib {
    /// Create a new StofLib.
    #[wasm_bindgen(constructor)]
    pub fn new(scope: &str) -> Self {
        Self { scope: scope.to_owned() }
    }

    /// Name of this library.
    /// This is how it will be referenced from within Stof.
    pub fn name(&self) -> String {
        self.scope.clone()
    }
}
