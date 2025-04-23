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

pub mod library;
use std::{io::Cursor, ops::Deref, path::Path, sync::Arc};
use bytes::Bytes;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use image::{imageops::FilterType, metadata::Orientation, DynamicImage, ImageFormat, ImageReader};
use serde::{Deserialize, Serialize};
use crate::{lang::SError, Data, Format, IntoNodeRef, SData, SDoc, SField, SGraph, SNodeRef, SVal};


/// Image.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SImage {
    /// Raw image as bytes.
    pub raw_image: Vec<u8>,
    
    #[serde(skip)]
    pub dynamic_image: Option<DynamicImage>,
}

/// Impl Data for Image.
/// Not included by default, so not a "core_data" type.
/// Default library name will be the Serde Tagname (if exists).
#[typetag::serde(name = "Image")]
impl Data for SImage {}

/// Implement SImage.
impl SImage {
    /// Create a new SImage from a file.
    pub fn from_file<P>(path: P) -> Result<Self, String>
        where P: AsRef<Path>,
    {
        let res = ImageReader::open(path);
        match res {
            Ok(reader) => {
                let decoded = reader.decode();
                match decoded {
                    Ok(image) => {
                        let mut bytes: Vec<u8> = Vec::new();
                        if let Err(error) = image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png) {
                            return Err(error.to_string());
                        }
                        Ok(Self {
                            raw_image: bytes,
                            dynamic_image: Some(image),
                        })
                    },
                    Err(error) => {
                        Err(error.to_string())
                    }
                }
            },
            Err(error) => {
                Err(error.to_string())
            }
        }
    }

    /// Create a new SImage from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if let Ok(reader) = ImageReader::new(Cursor::new(bytes)).with_guessed_format() {
            if let Ok(image) = reader.decode() {
                let mut bytes: Vec<u8> = Vec::new();
                if let Err(error) = image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png) {
                    return Err(error.to_string());
                }
                return Ok(Self {
                    raw_image: bytes,
                    dynamic_image: Some(image),
                });
            }
        }
        Err("Could not convert bytes into an image".into())
    }
    
    /// Ensure dynamic image from raw image.
    pub fn ensure_dynamic(&mut self) -> bool {
        if self.dynamic_image.is_some() {
            return true;
        } else if let Ok(reader) = ImageReader::new(Cursor::new(&self.raw_image)).with_guessed_format() {
            if let Ok(image) = reader.decode() {
                self.dynamic_image = Some(image);
                return true;
            }
        }
        false
    }

    /// Save dynamic image to raw image.
    pub fn save_image(&mut self) -> bool {
        if let Some(image) = &self.dynamic_image {
            let mut bytes: Vec<u8> = Vec::new();
            if let Err(_error) = image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png) {
                return false;
            }
            self.raw_image = bytes;
            return true;
        }
        false
    }

    /// PNG bytes.
    pub fn png_bytes(&mut self) -> Option<Vec<u8>> {
        if !self.ensure_dynamic() { return None; }
        if let Some(image) = &self.dynamic_image {
            let mut bytes: Vec<u8> = Vec::new();
            if let Err(_error) = image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png) {
                return None;
            }
            return Some(bytes);
        }
        None
    }

    /// JPEG bytes.
    pub fn jpeg_bytes(&mut self) -> Option<Vec<u8>> {
        if !self.ensure_dynamic() { return None; }
        if let Some(image) = &self.dynamic_image {
            let mut bytes: Vec<u8> = Vec::new();
            if let Err(_error) = image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Jpeg) {
                return None;
            }
            return Some(bytes);
        }
        None
    }

    /// Gif bytes.
    pub fn gif_bytes(&mut self) -> Option<Vec<u8>> {
        if !self.ensure_dynamic() { return None; }
        if let Some(image) = &self.dynamic_image {
            let mut bytes: Vec<u8> = Vec::new();
            if let Err(_error) = image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Gif) {
                return None;
            }
            return Some(bytes);
        }
        None
    }

    /// Webp bytes.
    pub fn webp_bytes(&mut self) -> Option<Vec<u8>> {
        if !self.ensure_dynamic() { return None; }
        if let Some(image) = &self.dynamic_image {
            let mut bytes: Vec<u8> = Vec::new();
            if let Err(_error) = image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::WebP) {
                return None;
            }
            return Some(bytes);
        }
        None
    }

    /// PNM bytes.
    pub fn pnm_bytes(&mut self) -> Option<Vec<u8>> {
        if !self.ensure_dynamic() { return None; }
        if let Some(image) = &self.dynamic_image {
            let mut bytes: Vec<u8> = Vec::new();
            if let Err(_error) = image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Pnm) {
                return None;
            }
            return Some(bytes);
        }
        None
    }

    /// Tiff bytes.
    pub fn tiff_bytes(&mut self) -> Option<Vec<u8>> {
        if !self.ensure_dynamic() { return None; }
        if let Some(image) = &self.dynamic_image {
            let mut bytes: Vec<u8> = Vec::new();
            if let Err(_error) = image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Tiff) {
                return None;
            }
            return Some(bytes);
        }
        None
    }

    /// Tga bytes.
    pub fn tga_bytes(&mut self) -> Option<Vec<u8>> {
        if !self.ensure_dynamic() { return None; }
        if let Some(image) = &self.dynamic_image {
            let mut bytes: Vec<u8> = Vec::new();
            if let Err(_error) = image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Tga) {
                return None;
            }
            return Some(bytes);
        }
        None
    }

    /// Dds bytes.
    pub fn dds_bytes(&mut self) -> Option<Vec<u8>> {
        if !self.ensure_dynamic() { return None; }
        if let Some(image) = &self.dynamic_image {
            let mut bytes: Vec<u8> = Vec::new();
            if let Err(_error) = image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Dds) {
                return None;
            }
            return Some(bytes);
        }
        None
    }

    /// Bmp bytes.
    pub fn bmp_bytes(&mut self) -> Option<Vec<u8>> {
        if !self.ensure_dynamic() { return None; }
        if let Some(image) = &self.dynamic_image {
            let mut bytes: Vec<u8> = Vec::new();
            if let Err(_error) = image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Bmp) {
                return None;
            }
            return Some(bytes);
        }
        None
    }

    /// Ico bytes.
    pub fn ico_bytes(&mut self) -> Option<Vec<u8>> {
        if !self.ensure_dynamic() { return None; }
        if let Some(image) = &self.dynamic_image {
            let mut bytes: Vec<u8> = Vec::new();
            if let Err(_error) = image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Ico) {
                return None;
            }
            return Some(bytes);
        }
        None
    }

    /// Hdr bytes.
    pub fn hdr_bytes(&mut self) -> Option<Vec<u8>> {
        if !self.ensure_dynamic() { return None; }
        if let Some(image) = &self.dynamic_image {
            let mut bytes: Vec<u8> = Vec::new();
            if let Err(_error) = image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Hdr) {
                return None;
            }
            return Some(bytes);
        }
        None
    }

    /// OpenExr bytes.
    pub fn open_exr_bytes(&mut self) -> Option<Vec<u8>> {
        if !self.ensure_dynamic() { return None; }
        if let Some(image) = &self.dynamic_image {
            let mut bytes: Vec<u8> = Vec::new();
            if let Err(_error) = image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::OpenExr) {
                return None;
            }
            return Some(bytes);
        }
        None
    }

    /// Farbfeld bytes.
    pub fn farbfeld_bytes(&mut self) -> Option<Vec<u8>> {
        if !self.ensure_dynamic() { return None; }
        if let Some(image) = &self.dynamic_image {
            let mut bytes: Vec<u8> = Vec::new();
            if let Err(_error) = image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Farbfeld) {
                return None;
            }
            return Some(bytes);
        }
        None
    }

    /// Avif bytes.
    pub fn avif_bytes(&mut self) -> Option<Vec<u8>> {
        if !self.ensure_dynamic() { return None; }
        if let Some(image) = &self.dynamic_image {
            let mut bytes: Vec<u8> = Vec::new();
            if let Err(_error) = image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Avif) {
                return None;
            }
            return Some(bytes);
        }
        None
    }

    /// Qoi bytes.
    pub fn qoi_bytes(&mut self) -> Option<Vec<u8>> {
        if !self.ensure_dynamic() { return None; }
        if let Some(image) = &self.dynamic_image {
            let mut bytes: Vec<u8> = Vec::new();
            if let Err(_error) = image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Qoi) {
                return None;
            }
            return Some(bytes);
        }
        None
    }

    /// Pcx bytes.
    pub fn pcx_bytes(&mut self) -> Option<Vec<u8>> {
        if !self.ensure_dynamic() { return None; }
        if let Some(image) = &self.dynamic_image {
            let mut bytes: Vec<u8> = Vec::new();
            if let Err(_error) = image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Pcx) {
                return None;
            }
            return Some(bytes);
        }
        None
    }

    /// Width of this image.
    pub fn width(&mut self) -> u32 {
        if !self.ensure_dynamic() { return 0; }
        if let Some(image) = &self.dynamic_image {
            return image.width();
        }
        0
    }

    /// Height of this image.
    pub fn height(&mut self) -> u32 {
        if !self.ensure_dynamic() { return 0; }
        if let Some(image) = &self.dynamic_image {
            return image.height();
        }
        0
    }

    /// Grayscale.
    /// Turns this image into a grayscale version of itself.
    pub fn grayscale(&mut self) {
        if !self.ensure_dynamic() { return; }
        let mut grayscale = None;
        if let Some(image) = &self.dynamic_image {
            grayscale = Some(image.grayscale());
        }
        if let Some(image) = grayscale {
            self.dynamic_image = Some(image);
            self.save_image();
        }
    }

    /// Invert the colors of this image.
    pub fn invert(&mut self) {
        if !self.ensure_dynamic() { return; }
        if let Some(image) = &mut self.dynamic_image {
            image.invert();
            self.save_image();
        }
    }

    /// Resize this image (keeps aspect ratio).
    pub fn resize(&mut self, width: u32, height: u32, filter: FilterType) {
        if !self.ensure_dynamic() { return; }
        let mut resized = None;
        if let Some(image) = &self.dynamic_image {
            resized = Some(image.resize(width, height, filter));
        }
        if let Some(image) = resized {
            self.dynamic_image = Some(image);
            self.save_image();
        }
    }

    /// Resize this image exactly (does not keep aspect ratio).
    pub fn resize_exact(&mut self, width: u32, height: u32, filter: FilterType) {
        if !self.ensure_dynamic() { return; }
        let mut resized = None;
        if let Some(image) = &self.dynamic_image {
            resized = Some(image.resize_exact(width, height, filter));
        }
        if let Some(image) = resized {
            self.dynamic_image = Some(image);
            self.save_image();
        }
    }

    /// Scale this image down to fit within a specific size.
    pub fn thumbnail(&mut self, width: u32, height: u32) {
        if !self.ensure_dynamic() { return; }
        let mut resized = None;
        if let Some(image) = &self.dynamic_image {
            resized = Some(image.thumbnail(width, height));
        }
        if let Some(image) = resized {
            self.dynamic_image = Some(image);
            self.save_image();
        }
    }

    /// Scale this image down to fit within a specific size without preserving aspect ratio.
    pub fn thumbnail_exact(&mut self, width: u32, height: u32) {
        if !self.ensure_dynamic() { return; }
        let mut resized = None;
        if let Some(image) = &self.dynamic_image {
            resized = Some(image.thumbnail_exact(width, height));
        }
        if let Some(image) = resized {
            self.dynamic_image = Some(image);
            self.save_image();
        }
    }

    /// Performs a guassian blur on this image.
    pub fn blur(&mut self, sigma: f32) {
        if !self.ensure_dynamic() { return; }
        let mut resized = None;
        if let Some(image) = &self.dynamic_image {
            resized = Some(image.blur(sigma));
        }
        if let Some(image) = resized {
            self.dynamic_image = Some(image);
            self.save_image();
        }
    }

    /// Fast blur.
    pub fn fast_blur(&mut self, sigma: f32) {
        if !self.ensure_dynamic() { return; }
        let mut resized = None;
        if let Some(image) = &self.dynamic_image {
            resized = Some(image.fast_blur(sigma));
        }
        if let Some(image) = resized {
            self.dynamic_image = Some(image);
            self.save_image();
        }
    }

    /// Adjust contrast. Positive to increase, negative to decrease.
    pub fn adjust_contrast(&mut self, contrast: f32) {
        if !self.ensure_dynamic() { return; }
        let mut altered = None;
        if let Some(image) = &self.dynamic_image {
            altered = Some(image.adjust_contrast(contrast));
        }
        if let Some(image) = altered {
            self.dynamic_image = Some(image);
            self.save_image();
        }
    }

    /// Brighten the pixels of this image.
    pub fn brighten(&mut self, value: i32) {
        if !self.ensure_dynamic() { return; }
        let mut altered = None;
        if let Some(image) = &self.dynamic_image {
            altered = Some(image.brighten(value));
        }
        if let Some(image) = altered {
            self.dynamic_image = Some(image);
            self.save_image();
        }
    }

    /// Flip vertically.
    pub fn flip_vertical(&mut self) {
        if !self.ensure_dynamic() { return; }
        if let Some(image) = &mut self.dynamic_image {
            image.apply_orientation(Orientation::FlipVertical);
            self.save_image();
        }
    }

    /// Flip horizontally.
    pub fn flip_horizontal(&mut self) {
        if !self.ensure_dynamic() { return; }
        if let Some(image) = &mut self.dynamic_image {
            image.apply_orientation(Orientation::FlipHorizontal);
            self.save_image();
        }
    }

    /// Rotate 90 degrees clockwise.
    pub fn rotate_90(&mut self) {
        if !self.ensure_dynamic() { return; }
        if let Some(image) = &mut self.dynamic_image {
            image.apply_orientation(Orientation::Rotate90);
            self.save_image();
        }
    }

    /// Rotate 180 degrees clockwise.
    pub fn rotate_180(&mut self) {
        if !self.ensure_dynamic() { return; }
        if let Some(image) = &mut self.dynamic_image {
            image.apply_orientation(Orientation::Rotate180);
            self.save_image();
        }
    }

    /// Rotate 270 degrees clockwise.
    pub fn rotate_270(&mut self) {
        if !self.ensure_dynamic() { return; }
        if let Some(image) = &mut self.dynamic_image {
            image.apply_orientation(Orientation::Rotate270);
            self.save_image();
        }
    }


    /*****************************************************************************
     * Format Helpers.
     *****************************************************************************/

    /// Parse into a new graph.
    pub fn parse(bytes: &Bytes) -> Result<SGraph, String> {
        let mut graph = SGraph::default();
        let root = graph.insert_root("root");

        let image = SImage::from_bytes(&bytes)?;
        let dref = SData::insert_new(&mut graph, &root, Box::new(image)).unwrap();

        let field = SField::new("image", SVal::Data(dref));
        SData::insert_new(&mut graph, &root, Box::new(field));
        
        Ok(graph)
    }

    /// To bytes.
    pub fn to_bytes(pid: &str, doc: &SDoc) -> Result<Bytes, SError> {
        if let Some(field) = SField::field(&doc.graph, "image", '.', doc.graph.main_root().as_ref()) {
            match &field.value {
                SVal::Data(dref) => {
                    if let Some(image) = SData::get::<SImage>(&doc.graph, dref) {
                        return Ok(Bytes::from(image.raw_image.clone())); // PNG image by default
                    }
                },
                SVal::Blob(bytes) => return Ok(Bytes::from(bytes.clone())),
                SVal::Boxed(val) => {
                    let val = val.lock().unwrap();
                    let val = val.deref();
                    match val {
                        SVal::Blob(bytes) => return Ok(Bytes::from(bytes.clone())),
                        SVal::Data(dref) => {
                            if let Some(image) = SData::get::<SImage>(&doc.graph, dref) {
                                return Ok(Bytes::from(image.raw_image.clone())); // PNG image by default
                            }
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        }
        Err(SError::fmt(pid, doc, "image", "did not find image on the main root of this graph"))
    }

    /// Node to bytes.
    pub fn node_to_bytes(pid: &str, doc: &SDoc, node: impl IntoNodeRef) -> Result<Bytes, SError> {
        if let Some(field) = SField::field(&doc.graph, "image", '.', Some(&node.node_ref())) {
            match &field.value {
                SVal::Data(dref) => {
                    if let Some(image) = SData::get::<SImage>(&doc.graph, dref) {
                        return Ok(Bytes::from(image.raw_image.clone())); // PNG image by default
                    }
                },
                SVal::Blob(bytes) => return Ok(Bytes::from(bytes.clone())),
                SVal::Boxed(val) => {
                    let val = val.lock().unwrap();
                    let val = val.deref();
                    match val {
                        SVal::Blob(bytes) => return Ok(Bytes::from(bytes.clone())),
                        SVal::Data(dref) => {
                            if let Some(image) = SData::get::<SImage>(&doc.graph, dref) {
                                return Ok(Bytes::from(image.raw_image.clone())); // PNG image by default
                            }
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        }
        Err(SError::fmt(pid, doc, "image", "did not find image on the requested node"))
    }

    /// Header import.
    pub fn header_import(pid: &str, doc: &mut SDoc, bytes: &mut Bytes, as_name: &str) -> Result<(), SError> {
        let res = SImage::parse(bytes);
        if res.is_err() {
            return Err(SError::fmt(pid, &doc, "image", "could not parse bytes into a graph (header import)"));
        }
        let mut graph = res.unwrap();
        if as_name.len() > 0 && as_name != "root" {
            let mut path = as_name.replace(".", "/");
            if as_name.starts_with("self") || as_name.starts_with("super") {
                if let Some(ptr) = doc.self_ptr(pid) {
                    path = format!("{}/{}", ptr.path(&doc.graph), path);
                }
            }

            // as_name is really a location, so ensure the nodes and move it there
            let mut loc_graph = SGraph::default();
            let loc = loc_graph.ensure_nodes(&path, '/', true, None);
            if let Some(main) = graph.main_root() {
                if let Some(main) = main.node(&graph) {
                    loc_graph.absorb_external_node(&graph, main, &loc);
                }
            }
            graph = loc_graph;
        }
        doc.graph.default_absorb_merge(graph)
    }

    /// String import (base64 image string).
    pub fn string_import(pid: &str, doc: &mut SDoc, src: &str, as_name: &str) -> Result<(), SError> {
        if let Ok(bytes) = STANDARD.decode(src) {
            let mut bytes = Bytes::from(bytes);
            SImage::header_import(pid, doc, &mut bytes, as_name)
        } else {
            Err(SError::fmt(pid, &doc, "image", "failed to decode base64 image string"))
        }
    }

    /// Export bytes.
    pub fn export_bytes(pid: &str, doc: &SDoc, node: Option<&SNodeRef>) -> Result<Bytes, SError> {
        if node.is_some() {
            SImage::node_to_bytes(pid, doc, node)
        } else {
            SImage::to_bytes(pid, &doc)
        }
    }

    /// Export image as base64 string.
    pub fn export_string(pid: &str, doc: &SDoc, node: Option<&SNodeRef>) -> Result<String, SError> {
        let bytes = Self::export_bytes(pid, doc, node)?;
        let res = STANDARD.encode(&bytes);
        Ok(res)
    }
}


/// Add image formats to document.
pub fn load_image_formats(doc: &mut SDoc) {
    doc.load_format(Arc::new(PNG {}));
    doc.load_format(Arc::new(JPEG {}));
    doc.load_format(Arc::new(GIF {}));
    doc.load_format(Arc::new(WEBP {}));
    doc.load_format(Arc::new(TIF {}));
    doc.load_format(Arc::new(TIFF {}));
    doc.load_format(Arc::new(BMP {}));
    doc.load_format(Arc::new(ICO {}));
}


/// PNG format.
pub struct PNG;
impl Format for PNG {
    /// Format getter.
    fn format(&self) -> String {
        "png".to_string()
    }

    /// Content type (general image bytes).
    fn content_type(&self) -> String {
        "image/png".to_string()
    }

    /// Header import.
    fn header_import(&self, pid: &str, doc: &mut SDoc, _content_type: &str, bytes: &mut Bytes, as_name: &str) -> Result<(), SError> {
        SImage::header_import(pid, doc, bytes, as_name)
    }

    /// String import (base64 image string).
    fn string_import(&self, pid: &str, doc: &mut SDoc, src: &str, as_name: &str) -> Result<(), SError> {
        SImage::string_import(pid, doc, src, as_name)
    }

    /// File import.
    fn file_import(&self, pid: &str, doc: &mut SDoc, format: &str, full_path: &str, _extension: &str, as_name: &str) -> Result<(), SError> {
        let mut bytes = doc.fs_read_blob(pid, full_path)?;
        self.header_import(pid, doc, format, &mut bytes, as_name)
    }

    /// Export bytes.
    fn export_bytes(&self, pid: &str, doc: &SDoc, node: Option<&SNodeRef>) -> Result<Bytes, SError> {
        SImage::export_bytes(pid, doc, node)
    }

    /// Export image as base64 string.
    fn export_string(&self, pid: &str, doc: &SDoc, node: Option<&SNodeRef>) -> Result<String, SError> {
        SImage::export_string(pid, doc, node)
    }
}


/// JPEG format.
pub struct JPEG;
impl Format for JPEG {
    /// Format getter.
    fn format(&self) -> String {
        "jpg".to_string()
    }

    /// Content type (general image bytes).
    fn content_type(&self) -> String {
        "image/jpeg".to_string()
    }

    /// Header import.
    fn header_import(&self, pid: &str, doc: &mut SDoc, _content_type: &str, bytes: &mut Bytes, as_name: &str) -> Result<(), SError> {
        SImage::header_import(pid, doc, bytes, as_name)
    }

    /// String import (base64 image string).
    fn string_import(&self, pid: &str, doc: &mut SDoc, src: &str, as_name: &str) -> Result<(), SError> {
        SImage::string_import(pid, doc, src, as_name)
    }

    /// File import.
    fn file_import(&self, pid: &str, doc: &mut SDoc, format: &str, full_path: &str, _extension: &str, as_name: &str) -> Result<(), SError> {
        let mut bytes = doc.fs_read_blob(pid, full_path)?;
        self.header_import(pid, doc, format, &mut bytes, as_name)
    }

    /// Export bytes.
    fn export_bytes(&self, pid: &str, doc: &SDoc, node: Option<&SNodeRef>) -> Result<Bytes, SError> {
        SImage::export_bytes(pid, doc, node)
    }

    /// Export image as base64 string.
    fn export_string(&self, pid: &str, doc: &SDoc, node: Option<&SNodeRef>) -> Result<String, SError> {
        SImage::export_string(pid, doc, node)
    }
}


/// GIF format.
pub struct GIF;
impl Format for GIF {
    /// Format getter.
    fn format(&self) -> String {
        "gif".to_string()
    }

    /// Content type (general image bytes).
    fn content_type(&self) -> String {
        "image/gif".to_string()
    }

    /// Header import.
    fn header_import(&self, pid: &str, doc: &mut SDoc, _content_type: &str, bytes: &mut Bytes, as_name: &str) -> Result<(), SError> {
        SImage::header_import(pid, doc, bytes, as_name)
    }

    /// String import (base64 image string).
    fn string_import(&self, pid: &str, doc: &mut SDoc, src: &str, as_name: &str) -> Result<(), SError> {
        SImage::string_import(pid, doc, src, as_name)
    }

    /// File import.
    fn file_import(&self, pid: &str, doc: &mut SDoc, format: &str, full_path: &str, _extension: &str, as_name: &str) -> Result<(), SError> {
        let mut bytes = doc.fs_read_blob(pid, full_path)?;
        self.header_import(pid, doc, format, &mut bytes, as_name)
    }

    /// Export bytes.
    fn export_bytes(&self, pid: &str, doc: &SDoc, node: Option<&SNodeRef>) -> Result<Bytes, SError> {
        SImage::export_bytes(pid, doc, node)
    }

    /// Export image as base64 string.
    fn export_string(&self, pid: &str, doc: &SDoc, node: Option<&SNodeRef>) -> Result<String, SError> {
        SImage::export_string(pid, doc, node)
    }
}


/// WEBP format.
pub struct WEBP;
impl Format for WEBP {
    /// Format getter.
    fn format(&self) -> String {
        "webp".to_string()
    }

    /// Content type (general image bytes).
    fn content_type(&self) -> String {
        "image/webp".to_string()
    }

    /// Header import.
    fn header_import(&self, pid: &str, doc: &mut SDoc, _content_type: &str, bytes: &mut Bytes, as_name: &str) -> Result<(), SError> {
        SImage::header_import(pid, doc, bytes, as_name)
    }

    /// String import (base64 image string).
    fn string_import(&self, pid: &str, doc: &mut SDoc, src: &str, as_name: &str) -> Result<(), SError> {
        SImage::string_import(pid, doc, src, as_name)
    }

    /// File import.
    fn file_import(&self, pid: &str, doc: &mut SDoc, format: &str, full_path: &str, _extension: &str, as_name: &str) -> Result<(), SError> {
        let mut bytes = doc.fs_read_blob(pid, full_path)?;
        self.header_import(pid, doc, format, &mut bytes, as_name)
    }

    /// Export bytes.
    fn export_bytes(&self, pid: &str, doc: &SDoc, node: Option<&SNodeRef>) -> Result<Bytes, SError> {
        SImage::export_bytes(pid, doc, node)
    }

    /// Export image as base64 string.
    fn export_string(&self, pid: &str, doc: &SDoc, node: Option<&SNodeRef>) -> Result<String, SError> {
        SImage::export_string(pid, doc, node)
    }
}


/// TIF format.
pub struct TIF;
impl Format for TIF {
    /// Format getter.
    fn format(&self) -> String {
        "tif".to_string()
    }

    /// Content type (general image bytes).
    fn content_type(&self) -> String {
        "image/tiff".to_string()
    }

    /// Header import.
    fn header_import(&self, pid: &str, doc: &mut SDoc, _content_type: &str, bytes: &mut Bytes, as_name: &str) -> Result<(), SError> {
        SImage::header_import(pid, doc, bytes, as_name)
    }

    /// String import (base64 image string).
    fn string_import(&self, pid: &str, doc: &mut SDoc, src: &str, as_name: &str) -> Result<(), SError> {
        SImage::string_import(pid, doc, src, as_name)
    }

    /// File import.
    fn file_import(&self, pid: &str, doc: &mut SDoc, format: &str, full_path: &str, _extension: &str, as_name: &str) -> Result<(), SError> {
        let mut bytes = doc.fs_read_blob(pid, full_path)?;
        self.header_import(pid, doc, format, &mut bytes, as_name)
    }

    /// Export bytes.
    fn export_bytes(&self, pid: &str, doc: &SDoc, node: Option<&SNodeRef>) -> Result<Bytes, SError> {
        SImage::export_bytes(pid, doc, node)
    }

    /// Export image as base64 string.
    fn export_string(&self, pid: &str, doc: &SDoc, node: Option<&SNodeRef>) -> Result<String, SError> {
        SImage::export_string(pid, doc, node)
    }
}


/// TIFF format.
pub struct TIFF;
impl Format for TIFF {
    /// Format getter.
    fn format(&self) -> String {
        "tiff".to_string()
    }

    /// Content type (general image bytes).
    fn content_type(&self) -> String {
        "image/tiff".to_string()
    }

    /// Header import.
    fn header_import(&self, pid: &str, doc: &mut SDoc, _content_type: &str, bytes: &mut Bytes, as_name: &str) -> Result<(), SError> {
        SImage::header_import(pid, doc, bytes, as_name)
    }

    /// String import (base64 image string).
    fn string_import(&self, pid: &str, doc: &mut SDoc, src: &str, as_name: &str) -> Result<(), SError> {
        SImage::string_import(pid, doc, src, as_name)
    }

    /// File import.
    fn file_import(&self, pid: &str, doc: &mut SDoc, format: &str, full_path: &str, _extension: &str, as_name: &str) -> Result<(), SError> {
        let mut bytes = doc.fs_read_blob(pid, full_path)?;
        self.header_import(pid, doc, format, &mut bytes, as_name)
    }

    /// Export bytes.
    fn export_bytes(&self, pid: &str, doc: &SDoc, node: Option<&SNodeRef>) -> Result<Bytes, SError> {
        SImage::export_bytes(pid, doc, node)
    }

    /// Export image as base64 string.
    fn export_string(&self, pid: &str, doc: &SDoc, node: Option<&SNodeRef>) -> Result<String, SError> {
        SImage::export_string(pid, doc, node)
    }
}


/// BMP format.
pub struct BMP;
impl Format for BMP {
    /// Format getter.
    fn format(&self) -> String {
        "bmp".to_string()
    }

    /// Content type (general image bytes).
    fn content_type(&self) -> String {
        "image/bmp".to_string()
    }

    /// Header import.
    fn header_import(&self, pid: &str, doc: &mut SDoc, _content_type: &str, bytes: &mut Bytes, as_name: &str) -> Result<(), SError> {
        SImage::header_import(pid, doc, bytes, as_name)
    }

    /// String import (base64 image string).
    fn string_import(&self, pid: &str, doc: &mut SDoc, src: &str, as_name: &str) -> Result<(), SError> {
        SImage::string_import(pid, doc, src, as_name)
    }

    /// File import.
    fn file_import(&self, pid: &str, doc: &mut SDoc, format: &str, full_path: &str, _extension: &str, as_name: &str) -> Result<(), SError> {
        let mut bytes = doc.fs_read_blob(pid, full_path)?;
        self.header_import(pid, doc, format, &mut bytes, as_name)
    }

    /// Export bytes.
    fn export_bytes(&self, pid: &str, doc: &SDoc, node: Option<&SNodeRef>) -> Result<Bytes, SError> {
        SImage::export_bytes(pid, doc, node)
    }

    /// Export image as base64 string.
    fn export_string(&self, pid: &str, doc: &SDoc, node: Option<&SNodeRef>) -> Result<String, SError> {
        SImage::export_string(pid, doc, node)
    }
}


/// ICO format.
pub struct ICO;
impl Format for ICO {
    /// Format getter.
    fn format(&self) -> String {
        "ico".to_string()
    }

    /// Content type (general image bytes).
    fn content_type(&self) -> String {
        "image/vnd.microsoft.icon".to_string()
    }

    /// Header import.
    fn header_import(&self, pid: &str, doc: &mut SDoc, _content_type: &str, bytes: &mut Bytes, as_name: &str) -> Result<(), SError> {
        SImage::header_import(pid, doc, bytes, as_name)
    }

    /// String import (base64 image string).
    fn string_import(&self, pid: &str, doc: &mut SDoc, src: &str, as_name: &str) -> Result<(), SError> {
        SImage::string_import(pid, doc, src, as_name)
    }

    /// File import.
    fn file_import(&self, pid: &str, doc: &mut SDoc, format: &str, full_path: &str, _extension: &str, as_name: &str) -> Result<(), SError> {
        let mut bytes = doc.fs_read_blob(pid, full_path)?;
        self.header_import(pid, doc, format, &mut bytes, as_name)
    }

    /// Export bytes.
    fn export_bytes(&self, pid: &str, doc: &SDoc, node: Option<&SNodeRef>) -> Result<Bytes, SError> {
        SImage::export_bytes(pid, doc, node)
    }

    /// Export image as base64 string.
    fn export_string(&self, pid: &str, doc: &SDoc, node: Option<&SNodeRef>) -> Result<String, SError> {
        SImage::export_string(pid, doc, node)
    }
}


#[cfg(test)]
mod tests {
    use crate::SDoc;
    use super::SImage;


    #[test]
    fn load_save_image() {
        let serialized;
        {
            let image = SImage::from_file("src/image/norse.png").expect("could not load image");
            serialized = serde_json::to_string(&image).expect("serialize error");
        }
        let mut image: SImage = serde_json::from_str(&serialized).unwrap();
        image.ensure_dynamic();
        assert!(image.dynamic_image.is_some());
    }


    #[test]
    fn stof_image_test_suite() {
        SDoc::test_file("src/image/tests.stof", true);
    }
}
