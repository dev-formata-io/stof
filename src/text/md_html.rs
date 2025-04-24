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

use markdown::to_html;
use crate::{lang::SError, Library, SDoc, SVal};


#[derive(Default)]
pub struct MDHTMLLibrary;
impl Library for MDHTMLLibrary {
    fn scope(&self) -> String {
        "Markdown".to_string()
    }

    fn call(&self, pid: &str, doc: &mut SDoc, name: &str, parameters: &mut Vec<SVal>) -> Result<SVal, SError> {
        match name {
            "toHTML" => {
                if parameters.len() > 0 {
                    let md = parameters.pop().unwrap().owned_to_string();
                    return Ok(SVal::String(to_html(&md)));
                }
                Err(SError::custom(pid, &doc, "MarkdownError", "Markdown.toHTML requires a string (markdown) parameter"))
            },
            _ => {
                Err(SError::custom(pid, &doc, "MarkdownError", &format!("{name} is not a function in the Markdown library")))
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::SDoc;

    #[test]
    fn md_to_html() {
        let mut doc = SDoc::src(r#"
            #[test]
            fn md_to_html() {
                let md = '## Hi, *sun*!';
                let html = Markdown.toHTML(md);
                assertEq(html, "<h2>Hi, <em>sun</em>!</h2>");
            }
        "#, "stof").unwrap();
        let res = doc.run_tests(true, None);
        match res {
            Ok(res) => {
                println!("{res}");
            },
            Err(err) => {
                panic!("{err}");
            }
        }
    }
}
