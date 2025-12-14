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

use arcstr::ArcStr;
use compact_str::CompactString;
use nanoid::nanoid;
use std::{fmt::{self, Display}, ops::Deref};
use serde::{Deserialize, Deserializer, Serialize, Serializer};


/// Stof ID - Optimized string-based identifier
/// 
/// Uses CompactString for:
/// - Zero heap allocation for strings ≤23 bytes (most IDs)
/// - Cheap cloning (inline or refcounted)
/// - Human-readable identifiers
/// - Paths, names, and generated IDs
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SId(pub CompactString);
impl SId {
    /// Create a new random ID (14 chars, fits inline!)
    #[inline]
    pub fn new() -> Self {
        Self(CompactString::new(nanoid!(14)))
    }

    /// Create from any string value
    #[inline]
    pub fn from_str(s: impl AsRef<str>) -> Self {
        Self(CompactString::new(s))
    }

    /// Get as string slice
    #[inline]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Length in bytes
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Check if stored inline (no heap allocation)
    /// Most paths and all generated IDs will be inline!
    #[inline]
    pub fn is_inline(&self) -> bool {
        !self.0.is_heap_allocated()
    }

    /// Memory diagnostics
    pub fn memory_info(&self) -> String {
        format!(
            "len={}, inline={}, heap={}",
            self.len(),
            self.is_inline(),
            if self.is_inline() { 0 } else { self.len() }
        )
    }
}
impl Default for SId {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for SId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Deref for SId {
    type Target = str;
    
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for SId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<&str> for SId {
    fn from(s: &str) -> Self {
        Self::from_str(s)
    }
}

impl From<String> for SId {
    fn from(s: String) -> Self {
        Self(CompactString::from(s))
    }
}

impl From<CompactString> for SId {
    fn from(s: CompactString) -> Self {
        Self(s)
    }
}

impl From<&String> for SId {
    fn from(s: &String) -> Self {
        Self::from_str(s)
    }
}

impl From<ArcStr> for SId {
    fn from(value: ArcStr) -> Self {
        Self::from_str(&value)
    }
}

impl From<&ArcStr> for SId {
    fn from(value: &ArcStr) -> Self {
        Self::from_str(&value)
    }
}

impl From<&SId> for SId {
    fn from(value: &SId) -> Self {
        value.clone()
    }
}


// Serialize as a simple string
impl Serialize for SId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

// Deserialize from either old Bytes format or new string format
impl<'de> Deserialize<'de> for SId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{self, Visitor};
        
        struct SIdVisitor;
        
        impl<'de> Visitor<'de> for SIdVisitor {
            type Value = SId;
            
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string or bytes representing an SId")
            }
            
            // New format: direct string (most common)
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(SId::from_str(v))
            }
            
            // New format: owned string
            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(SId::from(v))
            }
            
            // Old format: Bytes serialized as bytes (some formats)
            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let s = std::str::from_utf8(v)
                    .map_err(|_| de::Error::custom("invalid UTF-8 in SId bytes"))?;
                Ok(SId::from_str(s))
            }
            
            // Old format: Bytes serialized as byte buffer
            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                let s = String::from_utf8(v)
                    .map_err(|e| de::Error::custom(format!("invalid UTF-8 in SId: {}", e)))?;
                Ok(SId::from(s))
            }
        }
        
        // Use deserialize_string for bincode (schema-based)
        // This is more efficient and works because both Bytes and String
        // serialize identically in bincode
        if deserializer.is_human_readable() {
            // For human-readable formats (JSON, YAML, etc.), use visitor
            // to handle potential different representations
            deserializer.deserialize_str(SIdVisitor)
        } else {
            // For binary formats (bincode, postcard, etc.), just deserialize as String
            // This is faster and works because the binary representation is identical
            let s = String::deserialize(deserializer)?;
            Ok(SId::from(s))
        }
    }
}


#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use serde::Serialize;
    use crate::model::SId;

    #[test]
    fn default() {
        let id = SId::default();
        assert_eq!(id.len(), 14);
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
    fn cloned() {
        let id = SId::new();
        let cl = id.clone();
        assert_eq!(cl, id);

        let an = SId::new();
        assert_ne!(cl, an);
    }

    #[test]
    fn to_bytes() {
        let id = SId::default();
        let by = Bytes::from(id.to_string());
        assert_eq!(id.as_ref(), &by);
    }
    
    #[test]
    fn test_deserialize_new_format() {
        let id = SId::from_str("test_id_123");
        let serialized = bincode::serialize(&id).unwrap();
        let deserialized: SId = bincode::deserialize(&serialized).unwrap();
        assert_eq!(id, deserialized);
    }
    
    #[test]
    fn test_deserialize_old_bytes_format() {
        // Simulate old format: Bytes wrapping a string
        #[derive(Serialize)]
        struct OldSId(Bytes);
        
        let old_id = OldSId(Bytes::from("old_test_id"));
        let serialized = bincode::serialize(&old_id).unwrap();
        
        // Should deserialize into new SId format
        let deserialized: SId = bincode::deserialize(&serialized).unwrap();
        assert_eq!(deserialized.as_str(), "old_test_id");
    }
    
    #[test]
    fn test_roundtrip() {
        let original = SId::from_str("roundtrip_test");
        let serialized = bincode::serialize(&original).unwrap();
        let deserialized: SId = bincode::deserialize(&serialized).unwrap();
        assert_eq!(original, deserialized);
    }
    
    #[test]
    fn test_inline_optimization() {
        let id = SId::new(); // 14 chars
        assert!(id.is_inline());
        
        let serialized = bincode::serialize(&id).unwrap();
        let deserialized: SId = bincode::deserialize(&serialized).unwrap();
        assert!(deserialized.is_inline());
    }
}
