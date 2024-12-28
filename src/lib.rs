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

pub mod core;
pub use core::*;

pub mod data;
pub use data::*;

pub mod stof;
pub use stof::*;

pub mod text;
pub mod bytes;

#[cfg(feature = "wasm")]
pub mod js;

#[cfg(feature = "json")]
pub mod json;

#[cfg(feature = "toml")]
pub mod toml;

#[cfg(feature = "yaml")]
pub mod yaml;

#[cfg(feature = "xml")]
pub mod xml;

#[cfg(feature = "urlencoded")]
pub mod urlencoded;

#[cfg(test)]
mod tests;
