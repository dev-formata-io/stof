//
// Copyright 2025 Formata, Inc. All rights reserved.
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

use std::{collections::HashSet, fs::{self, File}, io::{self, Read, Seek, Write}, path::{Path, PathBuf}, sync::Arc};
use bytes::Bytes;
use rustc_hash::FxHashMap;
use nanoid::nanoid;
use regex::Regex;
use walkdir::{DirEntry, WalkDir};
use crate::model::FS_LIB;
use zip::{result::ZipResult, write::SimpleFileOptions};
use crate::{model::{Field, Format, Func, Graph, NodeRef, Profile, SELF_STR_KEYWORD, SUPER_STR_KEYWORD, stof_std::load::LOAD_ATTR}, parser::context::ParseContext, runtime::{Error, Val}};


#[derive(Debug, Clone, Default)]
/// .pkg format: a zip of a package's files, with its pkg.stof at the top.
pub struct StofPackageFormat;
impl StofPackageFormat {
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

        let Ok(file) = fs::File::create(&path) else { return None; }; // Ex. no permission to write there
        let walkdir = WalkDir::new(dir_path);
        let iter = walkdir.into_iter();
        let res = Self::zip_directory(&mut iter.filter_map(|e| e.ok()), dir_path, file, zip::CompressionMethod::Bzip2, included, excluded);
        if res.is_err() {
            return None;
        }
        return Some(path);
    }

    /// Zip the directory into an output file.
    fn zip_directory<T: Write + Seek>(iter: &mut dyn Iterator<Item = DirEntry>, prefix: &str, writer: T, method: zip::CompressionMethod, included: &HashSet<String>, excluded: &HashSet<String>) -> ZipResult<()> {
        let mut zip = zip::ZipWriter::new(writer);
        let options = SimpleFileOptions::default().compression_method(method).unix_permissions(0o755);

        let pref = Path::new(prefix);
        let mut buffer = Vec::new();
        'entries: for entry in iter {
            let path = entry.path();
            let Ok(name) = path.strip_prefix(pref) else { continue 'entries; };
            // a name that isn't UTF-8 can't be matched or stored as a package path: left out (was a panic)
            let Some(path_as_string) = name.to_str().map(str::to_owned) else { continue 'entries; };
            
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
     * Read.
     *****************************************************************************/

    /// A package's files from its bytes (a .pkg file), in memory: (name, bytes), names with '/' separators. Files
    /// that unzip to more than 256 MB in all are an error.
    pub fn files(bytes: &[u8]) -> Result<Vec<(String, Bytes)>, Error> {
        read_package(bytes, MAX_PACKAGE_BYTES)
    }

    /// One file of a package, from its bytes (only that file is unzipped), if it's there and isn't larger than
    /// 256 MB.
    pub fn file(bytes: &[u8], name: &str) -> Option<Bytes> {
        let mut archive = zip::ZipArchive::new(io::Cursor::new(bytes)).ok()?;
        let file = archive.by_name(name).ok()?;
        let mut contents = Vec::new();
        file.take(MAX_PACKAGE_BYTES + 1).read_to_end(&mut contents).ok()?;
        if contents.len() as u64 > MAX_PACKAGE_BYTES { return None; }
        Some(Bytes::from(contents))
    }

    /// The version a package's pkg.stof declares (its top-level `version: '1.4.0'`, quoted or not), from its bytes.
    pub fn declared_version(bytes: &[u8]) -> Option<String> {
        let pkg = Self::file(bytes, "pkg.stof")?;
        let src = std::str::from_utf8(&pkg).ok()?;
        for line in src.lines() {
            if line.starts_with([' ', '\t']) { continue; } // only the top level
            let Some(rest) = line.strip_prefix("version") else { continue };
            let Some(value) = rest.trim_start().strip_prefix(':') else { continue };
            let value = value.split("//").next().unwrap_or_default().trim().trim_end_matches([',', ';']).trim();
            let value = value.trim_matches(|c| c == '\'' || c == '"');
            if !value.is_empty() { return Some(value.to_string()); }
        }
        None
    }


    /*****************************************************************************
     * Unzip.
     *****************************************************************************/
    
    /// Unzip a package file into an output directory.
    pub fn unzip_file(pkg_file_path: &str, output_dir_path: &str) {
        let _ = fs::create_dir_all(output_dir_path);
        if let Ok(file) = fs::File::open(pkg_file_path) {
            if let Ok(mut archive) = zip::ZipArchive::new(file) {
                for i in 0..archive.len() {
                    // an entry that can't be read (Ex. a corrupt package) is skipped (was a panic)
                    let Ok(mut file) = archive.by_index(i) else { continue; };
                    
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
    /// A package's bytes (a .pkg file, Ex. fetched by Std.import): read in memory, so no file system is needed.
    fn binary_import(&self, graph: &mut Graph, _format: &str, bytes: Bytes, node: Option<NodeRef>, profile: &Profile) -> Result<(), Error> {
        let mut context = ParseContext::new(graph, profile.clone());
        if let Some(node) = node { context.push_self_node(node); }
        import_package(&bytes, &mut context)
    }
    fn parser_bytes_import(&self, _format: &str, bytes: Bytes, context: &mut ParseContext) -> Result<(), Error> {
        import_package(&bytes, context)
    }
    fn parser_import(&self, _format: &str, path: &str, context: &mut ParseContext) -> Result<(), Error> {
        if !context.graph.has_libfunc(&FS_LIB, "read") { return Err(Error::FormatFileImportNotAllowed); } // file system access is opt-in
        if path.ends_with(".pkg") {
            // a package file: its bytes, read in memory like any package
            let bytes = fs::read(path).map_err(|error| Error::FormatFileImportFsError(format!("{error}: {path}")))?;
            return import_package(&bytes, context);
        }
        let package_path = if path.ends_with(".stof") {
            path.to_string() // a package's "pkg.stof"
        } else if Path::new(path).is_dir() {
            Path::new(path).join("pkg.stof").to_string_lossy().to_string()
        } else {
            // a package directory, with a "pkg.stof" file
            let mut buf = path.split('.').collect::<Vec<&str>>();
            if buf.len() > 1 { buf.pop(); }
            format!("{}/pkg.stof", buf.join("."))
        };

        let destination = context.self_ptr();
        let mut pkg_graph = Graph::default();
        pkg_graph.quiet(context.graph.quiet);
        #[cfg(feature = "system")]
        pkg_graph.allow_system(); // the host allowed file access above
        pkg_graph.parse_stof_file("stof", &package_path, None, context.profile.clone())?;

        context.push_relative_import_stack_file(&package_path);
        let res = import_field(&mut pkg_graph, context);
        context.pop_relative_import_stack();
        res?;
        copy_loaders(&pkg_graph, context.graph, &destination);
        Ok(())
    }
    fn file_import(&self, graph: &mut Graph, format: &str, path: &str, node: Option<NodeRef>, profile: &Profile) -> Result<(), Error> {
        let mut context = ParseContext::new(graph, profile.clone());
        if let Some(node) = node { context.push_self_node(node); }
        self.parser_import(format, path, &mut context)
    }
}


/// The most a package's files may take once they're unzipped, without a memory limit (the host's, otherwise).
const MAX_PACKAGE_BYTES: u64 = 256 * 1024 * 1024;


/// Import a package from its bytes into the context's current object: its pkg.stof's `import` files (read from the
/// package, not the disk), then its pkg.stof's #[load] functions, so its dependencies load with it.
///
/// A package's imports stay in the package: a relative path is a file in it, and only a package ('@name') comes from
/// somewhere else (the package directory, with the file system).
pub(crate) fn import_package(bytes: &[u8], context: &mut ParseContext) -> Result<(), Error> {
    let destination = context.self_ptr();
    let limit = context.graph.max_memory.map(|max| max as u64).unwrap_or(MAX_PACKAGE_BYTES);
    let entries = read_package(bytes, limit)?;

    // the package's files, as paths in a directory of their own that isn't on any disk
    let dir = PathBuf::from("/").join(format!("__stof_pkg_{}", nanoid!(10, &nanoid::alphabet::SAFE[2..])));
    let mut files = FxHashMap::default();
    for (name, bytes) in entries {
        let mut path = dir.clone();
        for part in name.split('/') { if !part.is_empty() { path.push(part); } }
        files.insert(path, bytes);
    }
    let pkg_path = dir.join("pkg.stof");
    let Some(pkg_bytes) = files.get(&pkg_path).cloned() else {
        return Err(Error::PKGImport("a package (.pkg) needs a pkg.stof at its top".into()));
    };
    let Ok(pkg_src) = std::str::from_utf8(&pkg_bytes) else {
        return Err(Error::PKGImport("a package's pkg.stof isn't UTF-8 text".into()));
    };

    let mut pkg_graph = Graph::default();
    pkg_graph.quiet(context.graph.quiet);
    pkg_graph.string_import("stof", pkg_src, None, &context.profile)?;

    let pkg_file = pkg_path.to_string_lossy().to_string();
    context.package_files.push((dir, Arc::new(files)));
    context.push_relative_import_stack_file(&pkg_file);
    context.file_stack.push(pkg_file);
    let res = import_field(&mut pkg_graph, context);
    context.file_stack.pop();
    context.pop_relative_import_stack();
    context.package_files.pop();
    res?;
    copy_loaders(&pkg_graph, context.graph, &destination);
    Ok(())
}


/// A package's files (name, bytes), read from its zip bytes. Unreadable entries are skipped; files that unzip to more
/// than the limit are an error (Ex. a zip bomb).
fn read_package(bytes: &[u8], limit: u64) -> Result<Vec<(String, Bytes)>, Error> {
    let mut archive = zip::ZipArchive::new(io::Cursor::new(bytes)).map_err(|error| Error::PKGImport(format!("not a package (.pkg) file: {error}")))?;
    let mut entries = Vec::new();
    let mut total: u64 = 0;
    for i in 0..archive.len() {
        let Ok(mut file) = archive.by_index(i) else { continue; }; // a corrupt entry is skipped (was a panic)
        if file.is_dir() { continue; }
        let Some(name) = file.enclosed_name() else { continue; }; // a name that would leave the package
        let Some(name) = name.to_str().map(|name| name.replace('\\', "/")) else { continue; };
        let mut contents = Vec::new();
        let remaining = limit.saturating_sub(total);
        if (&mut file).take(remaining + 1).read_to_end(&mut contents).is_err() { continue; }
        total += contents.len() as u64;
        if total > limit {
            return Err(Error::PKGImport(format!("a package's files unzip to more than {limit} bytes")));
        }
        entries.push((name, Bytes::from(contents)));
    }
    Ok(entries)
}


/// The pkg.stof's `import` field: the files it imports (Ex. `import: 'main.stof'`, or a list, or objects with a path,
/// format, and where they go).
fn import_field(pkg_graph: &mut Graph, context: &mut ParseContext) -> Result<(), Error> {
    let Some(import_ref) = Field::field_from_path(pkg_graph, "root.import", None) else { return Ok(()) };
    let Some(import_val) = pkg_graph.get_stof_data::<Field>(&import_ref).map(|field| field.value.val.read().clone()) else { return Ok(()) };
    perform_imports(pkg_graph, import_val, context)
}


/// The pkg.stof's #[load] functions (its dependencies), added to the object the package went into, so they load with
/// it (Std.import runs them, and so does `load`). One whose name is taken there (Ex. a package with a `greet` function
/// and a `#[load] fn greet()` for its dependency) gets another name: a #[load] function's name doesn't matter, and
/// the package's own data stays.
fn copy_loaders(pkg_graph: &Graph, graph: &mut Graph, destination: &NodeRef) {
    let Some(root) = pkg_graph.main_root() else { return };
    let Some(node) = root.node(pkg_graph) else { return };
    let mut loaders: Vec<_> = node.data.values().cloned().collect();
    loaders.sort();
    for dref in loaders {
        let Some(data) = dref.data(pkg_graph) else { continue };
        let Some(func) = data.data.as_dyn_any().downcast_ref::<Func>() else { continue };
        if !func.attributes.contains_key(LOAD_ATTR.as_str()) { continue; }
        let mut copy = data.clone();
        copy.invalidate_nodes();
        copy.nodes = Default::default();
        let taken = |graph: &Graph, name: &str| destination.node(graph).is_some_and(|node| node.data.contains_key(name));
        let base = copy.name.as_ref().to_string();
        if taken(graph, &base) {
            let mut name = format!("{base}_load");
            let mut n = 2;
            while taken(graph, &name) { name = format!("{base}_load_{n}"); n += 1; }
            copy.name = name.into();
        }
        graph.insert_data(destination, copy);
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
                if scope.starts_with(SELF_STR_KEYWORD.as_str()) || scope.starts_with(SUPER_STR_KEYWORD.as_str()) {
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
            return Err(Error::PKGImport("invalid pkg.stof import field value".into()));
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use std::io::Write;
    use super::StofPackageFormat;

    #[test]
    /// A package's bytes import without a file system (in memory), into an object; its pkg.stof's #[load] functions
    /// come along, and its fields stay out. (The fixture is the JavaScript check's, zipped with bzip2 by another tool.)
    fn bytes_without_files() {
        use crate::{model::{Graph, Profile}, runtime::Val};
        let bytes = std::fs::read("tests/js/fixtures/tools.pkg").unwrap();
        let mut graph = Graph::default();
        graph.quiet(true);
        graph.parse_stof_src("tools: {}\nfn check() -> str `${self.tools.tool()} ${self.tools.name ?? 'no name'}`\nfn loaded() -> bool self.tools.tools_loaded", None, Profile::default()).unwrap();
        let tools = crate::model::SPath::node(&graph, "root.tools", None).unwrap();
        graph.binary_import("pkg", bytes.into(), Some(tools), &Profile::default()).unwrap();
        assert_eq!(graph.call("check", None, vec![]).unwrap(), Val::from("tool util no name"));
        assert_eq!(graph.load(None).unwrap(), Val::from(1));
        assert_eq!(graph.call("loaded", None, vec![]).unwrap(), Val::from(true));
    }

    #[test]
    /// A pkg.stof #[load] function whose name the package's own data has (Ex. a `greet` function) gets another name;
    /// so does one another package brought there.
    fn loader_names() {
        use crate::{model::{Graph, Profile}, runtime::Val};
        let zipped = |pkg: &str, main: &str| {
            let mut buf = std::io::Cursor::new(Vec::new());
            {
                let mut writer = zip::ZipWriter::new(&mut buf);
                writer.start_file("pkg.stof", zip::write::SimpleFileOptions::default()).unwrap();
                writer.write_all(pkg.as_bytes()).unwrap();
                writer.start_file("main.stof", zip::write::SimpleFileOptions::default()).unwrap();
                writer.write_all(main.as_bytes()).unwrap();
                writer.finish().unwrap();
            }
            buf.into_inner()
        };
        let a = zipped("import: 'main.stof'\n#[load] fn greet() { self.a_loaded = true; }\n", "fn greet() -> str 'hi'\n");
        let b = zipped("import: 'main.stof'\n#[load] fn greet() { self.b_loaded = true; }\n", "fn other() -> str 'other'\n");
        let mut graph = Graph::default();
        graph.quiet(true);
        graph.binary_import("pkg", a.into(), None, &Profile::default()).unwrap();
        graph.binary_import("pkg", b.into(), None, &Profile::default()).unwrap();
        graph.parse_stof_src("fn check() -> str `${self.greet()} ${self.other()} ${self.a_loaded} ${self.b_loaded}`", None, Profile::default()).unwrap();
        assert_eq!(graph.load(None).unwrap(), Val::from(2), "both packages' #[load] functions");
        assert_eq!(graph.call("check", None, vec![]).unwrap(), Val::from("hi other true true"));
    }

    #[test]
    /// A package whose entry can't be read (its local header is broken, while the directory still lists it) is
    /// skipped when unzipped, not a panic.
    fn corrupt_entry_is_not_a_panic() {
        let mut buf = std::io::Cursor::new(Vec::new());
        {
            let mut writer = zip::ZipWriter::new(&mut buf);
            writer.start_file("pkg.stof", zip::write::SimpleFileOptions::default()).unwrap();
            writer.write_all(b"value: 42").unwrap();
            writer.finish().unwrap();
        }
        let mut bytes = buf.into_inner();
        bytes[0] = b'X'; // the entry's local header signature (PK\x03\x04)

        let dir = std::env::temp_dir().join(format!("stof_corrupt_pkg_{}", nanoid::nanoid!()));
        let file = dir.with_extension("pkg");
        std::fs::write(&file, &bytes).unwrap();
        StofPackageFormat::unzip_file(file.to_str().unwrap(), dir.to_str().unwrap());
        assert!(!dir.join("pkg.stof").exists());
        let _ = std::fs::remove_file(&file);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
