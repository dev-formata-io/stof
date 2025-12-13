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

use core::str;
use std::ops::Deref;
use bytes::Bytes;
use crate::{model::{Field, Format, Graph, NodeRef, Profile}, runtime::{Error, Val, Variable}};


#[derive(Debug)]
pub struct BytesFormat;
impl Format for BytesFormat {
    fn identifiers(&self) -> Vec<String> {
        vec!["bytes".into()]
    }
    fn content_type(&self) -> String {
        "application/octet-stream".into()
    }
    fn string_import(&self, graph: &mut Graph, format: &str, src: &str, node: Option<NodeRef>, profile: &Profile) -> Result<(), Error> {
        self.binary_import(graph, format, Bytes::from(src.to_string()), node, profile)
    }
    fn string_export(&self, graph: &Graph, format: &str, node: Option<NodeRef>) -> Result<String, Error> {
        let bytes = self.binary_export(graph, format, node)?;
        match str::from_utf8(&bytes) {
            Ok(res) => {
                Ok(res.to_string())
            },
            Err(error) => {
                Err(Error::BYTESExport(error.to_string()))
            }
        }
    }
    fn binary_import(&self, graph: &mut Graph, _format: &str, bytes: Bytes, node: Option<NodeRef>, _profile: &Profile) -> Result<(), Error> {
        if bytes.is_empty() { return Ok(()); }
        let mut parse_node = graph.ensure_main_root();
        if let Some(nd) = node {
            parse_node = nd;
        }
        if let Some(field_ref) = Field::direct_field(&graph, &parse_node, "bytes") {
            let mut fvar = None;
            if let Some(field) = graph.get_stof_data::<Field>(&field_ref) {
                fvar = Some(field.value.clone());
            }
            if let Some(mut fvar) = fvar {
                fvar.set(&Variable::val(Val::Blob(bytes)), graph, None)?;
                if let Some(field) = graph.get_mut_stof_data::<Field>(&field_ref) {
                    field.value = fvar;
                }
            }
        } else {
            graph.insert_stof_data(&parse_node, "bytes", Box::new(Field::new(Variable::val(Val::Blob(bytes)), None)), None);
        }
        Ok(())
    }
    fn binary_export(&self, graph: &Graph, _format: &str, node: Option<NodeRef>) -> Result<Bytes, Error> {
        let exp_node;
        if let Some(nd) = node {
            exp_node = nd;
        } else {
            exp_node = graph.main_root().expect("graph does not have a main 'root' node for default BYTES export");
        }
        if let Some(field_ref) = Field::direct_field(&graph, &exp_node, "bytes") {
            if let Some(field) = graph.get_stof_data::<Field>(&field_ref) {
                match field.value.val.read().deref() {
                    Val::Blob(vec) => {
                        Ok(Bytes::from(vec.clone()))
                    },
                    Val::Str(str) => {
                        Ok(Bytes::from(str.to_string()))
                    },
                    _ => {
                        Err(Error::BYTESExport("'bytes' field value must be a string or blob".into()))
                    }
                }
            } else {
                Ok("".into())
            }
        } else {
            Ok("".into())
        }
    }
}
