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

use colored::Colorize;
use nom::{branch::alt, bytes::complete::{tag, take_until}, character::complete::{char, space0}, combinator::opt, multi::separated_list1, sequence::{delimited, preceded, terminated}, IResult, Parser};
use crate::{model::{Data, SId}, parser::{context::ParseContext, doc::StofParseError, expr::blob_number, whitespace::whitespace}};


/// Parse binary data into the parse context.
pub fn parse_data<'a>(input: &'a str, context: &mut ParseContext) -> IResult<&'a str, (), StofParseError> {
    let (input, _) = whitespace(input)?;
    
    // Optionally start the data declaration with "data"
    let (input, version) = opt(preceded(tag("data"), opt(preceded(char('@'), take_until(" "))))).parse(input)?;
    let (input, _) = space0(input)?;

    // Get version of the data being parsed as a string
    if let Some(version) = version {
        if let Some(_version) = version {
            // Use this version in the future if/when binary data format changes
            //println!("DATA VERSION: {version}");
        }
    }

    // Parse the binary data like a normal blob
    let (input, bytes) = delimited(
        char('|'),
        terminated(separated_list1(char(','), blob_number), whitespace),
        alt((
            preceded(char(','), preceded(whitespace, char('|'))),
            char('|')
        ))
    ).parse(input)?;

    // Optionally end the binary data declaration with a semicolon or a comma
    let (input, _) = opt(preceded(space0, alt((char(';'), char(','))))).parse(input)?;

    let self_ptr = context.self_ptr();
    if let Ok(mut data) = bincode::deserialize::<Data>(&bytes) {
        // avoid colliding with existing data
        if data.id.data_exists(&context.graph) {
            data.id = SId::default();
        }

        // remove existing nodes (inserting a new data)
        data.nodes.clear();

        if let Some(_dref) = context.graph.insert_data(&self_ptr, data) {
            return Ok((input, ()));
        }
    }
    
    // Do not throw an error here - failing to load data should be allowed for various reasons...
    let path = self_ptr.node_path(&context.graph, true).unwrap().join(".");
    println!("{} {} {}", "failed to deserialize stof data @".dimmed(), path.purple(), "missing data".red());
    return Ok((input, ()));
}
