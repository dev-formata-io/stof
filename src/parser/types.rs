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

use arcstr::literal;
use nom::{branch::alt, bytes::complete::tag, character::complete::{char, multispace0}, combinator::{map, opt, recognize, value}, multi::{separated_list0, separated_list1}, sequence::{delimited, preceded, terminated}, IResult, Parser};
use crate::{model::SId, parser::{doc::StofParseError, ident::ident_type}, runtime::{NumT, Type, Units}};


/// Parse type standalone parser.
pub fn parse_type_complete(input: &str) -> Result<Type, nom::Err<StofParseError>> {
    let res = parse_type(input)?;
    Ok(res.1)
}

/// Parse a string into a Type.
pub fn parse_type(input: &str) -> IResult<&str, Type, StofParseError> {
    let (input, mut res_type) = map((
        multispace0,
        alt((
            parse_union,
            value(Type::Null, tag("null")),
            value(Type::Void, tag("void")),
            value(Type::Num(NumT::Int), tag("int")),
            value(Type::Num(NumT::Float), tag("float")),
            value(Type::Str, tag("str")),
            value(Type::Prompt, tag("prompt")),
            value(Type::Ver, tag("ver")),
            value(Type::Blob, tag("blob")),
            value(Type::Bool, tag("bool")),
            value(Type::List, tag("list")),
            value(Type::Unknown, tag("unknown")),
            value(Type::Data(literal!("data")), tag("data")),
            alt((parse_fn_sig, value(Type::Fn, tag("fn")))),
            value(Type::Obj(SId::from("obj")), tag("obj")),
            value(Type::Set, tag("set")),
            value(Type::Map, tag("map")),
            parse_custom_data,
            parse_promise,
            parse_obj_or_units,
            alt((parse_group, parse_tuple)),
        ))
    ), |(_, ty)| ty).parse(input)?;

    // optionally a NotNull type?
    let (input, nn) = opt(char('!')).parse(input)?;
    if nn.is_some() { res_type = Type::NotNull(Box::new(res_type)); }

    Ok((input, res_type))
}

/// Inner types do not contain the Union as a possibility
/// Unions cannot contain unions, nor can tuples
fn parse_inner_union(input: &str) -> IResult<&str, Type, StofParseError> {
    map((
        multispace0,
        alt((
            value(Type::Null, tag("null")),
            value(Type::Void, tag("void")),
            value(Type::Num(NumT::Int), tag("int")),
            value(Type::Num(NumT::Float), tag("float")),
            value(Type::Str, tag("str")),
            value(Type::Prompt, tag("prompt")),
            value(Type::Ver, tag("ver")),
            value(Type::Blob, tag("blob")),
            value(Type::Bool, tag("bool")),
            value(Type::List, tag("list")),
            value(Type::Unknown, tag("unknown")),
            value(Type::Data(literal!("data")), tag("data")),
            alt((parse_fn_sig, value(Type::Fn, tag("fn")))),
            value(Type::Obj(SId::from("obj")), tag("obj")),
            value(Type::Set, tag("set")),
            value(Type::Map, tag("map")),
            parse_custom_data,
            parse_promise,
            parse_obj_or_units,
            alt((parse_group, parse_tuple)),
        )),
        opt(char('!')), // a not-null member (Ex. `int! | str`, `(int!, str)`)
    ), |(_, ty, nn)| if nn.is_some() { Type::NotNull(Box::new(ty)) } else { ty }).parse(input)
}

/// Parse object or units type.
fn parse_obj_or_units(input: &str) -> IResult<&str, Type, StofParseError> {
    let (input, parsed) = recognize(separated_list1(char('.'), ident_type)).parse(input)?;

    let units = Units::from(parsed);
    if units.has_units() && !units.is_undefined() {
        Ok((input, Type::Num(NumT::Units(units))))
    } else {
        Ok((input, Type::Obj(parsed.into())))
    }
}

/// Parse a type in parentheses, for grouping: `(fn(int) -> int) | str`, `fn(int) -> (int | null)`.
/// One type in parentheses is that type; two or more are a tuple (`(int, str)`).
fn parse_group(input: &str) -> IResult<&str, Type, StofParseError> {
    delimited(char('('), parse_type, preceded(multispace0, char(')'))).parse(input)
}

/// Parse tuple type.
fn parse_tuple(input: &str) -> IResult<&str, Type, StofParseError> {
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
fn parse_union(input: &str) -> IResult<&str, Type, StofParseError> {
    map(
        parse_union_list,
        |list| Type::Union(list.into_iter().collect())
    ).parse(input)
}
fn parse_union_list(input: &str) -> IResult<&str, Vec<Type>, StofParseError> {
    let mut parser = separated_list1(preceded(multispace0, tag("|")), parse_inner_union);
    let (input, vals) = parser.parse(input)?;
    if vals.len() < 2 {
        Err(nom::Err::Error(StofParseError::from(format!("union type must have at least 2 values: {vals:?}"))))
    } else {
        Ok((input, vals))
    }
}

/// Parse a function signature type: `fn(str, int) -> str`, or `fn(str)` for any return type.
/// The return type takes the rest of the type (`fn(int) -> int | null` returns an int or null).
fn parse_fn_sig(input: &str) -> IResult<&str, Type, StofParseError> {
    let (input, _) = tag("fn").parse(input)?;
    let (input, params) = preceded(multispace0, delimited(
        char('('),
        separated_list0(delimited(multispace0, char(','), multispace0), parse_type),
        preceded(multispace0, char(')'))
    )).parse(input)?;
    let (input, ret) = opt(preceded(delimited(multispace0, tag("->"), multispace0), parse_type)).parse(input)?;
    Ok((input, Type::FnSig(params.into_iter().collect(), Box::new(ret.unwrap_or(Type::Unknown)))))
}

/// Parse custom data type.
fn parse_custom_data(input: &str) -> IResult<&str, Type, StofParseError> {
    map(
        delimited(tag("Data<"), ident_type, char('>')),
        |ct| Type::Data(ct.into())
    ).parse(input)
}

/// Parse promise type.
fn parse_promise(input: &str) -> IResult<&str, Type, StofParseError> {
    map(
        delimited(tag("Promise<"), parse_inner_promise, char('>')),
        |ct| Type::Promise(ct.into())
    ).parse(input)
}
fn parse_inner_promise(input: &str) -> IResult<&str, Type, StofParseError> {
    alt((
        parse_union,
        value(Type::Null, tag("null")),
        value(Type::Void, tag("void")),
        value(Type::Num(NumT::Int), tag("int")),
        value(Type::Num(NumT::Float), tag("float")),
        value(Type::Str, tag("str")),
        value(Type::Prompt, tag("prompt")),
        value(Type::Ver, tag("ver")),
        value(Type::Blob, tag("blob")),
        value(Type::Bool, tag("bool")),
        value(Type::List, tag("list")),
        value(Type::Unknown, tag("unknown")),
        value(Type::Data(literal!("data")), tag("data")),
        alt((parse_fn_sig, value(Type::Fn, tag("fn")))),
        value(Type::Obj(SId::from("obj")), tag("obj")),
        value(Type::Set, tag("set")),
        value(Type::Map, tag("map")),
        parse_custom_data,
        parse_obj_or_units,
        alt((parse_group, parse_tuple)),
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
    fn parse_fn_signatures() {
        let int = Type::Num(NumT::Int);
        assert_eq!(parse_type_complete("fn").unwrap(), Type::Fn);
        assert_eq!(parse_type_complete("fn()").unwrap(), Type::FnSig(vector![], Box::new(Type::Unknown)));
        assert_eq!(parse_type_complete("fn(str) -> str").unwrap(), Type::FnSig(vector![Type::Str], Box::new(Type::Str)));
        assert_eq!(parse_type_complete("fn( int ,str )->void").unwrap(), Type::FnSig(vector![int.clone(), Type::Str], Box::new(Type::Void)));
        // the return type takes the rest of the type
        assert_eq!(parse_type_complete("fn(int) -> int | null").unwrap(), Type::FnSig(vector![int.clone()], Box::new(Type::Union(vector![int.clone(), Type::Null]))));
        assert_eq!(parse_type_complete("int | fn(str)").unwrap(), Type::Union(vector![int.clone(), Type::FnSig(vector![Type::Str], Box::new(Type::Unknown))]));
        assert_eq!(parse_type_complete("fn(fn(int) -> int, int) -> int").unwrap(), Type::FnSig(vector![Type::FnSig(vector![int.clone()], Box::new(int.clone())), int.clone()], Box::new(int.clone())));
        assert_eq!(parse_type_complete("(fn(int), str)").unwrap(), Type::Tup(vector![Type::FnSig(vector![int.clone()], Box::new(Type::Unknown)), Type::Str]));
        assert_eq!(parse_type_complete("Promise<fn(str) -> str>").unwrap(), Type::Promise(Box::new(Type::FnSig(vector![Type::Str], Box::new(Type::Str)))));
        // written back the same way
        assert_eq!(parse_type_complete("fn(int, str) -> int | null").unwrap().type_of(), "fn(int, str) -> int | null");
        assert_eq!(parse_type_complete("fn(str)").unwrap().type_of(), "fn(str)");
    }

    #[test]
    fn parse_groups() {
        let int = Type::Num(NumT::Int);
        assert_eq!(parse_type_complete("(int)").unwrap(), int.clone());
        assert_eq!(parse_type_complete("( str | int )").unwrap(), Type::Union(vector![Type::Str, int.clone()]));
        assert_eq!(parse_type_complete("(fn(int) -> int) | str").unwrap(), Type::Union(vector![Type::FnSig(vector![int.clone()], Box::new(int.clone())), Type::Str]));
        assert_eq!(parse_type_complete("fn(int) -> (int | null)").unwrap(), Type::FnSig(vector![int.clone()], Box::new(Type::Union(vector![int.clone(), Type::Null]))));
        assert_eq!(parse_type_complete("((fn(str)), int)").unwrap(), Type::Tup(vector![Type::FnSig(vector![Type::Str], Box::new(Type::Unknown)), int.clone()]));
        assert_eq!(parse_type_complete("(int, str)").unwrap(), Type::Tup(vector![int.clone(), Type::Str]));
        assert!(matches!(parse_type_complete("(int)!").unwrap(), Type::NotNull(inner) if matches!(*inner, Type::Num(NumT::Int))));
    }

    #[test]
    fn parse_not_null_members() {
        let nn = |ty: Type| Type::NotNull(Box::new(ty));
        let shape = |ty: &Type| format!("{ty:?}");
        let int = Type::Num(NumT::Int);
        assert_eq!(shape(&parse_type_complete("(int!, str)").unwrap()), shape(&Type::Tup(vector![nn(int.clone()), Type::Str])));
        assert_eq!(shape(&parse_type_complete("int! | str!").unwrap()), shape(&Type::Union(vector![nn(int.clone()), nn(Type::Str)])));
        assert_eq!(shape(&parse_type_complete("int | str!").unwrap()), shape(&Type::Union(vector![int.clone(), nn(Type::Str)])));
        assert_eq!(shape(&parse_type_complete("(int | str)!").unwrap()), shape(&nn(Type::Union(vector![int.clone(), Type::Str]))));
        assert_eq!(shape(&parse_type_complete("fn(int!) -> str!").unwrap()), shape(&Type::FnSig(vector![nn(int.clone())], Box::new(nn(Type::Str)))));
        assert_eq!(shape(&parse_type_complete("int!").unwrap()), shape(&nn(int.clone())));
        // written back so they parse the same
        for text in ["(int!, str)", "int! | str!", "(int | str)!", "fn(int!) -> str!", "int!", "(fn(str) -> str) | int",
                     "fn(str) | int", "int | (fn(int) -> int)", "fn(int) -> int | null", "(fn(int!) -> int, str!)"] {
            let ty = parse_type_complete(text).unwrap();
            assert_eq!(ty.type_of(), text);
            assert_eq!(shape(&parse_type_complete(&ty.type_of()).unwrap()), shape(&ty), "{text}");
        }
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
