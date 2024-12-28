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

use js_sys::{Array, Uint8Array};
use wasm_bindgen::JsValue;
use crate::{SDataRef, SDoc, SNodeRef, SNum, SVal};


/// Implement into JsValue for SVal
impl From<SVal> for JsValue {
    fn from(value: SVal) -> Self {
        match value {
            SVal::Null => Self::null(),
            SVal::Blob(blob) => {
                let array = Uint8Array::from(blob.as_ref());
                Self::from(array)
            },
            SVal::Bool(val) => Self::from_bool(val),
            SVal::String(val) => Self::from_str(&val),
            SVal::Number(num) => {
                match num {
                    SNum::I64(val) => Self::from(val),
                    SNum::F64(val) => Self::from_f64(val),
                    SNum::Units(val, _) => Self::from_f64(val),
                }
            },
            SVal::FnPtr(dref) => Self::from_str(&dref.id), // Gets turned into an ID for a StofData!
            SVal::Object(nref) => Self::from_str(&nref.id), // Gets turned into an ID for a StofNode!
            SVal::Array(vals) => {
                let array = Array::new();
                for val in vals {
                    array.push(&JsValue::from(val));
                }
                Self::from(array)
            },
            SVal::Tuple(vals) => {
                let array = Array::new();
                for val in vals {
                    array.push(&JsValue::from(val));
                }
                Self::from(array)
            },
            SVal::Void => Self::undefined(),
        }
    }
}
impl From<(JsValue, &mut SDoc)> for SVal {
    fn from((value, doc): (JsValue, &mut SDoc)) -> Self {
        if value.is_null() || value.is_undefined() { return Self::Null; }
        if let Some(val) = value.as_bool() {
            return Self::Bool(val);
        }
        if let Some(val) = value.as_f64() {
            return Self::Number(SNum::F64(val));
        }
        if let Some(val) = value.as_string() {
            // Check if this value is a node reference
            let nref = SNodeRef::from(&val);
            if nref.exists(&doc.graph) {
                return Self::Object(nref);
            }
            // Check if this value is a data (function ptr) reference
            // This works because FnPtr is the only SVal to be a SDataRef right now...
            // In the future, we might have to introduce some id prefix for this.
            let dref = SDataRef::from(&val);
            if dref.exists(&doc.graph) {
                return Self::FnPtr(dref);
            }
            return Self::String(val);
        }
        if value.is_array() {
            let mut res = Vec::new();
            let array = Array::from(&value);
            for val in array {
                res.push(Self::from((val, &mut *doc)));
            }
            return Self::Array(res);
        }

        // Finally, try casting to a blob type
        let intarray = Uint8Array::from(value);
        Self::Blob(intarray.to_vec())
    }
}
impl From<(JsValue, &SDoc)> for SVal {
    fn from((value, doc): (JsValue, &SDoc)) -> Self {
        if value.is_null() || value.is_undefined() { return Self::Null; }
        if let Some(val) = value.as_bool() {
            return Self::Bool(val);
        }
        if let Some(val) = value.as_f64() {
            return Self::Number(SNum::F64(val));
        }
        if let Some(val) = value.as_string() {
            // Check if this value is a node reference
            let nref = SNodeRef::from(&val);
            if nref.exists(&doc.graph) {
                return Self::Object(nref);
            }
            // Check if this value is a data (function ptr) reference
            // This works because FnPtr is the only SVal to be a SDataRef right now...
            // In the future, we might have to introduce some id prefix for this.
            let dref = SDataRef::from(&val);
            if dref.exists(&doc.graph) {
                return Self::FnPtr(dref);
            }
            return Self::String(val);
        }
        if value.is_array() {
            let mut res = Vec::new();
            let array = Array::from(&value);
            for val in array {
                res.push(Self::from((val, doc)));
            }
            return Self::Array(res);
        }

        // Finally, try casting to a blob type
        let intarray = Uint8Array::from(value);
        Self::Blob(intarray.to_vec())
    }
}
