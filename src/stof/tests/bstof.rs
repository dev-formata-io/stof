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

use std::fs;
use crate::SDoc;


#[test]
fn hello_file_roundtrip() {
    {
        let mut doc = SDoc::file("src/stof/tests/hello.stof", "stof").unwrap();
        assert!(doc.bin_file_out("src/stof/tests/hello.bstof", "bstof").is_ok());
    }
    let mut doc = SDoc::file("src/stof/tests/hello.bstof", "bstof").unwrap();
    let res = doc.call_func("main", None, vec![]).unwrap();
    assert_eq!(res, "hello".into());
}


#[test]
fn loaded_bstof_types() {
    let saved_stof = r#"
    type CustomType {
        name: str = 'Bob';

        fn new(): CustomType {
            return new CustomType {};
        }

        fn static(): str {
            return 'hello, world';
        }
    }

    test: CustomType::new()
    "#;
    let mut saved = SDoc::src(saved_stof, "stof").unwrap();
    saved.bin_file_out("src/stof/tests/saved.bstof", "bstof").unwrap();

    {
        let mut doc = SDoc::default();
        doc.file_import("main", "bstof", "src/stof/tests/saved.bstof", "bstof", "").unwrap();

        let badge_stof = r#"
        fn access(): str {
            return 'This is a thing';
        }
        "#;
        doc.string_import("main", "stof", badge_stof, "Badge").unwrap();

        let req_stof = r#"
        #[main]
        fn run() {
            pln(CustomType::static());
            pln(root.test.static());
            pln(typename root.test);

            pln(CustomType::new());

            pln(Badge.access());
        }
        "#;
        doc.string_import("main", "stof", req_stof, "Request").unwrap();
        doc.run(None, None).unwrap();

        let bytes = doc.export_bytes("main", "bstof", doc.graph.main_root().as_ref()).unwrap();
        fs::write("src/stof/tests/saved.bstof", bytes).unwrap();
    }

    println!("\nSEP\n\n");
    {
        let mut doc = SDoc::default();
        doc.file_import("main", "bstof", "src/stof/tests/saved.bstof", "bstof", "").unwrap();

        let badge_stof = r#"
        fn access(): str {
            return 'This is a thing';
        }
        "#;
        doc.string_import("main", "stof", badge_stof, "Badge").unwrap();

        let req_stof = r#"
        #[main]
        fn run() {
            pln(CustomType::static());
            pln(root.test.static());
            pln(typename root.test);

            pln(CustomType::new());

            pln(Badge.access());
        }
        "#;
        doc.string_import("main", "stof", req_stof, "Request").unwrap();
        doc.run(None, None).unwrap();

        let bytes = doc.export_bytes("main", "bstof", doc.graph.main_root().as_ref()).unwrap();
        fs::write("src/stof/tests/saved.bstof", bytes).unwrap();
    }

    println!("\nSEP\n\n");
    {
        let mut doc = SDoc::default();
        doc.file_import("main", "bstof", "src/stof/tests/saved.bstof", "bstof", "").unwrap();

        let badge_stof = r#"
        fn access(): str {
            return 'This is a thing';
        }
        "#;
        doc.string_import("main", "stof", badge_stof, "Badge").unwrap();

        let req_stof = r#"
        #[main]
        fn run() {
            pln(CustomType::static());
            pln(root.test.static());
            pln(typename root.test);

            pln(CustomType::new());

            pln(Badge.access());
        }
        "#;
        doc.string_import("main", "stof", req_stof, "Request").unwrap();
        doc.run(None, None).unwrap();

        let bytes = doc.export_bytes("main", "bstof", doc.graph.main_root().as_ref()).unwrap();
        fs::write("src/stof/tests/saved.bstof", bytes).unwrap();
    }
}
