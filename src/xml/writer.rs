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

use crate::{json::JSON, lang::SError, SDoc, SNodeRef};


/// Write XML string from an AseGraph.
/// Warning - looses original XML formatting if coming from XML.
pub fn write_xml(pid: &str, doc: &SDoc) -> Result<String, SError> {
    let value = JSON::to_value(pid, doc)?;
    let res = serde_xml_rs::to_string(&value);
    match res {
        Ok(value) => {
            Ok(value)
        },
        Err(error) => {
            Err(SError::fmt(pid, doc, "xml", &format!("node write error: {}", error.to_string())))
        }
    }
}


/// Write XML string from an AseGraph and node.
/// Warning - looses original XML formatting if coming from XML.
pub fn node_write_xml(pid: &str, doc: &SDoc, node: &SNodeRef) -> Result<String, SError> {
    let value = JSON::to_node_value(&doc.graph, node);
    let res = serde_xml_rs::to_string(&value);
    match res {
        Ok(value) => {
            Ok(value)
        },
        Err(error) => {
            Err(SError::fmt(pid, doc, "xml", &format!("node write error: {}", error.to_string())))
        }
    }
}
