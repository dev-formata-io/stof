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
fn stof_assert_false() {
    let stof = r#"
        #[test]
        #[silent]
        fn assert_test() {
            assert(false);
        }
    "#;
    let mut doc = SDoc::src(&stof, "stof").unwrap();
    let res = doc.run_tests(true, None);
    assert!(res.is_err());
}


#[test]
fn stof_assert_true() {
    let stof = r#"
        #[test]
        #[silent]
        fn assert_test() {
            assert(true);
        }
    "#;
    let mut doc = SDoc::src(&stof, "stof").unwrap();
    let res = doc.run_tests(true, None);
    assert!(res.is_ok());
}


#[test]
fn stof_assert_eq_true() {
    let stof = r#"
        #[test]
        #[silent]
        fn assert_test() {
            assertEq(false, true);
        }
    "#;
    let mut doc = SDoc::src(&stof, "stof").unwrap();
    let res = doc.run_tests(true, None);
    assert!(res.is_err());
}


#[test]
fn stof_assert_eq_false() {
    let stof = r#"
        #[test]
        #[silent]
        fn assert_test() {
            assertEq(false, false);
        }
    "#;
    let mut doc = SDoc::src(&stof, "stof").unwrap();
    let res = doc.run_tests(true, None);
    assert!(res.is_ok());
}


#[test]
fn stof_test_suite() {
    SDoc::test_file("src/tests/tests.stof", true);
}