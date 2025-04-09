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

use std::{collections::{BTreeMap, BTreeSet}, sync::{Arc, RwLock}};
use bytes::Bytes;
use js_sys::{Function, Uint8Array};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use crate::{json::JSON, SDataRef, SDoc, SNodeRef, SVal};
use super::{StofData, StofLib, StofLibFunc, StofNode};


// Workaround for Wasm-Pack Error
#[cfg(target_family = "wasm")]
mod wasm_workaround {
    extern "C" {
        pub(super) fn __wasm_call_ctors();
    }
}
#[wasm_bindgen(start)]
fn start() {

    // stof::data::field::_::__ctor::h5fcded453a464929: Read a negative address value from the stack. Did we run out of memory?

    #[cfg(target_family = "wasm")]
    unsafe { wasm_workaround::__wasm_call_ctors() };
}


lazy_static! {
    // Stof document libraries.
    // Document ID -> (Library Name -> Library functions)
    pub(super) static ref DOC_LIBS: Arc<RwLock<BTreeMap<String, BTreeMap<String, BTreeMap<String, StofLibFunc>>>>> = Arc::new(RwLock::new(BTreeMap::new()));
}

// Stof Documents. TODO: find a way to make this safe... maybe SyncUnsafeCell or something similar?
pub(super) static mut DOCS: BTreeMap<String, SDoc> = BTreeMap::new();


/// Insert a document libfunc.
fn insert_global_libfunc(doc_id: &str, lib: &str, name: &str, func: JsValue) {
    let libfunc = StofLibFunc { name: name.to_owned(), func: Function::from(func) };
    let mut doclibs = DOC_LIBS.write().unwrap();
    if let Some(libs) = doclibs.get_mut(doc_id) {
        if let Some(lib) = libs.get_mut(lib) {
            lib.insert(name.to_string(), libfunc);
        } else {
            let mut map = BTreeMap::new();
            map.insert(name.to_string(), libfunc);
            libs.insert(lib.to_string(), map);
        }
    } else {
        let mut libs = BTreeMap::new();
        let mut map = BTreeMap::new();
        map.insert(name.to_string(), libfunc);
        libs.insert(lib.to_string(), map);
        doclibs.insert(doc_id.to_owned(), libs);
    }
}


/// JS Stof Document.
#[wasm_bindgen]
pub struct StofDoc {
    id: String,
    owner: bool,
}
impl Drop for StofDoc {
    fn drop(&mut self) {
        if self.owner {
            unsafe { DOCS.remove(&self.id); }
        }
    }
}
impl StofDoc {
    /// Create a non-owner document reference from a document ID.
    /// Careful to make sure that the ID exists in the DOCS.
    pub fn from_id(id: &str) -> Self {
        Self { id: id.to_owned(), owner: false, }
    }

    /// Create an StofDoc from an SDoc.
    pub fn from_doc(doc: SDoc) -> Self {
        let id = doc.graph.id.clone();
        unsafe { DOCS.insert(doc.graph.id.to_owned(), doc); }
        Self { id, owner: true, }
    }

    /// Insert a libfunc.
    pub fn insert_libfunc(&self, lib: &str, name: &str, func: JsValue) {
        insert_global_libfunc(&self.id, lib, name, func);
    }
}
#[wasm_bindgen]
impl StofDoc {
    /// Construct a new StofDoc with a name.
    /// Optionally provide some existing data to load in the format of choice (leave empty if not).
    ///
    /// If loading a JS object, use 'js' instead, passing the object.
    #[wasm_bindgen(constructor)]
    pub fn new(name: &str, src: &str, format: &str) -> Result<Self, String> {
        if src.len() > 0 && format.len() > 0 {
            if let Ok(mut doc) = SDoc::src(src, format) {
                doc.graph.name = name.to_string();
                return Ok(Self::from_doc(doc));
            }
        } else {
            let mut doc = SDoc::default();
            doc.graph.name = name.to_string();
            return Ok(Self::from_doc(doc));
        }
        Err(format!("Was not able to create a document with the format '{}'. Make sure this document has this format loaded and try again.", format))
    }

    /// Construct a new StofDoc using a JS object.
    /// This will convert the object into a serde_json::Value before creating a document out of it, capturing it's fields, sub-objects, etc...
    pub fn js(name: &str, jsobj: JsValue) -> Result<Self, String> {
        if let Ok(value) = serde_wasm_bindgen::from_value::<serde_json::Value>(jsobj) {
            let mut doc = SDoc::new(JSON::from_value(value));
            doc.graph.name = name.to_string();
            return Ok(Self::from_doc(doc))
        }
        Err(format!("Failed to create a JSON value out of the object provided"))
    }

    /// Construct a new StofDoc using bytes and a format.
    pub fn bytes(name: &str, bytes: JsValue, format: &str) -> Result<Self, String> {
        let array = Uint8Array::from(bytes);
        let bytes = Bytes::from(array.to_vec());
        if let Ok(mut doc) = SDoc::bytes(bytes, format) {
            doc.graph.name = name.to_string();
            return Ok(Self::from_doc(doc));
        }
        Err(format!("Was not able to create a document from bytes with the format '{}'", format))
    }

    /// Get the ID of this document.
    /// This is a unique string ID, generated with nanoid. Can be used for storage, etc...
    pub fn id(&self) -> String {
        self.id.clone()
    }

    /// Get the name of this document.
    pub fn name(&self) -> String {
        unsafe {
            if let Some(doc) = DOCS.get(&self.id) {
                return doc.graph.name.clone();
            }
        }
        String::default()
    }

    /// Set the name of this document.
    #[wasm_bindgen(js_name = setName)]
    pub fn set_name(&self, name: &str) {
        unsafe {
            if let Some(doc) = DOCS.get_mut(&self.id) {
                doc.graph.name = name.to_owned();
            }
        }
    }

    /// Get the version of this document.
    pub fn version(&self) -> String {
        unsafe {
            if let Some(doc) = DOCS.get(&self.id) {
                return format!("{:?}", doc.graph.version);
            }
        }
        String::default()
    }


    /*****************************************************************************
     * Formats.
     *****************************************************************************/
    
    /// Get all of the available formats.
    #[wasm_bindgen(js_name = availableFormats)]
    pub fn available_formats(&self) -> Vec<String> {
        unsafe {
            if let Some(doc) = DOCS.get(&self.id) {
                return doc.available_formats().into_iter().collect();
            }
        }
        vec![]
    }

    /// Get the content type for a format.
    #[wasm_bindgen(js_name = formatContentType)]
    pub fn format_content_type(&self, format: &str) -> Option<String> {
        unsafe {
            if let Some(doc) = DOCS.get(&self.id) {
                return doc.format_content_type(format);
            }
        }
        None
    }

    /// Header import.
    /// Used for importing bytes (Uint8Arrays) into this document with the given format.
    ///
    /// If given an explicit 'format' that exists, stof will try to use that one first. Otherwise, stof will look through all of the
    /// available formats for one with a content type that matches 'content_type'. If no matches exist, any formats that stof has
    /// with a 'format' that is contained in 'content_type' will be used as a fallback. If all of those fail, stof will use its own
    /// 'bytes' format to add the raw Vec<u8> as a blob value in the main root under the field name 'bytes'.
    ///
    /// Ex. utf-8 encoded JSON parsed into the main root: `header_import('json', '', bytes, '')`
    ///
    /// Ex. utf-8 encoded JSON parsed into the root object named 'Import': `header_import('', 'application/json', bytes, 'Import')`
    ///
    /// Ex. bstof encoded byte array: `header_import('bstof', 'application/bstof', bytes, '')`
    ///
    /// Ex. unstructured and unknown format bytes into the path 'Imports.Unknown': `header_import('', '', bytes, 'Imports.Unknown')`.
    /// The 'bytes' field with the blob (Vec<u8>) value will exist at `Imports.Unknown.bytes`.
    #[wasm_bindgen(js_name = headerImport)]
    pub fn header_import(&self, format: &str, content_type: &str, bytes: JsValue, as_name: &str) -> Result<bool, String> {
        let array = Uint8Array::from(bytes);
        let mut bytes = Bytes::from(array.to_vec());
        unsafe {
            if let Some(doc) = DOCS.get_mut(&self.id) {
                if let Ok(_) = doc.header_import("main", format, content_type, &mut bytes, as_name) {
                    return Ok(true);
                }
            }
        }
        Err(format!("Was not able to import bytes with the format '{}' and content type '{}'", format, content_type))
    }

    /// String import.
    /// Used for importing/parsing strings into this document with the given format.
    ///
    /// Ex. JSON string parsed into the main root: `string_import('json', '{ "field": true }', '')`
    ///
    /// Ex. TOML string parsed into the path 'toml_import': `string_import('toml', toml_src, 'toml_import')`.
    /// Now, in this document, all of the toml_src has been put into the location 'root.toml_import'. If toml_src contained a field named
    /// 'message' with the value 'hello, world', you can now access it in Stof with 'self.toml_import.message' if in the main root of this doc.
    ///
    /// Ex. URLencoded string parsed into the root named 'Import': `string_import('urlencoded', 'field=true&another=false', 'Import')`.
    /// After this, `assertEq(Import.field, true)` and `assertEq(Import.another, false)`.
    #[wasm_bindgen(js_name = stringImport)]
    pub fn string_import(&self, format: &str, src: &str, as_name: &str) -> Result<bool, String> {
        unsafe {
            if let Some(doc) = DOCS.get_mut(&self.id) {
                if let Ok(_) = doc.string_import("main", format, src, as_name) {
                    return Ok(true);
                }
            }
        }
        Err(format!("Was not able to import string src with the format '{}'", format))
    }

    /// File import.
    /// Used for importing/parsing files into this document with the given format.
    ///
    /// By default, the "fs" (file system) library is not included, so you'll need to implement the following functions yourself:
    /// - "fs.read" with one path (str) parameter `doc.insertLibFunc('fs', 'read', (path: string):string => {...}`
    /// - "fs.read_blob" with one path (str) parameter `doc.insertLibFunc('fs', 'read_blob', (path: string):Uint8Array => {...}`
    /// - "fs.write" with two parameters `doc.insertLibFunc('fs', 'write', (path: string, contents: string) => {...}`
    /// - "fs.write_blob" with two parameters `doc.insertLibFunc('fs', 'write_blob', (path: string, contents: Uint8Array) => {...}`
    #[wasm_bindgen(js_name = fileImport)]
    pub fn file_import(&self, format: &str, path: &str, extension: &str, as_name: &str) -> Result<bool, String> {
        unsafe {
            if let Some(doc) = DOCS.get_mut(&self.id) {
                if let Ok(_) = doc.file_import("main", format, path, extension, as_name) {
                    return Ok(true);
                }
            }
        }
        Err(format!("Was not able to import string src with the format '{}'", format))
    }

    /// Export this document to a string using the format 'format'.
    #[wasm_bindgen(js_name = exportString)]
    pub fn export_string(&self, format: &str) -> Result<String, String> {
        unsafe {
            if let Some(doc) = DOCS.get(&self.id) {
                if let Ok(res) = doc.export_string("main", format, None) {
                    return Ok(res);
                }
            }
        }
        Err(format!("Could not export this document as a string in the format '{}'", format))
    }

    /// Export a node to a string using the format 'format'.
    #[wasm_bindgen(js_name = exportStringFor)]
    pub fn export_string_for(&self, format: &str, node: &StofNode) -> Result<String, String> {
        unsafe {
            if let Some(doc) = DOCS.get(&self.id) {
                if let Ok(res) = doc.export_string("main", format, Some(&node.node_ref())) {
                    return Ok(res);
                }
            }
        }
        Err(format!("Could not export this node as a string in the format '{}'", format))
    }

    /// Export this document to bytes using the format 'format'.
    #[wasm_bindgen(js_name = exportBytes)]
    pub fn export_bytes(&self, format: &str) -> Result<JsValue, String> {
        unsafe {
            if let Some(doc) = DOCS.get(&self.id) {
                if let Ok(bytes) = doc.export_bytes("main", format, None) {
                    return Ok(JsValue::from(Uint8Array::from(bytes.as_ref())));
                }
            }
        }
        Err(format!("Could not export this document as bytes in the format '{}'", format))
    }

    /// Export a node to bytes using the format 'format'.
    /// Some formats (like 'bstof') do not export for a singular node. It is up to the format
    /// how it gets exported!
    #[wasm_bindgen(js_name = exportBytesFor)]
    pub fn export_bytes_for(&self, format: &str, node: &StofNode) -> Result<JsValue, String> {
        unsafe {
            if let Some(doc) = DOCS.get(&self.id) {
                if let Ok(bytes) = doc.export_bytes("main", format, Some(&node.node_ref())) {
                    return Ok(JsValue::from(Uint8Array::from(bytes.as_ref())));
                }
            }
        }
        Err(format!("Could not export this document as bytes in the format '{}'", format))
    }


    /*****************************************************************************
     * Libraries.
     *****************************************************************************/
    
    /// Get all of the available libraries.
    #[wasm_bindgen(js_name = availableLibraries)]
    pub fn available_libraries(&self) -> Vec<String> {
        unsafe {
            if let Some(doc) = DOCS.get(&self.id) {
                return doc.available_libraries().into_iter().collect();
            }
        }
        vec![]
    }

    /// Insert a custom JS library function.
    #[wasm_bindgen(js_name = insertLibFunc)]
    pub fn insert_library_function(&self, lib: &str, name: &str, func: JsValue) {
        self.insert_libfunc(lib, name, func);
    }

    /// Create all libraries from library functions.
    /// Creates callable libraries out of all of the inserted custom library functions.
    /// This is required before you can use the libraries within this document.
    #[wasm_bindgen(js_name = createLibs)]
    pub fn create_custom_js_libraries(&self) {
        let mut to_insert = Vec::new();
        {
            let doc_libs = DOC_LIBS.read().unwrap();
            if let Some(libs) = doc_libs.get(&self.id) {
                for (lib, _funcs) in libs.iter() {
                    to_insert.push(StofLib::new(lib));
                }
            }
        }
        unsafe {
            if let Some(doc) = DOCS.get_mut(&self.id) {
                for lib in to_insert {
                    doc.load_lib(Arc::new(lib));
                }
            }
        }
    }


    /*****************************************************************************
     * Data Interface.
     *****************************************************************************/
    
    /// Get a value from this document from a path.
    /// If the path points to a field, the value will be retrieved.
    /// If the path points to a function, it will be called. Param is the function attribute 'get' if any.
    pub fn get(&self, path: &str) -> JsValue {
        unsafe {
            if let Some(doc) = DOCS.get_mut(&self.id) {
                if let Some(val) = doc.get(path, None) {
                    return val.into();
                }
            }
        }
        JsValue::undefined()
    }

    /// Call a function in this document at the given path.
    #[wasm_bindgen(js_name = callFunc)]
    pub fn call_func(&self, path: &str, params: Vec<JsValue>) -> Result<JsValue, String> {
        unsafe {
            if let Some(doc) = DOCS.get_mut(&self.id) {
                let params = params.into_iter().map(|p| SVal::from((p, &*doc))).collect();
                return match doc.call_func(path, None, params) {
                    Ok(result) => {
                        Ok(JsValue::from(result))
                    },
                    Err(error) => {
                        Err(format!("{}", error.to_string(&doc.graph)))
                    }
                };
            }
        }
        Err(format!("CallFuncError: document not found"))
    }

    /// Run this document, calling all #[main] functions.
    pub fn run(&self) -> Option<String> {
        unsafe {
            if let Some(doc) = DOCS.get_mut(&self.id) {
                let res = doc.run(None, None);
                return match res {
                    Ok(_) => None,
                    Err(error) => Some(error),
                };
            }
        }
        Some("RunError: document not found".into())
    }

    /// Run this node, calling all #[main] functions on or under this node.
    #[wasm_bindgen(js_name = runAt)]
    pub fn run_node(&self, node: &StofNode) -> Option<String> {
        unsafe {
            if let Some(doc) = DOCS.get_mut(&self.id) {
                let res = doc.run(Some(&node.node_ref()), None);
                return match res {
                    Ok(_) => None,
                    Err(error) => Some(error),
                };
            }
        }
        Some("RunError: document not found".into())
    }


    /*****************************************************************************
     * Graph Interface.
     *****************************************************************************/
    
    /// Main root.
    /// This is the first root in the graph, commonly named 'root'.
    #[wasm_bindgen(js_name = mainRoot)]
    pub fn main_root(&self) -> Option<StofNode> {
        unsafe {
            if let Some(doc) = DOCS.get(&self.id) {
                if let Some(root) = doc.graph.main_root() {
                    return Some(StofNode::new(&root.id));
                }
            }
        }
        None
    }

    /// Root by name.
    #[wasm_bindgen(js_name = rootByName)]
    pub fn root_by_name(&self, name: &str) -> Option<StofNode> {
        unsafe {
            if let Some(doc) = DOCS.get(&self.id) {
                if let Some(root) = doc.graph.root_by_name(name) {
                    return Some(StofNode::new(&root.id));
                }
            }
        }
        None
    }

    /// Is a root?
    #[wasm_bindgen(js_name = isRoot)]
    pub fn is_root(&self, node: &StofNode) -> bool {
        unsafe {
            if let Some(doc) = DOCS.get(&self.id) {
                return doc.graph.is_root_id(&node.id());
            }
        }
        false
    }

    /// Roots.
    pub fn roots(&self) -> Vec<StofNode> {
        let mut roots = Vec::new();
        unsafe {
            if let Some(doc) = DOCS.get(&self.id) {
                for root in &doc.graph.roots {
                    roots.push(StofNode::new(&root.id));
                }
            }
        }
        roots
    }

    /// Insert a new root node.
    #[wasm_bindgen(js_name = insertRoot)]
    pub fn insert_root(&self, name: &str) -> StofNode {
        unsafe {
            if let Some(doc) = DOCS.get_mut(&self.id) {
                let nref = doc.graph.insert_root(name);
                return StofNode::new(&nref.id);
            }
        }
        StofNode::new("dne")
    }

    /// Insert a new node with a parent.
    /// If the parent doesn't exist, this will create a root.
    #[wasm_bindgen(js_name = insertNode)]
    pub fn insert_node(&self, name: &str, parent: &StofNode) -> StofNode {
        unsafe {
            if let Some(doc) = DOCS.get_mut(&self.id) {
                let nref = doc.graph.insert_node(name, Some(&parent.node_ref()));
                return StofNode::new(&nref.id);
            }
        }
        StofNode::new("dne")
    }

    /// Insert a new node with a specific ID.
    #[wasm_bindgen(js_name = insertNodeWithId)]
    pub fn insert_node_with_id(&self, name: &str, id: &str, parent: &StofNode) -> StofNode {
        unsafe {
            if let Some(doc) = DOCS.get_mut(&self.id) {
                let nref = doc.graph.insert_node_with_id(name, id, Some(&parent.node_ref()));
                return StofNode::new(&nref.id);
            }
        }
        StofNode::new("dne")
    }

    /// Remove a node.
    /// Removes all data on this node, deleting from the graph if this is the only node
    /// it is referenced on.
    #[wasm_bindgen(js_name = removeNode)]
    pub fn remove_node(&self, node: &StofNode) -> bool {
        unsafe {
            if let Some(doc) = DOCS.get_mut(&self.id) {
                return doc.graph.remove_node(&node.node_ref());
            }
        }
        false
    }

    /// Get all children of a node, on all children, grandchildren, etc...
    #[wasm_bindgen(js_name = allChildren)]
    pub fn all_children(&self, node: &StofNode) -> Vec<StofNode> {
        unsafe {
            if let Some(doc) = DOCS.get(&self.id) {
                return doc.graph.all_children(&node.node_ref()).into_iter().map(|nref| StofNode::new(&nref.id)).collect();
            }
        }
        vec![]
    }

    /// Create new data on a node.
    #[wasm_bindgen(js_name = createData)]
    pub fn create_data(&self, node: &StofNode, value: JsValue) -> Result<StofData, String> {
        StofData::construct(self, node, value)
    }

    /// Create new data on a node with an ID.
    #[wasm_bindgen(js_name = createDataWithId)]
    pub fn create_data_with_id(&self, node: &StofNode, id: &str, value: JsValue) -> Result<StofData, String> {
        StofData::construct_with_id(self, node, id, value)
    }

    /// Put data onto a node.
    #[wasm_bindgen(js_name = putData)]
    pub fn put_data(&self, node: &StofNode, data: &StofData) -> bool {
        unsafe {
            if let Some(doc) = DOCS.get_mut(&self.id) {
                return doc.graph.put_data_ref(&node.node_ref(), &data.data_ref());
            }
        }
        false
    }

    /// Remove data from everywhere in this document.
    #[wasm_bindgen(js_name = removeData)]
    pub fn remove_data(&self, data: &StofData) -> bool {
        data.remove(self)
    }

    /// Remove data from a specific node in this document.
    #[wasm_bindgen(js_name = removeDataFrom)]
    pub fn remove_data_from(&self, data: &StofData, node: &StofNode) -> bool {
        data.remove_from(self, node)
    }

    /// Flush node deadpool.
    pub fn flush_node_deadpool(&self) -> Vec<JsValue> {
        let mut res = Vec::new();
        unsafe {
            if let Some(doc) = DOCS.get_mut(&self.id) {
                for node in doc.graph.flush_node_deadpool() {
                    let rep = NodeRep {
                        id: node.id,
                        name: node.name,
                        parent: node.parent,
                        children: node.children,
                        data: node.data,
                    };
                    res.push(serde_wasm_bindgen::to_value(&rep).expect("Error creating node rep for flush deadpool."));
                }
            }
        }
        res
    }

    /// Flush data deadpool.
    pub fn flush_data_deadpool(&self) -> Vec<JsValue> {
        let mut res = Vec::new();
        unsafe {
            if let Some(doc) = DOCS.get_mut(&self.id) {
                for data in doc.graph.flush_data_deadpool() {
                    let rep = DataRep {
                        id: data.id,
                        nodes: data.nodes,
                    };
                    res.push(serde_wasm_bindgen::to_value(&rep).expect("Error creating data rep for flush deadpool."));
                }
            }
        }
        res
    }

    /// Flush nodes.
    /// Collect dirty nodes for validation.
    /// For no limit, pass -1.
    pub fn flush_nodes(&self, limit: i32) -> Vec<StofNode> {
        unsafe {
            if let Some(doc) = DOCS.get_mut(&self.id) {
                let nodes = doc.graph.flush_nodes(limit);
                let mut res = Vec::new();
                for node in nodes {
                    res.push(StofNode::new(&node.id));
                }
                return res;
            }
        }
        vec![]
    }

    /// Flush data.
    /// Collect dirty data for validation.
    /// For no limit, pass -1.
    pub fn flush_data(&self, limit: i32) -> Vec<StofData> {
        unsafe {
            if let Some(doc) = DOCS.get_mut(&self.id) {
                let data = doc.graph.flush_data(limit);
                let mut res = Vec::new();
                for dta in data {
                    res.push(StofData::new(&dta.id));
                }
                return res;
            }
        }
        vec![]
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
struct NodeRep {
    id: String,
    name: String,
    parent: Option<SNodeRef>,
    children: BTreeSet<SNodeRef>,
    data: BTreeSet<SDataRef>,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
struct DataRep {
    id: String,
    nodes: Vec<SNodeRef>,
}
