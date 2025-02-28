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

use std::{collections::HashSet, fs::{self, File}, io::{self, Error, Read, Seek, Write}, path::{Path, PathBuf}};
use anyhow::Context;
use bytes::Bytes;
use nanoid::nanoid;
use regex::Regex;
use walkdir::{DirEntry, WalkDir};
use zip::write::SimpleFileOptions;
use crate::{lang::SError, Format, SDoc, SField, SVal};


/// Stof pkg format interface.
pub struct PKG {
    pub temp_dir: String,
}
impl Default for PKG {
    fn default() -> Self {
        Self {
            temp_dir: format!("{}/__stof_staging__", std::env::temp_dir().display()),
        }
    }
}
impl PKG {
    /// Remove file.
    pub fn remove_zip(path: &str) -> Result<(), Error> {
        fs::remove_file(path)
    }

    /// Create a temp zip file.
    pub fn create_temp_zip(&self, dir_path: &str, included: &HashSet<String>, excluded: &HashSet<String>) -> Option<String> {
        let _ = fs::create_dir_all(&self.temp_dir);
        let path = format!("{}/{}.pkg", self.temp_dir, nanoid!());
        PKG::create_package_zip(dir_path, &path, included, excluded)
    }

    /// Create package zip file.
    /// If successful, returns a path to the newly created zip file (dest_path).
    pub fn create_package_zip(dir_path: &str, dest_path: &str, included: &HashSet<String>, excluded: &HashSet<String>) -> Option<String> {
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
        let res = PKG::zip_directory(&mut iter.filter_map(|e| e.ok()), dir_path, file, zip::CompressionMethod::Bzip2, included, excluded);
        if res.is_err() {
            return None;
        }
        return Some(path);
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

            // Filter whether this file/dir should be included
            if path_as_string.contains("__stof__") {
                continue 'entries;
            }
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

    /// Unzip zip file bytes into a temp directory.
    /// Returns the path to the temp directory in which the bytes were extracted.
    /// Remember to delete the temporary directory once you are done with it.
    pub fn unzip_temp_bytes(&self, bytes: &Bytes) -> Option<String> {
        let outdir = format!("{}/{}", &self.temp_dir, nanoid!());
        let _ = fs::create_dir_all(&outdir);
        
        let tmp_file_path = format!("{}/{}.pkg", &self.temp_dir, nanoid!());
        let _ = fs::write(&tmp_file_path, bytes);

        PKG::unzip_file(&tmp_file_path, &outdir);
        let _ = fs::remove_file(&tmp_file_path);

        Some(outdir)
    }
    
    /// Unzip a zip file into an output directory.
    pub fn unzip_file(zip_file_path: &str, output_dir_path: &str) {
        let _ = fs::create_dir_all(output_dir_path);
        if let Ok(file) = fs::File::open(zip_file_path) {
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

    /// Unzip zip file bytes into the __stof__ pkg directory.
    pub fn unzip_pkg_bytes(&self, package_name_path: &str, bytes: &Bytes) -> String {
        let _ = fs::create_dir_all(&self.temp_dir);
        let tmp_file_path = format!("{}/{}.pkg", &self.temp_dir, nanoid!());
        let _ = fs::write(&tmp_file_path, bytes);
        let outdir = PKG::unzip_pkg(package_name_path, &tmp_file_path);
        let _ = fs::remove_file(&tmp_file_path);
        outdir
    }

    /// Unzip zip file into the __stof__ pkg directory.
    pub fn unzip_pkg(package_name_path: &str, zip_file_path: &str) -> String {
        let outdir = format!("__stof__/{}", package_name_path.trim_start_matches("@"));
        PKG::unzip_file(zip_file_path, &outdir);
        outdir
    }
}
impl Format for PKG {
    /// Format identifier.
    fn format(&self) -> String {
        "pkg".to_string()
    }

    /// Content type.
    fn content_type(&self) -> String {
        "application/pkg+octet-stream".to_string()
    }

    /// Bytes import.
    /// Import a package zip file in bytes, creating a temp package directory and importing it.
    fn header_import(&self, pid: &str, doc: &mut SDoc, _content_type: &str, bytes: &mut Bytes, as_name: &str) -> Result<(), SError> {
        if let Some(temp_dir_path) = self.unzip_temp_bytes(bytes) {
            let full_path = format!("{}.stof", temp_dir_path);
            let res = self.file_import(pid, doc, "pkg", &full_path, "stof", as_name);
            let _ = fs::remove_dir_all(&temp_dir_path);
            return res;
        }
        Err(SError::custom(pid, doc, "PkgImportBytesError", "error creating temporary package"))
    }

    /// Package import.
    /// Looks at the "pkg.stof" "import" or "imports" field for files to import into this document.
    /// Import a directory containing a "pkg.stof" file.
    /// Import a zip (.pkg) file containing a "pkg.stof" file.
    fn file_import(&self, pid: &str, doc: &mut SDoc, _format: &str, full_path: &str, extension: &str, as_name: &str) -> Result<(), SError> {
        let mut import_path = full_path.to_owned();
        let mut created_temp = false;
        if extension == "pkg" {
            // unzip this import path to a temporary directory so that it can be imported
            import_path = format!("{}/{}", &self.temp_dir, nanoid!());
            PKG::unzip_file(full_path, &import_path);
            created_temp = true;
            import_path.push_str(".stof"); // just because of the below logic (importing a dir instead of a file)
        }
        let import_path_clone = import_path.clone();
        let cleanup = move || {
            if created_temp {
                let _ = fs::remove_dir_all(&import_path_clone);
            }
        };

        let mut buf = import_path.split('.').collect::<Vec<&str>>();
        if buf.len() > 1 { buf.pop(); }
        let cwd = buf.join(".");
        let path = format!("{}/pkg.stof", &cwd);
        if let Ok(pkg) = SDoc::file(&path, "stof") {
            if let Some(field) = SField::field(&pkg.graph, "root.import", '.', None) {
                let mut pkg_format = "stof".to_string();
                match &field.value {
                    SVal::String(path) => {
                        let pkg_path = format!("{}/{}", &cwd, path);
                        if let Err(error) = doc.file_import(pid, &pkg_format, &pkg_path, &pkg_format, as_name) {
                            cleanup();
                            return Err(error);
                        }
                    },
                    SVal::Object(nref) => {
                        if let Some(format_field) = SField::field(&pkg.graph, "format", '.', Some(nref)) {
                            pkg_format = format_field.to_string();
                        }
                        if let Some(path_field) = SField::field(&pkg.graph, "path", '.', Some(nref)) {
                            let pkg_path = format!("{}/{}", &cwd, path_field.to_string());
                            if let Err(error) = doc.file_import(pid, &pkg_format, &pkg_path, &pkg_format, as_name) {
                                cleanup();
                                return Err(error);
                            }
                        }
                    },
                    _ => {}
                }
                cleanup();
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
                                    if let Err(error) = doc.file_import(pid, &pkg_format, &pkg_path, &pkg_format, as_name) {
                                        cleanup();
                                        return Err(error);
                                    }
                                },
                                SVal::Object(nref) => {
                                    if let Some(format_field) = SField::field(&pkg.graph, "format", '.', Some(nref)) {
                                        pkg_format = format_field.to_string();
                                    }
                                    if let Some(path_field) = SField::field(&pkg.graph, "path", '.', Some(nref)) {
                                        let pkg_path = format!("{}/{}", &cwd, path_field.to_string());
                                        if let Err(error) = doc.file_import(pid, &pkg_format, &pkg_path, &pkg_format, as_name) {
                                            cleanup();
                                            return Err(error);
                                        }
                                    }
                                },
                                _ => {}
                            }
                        }
                    },
                    _ => {}
                }
                cleanup();
                return Ok(());
            }
        }
        cleanup();
        Err(SError::custom(pid, doc, "PkgImportError", "package not found"))
    }
}
