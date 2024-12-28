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

use std::sync::Arc;
use anyhow::{anyhow, Result};
use js_sys::Function;
use wasm_bindgen::prelude::*;
use crate::{Library, SDoc, SVal};
use super::StofDoc;


/// JS Doc Lib Func.
pub struct StofLibFunc {
    pub name: String,
    pub func: Function,
}


/// JS Stof Lib.
#[wasm_bindgen]
pub struct StofLib {
    scope: String,
}
impl Library for StofLib {
    fn scope(&self) -> String {
        self.scope.clone()
    }
    fn call(&self, _pid: &str, doc: &mut SDoc, name: &str, parameters: &mut Vec<SVal>) -> Result<SVal> {
        let context = JsValue::NULL;
        let mut func = None;
        if let Some(lib) = doc.libfuncs.read().unwrap().get(&self.scope) {
            if let Some(libfunc) = lib.get(name) {
                func = Some(libfunc.func.clone());
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
        Err(anyhow!("Failed to execute '{}' in library '{}'", name, &self.scope))
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

    /// Load a library into a document.
    pub fn load(doc: &mut StofDoc, lib: Self) {
        doc.doc_mut().load_lib(Arc::new(lib));
    }
}
