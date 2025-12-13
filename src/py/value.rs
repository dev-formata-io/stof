//
// Copyright 2025 Formata, Inc. All rights reserved.
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

use bytes::Bytes;
use imbl::{OrdMap, OrdSet, Vector};
use pyo3::{Bound, PyAny, Python, types::{PyAnyMethods, PyBool, PyBoolMethods, PyByteArray, PyByteArrayMethods, PyBytes, PyBytesMethods, PyComplex, PyComplexMethods, PyDict, PyDictMethods, PyFloat, PyFloatMethods, PyFrozenSet, PyInt, PyList, PyNone, PyRange, PySet, PyString, PyStringMethods, PyTuple}};
use crate::{model::{Func, Graph, SId}, runtime::{Num, Type, Val, ValRef}};


pub fn py_any_to_val(py: &Bound<'_, PyAny>, graph: &Graph) -> Val {
    // None
    if let Ok(_res) = py.cast::<PyNone>() {
        return Val::Null;
    }

    // Text
    // PyString is used to map Stof Obj & Data pointers too
    if let Ok(res) = py.cast::<PyString>() {
        if let Ok(value) = res.to_str() {
            if value.len() < 100 { // large strings go right into an ArcStr and skip ptr checks
                let sid = SId::from(value);
                if sid.node_exists(graph) {
                    return Val::Obj(sid);
                }
                if sid.data_exists(graph) {
                    if sid.type_of::<Func>(graph) {
                        return Val::Fn(sid);
                    }
                    return Val::Data(sid);
                }
                if value.contains("_pr:_ms") {
                    let id_type = value.split("_pr:_ms").collect::<Vec<_>>();
                    if id_type.len() == 2 {
                        let pid = SId::from(id_type[0]);
                        let ty = Type::from(id_type[1]);
                        return Val::Promise(pid, ty);
                    }
                }
            }
            return Val::Str(value.into());
        }
    }

    // Bool
    if let Ok(res) = py.cast::<PyBool>() {
        if res.is_true() { return Val::Bool(true); }
        return Val::Bool(false);
    }

    // Numeric
    if let Ok(res) = py.cast::<PyComplex>() {
        // just take real (TODO: add complex numbers to Stof?)
        let real = res.real();
        return Val::Num(Num::Float(real));
    }
    if let Ok(res) = py.cast::<PyFloat>() {
        let v = res.value();
        return Val::Num(Num::Float(v));
    }
    if let Ok(res) = py.cast::<PyInt>() {
        if let Ok(v) = res.extract::<i64>() {
            return Val::Num(Num::Int(v));
        }
    }

    // Sequence
    if let Ok(res) = py.cast::<PyList>() {
        let mut list = Vector::default();
        for val in res {
            list.push_back(ValRef::new(py_any_to_val(&val, graph)));
        }
        return Val::List(list);
    }
    if let Ok(res) = py.cast::<PyTuple>() {
        let mut tup = Vector::default();
        for val in res {
            tup.push_back(ValRef::new(py_any_to_val(&val, graph)));
        }
        return Val::Tup(tup);
    }
    if let Ok(res) = py.cast::<PyRange>() {
        if let Ok(iter) = res.try_iter() {
            let mut list = Vector::default();
            for val in iter {
                match val {
                    Ok(val) => {
                        list.push_back(ValRef::new(py_any_to_val(&val, graph)));
                    },
                    Err(_) => {},
                }
            }
            return Val::List(list);
        }
    }

    // Mapping
    if let Ok(res) = py.cast::<PyDict>() {
        let mut stof_map = OrdMap::default();
        for (key, val) in res {
            stof_map.insert(
                ValRef::new(py_any_to_val(&key, graph)),
                ValRef::new(py_any_to_val(&val, graph))
            );
        }
        return Val::Map(stof_map);
    }
    if let Ok(res) = py.cast::<PySet>() {
        let mut stof_set = OrdSet::default();
        for val in res {
            stof_set.insert(ValRef::new(py_any_to_val(&val, graph)));
        }
        return Val::Set(stof_set);
    }
    if let Ok(res) = py.cast::<PyFrozenSet>() {
        let mut stof_set = OrdSet::default();
        for val in res {
            stof_set.insert(ValRef::new(py_any_to_val(&val, graph)));
        }
        return Val::Set(stof_set);
    }

    // Binary
    if let Ok(res) = py.cast::<PyBytes>() {
        let bytes = res.as_bytes();
        return Val::Blob(Bytes::from(bytes.to_vec()));
    }
    if let Ok(res) = py.cast::<PyByteArray>() {
        let bytes = Bytes::from(res.to_vec());
        return Val::Blob(bytes);
    }
    
    // MemoryView (TODO? No as_bytes impl)
    //if let Ok(res) = py.cast::<PyMemoryView>() {}

    // Something else goes to null
    Val::Null
}


#[allow(unused)]
pub fn py_any_to_raw_val(py: &Bound<'_, PyAny>) -> Val {
    // None
    if let Ok(_res) = py.cast::<PyNone>() {
        return Val::Null;
    }

    // Text
    // PyString is used to map Stof Obj & Data pointers too
    if let Ok(res) = py.cast::<PyString>() {
        if let Ok(value) = res.to_str() {
            if value.len() < 100 { // large strings go right into an ArcStr and skip ptr checks
                if value.contains("_pr:_ms") {
                    let id_type = value.split("_pr:_ms").collect::<Vec<_>>();
                    if id_type.len() == 2 {
                        let pid = SId::from(id_type[0]);
                        let ty = Type::from(id_type[1]);
                        return Val::Promise(pid, ty);
                    }
                }
            }
            return Val::Str(value.into());
        }
    }

    // Bool
    if let Ok(res) = py.cast::<PyBool>() {
        if res.is_true() { return Val::Bool(true); }
        return Val::Bool(false);
    }

    // Numeric
    if let Ok(res) = py.cast::<PyComplex>() {
        // just take real (TODO: add complex numbers to Stof?)
        let real = res.real();
        return Val::Num(Num::Float(real));
    }
    if let Ok(res) = py.cast::<PyFloat>() {
        let v = res.value();
        return Val::Num(Num::Float(v));
    }
    if let Ok(res) = py.cast::<PyInt>() {
        if let Ok(v) = res.extract::<i64>() {
            return Val::Num(Num::Int(v));
        }
    }

    // Sequence
    if let Ok(res) = py.cast::<PyList>() {
        let mut list = Vector::default();
        for val in res {
            list.push_back(ValRef::new(py_any_to_raw_val(&val)));
        }
        return Val::List(list);
    }
    if let Ok(res) = py.cast::<PyTuple>() {
        let mut tup = Vector::default();
        for val in res {
            tup.push_back(ValRef::new(py_any_to_raw_val(&val)));
        }
        return Val::Tup(tup);
    }
    if let Ok(res) = py.cast::<PyRange>() {
        if let Ok(iter) = res.try_iter() {
            let mut list = Vector::default();
            for val in iter {
                match val {
                    Ok(val) => {
                        list.push_back(ValRef::new(py_any_to_raw_val(&val)));
                    },
                    Err(_) => {},
                }
            }
            return Val::List(list);
        }
    }

    // Mapping
    if let Ok(res) = py.cast::<PyDict>() {
        let mut stof_map = OrdMap::default();
        for (key, val) in res {
            stof_map.insert(
                ValRef::new(py_any_to_raw_val(&key)),
                ValRef::new(py_any_to_raw_val(&val))
            );
        }
        return Val::Map(stof_map);
    }
    if let Ok(res) = py.cast::<PySet>() {
        let mut stof_set = OrdSet::default();
        for val in res {
            stof_set.insert(ValRef::new(py_any_to_raw_val(&val)));
        }
        return Val::Set(stof_set);
    }
    if let Ok(res) = py.cast::<PyFrozenSet>() {
        let mut stof_set = OrdSet::default();
        for val in res {
            stof_set.insert(ValRef::new(py_any_to_raw_val(&val)));
        }
        return Val::Set(stof_set);
    }

    // Binary
    if let Ok(res) = py.cast::<PyBytes>() {
        let bytes = res.as_bytes();
        return Val::Blob(Bytes::from(bytes.to_vec()));
    }
    if let Ok(res) = py.cast::<PyByteArray>() {
        let bytes = Bytes::from(res.to_vec());
        return Val::Blob(bytes);
    }
    
    // MemoryView (TODO? No as_bytes impl)
    //if let Ok(res) = py.cast::<PyMemoryView>() {}

    // Something else goes to null
    Val::Null
}


pub fn val_to_py(py: Python<'_>, val: Val) -> Bound<'_, PyAny> {
    match val {
        Val::Void |
        Val::Null => PyNone::get(py).to_owned().into_any(),
        Val::Bool(v) => PyBool::new(py, v).to_owned().into_any(),
        Val::Num(num) => {
            match num {
                Num::Float(v) => PyFloat::new(py, v).into_any(),
                Num::Int(v) => PyInt::new(py, v).into_any(),
                Num::Units(v, _) => PyFloat::new(py, v).into_any(),
            }
        },
        Val::Blob(bytes) => PyByteArray::new(py, &bytes).into_any(),

        // string values
        Val::Str(v) => PyString::new(py, &v).into_any(),
        Val::Prompt(v) => PyString::new(py, &v.to_string()).into_any(),
        Val::Ver(..) => PyString::new(py, &val.to_string()).into_any(),
        Val::Data(dref) => PyString::new(py, dref.as_ref()).into_any(),
        Val::Fn(dref) => PyString::new(py, dref.as_ref()).into_any(),
        Val::Obj(nref) => PyString::new(py, nref.as_ref()).into_any(),
        Val::Promise(id, pt) => PyString::new(py, &format!("{}_pr:_ms{}", id.as_ref(), pt.type_of())).into_any(),
    
        // collections
        Val::Map(map) => {
            let dict = PyDict::new(py);
            for (k, v) in map {
                dict.set_item(val_to_py(py, k.read().clone()), val_to_py(py, v.read().clone())).expect("failed to set py dict item");
            }
            dict.into_any()
        },
        Val::Set(set) => {
            let mut items = vec![];
            for v in set {
                items.push(val_to_py(py, v.read().clone()));
            }
            PySet::new(py, items).unwrap().into_any()
        },
        Val::List(list) => {
            let mut items = vec![];
            for v in list {
                items.push(val_to_py(py, v.read().clone()));
            }
            PyList::new(py, items).unwrap().into_any()
        },
        Val::Tup(tup) => {
            let mut items = vec![];
            for v in tup {
                items.push(val_to_py(py, v.read().clone()));
            }
            PyTuple::new(py, items).unwrap().into_any()
        },
    }
}
