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

use std::{ops::Deref, sync::Arc};
use crate::{model::{Field, Format, Graph, NodeRef}, runtime::{Error, Val, Variable}};
use base64::{engine::general_purpose::STANDARD, Engine as _};

mod image;
use nanoid::nanoid;
use image::Image;

pub mod imglib;
pub use imglib::*;


pub fn load_image_formats(graph: &mut Graph) {
    graph.load_format(Arc::new(PngFormat{}));
    graph.load_format(Arc::new(JpegFormat{}));
    graph.load_format(Arc::new(GifFormat{}));
    graph.load_format(Arc::new(WebpFormat{}));
    graph.load_format(Arc::new(TiffFormat{}));
    graph.load_format(Arc::new(BmpFormat{}));
    graph.load_format(Arc::new(IcoFormat{}));
}


trait ImageFormat {
    fn img_binary_import(&self, graph: &mut Graph, _format: &str, bytes: bytes::Bytes, node: Option<NodeRef>) -> Result<(), Error> {
        match Image::from_bytes(bytes.to_vec()) {
            Ok(image) => {
                let mut parse_node = graph.ensure_main_root();
                if let Some(nd) = node {
                    parse_node = nd;
                }
                if let Some(image_ref) = graph.insert_stof_data(&parse_node, &nanoid!(), Box::new(image), None) {
                    if let Some(field_ref) = Field::direct_field(&graph, &parse_node, "image") {
                        let mut fvar = None;
                        if let Some(field) = graph.get_stof_data::<Field>(&field_ref) {
                            fvar = Some(field.value.clone());
                        }
                        if let Some(mut fvar) = fvar {
                            fvar.set(&Variable::val(Val::Data(image_ref)), graph, None)?;
                            if let Some(field) = graph.get_mut_stof_data::<Field>(&field_ref) {
                                field.value = fvar;
                            }
                        }
                    } else {
                        graph.insert_stof_data(&parse_node, "image", Box::new(Field::new(Variable::val(Val::Data(image_ref)), None)), None);
                    }
                }
                Ok(())
            },
            Err(error) => {
                Err(Error::ImageImport(error))
            },
        }
    }

    fn img_binary_export(&self, graph: &Graph, _format: &str, node: Option<NodeRef>) -> Result<bytes::Bytes, Error> {
        let exp_node;
        if let Some(nd) = node {
            exp_node = nd;
        } else {
            exp_node = graph.main_root().expect("graph does not have a main 'root' node for default Image export");
        }
        if let Some(field_ref) = Field::direct_field(&graph, &exp_node, "image") {
            if let Some(field) = graph.get_stof_data::<Field>(&field_ref) {
                match field.value.val.read().deref() {
                    Val::Data(dref) => {
                        if let Some(image) = graph.get_stof_data::<Image>(dref) { 
                            Ok(bytes::Bytes::from(image.raw.clone()))
                        } else {
                            Err(Error::ImageExport("'image' field value must be an Image data value".into()))
                        }
                    },
                    _ => {
                        Err(Error::ImageExport("'image' field value must be an Image data value".into()))
                    }
                }
            } else {
                Ok("".into())
            }
        } else {
            Ok("".into())
        }
    }
    
    fn img_string_export(&self, graph: &Graph, format: &str, node: Option<NodeRef>) -> Result<String, Error> {
        let bytes = self.img_binary_export(graph, format, node)?;
        Ok(STANDARD.encode(&bytes))
    }

    fn img_string_import(&self, graph: &mut Graph, format: &str, src: &str, node: Option<NodeRef>) -> Result<(), Error> {
        match STANDARD.decode(src) {
            Ok(bytes) => {
                self.img_binary_import(graph, format, bytes::Bytes::from(bytes), node)
            },
            Err(error) => {
                Err(Error::ImageImport(error.to_string()))
            }
        }
    }
}

#[derive(Debug)]
pub struct PngFormat;
impl ImageFormat for PngFormat {}
impl Format for PngFormat {
    fn identifiers(&self) -> Vec<String> {
        vec!["png".into()]
    }
    fn content_type(&self) -> String {
        "image/png".into()
    }
    fn binary_import(&self, graph: &mut Graph, format: &str, bytes: bytes::Bytes, node: Option<NodeRef>) -> Result<(), Error> {
       self.img_binary_import(graph, format, bytes, node)
    }
    fn binary_export(&self, graph: &Graph, format: &str, node: Option<NodeRef>) -> Result<bytes::Bytes, Error> {
        self.img_binary_export(graph, format, node)
    }
    fn string_import(&self, graph: &mut Graph, format: &str, src: &str, node: Option<NodeRef>) -> Result<(), Error> {
        self.img_string_import(graph, format, src, node)
    }
    fn string_export(&self, graph: &Graph, format: &str, node: Option<NodeRef>) -> Result<String, Error> {
        self.img_string_export(graph, format, node)
    }
}

#[derive(Debug)]
pub struct JpegFormat;
impl ImageFormat for JpegFormat {}
impl Format for JpegFormat {
    fn identifiers(&self) -> Vec<String> {
        vec!["jpg".into(), "jpeg".into()]
    }
    fn content_type(&self) -> String {
        "image/jpeg".into()
    }
    fn binary_import(&self, graph: &mut Graph, format: &str, bytes: bytes::Bytes, node: Option<NodeRef>) -> Result<(), Error> {
       self.img_binary_import(graph, format, bytes, node)
    }
    fn binary_export(&self, graph: &Graph, format: &str, node: Option<NodeRef>) -> Result<bytes::Bytes, Error> {
        self.img_binary_export(graph, format, node)
    }
    fn string_import(&self, graph: &mut Graph, format: &str, src: &str, node: Option<NodeRef>) -> Result<(), Error> {
        self.img_string_import(graph, format, src, node)
    }
    fn string_export(&self, graph: &Graph, format: &str, node: Option<NodeRef>) -> Result<String, Error> {
        self.img_string_export(graph, format, node)
    }
}

#[derive(Debug)]
pub struct GifFormat;
impl ImageFormat for GifFormat {}
impl Format for GifFormat {
    fn identifiers(&self) -> Vec<String> {
        vec!["gif".into()]
    }
    fn content_type(&self) -> String {
        "image/gif".into()
    }
    fn binary_import(&self, graph: &mut Graph, format: &str, bytes: bytes::Bytes, node: Option<NodeRef>) -> Result<(), Error> {
       self.img_binary_import(graph, format, bytes, node)
    }
    fn binary_export(&self, graph: &Graph, format: &str, node: Option<NodeRef>) -> Result<bytes::Bytes, Error> {
        self.img_binary_export(graph, format, node)
    }
    fn string_import(&self, graph: &mut Graph, format: &str, src: &str, node: Option<NodeRef>) -> Result<(), Error> {
        self.img_string_import(graph, format, src, node)
    }
    fn string_export(&self, graph: &Graph, format: &str, node: Option<NodeRef>) -> Result<String, Error> {
        self.img_string_export(graph, format, node)
    }
}

#[derive(Debug)]
pub struct WebpFormat;
impl ImageFormat for WebpFormat {}
impl Format for WebpFormat {
    fn identifiers(&self) -> Vec<String> {
        vec!["webp".into()]
    }
    fn content_type(&self) -> String {
        "image/webp".into()
    }
    fn binary_import(&self, graph: &mut Graph, format: &str, bytes: bytes::Bytes, node: Option<NodeRef>) -> Result<(), Error> {
       self.img_binary_import(graph, format, bytes, node)
    }
    fn binary_export(&self, graph: &Graph, format: &str, node: Option<NodeRef>) -> Result<bytes::Bytes, Error> {
        self.img_binary_export(graph, format, node)
    }
    fn string_import(&self, graph: &mut Graph, format: &str, src: &str, node: Option<NodeRef>) -> Result<(), Error> {
        self.img_string_import(graph, format, src, node)
    }
    fn string_export(&self, graph: &Graph, format: &str, node: Option<NodeRef>) -> Result<String, Error> {
        self.img_string_export(graph, format, node)
    }
}

#[derive(Debug)]
pub struct TiffFormat;
impl ImageFormat for TiffFormat {}
impl Format for TiffFormat {
    fn identifiers(&self) -> Vec<String> {
        vec!["tif".into(), "tiff".into()]
    }
    fn content_type(&self) -> String {
        "image/tiff".into()
    }
    fn binary_import(&self, graph: &mut Graph, format: &str, bytes: bytes::Bytes, node: Option<NodeRef>) -> Result<(), Error> {
       self.img_binary_import(graph, format, bytes, node)
    }
    fn binary_export(&self, graph: &Graph, format: &str, node: Option<NodeRef>) -> Result<bytes::Bytes, Error> {
        self.img_binary_export(graph, format, node)
    }
    fn string_import(&self, graph: &mut Graph, format: &str, src: &str, node: Option<NodeRef>) -> Result<(), Error> {
        self.img_string_import(graph, format, src, node)
    }
    fn string_export(&self, graph: &Graph, format: &str, node: Option<NodeRef>) -> Result<String, Error> {
        self.img_string_export(graph, format, node)
    }
}

#[derive(Debug)]
pub struct BmpFormat;
impl ImageFormat for BmpFormat {}
impl Format for BmpFormat {
    fn identifiers(&self) -> Vec<String> {
        vec!["bmp".into()]
    }
    fn content_type(&self) -> String {
        "image/bmp".into()
    }
    fn binary_import(&self, graph: &mut Graph, format: &str, bytes: bytes::Bytes, node: Option<NodeRef>) -> Result<(), Error> {
       self.img_binary_import(graph, format, bytes, node)
    }
    fn binary_export(&self, graph: &Graph, format: &str, node: Option<NodeRef>) -> Result<bytes::Bytes, Error> {
        self.img_binary_export(graph, format, node)
    }
    fn string_import(&self, graph: &mut Graph, format: &str, src: &str, node: Option<NodeRef>) -> Result<(), Error> {
        self.img_string_import(graph, format, src, node)
    }
    fn string_export(&self, graph: &Graph, format: &str, node: Option<NodeRef>) -> Result<String, Error> {
        self.img_string_export(graph, format, node)
    }
}

#[derive(Debug)]
pub struct IcoFormat;
impl ImageFormat for IcoFormat {}
impl Format for IcoFormat {
    fn identifiers(&self) -> Vec<String> {
        vec!["ico".into()]
    }
    fn content_type(&self) -> String {
        "image/vnd.microsoft.icon".into()
    }
    fn binary_import(&self, graph: &mut Graph, format: &str, bytes: bytes::Bytes, node: Option<NodeRef>) -> Result<(), Error> {
       self.img_binary_import(graph, format, bytes, node)
    }
    fn binary_export(&self, graph: &Graph, format: &str, node: Option<NodeRef>) -> Result<bytes::Bytes, Error> {
        self.img_binary_export(graph, format, node)
    }
    fn string_import(&self, graph: &mut Graph, format: &str, src: &str, node: Option<NodeRef>) -> Result<(), Error> {
        self.img_string_import(graph, format, src, node)
    }
    fn string_export(&self, graph: &Graph, format: &str, node: Option<NodeRef>) -> Result<String, Error> {
        self.img_string_export(graph, format, node)
    }
}
