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


pub mod import;
pub use import::*;

pub mod export;
pub use export::*;

use crate::model::Format;


#[derive(Debug)]
pub struct JsonFormat;
impl JsonFormat {
    
}
impl Format for JsonFormat {
    fn identifiers(&self) -> Vec<String> {
        vec!["json".into()]
    }
    fn content_type(&self) -> String {
        "application/json".into()
    }
    // TODO
}
