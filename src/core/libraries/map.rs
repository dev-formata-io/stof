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

use anyhow::{anyhow, Result};
use crate::{Library, SDoc, SVal};
use super::Object;


/// Map library.
#[derive(Default, Debug)]
pub struct MapLibrary;
impl Object for MapLibrary {}
impl Library for MapLibrary {
    fn scope(&self) -> String {
        "Map".to_string()
    }

    fn call(&self, pid: &str, doc: &mut SDoc, name: &str, parameters: &mut Vec<SVal>) -> Result<SVal> {
        match name {
            
            _ => {}
        }

        // try object scope
        if let Ok(val) = Self::object_call(pid, doc, name, parameters) {
            return Ok(val);
        }
        Err(anyhow!("Did not find the requested '{}' function in the Map library", name))
    }
}
