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
fn export_simple_stof() {
    let json = r#"
    {
        "a": "A",
        "b": "B",
        "c": "C"
    }
    "#;
    let doc = SDoc::src(json, "json").unwrap();
    let _export = doc.export_string("stof", None).unwrap();
    //assert_eq!(export, "a: \"A\"\nb: \"B\"\nc: \"C\"");
}


#[test]
fn export_type() {
    let stof = r#"
        #[testexport]
        type MyType {
            x: m = 0
            y: mm
            z: km

            fn getX(): m { return self.x; }
        }

        root Geometry: {
            type Point {
                x: float = 0;
                y: float = 0;
                z: float = 0;

                fn len(): m {
                    let x = self.x as m;
                    let y = self.y as m;
                    let z = self.z as m;
                    return Number.sqrt(x.pow(2) + y.pow(2) + z.pow(2)).round(2);
                }
            }
        }
    "#;
    let doc = SDoc::src(stof, "stof").unwrap();
    let export = doc.export_bytes("bstof", None).unwrap();

    let mut import_doc = SDoc::bytes(export, "bstof").unwrap();
    import_doc.string_import("stof", r#"
        root Geometry: {
            type Point2D {
                x: float;
                y: float;

                fn len(): float {
                    let x = self.x;
                    let y = self.y;

                    // If x or y is a length, cast them to meters
                    if (x.hasUnits() && y.hasUnits()) {
                        if (!x.isLength() || !y.isLength()) {
                            throw('Cannot perform len on non-length units');
                        }
                        x = x as m;
                        y = y as m;
                    } else if (x.hasUnits()) {
                        if (!x.isLength()) {
                            throw('Cannot perform len on non-length units');
                        }
                        x = x as m;
                    } else if (y.hasUnits()) {
                        if (!y.isLength()) {
                            throw('Cannot perform len on non-length units');
                        }
                        y = y as m;
                    }

                    return Number.sqrt(x.pow(2) + y.pow(2)).round(2);
                }
            }
        }
        type Hello extends Geometry.Point {
            fn hello(): str {
                return `(${self.x}, ${self.y}, ${self.z})`;
            }
        }

        fn test(): str {
            let mytype = new MyType {
                y: 1,
                z: 1
            };
            assertEq(mytype.getX(), 0);

            let point = new Geometry.Point {
                x: 1m;
                y: 200cm;
                z: 3000mm;
            };
            assertEq(point.len(), 374cm);

            let point2 = new Geometry.Point2D {
                x: 2m; // 2s would error - no units would be just fine
                y: 4000000um;
            };
            assertEq(point2.len(), 4.47m);

            let hello = new Hello {
                y: 200cm;
                z: 3000mm;
            };
            hello.reference(point, 'x'); // The x field on point is now in 2 spots
            assertEq(hello.len(), 374cm);
            assertEq(hello.hello(), '(1m, 200cm, 3000mm)');

            hello.x = 3m;
            point.x = 2m; // We've just set the same field 2 times...
            assertEq(hello.len(), 4.12m);
            assertEq(point.len(), 412cm);

            point.remove('x'); // drop would remove from everywhere, remove just removes from the object
            point.x = 1m;
            assertEq(hello.len(), 4.12m);
            assertEq(point.len(), 374cm);

            return 'test';
        }
    "#, "").unwrap();
    let res = import_doc.call_func("test", None, vec![]).unwrap();
    assert_eq!(res, "test".into());
}


#[test]
fn export_stof_string_inner() {
    let stof = r#"
        name: 'CJ'
        age: 29
        male: true

        parent: {
            name: 'Jody'
            male: false
            arr: [1, 2, 'hi', { dude: false val: 3km }, { another: false inner: [{ messedup: true }]}]

            #[field]
            fn isFemale(): bool {
                return !self.male;
            }
        }

        #[another]
        #[dude]
        #[main]
        fn main(): str {
            let stof0 = stringify(self, 'stof');
            let stof = stringify(self, 'stof');
            assertEq(stof, stof0); // must be a predictable order of fields, etc...

            parse(stof, 'stof', 'self.check');
            assertEq(self.parent.name, self.check.parent.name);
            assertEq(self.parent.male, self.check.parent.male);
            assertEq(self.parent.arr[2], self.check.parent.arr[2]);
            assertEq(self.check.parent.arr[1], 2);

            return 'done';
        }
    "#;
    let mut doc = SDoc::src(stof, "stof").unwrap();
    let res = doc.call_func("main", None, vec![]).unwrap();
    assert_eq!(res, "done".into())
}
