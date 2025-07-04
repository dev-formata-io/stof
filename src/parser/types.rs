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

use arcstr::literal;
use nom::{branch::alt, bytes::complete::tag, character::complete::{char, multispace0}, combinator::{map, value}, error::{Error, ErrorKind}, multi::separated_list1, sequence::{delimited, preceded, terminated}, IResult, Parser};
use crate::{model::SId, parser::ident::ident, runtime::{NumT, Type, Units}};


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
            value(Type::Obj(SId::from("obj")), tag("obj")),
            value(Type::Set, tag("set")),
            value(Type::Map, tag("map")),
            parse_custom_data,
            parse_promise,
            parse_units,
            parse_obj,
            parse_tuple,
        ))
    ), |(_, ty)| ty).parse(input)
}

/// Inner types do not contain the Union as a possibility
/// Unions cannot contain unions, nor can tuples
fn parse_inner_union(input: &str) -> IResult<&str, Type> {
    map((
        multispace0,
        alt((
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
            value(Type::Obj(SId::from("obj")), tag("obj")),
            value(Type::Set, tag("set")),
            value(Type::Map, tag("map")),
            parse_custom_data,
            parse_promise,
            parse_units,
            parse_obj,
            parse_tuple,
        )),
        multispace0
    ), |(_, ty, _)| ty).parse(input)
}

/// Parse unit type.
fn parse_units(input: &str) -> IResult<&str, Type> {
    let units = Units::from(input);
    if units.has_units() && !units.is_undefined() {
        Ok(("", Type::Num(NumT::Units(units))))
    } else {
        Err(nom::Err::Error(Error { input, code: ErrorKind::IsNot }))
    }
}

/// Parse object type.
fn parse_obj(input: &str) -> IResult<&str, Type> {
    map(
        ident,
        |res| Type::Obj(res.into())
    ).parse(input)
}

/// Parse tuple type.
fn parse_tuple(input: &str) -> IResult<&str, Type> {
    map(
        delimited(
            preceded(char('('), multispace0),
            separated_list1(
                delimited(multispace0, char(','), multispace0),
                parse_inner_union
            ),
            terminated(multispace0, char(')'))
        ),
        |list| Type::Tup(list.into_iter().collect())
    ).parse(input)
}

/// Parse union type.
fn parse_union(input: &str) -> IResult<&str, Type> {
    map(
        parse_union_list,
        |list| Type::Union(list.into_iter().collect())
    ).parse(input)
}
fn parse_union_list(input: &str) -> IResult<&str, Vec<Type>> {
    let mut parser = separated_list1(tag("|"), parse_inner_union);
    let (input, vals) = parser.parse(input)?;
    if vals.len() < 2 {
        Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::NoneOf)))
    } else {
        Ok((input, vals))
    }
}

/// Parse custom data type.
fn parse_custom_data(input: &str) -> IResult<&str, Type> {
    map(
        delimited(tag("Data<"), ident, char('>')),
        |ct| Type::Data(ct.into())
    ).parse(input)
}

/// Parse promise type.
fn parse_promise(input: &str) -> IResult<&str, Type> {
    map(
        delimited(tag("Promise<"), parse_inner_promise, char('>')),
        |ct| Type::Promise(ct.into())
    ).parse(input)
}
fn parse_inner_promise(input: &str) -> IResult<&str, Type> {
    alt((
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
        value(Type::Obj(SId::from("obj")), tag("obj")),
        value(Type::Set, tag("set")),
        value(Type::Map, tag("map")),
        parse_custom_data,
        parse_units,
        parse_obj,
        parse_tuple,
    )).parse(input)
}


#[cfg(test)]
mod tests {
    use arcstr::{literal, ArcStr};
    use imbl::vector;
    use crate::{model::SId, parser::types::parse_type_complete, runtime::{NumT, Type, Units}};

    #[test]
    fn from_str() {
        assert_eq!(Type::from("str"), Type::Str);
        assert_eq!(Type::from("\n\t\t\t\tbool\n\t\n\n\r"), Type::Bool);
        assert_eq!(Type::from("ms|seconds|ns"), Type::Union(vector![
            Type::Num(NumT::Units(Units::Milliseconds)),
            Type::Num(NumT::Units(Units::Seconds)),
            Type::Num(NumT::Units(Units::Nanoseconds))
        ]));
        assert_eq!(Type::from(literal!("blob")), Type::Blob);
        assert_eq!(Type::from(String::from("fn")), Type::Fn);
    }

    #[test]
    fn parse_custom_data() {
        assert_eq!(parse_type_complete("Data<PDF>").unwrap(), Type::Data("PDF".into()));
        assert_eq!(parse_type_complete("Data<Image>").unwrap(), Type::Data("Image".into()));
    }

    #[test]
    fn parse_tuples() {
        assert_eq!(parse_type_complete("(int,str)").unwrap(), Type::Tup(vector![Type::Num(NumT::Int), Type::Str]));
        assert_eq!(parse_type_complete("(  str    \n,  \n\tstr    )").unwrap(), Type::Tup(vector![Type::Str, Type::Str]));
        assert_eq!(parse_type_complete("((bool, (str, str), blob), fn)").unwrap(), Type::Tup(vector![Type::Tup(vector![Type::Bool, Type::Tup(vector![Type::Str, Type::Str]), Type::Blob]), Type::Fn]));
    }

    #[test]
    fn parse_union() {
        assert_eq!(parse_type_complete("int | str").unwrap(), Type::Union(vector![Type::Num(NumT::Int), Type::Str]));
        assert_eq!(parse_type_complete("(bool, fn, blob) \n\t\t| str    \n | fn\n\n").unwrap(), Type::Union(vector![Type::Tup(vector![Type::Bool, Type::Fn, Type::Blob]), Type::Str, Type::Fn]));
        assert_eq!(parse_type_complete("bool|blob|fn|str").unwrap(), Type::Union(vector![Type::Bool, Type::Blob, Type::Fn, Type::Str]));
    }

    #[test]
    fn parse_littypes() {
        assert_eq!(parse_type_complete("int, ").unwrap(), Type::Num(NumT::Int));
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

    #[test]
    fn parse_promise_works() {
        assert_eq!(parse_type_complete("Promise<str>").unwrap(), Type::Promise(Box::new(Type::Str)));
        assert_eq!(parse_type_complete("Promise<Data<PDF>>").unwrap(), Type::Promise(Box::new(Type::Data(ArcStr::from("PDF")))));
        assert_eq!(parse_type_complete("Promise<str | bool | int>").unwrap(), Type::Promise(Box::new(Type::Union(vector![Type::Str, Type::Bool, Type::Num(NumT::Int)]))));
        assert_eq!(parse_type_complete("Promise<(str, bool, blob)>").unwrap(), Type::Promise(Box::new(Type::Tup(vector![Type::Str, Type::Bool, Type::Blob]))));
    }

    #[test]
    fn type_equality() {
        assert_eq!(Type::Unknown, Type::Null);
        assert_eq!(Type::Unknown, Type::Bool);
        assert_eq!(Type::Unknown, Type::Obj(SId::from("obj")));
        assert_eq!(Type::Null, Type::Unknown);
        assert_eq!(Type::Bool, Type::Unknown);
        assert_eq!(Type::Obj(SId::from("obj")), Type::Unknown);
        assert_eq!(Type::Unknown, Type::Union(vector![Type::Bool, Type::Str, Type::Blob]));

        assert_eq!(Type::Bool, Type::Union(vector![Type::Bool, Type::Str, Type::Blob]));
        assert_eq!(Type::Str, Type::Union(vector![Type::Bool, Type::Str, Type::Blob]));
        assert_ne!(Type::Ver, Type::Union(vector![Type::Bool, Type::Str, Type::Blob]));

        assert_eq!(Type::Bool, Type::Promise(Box::new(Type::Bool)));
        assert_eq!(Type::Promise(Box::new(Type::Bool)), Type::Promise(Box::new(Type::Bool)));
        assert_eq!(Type::Promise(Box::new(Type::Bool)), Type::Bool);
        assert_ne!(Type::Promise(Box::new(Type::Bool)), Type::Promise(Box::new(Type::Str)));
        assert_ne!(Type::Promise(Box::new(Type::Bool)), Type::Str);
    }
}
