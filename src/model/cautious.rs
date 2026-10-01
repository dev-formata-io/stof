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

//! Safe binary (BSTF/bincode) decoding for untrusted bytes.
//!
//! bincode trusts the lengths it reads, and some collections (imbl's Vector) preallocate whatever length the
//! data claims, so a corrupt or hostile blob could ask for exabytes and abort the process (an allocation
//! failure can't be caught, and in wasm it traps the whole engine). Deeply nested data could also overflow the
//! stack. This wraps the deserializer so collections never preallocate more than a small amount (they still
//! grow to the real size), nesting is bounded, and byte reads can't exceed the input. The encoding is
//! unchanged: same bytes as `bincode::serialize`.

use serde::de::{self, DeserializeOwned, DeserializeSeed, Deserializer, EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor};
use std::fmt;
use bincode::Options;
use crate::parser::source::grow;


/// Max nesting depth for decoded data. Native builds (`stacker` feature) also grow the stack as needed, but
/// the limit is the same everywhere so a document that decodes on one platform decodes on all.
/// Typetag/erased-serde stack use grows faster than linearly with nesting (each `Box<dyn ..>` level wraps the
/// previous one), so this is what keeps a 1MB wasm stack safe. Each nested block or call in a function is ~3
/// levels; Limitr's whole spec peaks at 41.
const MAX_DEPTH: usize = 128;

/// Max preallocation hint for collections (they grow past it as elements actually decode).
const MAX_HINT: usize = 4096;


/// Decode bincode bytes (same format as `bincode::deserialize`) without trusting their lengths.
pub fn bincode_deserialize<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, bincode::Error> {
    let options = bincode::DefaultOptions::new()
        .with_fixint_encoding()
        .allow_trailing_bytes()
        .with_limit(bytes.len() as u64);
    let mut deserializer = bincode::Deserializer::from_slice(bytes, options);
    T::deserialize(Cautious { de: &mut deserializer, depth: 0 })
}


struct Cautious<D> { de: D, depth: usize }
impl<D> Cautious<D> {
    fn deeper<E: de::Error>(&self) -> Result<usize, E> {
        if self.depth >= MAX_DEPTH { Err(E::custom("data is nested too deeply")) } else { Ok(self.depth + 1) }
    }
}

struct Wrap<V> { inner: V, depth: usize }

macro_rules! forward_deserialize {
    ($($method:ident),*) => {
        $(
            fn $method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
                let depth = self.deeper()?;
                grow(move || self.de.$method(Wrap { inner: visitor, depth }))
            }
        )*
    };
}

impl<'de, D: Deserializer<'de>> Deserializer<'de> for Cautious<D> {
    type Error = D::Error;

    forward_deserialize!(deserialize_any, deserialize_bool, deserialize_i8, deserialize_i16, deserialize_i32, deserialize_i64,
        deserialize_i128, deserialize_u8, deserialize_u16, deserialize_u32, deserialize_u64, deserialize_u128, deserialize_f32,
        deserialize_f64, deserialize_char, deserialize_str, deserialize_string, deserialize_bytes, deserialize_byte_buf,
        deserialize_option, deserialize_unit, deserialize_seq, deserialize_map, deserialize_identifier, deserialize_ignored_any);

    fn deserialize_unit_struct<V: Visitor<'de>>(self, name: &'static str, visitor: V) -> Result<V::Value, Self::Error> {
        let depth = self.deeper()?;
        grow(move || self.de.deserialize_unit_struct(name, Wrap { inner: visitor, depth }))
    }
    fn deserialize_newtype_struct<V: Visitor<'de>>(self, name: &'static str, visitor: V) -> Result<V::Value, Self::Error> {
        let depth = self.deeper()?;
        grow(move || self.de.deserialize_newtype_struct(name, Wrap { inner: visitor, depth }))
    }
    fn deserialize_tuple<V: Visitor<'de>>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error> {
        let depth = self.deeper()?;
        grow(move || self.de.deserialize_tuple(len, Wrap { inner: visitor, depth }))
    }
    fn deserialize_tuple_struct<V: Visitor<'de>>(self, name: &'static str, len: usize, visitor: V) -> Result<V::Value, Self::Error> {
        let depth = self.deeper()?;
        grow(move || self.de.deserialize_tuple_struct(name, len, Wrap { inner: visitor, depth }))
    }
    fn deserialize_struct<V: Visitor<'de>>(self, name: &'static str, fields: &'static [&'static str], visitor: V) -> Result<V::Value, Self::Error> {
        let depth = self.deeper()?;
        grow(move || self.de.deserialize_struct(name, fields, Wrap { inner: visitor, depth }))
    }
    fn deserialize_enum<V: Visitor<'de>>(self, name: &'static str, variants: &'static [&'static str], visitor: V) -> Result<V::Value, Self::Error> {
        let depth = self.deeper()?;
        grow(move || self.de.deserialize_enum(name, variants, Wrap { inner: visitor, depth }))
    }
    fn is_human_readable(&self) -> bool { self.de.is_human_readable() }
}


macro_rules! forward_visit {
    ($($method:ident: $ty:ty),*) => {
        $(
            fn $method<E: de::Error>(self, v: $ty) -> Result<Self::Value, E> { self.inner.$method(v) }
        )*
    };
}

impl<'de, V: Visitor<'de>> Visitor<'de> for Wrap<V> {
    type Value = V::Value;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result { self.inner.expecting(formatter) }

    forward_visit!(visit_bool: bool, visit_i8: i8, visit_i16: i16, visit_i32: i32, visit_i64: i64, visit_i128: i128,
        visit_u8: u8, visit_u16: u16, visit_u32: u32, visit_u64: u64, visit_u128: u128, visit_f32: f32, visit_f64: f64,
        visit_char: char, visit_str: &str, visit_borrowed_str: &'de str, visit_string: String, visit_bytes: &[u8],
        visit_borrowed_bytes: &'de [u8], visit_byte_buf: Vec<u8>);

    fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> { self.inner.visit_none() }
    fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> { self.inner.visit_unit() }
    fn visit_some<D: Deserializer<'de>>(self, de: D) -> Result<Self::Value, D::Error> {
        self.inner.visit_some(Cautious { de, depth: self.depth })
    }
    fn visit_newtype_struct<D: Deserializer<'de>>(self, de: D) -> Result<Self::Value, D::Error> {
        self.inner.visit_newtype_struct(Cautious { de, depth: self.depth })
    }
    fn visit_seq<A: SeqAccess<'de>>(self, seq: A) -> Result<Self::Value, A::Error> {
        self.inner.visit_seq(Wrap { inner: seq, depth: self.depth })
    }
    fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
        self.inner.visit_map(Wrap { inner: map, depth: self.depth })
    }
    fn visit_enum<A: EnumAccess<'de>>(self, data: A) -> Result<Self::Value, A::Error> {
        self.inner.visit_enum(Wrap { inner: data, depth: self.depth })
    }
}


/// A seed whose deserializer is wrapped too.
struct Seed<S> { inner: S, depth: usize }
impl<'de, S: DeserializeSeed<'de>> DeserializeSeed<'de> for Seed<S> {
    type Value = S::Value;
    fn deserialize<D: Deserializer<'de>>(self, de: D) -> Result<Self::Value, D::Error> {
        self.inner.deserialize(Cautious { de, depth: self.depth })
    }
}

impl<'de, A: SeqAccess<'de>> SeqAccess<'de> for Wrap<A> {
    type Error = A::Error;
    fn next_element_seed<T: DeserializeSeed<'de>>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error> {
        self.inner.next_element_seed(Seed { inner: seed, depth: self.depth })
    }
    fn size_hint(&self) -> Option<usize> { self.inner.size_hint().map(|hint| hint.min(MAX_HINT)) }
}

impl<'de, A: MapAccess<'de>> MapAccess<'de> for Wrap<A> {
    type Error = A::Error;
    fn next_key_seed<K: DeserializeSeed<'de>>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error> {
        self.inner.next_key_seed(Seed { inner: seed, depth: self.depth })
    }
    fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value, Self::Error> {
        self.inner.next_value_seed(Seed { inner: seed, depth: self.depth })
    }
    fn size_hint(&self) -> Option<usize> { self.inner.size_hint().map(|hint| hint.min(MAX_HINT)) }
}

impl<'de, A: EnumAccess<'de>> EnumAccess<'de> for Wrap<A> {
    type Error = A::Error;
    type Variant = Wrap<A::Variant>;
    fn variant_seed<V: DeserializeSeed<'de>>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error> {
        let depth = self.depth;
        let (value, variant) = self.inner.variant_seed(Seed { inner: seed, depth })?;
        Ok((value, Wrap { inner: variant, depth }))
    }
}

impl<'de, A: VariantAccess<'de>> VariantAccess<'de> for Wrap<A> {
    type Error = A::Error;
    fn unit_variant(self) -> Result<(), Self::Error> { self.inner.unit_variant() }
    fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value, Self::Error> {
        self.inner.newtype_variant_seed(Seed { inner: seed, depth: self.depth })
    }
    fn tuple_variant<V: Visitor<'de>>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error> {
        self.inner.tuple_variant(len, Wrap { inner: visitor, depth: self.depth })
    }
    fn struct_variant<V: Visitor<'de>>(self, fields: &'static [&'static str], visitor: V) -> Result<V::Value, Self::Error> {
        self.inner.struct_variant(fields, Wrap { inner: visitor, depth: self.depth })
    }
}


#[cfg(test)]
mod tests {
    use imbl::Vector;
    use super::bincode_deserialize;

    #[test]
    /// Same bytes as plain bincode, but lying lengths and deep nesting are errors instead of aborts.
    fn untrusted_bytes() {
        let value: (Vector<String>, Option<u32>) = (Vector::from(vec!["a".to_string(), "b".to_string()]), Some(7));
        let bytes = bincode::serialize(&value).unwrap();
        assert_eq!(bincode_deserialize::<(Vector<String>, Option<u32>)>(&bytes).unwrap(), value);

        // a vector claiming u64::MAX / 2 elements (would preallocate exabytes)
        let mut lying = (u64::MAX / 2).to_le_bytes().to_vec();
        lying.extend([1, 0, 0, 0, 0, 0, 0, 0, b'x']);
        assert!(bincode_deserialize::<Vector<String>>(&lying).is_err());

        // nesting deeper than the limit is an error, not a stack overflow
        let mut val = crate::runtime::Val::Null;
        for _ in 0..200 { val = crate::runtime::Val::List(Vector::from(vec![crate::runtime::ValRef::new(val)])); }
        let bytes = bincode::serialize(&val).unwrap();
        assert!(bincode_deserialize::<crate::runtime::Val>(&bytes).is_err());

        // a string claiming more bytes than the input has
        let mut long = 1_000_000u64.to_le_bytes().to_vec();
        long.push(b'x');
        assert!(bincode_deserialize::<String>(&long).is_err());
    }
}
