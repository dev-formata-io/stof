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

    /// Source from somewhere else (Ex. another party's tools): declarations only. Nothing runs while it's parsed
    /// (no expressions: field and attribute values must be literals; no #[init] functions), it imports nothing, and it
    /// declares no #[type]s (types are found by name across the document). Its functions are compiled, not run: seal
    /// the part it was parsed into (confined) before calling them.
    pub untrusted: bool,
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
            untrusted: false,
        }
    }

    /// Docs profile.
    /// No debug info, optional tests, but include docs.
    pub fn docs(tests: bool) -> Self {
        let mut exclude_attributes = FxHashSet::default();
        if !tests { exclude_attributes.insert("test".into()); }
        Self {
            name: "docs".to_string(),
            debug_info: false,
            docs: true,
            exclude_attributes,
            untrusted: false,
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
            untrusted: false,
        }
    }

    /// Untrusted profile ("untrusted"): source from somewhere else, declarations only (see `untrusted`). Test
    /// attributes are excluded.
    pub fn untrusted() -> Self {
        let mut exclude_attributes = FxHashSet::default();
        exclude_attributes.insert("test".into());
        Self {
            name: "untrusted".to_string(),
            debug_info: false,
            docs: false,
            exclude_attributes,
            untrusted: true,
        }
    }
}
