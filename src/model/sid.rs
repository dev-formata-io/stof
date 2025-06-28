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

use bytes::Bytes;
use nanoid::nanoid;
use std::{fmt::Display, ops::Deref};
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord,)]
/// Stof ID.
/// Cross between nanoid (flexibility, speed, size) & uuid (storage, hashing, etc.)
/// Can be smaller than UUID and generated much quicker.
pub struct SId(pub Bytes);
impl SId {
    /// Create a new id with a specific string length.
    pub fn new(length: usize) -> Self {
        Self(Bytes::from(nanoid!(length)))
    }

    /// Length of this id (bytes).
    pub fn len(&self) -> usize {
        self.0.len()
    }
}
impl Default for SId {
    fn default() -> Self {
        Self::new(20)
    }
}
impl Deref for SId {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}
impl AsRef<str> for SId {
    fn as_ref(&self) -> &str {
        unsafe {
            std::str::from_utf8_unchecked(&self.0)
        }
    }
}
impl Display for SId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}
impl<T: ?Sized + ToString> From<&T> for SId {
    fn from(value: &T) -> Self {
        Self(Bytes::from(value.to_string()))
    }
}


#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use crate::model::SId;

    #[test]
    fn default() {
        let id = SId::default();
        assert_eq!(id.len(), 20);
    }

    #[test]
    fn from_string() {
        let id = SId::from("hellothere");
        assert_eq!(id.to_string(), "hellothere");
        assert_eq!(id.as_ref(), "hellothere");

        let an = SId::from(&id);
        assert_eq!(id.to_string(), an.to_string());
        assert!(id == an);
    }

    #[test]
    fn custom_length() {
        let id = SId::new(10);
        assert_eq!(id.len(), 10);
    }

    #[test]
    fn cloned() {
        let id = SId::new(35);
        let cl = id.clone();
        assert_eq!(cl, id);

        let an = SId::new(35);
        assert_ne!(cl, an);
    }

    #[test]
    fn to_bytes() {
        let id = SId::default();
        let by = Bytes::from(id.to_vec());
        assert_eq!(id.0, by);
        assert_eq!(id.as_ref(), &by);
    }
}
