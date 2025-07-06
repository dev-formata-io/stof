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

use arcstr::{literal, ArcStr};
use imbl::Vector;
use serde::{Deserialize, Serialize};
use crate::{model::SId, parser::types::parse_type_complete, runtime::Units};


// Literal string types.
const NULL: ArcStr = literal!("null");
const VOID: ArcStr = literal!("void");
const UNKNOWN: ArcStr = literal!("unknown");
const MAP: ArcStr = literal!("map");
const SET: ArcStr = literal!("set");
const LIST: ArcStr = literal!("list");
const BOOL: ArcStr = literal!("bool");
const BLOB: ArcStr = literal!("blob");
const FUNC: ArcStr = literal!("fn");
pub(super) const DATA: ArcStr = literal!("data");
pub(super) const OBJ: ArcStr = literal!("obj");
const VER: ArcStr = literal!("ver");
const STR: ArcStr = literal!("str");
const INT: ArcStr = literal!("int");
const FLOAT: ArcStr = literal!("float");


#[derive(Debug, Clone, Deserialize, Serialize, Default, Hash)]
/// Type.
pub enum Type {
    #[default]
    Void,
    Null,

    Promise(Box<Self>),

    Bool,
    Num(NumT),
    Str,
    Ver,

    Obj(SId), // Prototypes
    Fn,
    Data(ArcStr), // typetag lib linking, etc.

    Blob,

    List,
    Tup(Vector<Self>),
    Map,
    Set,

    Unknown,
    Union(Vector<Self>),
}
impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        match other {
            Self::Unknown => return true,
            Self::Union(types) => {
                match self {
                    Self::Union(otypes) => {
                        for ty in types {
                            for oty in otypes {
                                if ty.eq(oty) {
                                    return true;
                                }
                            }
                        }
                        return false;
                    },
                    sf => {
                        for ty in types {
                            if ty.eq(sf) {
                                return true;
                            }
                        }
                        return false;
                    }
                }
            },
            Self::Promise(ty) => {
                match self {
                    Self::Promise(oty) => return ty == oty,
                    _ => return **ty == *self,
                }
            },
            _ => {}
        }
        match self {
            Self::Union(types) => {
                match other {
                    Self::Union(otypes) => {
                        for ty in types {
                            for oty in otypes {
                                if ty.eq(oty) {
                                    return true;
                                }
                            }
                        }
                        return false;
                    },
                    other => {
                        for ty in types {
                            if ty.eq(other) {
                                return true;
                            }
                        }
                        return false;
                    }
                }
            },
            Self::Void => {
                match other {
                    Self::Void => true,
                    _ => false,
                }
            },
            Self::Null => {
                match other {
                    Self::Null => true,
                    _ => false,
                }
            },
            Self::Bool => {
                match other {
                    Self::Bool => true,
                    _ => false,
                }
            },
            Self::Num(t) => {
                match other {
                    Self::Num(ot) => t.eq(ot),
                    _ => false,
                }
            },
            Self::Str => {
                match other {
                    Self::Str => true,
                    _ => false,
                }
            },
            Self::Ver => {
                match other {
                    Self::Ver => true,
                    _ => false,
                }
            },
            Self::Obj(t) => {
                match other {
                    Self::Obj(ot) => t.eq(ot),
                    _ => false,
                }
            },
            Self::Fn => {
                match other {
                    Self::Fn => true,
                    _ => false,
                }
            },
            Self::Data(t) => {
                match other {
                    Self::Data(ot) => t.eq(ot),
                    _ => false, // gen type handled in cast
                }
            },
            Self::List => {
                match other {
                    Self::List => true,
                    _ => false,
                }
            },
            Self::Map => {
                match other {
                    Self::Map => true,
                    _ => false,
                }
            },
            Self::Set => {
                match other {
                    Self::Set => true,
                    _ => false,
                }
            },
            Self::Tup(types) => {
                match other {
                    Self::Tup(otypes) => types.eq(otypes),
                    _ => false,
                }
            },
            Self::Blob => {
                match other {
                    Self::Blob => true,
                    _ => false,
                }
            },
            Self::Promise(ty) => {
                match other {
                    Self::Promise(oty) => ty == oty,
                    _ => **ty == *other,
                }
            },
            Self::Unknown => true,
        }
    }
}
impl Eq for Type {}
impl Type {
    #[inline]
    pub fn empty(&self) -> bool {
        match self {
            Self::Null |
            Self::Void => true,
            _ => false,
        }
    }

    pub fn type_of(&self) -> ArcStr {
        match self {
            Self::Union(types) => {
                let mut geo = String::default();
                for ty in types {
                    if geo.len() < 1 {
                        geo.push_str(&ty.type_of());
                    } else {
                        geo.push_str(&format!(" | {}", ty.type_of()));
                    }
                }
                geo.into()
            },
            Self::Unknown => UNKNOWN,
            Self::Map => MAP,
            Self::Set => SET,
            Self::List => LIST,
            Self::Bool => BOOL,
            Self::Blob => BLOB,
            Self::Fn => FUNC,
            Self::Data(tname) => {
                let dta = DATA;
                if tname == &dta {
                    return dta;
                }
                format!("Data<{}>", tname).into()
            },
            Self::Null => NULL,
            Self::Num(num) => num.type_of(),
            Self::Ver => VER,
            Self::Str => STR,
            Self::Tup(vals) => {
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
                res.into()
            },
            Self::Void => VOID,
            Self::Obj(ctype) => ctype.as_ref().into(),
            Self::Promise(ty) => format!("Promise<{}>", ty.type_of()).into(),
        }
    }

    pub fn md_type_of(&self) -> String {
        self.type_of().replace("<", "\\<")
    }

    /// Generic libname.
    pub fn gen_lib_name(&self) -> ArcStr {
        match self {
            Self::Unknown |
            Self::Null |
            Self::Union(_) |
            Self::Void => literal!("Empty"),
            Self::List => literal!("List"),
            Self::Map => literal!("Map"),
            Self::Set => literal!("Set"),
            Self::Blob => literal!("Blob"),
            Self::Bool => literal!("Bool"),
            Self::Fn => literal!("Fn"),
            Self::Num(_) => literal!("Num"),
            Self::Data(_) => literal!("Data"),
            Self::Str => literal!("Str"),
            Self::Obj(_) => literal!("Obj"),
            Self::Promise(_) => literal!("Promise"),
            Self::Ver => literal!("Ver"),
            Self::Tup(_) => literal!("Tup"),
        }
    }
}
impl<T: AsRef<str>> From<T> for Type {
    fn from(value: T) -> Self {
        parse_type_complete(value.as_ref()).expect(&format!("failed to parse stof type string '{}' into a valid Type", value.as_ref()))
    }
}
impl ToString for Type {
    fn to_string(&self) -> String {
        self.type_of().to_string()
    }
}


#[derive(Debug, Clone, Copy, Deserialize, Serialize, Hash)]
/// Number Type.
pub enum NumT {
    Int,
    Float,
    Units(Units),
}
impl PartialEq for NumT {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Int => {
                match other {
                    Self::Int => true,
                    _ => false,
                }
            },
            Self::Float => {
                match other {
                    Self::Float => true,
                    _ => false,
                }
            },
            Self::Units(units) => {
                match other {
                    Self::Float => true,
                    Self::Units(ounits) => {
                        units == ounits
                    },
                    _ => false,
                }
            },
        }
    }
}
impl Eq for NumT {}
impl NumT {
    pub fn type_of(&self) -> ArcStr {
        match self {
            Self::Float => FLOAT,
            Self::Int => INT,
            Self::Units(units) => units.to_string(),
        }
    }
}
