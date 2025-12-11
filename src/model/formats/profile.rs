//
// Copyright 2025 Formata, Inc. All rights reserved.
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

use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Profile.
pub struct Profile {
    /// The name of this profile.
    pub name: String,

    /// Debug information? TODO
    pub debug_info: bool,

    /// Import with doc info (if supported)?
    pub docs: bool,

    /// Excluded attributes (both functions and fields).
    pub exclude_attributes: FxHashSet<String>,
}
impl Default for Profile {
    fn default() -> Self {
        Self::prod()
    }
}
impl Profile {
    /// Test profile.
    /// All attributes are included.
    pub fn test() -> Self {
        Self {
            name: "test".to_string(),
            debug_info: true,
            docs: false,
            exclude_attributes: Default::default(),
        }
    }

    /// Debug profile.
    /// Prod, but with debug info & docs.
    pub fn debug() -> Self {
        let mut exclude_attributes = FxHashSet::default();
        exclude_attributes.insert("test".into());
        Self {
            name: "debug".to_string(),
            debug_info: true,
            docs: true,
            exclude_attributes,
        }
    }

    /// Prod profile.
    /// Test attributes are excluded.
    pub fn prod() -> Self {
        let mut exclude_attributes = FxHashSet::default();
        exclude_attributes.insert("test".into());
        Self {
            name: "prod".to_string(),
            debug_info: false,
            docs: false,
            exclude_attributes,
        }
    }
}
