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

use std::ops::{Deref, DerefMut};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use image::imageops::FilterType;
use crate::{lang::SError, Library, SData, SDataRef, SDoc, SNum, SVal};
use super::SImage;


/// Image library.
#[derive(Default, Debug)]
pub struct SImageLibrary;
impl SImageLibrary {
    /// Call image operation.
    pub fn operate(&self, pid: &str, doc: &mut SDoc, name: &str, image_ref: &SDataRef, parameters: &mut Vec<SVal>) -> Result<SVal, SError> {
        match name {
            /*****************************************************************************
             * Data Library Functions.
             *****************************************************************************/
            "exists" => {
                Ok(SVal::Bool(image_ref.exists(&doc.graph)))
            },
            "objects" => {
                let mut objects = Vec::new();
                for node in image_ref.nodes(&doc.graph) {
                    objects.push(SVal::Object(node));
                }
                Ok(SVal::Array(objects))
            },
            "id" => {
                Ok(SVal::String(image_ref.id.clone()))
            },
            "drop" => {
                let mut from = None;
                if parameters.len() > 0 {
                    match &parameters[0] {
                        SVal::Object(nref) => {
                            from = Some(nref.clone());
                        },
                        SVal::Boxed(val) => {
                            let val = val.lock().unwrap();
                            let val = val.deref();
                            match val {
                                SVal::Object(nref) => {
                                    from = Some(nref.clone());
                                },
                                _ => {
                                    return Err(SError::data(pid, &doc, "drop", "cannot drop from anything other than an object"));
                                }
                            }
                        },
                        _ => {
                            return Err(SError::data(pid, &doc, "drop", "cannot drop from anything other than an object"));
                        }
                    }
                }
                Ok(SVal::Bool(doc.graph.remove_data(image_ref.clone(), from.as_ref())))
            },
            "attach" => {
                if parameters.len() < 1 {
                    return Err(SError::data(pid, &doc, "attach", "attach must have an object argument to attach this data to"));
                }
                match &parameters[0] {
                    SVal::Object(nref) => {
                        Ok(SVal::Bool(doc.graph.put_data_ref(nref, image_ref.clone())))
                    },
                    SVal::Boxed(val) => {
                        let val = val.lock().unwrap();
                        let val = val.deref();
                        match val {
                            SVal::Object(nref) => {
                                Ok(SVal::Bool(doc.graph.put_data_ref(nref, image_ref.clone())))
                            },
                            _ => {
                                Err(SError::data(pid, &doc, "attach", "attach must have an object argument to attach this data to"))
                            }
                        }
                    },
                    _ => {
                        Err(SError::data(pid, &doc, "attach", "attach must have an object argument to attach this data to"))
                    }
                }
            },
            "move" => {
                if parameters.len() < 2 {
                    return Err(SError::data(pid, &doc, "move", "move must have an object 'from' and an object 'to'"));
                }
                
                let from;
                let to;
                match &parameters[0] {
                    SVal::Object(nref) => {
                        from = Some(nref.clone());
                    },
                    SVal::Boxed(val) => {
                        let val = val.lock().unwrap();
                        let val = val.deref();
                        match val {
                            SVal::Object(nref) => {
                                from = Some(nref.clone());
                            },
                            _ => {
                                return Err(SError::data(pid, &doc, "move", "move 'from' must be an object"));
                            }
                        }
                    },
                    _ => {
                        return Err(SError::data(pid, &doc, "move", "move 'from' must be an object"));
                    }
                }
                match &parameters[1] {
                    SVal::Object(nref) => {
                        to = Some(nref.clone());
                    },
                    SVal::Boxed(val) => {
                        let val = val.lock().unwrap();
                        let val = val.deref();
                        match val {
                            SVal::Object(nref) => {
                                to = Some(nref.clone());
                            },
                            _ => {
                                return Err(SError::data(pid, &doc, "move", "move 'to' must be an object"));
                            }
                        }
                    },
                    _ => {
                        return Err(SError::data(pid, &doc, "move", "move 'to' must be an object"));
                    }
                }
                
                if let Some(from) = from {
                    if let Some(to) = to {
                        if doc.graph.put_data_ref(to, image_ref.clone()) {
                            if doc.graph.remove_data(image_ref.clone(), Some(&from)) {
                                return Ok(SVal::Bool(true));
                            }
                        }
                        return Ok(SVal::Bool(false));
                    }
                }
                Err(SError::data(pid, &doc, "move", "move must have an object 'from' and an object 'to'"))
            },

            /*****************************************************************************
             * Image Library Functions.
             *****************************************************************************/
            // Test whether the "data" is an image or not.
            "isImage" => {
                if let Some(_) = SData::get::<SImage>(&doc.graph, image_ref) {
                    return Ok(SVal::Bool(true));
                }
                Ok(SVal::Bool(false))
            },
            "tagname" => {
                if let Some(tagname) = SData::tagname(&doc.graph, image_ref) {
                    return Ok(SVal::String(tagname));
                }
                Ok(SVal::Null)
            },
            // Return a clone of this image
            "clone" => {
                let mut node = doc.self_ptr(pid);
                if parameters.len() > 0 {
                    match &parameters[0] {
                        SVal::Object(nref) => {
                            node = Some(nref.clone());
                        },
                        SVal::Boxed(val) => {
                            let val = val.lock().unwrap();
                            let val = val.deref();
                            match val {
                                SVal::Object(nref) => {
                                    node = Some(nref.clone());
                                },
                                _ => {}
                            }
                        },
                        _ => {}
                    }
                }

                let mut image = None;
                if let Some(eimage) = SData::get::<SImage>(&doc.graph, image_ref) {
                    image = Some(eimage.clone());
                }
                if let Some(image) = image {
                    if let Some(node) = node {
                        let dref = SData::insert_new(&mut doc.graph, &node, Box::new(image)).unwrap();
                        return Ok(SVal::Data(dref));
                    }
                }
                Ok(SVal::Null)
            },
            "width" => {
                if let Some(image) = SData::get_mut::<SImage>(&mut doc.graph, image_ref) {
                    let val = image.width();
                    return Ok(SVal::Number(SNum::I64(val as i64)));
                }
                Ok(SVal::Null)
            },
            "height" => {
                if let Some(image) = SData::get_mut::<SImage>(&mut doc.graph, image_ref) {
                    let val = image.height();
                    return Ok(SVal::Number(SNum::I64(val as i64)));
                }
                Ok(SVal::Null)
            },
            // Image modifiers.
            "grayscale" => {
                if let Some(image) = SData::get_mut::<SImage>(&mut doc.graph, image_ref) {
                    image.grayscale();
                    Ok(SVal::Bool(true))
                } else {
                    Ok(SVal::Bool(false))
                }
            },
            "invert" => {
                if let Some(image) = SData::get_mut::<SImage>(&mut doc.graph, image_ref) {
                    image.invert();
                    Ok(SVal::Bool(true))
                } else {
                    Ok(SVal::Bool(false))
                }
            },
            "flipVertical" => {
                if let Some(image) = SData::get_mut::<SImage>(&mut doc.graph, image_ref) {
                    image.flip_vertical();
                    Ok(SVal::Bool(true))
                } else {
                    Ok(SVal::Bool(false))
                }
            },
            "flipHorizontal" => {
                if let Some(image) = SData::get_mut::<SImage>(&mut doc.graph, image_ref) {
                    image.flip_horizontal();
                    Ok(SVal::Bool(true))
                } else {
                    Ok(SVal::Bool(false))
                }
            },
            "rotate90" => {
                if let Some(image) = SData::get_mut::<SImage>(&mut doc.graph, image_ref) {
                    image.rotate_90();
                    Ok(SVal::Bool(true))
                } else {
                    Ok(SVal::Bool(false))
                }
            },
            "rotate180" => {
                if let Some(image) = SData::get_mut::<SImage>(&mut doc.graph, image_ref) {
                    image.rotate_180();
                    Ok(SVal::Bool(true))
                } else {
                    Ok(SVal::Bool(false))
                }
            },
            "rotate270" => {
                if let Some(image) = SData::get_mut::<SImage>(&mut doc.graph, image_ref) {
                    image.rotate_270();
                    Ok(SVal::Bool(true))
                } else {
                    Ok(SVal::Bool(false))
                }
            },
            "resize" => {
                if parameters.len() > 1 {
                    let mut width = 0;
                    let mut height = 0;
                    match &parameters[0] {
                        SVal::Number(num) => {
                            width = num.int() as u32;
                        },
                        _ => {}
                    }
                    match &parameters[1] {
                        SVal::Number(num) => {
                            height = num.int() as u32;
                        },
                        _ => {}
                    }
                    if width > 0 && height > 0 {
                        if let Some(image) = SData::get_mut::<SImage>(&mut doc.graph, image_ref) {
                            image.resize(width, height, FilterType::CatmullRom); // balanced speed and look
                            return Ok(SVal::Bool(true));
                        }
                    }
                    return Ok(SVal::Bool(false));
                }
                Err(SError::custom(pid, &doc, "ImageLibraryResize", "resize takes a width integer and a height integer"))
            },
            "resizeExact" => {
                if parameters.len() > 1 {
                    let mut width = 0;
                    let mut height = 0;
                    match &parameters[0] {
                        SVal::Number(num) => {
                            width = num.int() as u32;
                        },
                        _ => {}
                    }
                    match &parameters[1] {
                        SVal::Number(num) => {
                            height = num.int() as u32;
                        },
                        _ => {}
                    }
                    if width > 0 && height > 0 {
                        if let Some(image) = SData::get_mut::<SImage>(&mut doc.graph, image_ref) {
                            image.resize_exact(width, height, FilterType::CatmullRom); // balanced speed and look
                            return Ok(SVal::Bool(true));
                        }
                    }
                    return Ok(SVal::Bool(false));
                }
                Err(SError::custom(pid, &doc, "ImageLibraryResize", "resize takes a width integer and a height integer"))
            },
            "thumbnail" => {
                if parameters.len() > 1 {
                    let mut width = 0;
                    let mut height = 0;
                    match &parameters[0] {
                        SVal::Number(num) => {
                            width = num.int() as u32;
                        },
                        _ => {}
                    }
                    match &parameters[1] {
                        SVal::Number(num) => {
                            height = num.int() as u32;
                        },
                        _ => {}
                    }
                    if width > 0 && height > 0 {
                        if let Some(image) = SData::get_mut::<SImage>(&mut doc.graph, image_ref) {
                            image.thumbnail(width, height);
                            return Ok(SVal::Bool(true));
                        }
                    }
                    return Ok(SVal::Bool(false));
                }
                Err(SError::custom(pid, &doc, "ImageLibraryThumbnail", "thumbnail takes a width integer and a height integer"))
            },
            "thumbnailExact" => {
                if parameters.len() > 1 {
                    let mut width = 0;
                    let mut height = 0;
                    match &parameters[0] {
                        SVal::Number(num) => {
                            width = num.int() as u32;
                        },
                        _ => {}
                    }
                    match &parameters[1] {
                        SVal::Number(num) => {
                            height = num.int() as u32;
                        },
                        _ => {}
                    }
                    if width > 0 && height > 0 {
                        if let Some(image) = SData::get_mut::<SImage>(&mut doc.graph, image_ref) {
                            image.thumbnail_exact(width, height);
                            return Ok(SVal::Bool(true));
                        }
                    }
                    return Ok(SVal::Bool(false));
                }
                Err(SError::custom(pid, &doc, "ImageLibraryThumbnail", "thumbnail takes a width integer and a height integer"))
            },
            "blur" => {
                if parameters.len() > 0 {
                    let mut val = 0.0;
                    match &parameters[0] {
                        SVal::Number(num) => {
                            val = num.float() as f32;
                        },
                        _ => {}
                    }
                    if let Some(image) = SData::get_mut::<SImage>(&mut doc.graph, image_ref) {
                        image.blur(val);
                        return Ok(SVal::Bool(true));
                    }
                    return Ok(SVal::Bool(false));
                }
                Err(SError::custom(pid, &doc, "ImageLibraryBlur", "expecting an f32 parameter"))
            },
            "blurFast" => {
                if parameters.len() > 0 {
                    let mut val = 0.0;
                    match &parameters[0] {
                        SVal::Number(num) => {
                            val = num.float() as f32;
                        },
                        _ => {}
                    }
                    if let Some(image) = SData::get_mut::<SImage>(&mut doc.graph, image_ref) {
                        image.fast_blur(val);
                        return Ok(SVal::Bool(true));
                    }
                    return Ok(SVal::Bool(false));
                }
                Err(SError::custom(pid, &doc, "ImageLibraryBlurFast", "expecting an f32 parameter"))
            },
            "adjustContrast" => {
                if parameters.len() > 0 {
                    let mut val = 0.0;
                    match &parameters[0] {
                        SVal::Number(num) => {
                            val = num.float() as f32;
                        },
                        _ => {}
                    }
                    if let Some(image) = SData::get_mut::<SImage>(&mut doc.graph, image_ref) {
                        image.adjust_contrast(val);
                        return Ok(SVal::Bool(true));
                    }
                    return Ok(SVal::Bool(false));
                }
                Err(SError::custom(pid, &doc, "ImageLibraryAdjustContrast", "expecting an f32 parameter"))
            },
            "brighten" => {
                if parameters.len() > 0 {
                    let mut val = 0;
                    match &parameters[0] {
                        SVal::Number(num) => {
                            val = num.int() as i32;
                        },
                        _ => {}
                    }
                    if let Some(image) = SData::get_mut::<SImage>(&mut doc.graph, image_ref) {
                        image.brighten(val);
                        return Ok(SVal::Bool(true));
                    }
                    return Ok(SVal::Bool(false));
                }
                Err(SError::custom(pid, &doc, "ImageLibraryBrighten", "expecting an i32 parameter"))
            },
            // Get raw image representation. This is PNG bytes by default.
            "blobify" |
            "blob" => {
                let mut bytes = Vec::new();
                if let Some(image) = SData::get::<SImage>(&doc.graph, image_ref) {
                    bytes = image.raw_image.clone();
                }
                Ok(SVal::Blob(bytes))
            },
            "png" => {
                if let Some(image) = SData::get_mut::<SImage>(&mut doc.graph, image_ref) {
                    if let Some(bytes) = image.png_bytes() {
                        return Ok(SVal::Blob(bytes));
                    }
                }
                Ok(SVal::Null)
            },
            "jpg" |
            "jpeg" => {
                if let Some(image) = SData::get_mut::<SImage>(&mut doc.graph, image_ref) {
                    if let Some(bytes) = image.jpeg_bytes() {
                        return Ok(SVal::Blob(bytes));
                    }
                }
                Ok(SVal::Null)
            },
            "gif" => {
                if let Some(image) = SData::get_mut::<SImage>(&mut doc.graph, image_ref) {
                    if let Some(bytes) = image.gif_bytes() {
                        return Ok(SVal::Blob(bytes));
                    }
                }
                Ok(SVal::Null)
            },
            "webp" => {
                if let Some(image) = SData::get_mut::<SImage>(&mut doc.graph, image_ref) {
                    if let Some(bytes) = image.webp_bytes() {
                        return Ok(SVal::Blob(bytes));
                    }
                }
                Ok(SVal::Null)
            },
            "tiff" |
            "tif" => {
                if let Some(image) = SData::get_mut::<SImage>(&mut doc.graph, image_ref) {
                    if let Some(bytes) = image.tiff_bytes() {
                        return Ok(SVal::Blob(bytes));
                    }
                }
                Ok(SVal::Null)
            },
            "bmp" => {
                if let Some(image) = SData::get_mut::<SImage>(&mut doc.graph, image_ref) {
                    if let Some(bytes) = image.bmp_bytes() {
                        return Ok(SVal::Blob(bytes));
                    }
                }
                Ok(SVal::Null)
            },
            "ico" => {
                if let Some(image) = SData::get_mut::<SImage>(&mut doc.graph, image_ref) {
                    if let Some(bytes) = image.ico_bytes() {
                        return Ok(SVal::Blob(bytes));
                    }
                }
                Ok(SVal::Null)
            },
            _ => {
                Err(SError::custom(pid, &doc, "ImageLibraryNotFound", &format!("{} is not a function in the Image Library", name)))
            }
        }
    }
}
impl Library for SImageLibrary {
    /// Scope.
    fn scope(&self) -> String {
        "Image".to_string()
    }
    
    /// Call into the Image library.
    fn call(&self, pid: &str, doc: &mut SDoc, name: &str, parameters: &mut Vec<SVal>) -> Result<SVal, SError> {
        if parameters.len() > 0 {
            match name {
                "toString" => {
                    return Ok(SVal::String(parameters[0].print(doc)));
                },
                "or" => {
                    for param in parameters.drain(..) {
                        if !param.is_empty() {
                            return Ok(param);
                        }
                    }
                    return Ok(SVal::Null);
                },
                "from" => {
                    if parameters.len() < 1 {
                        return Err(SError::custom(pid, &doc, "ImageLibraryFrom", "image 'from' takes 1 blob or string parameter and an optional object"));
                    }

                    let mut context = doc.self_ptr(pid);
                    if parameters.len() > 1 {
                        match parameters.pop().unwrap() {
                            SVal::Object(nref) => {
                                context = Some(nref);
                            },
                            SVal::Boxed(val) => {
                                let val = val.lock().unwrap();
                                let val = val.deref();
                                match val {
                                    SVal::Object(nref) => {
                                        context = Some(nref.clone());
                                    },
                                    _ => {
                                        return Err(SError::custom(pid, &doc, "ImageLibraryFrom", "image 'from' second parameter must be an object"));
                                    }
                                }
                            },
                            _ => {
                                return Err(SError::custom(pid, &doc, "ImageLibraryFrom", "image 'from' second parameter must be an object"));
                            }
                        }
                    }
                    if context.is_none() {
                        return Err(SError::custom(pid, &doc, "ImageLibraryFrom", "context not found"));
                    }

                    match parameters.pop().unwrap() {
                        SVal::Blob(bytes) => {
                            if let Ok(image) = SImage::from_bytes(bytes) {
                                let node = context.unwrap();
                                let dref = SData::insert_new(&mut doc.graph, &node, Box::new(image)).unwrap();
                                return Ok(SVal::Data(dref));
                            }
                            return Err(SError::custom(pid, &doc, "ImageLibraryFrom", "image could not be parsed from bytes"));
                        },
                        SVal::String(base64) => {
                            if let Ok(bytes) = STANDARD.decode(base64) {
                                if let Ok(image) = SImage::from_bytes(bytes) {
                                    let node = context.unwrap();
                                    let dref = SData::insert_new(&mut doc.graph, &node, Box::new(image)).unwrap();
                                    return Ok(SVal::Data(dref));
                                }
                                return Err(SError::custom(pid, &doc, "ImageLibraryFrom", "image could not be parsed from bytes"));
                            }
                            return Err(SError::custom(pid, &doc, "ImageLibraryFrom", "failed to decode base64 image string"));
                        },
                        SVal::Boxed(val) => {
                            let val = val.lock().unwrap();
                            let val = val.clone().unbox();
                            match val {
                                SVal::Blob(bytes) => {
                                    if let Ok(image) = SImage::from_bytes(bytes) {
                                        let node = context.unwrap();
                                        let dref = SData::insert_new(&mut doc.graph, &node, Box::new(image)).unwrap();
                                        return Ok(SVal::Data(dref));
                                    }
                                    return Err(SError::custom(pid, &doc, "ImageLibraryFrom", "image could not be parsed from bytes"));
                                },
                                SVal::String(base64) => {
                                    if let Ok(bytes) = STANDARD.decode(base64) {
                                        if let Ok(image) = SImage::from_bytes(bytes) {
                                            let node = context.unwrap();
                                            let dref = SData::insert_new(&mut doc.graph, &node, Box::new(image)).unwrap();
                                            return Ok(SVal::Data(dref));
                                        }
                                        return Err(SError::custom(pid, &doc, "ImageLibraryFrom", "image could not be parsed from bytes"));
                                    }
                                    return Err(SError::custom(pid, &doc, "ImageLibraryFrom", "failed to decode base64 image string"));
                                },
                                _ => {
                                    return Err(SError::custom(pid, &doc, "ImageLibraryFrom", "image 'from' must be from a blob or base64 encoded string"))
                                }
                            }
                        },
                        _ => {
                            return Err(SError::custom(pid, &doc, "ImageLibraryFrom", "image 'from' must be from a blob or base64 encoded string"))
                        }
                    }
                },
                _ => {}
            }

            let mut params;
            if parameters.len() > 1 {
                params = parameters.drain(1..).collect();
            } else {
                params = Vec::new();
            }
            match &mut parameters[0] {
                SVal::Data(val) => {
                    return self.operate(pid, doc, name, val, &mut params);
                },
                SVal::Boxed(val) => {
                    let mut val = val.lock().unwrap();
                    let val = val.deref_mut();
                    match val {
                        SVal::Data(val) => {
                            return self.operate(pid, doc, name, val, &mut params);
                        },
                        _ => {
                            return Err(SError::custom(pid, &doc, "ImageInvalidArgument", "image argument not found"));
                        }
                    }
                },
                _ => {
                    return Err(SError::custom(pid, &doc, "ImageInvalidArgument", "image argument not found"));
                }
            }
        } else {
            return Err(SError::custom(pid, &doc, "ImageInvalidArgument", "image argument not found"));
        }
    }
}
