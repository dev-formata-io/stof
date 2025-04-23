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

use std::ops::Deref;
use serde::{Deserialize, Serialize};
use crate::{parse_type, SDoc, SUnits};


/// Stof Value Types.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub enum SType {
    #[default]
    Void,
    Null,
    Bool,
    Number(SNumType),
    SemVer,
    String,
    Object(String),
    FnPtr,
    Data,
    Array,
    Map,
    Set,
    Tuple(Vec<SType>),
    Blob,
    Unknown,
    Boxed(Box<Self>),
}
impl PartialEq for SType {
    fn eq(&self, other: &Self) -> bool {
        if other.is_unknown() {
            return true; // unknown always matches...
        }
        match self {
            Self::Boxed(val) => {
                match other {
                    Self::Boxed(oval) => {
                        val.deref().eq(oval.deref())
                    },
                    _ => false,
                }
            },
            Self::Void => other.is_void(),
            Self::Null => other.is_null(),
            Self::Bool => other.is_bool(),
            Self::Number(ntype) => {
                match other {
                    Self::Number(otype) => ntype == otype,
                    _ => false,
                }
            },
            Self::String => other.is_string(),
            Self::SemVer => other.is_semver(),
            Self::Object(ntype) => {
                match other {
                    Self::Object(otype) => ntype == otype,
                    _ => false,
                }
            },
            Self::FnPtr => other.is_function_pointer(),
            Self::Data => other.is_data(),
            Self::Array => other.is_array(),
            Self::Map => other.is_map(),
            Self::Set => other.is_set(),
            Self::Tuple(vals) => {
                match other {
                    Self::Tuple(ovals) => vals == ovals,
                    _ => false,
                }
            },
            Self::Blob => other.is_blob(),
            Self::Unknown => true,
        }
    }
}
impl SType {
    /// Is collection?
    pub fn is_collection(&self) -> bool {
        match self {
            SType::Array |
            SType::Map |
            SType::Set |
            SType::Tuple(_) => true,
            _ => false
        }
    }

    /// Is boxed?
    pub fn is_boxed(&self) -> bool {
        match self {
            SType::Boxed(_) => true,
            _ => false,
        }
    }

    /// Is unknown?
    pub fn is_unknown(&self) -> bool {
        match self {
            SType::Unknown => true,
            _ => false
        }
    }

    /// Is pointer?
    pub fn is_pointer(&self) -> bool {
        match self {
            SType::FnPtr => true,
            _ => false
        }
    }

    /// Is vlaue?
    pub fn is_value(&self) -> bool {
        match self {
            SType::FnPtr |
            SType::Array |
            SType::Tuple(_) => false,
            _ => true
        }
    }

    /// Is void?
    pub fn is_void(&self) -> bool {
        match self {
            SType::Void => true,
            _ => false
        }
    }

    /// Is null?
    pub fn is_null(&self) -> bool {
        match self {
            SType::Null => true,
            _ => false,
        }
    }

    /// Is semver?
    pub fn is_semver(&self) -> bool {
        match self {
            SType::SemVer => true,
            _ => false,
        }
    }

    /// Is object?
    pub fn is_object(&self) -> bool {
        match self {
            SType::Object(_) => true,
            SType::Boxed(stype) => stype.is_object(),
            _ =>  false
        }
    }

    /// Is void or null? (empty type)
    pub fn is_empty(&self) -> bool {
        match self {
            SType::Null |
            SType::Void => true,
            _ => false,
        }
    }

    /// Is bool?
    pub fn is_bool(&self) -> bool {
        match self {
            SType::Bool => true,
            _ => false,
        }
    }

    /// Is string?
    pub fn is_string(&self) -> bool {
        match self {
            SType::String => true,
            SType::Boxed(stype) => stype.is_string(),
            _ => false,
        }
    }

    /// Is number?
    pub fn is_number(&self) -> bool {
        match self {
            SType::Number(_) => true,
            _ => false,
        }
    }

    /// Is array?
    pub fn is_array(&self) -> bool {
        match self {
            SType::Array => true,
            _ => false,
        }
    }

    /// Is a set?
    pub fn is_set(&self) -> bool {
        match self {
            SType::Set => true,
            _ => false,
        }
    }

    /// Is a map?
    pub fn is_map(&self) -> bool {
        match self {
            SType::Map => true,
            _ => false,
        }
    }

    /// Is binary blob?
    pub fn is_blob(&self) -> bool {
        match self {
            SType::Blob => true,
            _ => false,
        }
    }

    /// Is tuple?
    pub fn is_tuple(&self) -> bool {
        match self {
            SType::Tuple(_) => true,
            _ => false,
        }
    }

    /// Is function pointer?
    pub fn is_function_pointer(&self) -> bool {
        match self {
            SType::FnPtr => true,
            _ => false
        }
    }

    /// Is opaque data ref?
    pub fn is_data(&self) -> bool {
        match self {
            SType::Data => true,
            SType::Boxed(stype) => stype.is_data(),
            _ => false
        }
    }

    /// Tuple type.
    pub fn tuple(types: Vec<SType>) -> Self {
        Self::Tuple(types)
    }

    /// I64.
    pub fn i64() -> Self {
        Self::Number(SNumType::I64)
    }

    /// F64.
    pub fn f64() -> Self {
        Self::Number(SNumType::F64)
    }

    /// Units.
    pub fn units(units: SUnits) -> Self {
        Self::Number(SNumType::Units(units))
    }

    /// Is a root object type?
    pub fn is_root_object(&self) -> bool {
        match self {
            Self::Object(val) => val == "root",
            _ => false,
        }
    }

    /// Is a base object type?
    pub fn is_base_object(&self) -> bool {
        match self {
            Self::Object(val) => val == "obj",
            _ => false,
        }
    }

    /// Std Library Name.
    pub fn std_libname(&self) -> String {
        match self {
            SType::Unknown |
            SType::Null |
            SType::Void => String::default(),
            SType::Array => "Array".to_owned(),
            SType::Map => "Map".to_owned(),
            SType::Set => "Set".to_owned(),
            SType::FnPtr => "Function".to_owned(),
            SType::Data => "Data".to_owned(),
            SType::String => "String".to_owned(),
            SType::SemVer => "SemVer".to_owned(),
            SType::Number(_) => "Number".to_owned(),
            SType::Bool => "Bool".to_owned(),
            SType::Tuple(_) => "Tuple".to_owned(),
            SType::Blob => "Blob".to_owned(),
            SType::Object(_typename) => {
                "Object".to_owned()
            },
            SType::Boxed(btype) => {
                btype.std_libname()
            },
        }
    }

    /// Data type libname.
    pub fn data_type_libname(doc: &SDoc, libname: &str, tagname: &str) -> String {
        if libname == "Data" && doc.libraries.libraries.contains_key(tagname) {
            return tagname.to_owned();
        }
        libname.to_owned()
    }

    /// Typeof.
    pub fn type_of(&self) -> String {
        match self {
            Self::Boxed(boxed) => {
                format!("Box<{}>", boxed.type_of())
            },
            Self::Unknown => "unknown".into(),
            Self::Map => "map".into(),
            Self::Set => "set".into(),
            Self::Array => "vec".into(),
            Self::Bool => "bool".into(),
            Self::Blob => "blob".into(),
            Self::FnPtr => "fn".into(),
            Self::Data => "data".into(),
            Self::Null => "null".into(),
            Self::Number(ntype) => {
                ntype.type_of()
            },
            Self::SemVer => "semver".into(),
            Self::String => "str".into(),
            Self::Tuple(vals) => {
                let mut res = "(".to_string();
                for i in 0..vals.len() {
                    let v = &vals[i];
                    let type_of = v.type_of();
                    if i < vals.len() - 1 {
                        res.push_str(&format!("{}, ", type_of));
                    } else {
                        res.push_str(&type_of);
                    }
                }
                res.push_str(")");
                res
            },
            Self::Void => "void".into(),
            Self::Object(ctype) => ctype.clone(),
        }
    }
}
impl From<&str> for SType {
    fn from(value: &str) -> Self {
        parse_type(value).expect("failed to parse stof type string into an SType")
    }
}


/// Number types.
#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub enum SNumType {
    I64,
    F64,
    Units(SUnits),
}
impl PartialEq for SNumType {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::I64 => {
                match other {
                    Self::I64 => true,
                    _ => false,
                }
            },
            Self::F64 => {
                match other {
                    Self::F64 => true,
                    _ => false,
                }
            },
            Self::Units(units) => {
                match other {
                    Self::F64 => true,
                    Self::Units(ounits) => {
                        units == ounits
                    },
                    _ => false,
                }
            },
        }
    }
}
impl SNumType {
    ///
    /// Type of.
    ///
    pub fn type_of(&self) -> String {
        match self {
            Self::F64 => "float".into(),
            Self::I64 => "int".into(),
            Self::Units(units) => units.to_string(),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::SType;

    #[test]
    fn parse_int_type() {
        let stype = SType::from("int");
        assert_eq!(stype, SType::Number(crate::SNumType::I64));
    }

    #[test]
    fn parse_tuple_type() {
        let stype = SType::from("(str, str)");
        assert_eq!(stype, SType::Tuple(vec![SType::String, SType::String]));
    }

    #[test]
    fn parse_sub_tuple_type() {
        let stype = SType::from("(str, ((str, ((str, str), str)), str))");
        assert_eq!(stype, SType::Tuple(vec![SType::String, SType::Tuple(vec![SType::Tuple(vec![SType::String, SType::Tuple(vec![SType::Tuple(vec![SType::String, SType::String]), SType::String])]), SType::String])]));
    }
}
