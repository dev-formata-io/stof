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

use std::{collections::BTreeMap, ops::{Deref, DerefMut}};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use crate::{lang::SError, Library, SData, SDataRef, SDoc, SVal};
use super::SPDF;


/// PDF library.
#[derive(Default, Debug)]
pub struct SPDFLibrary;
impl SPDFLibrary {
    /// Call operation.
    pub fn operate(&self, pid: &str, doc: &mut SDoc, name: &str, doc_ref: &SDataRef, parameters: &mut Vec<SVal>) -> Result<SVal, SError> {
        match name {
            /*****************************************************************************
             * Data Library Functions.
             *****************************************************************************/
            "exists" => {
                Ok(SVal::Bool(doc_ref.exists(&doc.graph)))
            },
            "objects" => {
                let mut objects = Vec::new();
                for node in doc_ref.nodes(&doc.graph) {
                    objects.push(SVal::Object(node));
                }
                Ok(SVal::Array(objects))
            },
            "id" => {
                Ok(SVal::String(doc_ref.id.clone()))
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
                Ok(SVal::Bool(doc.graph.remove_data(doc_ref.clone(), from.as_ref())))
            },
            "attach" => {
                if parameters.len() < 1 {
                    return Err(SError::data(pid, &doc, "attach", "attach must have an object argument to attach this data to"));
                }
                match &parameters[0] {
                    SVal::Object(nref) => {
                        Ok(SVal::Bool(doc.graph.put_data_ref(nref, doc_ref.clone())))
                    },
                    SVal::Boxed(val) => {
                        let val = val.lock().unwrap();
                        let val = val.deref();
                        match val {
                            SVal::Object(nref) => {
                                Ok(SVal::Bool(doc.graph.put_data_ref(nref, doc_ref.clone())))
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
                        if doc.graph.put_data_ref(to, doc_ref.clone()) {
                            if doc.graph.remove_data(doc_ref.clone(), Some(&from)) {
                                return Ok(SVal::Bool(true));
                            }
                        }
                        return Ok(SVal::Bool(false));
                    }
                }
                Err(SError::data(pid, &doc, "move", "move must have an object 'from' and an object 'to'"))
            },

            /*****************************************************************************
             * PDF Library Functions.
             *****************************************************************************/
            // Test whether the "data" is a PDF or not.
            "isPDF" => {
                if let Some(_) = SData::get::<SPDF>(&doc.graph, doc_ref) {
                    return Ok(SVal::Bool(true));
                }
                Ok(SVal::Bool(false))
            },
            "tagname" => {
                if let Some(tagname) = SData::tagname(&doc.graph, doc_ref) {
                    return Ok(SVal::String(tagname));
                }
                Ok(SVal::Null)
            },
            // Return a clone of this PDF
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

                let mut pdf = None;
                if let Some(epdf) = SData::get::<SPDF>(&doc.graph, doc_ref) {
                    pdf = Some(epdf.clone());
                }
                if let Some(pdf) = pdf {
                    if let Some(node) = node {
                        let dref = SData::insert_new(&mut doc.graph, &node, Box::new(pdf)).unwrap();
                        return Ok(SVal::Data(dref));
                    }
                }
                Ok(SVal::Null)
            },
            // Extract all images from this document.
            "extractImages" => {
                if let Some(pdf) = SData::get::<SPDF>(&doc.graph, doc_ref) {
                    let mut array = Vec::new();
                    for image in pdf.extract_images() {
                        let mut map = BTreeMap::new();

                        map.insert("content".into(), SVal::Blob(image.content.to_vec()).to_box());
                        map.insert("id".into(), SVal::Tuple(vec![SVal::from(image.id.0 as i64), SVal::from(image.id.1 as i64)]));
                        map.insert("width".into(), SVal::from(image.width));
                        map.insert("height".into(), SVal::from(image.height));
                        if let Some(color_space) = image.color_space {
                            map.insert("color_space".into(), color_space.into());
                        }
                        if let Some(filters) = image.filters {
                            map.insert("filters".into(), filters.into());
                        }
                        
                        array.push(SVal::Map(map));
                    }
                    return Ok(SVal::Array(array));
                }
                Ok(SVal::Null)
            },
            // Extract images from a single page (numbers starting at 1).
            "extractPageImages" => {
                if parameters.len() > 0 {
                    match &parameters[0] {
                        SVal::Number(num) => {
                            let page = num.int() as u32;
                            if let Some(pdf) = SData::get::<SPDF>(&doc.graph, doc_ref) {
                                if let Some(images) = pdf.extract_single_page_images(page) {
                                    let mut array = Vec::new();
                                    for image in images {
                                        let mut map = BTreeMap::new();

                                        map.insert("content".into(), SVal::Blob(image.content.to_vec()).to_box());
                                        map.insert("id".into(), SVal::Tuple(vec![SVal::from(image.id.0 as i64), SVal::from(image.id.1 as i64)]));
                                        map.insert("width".into(), SVal::from(image.width));
                                        map.insert("height".into(), SVal::from(image.height));
                                        if let Some(color_space) = image.color_space {
                                            map.insert("color_space".into(), color_space.into());
                                        }
                                        if let Some(filters) = image.filters {
                                            map.insert("filters".into(), filters.into());
                                        }
                                        
                                        array.push(SVal::Map(map));
                                    }
                                    return Ok(SVal::Array(array));
                                }
                                return Ok(SVal::Array(vec![]));
                            }
                            return Ok(SVal::Null);
                        },
                        _ => {}
                    }
                }
                Err(SError::custom(pid, &doc, "PDFLibraryExtractTextError", "expecting a page number in which to extract text from"))
            },
            // Extract all of the text from this document.
            "extractText" => {
                if let Some(pdf) = SData::get::<SPDF>(&doc.graph, doc_ref) {
                    return Ok(SVal::String(pdf.extract_text()));
                }
                Ok(SVal::String(String::default()))
            },
            // Extract text from a single page (numbers starting at 1).
            "extractPageText" => {
                if parameters.len() > 0 {
                    match &parameters[0] {
                        SVal::Number(num) => {
                            let page = num.int() as u32;
                            if let Some(pdf) = SData::get::<SPDF>(&doc.graph, doc_ref) {
                                if let Some(text) = pdf.extract_single_page_text(page) {
                                    return Ok(SVal::String(text));
                                }
                            }
                            return Ok(SVal::Null);
                        },
                        _ => {}
                    }
                }
                Err(SError::custom(pid, &doc, "PDFLibraryExtractTextError", "expecting a page number in which to extract text from"))
            },
            // Get raw PDF bytes representation.
            "blobify" |
            "blob" => {
                let mut bytes = Vec::new();
                if let Some(pdf) = SData::get::<SPDF>(&doc.graph, doc_ref) {
                    let mut mutable = pdf.doc.clone();
                    let _ = mutable.save_to(&mut bytes);
                }
                Ok(SVal::Blob(bytes))
            },
            _ => {
                Err(SError::custom(pid, &doc, "PDFLibraryNotFound", &format!("{} is not a function in the PDF Library", name)))
            }
        }
    }
}
impl Library for SPDFLibrary {
    /// Scope.
    fn scope(&self) -> String {
        "PDF".to_string()
    }
    
    /// Call into the PDF library.
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
                        return Err(SError::custom(pid, &doc, "PDFLibraryFrom", "PDF 'from' takes 1 blob or string parameter and an optional object"));
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
                                        return Err(SError::custom(pid, &doc, "PDFLibraryFrom", "PDF 'from' second parameter must be an object"));
                                    }
                                }
                            },
                            _ => {
                                return Err(SError::custom(pid, &doc, "PDFLibraryFrom", "PDF 'from' second parameter must be an object"));
                            }
                        }
                    }
                    if context.is_none() {
                        return Err(SError::custom(pid, &doc, "PDFLibraryFrom", "context not found"));
                    }

                    match parameters.pop().unwrap() {
                        SVal::Blob(bytes) => {
                            if let Ok(pdf) = SPDF::from_bytes(bytes) {
                                let node = context.unwrap();
                                let dref = SData::insert_new(&mut doc.graph, &node, Box::new(pdf)).unwrap();
                                return Ok(SVal::Data(dref));
                            }
                            return Err(SError::custom(pid, &doc, "PDFLibraryFrom", "PDF could not be parsed from bytes"));
                        },
                        SVal::String(base64) => {
                            if let Ok(bytes) = STANDARD.decode(base64) {
                                if let Ok(pdf) = SPDF::from_bytes(bytes) {
                                    let node = context.unwrap();
                                    let dref = SData::insert_new(&mut doc.graph, &node, Box::new(pdf)).unwrap();
                                    return Ok(SVal::Data(dref));
                                }
                                return Err(SError::custom(pid, &doc, "PDFLibraryFrom", "PDF could not be parsed from bytes"));
                            }
                            return Err(SError::custom(pid, &doc, "PDFLibraryFrom", "failed to decode base64 PDF string"));
                        },
                        SVal::Boxed(val) => {
                            let val = val.lock().unwrap();
                            let val = val.clone().unbox();
                            match val {
                                SVal::Blob(bytes) => {
                                    if let Ok(pdf) = SPDF::from_bytes(bytes) {
                                        let node = context.unwrap();
                                        let dref = SData::insert_new(&mut doc.graph, &node, Box::new(pdf)).unwrap();
                                        return Ok(SVal::Data(dref));
                                    }
                                    return Err(SError::custom(pid, &doc, "PDFLibraryFrom", "PDF could not be parsed from bytes"));
                                },
                                SVal::String(base64) => {
                                    if let Ok(bytes) = STANDARD.decode(base64) {
                                        if let Ok(pdf) = SPDF::from_bytes(bytes) {
                                            let node = context.unwrap();
                                            let dref = SData::insert_new(&mut doc.graph, &node, Box::new(pdf)).unwrap();
                                            return Ok(SVal::Data(dref));
                                        }
                                        return Err(SError::custom(pid, &doc, "PDFLibraryFrom", "PDF could not be parsed from bytes"));
                                    }
                                    return Err(SError::custom(pid, &doc, "PDFLibraryFrom", "failed to decode base64 PDF string"));
                                },
                                _ => {
                                    return Err(SError::custom(pid, &doc, "PDFLibraryFrom", "PDF 'from' must be from a blob or base64 encoded string"))
                                }
                            }
                        },
                        _ => {
                            return Err(SError::custom(pid, &doc, "PDFLibraryFrom", "PDF 'from' must be from a blob or base64 encoded string"))
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
                            return Err(SError::custom(pid, &doc, "PDFInvalidArgument", "PDF argument not found"));
                        }
                    }
                },
                _ => {
                    return Err(SError::custom(pid, &doc, "PDFInvalidArgument", "PDF argument not found"));
                }
            }
        } else {
            return Err(SError::custom(pid, &doc, "PDFInvalidArgument", "PDF argument not found"));
        }
    }
}
