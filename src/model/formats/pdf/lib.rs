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

use std::sync::Arc;
use arcstr::{literal, ArcStr};
use imbl::{vector, OrdMap};
use serde::{Deserialize, Serialize};
use crate::{model::{pdf::pdf::Pdf, Graph, LibFunc, Param}, runtime::{instruction::{Instruction, Instructions}, proc::ProcEnv, Error, Type, Val, ValRef, Variable}};

const PDF_LIB: ArcStr = literal!("Pdf");


pub fn insert_pdf_library(graph: &mut Graph) {
    graph.insert_libfunc(LibFunc {
        library: PDF_LIB.clone(),
        name: "extract_text".into(),
        is_async: false,
        docs: r#"# Pdf.extract_text(pdf: Data\<Pdf>) -> str
Given a data pointer to a PDF document, extract all text from the PDF file and return it as a string.
```rust
// import './test_stof_pdf.pdf'; // taken from stof PDF format tests
const text = self.pdf.extract_text();
assert_eq(text, "Example Stof\nDocument\n");
```
"#.into(),
        params: vector![
            Param { name: "pdf".into(), param_type: Type::Data(PDF_LIB.clone()), default: None, },
        ],
        unbounded_args: false,
        return_type: None,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(PdfIns::ExtractText));
            Ok(instructions)
        })
    });

    graph.insert_libfunc(LibFunc {
        library: PDF_LIB.clone(),
        name: "extract_images".into(),
        is_async: false,
        docs: r#"# Pdf.extract_images(pdf: Data\<Pdf>) -> list
Given a data pointer to a PDF document, extract all images from every page, returning them as a list of maps with image data.
```rust
// import './test_stof_pdf.pdf'; // taken from stof PDF format tests
const images = self.pdf.extract_images();
assert_eq(images.len(), 1);
assert_eq(images[0].get('height'), 500);
assert_eq(images[0].get('width'), 1250);
```
"#.into(),
        params: vector![
            Param { name: "pdf".into(), param_type: Type::Data(PDF_LIB.clone()), default: None, },
        ],
        unbounded_args: false,
        return_type: None,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(PdfIns::ExtractImages));
            Ok(instructions)
        })
    });
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// PDF library functions.
enum PdfIns {
    ExtractText,
    ExtractImages,
}
#[typetag::serde(name = "PdfIns")]
impl Instruction for PdfIns {
    fn exec(&self, env: &mut ProcEnv, graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        match self {
            Self::ExtractText => {
                if let Some(var) = env.stack.pop() {
                    if let Some(dref) = var.try_data_or_func() {
                        if let Some(pdf) = graph.get_stof_data::<Pdf>(&dref) {
                            env.stack.push(Variable::val(Val::Str(pdf.extract_text().into())));
                            return Ok(None);
                        }
                    }
                }
                Err(Error::PdfExtractText)
            },
            Self::ExtractImages => {
                if let Some(var) = env.stack.pop() {
                    if let Some(dref) = var.try_data_or_func() {
                        if let Some(pdf) = graph.get_stof_data::<Pdf>(&dref) {
                            let mut list = vector![];
                            for image in pdf.extract_images() {
                                let mut map = OrdMap::default();

                                map.insert(ValRef::new(Val::Str("content".into())), ValRef::new(Val::Blob(image.content.to_vec())));
                                map.insert(ValRef::new(Val::Str("id".into())), ValRef::new(Val::Tup(vector![ValRef::new(Val::from(image.id.0)), ValRef::new(Val::from(image.id.1))])));
                                map.insert(ValRef::new(Val::Str("width".into())), ValRef::new(Val::from(image.width)));
                                map.insert(ValRef::new(Val::Str("height".into())), ValRef::new(Val::from(image.height)));
                                if let Some(color_space) = image.color_space {
                                    map.insert(ValRef::new(Val::Str("color_space".into())), ValRef::new(Val::from(color_space.as_str())));
                                }

                                list.push_back(ValRef::new(Val::Map(map)));
                            }
                            env.stack.push(Variable::val(Val::List(list)));
                            return Ok(None);
                        }
                    }
                }
                Err(Error::PdfExtractImages)
            }
        }
    }
}
