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

use nom::{bytes::complete::tag, character::complete::multispace0, combinator::opt, sequence::terminated, IResult, Parser};
use crate::{model::{Func, ASYNC_FUNC_ATTR}, parser::whitespace::whitespace, runtime::Val};


/// Parse a function.
pub fn parse_function(input: &str) -> IResult<&str, Func> {
    let mut func = Func::default();

    // TODO optional attributes before doc comments
    // TODO doc comments before whitespace

    let (input, _) = whitespace(input)?;
    // TODO optional attributes before func def

    let (input, async_fn) = opt(terminated(tag("async"), multispace0)).parse(input)?;
    if async_fn.is_some() && !func.attributes.contains_key(&ASYNC_FUNC_ATTR) {
        func.attributes.insert(ASYNC_FUNC_ATTR.clone(), Val::Void);
    }

    

    Ok((input, func))
}
