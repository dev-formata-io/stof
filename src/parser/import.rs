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

use nom::{branch::alt, bytes::complete::tag, character::complete::{char, multispace0}, combinator::{opt, recognize}, multi::separated_list1, sequence::{delimited, preceded}, IResult, Parser};
use crate::{model::{SELF_KEYWORD, SUPER_KEYWORD}, parser::{context::ParseContext, ident::ident, string::{double_string, single_string}, whitespace::whitespace}};


/// Parse an import statement into a graph.
pub fn import<'a>(input: &'a str, context: &mut ParseContext) -> IResult<&'a str, ()> {
    let (input, (format, path, scope)) = parse_import(input)?;

    let mut start = None;
    if scope.starts_with(SELF_KEYWORD.as_ref()) || scope.starts_with(SUPER_KEYWORD.as_ref()) {
        start = Some(context.self_ptr());
    }
    let node = context.graph.ensure_named_nodes(&scope, start, true, None);
    
    match context.parse_from_file(&format, &path, node) {
        Ok(_) => {
            Ok((input, ()))
        },
        Err(error) => { // TODO
            println!("{error:?}");
            return Err(nom::Err::Failure(nom::error::Error {
                input: "failed to import data",
                code: nom::error::ErrorKind::Fail
            }));
        }
    }
}


/// Parse an import statement (format, path, scope).
pub(self) fn parse_import(input: &str) -> IResult<&str, (String, String, String)> {
    let (input, _) = whitespace(input)?;
    let (input, _) = tag("import").parse(input)?;
    let (input, format) = opt(preceded(multispace0, ident)).parse(input)?;
    let (input, mut path) = preceded(multispace0, alt((single_string, double_string))).parse(input)?;
    let (input, scope) = opt(preceded(delimited(multispace0, alt((tag("as"), tag("on"))), multispace0), recognize(separated_list1(char('.'), ident)))).parse(input)?;
    let (input, _) = opt(preceded(multispace0, alt((char(';'), char(','))))).parse(input)?;

    path = path.trim().to_string();

    let mut res_format = "stof".to_string();
    if let Some(fmt) = format {
        res_format = fmt.to_string();
    } else {
        let path_list = path.trim_start_matches('.').split('.').collect::<Vec<_>>();
        if path_list.len() > 1 {
            res_format = path_list.last().unwrap().to_string();
        }
    }

    let mut res_scope = "self".to_string();
    if let Some(scp) = scope {
        res_scope = scp.to_string();
    }

    Ok((input, (res_format, path, res_scope)))
}


#[cfg(test)]
mod tests {
    use crate::parser::import::parse_import;

    #[test]
    fn basic_import() {
        let (_input, (format, path, scope)) = parse_import("\n\nimport './hello'\n\n").unwrap();
        assert_eq!(format, "stof");
        assert_eq!(path, "./hello");
        assert_eq!(scope, "root");
    }

    #[test]
    fn ext_import() {
        let (_input, (format, path, scope)) = parse_import("\n\nimport './hello.json'\n\n").unwrap();
        assert_eq!(format, "json");
        assert_eq!(path, "./hello.json");
        assert_eq!(scope, "root");
    }

    #[test]
    fn fmt_import() {
        let (_input, (format, path, scope)) = parse_import("\n\nimport pkg './hello.json'\n\n").unwrap();
        assert_eq!(format, "pkg");
        assert_eq!(path, "./hello.json");
        assert_eq!(scope, "root");
    }

    #[test]
    fn scope_import() {
        let (_input, (format, path, scope)) = parse_import("\n\nimport './hello.json' as self.Example;\n\n").unwrap();
        assert_eq!(format, "json");
        assert_eq!(path, "./hello.json");
        assert_eq!(scope, "self.Example");
    }

    #[test]
    fn together_import() {
        let (_input, (format, path, scope)) = parse_import("\n\nimport yaml \"src/dude/hello\" as Another.Sub.myobj;\n\n").unwrap();
        assert_eq!(format, "yaml");
        assert_eq!(path, "src/dude/hello");
        assert_eq!(scope, "Another.Sub.myobj");
    }
}
