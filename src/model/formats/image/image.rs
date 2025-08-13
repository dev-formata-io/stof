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

use std::{io::Cursor, path::Path};
use image::{imageops::FilterType, metadata::Orientation, DynamicImage, ImageFormat, ImageReader};
use serde::{Deserialize, Serialize};
use crate::model::StofData;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    pub raw: Vec<u8>,

    #[serde(skip)]
    pub dynamic: Option<DynamicImage>,
}

#[typetag::serde(name = "Image")]
impl StofData for Image {}

impl Image {
    #[allow(unused)]
    /// Create a new Image from a file.
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
                            raw: bytes,
                            dynamic: Some(image),
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

    /// Create a new Image from bytes.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, String> {
        if let Ok(mut reader) = ImageReader::new(Cursor::new(bytes)).with_guessed_format() {
            if reader.format().is_none() {
                reader.set_format(ImageFormat::Png); // PNG is default image format if one cannot be determined
            }
            if let Ok(image) = reader.decode() {
                let mut bytes: Vec<u8> = Vec::new();
                if let Err(error) = image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png) {
                    return Err(error.to_string());
                }
                return Ok(Self {
                    raw: bytes,
                    dynamic: Some(image),
                });
            }
        }
        Err("Could not convert bytes into an image".into())
    }
    
    /// Ensure dynamic image from raw image.
    pub fn ensure_dynamic(&mut self) -> bool {
        if self.dynamic.is_some() {
            return true;
        } else if let Ok(mut reader) = ImageReader::new(Cursor::new(&self.raw)).with_guessed_format() {
            if reader.format().is_none() {
                reader.set_format(ImageFormat::Png); // PNG is default image format if one cannot be determined
            }
            if let Ok(image) = reader.decode() {
                self.dynamic = Some(image);
                return true;
            }
        }
        false
    }

    /// Save dynamic image to raw image.
    pub fn save_image(&mut self) -> bool {
        if let Some(image) = &self.dynamic {
            let mut bytes: Vec<u8> = Vec::new();
            if let Err(_error) = image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png) {
                return false;
            }
            self.raw = bytes;
            return true;
        }
        false
    }

    /// PNG bytes.
    pub fn png_bytes(&mut self) -> Option<Vec<u8>> {
        if !self.ensure_dynamic() { return None; }
        if let Some(image) = &self.dynamic {
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
        if let Some(image) = &self.dynamic {
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
        if let Some(image) = &self.dynamic {
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
        if let Some(image) = &self.dynamic {
            let mut bytes: Vec<u8> = Vec::new();
            if let Err(_error) = image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::WebP) {
                return None;
            }
            return Some(bytes);
        }
        None
    }

    #[allow(unused)]
    /// PNM bytes.
    pub fn pnm_bytes(&mut self) -> Option<Vec<u8>> {
        if !self.ensure_dynamic() { return None; }
        if let Some(image) = &self.dynamic {
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
        if let Some(image) = &self.dynamic {
            let mut bytes: Vec<u8> = Vec::new();
            if let Err(_error) = image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Tiff) {
                return None;
            }
            return Some(bytes);
        }
        None
    }

    #[allow(unused)]
    /// Tga bytes.
    pub fn tga_bytes(&mut self) -> Option<Vec<u8>> {
        if !self.ensure_dynamic() { return None; }
        if let Some(image) = &self.dynamic {
            let mut bytes: Vec<u8> = Vec::new();
            if let Err(_error) = image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Tga) {
                return None;
            }
            return Some(bytes);
        }
        None
    }

    #[allow(unused)]
    /// Dds bytes.
    pub fn dds_bytes(&mut self) -> Option<Vec<u8>> {
        if !self.ensure_dynamic() { return None; }
        if let Some(image) = &self.dynamic {
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
        if let Some(image) = &self.dynamic {
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
        if let Some(image) = &self.dynamic {
            let mut bytes: Vec<u8> = Vec::new();
            if let Err(_error) = image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Ico) {
                return None;
            }
            return Some(bytes);
        }
        None
    }

    #[allow(unused)]
    /// Hdr bytes.
    pub fn hdr_bytes(&mut self) -> Option<Vec<u8>> {
        if !self.ensure_dynamic() { return None; }
        if let Some(image) = &self.dynamic {
            let mut bytes: Vec<u8> = Vec::new();
            if let Err(_error) = image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Hdr) {
                return None;
            }
            return Some(bytes);
        }
        None
    }

    #[allow(unused)]
    /// OpenExr bytes.
    pub fn open_exr_bytes(&mut self) -> Option<Vec<u8>> {
        if !self.ensure_dynamic() { return None; }
        if let Some(image) = &self.dynamic {
            let mut bytes: Vec<u8> = Vec::new();
            if let Err(_error) = image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::OpenExr) {
                return None;
            }
            return Some(bytes);
        }
        None
    }

    #[allow(unused)]
    /// Farbfeld bytes.
    pub fn farbfeld_bytes(&mut self) -> Option<Vec<u8>> {
        if !self.ensure_dynamic() { return None; }
        if let Some(image) = &self.dynamic {
            let mut bytes: Vec<u8> = Vec::new();
            if let Err(_error) = image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Farbfeld) {
                return None;
            }
            return Some(bytes);
        }
        None
    }

    #[allow(unused)]
    /// Avif bytes.
    pub fn avif_bytes(&mut self) -> Option<Vec<u8>> {
        if !self.ensure_dynamic() { return None; }
        if let Some(image) = &self.dynamic {
            let mut bytes: Vec<u8> = Vec::new();
            if let Err(_error) = image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Avif) {
                return None;
            }
            return Some(bytes);
        }
        None
    }

    #[allow(unused)]
    /// Qoi bytes.
    pub fn qoi_bytes(&mut self) -> Option<Vec<u8>> {
        if !self.ensure_dynamic() { return None; }
        if let Some(image) = &self.dynamic {
            let mut bytes: Vec<u8> = Vec::new();
            if let Err(_error) = image.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Qoi) {
                return None;
            }
            return Some(bytes);
        }
        None
    }

    #[allow(unused)]
    /// Pcx bytes.
    pub fn pcx_bytes(&mut self) -> Option<Vec<u8>> {
        if !self.ensure_dynamic() { return None; }
        if let Some(image) = &self.dynamic {
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
        if let Some(image) = &self.dynamic {
            return image.width();
        }
        0
    }

    /// Height of this image.
    pub fn height(&mut self) -> u32 {
        if !self.ensure_dynamic() { return 0; }
        if let Some(image) = &self.dynamic {
            return image.height();
        }
        0
    }

    /// Grayscale.
    /// Turns this image into a grayscale version of itself.
    pub fn grayscale(&mut self) {
        if !self.ensure_dynamic() { return; }
        let mut grayscale = None;
        if let Some(image) = &self.dynamic {
            grayscale = Some(image.grayscale());
        }
        if let Some(image) = grayscale {
            self.dynamic = Some(image);
            self.save_image();
        }
    }

    /// Invert the colors of this image.
    pub fn invert(&mut self) {
        if !self.ensure_dynamic() { return; }
        if let Some(image) = &mut self.dynamic {
            image.invert();
            self.save_image();
        }
    }

    /// Resize this image (keeps aspect ratio).
    pub fn resize(&mut self, width: u32, height: u32, filter: FilterType) {
        if !self.ensure_dynamic() { return; }
        let mut resized = None;
        if let Some(image) = &self.dynamic {
            resized = Some(image.resize(width, height, filter));
        }
        if let Some(image) = resized {
            self.dynamic = Some(image);
            self.save_image();
        }
    }

    /// Resize this image exactly (does not keep aspect ratio).
    pub fn resize_exact(&mut self, width: u32, height: u32, filter: FilterType) {
        if !self.ensure_dynamic() { return; }
        let mut resized = None;
        if let Some(image) = &self.dynamic {
            resized = Some(image.resize_exact(width, height, filter));
        }
        if let Some(image) = resized {
            self.dynamic = Some(image);
            self.save_image();
        }
    }

    /// Scale this image down to fit within a specific size.
    pub fn thumbnail(&mut self, width: u32, height: u32) {
        if !self.ensure_dynamic() { return; }
        let mut resized = None;
        if let Some(image) = &self.dynamic {
            resized = Some(image.thumbnail(width, height));
        }
        if let Some(image) = resized {
            self.dynamic = Some(image);
            self.save_image();
        }
    }

    /// Scale this image down to fit within a specific size without preserving aspect ratio.
    pub fn thumbnail_exact(&mut self, width: u32, height: u32) {
        if !self.ensure_dynamic() { return; }
        let mut resized = None;
        if let Some(image) = &self.dynamic {
            resized = Some(image.thumbnail_exact(width, height));
        }
        if let Some(image) = resized {
            self.dynamic = Some(image);
            self.save_image();
        }
    }

    /// Performs a guassian blur on this image.
    pub fn blur(&mut self, sigma: f32) {
        if !self.ensure_dynamic() { return; }
        let mut resized = None;
        if let Some(image) = &self.dynamic {
            resized = Some(image.blur(sigma));
        }
        if let Some(image) = resized {
            self.dynamic = Some(image);
            self.save_image();
        }
    }

    /// Fast blur.
    pub fn fast_blur(&mut self, sigma: f32) {
        if !self.ensure_dynamic() { return; }
        let mut resized = None;
        if let Some(image) = &self.dynamic {
            resized = Some(image.fast_blur(sigma));
        }
        if let Some(image) = resized {
            self.dynamic = Some(image);
            self.save_image();
        }
    }

    /// Adjust contrast. Positive to increase, negative to decrease.
    pub fn adjust_contrast(&mut self, contrast: f32) {
        if !self.ensure_dynamic() { return; }
        let mut altered = None;
        if let Some(image) = &self.dynamic {
            altered = Some(image.adjust_contrast(contrast));
        }
        if let Some(image) = altered {
            self.dynamic = Some(image);
            self.save_image();
        }
    }

    /// Brighten the pixels of this image.
    pub fn brighten(&mut self, value: i32) {
        if !self.ensure_dynamic() { return; }
        let mut altered = None;
        if let Some(image) = &self.dynamic {
            altered = Some(image.brighten(value));
        }
        if let Some(image) = altered {
            self.dynamic = Some(image);
            self.save_image();
        }
    }

    /// Flip vertically.
    pub fn flip_vertical(&mut self) {
        if !self.ensure_dynamic() { return; }
        if let Some(image) = &mut self.dynamic {
            image.apply_orientation(Orientation::FlipVertical);
            self.save_image();
        }
    }

    /// Flip horizontally.
    pub fn flip_horizontal(&mut self) {
        if !self.ensure_dynamic() { return; }
        if let Some(image) = &mut self.dynamic {
            image.apply_orientation(Orientation::FlipHorizontal);
            self.save_image();
        }
    }

    /// Rotate 90 degrees clockwise.
    pub fn rotate_90(&mut self) {
        if !self.ensure_dynamic() { return; }
        if let Some(image) = &mut self.dynamic {
            image.apply_orientation(Orientation::Rotate90);
            self.save_image();
        }
    }

    /// Rotate 180 degrees clockwise.
    pub fn rotate_180(&mut self) {
        if !self.ensure_dynamic() { return; }
        if let Some(image) = &mut self.dynamic {
            image.apply_orientation(Orientation::Rotate180);
            self.save_image();
        }
    }

    /// Rotate 270 degrees clockwise.
    pub fn rotate_270(&mut self) {
        if !self.ensure_dynamic() { return; }
        if let Some(image) = &mut self.dynamic {
            image.apply_orientation(Orientation::Rotate270);
            self.save_image();
        }
    }
}
