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

use std::{ops::Deref, sync::Arc};
use arcstr::{literal, ArcStr};
use image::imageops::FilterType;
use imbl::vector;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use crate::{model::{image::image::Image, Graph, LibFunc, Param}, runtime::{instruction::{Instruction, Instructions}, proc::ProcEnv, Error, NumT, Type, Val, Variable}};


const IMAGE_LIB: ArcStr = literal!("Image");
pub fn insert_image_library(graph: &mut Graph) {
    // Width
    graph.insert_libfunc(LibFunc {
        library: IMAGE_LIB.clone(),
        name: "width".into(),
        is_async: false,
        docs: r#"# Image.width(img: Data\<Image>) -> int
Width in pixels of this image.
"#.into(),
        params: vector![
            Param { name: "img".into(), param_type: Type::Data(IMAGE_LIB.clone()), default: None, },
        ],
        unbounded_args: false,
        return_type: None,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(ImageIns::Width));
            Ok(instructions)
        })
    });

    // Height
    graph.insert_libfunc(LibFunc {
        library: IMAGE_LIB.clone(),
        name: "height".into(),
        is_async: false,
        docs: r#"# Image.height(img: Data\<Image>) -> int
Height in pixels of this image.
"#.into(),
        params: vector![
            Param { name: "img".into(), param_type: Type::Data(IMAGE_LIB.clone()), default: None, },
        ],
        unbounded_args: false,
        return_type: None,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(ImageIns::Height));
            Ok(instructions)
        })
    });

    // Grayscale
    graph.insert_libfunc(LibFunc {
        library: IMAGE_LIB.clone(),
        name: "grayscale".into(),
        is_async: false,
        docs: r#"# Image.grayscale(img: Data\<Image>) -> void
Turn this image into grayscale.
"#.into(),
        params: vector![
            Param { name: "img".into(), param_type: Type::Data(IMAGE_LIB.clone()), default: None, },
        ],
        unbounded_args: false,
        return_type: None,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(ImageIns::Grayscale));
            Ok(instructions)
        })
    });

    // Invert
    graph.insert_libfunc(LibFunc {
        library: IMAGE_LIB.clone(),
        name: "invert".into(),
        is_async: false,
        docs: r#"# Image.invert(img: Data\<Image>) -> void
Invert this image.
"#.into(),
        params: vector![
            Param { name: "img".into(), param_type: Type::Data(IMAGE_LIB.clone()), default: None, },
        ],
        unbounded_args: false,
        return_type: None,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(ImageIns::Invert));
            Ok(instructions)
        })
    });

    // Flip vertical
    graph.insert_libfunc(LibFunc {
        library: IMAGE_LIB.clone(),
        name: "flip_vertical".into(),
        is_async: false,
        docs: r#"# Image.flip_vertical(img: Data\<Image>) -> void
Flip this image vertically.
"#.into(),
        params: vector![
            Param { name: "img".into(), param_type: Type::Data(IMAGE_LIB.clone()), default: None, },
        ],
        unbounded_args: false,
        return_type: None,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(ImageIns::FlipVertical));
            Ok(instructions)
        })
    });

    // Flip horizontal
    graph.insert_libfunc(LibFunc {
        library: IMAGE_LIB.clone(),
        name: "flip_horizontal".into(),
        is_async: false,
        docs: r#"# Image.flip_horizontal(img: Data\<Image>) -> void
Flip this image horizontally.
"#.into(),
        params: vector![
            Param { name: "img".into(), param_type: Type::Data(IMAGE_LIB.clone()), default: None, },
        ],
        unbounded_args: false,
        return_type: None,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(ImageIns::FlipHorizontal));
            Ok(instructions)
        })
    });

    // Rotate 90
    graph.insert_libfunc(LibFunc {
        library: IMAGE_LIB.clone(),
        name: "rotate_90".into(),
        is_async: false,
        docs: r#"# Image.rotate_90(img: Data\<Image>) -> void
Rotate this image 90 degrees clockwise.
"#.into(),
        params: vector![
            Param { name: "img".into(), param_type: Type::Data(IMAGE_LIB.clone()), default: None, },
        ],
        unbounded_args: false,
        return_type: None,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(ImageIns::Rotate90));
            Ok(instructions)
        })
    });

    // Rotate 180
    graph.insert_libfunc(LibFunc {
        library: IMAGE_LIB.clone(),
        name: "rotate_180".into(),
        is_async: false,
        docs: r#"# Image.rotate_180(img: Data\<Image>) -> void
Rotate this image 180 degrees clockwise.
"#.into(),
        params: vector![
            Param { name: "img".into(), param_type: Type::Data(IMAGE_LIB.clone()), default: None, },
        ],
        unbounded_args: false,
        return_type: None,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(ImageIns::Rotate180));
            Ok(instructions)
        })
    });

    // Rotate 270
    graph.insert_libfunc(LibFunc {
        library: IMAGE_LIB.clone(),
        name: "rotate_270".into(),
        is_async: false,
        docs: r#"# Image.rotate_270(img: Data\<Image>) -> void
Rotate this image 270 degrees clockwise.
"#.into(),
        params: vector![
            Param { name: "img".into(), param_type: Type::Data(IMAGE_LIB.clone()), default: None, },
        ],
        unbounded_args: false,
        return_type: None,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(ImageIns::Rotate270));
            Ok(instructions)
        })
    });

    // Resize
    graph.insert_libfunc(LibFunc {
        library: IMAGE_LIB.clone(),
        name: "resize".into(),
        is_async: false,
        docs: r#"# Image.resize(img: Data\<Image>, width: int, height: int) -> bool
Resize this image, preserving it's aspect ratio. Will return true if the image was successfully resized.
"#.into(),
        params: vector![
            Param { name: "img".into(), param_type: Type::Data(IMAGE_LIB.clone()), default: None, },
            Param { name: "width".into(), param_type: Type::Num(NumT::Int), default: None, },
            Param { name: "height".into(), param_type: Type::Num(NumT::Int), default: None, },
        ],
        unbounded_args: false,
        return_type: None,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(ImageIns::Resize));
            Ok(instructions)
        })
    });

    // Resize exact
    graph.insert_libfunc(LibFunc {
        library: IMAGE_LIB.clone(),
        name: "resize_exact".into(),
        is_async: false,
        docs: r#"# Image.resize_exact(img: Data\<Image>, width: int, height: int) -> bool
Resize this image, without preserving it's aspect ratio. Will return true if the image was successfully resized.
"#.into(),
        params: vector![
            Param { name: "img".into(), param_type: Type::Data(IMAGE_LIB.clone()), default: None, },
            Param { name: "width".into(), param_type: Type::Num(NumT::Int), default: None, },
            Param { name: "height".into(), param_type: Type::Num(NumT::Int), default: None, },
        ],
        unbounded_args: false,
        return_type: None,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(ImageIns::ResizeExact));
            Ok(instructions)
        })
    });

    // Thumbnail
    graph.insert_libfunc(LibFunc {
        library: IMAGE_LIB.clone(),
        name: "thumbnail".into(),
        is_async: false,
        docs: r#"# Image.thumbnail(img: Data\<Image>, width: int, height: int) -> bool
Resize this image into a thumbnail, preserving it's aspect ratio. Will return true if the image was successfully resized.
"#.into(),
        params: vector![
            Param { name: "img".into(), param_type: Type::Data(IMAGE_LIB.clone()), default: None, },
            Param { name: "width".into(), param_type: Type::Num(NumT::Int), default: None, },
            Param { name: "height".into(), param_type: Type::Num(NumT::Int), default: None, },
        ],
        unbounded_args: false,
        return_type: None,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(ImageIns::Thumbnail));
            Ok(instructions)
        })
    });

    // Thumbnail exact
    graph.insert_libfunc(LibFunc {
        library: IMAGE_LIB.clone(),
        name: "thumbnail_exact".into(),
        is_async: false,
        docs: r#"# Image.thumbnail_exact(img: Data\<Image>, width: int, height: int) -> bool
Resize this image into a thumbnail, without preserving it's aspect ratio. Will return true if the image was successfully resized.
"#.into(),
        params: vector![
            Param { name: "img".into(), param_type: Type::Data(IMAGE_LIB.clone()), default: None, },
            Param { name: "width".into(), param_type: Type::Num(NumT::Int), default: None, },
            Param { name: "height".into(), param_type: Type::Num(NumT::Int), default: None, },
        ],
        unbounded_args: false,
        return_type: None,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(ImageIns::ThumbnailExact));
            Ok(instructions)
        })
    });

    // Blur
    graph.insert_libfunc(LibFunc {
        library: IMAGE_LIB.clone(),
        name: "blur".into(),
        is_async: false,
        docs: r#"# Image.blur(img: Data\<Image>, blur: float) -> void
Blur this image with the given blur value (sigma in a gaussian blur).
"#.into(),
        params: vector![
            Param { name: "img".into(), param_type: Type::Data(IMAGE_LIB.clone()), default: None, },
            Param { name: "blur".into(), param_type: Type::Num(NumT::Float), default: None, },
        ],
        unbounded_args: false,
        return_type: None,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(ImageIns::Blur));
            Ok(instructions)
        })
    });

    // Blur fast
    graph.insert_libfunc(LibFunc {
        library: IMAGE_LIB.clone(),
        name: "fast_blur".into(),
        is_async: false,
        docs: r#"# Image.fast_blur(img: Data\<Image>, blur: float) -> void
Blur this image with the given blur value (sigma in a gaussian blur).
"#.into(),
        params: vector![
            Param { name: "img".into(), param_type: Type::Data(IMAGE_LIB.clone()), default: None, },
            Param { name: "blur".into(), param_type: Type::Num(NumT::Float), default: None, },
        ],
        unbounded_args: false,
        return_type: None,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(ImageIns::BlurFast));
            Ok(instructions)
        })
    });

    // Adjust contrast
    graph.insert_libfunc(LibFunc {
        library: IMAGE_LIB.clone(),
        name: "contrast".into(),
        is_async: false,
        docs: r#"# Image.contrast(img: Data\<Image>, contrast: float) -> void
Set the contrast of this image (positive to increase, negative to decrease).
"#.into(),
        params: vector![
            Param { name: "img".into(), param_type: Type::Data(IMAGE_LIB.clone()), default: None, },
            Param { name: "contrast".into(), param_type: Type::Num(NumT::Float), default: None, },
        ],
        unbounded_args: false,
        return_type: None,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(ImageIns::Contrast));
            Ok(instructions)
        })
    });

    // Brighten
    graph.insert_libfunc(LibFunc {
        library: IMAGE_LIB.clone(),
        name: "brighten".into(),
        is_async: false,
        docs: r#"# Image.brighten(img: Data\<Image>, brighten: int) -> void
Brighten this image with the given value (positive to increase each pixel and negative to decrease).
"#.into(),
        params: vector![
            Param { name: "img".into(), param_type: Type::Data(IMAGE_LIB.clone()), default: None, },
            Param { name: "brighten".into(), param_type: Type::Num(NumT::Int), default: None, },
        ],
        unbounded_args: false,
        return_type: None,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(ImageIns::Brighten));
            Ok(instructions)
        })
    });

    // Blob
    graph.insert_libfunc(LibFunc {
        library: IMAGE_LIB.clone(),
        name: "blob".into(),
        is_async: false,
        docs: r#"# Image.blob(img: Data\<Image>) -> blob
Transform this image into a binary blob value (raw binary is a PNG).
"#.into(),
        params: vector![
            Param { name: "img".into(), param_type: Type::Data(IMAGE_LIB.clone()), default: None, },
        ],
        unbounded_args: false,
        return_type: None,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(ImageIns::Blob));
            Ok(instructions)
        })
    });

    // Png
    graph.insert_libfunc(LibFunc {
        library: IMAGE_LIB.clone(),
        name: "png".into(),
        is_async: false,
        docs: r#"# Image.png(img: Data\<Image>) -> blob
Transform this image into a binary blob value that is in PNG format.
"#.into(),
        params: vector![
            Param { name: "img".into(), param_type: Type::Data(IMAGE_LIB.clone()), default: None, },
        ],
        unbounded_args: false,
        return_type: None,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(ImageIns::Png));
            Ok(instructions)
        })
    });

    // Jpeg
    graph.insert_libfunc(LibFunc {
        library: IMAGE_LIB.clone(),
        name: "jpeg".into(),
        is_async: false,
        docs: r#"# Image.jpeg(img: Data\<Image>) -> blob
Transform this image into a binary blob value that is in JPEG format.
"#.into(),
        params: vector![
            Param { name: "img".into(), param_type: Type::Data(IMAGE_LIB.clone()), default: None, },
        ],
        unbounded_args: false,
        return_type: None,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(ImageIns::Jpeg));
            Ok(instructions)
        })
    });

    // Gif
    graph.insert_libfunc(LibFunc {
        library: IMAGE_LIB.clone(),
        name: "gif".into(),
        is_async: false,
        docs: r#"# Image.gif(img: Data\<Image>) -> blob
Transform this image into a binary blob value that is in GIF format.
"#.into(),
        params: vector![
            Param { name: "img".into(), param_type: Type::Data(IMAGE_LIB.clone()), default: None, },
        ],
        unbounded_args: false,
        return_type: None,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(ImageIns::Gif));
            Ok(instructions)
        })
    });

    // Webp
    graph.insert_libfunc(LibFunc {
        library: IMAGE_LIB.clone(),
        name: "webp".into(),
        is_async: false,
        docs: r#"# Image.webp(img: Data\<Image>) -> blob
Transform this image into a binary blob value that is in WEBP format.
"#.into(),
        params: vector![
            Param { name: "img".into(), param_type: Type::Data(IMAGE_LIB.clone()), default: None, },
        ],
        unbounded_args: false,
        return_type: None,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(ImageIns::Webp));
            Ok(instructions)
        })
    });

    // Tiff
    graph.insert_libfunc(LibFunc {
        library: IMAGE_LIB.clone(),
        name: "tiff".into(),
        is_async: false,
        docs: r#"# Image.tiff(img: Data\<Image>) -> blob
Transform this image into a binary blob value that is in TIFF format.
"#.into(),
        params: vector![
            Param { name: "img".into(), param_type: Type::Data(IMAGE_LIB.clone()), default: None, },
        ],
        unbounded_args: false,
        return_type: None,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(ImageIns::Tiff));
            Ok(instructions)
        })
    });

    // Bmp
    graph.insert_libfunc(LibFunc {
        library: IMAGE_LIB.clone(),
        name: "bmp".into(),
        is_async: false,
        docs: r#"# Image.bmp(img: Data\<Image>) -> blob
Transform this image into a binary blob value that is in BMP format.
"#.into(),
        params: vector![
            Param { name: "img".into(), param_type: Type::Data(IMAGE_LIB.clone()), default: None, },
        ],
        unbounded_args: false,
        return_type: None,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(ImageIns::Bmp));
            Ok(instructions)
        })
    });

    // Ico
    graph.insert_libfunc(LibFunc {
        library: IMAGE_LIB.clone(),
        name: "ico".into(),
        is_async: false,
        docs: r#"# Image.ico(img: Data\<Image>) -> blob
Transform this image into a binary blob value that is in ICO format.
"#.into(),
        params: vector![
            Param { name: "img".into(), param_type: Type::Data(IMAGE_LIB.clone()), default: None, },
        ],
        unbounded_args: false,
        return_type: None,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(ImageIns::Ico));
            Ok(instructions)
        })
    });

    // From blob
    graph.insert_libfunc(LibFunc {
        library: IMAGE_LIB.clone(),
        name: "from_blob".into(),
        is_async: false,
        docs: r#"# Image.from_blob(bytes: blob) -> Data\<Image>
Create an image (on the calling/current object) given a binary blob, attempting to auto-detect the image's format.
"#.into(),
        params: vector![
            Param { name: "bytes".into(), param_type: Type::Blob, default: None, },
        ],
        unbounded_args: false,
        return_type: None,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(ImageIns::FromBlob));
            Ok(instructions)
        })
    });
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum ImageIns {
    Width,
    Height,
    Grayscale,
    Invert,
    FlipVertical,
    FlipHorizontal,
    Rotate90,
    Rotate180,
    Rotate270,

    Resize,
    ResizeExact,
    Thumbnail,
    ThumbnailExact,

    Blur,
    BlurFast,
    Contrast,
    Brighten,

    Blob,
    Png,
    Jpeg,
    Gif,
    Webp,
    Tiff,
    Bmp,
    Ico,

    FromBlob,
}
#[typetag::serde(name = "ImageIns")]
impl Instruction for ImageIns {
    fn exec(&self, env: &mut ProcEnv, graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        match self {
            Self::Width => {
                if let Some(var) = env.stack.pop() {
                    if let Some(dref) = var.try_data_or_func() {
                        if let Some(image) = graph.get_mut_stof_data::<Image>(&dref) {
                            env.stack.push(Variable::val(Val::from(image.width())));
                            return Ok(None);
                        }
                    }
                }
                Err(Error::ImageWidth)
            },
            Self::Height => {
                if let Some(var) = env.stack.pop() {
                    if let Some(dref) = var.try_data_or_func() {
                        if let Some(image) = graph.get_mut_stof_data::<Image>(&dref) {
                            env.stack.push(Variable::val(Val::from(image.height())));
                            return Ok(None);
                        }
                    }
                }
                Err(Error::ImageHeight)
            },
            Self::Grayscale => {
                if let Some(var) = env.stack.pop() {
                    if let Some(dref) = var.try_data_or_func() {
                        if let Some(image) = graph.get_mut_stof_data::<Image>(&dref) {
                            image.grayscale();
                            return Ok(None);
                        }
                    }
                }
                Err(Error::ImageGrayscale)
            },
            Self::Invert => {
                if let Some(var) = env.stack.pop() {
                    if let Some(dref) = var.try_data_or_func() {
                        if let Some(image) = graph.get_mut_stof_data::<Image>(&dref) {
                            image.invert();
                            return Ok(None);
                        }
                    }
                }
                Err(Error::ImageInvert)
            },
            Self::FlipVertical => {
                if let Some(var) = env.stack.pop() {
                    if let Some(dref) = var.try_data_or_func() {
                        if let Some(image) = graph.get_mut_stof_data::<Image>(&dref) {
                            image.flip_vertical();
                            return Ok(None);
                        }
                    }
                }
                Err(Error::ImageFlipVertical)
            },
            Self::FlipHorizontal => {
                if let Some(var) = env.stack.pop() {
                    if let Some(dref) = var.try_data_or_func() {
                        if let Some(image) = graph.get_mut_stof_data::<Image>(&dref) {
                            image.flip_horizontal();
                            return Ok(None);
                        }
                    }
                }
                Err(Error::ImageFlipHorizontal)
            },
            Self::Rotate90 => {
                if let Some(var) = env.stack.pop() {
                    if let Some(dref) = var.try_data_or_func() {
                        if let Some(image) = graph.get_mut_stof_data::<Image>(&dref) {
                            image.rotate_90();
                            return Ok(None);
                        }
                    }
                }
                Err(Error::ImageRotate90)
            },
            Self::Rotate180 => {
                if let Some(var) = env.stack.pop() {
                    if let Some(dref) = var.try_data_or_func() {
                        if let Some(image) = graph.get_mut_stof_data::<Image>(&dref) {
                            image.rotate_180();
                            return Ok(None);
                        }
                    }
                }
                Err(Error::ImageRotate180)
            },
            Self::Rotate270 => {
                if let Some(var) = env.stack.pop() {
                    if let Some(dref) = var.try_data_or_func() {
                        if let Some(image) = graph.get_mut_stof_data::<Image>(&dref) {
                            image.rotate_270();
                            return Ok(None);
                        }
                    }
                }
                Err(Error::ImageRotate270)
            },

            Self::Resize => {
                if let Some(height_var) = env.stack.pop() {
                    if let Some(width_var) = env.stack.pop() {
                        if let Some(var) = env.stack.pop() {
                            if let Some(dref) = var.try_data_or_func() {
                                if let Some(image) = graph.get_mut_stof_data::<Image>(&dref) {
                                    let mut resized = false;
                                    match height_var.val.read().deref() {
                                        Val::Num(height) => {
                                            match width_var.val.read().deref() {
                                                Val::Num(width) => {
                                                    let height = height.int();
                                                    let width = width.int();
                                                    if height > 0 && width > 0 {
                                                        resized = true;
                                                        image.resize(width as u32, height as u32, FilterType::CatmullRom); // balanced speed and looks
                                                    }
                                                },
                                                _ => {}
                                            }
                                        },
                                        _ => {}
                                    }
                                    env.stack.push(Variable::val(Val::Bool(resized)));
                                    return Ok(None);
                                }
                            }
                        }
                    }
                }
                Err(Error::ImageResize)
            },
            Self::ResizeExact => {
                if let Some(height_var) = env.stack.pop() {
                    if let Some(width_var) = env.stack.pop() {
                        if let Some(var) = env.stack.pop() {
                            if let Some(dref) = var.try_data_or_func() {
                                if let Some(image) = graph.get_mut_stof_data::<Image>(&dref) {
                                    let mut resized = false;
                                    match height_var.val.read().deref() {
                                        Val::Num(height) => {
                                            match width_var.val.read().deref() {
                                                Val::Num(width) => {
                                                    let height = height.int();
                                                    let width = width.int();
                                                    if height > 0 && width > 0 {
                                                        resized = true;
                                                        image.resize_exact(width as u32, height as u32, FilterType::CatmullRom); // balanced speed and looks
                                                    }
                                                },
                                                _ => {}
                                            }
                                        },
                                        _ => {}
                                    }
                                    env.stack.push(Variable::val(Val::Bool(resized)));
                                    return Ok(None);
                                }
                            }
                        }
                    }
                }
                Err(Error::ImageResizeExact)
            },
            Self::Thumbnail => {
                if let Some(height_var) = env.stack.pop() {
                    if let Some(width_var) = env.stack.pop() {
                        if let Some(var) = env.stack.pop() {
                            if let Some(dref) = var.try_data_or_func() {
                                if let Some(image) = graph.get_mut_stof_data::<Image>(&dref) {
                                    let mut resized = false;
                                    match height_var.val.read().deref() {
                                        Val::Num(height) => {
                                            match width_var.val.read().deref() {
                                                Val::Num(width) => {
                                                    let height = height.int();
                                                    let width = width.int();
                                                    if height > 0 && width > 0 {
                                                        resized = true;
                                                        image.thumbnail(width as u32, height as u32);
                                                    }
                                                },
                                                _ => {}
                                            }
                                        },
                                        _ => {}
                                    }
                                    env.stack.push(Variable::val(Val::Bool(resized)));
                                    return Ok(None);
                                }
                            }
                        }
                    }
                }
                Err(Error::ImageThumbnail)
            },
            Self::ThumbnailExact => {
                if let Some(height_var) = env.stack.pop() {
                    if let Some(width_var) = env.stack.pop() {
                        if let Some(var) = env.stack.pop() {
                            if let Some(dref) = var.try_data_or_func() {
                                if let Some(image) = graph.get_mut_stof_data::<Image>(&dref) {
                                    let mut resized = false;
                                    match height_var.val.read().deref() {
                                        Val::Num(height) => {
                                            match width_var.val.read().deref() {
                                                Val::Num(width) => {
                                                    let height = height.int();
                                                    let width = width.int();
                                                    if height > 0 && width > 0 {
                                                        resized = true;
                                                        image.thumbnail_exact(width as u32, height as u32);
                                                    }
                                                },
                                                _ => {}
                                            }
                                        },
                                        _ => {}
                                    }
                                    env.stack.push(Variable::val(Val::Bool(resized)));
                                    return Ok(None);
                                }
                            }
                        }
                    }
                }
                Err(Error::ImageThumbnailExact)
            },

            Self::Blur => {
                if let Some(blur_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        if let Some(dref) = var.try_data_or_func() {
                            if let Some(image) = graph.get_mut_stof_data::<Image>(&dref) {
                                match blur_var.val.read().deref() {
                                    Val::Num(blur) => {
                                        image.blur(blur.float(None) as f32);
                                    },
                                    _ => {}
                                }
                                return Ok(None);
                            }
                        }
                    }
                }
                Err(Error::ImageBlur)
            },
            Self::BlurFast => {
                if let Some(blur_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        if let Some(dref) = var.try_data_or_func() {
                            if let Some(image) = graph.get_mut_stof_data::<Image>(&dref) {
                                match blur_var.val.read().deref() {
                                    Val::Num(blur) => {
                                        image.fast_blur(blur.float(None) as f32);
                                    },
                                    _ => {}
                                }
                                return Ok(None);
                            }
                        }
                    }
                }
                Err(Error::ImageBlurFast)
            },
            Self::Contrast => {
                if let Some(contrast_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        if let Some(dref) = var.try_data_or_func() {
                            if let Some(image) = graph.get_mut_stof_data::<Image>(&dref) {
                                match contrast_var.val.read().deref() {
                                    Val::Num(contrast) => {
                                        image.adjust_contrast(contrast.float(None) as f32);
                                    },
                                    _ => {}
                                }
                                return Ok(None);
                            }
                        }
                    }
                }
                Err(Error::ImageAdjustContrast)
            },
            Self::Brighten => {
                if let Some(bright_var) = env.stack.pop() {
                    if let Some(var) = env.stack.pop() {
                        if let Some(dref) = var.try_data_or_func() {
                            if let Some(image) = graph.get_mut_stof_data::<Image>(&dref) {
                                match bright_var.val.read().deref() {
                                    Val::Num(bright) => {
                                        image.brighten(bright.int() as i32);
                                    },
                                    _ => {}
                                }
                                return Ok(None);
                            }
                        }
                    }
                }
                Err(Error::ImageBrighten)
            },

            Self::Blob => {
                if let Some(var) = env.stack.pop() {
                    if let Some(dref) = var.try_data_or_func() {
                        if let Some(image) = graph.get_stof_data::<Image>(&dref) {
                            env.stack.push(Variable::val(Val::Blob(image.raw.clone())));
                            return Ok(None);
                        }
                    }
                }
                Err(Error::ImageBlob)
            },
            Self::Png => {
                if let Some(var) = env.stack.pop() {
                    if let Some(dref) = var.try_data_or_func() {
                        if let Some(image) = graph.get_mut_stof_data::<Image>(&dref) {
                            if let Some(bytes) = image.png_bytes() {
                                env.stack.push(Variable::val(Val::Blob(bytes)));
                            } else {
                                env.stack.push(Variable::val(Val::Null));
                            }
                            return Ok(None);
                        }
                    }
                }
                Err(Error::ImagePng)
            },
            Self::Jpeg => {
                if let Some(var) = env.stack.pop() {
                    if let Some(dref) = var.try_data_or_func() {
                        if let Some(image) = graph.get_mut_stof_data::<Image>(&dref) {
                            if let Some(bytes) = image.jpeg_bytes() {
                                env.stack.push(Variable::val(Val::Blob(bytes)));
                            } else {
                                env.stack.push(Variable::val(Val::Null));
                            }
                            return Ok(None);
                        }
                    }
                }
                Err(Error::ImageJpeg)
            },
            Self::Gif => {
                if let Some(var) = env.stack.pop() {
                    if let Some(dref) = var.try_data_or_func() {
                        if let Some(image) = graph.get_mut_stof_data::<Image>(&dref) {
                            if let Some(bytes) = image.gif_bytes() {
                                env.stack.push(Variable::val(Val::Blob(bytes)));
                            } else {
                                env.stack.push(Variable::val(Val::Null));
                            }
                            return Ok(None);
                        }
                    }
                }
                Err(Error::ImageGif)
            },
            Self::Webp => {
                if let Some(var) = env.stack.pop() {
                    if let Some(dref) = var.try_data_or_func() {
                        if let Some(image) = graph.get_mut_stof_data::<Image>(&dref) {
                            if let Some(bytes) = image.webp_bytes() {
                                env.stack.push(Variable::val(Val::Blob(bytes)));
                            } else {
                                env.stack.push(Variable::val(Val::Null));
                            }
                            return Ok(None);
                        }
                    }
                }
                Err(Error::ImageWebp)
            },
            Self::Tiff => {
                if let Some(var) = env.stack.pop() {
                    if let Some(dref) = var.try_data_or_func() {
                        if let Some(image) = graph.get_mut_stof_data::<Image>(&dref) {
                            if let Some(bytes) = image.tiff_bytes() {
                                env.stack.push(Variable::val(Val::Blob(bytes)));
                            } else {
                                env.stack.push(Variable::val(Val::Null));
                            }
                            return Ok(None);
                        }
                    }
                }
                Err(Error::ImageTiff)
            },
            Self::Bmp => {
                if let Some(var) = env.stack.pop() {
                    if let Some(dref) = var.try_data_or_func() {
                        if let Some(image) = graph.get_mut_stof_data::<Image>(&dref) {
                            if let Some(bytes) = image.bmp_bytes() {
                                env.stack.push(Variable::val(Val::Blob(bytes)));
                            } else {
                                env.stack.push(Variable::val(Val::Null));
                            }
                            return Ok(None);
                        }
                    }
                }
                Err(Error::ImageBmp)
            },
            Self::Ico => {
                if let Some(var) = env.stack.pop() {
                    if let Some(dref) = var.try_data_or_func() {
                        if let Some(image) = graph.get_mut_stof_data::<Image>(&dref) {
                            if let Some(bytes) = image.ico_bytes() {
                                env.stack.push(Variable::val(Val::Blob(bytes)));
                            } else {
                                env.stack.push(Variable::val(Val::Null));
                            }
                            return Ok(None);
                        }
                    }
                }
                Err(Error::ImageIco)
            },

            Self::FromBlob => {
                if let Some(var) = env.stack.pop() {
                    match var.val.read().deref() {
                        Val::Blob(blob) => {
                            match Image::from_bytes(blob.clone()) {
                                Ok(image) => {
                                    if let Some(dref) = graph.insert_stof_data(&env.self_ptr(), &nanoid!(), Box::new(image), None) {
                                        env.stack.push(Variable::val(Val::Data(dref)));
                                    } else {
                                        env.stack.push(Variable::val(Val::Null));
                                    }
                                    return Ok(None);
                                },
                                Err(_error) => {
                                    // could not create the image
                                }
                            }
                        },
                        _ => {}
                    }
                }
                Err(Error::ImageFromBlob)
            },
        }
    }
}
