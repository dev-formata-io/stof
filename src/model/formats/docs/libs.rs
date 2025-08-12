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

use arcstr::literal;
use crate::model::Graph;


/// Insert library documentation for each library.
pub fn insert_lib_documentation(graph: &mut Graph) {
    std_lib(graph);
    num_lib(graph);
    str_lib(graph);
    ver_lib(graph);
    blob_lib(graph);
    fn_lib(graph);
    obj_lib(graph);
    data_lib(graph);
    list_lib(graph);
    set_lib(graph);
    map_lib(graph);
    tup_lib(graph);
    
    md_lib(graph);
    time_lib(graph);
    fs_lib(graph);
    http_lib(graph);
    pdf_lib(graph);
    image_lib(graph);
}

/// Std.
fn std_lib(graph: &mut Graph) {
    graph.insert_libdoc(literal!("Std"), 
r#"# Standard Library (Std)
Functions in the 'Std' library are foundational to Stof and therefore do not requre one to explicitly reference 'Std' when calling. Within the standard library, you'll find functions for asserting values, printing to the console, throwing errors, putting processes to sleep, etc. Note for advanced users that it is possible to extend or modify this library as needed.

## Example Usage
```rust
#[main]
fn main() {
    Std.pln('printing a line');
    pln('printing another line'); // no 'Std' needed for this library
}
```
"#.into());
}

/// Num.
fn num_lib(graph: &mut Graph) {
    graph.insert_libdoc(literal!("Num"), 
r#"# Number Library (Num)
Library for manipulating and using numbers, automatically linked to the number types (int, float, & units).

## Example Usage
```rust
#[main]
fn main() {
    assert_eq(Num.abs(-23), 23);

    const val = -5;
    assert_eq(val.abs(), 5);
}
```
"#.into());
}

/// Str.
fn str_lib(graph: &mut Graph) {
    graph.insert_libdoc(literal!("Str"), 
r#"# String Library (Str)
Library for manipulating strings, automatically linked to the 'str' type.

## Example Usage
```rust
#[main]
fn main() {
    assert_eq("hello, world".split(", "), ['hello', 'world']);
}
```
"#.into());
}

/// Ver.
fn ver_lib(graph: &mut Graph) {
    graph.insert_libdoc(literal!("Ver"), 
r#"# Semantic Version Library (Ver)
Library for working with semantic versioning. Versions are a base type in Stof (ver).

## Example Usage
```rust
#[main]
fn main() {
    const version = 1.2.3;
    assert_eq(version.major(), 1);
    assert_eq(version.minor(), 2);
    assert_eq(version.patch(), 3);
    assert_eq(version as str, "1.2.3");
}
```
"#.into());
}

/// Blob.
fn blob_lib(graph: &mut Graph) {
    graph.insert_libdoc(literal!("Blob"), 
r#"# Blob Library (Blob)
Library for working with binary blobs (Vec\<u8> or Uint8Array), linked to the 'blob' type. Useful when working with web APIs, raw binary data, and in exchange scenarios between formats (see "blobify" in the Std library for more details).
"#.into());
}

/// Fn.
fn fn_lib(graph: &mut Graph) {
    graph.insert_libdoc(literal!("Fn"), 
r#"# Function Library (Fn)
Library for working with and calling functions, linked to the 'fn' type.

## Example Usage
```rust
#[main]
fn main() {
    const f = ():str => 'hello';
    assert_eq(Fn.call(f), 'hello');
    assert_eq(f.call(), 'hello');
    assert_eq(f(), 'hello');
}
```
"#.into());
}

/// Obj.
fn obj_lib(graph: &mut Graph) {
    graph.insert_libdoc(literal!("Obj"), 
r#"# Object Library (Obj)
Library that is linked to the 'obj' type.

## Example Usage
```rust
#[main]
fn main() {
    const o = new {};
    assert_eq(Obj.parent(o), self);
    assert_eq(o.parent(), self);
}
```
"#.into());
}

/// Data.
fn data_lib(graph: &mut Graph) {
    graph.insert_libdoc(literal!("Data"), 
r#"# Data Library (Data)
Library for working with opaque data pointers. If referenced explicitely, will work with custom data also, like PDFs, Images, etc.

## Example Usage
```rust
fn hi() -> str { 'hi' }

#[test]
fn main() {
    const o = new {};
    
    const func = self.hi;
    const dta = func.data();
    dta.attach(o);
    
    assert_eq(o.hi(), 'hi');
}
```
"#.into());
}

/// List.
fn list_lib(graph: &mut Graph) {
    graph.insert_libdoc(literal!("List"), 
r#"# List Library (List)
Library linked to the 'list' type.

## Example Usage
```rust
#[main]
fn main() {
    const array = [1, 2, 3];
    assert_eq(array.len(), 3);
    assert_eq(List.len(array), 3);
}
```
"#.into());
}

/// Set.
fn set_lib(graph: &mut Graph) {
    graph.insert_libdoc(literal!("Set"), 
r#"# Set Library (Set)
Library linked to the 'set' type.

## Example Usage
```rust
#[main]
fn main() {
    const a = {1, 2, 3};
    assert_eq(a.len(), 3);
    assert_eq(Set.len(a), 3);
}
```
"#.into());
}

/// Map.
fn map_lib(graph: &mut Graph) {
    graph.insert_libdoc(literal!("Map"), 
r#"# Map Library (Map)
Library linked to the 'map' type.

## Example Usage
```rust
#[main]
fn main() {
    const a = {1: 'hi'};
    assert_eq(a.len(), 1);
    assert_eq(Map.len(a), 1);
}
```
"#.into());
}

/// Tup.
fn tup_lib(graph: &mut Graph) {
    graph.insert_libdoc(literal!("Tup"), 
r#"# Tuple Library (Tup)
Library linked with the tuple type.

## Example Usage
```rust
#[main]
fn main() {
    const a = (1, 2);
    assert_eq(a[0], 1);
    assert_eq(a.at(1), 2);
    assert_eq(Tup.at(a, 1), 2);
}
```
"#.into());
}

/// Md.
fn md_lib(graph: &mut Graph) {
    graph.insert_libdoc(literal!("Md"), 
r#"# Markdown Library (Md)
Helper functions for common markdown operations, like turning Md strings into HTML.
"#.into());
}

/// Time.
fn time_lib(graph: &mut Graph) {
    graph.insert_libdoc(literal!("Time"), 
r#"# Time Library (Time)
Functions for working with time.
"#.into());
}

/// fs.
fn fs_lib(graph: &mut Graph) {
    graph.insert_libdoc(literal!("fs"), 
r#"# File System Library (fs)
Functions for working with the file system. Requires the "system" feature to automatically be added, otherwise, remove this library to further sandbox your environment.
"#.into());
}

/// Http.
fn http_lib(graph: &mut Graph) {
    graph.insert_libdoc(literal!("Http"), 
r#"# HTTP Network Library (Http)
Functions for working with HTTP over the network (async fetch, etc.).
"#.into());
}

/// Pdf.
fn pdf_lib(graph: &mut Graph) {
    graph.insert_libdoc(literal!("Pdf"), 
r#"# PDF Library (Pdf)
Functions for working with PDF files, loaded into Stof via the custom Data<Pdf> type.
"#.into());
}

/// Image.
fn image_lib(graph: &mut Graph) {
    graph.insert_libdoc(literal!("Image"), 
r#"# Image Library (Image)
Functions for working with images, loaded into Stof via the custom Data<Image> type. This can be done with several image formats.
"#.into());
}
