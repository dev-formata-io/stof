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

use std::{collections::HashSet, fs::{self, File}, io::{self, Read, Seek, Write}, path::{Path, PathBuf}};
use anyhow::Context;
use bytes::Bytes;
use nanoid::nanoid;
use regex::Regex;
use walkdir::{DirEntry, WalkDir};
use zip::write::SimpleFileOptions;
use crate::{model::{Field, Format, Graph, NodeRef, SELF_KEYWORD, SUPER_KEYWORD}, parser::context::ParseContext, runtime::{Error, Val}};


#[derive(Debug, Clone)]
/// .pkg format.
pub struct StofPackageFormat {
    pub tmp: String,
}
impl Default for StofPackageFormat {
    fn default() -> Self {
        Self {
            tmp: format!("{}/__stoftmp__", std::env::temp_dir().display()),
        }
    }
}
impl StofPackageFormat {
    /// Remove file helper.
    pub fn remove(path: &str) -> Result<(), std::io::Error> {
        fs::remove_file(path)
    }


    /*****************************************************************************
     * Create.
     *****************************************************************************/
    
    /// Create package file at a destination path.
    /// If successful, returns a path to the newly created zip file path, otherwise None.
    pub fn create_package_file(dir_path: &str, dest_path: &str, included: &HashSet<String>, excluded: &HashSet<String>) -> Option<String> {
        let mut path = dest_path.to_string();
        if !path.ends_with(".pkg") { path = format!("{}.pkg", path); }

        // Make sure the destination directory exists
        let mut dir_pth_buf = path.split('/').collect::<Vec<&str>>();
        dir_pth_buf.pop();
        let dir_pth = dir_pth_buf.join("/");
        if dir_pth.len() > 0 { let _ = fs::create_dir_all(&dir_pth); }

        let file = fs::File::create(&path).unwrap();
        let walkdir = WalkDir::new(dir_path);
        let iter = walkdir.into_iter();
        let res = Self::zip_directory(&mut iter.filter_map(|e| e.ok()), dir_path, file, zip::CompressionMethod::Bzip2, included, excluded);
        if res.is_err() {
            return None;
        }
        return Some(path);
    }

    /// Create a temp package file.
    pub fn create_temp_package_file(&self, dir_path: &str, included: &HashSet<String>, excluded: &HashSet<String>) -> Option<String> {
        let _ = fs::create_dir_all(&self.tmp);
        let path = format!("{}/{}.pkg", self.tmp, nanoid!());
        Self::create_package_file(dir_path, &path, included, excluded)
    }

    /// Zip the directory into an output file.
    fn zip_directory<T: Write + Seek>(iter: &mut dyn Iterator<Item = DirEntry>, prefix: &str, writer: T, method: zip::CompressionMethod, included: &HashSet<String>, excluded: &HashSet<String>) -> anyhow::Result<()> {
        let mut zip = zip::ZipWriter::new(writer);
        let options = SimpleFileOptions::default().compression_method(method).unix_permissions(0o755);

        let pref = Path::new(prefix);
        let mut buffer = Vec::new();
        'entries: for entry in iter {
            let path = entry.path();
            let name = path.strip_prefix(pref).unwrap();
            let path_as_string = name
                .to_str()
                .map(str::to_owned)
                .with_context(|| format!("{name:?} Is a Non UTF-8 Path"))?;
            
            if included.len() > 0 {
                let mut found_match = false;
                for include in included {
                    if let Ok(re) = Regex::new(&include) {
                        if re.is_match(&path_as_string) {
                            found_match = true;
                            break;
                        }
                    }
                }
                if !found_match {
                    continue 'entries;
                }
            }
            if excluded.len() > 0 {
                for exclude in excluded {
                    if let Ok(re) = Regex::new(&exclude) {
                        if re.is_match(&path_as_string) {
                            continue 'entries;
                        }
                    }
                }
            }

            if path.is_file() {
                zip.start_file(path_as_string, options)?;
                let mut f = File::open(path)?;

                f.read_to_end(&mut buffer)?;
                zip.write_all(&buffer)?;
                buffer.clear();
            } else if !name.as_os_str().is_empty() {
                zip.add_directory(path_as_string, options)?;
            }
        }
        zip.finish()?;
        Ok(())
    }


    /*****************************************************************************
     * Unzip.
     *****************************************************************************/
    
    /// Unzip package file bytes into an output directory.
    pub fn unzip_bytes(&self, output_dir_path: &str, bytes: &Bytes) {
        let _ = fs::create_dir_all(&self.tmp);
        let tmp_file_path = format!("{}/{}.pkg", &self.tmp, nanoid!());
        let _ = fs::write(&tmp_file_path, bytes);

        Self::unzip_file(&tmp_file_path, output_dir_path);
        let _ = fs::remove_file(&tmp_file_path);
    }

    /// Unzip package file bytes into a temp directory.
    /// Returns the path to the temp directory in which the bytes were extracted.
    /// Remember to delete the temporary directory once you are done with it.
    pub fn unzip_bytes_to_temp(&self, bytes: &Bytes) -> Option<String> {
        let outdir = format!("{}/{}", &self.tmp, nanoid!());
        let _ = fs::create_dir_all(&outdir);
        
        let tmp_file_path = format!("{}/{}.pkg", &self.tmp, nanoid!());
        let _ = fs::write(&tmp_file_path, bytes);

        Self::unzip_file(&tmp_file_path, &outdir);
        let _ = fs::remove_file(&tmp_file_path);

        Some(outdir)
    }

    /// Unzip a package file into an output directory.
    pub fn unzip_file(pkg_file_path: &str, output_dir_path: &str) {
        let _ = fs::create_dir_all(output_dir_path);
        if let Ok(file) = fs::File::open(pkg_file_path) {
            if let Ok(mut archive) = zip::ZipArchive::new(file) {
                for i in 0..archive.len() {
                    let mut file = archive.by_index(i).unwrap();
                    
                    let outname = match file.enclosed_name() {
                        Some(path) => path,
                        None => continue,
                    };
                    
                    let mut outpath = PathBuf::from(output_dir_path);
                    outpath.push(outname);
                    
                    if file.is_dir() {
                        let _ = fs::create_dir_all(&outpath);
                    } else {
                        if let Some(p) = outpath.parent() {
                            if !p.exists() {
                                let _ = fs::create_dir_all(p);
                            }
                        }
                        if let Ok(mut outfile) = fs::File::create(&outpath) {
                            let _ = io::copy(&mut file, &mut outfile);
                        }
                    }
                }
            }
        }
    }
}
impl Format for StofPackageFormat {
    fn identifiers(&self) -> Vec<String> {
        vec!["pkg".into()]
    }
    fn content_type(&self) -> String {
        "application/octet-stream+pkg".into()
    }
    fn binary_import(&self, graph: &mut Graph, format: &str, bytes: Bytes, node: Option<NodeRef>) -> Result<(), Error> {
        if let Some(path) = self.unzip_bytes_to_temp(&bytes) {
            let res = self.file_import(graph, format, &path, node);
            let _ = fs::remove_dir_all(&path);
            res
        } else {
            Err(Error::PKGImport(format!("error unzipping bytes into temp PKG for binary import")))
        }
    }
    fn parser_import(&self, _format: &str, path: &str, context: &mut ParseContext) -> Result<(), Error> {
        let mut package_path = path.to_string();
        let mut cleanup_dir = None;

        if path.ends_with(".pkg") {
            // This is a zip file that should be unpacked
            let tmp_dir = format!("{}/{}", &self.tmp, nanoid!());
            Self::unzip_file(path, &tmp_dir);
            package_path = format!("{tmp_dir}/pkg.stof");
            cleanup_dir = Some(tmp_dir);
        } else if path.ends_with(".stof") {
            // This is treated as a package file "pkg.stof"
        } else {
            // This is a package directory path that contains a "pkg.stof" file
            let mut buf = path.split('.').collect::<Vec<&str>>();
            if buf.len() > 1 { buf.pop(); }
            let cwd = buf.join(".");
            package_path = format!("{}/pkg.stof", &cwd);
        }
        let cleanup = move || {
            if let Some(cleanup_dir) = cleanup_dir {
                let _ = fs::remove_dir_all(&cleanup_dir);
            }
        };

        let mut pkg_graph = Graph::default();
        let res = pkg_graph.parse_stof_file("stof", &package_path, None, false);
        if res.is_err() {
            cleanup();
            return res;
        }

        if let Some(import_ref) = Field::field_from_path(&mut pkg_graph, "root.import", None) {
            let mut import_value = None;
            if let Some(import_field) = pkg_graph.get_stof_data::<Field>(&import_ref) {
                import_value = Some(import_field.value.val.read().clone());
            }
            if let Some(import_val) = import_value {
                context.push_relative_import_stack_file(&package_path);
                let res = perform_imports(&pkg_graph, import_val, context);
                context.pop_relative_import_stack();
                if res.is_err() {
                    cleanup();
                    return res;
                }
            }
        }

        cleanup();
        Ok(())
    }
    fn file_import(&self, graph: &mut Graph, _format: &str, path: &str, node: Option<NodeRef>) -> Result<(), Error> {
        let mut package_path = path.to_string();
        let mut cleanup_dir = None;

        if path.ends_with(".pkg") {
            // This is a zip file that should be unpacked
            let tmp_dir = format!("{}/{}", &self.tmp, nanoid!());
            Self::unzip_file(path, &tmp_dir);
            package_path = format!("{tmp_dir}/pkg.stof");
            cleanup_dir = Some(tmp_dir);
        } else if path.ends_with(".stof") {
            // This is treated as a package file "pkg.stof"
        } else {
            // This is a package directory path that contains a "pkg.stof" file
            let mut buf = path.split('.').collect::<Vec<&str>>();
            if buf.len() > 1 { buf.pop(); }
            let cwd = buf.join(".");
            package_path = format!("{}/pkg.stof", &cwd);
        }
        let cleanup = move || {
            if let Some(cleanup_dir) = cleanup_dir {
                let _ = fs::remove_dir_all(&cleanup_dir);
            }
        };

        let mut pkg_graph = Graph::default();
        let res = pkg_graph.parse_stof_file("stof", &package_path, None, false);
        if res.is_err() {
            cleanup();
            return res;
        }

        if let Some(import_ref) = Field::field_from_path(&mut pkg_graph, "root.import", None) {
            let mut import_value = None;
            if let Some(import_field) = pkg_graph.get_stof_data::<Field>(&import_ref) {
                import_value = Some(import_field.value.val.read().clone());
            }
            if let Some(import_val) = import_value {
                let mut context = ParseContext::new(graph);
                if let Some(node) = node {
                    context.push_self_node(node);
                }
                context.push_relative_import_stack_file(&package_path);
                let res = perform_imports(&pkg_graph, import_val, &mut context);
                context.pop_relative_import_stack();
                if res.is_err() {
                    cleanup();
                    return res;
                }
            }
        }

        cleanup();
        Ok(())
    }
}

fn perform_imports(pkg_graph: &Graph, import_val: Val, context: &mut ParseContext) -> Result<(), Error> {
    match import_val {
        Val::Str(path) => {
            let mut res_format = "stof".to_string();
            let path_list = path.trim_start_matches('.').split('.').collect::<Vec<_>>();
            if path_list.len() > 1 {
                res_format = path_list.last().unwrap().to_string();
            }
            context.parse_from_file(&res_format, &format!("./{}", path), None)?;
        },
        Val::Obj(nref) => {
            let mut path = String::default();
            if let Some(path_ref) = Field::direct_field(pkg_graph, &nref, "path") {
                if let Some(path_field) = pkg_graph.get_stof_data::<Field>(&path_ref) {
                    path = path_field.value.val.read().print(pkg_graph);
                }
            }

            let mut format = "stof".to_string();
            if let Some(format_ref) = Field::direct_field(pkg_graph, &nref, "format") {
                if let Some(format_field) = pkg_graph.get_stof_data::<Field>(&format_ref) {
                    format = format_field.value.val.read().print(pkg_graph);
                }
            } else if path.len() > 0 {
                let path_list = path.trim_start_matches('.').split('.').collect::<Vec<_>>();
                if path_list.len() > 1 {
                    format = path_list.last().unwrap().to_string();
                }
            }

            let mut scope = "self".to_string();
            if let Some(scope_ref) = Field::direct_field(pkg_graph, &nref, "as") {
                if let Some(scope_field) = pkg_graph.get_stof_data::<Field>(&scope_ref) {
                    scope = scope_field.value.val.read().print(pkg_graph);
                }
            }
            if let Some(scope_ref) = Field::direct_field(pkg_graph, &nref, "on") {
                if let Some(scope_field) = pkg_graph.get_stof_data::<Field>(&scope_ref) {
                    scope = scope_field.value.val.read().print(pkg_graph);
                }
            }
            if let Some(scope_ref) = Field::direct_field(pkg_graph, &nref, "scope") {
                if let Some(scope_field) = pkg_graph.get_stof_data::<Field>(&scope_ref) {
                    scope = scope_field.value.val.read().print(pkg_graph);
                }
            }

            if path.len() > 0 {
                let mut start = None;
                if scope.starts_with(SELF_KEYWORD.as_ref()) || scope.starts_with(SUPER_KEYWORD.as_ref()) {
                    start = Some(context.self_ptr());
                }
                let node = context.graph.ensure_named_nodes(&scope, start, true, None);
                context.parse_from_file(&format, &format!("./{}", path), node)?;
            }
        },
        Val::Tup(vals) |
        Val::List(vals) => {
            for val in vals {
                perform_imports(pkg_graph, val.read().clone(), context)?;
            }
        },
        Val::Set(vals) => {
            for val in vals {
                perform_imports(pkg_graph, val.read().clone(), context)?;
            }
        },
        _ => {
            return Err(Error::PKGImport(format!("invalid pkg.stof import field value")));
        }
    }
    Ok(())
}
