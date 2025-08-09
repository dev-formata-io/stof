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

use std::ops::Deref;
use crate::{model::{Field, Format, Graph, NodeRef}, runtime::{Error, Val, Variable}};
use base64::{engine::general_purpose::STANDARD, Engine as _};

mod pdf;
use nanoid::nanoid;
use pdf::Pdf;

pub mod lib;
pub use lib::*;


#[derive(Debug)]
pub struct PdfFormat;
impl Format for PdfFormat {
    fn identifiers(&self) -> Vec<String> {
        vec!["pdf".into()]
    }
    fn content_type(&self) -> String {
        "application/pdf".into()
    }
    fn binary_import(&self, graph: &mut Graph, _format: &str, bytes: bytes::Bytes, node: Option<NodeRef>) -> Result<(), Error> {
        match Pdf::from_bytes(bytes.to_vec()) {
            Ok(pdf) => {
                let mut parse_node = graph.ensure_main_root();
                if let Some(nd) = node {
                    parse_node = nd;
                }
                if let Some(pdf_ref) = graph.insert_stof_data(&parse_node, &nanoid!(), Box::new(pdf), None) {
                    if let Some(field_ref) = Field::direct_field(&graph, &parse_node, "pdf") {
                        let mut fvar = None;
                        if let Some(field) = graph.get_stof_data::<Field>(&field_ref) {
                            fvar = Some(field.value.clone());
                        }
                        if let Some(mut fvar) = fvar {
                            fvar.set(&Variable::val(Val::Data(pdf_ref)), graph, None)?;
                            if let Some(field) = graph.get_mut_stof_data::<Field>(&field_ref) {
                                field.value = fvar;
                            }
                        }
                    } else {
                        graph.insert_stof_data(&parse_node, "pdf", Box::new(Field::new(Variable::val(Val::Data(pdf_ref)), None)), None);
                    }
                }
                Ok(())
            },
            Err(error) => {
                Err(Error::PDFImport(error))
            },
        }
    }
    fn binary_export(&self, graph: &Graph, _format: &str, node: Option<NodeRef>) -> Result<bytes::Bytes, Error> {
        let exp_node;
        if let Some(nd) = node {
            exp_node = nd;
        } else {
            exp_node = graph.main_root().expect("graph does not have a main 'root' node for default PDF export");
        }
        if let Some(field_ref) = Field::direct_field(&graph, &exp_node, "pdf") {
            if let Some(field) = graph.get_stof_data::<Field>(&field_ref) {
                match field.value.val.read().deref() {
                    Val::Data(dref) => {
                        if let Some(pdf) = graph.get_stof_data::<Pdf>(dref) { 
                            let mut data: Vec<u8> = Vec::new();
                            let mut mutable = pdf.doc.clone();
                            let _ = mutable.save_to(&mut data);
                            Ok(bytes::Bytes::from(data))
                        } else {
                            Err(Error::PDFExport("'pdf' field value must be a PDF data value".into()))
                        }
                    },
                    _ => {
                        Err(Error::PDFExport("'pdf' field value must be a PDF data value".into()))
                    }
                }
            } else {
                Ok("".into())
            }
        } else {
            Ok("".into())
        }
    }
    fn string_export(&self, graph: &Graph, format: &str, node: Option<NodeRef>) -> Result<String, Error> {
        let bytes = self.binary_export(graph, format, node)?;
        Ok(STANDARD.encode(&bytes))
    }
    fn string_import(&self, graph: &mut Graph, format: &str, src: &str, node: Option<NodeRef>) -> Result<(), Error> {
        match STANDARD.decode(src) {
            Ok(bytes) => {
                self.binary_import(graph, format, bytes::Bytes::from(bytes), node)
            },
            Err(error) => {
                Err(Error::PDFImport(error.to_string()))
            }
        }
    }
}
