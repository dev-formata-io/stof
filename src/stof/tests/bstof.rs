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


#[test]
fn hello_file_roundtrip() {
    {
        let doc = SDoc::file("src/stof/tests/hello.stof", "stof").unwrap();
        assert!(doc.bin_file_out("src/stof/tests/hello.bstof", "bstof").is_ok());
    }
    let mut doc = SDoc::file("src/stof/tests/hello.bstof", "bstof").unwrap();
    let res = doc.run(None).pop().unwrap().1;
    assert_eq!(res, "hello".into());
}