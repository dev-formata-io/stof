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
use tokio::{runtime::{Builder, Runtime}, sync::Mutex};
use super::SDoc;


/// Stof Engine.
pub struct SEngine {
    pub doc: Arc<Mutex<SDoc>>,
    pub runtime: Arc<Runtime>,
}
impl Default for SEngine {
    fn default() -> Self {
        Self::new(SDoc::default())
    }
}
impl SEngine {
    /// Create a new engine.
    pub fn new(doc: SDoc) -> Self {
        Self {
            doc: Arc::new(Mutex::new(doc)),
            runtime: Arc::new(Builder::new_current_thread().build().unwrap()),
        }
    }
}
