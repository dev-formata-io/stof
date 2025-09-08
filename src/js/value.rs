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

use imbl::{OrdMap, OrdSet, Vector};
use js_sys::{Array, BigInt, Map, Set, Uint8Array};
use wasm_bindgen::JsValue;
use crate::{js::Stof, model::{Func, Graph, SId}, runtime::{Num, Type, Val, ValRef}};


impl From<(JsValue, &Stof)> for Val {
    fn from(value: (JsValue, &Stof)) -> Self {
        to_stof_value(value.0, value.1)
    }
}
impl From<(JsValue, &mut Stof)> for Val {
    fn from(value: (JsValue, &mut Stof)) -> Self {
        to_stof_value(value.0, &value.1)
    }
}
pub fn to_stof_value(js: JsValue, doc: &Stof) -> Val {
    to_graph_value(js, &doc.graph)
}
pub fn to_graph_value(js: JsValue, doc: &Graph) -> Val {
    if js.is_null() { return Val::Null; }
    if js.is_undefined() { return Val::Void; }
    if let Some(val) = js.as_bool() {
        return Val::Bool(val);
    }
    if let Some(val) = js.as_f64() {
        return Val::Num(Num::Float(val));
    }
    if let Some(val) = js.as_string() {
        let sid = SId::from(&val);
        if sid.node_exists(&doc) {
            return Val::Obj(sid);
        }
        if sid.data_exists(&doc) {
            if sid.type_of::<Func>(&doc) {
                return Val::Fn(sid);
            }
            return Val::Data(sid);
        }
        if val.contains("_pr:_ms") {
            let id_type = val.split("_pr:_ms").collect::<Vec<_>>();
            if id_type.len() == 2 {
                let pid = SId::from(id_type[0]);
                let ty = Type::from(id_type[1]);
                return Val::Promise(pid, ty);
            }
        }
        return Val::Str(val.into());
    }
    if js.is_array() {
        let mut res = Vector::default();
        let array = Array::from(&js);
        for val in array {
            res.push_back(ValRef::new(to_graph_value(val, doc)));
        }
        return Val::List(res);
    }

    #[allow(irrefutable_let_patterns)]
    if js.is_bigint() {
        let bigint = BigInt::from(js);
        if let Some(val) = bigint.as_f64() {
            return Val::Num(Num::Float(val));
        }
        Val::Null
    }
    else if let Ok(set) = Set::try_from(js.clone()) {
        let mut stof_set = OrdSet::default();
        for js_val in set.values() {
            if let Ok(js_val) = js_val {
                stof_set.insert(ValRef::new(to_graph_value(js_val, doc)));
            }
        }
        Val::Set(stof_set)
    } else if let Ok(map) = Map::try_from(js.clone()) {
        let mut stof_map = OrdMap::default();
        for js_pair in map.entries() {
            if let Ok(js_val) = js_pair {
                let arr = Array::from(&js_val);
                let key = arr.get(0);
                let val = arr.get(1);

                stof_map.insert(
                    ValRef::new(to_graph_value(key, doc)),
                    ValRef::new(to_graph_value(val, doc))
                );
            }
        }
        Val::Map(stof_map)
    } else {
        // cast to blob type
        let intarray = Uint8Array::from(js);
        Val::Blob(intarray.to_vec())
    }
}

#[allow(unused)]
pub fn to_raw_value(js: JsValue) -> Val {
    if js.is_null() { return Val::Null; }
    if js.is_undefined() { return Val::Void; }
    if let Some(val) = js.as_bool() {
        return Val::Bool(val);
    }
    if let Some(val) = js.as_f64() {
        return Val::Num(Num::Float(val));
    }
    if let Some(val) = js.as_string() {
        return Val::Str(val.into());
    }
    if js.is_array() {
        let mut res = Vector::default();
        let array = Array::from(&js);
        for val in array {
            res.push_back(ValRef::new(to_raw_value(val)));
        }
        return Val::List(res);
    }

    #[allow(irrefutable_let_patterns)]
    if js.is_bigint() {
        let bigint = BigInt::from(js);
        if let Some(val) = bigint.as_f64() {
            return Val::Num(Num::Float(val));
        }
        Val::Null
    }
    else if let Ok(set) = Set::try_from(js.clone()) {
        let mut stof_set = OrdSet::default();
        for js_val in set.values() {
            if let Ok(js_val) = js_val {
                stof_set.insert(ValRef::new(to_raw_value(js_val)));
            }
        }
        Val::Set(stof_set)
    } else if let Ok(map) = Map::try_from(js.clone()) {
        let mut stof_map = OrdMap::default();
        for js_pair in map.entries() {
            if let Ok(js_val) = js_pair {
                let arr = Array::from(&js_val);
                let key = arr.get(0);
                let val = arr.get(1);

                stof_map.insert(
                    ValRef::new(to_raw_value(key)),
                    ValRef::new(to_raw_value(val))
                );
            }
        }
        Val::Map(stof_map)
    } else {
        // cast to blob type
        let intarray = Uint8Array::from(js);
        Val::Blob(intarray.to_vec())
    }
}


impl From<Val> for JsValue {
    fn from(value: Val) -> Self {
        match value {
            Val::Void => Self::undefined(),
            Val::Null => Self::null(),
            Val::Blob(blob) => {
                let array = Uint8Array::from(blob.as_ref());
                Self::from(array)
            },
            Val::Bool(val) => Self::from_bool(val),
            Val::Str(val) => Self::from_str(&val),
            Val::Ver(..) => Self::from_str(&value.to_string()),
            Val::Num(num) => {
                match num {
                    Num::Int(val) => Self::from(val as i32),
                    Num::Float(val) => Self::from(val),
                    Num::Units(val, _) => Self::from(val),
                }
            },
            Val::Fn(ptr) => Self::from_str(ptr.as_ref()),
            Val::Data(ptr) => Self::from_str(ptr.as_ref()),
            Val::Obj(nref) => Self::from_str(nref.as_ref()),
            Val::List(vals) => {
                let array = Array::new();
                for val in vals {
                    array.push(&Self::from(val.read().clone()));
                }
                Self::from(array)
            },
            Val::Tup(vals) => {
                let array = Array::new();
                for val in vals {
                    array.push(&Self::from(val.read().clone()));
                }
                Self::from(array)
            },
            Val::Set(set) => {
                let js_set = Set::new(&Self::NULL);
                for val in set {
                    js_set.add(&Self::from(val.read().clone()));
                }
                Self::from(js_set)
            },
            Val::Map(map) => {
                let js_map = Map::new();
                for (k, v) in map {
                    js_map.set(&Self::from(k.read().clone()), &Self::from(v.read().clone()));
                }
                Self::from(js_map)
            },
            Val::Promise(id, pt) => {
                Self::from_str(&format!("{}_pr:_ms{}", id.as_ref(), pt.type_of()))
            },
        }
    }
}
