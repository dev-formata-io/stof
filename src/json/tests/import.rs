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
fn import_json_file() {
    let mut doc = SDoc::file("src/json/tests/example.json", "json").unwrap();
    assert_eq!(doc.get("glossary.self.GlossDiv.super.GlossDiv.GlossList.GlossEntry.SortAs", None).unwrap(), "SGML".into());
}


#[test]
fn parse_json() {
    let stof = r#"
        #[main]
        fn main(): str {
            let json = '
            {
                "test": "hello",
                "age": 29
            }
            ';

            assertEq(self.test, null);

            // Can check that the required format exists in the Doc
            if (!hasFormat('json')) {
                return 'Does not have the required format';
            }
            assert(formats().contains('toml')); // formats() is a vec of available formats for this Doc

            // Loads the JSON in the current node using the 'json' format
            parse(json, 'json');

            assertEq(self.test, 'hello');
            assertEq(self.age, 29);

            let array = blobify(self, 'json');   // UTF-8 JSON string vec<u8>
            let array_str = array as str;        // default casting between blob and str is UTF-8
            assertEq(array_str, '{"age":29,"test":"hello"}');
            assertEq(array, array_str as blob);

            parse(array, 'json', 'self.import'); // Will create the child 'import' from the current location
            assertEq(self.import.test, 'hello');
            assertEq(self.import.age, 29);

            return stringify(self, 'json');
        }
    "#;

    let mut doc = SDoc::src(stof, "stof").unwrap();
    let res = doc.call_func("main", None, vec![]).unwrap();
    assert_eq!(res, "{\"age\":29,\"import\":{\"age\":29,\"test\":\"hello\"},\"test\":\"hello\"}".into());
}


#[test]
fn parse_stof() {
    let stof = r#"
        #[main]
        fn main(): str {
            let src = '
                name: "CJ"
                age: 29
                fn getAge(): int { return self.age; }
            ';
            parse(src, 'stof');
            
            return `${self.name} is ${self.getAge()} years old`;
        }
    "#;

    let mut doc = SDoc::src(stof, "stof").unwrap();
    let res = doc.call_func("main", None, vec![]).unwrap();
    assert_eq!(res, "CJ is 29 years old".into());
}
