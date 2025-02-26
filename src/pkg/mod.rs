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

use crate::{lang::SError, Format, SDoc, SField, SVal};


/// Stof pkg format interface.
pub struct PKG;
impl Format for PKG {
    /// Format identifier.
    fn format(&self) -> String {
        "pkg".to_string()
    }

    /// Package import.
    /// Looks for a "pkg.stof" file in the given directory and uses it to import additional files.
    fn file_import(&self, pid: &str, doc: &mut SDoc, _format: &str, full_path: &str, _extension: &str, as_name: &str) -> Result<(), SError> {
        let mut buf = full_path.split('.').collect::<Vec<&str>>();
        buf.pop();
        let cwd = buf.join(".");
        let path = format!("{}/pkg.stof", &cwd);
        let pkg = SDoc::file(&path, "stof")?;

        if let Some(field) = SField::field(&pkg.graph, "root.import", '.', None) {
            let mut pkg_format = "stof".to_string();
            match &field.value {
                SVal::String(path) => {
                    let pkg_path = format!("{}/{}", &cwd, path);
                    doc.file_import(pid, &pkg_format, &pkg_path, &pkg_format, as_name)?;
                },
                SVal::Object(nref) => {
                    if let Some(format_field) = SField::field(&pkg.graph, "format", '.', Some(nref)) {
                        pkg_format = format_field.to_string();
                    }
                    if let Some(path_field) = SField::field(&pkg.graph, "path", '.', Some(nref)) {
                        let pkg_path = format!("{}/{}", &cwd, path_field.to_string());
                        doc.file_import(pid, &pkg_format, &pkg_path, &pkg_format, as_name)?;
                    }
                },
                _ => {}
            }
            return Ok(());
        }
        if let Some(field) = SField::field(&pkg.graph, "root.imports", '.', None) {
            match &field.value {
                SVal::Array(vals) => {
                    for val in vals {
                        let mut pkg_format = "stof".to_string();
                        match val {
                            SVal::String(path) => {
                                let pkg_path = format!("{}/{}", &cwd, path);
                                doc.file_import(pid, &pkg_format, &pkg_path, &pkg_format, as_name)?;
                            },
                            SVal::Object(nref) => {
                                if let Some(format_field) = SField::field(&pkg.graph, "format", '.', Some(nref)) {
                                    pkg_format = format_field.to_string();
                                }
                                if let Some(path_field) = SField::field(&pkg.graph, "path", '.', Some(nref)) {
                                    let pkg_path = format!("{}/{}", &cwd, path_field.to_string());
                                    doc.file_import(pid, &pkg_format, &pkg_path, &pkg_format, as_name)?;
                                }
                            },
                            _ => {}
                        }
                    }
                },
                _ => {}
            }
            return Ok(());
        }
        Err(SError::custom(pid, doc, "PkgImportError", "package not found"))
    }
}
