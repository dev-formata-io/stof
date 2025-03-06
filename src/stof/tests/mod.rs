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

use crate::SDoc;

mod bstof;
mod export;


#[test]
fn stof_run_out() {
    let stof = r#"
        root Test: {
            throws: {
                #[main]
                fn main() {
                    throw('not implemented');
                }
            }
            doesnt: {
                #[main]
                fn main() {
                    try {
                        throw('not implemented');
                    } catch {
                        pln('caught'); 
                    }
                }
            }
        }
    "#;
    let mut doc = SDoc::src(stof, "stof").unwrap();
    let res = doc.run(None, None);
    match res {
        Ok(_) => {
            println!("stof document ran successfully");
        },
        Err(error) => {
            println!("{error}");
        }
    }
}
