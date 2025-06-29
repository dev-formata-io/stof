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
use nom::{branch::alt, bytes::complete::tag, character::complete::{char, multispace0}, combinator::{map, peek, value}, error::{Error, ErrorKind}, multi::{separated_list0, separated_list1}, sequence::delimited, IResult, Parser};
use serde::{Deserialize, Serialize};
use crate::{parser::ident, runtime::Units};


#[derive(Debug, Clone, Deserialize, Serialize, Default)]
/// Type.
pub enum Type {
    #[default]
    Void,
    Null,

    Bool,
    Num(NumT),
    Str,
    Ver,

    Obj(ArcStr),
    Fn,
    Data(ArcStr),

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
                    _ => false,
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
            Self::Unknown => literal!("unknown"),
            Self::Map => literal!("map"),
            Self::Set => literal!("set"),
            Self::List => literal!("list"),
            Self::Bool => literal!("bool"),
            Self::Blob => literal!("blob"),
            Self::Fn => literal!("fn"),
            Self::Data(tname) => {
                let dta = literal!("data");
                if tname == &dta {
                    return dta;
                }
                format!("Data<{}>", tname).into()
            },
            Self::Null => literal!("null"),
            Self::Num(num) => num.type_of(),
            Self::Ver => literal!("ver"),
            Self::Str => literal!("str"),
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
            Self::Void => literal!("void"),
            Self::Obj(ctype) => ctype.clone(),
        }
    }

    pub fn md_type_of(&self) -> String {
        self.type_of().replace("<", "\\<")
    }
}


#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
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
            Self::Float => literal!("float"),
            Self::Int => literal!("int"),
            Self::Units(units) => units.to_string(),
        }
    }
}

/// Parse type standalone parser.
pub fn parse_type_complete(input: &str) -> Result<Type, nom::Err<nom::error::Error<&str>>> {
    let res = parse_type(input)?;
    Ok(res.1)
}

/// Parse a string into a Type.
pub fn parse_type(input: &str) -> IResult<&str, Type> {
    map((
        multispace0,
        alt((
            parse_custom_data,
            parse_union,
            value(Type::Null, tag("null")),
            value(Type::Void, tag("void")),
            value(Type::Num(NumT::Int), tag("int")),
            value(Type::Num(NumT::Float), tag("float")),
            value(Type::Str, tag("str")),
            value(Type::Ver, tag("ver")),
            value(Type::Blob, tag("blob")),
            value(Type::Bool, tag("bool")),
            value(Type::List, tag("list")),
            value(Type::Unknown, tag("unknown")),
            value(Type::Data(literal!("data")), tag("data")),
            value(Type::Fn, tag("fn")),
            value(Type::Obj(literal!("obj")), tag("obj")),
            value(Type::Set, tag("set")),
            value(Type::Map, tag("map")),
            parse_units,
            parse_obj,
            parse_tuple,
        )),
        multispace0
    ), |(_, ty, _)| ty).parse(input)
}

fn parse_inner_type(input: &str) -> IResult<&str, Type> {
    map((
        multispace0,
        alt((
            parse_custom_data,
            value(Type::Null, tag("null")),
            value(Type::Void, tag("void")),
            value(Type::Num(NumT::Int), tag("int")),
            value(Type::Num(NumT::Float), tag("float")),
            value(Type::Str, tag("str")),
            value(Type::Ver, tag("ver")),
            value(Type::Blob, tag("blob")),
            value(Type::Bool, tag("bool")),
            value(Type::List, tag("list")),
            value(Type::Unknown, tag("unknown")),
            value(Type::Data(literal!("data")), tag("data")),
            value(Type::Fn, tag("fn")),
            value(Type::Obj(literal!("obj")), tag("obj")),
            value(Type::Set, tag("set")),
            value(Type::Map, tag("map")),
            parse_units,
            parse_obj,
            parse_tuple,
        )),
        multispace0
    ), |(_, ty, _)| ty).parse(input)
}

fn parse_units(input: &str) -> IResult<&str, Type> {
    let units = Units::from(input);
    if units.has_units() && !units.is_undefined() {
        Ok(("", Type::Num(NumT::Units(units))))
    } else {
        Err(nom::Err::Error(Error { input, code: ErrorKind::IsNot }))
    }
}

fn parse_obj(input: &str) -> IResult<&str, Type> {
    map(
        ident,
        |res| Type::Obj(res.into())
    ).parse(input)
}

fn parse_tuple(input: &str) -> IResult<&str, Type> {
    map(
        delimited(
            (char('('), multispace0),
            separated_list0((multispace0, tag(","), multispace0), parse_type),
            (multispace0, char(')'))
        ),
        |list| Type::Tup(list.into_iter().collect())
    ).parse(input)
}

fn parse_union(input: &str) -> IResult<&str, Type> {
    peek(map(
        separated_list1(tag("|"), parse_inner_type),
        |list| Type::Union(list.into_iter().collect())
    )).parse(input)
}

fn parse_custom_data(input: &str) -> IResult<&str, Type> {
    map(
        delimited(tag("Data<"), ident, char('>')),
        |ct| Type::Data(ct.into())
    ).parse(input)
}


#[cfg(test)]
mod tests {
    use imbl::vector;
    use crate::runtime::{parse_type_complete, types::parse_tuple, NumT, Type, Units};

    #[test]
    fn parse_custom_data() {
        assert_eq!(parse_type_complete("Data<PDF>").unwrap(), Type::Data("PDF".into()));
        assert_eq!(parse_type_complete("Data<Image>").unwrap(), Type::Data("Image".into()));
    }

    #[test]
    fn parse_tuples() {
        //println!("{:?}", parse_tuple("(int, str)"));
        //assert_eq!(parse_type_complete("(int,str)").unwrap(), Type::Tup(vector![Type::Num(NumT::Int), Type::Str]));
    }

    #[test]
    fn parse_littypes() {
        assert_eq!(parse_type_complete("   null    ").unwrap(), Type::Null);
        assert_eq!(parse_type_complete(" null").unwrap(), Type::Null);
        assert_eq!(parse_type_complete("null    ").unwrap(), Type::Null);

        assert_eq!(parse_type_complete("void").unwrap(), Type::Void);
        assert_eq!(parse_type_complete("bool").unwrap(), Type::Bool);

        assert_eq!(parse_type_complete("int").unwrap(), Type::Num(NumT::Int));
        assert_eq!(parse_type_complete("float").unwrap(), Type::Num(NumT::Float));
        assert_eq!(parse_type_complete("ms").unwrap(), Type::Num(NumT::Units(Units::Milliseconds)));
        
        assert_eq!(parse_type_complete("str").unwrap(), Type::Str);
        assert_eq!(parse_type_complete("ver").unwrap(), Type::Ver);
        assert_eq!(parse_type_complete("obj").unwrap(), Type::Obj("obj".into()));
        assert_eq!(parse_type_complete("fn").unwrap(), Type::Fn);
        assert_eq!(parse_type_complete("data").unwrap(), Type::Data("data".into()));
        assert_eq!(parse_type_complete("blob").unwrap(), Type::Blob);
        assert_eq!(parse_type_complete("list").unwrap(), Type::List);
        assert_eq!(parse_type_complete("map").unwrap(), Type::Map);
        assert_eq!(parse_type_complete("set").unwrap(), Type::Set);
        assert_eq!(parse_type_complete("unknown").unwrap(), Type::Unknown);

        assert_eq!(parse_type_complete("CustomType").unwrap(), Type::Obj("CustomType".into()));
    }
}
