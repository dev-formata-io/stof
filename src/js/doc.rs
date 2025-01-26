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

use std::collections::{BTreeMap, BTreeSet};
use bytes::Bytes;
use js_sys::{Function, Uint8Array};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use crate::{json::JSON, SDataRef, SDoc, SNodeRef, SVal};
use super::{StofData, StofLibFunc, StofNode, StofLib};


/// JS Stof Document Interface.
#[wasm_bindgen]
pub struct StofDoc {
    doc: SDoc,
}
impl StofDoc {
    /// Create an StofDoc from an SDoc.
    pub fn from_doc(doc: SDoc) -> Self {
        Self { doc }
    }

    /// Get the document.
    pub fn doc(&self) -> &SDoc {
        &self.doc
    }

    /// Get the document in a mutable state.
    pub fn doc_mut(&mut self) -> &mut SDoc {
        &mut self.doc
    }

    /// Insert a libfunc.
    pub fn insert_libfunc(&mut self, lib: &str, name: &str, func: JsValue) {
        let libfunc = StofLibFunc { name: name.to_owned(), func: Function::from(func) };
        let mut libfuncs = self.doc.libfuncs.write().unwrap();
        if let Some(lib) = libfuncs.get_mut(lib) {
            lib.insert(name.to_string(), libfunc);
        } else {
            let mut map = BTreeMap::new();
            map.insert(name.to_string(), libfunc);
            libfuncs.insert(lib.to_string(), map);
        }
    }
}
#[wasm_bindgen]
impl StofDoc {
    /// Construct a new StofDoc with a name.
    /// Optionally provide some existing data to load in the format of choice (leave empty if not).
    ///
    /// If loading a JS object, use 'js' instead, passing the object.
    ///
    /// Built in formats:
    /// - json
    /// - stof
    /// - toml
    /// - xml
    /// - yaml
    /// - toml
    /// - urlencoded
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
        self.doc.graph.id.clone()
    }

    /// Get the name of this document.
    pub fn name(&self) -> String {
        self.doc.graph.name.clone()
    }

    /// Set the name of this document.
    #[wasm_bindgen(js_name = setName)]
    pub fn set_name(&mut self, name: &str) {
        self.doc.graph.name = name.to_string();
    }

    /// Version of this document.
    pub fn version(&self) -> String {
        format!("{:?}", self.doc.graph.version)
    }


    /*****************************************************************************
     * Formats.
     *****************************************************************************/
    
    /// Get all of the available formats.
    #[wasm_bindgen(js_name = availableFormats)]
    pub fn available_formats(&self) -> Vec<String> {
        self.doc.available_formats().into_iter().collect()
    }

    /// Get the content type for a format.
    #[wasm_bindgen(js_name = formatContentType)]
    pub fn format_content_type(&self, format: &str) -> Option<String> {
        self.doc.format_content_type(format)
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
    pub fn header_import(&mut self, format: &str, content_type: &str, bytes: JsValue, as_name: &str) -> Result<bool, String> {
        let array = Uint8Array::from(bytes);
        let mut bytes = Bytes::from(array.to_vec());
        if let Ok(_) = self.doc.header_import("main", format, content_type, &mut bytes, as_name) {
            return Ok(true);
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
    pub fn string_import(&mut self, format: &str, src: &str, as_name: &str) -> Result<bool, String> {
        if let Ok(_) = self.doc.string_import("main", format, src, as_name) {
            return Ok(true);
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
    pub fn file_import(&mut self, format: &str, path: &str, extension: &str, as_name: &str) -> Result<bool, String> {
        if let Ok(_) = self.doc.file_import("main", format, path, extension, as_name) {
            return Ok(true);
        }
        Err(format!("Was not able to import string src with the format '{}'", format))
    }

    /// Export this document to a string using the format 'format'.
    #[wasm_bindgen(js_name = exportString)]
    pub fn export_string(&self, format: &str) -> Result<String, String> {
        if let Ok(res) = self.doc.export_string("main", format, None) {
            return Ok(res);
        }
        Err(format!("Could not export this document as a string in the format '{}'", format))
    }

    /// Export a node to a string using the format 'format'.
    #[wasm_bindgen(js_name = exportStringFor)]
    pub fn export_string_for(&self, format: &str, node: &StofNode) -> Result<String, String> {
        if let Ok(res) = self.doc.export_string("main", format, Some(&node.node_ref())) {
            return Ok(res);
        }
        Err(format!("Could not export this node as a string in the format '{}'", format))
    }

    /// Export this document to bytes using the format 'format'.
    #[wasm_bindgen(js_name = exportBytes)]
    pub fn export_bytes(&self, format: &str) -> Result<JsValue, String> {
        if let Ok(bytes) = self.doc.export_bytes("main", format, None) {
            return Ok(JsValue::from(Uint8Array::from(bytes.as_ref())));
        }
        Err(format!("Could not export this document as bytes in the format '{}'", format))
    }

    /// Export a node to bytes using the format 'format'.
    /// Some formats (like 'bstof') do not export for a singular node. It is up to the format
    /// how it gets exported!
    #[wasm_bindgen(js_name = exportBytesFor)]
    pub fn export_bytes_for(&self, format: &str, node: &StofNode) -> Result<JsValue, String> {
        if let Ok(bytes) = self.doc.export_bytes("main", format, Some(&node.node_ref())) {
            return Ok(JsValue::from(Uint8Array::from(bytes.as_ref())));
        }
        Err(format!("Could not export this document as bytes in the format '{}'", format))
    }


    /*****************************************************************************
     * Libraries.
     *****************************************************************************/
    
    /// Get all of the available libraries.
    #[wasm_bindgen(js_name = availableLibraries)]
    pub fn available_libraries(&self) -> Vec<String> {
        self.doc.available_libraries().into_iter().collect()
    }


    /// Insert a custom JS library function.
    #[wasm_bindgen(js_name = insertLibFunc)]
    pub fn insert_library_function(&mut self, lib: &str, name: &str, func: JsValue) {
        self.insert_libfunc(lib, name, func);
    }


    /// Create all libraries from library functions.
    /// Creates callable libraries out of all of the inserted custom library functions.
    /// This is required before you can use the libraries within this document.
    #[wasm_bindgen(js_name = createLibs)]
    pub fn create_custom_js_libraries(&mut self) {
        let mut to_insert = Vec::new();
        {
            let libs = self.doc.libfuncs.read().unwrap();
            for (lib, _funcs) in libs.iter() {
                to_insert.push(StofLib::new(lib));
            }
        }
        for lib in to_insert {
            StofLib::load(self, lib);
        }
    }


    /*****************************************************************************
     * Data Interface.
     *****************************************************************************/
    
    /// Get a value from this document from a path.
    /// If the path points to a field, the value will be retrieved.
    /// If the path points to a function, it will be called. Param is the function attribute 'get' if any.
    pub fn get(&mut self, path: &str) -> JsValue {
        if let Some(val) = self.doc.get(path, None) {
            return val.into();
        }
        JsValue::undefined()
    }

    /// Call a function in this document at the given path.
    #[wasm_bindgen(js_name = callFunc)]
    pub fn call_func(&mut self, path: &str, params: Vec<JsValue>) -> Result<JsValue, String> {
        let params = params.into_iter().map(|p| SVal::from((p, &self.doc))).collect();
        match self.doc.call_func(path, None, params) {
            Ok(result) => {
                Ok(JsValue::from(result))
            },
            Err(error) => {
                Err(format!("{}", error.to_string(&self.doc.graph)))
            }
        }
    }

    /// Run this document, calling all #[main] functions.
    pub fn run(&mut self) -> Option<String> {
        let res = self.doc.run(None);
        match res {
            Ok(_) => None,
            Err(error) => Some(error),
        }
    }

    /// Run this node, calling all #[main] functions on or under this node.
    #[wasm_bindgen(js_name = runAt)]
    pub fn run_node(&mut self, node: &StofNode) -> Option<String> {
        let res = self.doc.run(Some(&node.node_ref()));
        match res {
            Ok(_) => None,
            Err(error) => Some(error),
        }
    }


    /*****************************************************************************
     * Graph Interface.
     *****************************************************************************/
    
    /// Main root.
    /// This is the first root in the graph, commonly named 'root'.
    #[wasm_bindgen(js_name = mainRoot)]
    pub fn main_root(&self) -> Option<StofNode> {
        if let Some(root) = self.doc.graph.main_root() {
            return Some(StofNode::new(&root.id));
        }
        None
    }

    /// Root by name.
    #[wasm_bindgen(js_name = rootByName)]
    pub fn root_by_name(&self, name: &str) -> Option<StofNode> {
        if let Some(root) = self.doc.graph.root_by_name(name) {
            return Some(StofNode::new(&root.id));
        }
        None
    }

    /// Is a root?
    #[wasm_bindgen(js_name = isRoot)]
    pub fn is_root(&self, node: &StofNode) -> bool {
        self.doc.graph.is_root_id(&node.id())
    }

    /// Roots.
    pub fn roots(&self) -> Vec<StofNode> {
        let mut roots = Vec::new();
        for root in &self.doc.graph.roots {
            roots.push(StofNode::new(&root.id));
        }
        roots
    }

    /// Insert a new root node.
    #[wasm_bindgen(js_name = insertRoot)]
    pub fn insert_root(&mut self, name: &str) -> StofNode {
        let nref = self.doc.graph.insert_root(name);
        StofNode::new(&nref.id)
    }

    /// Insert a new node with a parent.
    /// If the parent doesn't exist, this will create a root.
    #[wasm_bindgen(js_name = insertNode)]
    pub fn insert_node(&mut self, name: &str, parent: &StofNode) -> StofNode {
        let nref = self.doc.graph.insert_node(name, Some(&parent.node_ref()));
        StofNode::new(&nref.id)
    }

    /// Insert a new node with a specific ID.
    #[wasm_bindgen(js_name = insertNodeWithId)]
    pub fn insert_node_with_id(&mut self, name: &str, id: &str, parent: &StofNode) -> StofNode {
        let nref = self.doc.graph.insert_node_with_id(name, id, Some(&parent.node_ref()));
        StofNode::new(&nref.id)
    }

    /// Remove a node.
    /// Removes all data on this node, deleting from the graph if this is the only node
    /// it is referenced on.
    #[wasm_bindgen(js_name = removeNode)]
    pub fn remove_node(&mut self, node: &StofNode) -> bool {
        self.doc.graph.remove_node(&node.node_ref())
    }

    /// Get all children of a node, on all children, grandchildren, etc...
    #[wasm_bindgen(js_name = allChildren)]
    pub fn all_children(&self, node: &StofNode) -> Vec<StofNode> {
        self.doc.graph.all_children(&node.node_ref()).into_iter().map(|nref| StofNode::new(&nref.id)).collect()
    }

    /// Create new data on a node.
    #[wasm_bindgen(js_name = createData)]
    pub fn create_data(&mut self, node: &StofNode, value: JsValue) -> Result<StofData, String> {
        StofData::construct(self, node, value)
    }

    /// Create new data on a node with an ID.
    #[wasm_bindgen(js_name = createDataWithId)]
    pub fn create_data_with_id(&mut self, node: &StofNode, id: &str, value: JsValue) -> Result<StofData, String> {
        StofData::construct_with_id(self, node, id, value)
    }

    /// Put data onto a node.
    #[wasm_bindgen(js_name = putData)]
    pub fn put_data(&mut self, node: &StofNode, data: &StofData) -> bool {
        self.doc.graph.put_data_ref(&node.node_ref(), &data.data_ref())
    }

    /// Remove data from everywhere in this document.
    #[wasm_bindgen(js_name = removeData)]
    pub fn remove_data(&mut self, data: &StofData) -> bool {
        data.remove(self)
    }

    /// Remove data from a specific node in this document.
    #[wasm_bindgen(js_name = removeDataFrom)]
    pub fn remove_data_from(&mut self, data: &StofData, node: &StofNode) -> bool {
        data.remove_from(self, node)
    }

    /// Flush node deadpool.
    pub fn flush_node_deadpool(&mut self) -> Vec<JsValue> {
        let mut res = Vec::new();
        for node in self.doc.graph.flush_node_deadpool() {
            let rep = NodeRep {
                id: node.id,
                name: node.name,
                parent: node.parent,
                children: node.children,
                data: node.data,
            };
            res.push(serde_wasm_bindgen::to_value(&rep).expect("Error creating node rep for flush deadpool."));
        }
        res
    }

    /// Flush data deadpool.
    pub fn flush_data_deadpool(&mut self) -> Vec<JsValue> {
        let mut res = Vec::new();
        for data in self.doc.graph.flush_data_deadpool() {
            let rep = DataRep {
                id: data.id,
                nodes: data.nodes,
            };
            res.push(serde_wasm_bindgen::to_value(&rep).expect("Error creating data rep for flush deadpool."));
        }
        res
    }

    /// Flush nodes.
    /// Collect dirty nodes for validation.
    /// For no limit, pass -1.
    pub fn flush_nodes(&mut self, limit: i32) -> Vec<StofNode> {
        let nodes = self.doc.graph.flush_nodes(limit);
        let mut res = Vec::new();
        for node in nodes {
            res.push(StofNode::new(&node.id));
        }
        res
    }

    /// Flush data.
    /// Collect dirty data for validation.
    /// For no limit, pass -1.
    pub fn flush_data(&mut self, limit: i32) -> Vec<StofData> {
        let data = self.doc.graph.flush_data(limit);
        let mut res = Vec::new();
        for dta in data {
            res.push(StofData::new(&dta.id));
        }
        res
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
