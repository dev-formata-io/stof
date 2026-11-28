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

//! Std.import and Std.imported: pinned dependencies loaded into the graph at runtime.
//!
//! `import(url, sha256, into)` fetches the bytes with the document's own `Http.fetch` (so the host's network rules,
//! seals, and JavaScript's fetch all apply as they do to any fetch), checks them against the pinned hash, parses
//! them into `into`, and records the hash on that object. The record is a node attribute (`imports`: a map of
//! sha256 to { url, package, version }), so it's kept by BSTF, moves with the object, and goes when the object is
//! dropped. An import that the object already records is skipped without fetching.
//!
//! Packages ('@name' destinations, parsed into the root) have one version per document: the highest one required
//! (see `PackageState` and `decide`).

use std::{ops::Deref, sync::Arc};
use arcstr::{literal, ArcStr};
use bytes::Bytes;
use imbl::{vector, OrdMap, Vector};
use serde::{Deserialize, Serialize};
use crate::{model::{stof_std::{load::{load_attr_functions, LoadIns}, parse_source, STD_LIB}, DataRef, Field, Graph, LibFunc, NodeRef, Param, SPath, FS_LIB, SELF_STR_KEYWORD, SUPER_STR_KEYWORD}, runtime::{semver::Version, instruction::{Instruction, Instructions}, instructions::{call::FuncCall, trycatch::TryCatchIns, Base, AWAIT}, proc::ProcEnv, Error, Type, Val, ValRef, Variable}};


/// The node attribute that records an object's imports: a map of sha256 (lowercase hex) to { url, package, version }
/// (package and version only for packages, version only when one was given).
pub const IMPORTS_ATTR: ArcStr = literal!("imports");


/// A package destination ("@formata/intent" or "@formata/intent@1.4.0"): the name (without the leading "@") and the
/// version, if one is given.
pub fn split_package(into: &str) -> Result<(String, Option<Version>), String> {
    let name = into.trim().trim_start_matches('@');
    match name.rsplit_once('@') {
        Some((name, version)) => {
            let version = Version::parse(version).map_err(|error| format!("'{into}': {error}"))?;
            Ok((name.to_string(), Some(version)))
        },
        None => Ok((name.to_string(), None)),
    }
}


/// Std.import(url, sha256, into, format, profile, headers) -> Promise<bool>
pub fn std_import() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "import".into(),
        is_async: true,
        docs: r#"# async Std::import(url: str, sha256: str, into: str | obj, format: str = null, profile: str = "prod", headers: map = null) -> Promise<bool>
Load a dependency into this document at runtime: fetch `url`, check that its bytes have the pinned `sha256`, and parse them into `into` (a path or an object; a missing path is created). Returns true when it loaded them, or false when they're there already (nothing is fetched). Usually called from a `#[load]` function, which the host runs with `load()`: await it there, so a failed import fails `load()` (an import that isn't awaited fails on its own).

- **Pinned:** bytes with a different hash are an error, and nothing is parsed. The hash is of the bytes as fetched (`sha256(bytes)`). Pin a URL that never changes (Ex. a GitHub release's `/releases/download/v1.4.0/intent.pkg`, not `/releases/latest/...`).
- **Once per object:** the import is recorded on `into` (its `imports` attribute: hash to `{ url, package, version }`), so importing the same hash there again is skipped. The same hash into another object is a separate copy, and different hashes can go into one object. The record moves with the object, and goes when it's dropped.
- **Network:** it uses this document's `Http.fetch`, so it reaches only what the host allows (`allow_http`, `allow_http_hosts`, and a sealed part's `hosts`, checked again after redirects: GitHub release files redirect to `release-assets.githubusercontent.com`). `headers` go with the request (Ex. `{ 'Authorization': 'Bearer ' + env('GITHUB_TOKEN') }` for a private file). Without `Http` it's an error.
- **Packages:** an `into` like `'@formata/intent'` is a package, parsed into the root (where `import '@formata/intent'` would put it). With this document's file system (`fs`, allowed here), the checked bytes are also written to `stof/formata/intent.stof` (with the format's extension), where `import '@formata/intent'` reads them, and a later import uses that file instead of fetching (a changed file is fetched again). A host that only installs packages (the CLI) writes them without parsing.
- **Versions:** `'@formata/intent@1.4.0'` (a `ver`) is version 1.4.0 or a newer compatible one (the same major version; for 0.x, the same minor version; a pre-release only with its own version's pre-releases and later releases), and a `.pkg` must be the version its `pkg.stof` says. A document has one version of a package, the highest one its imports require: a lower one is skipped (false) when a higher compatible one is loaded or loading, and a higher one is parsed over a lower one. Two incompatible versions (Ex. 1.x and 2.x), or one version with different bytes, are errors. A package without a version uses whatever's there, and a version replaces it. A package your code imports from `stof/` (`import '@formata/intent'`) is the one used, whatever the version.
- **Cache:** a host can give documents a cache of fetched bytes by hash (Ex. the CLI); bytes from it are checked like fetched ones.
- **Any file:** data, code, text, images, documents. The format is `format`, else the URL's file extension when it's a format this document has (Ex. `.stof`, `.bstf`, `.pkg`, `.json`, `.toml`, `.png`), else the response's content type (a generic one, like `application/octet-stream` or `text/plain`, is ignored), else `bytes` when it isn't UTF-8, else `text` for another extension or content type (Ex. `.csv`), else `stof` (no extension). A format that only exports (Ex. `html`) is imported as text.
- **Its own `#[load]` functions** run once it's parsed and recorded (`load(into)`), so its dependencies load too (a `.pkg` brings the `#[load]` functions in its `pkg.stof`). A dependency two packages share loads once.
- **Errors** are thrown (the fetch, the response's status, the hash, a version conflict, the parse, or its `#[load]` functions), so true and false only mean loaded or already there. A failed parse records nothing, and parsing the same bytes again replaces what was parsed.
```rust
#[load] fn intent() { await import('https://github.com/formata/intent/releases/download/v1.4.0/intent.pkg', '9f2c...', '@formata/intent@1.4.0'); }
#[load] fn rules() { await import('https://example.com/rules.bstf', '4b1e...', 'root.deps.rules'); }
```
"#.into(),
        params: crate::params![
            Param { name: "url".into(), param_type: Type::Str, default: None },
            Param { name: "sha256".into(), param_type: Type::Str, default: None },
            Param { name: "into".into(), param_type: Type::Void, default: None },
            Param { name: "format".into(), param_type: Type::Void, default: Some(Arc::new(Base::Literal(Val::Null))) },
            Param { name: "profile".into(), param_type: Type::Str, default: Some(Arc::new(Base::Literal(Val::Str("prod".into())))) },
            Param { name: "headers".into(), param_type: Type::Void, default: Some(Arc::new(Base::Literal(Val::Null))) },
        ],
        return_type: Some(Type::Bool),
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(ImportIns::Start));
            Ok(instructions)
        })
    }
}


/// Std.imported(obj) -> list
pub fn std_imported() -> LibFunc {
    LibFunc {
        library: STD_LIB.clone(),
        name: "imported".into(),
        is_async: false,
        docs: r#"# Std::imported(obj: obj = null) -> list
What's been imported (with `import`, or a package with `import '@name'`) into this object and the objects in it, or into the whole document: a list of maps with `url`, `sha256`, and `into` (the object it went into), plus `package` and `version` for packages (`version` when it has one), read from the objects' records.
```rust
for (const dep in imported()) pln(dep.get('url'), dep.get('sha256'));
```
"#.into(),
        params: crate::params![
            Param { name: "obj".into(), param_type: Type::Void, default: Some(Arc::new(Base::Literal(Val::Null))) },
        ],
        return_type: Some(Type::List),
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(ImportIns::Imported));
            Ok(instructions)
        })
    }
}


/// Where `import` keeps and finds fetched bytes by their sha256, set by the host (`Graph::import_cache`). What comes
/// from it is checked against the pinned hash like fetched bytes, so a cache can't change what's imported.
pub trait ImportCache: std::fmt::Debug + Send + Sync {
    /// The bytes cached for this hash, if any.
    fn get(&self, sha256: &str) -> Option<Bytes>;
    /// Keep these bytes (already checked against this hash).
    fn put(&self, sha256: &str, bytes: &Bytes);
}


#[cfg(feature = "system")]
#[derive(Debug, Clone)]
/// An import cache in a directory: one file per hash (Ex. the CLI's user cache directory).
pub struct DirImportCache {
    pub dir: std::path::PathBuf,
}
#[cfg(feature = "system")]
impl DirImportCache {
    pub fn new(dir: impl Into<std::path::PathBuf>) -> Self {
        Self { dir: dir.into() }
    }
}
#[cfg(feature = "system")]
impl ImportCache for DirImportCache {
    fn get(&self, sha256: &str) -> Option<Bytes> {
        if normalize_hash(sha256).is_none() { return None; }
        std::fs::read(self.dir.join(sha256)).ok().map(Bytes::from)
    }
    fn put(&self, sha256: &str, bytes: &Bytes) {
        if normalize_hash(sha256).is_none() { return; }
        if std::fs::create_dir_all(&self.dir).is_err() { return; }
        // written whole, then renamed: a reader never sees part of a file
        let temp = self.dir.join(format!("{sha256}.{}.tmp", nanoid::nanoid!(8)));
        if std::fs::write(&temp, bytes).is_ok() && std::fs::rename(&temp, self.dir.join(sha256)).is_err() {
            let _ = std::fs::remove_file(&temp);
        }
    }
}


#[derive(Debug, Clone)]
/// The package a name resolves to in a document (`Graph::packages`): the one loading or loaded. An import of another
/// version replaces it only when it's higher (see `decide`), and one that was replaced while it fetched does nothing.
pub struct PackageState {
    pub version: Option<Version>,
    pub sha256: ArcStr,
    pub url: ArcStr,
    /// Parsed (or written, installing), not just started.
    pub loaded: bool,
    /// Read from the package directory by an `import '@name'` statement: the one used, whatever the version.
    pub file: bool,
    /// While it's loading: the runtime doing it (see `ProcEnv::alive`). A load whose runtime is gone without finishing
    /// it (Ex. a call that timed out) was abandoned, so it doesn't count.
    pub loader: Option<std::sync::Weak<()>>,
}
impl PackageState {
    /// Loaded, or still loading (not abandoned).
    fn standing(&self) -> bool {
        self.loaded || self.loader.as_ref().is_some_and(|alive| alive.strong_count() > 0)
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// What an import is doing, carried from step to step.
pub struct ImportJob {
    pub url: ArcStr,
    pub sha256: ArcStr,
    /// The object it parses into (None: only written to the package directory).
    pub into: Option<NodeRef>,
    /// A package ('@name'): its name (Ex. "formata/intent") and version, if given.
    pub name: Option<ArcStr>,
    pub version: Option<Version>,
    /// A package: its path in the package directory, without the extension (Ex. "stof/formata/intent").
    pub package: Option<ArcStr>,
    /// Can it write the package directory (this document's fs library, allowed here)?
    pub files: bool,
    pub format: Option<ArcStr>,
    pub profile: ArcStr,
    /// Request headers (a map), or null.
    pub headers: Val,
}
impl ImportJob {
    /// The package as written (Ex. "@formata/intent@1.4.0"), or the url, for messages.
    fn label(&self) -> String {
        match (&self.name, &self.version) {
            (Some(name), Some(version)) => format!("@{name}@{version}"),
            (Some(name), None) => format!("@{name}"),
            _ => self.url.to_string(),
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// Import instructions (their own type, so BSTF's variant indexes for the other libraries never shift).
pub enum ImportIns {
    /// (url, sha256, into, format, profile, headers) -> false when it's there already, otherwise Source.
    Start,
    /// (the package's file, or null) -> false when it's installed already, otherwise the bytes from the package's file,
    /// the cache, or a fetch, then Finish.
    Source { job: ImportJob },
    /// (error message, error value) -> null: no package file to use.
    NoFile,
    /// (error message, error value) -> the import error for a fetch that failed.
    FetchFailed { job: ImportJob },
    /// (response map) -> checked bytes, written to the package directory when needed, then Parse.
    /// (fetched: put in the cache; from_file: read from the package's file, so not written again)
    Finish { job: ImportJob, fetched: bool, from_file: bool },
    /// (error message, error value) -> the import error for a package file that couldn't be written.
    WriteFailed { job: ImportJob, path: ArcStr },
    /// -> parses, records, and runs the #[load] functions it added (true), or just true (only written).
    Parse { job: ImportJob, bytes: Bytes, format: ArcStr },
    /// (error message, error value) -> the import error for its #[load] functions failing.
    LoadFailed { url: ArcStr },
    /// (obj) -> list of { url, sha256, into, package, version }.
    Imported,
}
#[typetag::serde(name = "ImportIns")]
impl Instruction for ImportIns {
    fn exec(&self, env: &mut ProcEnv, graph: &mut Graph) -> Result<Option<Instructions>, Error> {
        match self {
            Self::Start => start(env, graph),
            Self::NoFile => {
                env.stack.pop(); // the error value
                env.stack.pop(); // the error message
                env.stack.push(Variable::val(Val::Null));
                Ok(None)
            },
            Self::Source { job } => abandon_on_error(graph, job, |graph| source(env, graph, job)),
            Self::FetchFailed { job } => {
                env.stack.pop(); // the error value (what a catch block gets)
                let message = env.stack.pop().map(|var| var.val.read().to_string()).unwrap_or_default();
                abandon(graph, job);
                Err(Error::Import(format!("import of '{}' failed: {}", job.url, unwrap_await(&message))))
            },
            Self::Finish { job, fetched, from_file } => abandon_on_error(graph, job, |graph| finish(env, graph, job, *fetched, *from_file)),
            Self::WriteFailed { job, path } => {
                env.stack.pop(); // the error value
                let message = env.stack.pop().map(|var| var.val.read().to_string()).unwrap_or_default();
                abandon(graph, job);
                Err(Error::Import(format!("import of '{}' couldn't write {path}: {}", job.url, unwrap_await(&message))))
            },
            Self::Parse { job, bytes, format } => abandon_on_error(graph, job, |graph| parse(env, graph, job, bytes, format)),
            Self::LoadFailed { url } => {
                env.stack.pop(); // the error value (what a catch block gets)
                let message = env.stack.pop().map(|var| var.val.read().to_string()).unwrap_or_default();
                Err(Error::Import(format!("import of '{url}' loaded, but its #[load] failed: {}", unwrap_await(&message))))
            },
            Self::Imported => {
                let scope = match env.stack.pop().map(|var| var.val.read().clone()) {
                    Some(Val::Obj(node)) => Some(node),
                    Some(Val::Null) | Some(Val::Void) | None => None,
                    Some(_) => return Err(Error::FuncArgsInfo("imported takes an object (or nothing, for the whole document)".into())),
                };
                let mut found = Vec::new();
                match &scope {
                    Some(node) => collect_imports(graph, node, &mut found),
                    None => {
                        let mut roots: Vec<NodeRef> = graph.roots.iter().cloned().collect();
                        roots.sort_by_key(|root| root.node_path(graph, true).map(|path| path.join(".")).unwrap_or_default());
                        for root in roots { collect_imports(graph, &root, &mut found); }
                    },
                }
                let list = found.into_iter().map(|(record, into)| {
                    let mut map = OrdMap::default();
                    let mut put = |key: &str, val: Val| { map.insert(ValRef::new(Val::Str(key.into())), ValRef::new(val)); };
                    put("url", Val::Str(record.url));
                    put("sha256", Val::Str(record.sha256));
                    put("into", Val::Obj(into));
                    if let Some(package) = record.package { put("package", Val::Str(package)); }
                    if let Some(version) = record.version { put("version", Val::Str(version.to_string().into())); }
                    ValRef::new(Val::Map(map))
                }).collect();
                env.stack.push(Variable::val(Val::List(list)));
                Ok(None)
            },
        }
    }
}


/// (url, sha256, into, format, profile, headers) -> false when it's there already, otherwise Source.
fn start(env: &mut ProcEnv, graph: &mut Graph) -> Result<Option<Instructions>, Error> {
    let headers = env.stack.pop().map(|var| var.val.read().clone()).unwrap_or(Val::Null);
    let profile = env.stack.pop().map(|var| var.val.read().clone()).unwrap_or(Val::Null);
    let format = env.stack.pop().map(|var| var.val.read().clone()).unwrap_or(Val::Null);
    let into = env.stack.pop().map(|var| var.val.read().clone()).unwrap_or(Val::Null);
    let sha256 = env.stack.pop().map(|var| var.val.read().clone()).unwrap_or(Val::Null);
    let url = env.stack.pop().map(|var| var.val.read().clone()).unwrap_or(Val::Null);

    let url: ArcStr = match url {
        Val::Str(url) if !url.trim().is_empty() => url.trim().into(),
        _ => return Err(Error::Import("import needs a url".into())),
    };
    let sha256 = match sha256 {
        Val::Str(hash) => normalize_hash(&hash).ok_or_else(|| Error::Import(format!("import of '{url}': sha256 must be 64 hexadecimal characters, the hash of the file (sha256(bytes))")))?,
        _ => return Err(Error::Import(format!("import of '{url}' needs the sha256 of the file it expects"))),
    };
    let format: Option<ArcStr> = match format {
        Val::Null | Val::Void => None,
        Val::Str(format) => Some(format.as_str().into()),
        _ => return Err(Error::Import(format!("import of '{url}': format must be a format name or content type"))),
    };
    let profile: ArcStr = match profile {
        Val::Null | Val::Void => "prod".into(),
        Val::Str(profile) => profile.as_str().into(),
        _ => return Err(Error::Import(format!("import of '{url}': profile must be a profile name (Ex. 'prod')"))),
    };
    let headers = match headers {
        Val::Null | Val::Void => Val::Null,
        Val::Map(map) => Val::Map(map),
        _ => return Err(Error::Import(format!("import of '{url}': headers must be a map (Ex. {{ 'Authorization': 'Bearer ...' }})"))),
    };

    let (name, version, package) = match &into {
        Val::Str(path) if path.trim().starts_with('@') => {
            let (name, version) = split_package(path).map_err(|error| Error::Import(format!("import of '{url}': {error}")))?;
            let package = package_path(&name, path.trim(), &url)?;
            (Some(ArcStr::from(name)), version, Some(package))
        },
        _ => (None, None, None),
    };
    let files = package.is_some() && can_write_files(graph);
    let target = if package.is_some() {
        if graph.import_vendor_only {
            if !files {
                return Err(Error::Import(format!("import of '{url}' installs a package, which needs this document's file system (fs), allowed here")));
            }
            None
        } else {
            Some(graph.ensure_main_root()) // where `import '@name'` puts it
        }
    } else {
        Some(resolve_into(graph, env, &into, &url)?)
    };

    // already imported here: nothing to fetch
    if let Some(target) = &target {
        if recorded(graph, target, &sha256) {
            if let (Some(name), Some(version)) = (&name, &version) { learn_version(graph, name, &sha256, version); }
            env.stack.push(Variable::val(Val::Bool(false)));
            return Ok(None);
        }
    }

    let job = ImportJob { url, sha256, into: target, name, version, package, files, format, profile, headers };
    if let Some(name) = &job.name {
        // one version of a package per document: the highest one required
        if !decide(graph, name, &job)? {
            env.stack.push(Variable::val(Val::Bool(false)));
            return Ok(None);
        }
        graph.packages.insert(name.clone(), PackageState { version: job.version.clone(), sha256: job.sha256.clone(), url: job.url.clone(), loaded: false, file: false, loader: Some(Arc::downgrade(&env.alive)) });
    }

    let mut instructions = Instructions::default();
    if let (true, Some(package)) = (job.files, &job.package) {
        // the package's file, if it's there already (read with this document's fs, like any read)
        let ext = package_extension(&job.url, job.format.as_deref(), None);
        instructions.push(Arc::new(TryCatchIns {
            try_ins: vector![lib_call("fs::read", vector![Val::Str(package_file(package, &ext).into())])],
            err_ins: vector![Arc::new(ImportIns::NoFile) as Arc<dyn Instruction>],
            catch_ins: vector![],
        }));
    } else {
        instructions.push(Arc::new(Base::Literal(Val::Null)));
    }
    instructions.push(Arc::new(ImportIns::Source { job }));
    Ok(Some(instructions))
}


/// (the package's file, or null) -> false when it's installed already, otherwise the bytes from the package's file, the
/// cache, or a fetch, then Finish.
fn source(env: &mut ProcEnv, graph: &mut Graph, job: &ImportJob) -> Result<Option<Instructions>, Error> {
    let file = env.stack.pop().map(|var| var.val.read().clone()).unwrap_or(Val::Null);
    if let Val::Blob(bytes) = file {
        if sha256_hex(&bytes).as_str() == job.sha256.as_str() {
            if job.into.is_none() {
                loaded(graph, job);
                env.stack.push(Variable::val(Val::Bool(false))); // installed already
                return Ok(None);
            }
            let mut instructions = Instructions::default();
            instructions.push(Arc::new(Base::Literal(response(bytes))));
            instructions.push(Arc::new(ImportIns::Finish { job: job.clone(), fetched: false, from_file: true }));
            return Ok(Some(instructions));
        }
    }
    if let Some(cache) = graph.import_cache.clone() {
        if let Some(bytes) = cache.get(&job.sha256) {
            if sha256_hex(&bytes).as_str() == job.sha256.as_str() {
                let mut instructions = Instructions::default();
                instructions.push(Arc::new(Base::Literal(response(bytes))));
                instructions.push(Arc::new(ImportIns::Finish { job: job.clone(), fetched: false, from_file: false }));
                return Ok(Some(instructions));
            }
        }
    }
    if !graph.libfuncs.get("Http").is_some_and(|lib| lib.contains_key("fetch")) {
        return Err(Error::Import(format!("import of '{}' needs network access, which this document doesn't have (the host can allow it: allow_http)", job.url)));
    }

    // the document's own Http.fetch: the host's network rules, seals, and JavaScript's fetch all apply
    // (a fetch that fails, Ex. a host it may not reach, becomes an import error that names the url)
    let mut instructions = Instructions::default();
    instructions.push(Arc::new(TryCatchIns {
        try_ins: vector![fetch_call(job), AWAIT.clone()],
        err_ins: vector![Arc::new(ImportIns::FetchFailed { job: job.clone() }) as Arc<dyn Instruction>],
        catch_ins: vector![],
    }));
    instructions.push(Arc::new(ImportIns::Finish { job: job.clone(), fetched: true, from_file: false }));
    Ok(Some(instructions))
}


/// (response map) -> checked bytes, written to the package directory when needed, then Parse.
fn finish(env: &mut ProcEnv, graph: &mut Graph, job: &ImportJob, fetched: bool, from_file: bool) -> Result<Option<Instructions>, Error> {
    let url = &job.url;
    let response = env.stack.pop().map(|var| var.val.read().clone()).unwrap_or(Val::Null);
    let Val::Map(response) = response else {
        return Err(Error::Import(format!("import of '{url}' failed: the fetch returned no response")));
    };
    let get = |key: &str| response.get(&ValRef::new(Val::Str(key.into()))).map(|val| val.read().clone());
    if let Some(error) = get("error") {
        return Err(Error::Import(format!("import of '{url}' failed: {}", error.to_string())));
    }
    if !get("ok").is_some_and(|ok| ok.truthy()) {
        let status = get("status").map(|status| status.to_string()).unwrap_or_else(|| "no status".into());
        return Err(Error::Import(format!("import of '{url}' failed: the server answered {status}")));
    }
    let Some(Val::Blob(bytes)) = get("bytes") else {
        return Err(Error::Import(format!("import of '{url}' failed: the response had no body")));
    };
    if fetched {
        // (the package's file and the cache were checked already)
        let actual = sha256_hex(&bytes);
        if actual.as_str() != job.sha256.as_str() {
            return Err(Error::Import(format!("import of '{url}' failed: its sha256 is {actual}, not the pinned {} (the file changed, or the pin is wrong)", job.sha256)));
        }
        if let Some(cache) = graph.import_cache.clone() { cache.put(&job.sha256, &bytes); }
    }
    if superseded(graph, job) {
        env.stack.push(Variable::val(Val::Bool(false))); // a higher version took its place while it fetched
        return Ok(None);
    }

    let content_type = match get("content_type") { Some(Val::Str(ctype)) => Some(ctype.to_string()), _ => None };
    let format: ArcStr = match &job.format {
        Some(format) => format.clone(),
        None => detect_format(graph, url, content_type.as_deref(), &bytes).into(),
    };
    check_declared_version(job, &format, &bytes)?;
    let mut instructions = Instructions::default();
    if let (true, Some(package)) = (job.files && !from_file, &job.package) {
        // the package's file (with this document's fs, like any write)
        let path: ArcStr = package_file(package, &package_extension(url, job.format.as_deref(), Some(&format))).into();
        let dir = path.rsplit_once('/').map(|(dir, _)| dir.to_string()).unwrap_or_default();
        instructions.push(Arc::new(TryCatchIns {
            try_ins: vector![
                lib_call("fs::create_dir", vector![Val::Str(dir.into())]),
                lib_call("fs::write", vector![Val::Str(path.clone()), Val::Blob(bytes.clone())]),
            ],
            err_ins: vector![Arc::new(ImportIns::WriteFailed { job: job.clone(), path }) as Arc<dyn Instruction>],
            catch_ins: vector![],
        }));
    }
    instructions.push(Arc::new(ImportIns::Parse { job: job.clone(), bytes: bytes.clone(), format }));
    Ok(Some(instructions))
}


/// -> parses, records, and runs the #[load] functions it added (true), or just true (only written).
fn parse(env: &mut ProcEnv, graph: &mut Graph, job: &ImportJob, bytes: &Bytes, format: &ArcStr) -> Result<Option<Instructions>, Error> {
    let url = &job.url;
    if superseded(graph, job) {
        env.stack.push(Variable::val(Val::Bool(false))); // a higher version took its place
        return Ok(None);
    }
    let Some(into) = &job.into else {
        loaded(graph, job);
        env.stack.push(Variable::val(Val::Bool(true))); // written to the package directory
        return Ok(None);
    };
    if !into.node_exists(graph) {
        return Err(Error::Import(format!("import of '{url}' failed: the object it was going into was dropped while it was fetched")));
    }
    let before = load_attr_functions(graph, &Some(into.clone()));
    let mut res = parse_source(graph, env, format, &job.profile, &Val::Blob(bytes.clone()), into.clone());
    if job.format.is_none() && matches!(res, Err(Error::FormatStringImportNotImplemented(_))) {
        // a format that only exports (Ex. html): its file is text, or bytes, as an `import` statement takes it
        let fallback = if std::str::from_utf8(bytes).is_ok() { "text" } else { "bytes" };
        res = parse_source(graph, env, fallback, &job.profile, &Val::Blob(bytes.clone()), into.clone());
    }
    if let Err(error) = res {
        return Err(match error {
            Error::Sealed(_) => error, // a seal's refusal stays a seal error
            error => Error::Import(format!("import of '{url}' failed to parse as {format}: {}", error.message())),
        });
    }
    let record = Record {
        url: url.clone(),
        sha256: job.sha256.clone(),
        package: job.name.as_ref().map(|name| format!("@{name}").into()),
        version: job.version.clone(),
        file: false,
    };
    record_import(graph, into, &record)?; // before its #[load] functions run, so they see it's here
    loaded(graph, job);

    // the #[load] functions it brought (dependencies of dependencies)
    let mut added: Vec<DataRef> = load_attr_functions(graph, &Some(into.clone())).into_iter().filter(|func| !before.contains(func)).collect();
    added.sort();
    let base = env.stack.len();
    let mut instructions = Instructions::default();
    instructions.push(Arc::new(TryCatchIns {
        try_ins: vector![
            Arc::new(LoadIns::Run { funcs: added.into_iter().collect() }) as Arc<dyn Instruction>,
            Arc::new(Base::PopUntilStackCount(base)) as Arc<dyn Instruction>,
        ],
        err_ins: vector![Arc::new(ImportIns::LoadFailed { url: url.clone() }) as Arc<dyn Instruction>],
        catch_ins: vector![],
    }));
    instructions.push(Arc::new(Base::Literal(Val::Bool(true))));
    Ok(Some(instructions))
}


/*****************************************************************************
 * Package versions.
 *****************************************************************************/

/// The package a name resolves to now: the one loading or loaded in this document, else the one its root records
/// (Ex. a document saved after it loaded), the highest version when it records more than one.
fn package_state(graph: &Graph, name: &str) -> Option<PackageState> {
    if let Some(state) = graph.packages.get(name).filter(|state| state.standing()) { return Some(state.clone()); }
    let root = graph.main_root()?;
    let package = format!("@{name}");
    let mut best: Option<PackageState> = None;
    for record in records(graph, &root) {
        if record.package.as_deref() != Some(package.as_str()) { continue; }
        let state = PackageState { version: record.version.clone(), sha256: record.sha256.clone(), url: record.url.clone(), loaded: true, file: record.file, loader: None };
        if state.file { return Some(state); } // what the code imported from stof/ is the one used
        let higher = match &best {
            None => true,
            Some(current) => match (&current.version, &state.version) {
                (None, Some(_)) => true,
                (Some(current), Some(version)) => version > current,
                _ => false,
            },
        };
        if higher { best = Some(state); }
    }
    best
}


/// Should this package import go ahead (true), or is it there already (false)? Errors for versions that can't share
/// a document: incompatible ones, or one version with different bytes.
/// - Nothing there yet: go ahead.
/// - A package your code imported from stof/ (`import '@name'`), or these same bytes: there already.
/// - Without a version: whatever's there is fine.
/// - There without a version: a version replaces it.
/// - A higher compatible version: there already; a lower one: this one replaces it (parsed over it).
fn decide(graph: &mut Graph, name: &str, job: &ImportJob) -> Result<bool, Error> {
    let Some(current) = package_state(graph, name) else { return Ok(true) };
    let label = job.label();
    if current.file { return Ok(false); }
    if current.sha256 == job.sha256 {
        if let Some(version) = &job.version { learn_version(graph, name, &job.sha256, version); }
        return Ok(false);
    }
    let Some(wanted) = &job.version else { return Ok(false) };
    let Some(have) = &current.version else { return Ok(true) };
    let state = if current.loaded { "loaded" } else { "loading" };
    if !wanted.compatible(have) {
        return Err(Error::Import(format!("import of {label} ('{}') conflicts with @{name}@{have} ({state}, from '{}'): {} and {} aren't compatible, and a document has one version of a package (the importers need to agree on one major version)", job.url, current.url, wanted.series(), have.series())));
    }
    if wanted == have {
        return Err(Error::Import(format!("import of {label} ('{}', sha256 {}) has different bytes than the same version {state} from '{}' (sha256 {}): a release changed, or a pin is wrong", job.url, job.sha256, current.url, current.sha256)));
    }
    Ok(wanted > have)
}


/// Was this import replaced while it fetched (a higher version decided on after it started)?
fn superseded(graph: &Graph, job: &ImportJob) -> bool {
    let Some(name) = &job.name else { return false };
    graph.packages.get(name).is_some_and(|state| state.standing() && state.sha256 != job.sha256)
}


/// A package's version must be the one its pkg.stof declares (a .pkg that declares one): pinning '@x@1.4.0' to a
/// file that says it's 1.0.0 is an error, not a 1.4.0.
#[allow(unused)]
fn check_declared_version(job: &ImportJob, format: &str, bytes: &[u8]) -> Result<(), Error> {
    #[cfg(feature = "pkg")]
    if let (Some(version), "pkg") = (&job.version, format) {
        if let Some(declared) = crate::model::StofPackageFormat::declared_version(bytes) {
            if Version::parse(&declared).ok().as_ref() != Some(version) {
                return Err(Error::Import(format!("import of {} ('{}'): its pkg.stof says it's version {declared}, not {version}", job.label(), job.url)));
            }
        }
    }
    Ok(())
}


/// This package import is done (parsed, or written when installing).
fn loaded(graph: &mut Graph, job: &ImportJob) {
    let Some(name) = &job.name else { return };
    if let Some(state) = graph.packages.get_mut(name) {
        if state.sha256 == job.sha256 { state.loaded = true; }
    }
}


/// This package import failed: it isn't loading anymore (what the document's root records stands again).
fn abandon(graph: &mut Graph, job: &ImportJob) {
    let Some(name) = &job.name else { return };
    if graph.packages.get(name).is_some_and(|state| state.sha256 == job.sha256 && !state.loaded) {
        graph.packages.remove(name);
    }
}

fn abandon_on_error(graph: &mut Graph, job: &ImportJob, step: impl FnOnce(&mut Graph) -> Result<Option<Instructions>, Error>) -> Result<Option<Instructions>, Error> {
    let res = step(graph);
    if res.is_err() { abandon(graph, job); }
    res
}


/// These bytes of a package turned out to be this version (imported without one first): remember it, so a later
/// version compares with it.
fn learn_version(graph: &mut Graph, name: &str, sha256: &ArcStr, version: &Version) {
    if let Some(state) = graph.packages.get_mut(name) {
        if &state.sha256 == sha256 && state.version.is_none() && !state.file { state.version = Some(version.clone()); }
    }
    let Some(root) = graph.main_root() else { return };
    if let Some(mut record) = records(graph, &root).into_iter().find(|record| &record.sha256 == sha256 && record.package.as_deref() == Some(format!("@{name}").as_str())) {
        if record.version.is_none() && !record.file {
            record.version = Some(version.clone());
            let _ = record_import(graph, &root, &record);
        }
    }
}


/// Record a package read from the package directory by an `import '@name'` statement (the parser does, so a
/// `#[load]` import of the same package uses it instead of parsing it again).
pub fn record_package_file(graph: &mut Graph, name: &str, path: &str, bytes: &[u8]) {
    let sha256 = sha256_hex(bytes);
    let url: ArcStr = path.replace('\\', "/").into();
    graph.packages.insert(name.into(), PackageState { version: None, sha256: sha256.clone(), url: url.clone(), loaded: true, file: true, loader: None });
    let root = graph.ensure_main_root();
    let _ = record_import(graph, &root, &Record { url, sha256, package: Some(format!("@{name}").into()), version: None, file: true });
}


/*****************************************************************************
 * Records.
 *****************************************************************************/

#[derive(Debug, Clone)]
/// One import an object records.
struct Record {
    url: ArcStr,
    sha256: ArcStr,
    /// A package: "@name".
    package: Option<ArcStr>,
    version: Option<Version>,
    /// Read from the package directory (`import '@name'`).
    file: bool,
}


/// The imports an object records.
fn records(graph: &Graph, node: &NodeRef) -> Vec<Record> {
    let Some(node) = node.node(graph) else { return vec![] };
    let Some(Val::Map(imports)) = node.attributes.get(IMPORTS_ATTR.as_str()) else { return vec![] };
    let mut found = Vec::new();
    for (sha256, value) in imports {
        let Val::Str(sha256) = sha256.read().deref().clone() else { continue };
        match value.read().deref() {
            Val::Str(url) => found.push(Record { url: url.clone(), sha256, package: None, version: None, file: false }),
            Val::Map(map) => {
                let get = |key: &str| map.get(&ValRef::new(Val::Str(key.into()))).map(|val| val.read().clone());
                let Some(Val::Str(url)) = get("url") else { continue };
                let package = match get("package") { Some(Val::Str(package)) => Some(package), _ => None };
                let version = match get("version") { Some(Val::Str(version)) => Version::parse(&version).ok(), _ => None };
                let file = get("file").is_some_and(|file| file.truthy());
                found.push(Record { url, sha256, package, version, file });
            },
            _ => {},
        }
    }
    found
}


/// Does this object record an import of this hash?
fn recorded(graph: &Graph, node: &NodeRef, sha256: &str) -> bool {
    let Some(node) = node.node(graph) else { return false };
    match node.attributes.get(IMPORTS_ATTR.as_str()) {
        Some(Val::Map(imports)) => imports.contains_key(&ValRef::new(Val::Str(sha256.into()))),
        _ => false,
    }
}


/// Record an import on the object it went into (adding to what it already records). A package replaces the record of
/// the version it was parsed over, so an object records one version of each package.
fn record_import(graph: &mut Graph, node: &NodeRef, record: &Record) -> Result<(), Error> {
    let mut imports = match node.node(graph).and_then(|node| node.attributes.get(IMPORTS_ATTR.as_str())) {
        Some(Val::Map(imports)) => imports.clone(),
        _ => OrdMap::default(),
    };
    if let Some(package) = &record.package {
        for other in records(graph, node) {
            if other.package.as_ref() == Some(package) && other.sha256 != record.sha256 {
                imports.remove(&ValRef::new(Val::Str(other.sha256.clone())));
            }
        }
    }
    let mut value = OrdMap::default();
    value.insert(ValRef::new(Val::Str("url".into())), ValRef::new(Val::Str(record.url.clone())));
    if let Some(package) = &record.package { value.insert(ValRef::new(Val::Str("package".into())), ValRef::new(Val::Str(package.clone()))); }
    if let Some(version) = &record.version { value.insert(ValRef::new(Val::Str("version".into())), ValRef::new(Val::Str(version.to_string().into()))); }
    if record.file { value.insert(ValRef::new(Val::Str("file".into())), ValRef::new(Val::Bool(true))); }
    imports.insert(ValRef::new(Val::Str(record.sha256.clone())), ValRef::new(Val::Map(value)));
    graph.set_attribute(Ok(node.clone()), IMPORTS_ATTR.as_str(), Some(Val::Map(imports)))?;
    Ok(())
}


/// Every import recorded on this object and the objects in it, with the object, in path order.
fn collect_imports(graph: &Graph, node: &NodeRef, found: &mut Vec<(Record, NodeRef)>) {
    let Some(inner) = node.node(graph) else { return };
    for record in records(graph, node) {
        found.push((record, node.clone()));
    }
    let mut children: Vec<NodeRef> = inner.children.iter().cloned().collect();
    children.sort_by_key(|child| child.node(graph).map(|child| child.name.as_ref().to_string()).unwrap_or_default());
    for child in children {
        collect_imports(graph, &child, found);
    }
}


/*****************************************************************************
 * Helpers.
 *****************************************************************************/

/// A call to one of this document's library functions (Ex. "fs::write"), so the host's and seals' rules apply.
fn lib_call(search: &'static str, args: Vector<Val>) -> Arc<dyn Instruction> {
    Arc::new(FuncCall {
        as_ref: false,
        cnull: false,
        stack: false,
        func: None,
        search: Some(ArcStr::from(search)),
        args: args.into_iter().map(|arg| Arc::new(Base::Literal(arg)) as Arc<dyn Instruction>).collect(),
        oself: None,
    })
}


/// Http.fetch(url), with the import's headers when it has them.
fn fetch_call(job: &ImportJob) -> Arc<dyn Instruction> {
    match &job.headers {
        Val::Map(_) => lib_call("Http::fetch", vector![Val::Str(job.url.clone()), Val::Str("get".into()), Val::Null, job.headers.clone()]),
        _ => lib_call("Http::fetch", vector![Val::Str(job.url.clone())]),
    }
}


/// A response map for bytes already here (a package's file, or the cache), as Http.fetch returns.
fn response(bytes: Bytes) -> Val {
    let mut map = OrdMap::default();
    map.insert(ValRef::new(Val::Str("ok".into())), ValRef::new(Val::Bool(true)));
    map.insert(ValRef::new(Val::Str("status".into())), ValRef::new(Val::from(200)));
    map.insert(ValRef::new(Val::Str("bytes".into())), ValRef::new(Val::Blob(bytes)));
    Val::Map(map)
}


/// An awaited error's message without the "awaited process failed: " prefixes it collects on the way.
fn unwrap_await(message: &str) -> &str {
    let mut message = message;
    while let Some(inner) = message.strip_prefix("awaited process failed: ") { message = inner; }
    message
}


/// The sha256 of bytes, in lowercase hexadecimal (as `import` pins it).
pub fn sha256_hex(bytes: &[u8]) -> ArcStr {
    use sha2::{Digest, Sha256};
    crate::model::seal::hex(&Sha256::digest(bytes)).into()
}


/// Can this import write the package directory? Only with this document's fs library (the host allowed the file
/// system), and, in a sealed part, only when its libs include fs.
fn can_write_files(graph: &Graph) -> bool {
    let has = |name: &str| graph.libfuncs.get(FS_LIB.as_str()).is_some_and(|lib| lib.contains_key(name));
    has("read") && has("write") && has("create_dir") && graph.confined.iter().all(|reach| reach.lib_allowed(FS_LIB.as_str()))
}


/// A package's path in the package directory, without the extension: 'formata/intent' is "stof/formata/intent" (as
/// `import '@formata/intent'` reads it). Each part is a plain name: letters, digits, '_', '-', and '.' (not "." or "..").
fn package_path(name: &str, into: &str, url: &str) -> Result<ArcStr, Error> {
    let parts: Vec<&str> = name.split('/').collect();
    let plain = |part: &&str| !part.is_empty() && *part != "." && *part != ".." && part.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'));
    if name.is_empty() || !parts.iter().all(plain) {
        return Err(Error::Import(format!("import of '{url}': '{into}' isn't a package name (Ex. '@formata/intent', or '@formata/intent@1.4.0' with a version: names of letters, digits, '_', '-', and '.', separated by '/')")));
    }
    Ok(format!("stof/{name}").into())
}


/// A package's file: its path with the format's extension (unless its name already ends with it).
fn package_file(package: &str, format: &str) -> String {
    if package.rsplit('/').next().is_some_and(|name| name.ends_with(&format!(".{format}"))) {
        package.to_string()
    } else {
        format!("{package}.{format}")
    }
}


/// A sha256 as 64 lowercase hexadecimal characters (surrounding spaces and uppercase are fine).
fn normalize_hash(hash: &str) -> Option<ArcStr> {
    let hash = hash.trim();
    if hash.len() == 64 && hash.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        Some(hash.to_ascii_lowercase().into())
    } else {
        None
    }
}


/// The object an import goes into: an object, or a path (a field holding an object, or a named object), created when
/// it's missing. A path starting with self or super starts at the calling object.
fn resolve_into(graph: &mut Graph, env: &ProcEnv, into: &Val, url: &str) -> Result<NodeRef, Error> {
    match into {
        Val::Obj(node) => {
            if node.node_exists(graph) { Ok(node.clone()) }
            else { Err(Error::Import(format!("import of '{url}': the object to import into doesn't exist"))) }
        },
        Val::Str(path) => {
            let path = path.trim();
            if path.is_empty() {
                return Err(Error::Import(format!("import of '{url}' needs somewhere to go (into: a path or an object)")));
            }
            let start = if path.starts_with(SELF_STR_KEYWORD.as_str()) || path.starts_with(SUPER_STR_KEYWORD.as_str()) { Some(env.self_ptr()) } else { None };
            if let Some(field_ref) = Field::field_from_path(graph, path, start.clone()) {
                if let Some(field) = graph.get_stof_data::<Field>(&field_ref) {
                    return match field.value.try_obj() {
                        Some(obj) => Ok(obj),
                        None => Err(Error::Import(format!("import of '{url}': '{path}' is a field that doesn't hold an object"))),
                    };
                }
            }
            if let Some(node) = SPath::node(graph, path, start.clone()) {
                return Ok(node);
            }
            match graph.ensure_named_nodes(path, start, true, None) {
                Some(node) => Ok(node),
                None => Err(Error::Import(format!("import of '{url}': couldn't create '{path}' to import into{}", graph.take_seal_violation().map(|why| format!(" (sealed: {why})")).unwrap_or_default()))),
            }
        },
        _ => Err(Error::Import(format!("import of '{url}' needs somewhere to go (into: a path or an object)"))),
    }
}


/// The format of fetched bytes, so anything imports: the URL's file extension when it names a format this document
/// has; else the response's content type when it names one (a generic one, like `application/octet-stream` or
/// `text/plain`, doesn't); else bytes that aren't UTF-8 are `bytes`; else text with another extension (Ex. `.csv`)
/// or content type (Ex. `text/csv`) is `text`; else (no extension, no content type that says) `stof`, as an
/// `import` statement takes a file without an extension.
fn detect_format(graph: &Graph, url: &str, content_type: Option<&str>, bytes: &[u8]) -> String {
    let ext = url_extension(url);
    if let Some(ext) = &ext {
        if graph.get_format(ext).is_some() { return ext.clone(); }
    }
    let mut other_type = false;
    if let Some(content_type) = content_type {
        let media = content_type.split(';').next().unwrap_or_default().trim().to_ascii_lowercase();
        let generic = matches!(media.as_str(), "" | "application/octet-stream" | "binary/octet-stream" | "text/plain");
        if !generic {
            if let Some(format) = graph.get_format_by_content_type(&media) {
                if let Some(id) = format.identifiers().first() {
                    return id.clone();
                }
            }
            other_type = true;
        }
    }
    if std::str::from_utf8(bytes).is_err() { return "bytes".into(); }
    if ext.is_some() || other_type { return "text".into(); }
    "stof".into()
}


/// A URL's file extension (lowercase), if its file has one.
fn url_extension(url: &str) -> Option<String> {
    let path = url.split(['?', '#']).next().unwrap_or_default();
    let file = path.rsplit('/').next().unwrap_or_default();
    match file.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() && !ext.is_empty() => Some(ext.to_ascii_lowercase()),
        _ => None,
    }
}


/// A package's file extension in the package directory: the format given, else the URL's own extension (a known
/// format's, or another, Ex. `.csv`, kept as it is), else the format it was parsed as (or stof, before it's fetched).
fn package_extension(url: &str, given: Option<&str>, parsed: Option<&str>) -> String {
    if let Some(given) = given { return given.to_string(); }
    if let Some(ext) = url_extension(url) { return ext; }
    parsed.unwrap_or("stof").to_string()
}


#[cfg(test)]
mod tests {
    use crate::{model::{Graph, Profile}, runtime::Val};

    #[test]
    /// The hash is checked before anything else (no network needed to see these errors).
    fn bad_arguments() {
        let mut graph = Graph::default();
        graph.parse_stof_src(r#"
            fn try_import(url: str, sha: unknown, into: unknown) -> str {
                try { await import(url, sha, into); return 'imported'; }
                catch (e, msg) { return msg; }
            }
        "#, None, Profile::default()).unwrap();
        let call = |graph: &mut Graph, url: &str, sha: Val, into: Val| graph.call("try_import", None, vec![Val::from(url), sha, into]).unwrap().to_string();
        assert!(call(&mut graph, "https://example.com/a.stof", Val::from("abc"), Val::from("root.a")).contains("64 hexadecimal characters"));
        assert!(call(&mut graph, "https://example.com/a.stof", Val::Null, Val::from("root.a")).contains("needs the sha256"));
        assert!(call(&mut graph, "", Val::from("a".repeat(64).as_str()), Val::from("root.a")).contains("needs a url"));
        assert!(call(&mut graph, "https://example.com/a.stof", Val::from("a".repeat(64).as_str()), Val::Null).contains("needs somewhere to go"));
        // a valid call, but this document has no network access
        let msg = call(&mut graph, "https://example.com/a.stof", Val::from("a".repeat(64).as_str()), Val::from("root.a"));
        assert!(msg.contains("needs network access"), "{msg}");
    }

    #[test]
    /// Package destinations: a name, and an optional version (not a wildcard).
    fn packages() {
        use super::split_package;
        use crate::runtime::semver::Version;
        let v = |text: &str| Version::parse(text).unwrap();
        assert_eq!(split_package("@formata/intent").unwrap(), ("formata/intent".into(), None));
        assert_eq!(split_package("@formata/intent@1.4.0").unwrap(), ("formata/intent".into(), Some(v("1.4.0"))));
        assert_eq!(split_package("@intent@v0.2.0-rc.1").unwrap().1, Some(v("0.2.0-rc.1")));
        assert!(split_package("@formata/intent@1.4").unwrap_err().contains("isn't a version"));
        assert!(split_package("@formata/intent@").is_err());
        assert!(split_package("@formata/intent@1.*").is_err());
    }

    #[test]
    /// Nothing recorded yet: an empty list.
    fn imported_empty() {
        let mut graph = Graph::default();
        graph.parse_stof_src("fn count() -> int imported().len()", None, Profile::default()).unwrap();
        assert_eq!(graph.call("count", None, vec![]).unwrap(), Val::from(0));
    }

    #[cfg(feature = "http")]
    mod served {
        use std::{collections::HashMap, io::{Read, Write}, net::TcpListener, sync::{Arc, Mutex}, thread};
        use bytes::Bytes;
        use sha2::{Digest, Sha256};
        use crate::{model::{Graph, Profile}, runtime::Val};

        /// A tiny HTTP server for the tests: path -> (status, content type, body), counting requests per path.
        struct Server {
            base: String,
            hits: Arc<Mutex<HashMap<String, usize>>>,
            /// The last request's head for each path (to check headers).
            heads: Arc<Mutex<HashMap<String, String>>>,
        }
        impl Server {
            fn start(routes: Vec<(&str, u16, &str, Vec<u8>)>) -> Self {
                let routes: Vec<(String, u16, String, Vec<u8>)> = routes.into_iter().map(|(path, status, ctype, body)| (path.to_string(), status, ctype.to_string(), body)).collect();
                Self::start_with(move |_| routes)
            }
            /// Routes made knowing the server's base url (for packages that import from it).
            fn start_with(make: impl FnOnce(&str) -> Vec<(String, u16, String, Vec<u8>)>) -> Self {
                let listener = TcpListener::bind("127.0.0.1:0").unwrap();
                let base = format!("http://{}", listener.local_addr().unwrap());
                let routes: HashMap<String, (u16, String, Vec<u8>)> = make(&base).into_iter().map(|(path, status, ctype, body)| (path, (status, ctype, body))).collect();
                let routes = Arc::new(routes);
                let hits = Arc::new(Mutex::new(HashMap::new()));
                let heads = Arc::new(Mutex::new(HashMap::new()));
                let counter = hits.clone();
                let seen = heads.clone();
                thread::spawn(move || {
                    for stream in listener.incoming() {
                        let Ok(mut stream) = stream else { continue };
                        let routes = routes.clone();
                        let counter = counter.clone();
                        let seen = seen.clone();
                        thread::spawn(move || {
                            let mut request = Vec::new();
                            let mut buf = [0u8; 1024];
                            while !request.windows(4).any(|w| w == b"\r\n\r\n") {
                                match stream.read(&mut buf) { Ok(0) | Err(_) => return, Ok(n) => request.extend_from_slice(&buf[..n]) }
                            }
                            let head = String::from_utf8_lossy(&request).to_string();
                            let path = head.split_whitespace().nth(1).unwrap_or("/").to_string();
                            *counter.lock().unwrap().entry(path.clone()).or_insert(0) += 1;
                            seen.lock().unwrap().insert(path.clone(), head.to_ascii_lowercase());
                            if path.starts_with("/slow/") { thread::sleep(std::time::Duration::from_millis(400)); }
                            let (status, ctype, body) = routes.get(&path).cloned().unwrap_or((404, "text/plain".into(), b"not found".to_vec()));
                            let head = format!("HTTP/1.1 {status} X\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len());
                            let _ = stream.write_all(head.as_bytes());
                            let _ = stream.write_all(&body);
                        });
                    }
                });
                Self { base, hits, heads }
            }
            fn hits(&self, path: &str) -> usize {
                *self.hits.lock().unwrap().get(path).unwrap_or(&0)
            }
            fn head(&self, path: &str) -> String {
                self.heads.lock().unwrap().get(path).cloned().unwrap_or_default()
            }
            fn host(&self) -> String {
                self.base.trim_start_matches("http://").to_string()
            }
        }

        fn sha(bytes: &[u8]) -> String {
            crate::model::seal::hex(&Sha256::digest(bytes)).to_string()
        }

        const PKG: &str = "fn hello() -> str 'hello'\nversion: 1\n";
        const PKG2: &str = "fn hello() -> str 'hello2'\nother: 2\n";
        const BROKEN: &str = "fn ok() -> int 1\nfn bad( {\n";
        const JSON: &str = r#"{"name": "json-dep", "count": 3}"#;

        fn served() -> Server {
            Server::start(vec![
                ("/pkg.stof", 200, "text/plain", PKG.as_bytes().to_vec()),
                ("/pkg2.stof", 200, "text/plain; charset=utf-8", PKG2.as_bytes().to_vec()),
                ("/plain", 200, "text/plain", PKG.as_bytes().to_vec()),
                ("/broken.stof", 200, "text/plain", BROKEN.as_bytes().to_vec()),
                ("/data.json", 200, "application/octet-stream", JSON.as_bytes().to_vec()),
                ("/data", 200, "application/json", JSON.as_bytes().to_vec()),
                ("/pkg.stof?v=2", 200, "text/plain", PKG.as_bytes().to_vec()),
                ("/error", 500, "text/plain", b"oops".to_vec()),
            ])
        }

        /// A document allowed to reach the test server, with its source after the server's details are filled in.
        fn doc(server: &Server, src: &str) -> Graph {
            let mut graph = Graph::default();
            graph.quiet(true);
            graph.allow_http_hosts([server.host()]);
            let src = src
                .replace("{base}", &server.base)
                .replace("{pkg}", &sha(PKG.as_bytes()))
                .replace("{pkg2}", &sha(PKG2.as_bytes()))
                .replace("{broken}", &sha(BROKEN.as_bytes()))
                .replace("{json}", &sha(JSON.as_bytes()));
            graph.parse_stof_src(&src, None, Profile::default()).unwrap();
            graph
        }

        fn call(graph: &mut Graph, func: &str) -> Val {
            match graph.call(func, None, vec![]) {
                Ok(val) => val,
                Err(error) => panic!("{func} failed: {error}"),
            }
        }

        #[test]
        /// Loads once: true, then false without fetching again.
        fn loads_then_skips() {
            let server = served();
            let mut graph = doc(&server, r#"
                fn first() -> bool await import('{base}/pkg.stof', '{pkg}', 'root.deps.a')
                fn hello() -> str root.deps.a.hello()
                fn version() -> int root.deps.a.version
            "#);
            assert_eq!(call(&mut graph, "first"), Val::from(true));
            assert_eq!(call(&mut graph, "hello"), Val::from("hello"));
            assert_eq!(call(&mut graph, "version"), Val::from(1));
            assert_eq!(call(&mut graph, "first"), Val::from(false));
            assert_eq!(server.hits("/pkg.stof"), 1, "a recorded import isn't fetched again");
        }

        #[test]
        /// The same hash in two places is two copies; two hashes in one place are both recorded.
        fn locations_and_hashes() {
            let server = served();
            let mut graph = doc(&server, r#"
                fn two_places() -> bool {
                    const a = await import('{base}/pkg.stof', '{pkg}', 'root.deps.a');
                    const b = await import('{base}/pkg.stof', '{pkg}', 'root.deps.b');
                    a && b
                }
                fn two_hashes() -> bool {
                    const a = await import('{base}/pkg.stof', '{pkg}', 'root.deps.c');
                    const b = await import('{base}/pkg2.stof', '{pkg2}', 'root.deps.c');
                    a && b
                }
                fn check() -> bool {
                    assert_eq(root.deps.a.hello(), 'hello');
                    assert_eq(root.deps.b.hello(), 'hello');
                    assert_eq(root.deps.c.hello(), 'hello2'); // the second replaced the same name, as parse does
                    assert_eq(root.deps.c.version, 1);
                    assert_eq(root.deps.c.other, 2);
                    assert_eq(imported(root.deps.c).len(), 2);
                    assert_eq(imported().len(), 4);
                    true
                }
            "#);
            assert_eq!(call(&mut graph, "two_places"), Val::from(true));
            assert_eq!(call(&mut graph, "two_hashes"), Val::from(true));
            assert_eq!(call(&mut graph, "check"), Val::from(true));
            assert_eq!(server.hits("/pkg.stof"), 3);
        }

        #[test]
        /// A different hash is an error, nothing is parsed, and nothing is recorded. Uppercase hashes are fine.
        fn pinned() {
            let server = served();
            let mut graph = doc(&server, r#"
                fn wrong() -> str {
                    try { await import('{base}/pkg.stof', '{pkg2}', 'root.deps.w'); return 'imported'; }
                    catch (e, msg) { return msg; }
                }
                fn nothing_there() -> bool !root?.deps?.w?.hello && imported().len() == 0
                fn upper() -> bool await import('{base}/pkg.stof', '{pkg}'.upper(), 'root.deps.u')
                fn upper_again() -> bool await import('{base}/pkg.stof', '{pkg}', 'root.deps.u')
            "#);
            let msg = call(&mut graph, "wrong").to_string();
            assert!(msg.contains("not the pinned"), "{msg}");
            assert_eq!(call(&mut graph, "nothing_there"), Val::from(true));
            assert_eq!(call(&mut graph, "upper"), Val::from(true));
            assert_eq!(call(&mut graph, "upper_again"), Val::from(false), "recorded in lowercase");
        }

        #[test]
        /// Server errors and parse errors are thrown, and record nothing. Error catch values name it Import.
        fn failures() {
            let server = served();
            let mut graph = doc(&server, r#"
                fn attempt(url: str, sha: str, into: str) -> str {
                    try { await import(url, sha, into); return 'imported'; }
                    catch (e, msg) { return msg; }
                }
                fn missing() -> str self.attempt('{base}/missing.stof', '{pkg}', 'root.deps.m')
                fn server_error() -> str self.attempt('{base}/error', '{pkg}', 'root.deps.e')
                fn broken() -> str self.attempt('{base}/broken.stof', '{broken}', 'root.deps.x')
                fn none_recorded() -> int imported().len()
            "#);
            let msg = call(&mut graph, "missing").to_string();
            assert!(msg.contains("answered 404"), "{msg}");
            let msg = call(&mut graph, "server_error").to_string();
            assert!(msg.contains("answered 500"), "{msg}");
            let msg = call(&mut graph, "broken").to_string();
            assert!(msg.contains("failed to parse as stof"), "{msg}");
            assert_eq!(call(&mut graph, "none_recorded"), Val::from(0));
            // a failed parse records nothing, so it's tried (and fetched) again
            let msg = call(&mut graph, "broken").to_string();
            assert!(msg.contains("failed to parse"), "{msg}");
            assert_eq!(server.hits("/broken.stof"), 2);
        }

        #[test]
        /// Where it goes: an object, a self path, an existing field, and a field that isn't an object.
        fn into_targets() {
            let server = served();
            let mut graph = doc(&server, r#"
                holder: {}
                number: 5
                child: { fn load_here() -> bool await import('{base}/pkg.stof', '{pkg}', 'self.inner') }
                fn as_obj() -> bool await import('{base}/pkg.stof', '{pkg}', self.holder)
                fn as_self_path() -> bool self.child.load_here()
                fn into_number() -> str {
                    try { await import('{base}/pkg.stof', '{pkg}', 'self.number'); return 'imported'; }
                    catch (e, msg) { return msg; }
                }
                fn check() -> bool {
                    assert_eq(self.holder.hello(), 'hello');
                    assert_eq(self.child.inner.hello(), 'hello');
                    assert_eq(imported(self.child)[0].get('into'), self.child.inner);
                    true
                }
            "#);
            assert_eq!(call(&mut graph, "as_obj"), Val::from(true));
            assert_eq!(call(&mut graph, "as_self_path"), Val::from(true));
            let msg = call(&mut graph, "into_number").to_string();
            assert!(msg.contains("doesn't hold an object"), "{msg}");
            assert_eq!(call(&mut graph, "check"), Val::from(true));
        }

        #[test]
        /// The format: the extension (over a generic content type), the content type, stof by default, or given.
        fn formats() {
            let server = served();
            let mut graph = doc(&server, r#"
                fn by_extension() -> str { await import('{base}/data.json', '{json}', 'root.ext'); root.ext.name }
                fn by_content_type() -> str { await import('{base}/data', '{json}', 'root.ctype'); root.ctype.name }
                fn plain_is_stof() -> str { await import('{base}/plain', '{pkg}', 'root.plain'); root.plain.hello() }
                fn query_string() -> str { await import('{base}/pkg.stof?v=2', '{pkg}', 'root.query'); root.query.hello() }
                fn given() -> int { await import('{base}/plain', '{pkg}', 'root.given', 'text'); root.given.text.len() }
            "#);
            assert_eq!(call(&mut graph, "by_extension"), Val::from("json-dep"));
            assert_eq!(call(&mut graph, "by_content_type"), Val::from("json-dep"));
            assert_eq!(call(&mut graph, "plain_is_stof"), Val::from("hello"));
            assert_eq!(call(&mut graph, "query_string"), Val::from("hello"));
            assert_eq!(call(&mut graph, "given"), Val::from(PKG.len() as i64));
        }

        #[test]
        /// The record travels in BSTF (no fetch after the round trip), moves with its object, and goes when it's dropped.
        fn records() {
            let server = served();
            let src = r#"
                fn load() -> bool await import('{base}/pkg.stof', '{pkg}', 'root.deps.a')
                other: {}
                fn listed() -> str imported()[0].get('url')
                fn move_it() -> bool root.deps.a.move(self.other)
                fn moved_path() -> str imported()[0].get('into').path()
                fn drop_it() -> int { drop(imported()[0].get('into')); imported().len() }
            "#;
            let mut graph = doc(&server, src);
            assert_eq!(call(&mut graph, "load"), Val::from(true));
            assert_eq!(call(&mut graph, "listed"), Val::from(format!("{}/pkg.stof", server.base).as_str()));

            let bytes: Bytes = graph.binary_export("bstf", None).unwrap();
            let mut back = Graph::default();
            back.allow_http_hosts([server.host()]);
            back.binary_import("bstf", bytes, None, &Profile::default()).unwrap();
            assert_eq!(call(&mut back, "load"), Val::from(false), "the record came with the document");
            assert_eq!(server.hits("/pkg.stof"), 1);

            assert_eq!(call(&mut back, "move_it"), Val::from(true));
            assert_eq!(call(&mut back, "moved_path"), Val::from("root.other.a"), "the record moved with its object");
            assert_eq!(call(&mut back, "drop_it"), Val::from(0));
            assert_eq!(call(&mut back, "load"), Val::from(true), "dropped with its object: imported again");
            assert_eq!(server.hits("/pkg.stof"), 2);
        }

        #[test]
        /// An import that isn't awaited still finishes before the call returns; concurrent imports end correct.
        fn not_awaited_and_concurrent() {
            let server = served();
            let mut graph = doc(&server, r#"
                fn start() { import('{base}/pkg.stof', '{pkg}', 'root.bg'); }
                fn loaded() -> str root.bg.hello()
                async fn one() -> bool await import('{base}/pkg2.stof', '{pkg2}', 'root.same')
                fn both() -> bool {
                    const a = self.one();
                    const b = self.one();
                    await a; await b;
                    root.same.hello() == 'hello2' && imported(root.same).len() == 1
                }
            "#);
            call(&mut graph, "start");
            assert_eq!(call(&mut graph, "loaded"), Val::from("hello"));
            assert_eq!(call(&mut graph, "both"), Val::from(true));
        }

        #[test]
        /// The network rules: a host the document isn't allowed, and a sealed part allowed no hosts.
        fn network_rules() {
            let server = served();
            let mut graph = Graph::default();
            graph.quiet(true);
            graph.allow_http_hosts(["api.example.com"]);
            let src = r#"
                fn blocked() -> str {
                    try { await import('{base}/pkg.stof', '{pkg}', 'root.deps.a'); return 'imported'; }
                    catch (e, msg) { return msg; }
                }
            "#.replace("{base}", &server.base).replace("{pkg}", &sha(PKG.as_bytes()));
            graph.parse_stof_src(&src, None, Profile::default()).unwrap();
            let msg = call(&mut graph, "blocked").to_string();
            assert!(msg.contains(&format!("import of '{}/pkg.stof' failed: ", server.base)) && msg.contains("not an allowed host"), "{msg}");
            assert_eq!(server.hits("/pkg.stof"), 0);

            let mut graph = doc(&server, r#"
                tool: {
                    fn get() -> str {
                        try { await import('{base}/pkg.stof', '{pkg}', self); return 'imported'; }
                        catch (e, msg) { return msg; }
                    }
                }
                #[init] fn lock() { seal(self.tool, hosts = [], libs = ['Std', 'Http']); }
                fn sealed_get() -> str self.tool.get()
            "#);
            let msg = call(&mut graph, "sealed_get").to_string();
            assert!(msg.contains("import of") && msg.contains("not a host this sealed part may reach"), "{msg}");
            assert_eq!(server.hits("/pkg.stof"), 0, "the seal allows no hosts");
        }
    
        #[test]
        /// Profiles: untrusted refuses a package that runs code while it's parsed; test keeps its tests.
        fn profiles() {
            let server = Server::start(vec![
                ("/init.stof", 200, "text/plain", b"#[init] fn setup() { self.ready = true; }\nready: false\n".to_vec()),
                ("/tests.stof", 200, "text/plain", b"#[test] fn checks() {}\nfn helper() -> int 1\n".to_vec()),
            ]);
            let init = sha(b"#[init] fn setup() { self.ready = true; }\nready: false\n");
            let tests = sha(b"#[test] fn checks() {}\nfn helper() -> int 1\n");
            let mut graph = doc(&server, &r#"
                fn untrusted() -> str {
                    try { await import('{base}/init.stof', '{init}', 'root.u', profile = 'untrusted'); return 'imported'; }
                    catch (e, msg) { return msg; }
                }
                fn prod() -> bool { await import('{base}/init.stof', '{init}', 'root.p'); root.p.ready }
                fn prod_tests() -> int { await import('{base}/tests.stof', '{tests}', 'root.t'); root.t.funcs().len() }
                fn test_tests() -> int { await import('{base}/tests.stof', '{tests}', 'root.tt', profile = 'test'); root.tt.funcs().len() }
            "#.replace("{init}", &init).replace("{tests}", &tests));
            let msg = call(&mut graph, "untrusted").to_string();
            assert!(msg.contains("failed to parse") && msg.contains("#[init]"), "{msg}");
            assert_eq!(call(&mut graph, "prod"), Val::from(true), "#[init] ran while it was parsed");
            assert_eq!(call(&mut graph, "prod_tests"), Val::from(1), "prod leaves #[test]s out");
            assert_eq!(call(&mut graph, "test_tests"), Val::from(2));
        }

        #[test]
        /// Importing into a sealed object is the seal's error, and records nothing.
        fn into_sealed() {
            let server = served();
            let mut graph = doc(&server, r#"
                locked: { fn existing() -> int 1 }
                #[init] fn lock() { seal(self.locked); }
                fn attempt() -> str {
                    try { await import('{base}/pkg.stof', '{pkg}', self.locked); return 'imported'; }
                    catch (e, msg) { return msg; }
                }
                fn none() -> int imported().len()
            "#);
            let msg = call(&mut graph, "attempt").to_string();
            assert!(msg.contains("sealed"), "{msg}");
            assert_eq!(call(&mut graph, "none"), Val::from(0));
        }

        #[test]
        /// imported() takes an object or nothing.
        fn imported_argument() {
            let server = served();
            let mut graph = doc(&server, r#"
                fn bad() -> str { try { imported(5); return 'listed'; } catch (e, msg) { return msg; } }
            "#);
            let msg = call(&mut graph, "bad").to_string();
            assert!(msg.contains("imported takes an object"), "{msg}");
        }
    
        /// Packages that import their own dependencies: C (nothing), B (imports C into root.deps.c), A (imports B), and
        /// D (also imports C into root.deps.c), and E (imports C into its own object). Sources with their hashes. (Pinned imports can't form a cycle: a package
        /// would need the hash of a package that contains its own hash.)
        fn packages(base: &str) -> (Vec<(String, u16, String, Vec<u8>)>, HashMap<&'static str, String>) {
            let c = "fn c() -> str 'c'\n".to_string();
            let b = format!("fn b() -> str 'b'\n#[load] fn deps() {{ await import('{base}/c.stof', '{}', 'root.deps.c'); }}\n", sha(c.as_bytes()));
            let a = format!("fn a() -> str 'a'\n#[load] fn deps() {{ await import('{base}/b.stof', '{}', 'root.deps.b'); }}\n", sha(b.as_bytes()));
            let d = format!("fn d() -> str 'd'\n#[load] fn deps() {{ await import('{base}/c.stof', '{}', 'root.deps.c'); }}\n", sha(c.as_bytes()));
            let e = format!("fn e() -> str 'e'\n#[load] fn deps() {{ await import('{base}/c.stof', '{}', 'self.c'); }}\n", sha(c.as_bytes()));
            let fails = "fn f() -> str 'f'\n#[load] fn deps() { throw('dependency missing'); }\n".to_string();
            let with_load = "#[load] fn deps() {}\nfn w() -> int 1\n".to_string();
            let mut hashes = HashMap::new();
            hashes.insert("a", sha(a.as_bytes()));
            hashes.insert("b", sha(b.as_bytes()));
            hashes.insert("c", sha(c.as_bytes()));
            hashes.insert("d", sha(d.as_bytes()));
            hashes.insert("e", sha(e.as_bytes()));
            hashes.insert("fails", sha(fails.as_bytes()));
            hashes.insert("with_load", sha(with_load.as_bytes()));
            let route = |path: &str, body: String| (path.to_string(), 200u16, "text/plain".to_string(), body.into_bytes());
            (vec![
                route("/a.stof", a), route("/b.stof", b), route("/c.stof", c), route("/d.stof", d),
                route("/e.stof", e), route("/fails.stof", fails), route("/with_load.stof", with_load),
            ], hashes)
        }

        fn package_server() -> (Server, HashMap<&'static str, String>) {
            let hashes = Arc::new(Mutex::new(HashMap::new()));
            let found = hashes.clone();
            let server = Server::start_with(move |base| {
                let (routes, hashes) = packages(base);
                *found.lock().unwrap() = hashes;
                routes
            });
            let hashes = hashes.lock().unwrap().clone();
            (server, hashes)
        }

        fn fill(src: &str, server: &Server, hashes: &HashMap<&'static str, String>) -> String {
            let mut src = src.replace("{base}", &server.base);
            for (name, hash) in hashes { src = src.replace(&format!("{{{name}}}"), hash); }
            src
        }

        #[test]
        /// Dependencies of dependencies load (each import runs its package's #[load]); a shared one loads once.
        fn dependencies_of_dependencies() {
            let (server, hashes) = package_server();
            let mut graph = Graph::default();
            graph.quiet(true);
            graph.allow_http_hosts([server.host()]);
            graph.parse_stof_src(&fill(r#"
                #[load] fn deps() {
                    await import('{base}/a.stof', '{a}', 'root.deps.a');
                    await import('{base}/d.stof', '{d}', 'root.deps.d');
                }
                fn run() -> int await load()
                fn check() -> bool {
                    assert_eq(root.deps.a.a(), 'a');
                    assert_eq(root.deps.b.b(), 'b');
                    assert_eq(root.deps.c.c(), 'c');
                    assert_eq(root.deps.d.d(), 'd');
                    assert_eq(imported().len(), 4);
                    true
                }
            "#, &server, &hashes), None, Profile::default()).unwrap();
            call(&mut graph, "run");
            assert_eq!(call(&mut graph, "check"), Val::from(true));
            assert_eq!(server.hits("/c.stof"), 1, "the shared dependency loaded once");
            // everything's recorded: loading again fetches nothing
            call(&mut graph, "run");
            for path in ["/a.stof", "/b.stof", "/c.stof", "/d.stof"] { assert_eq!(server.hits(path), 1, "{path}"); }
        }

        #[test]
        /// A package whose #[load] fails: the import fails, but it's recorded, so load() runs that #[load] again.
        fn package_load_fails() {
            let (server, hashes) = package_server();
            let mut graph = Graph::default();
            graph.quiet(true);
            graph.allow_http_hosts([server.host()]);
            graph.parse_stof_src(&fill(r#"
                fn attempt() -> str {
                    try { await import('{base}/fails.stof', '{fails}', 'root.deps.f'); return 'imported'; }
                    catch (e, msg) { return msg; }
                }
                fn recorded() -> int imported().len()
                fn parsed() -> str root.deps.f.f()
                fn reload() -> str { try { await load(); return 'loaded'; } catch (e, msg) { return msg; } }
            "#, &server, &hashes), None, Profile::default()).unwrap();
            let msg = call(&mut graph, "attempt").to_string();
            assert!(msg.contains("loaded, but its #[load] failed") && msg.contains("dependency missing"), "{msg}");
            assert_eq!(call(&mut graph, "recorded"), Val::from(1));
            assert_eq!(call(&mut graph, "parsed"), Val::from("f"));
            let msg = call(&mut graph, "reload").to_string();
            assert!(msg.contains("dependency missing"), "{msg}");
            assert_eq!(call(&mut graph, "attempt"), Val::from("imported"), "already recorded: skipped");
            assert_eq!(server.hits("/fails.stof"), 1);
        }

        #[test]
        /// An untrusted import can't bring #[load] functions (unconfined code took in someone else's source).
        fn untrusted_with_load() {
            let (server, hashes) = package_server();
            let mut graph = Graph::default();
            graph.quiet(true);
            graph.allow_http_hosts([server.host()]);
            graph.parse_stof_src(&fill(r#"
                fn attempt() -> str {
                    try { await import('{base}/with_load.stof', '{with_load}', 'root.u', profile = 'untrusted'); return 'imported'; }
                    catch (e, msg) { return msg; }
                }
                fn plain() -> bool await import('{base}/with_load.stof', '{with_load}', 'root.p')
            "#, &server, &hashes), None, Profile::default()).unwrap();
            let msg = call(&mut graph, "attempt").to_string();
            assert!(msg.contains("untrusted source can't have #[load] functions"), "{msg}");
            assert_eq!(call(&mut graph, "plain"), Val::from(true));
        }
    
        #[test]
        /// A sealed part can import into its own part, including a package with its own #[load] (its dependencies are
        /// confined by the same seal); a dependency outside the part is refused.
        fn sealed_part_imports() {
            let (server, hashes) = package_server();
            let mut graph = Graph::default();
            graph.quiet(true);
            graph.allow_http_hosts([server.host()]);
            graph.parse_stof_src(&fill(r#"
                part: {
                    fn own() -> str {
                        await import('{base}/e.stof', '{e}', 'self.e');
                        self.e.e() + self.e.c.c()
                    }
                    fn outside() -> str {
                        try { await import('{base}/b.stof', '{b}', 'self.b'); return 'imported'; }
                        catch (e, msg) { return msg; }
                    }
                }
                #[init] fn lock() { seal(self.part, hosts = ['HOST'], reads = []); } // reads: changes only its own part
                fn own() -> str self.part.own()
                fn outside() -> str self.part.outside()
                fn nothing_outside() -> bool !root?.deps?.c
            "#.replace("HOST", &server.host()).as_str(), &server, &hashes), None, Profile::default()).unwrap();
            assert_eq!(call(&mut graph, "own"), Val::from("ec"));
            let msg = call(&mut graph, "outside").to_string();
            assert!(msg.contains("its #[load] failed") && msg.contains("sealed"), "{msg}");
            assert_eq!(call(&mut graph, "nothing_outside"), Val::from(true), "the seal kept the dependency out of the document");
        }
    
        #[test]
        /// The Rust host's Graph::load: the document's #[load] imports, fetched once however often it's called.
        fn host_load() {
            let (server, hashes) = package_server();
            let mut graph = Graph::default();
            graph.quiet(true);
            graph.allow_http_hosts([server.host()]);
            graph.parse_stof_src(&fill(r#"
                #[load] fn deps() { await import('{base}/a.stof', '{a}', 'root.deps.a'); }
                fn chain() -> str root.deps.a.a() + root.deps.b.b() + root.deps.c.c()
            "#, &server, &hashes), None, Profile::default()).unwrap();
            assert_eq!(graph.load(None).unwrap(), Val::from(1));
            assert_eq!(call(&mut graph, "chain"), Val::from("abc"));
            assert_eq!(graph.load(None).unwrap(), Val::from(3), "the packages' #[load] functions are in the document now");
            for path in ["/a.stof", "/b.stof", "/c.stof"] { assert_eq!(server.hits(path), 1, "{path}"); }
        }
    
        /// A package folder for one test under the package directory ("stof/<unique>"), removed when it's done.
        struct PackageDir { name: String }
        impl PackageDir {
            fn new() -> Self { Self { name: format!("__import_test_{}", nanoid::nanoid!(8, &nanoid::alphabet::SAFE[2..])) } }
            fn file(&self, rest: &str) -> std::path::PathBuf { std::path::PathBuf::from(format!("stof/{}/{rest}", self.name)) }
        }
        impl Drop for PackageDir {
            fn drop(&mut self) { let _ = std::fs::remove_dir_all(format!("stof/{}", self.name)); }
        }

        fn with_files(server: &Server, src: &str, dir: &PackageDir) -> Graph {
            let mut graph = Graph::default();
            graph.quiet(true);
            graph.allow_system();
            graph.allow_http_hosts([server.host()]);
            parse_into(&mut graph, server, src, dir);
            graph
        }

        fn parse_into(graph: &mut Graph, server: &Server, src: &str, dir: &PackageDir) {
            let src = src
                .replace("{base}", &server.base)
                .replace("{pkg}", &sha(PKG.as_bytes()))
                .replace("{pkg2}", &sha(PKG2.as_bytes()))
                .replace("{json}", &sha(JSON.as_bytes()))
                .replace("{dir}", &dir.name);
            graph.parse_stof_src(&src, None, Profile::default()).unwrap();
        }

        #[test]
        /// '@name' with the file system: written to the package directory (with fs), parsed into the root, recorded;
        /// a later document finds the file and doesn't fetch it.
        fn package_with_files() {
            let server = served();
            let dir = PackageDir::new();
            let src = r#"
                fn get() -> bool await import('{base}/pkg.stof', '{pkg}', '@{dir}/tools/hello')
                fn hello() -> str root.hello()
            "#;
            let mut graph = with_files(&server, src, &dir);
            assert_eq!(call(&mut graph, "get"), Val::from(true));
            assert_eq!(call(&mut graph, "hello"), Val::from("hello"), "parsed into the root");
            assert_eq!(std::fs::read_to_string(dir.file("tools/hello.stof")).unwrap(), PKG, "written to the package directory");
            assert_eq!(call(&mut graph, "get"), Val::from(false));
            assert_eq!(server.hits("/pkg.stof"), 1);

            // another run: the file is there, so nothing's fetched
            let mut again = with_files(&server, src, &dir);
            assert_eq!(call(&mut again, "get"), Val::from(true));
            assert_eq!(call(&mut again, "hello"), Val::from("hello"));
            assert_eq!(server.hits("/pkg.stof"), 1, "read from the package's file");

            // a changed file isn't trusted: fetched again and written over
            std::fs::write(dir.file("tools/hello.stof"), "fn hello() -> str 'changed'\n").unwrap();
            let mut changed = with_files(&server, src, &dir);
            assert_eq!(call(&mut changed, "get"), Val::from(true));
            assert_eq!(call(&mut changed, "hello"), Val::from("hello"));
            assert_eq!(server.hits("/pkg.stof"), 2);
            assert_eq!(std::fs::read_to_string(dir.file("tools/hello.stof")).unwrap(), PKG);

            // the format's extension, from the URL (or its name already has it)
            let mut json = with_files(&server, r#"fn get() -> str { await import('{base}/data.json', '{json}', '@{dir}/data'); root.name }"#, &dir);
            assert_eq!(call(&mut json, "get"), Val::from("json-dep"));
            assert!(dir.file("data.json").exists());
        }

        #[test]
        /// '@name' without the file system: parsed into the root only (nothing written).
        fn package_without_files() {
            let server = served();
            let dir = PackageDir::new();
            let mut graph = doc(&server, &r#"
                fn get() -> bool await import('{base}/pkg.stof', '{pkg}', '@DIR/hello')
                fn hello() -> str root.hello()
                fn listed() -> int imported(root).len()
            "#.replace("DIR", &dir.name));
            assert_eq!(call(&mut graph, "get"), Val::from(true));
            assert_eq!(call(&mut graph, "hello"), Val::from("hello"));
            assert_eq!(call(&mut graph, "listed"), Val::from(1));
            assert!(!dir.file("hello.stof").exists(), "nothing written without fs");
            assert_eq!(call(&mut graph, "get"), Val::from(false));
        }

        #[test]
        /// Vendor only (the CLI installing): written, not parsed (no package code runs); then skipped. Without the
        /// file system it's an error.
        fn vendor_only() {
            let server = Server::start(vec![("/runs.stof", 200, "text/plain", b"#[init] fn ran() { root.ran = true; }\nfn hello() -> str 'hello'\n".to_vec())]);
            let runs = sha(b"#[init] fn ran() { root.ran = true; }\nfn hello() -> str 'hello'\n");
            let dir = PackageDir::new();
            let src = r#"
                fn install() -> bool await import('{base}/runs.stof', 'RUNS', '@{dir}/runs')
                fn did_run() -> bool root.ran ?? false
                fn parsed() -> bool root?.hello != null
            "#.replace("RUNS", &runs);
            let mut graph = with_files(&server, &src, &dir);
            graph.import_vendor_only = true;
            assert_eq!(call(&mut graph, "install"), Val::from(true));
            assert!(dir.file("runs.stof").exists());
            assert_eq!(call(&mut graph, "did_run"), Val::from(false), "no package code ran");
            assert_eq!(call(&mut graph, "parsed"), Val::from(false));
            assert_eq!(call(&mut graph, "install"), Val::from(false), "installed already");
            assert_eq!(server.hits("/runs.stof"), 1);

            let mut no_files = Graph::default();
            no_files.quiet(true);
            no_files.allow_http_hosts([server.host()]);
            no_files.import_vendor_only = true;
            parse_into(&mut no_files, &server, &format!("{src}\nfn attempt() -> str {{ try {{ self.install(); return 'installed'; }} catch (e, msg) {{ return msg; }} }}"), &dir);
            let msg = call(&mut no_files, "attempt").to_string();
            assert!(msg.contains("needs this document's file system"), "{msg}");
        }

        #[test]
        /// Package names are plain names: nothing that could leave the package directory.
        fn package_names() {
            let server = served();
            let dir = PackageDir::new();
            let mut graph = with_files(&server, r#"
                fn attempt(into: str) -> str {
                    try { await import('{base}/pkg.stof', '{pkg}', into); return 'imported'; }
                    catch (e, msg) { return msg; }
                }
            "#, &dir);
            for name in ["@", "@../escape", "@a/../../escape", "@a//b", "@/abs", "@a/b c", "@a\\b", "@./x"] {
                let msg = graph.call("attempt", None, vec![Val::from(name)]).unwrap().to_string();
                assert!(msg.contains("isn't a package name"), "{name}: {msg}");
            }
            assert!(!std::path::Path::new("escape.stof").exists() && !std::path::Path::new("../escape.stof").exists());
        }

        #[test]
        /// A sealed part whose libs don't include fs can't write the package directory: parsed into the graph only.
        fn package_sealed_without_fs() {
            let server = served();
            let dir = PackageDir::new();
            let mut graph = with_files(&server, &r#"
                part: { fn get() -> str { await import('{base}/pkg.stof', '{pkg}', '@{dir}/hello'); 'done' } }
                #[init] fn lock() { seal(self.part, hosts = ['HOST'], libs = ['Std', 'Http']); }
                fn get() -> str self.part.get()
            "#.replace("HOST", &server.host()), &dir);
            assert_eq!(call(&mut graph, "get"), Val::from("done"));
            assert!(!dir.file("hello.stof").exists(), "the seal's libs don't include fs");
        }

        #[derive(Debug, Default)]
        struct MemoryCache { files: Mutex<HashMap<String, Bytes>>, gets: Mutex<usize> }
        impl super::super::ImportCache for MemoryCache {
            fn get(&self, sha256: &str) -> Option<Bytes> { *self.gets.lock().unwrap() += 1; self.files.lock().unwrap().get(sha256).cloned() }
            fn put(&self, sha256: &str, bytes: &Bytes) { self.files.lock().unwrap().insert(sha256.to_string(), bytes.clone()); }
        }

        #[test]
        /// The host's cache: fetched bytes go in; a document with the cache doesn't fetch, even without network
        /// access; bytes in the cache that don't match the hash are ignored.
        fn cache() {
            let server = served();
            let cache = Arc::new(MemoryCache::default());
            let src = "fn get() -> str { await import('{base}/pkg.stof', '{pkg}', 'root.dep'); root.dep.hello() }";
            let mut graph = doc(&server, src);
            graph.import_cache = Some(cache.clone());
            assert_eq!(call(&mut graph, "get"), Val::from("hello"));
            assert_eq!(server.hits("/pkg.stof"), 1);
            assert!(cache.files.lock().unwrap().contains_key(&sha(PKG.as_bytes())));

            let mut offline = Graph::default(); // no network access at all
            offline.quiet(true);
            offline.import_cache = Some(cache.clone());
            offline.parse_stof_src(&src.replace("{base}", &server.base).replace("{pkg}", &sha(PKG.as_bytes())), None, Profile::default()).unwrap();
            assert_eq!(call(&mut offline, "get"), Val::from("hello"));
            assert_eq!(server.hits("/pkg.stof"), 1, "from the cache");

            cache.files.lock().unwrap().insert(sha(PKG.as_bytes()), Bytes::from_static(b"fn hello() -> str 'tampered'"));
            let mut fresh = doc(&server, src);
            fresh.import_cache = Some(cache.clone());
            assert_eq!(call(&mut fresh, "get"), Val::from("hello"), "a cache entry that doesn't match isn't used");
            assert_eq!(server.hits("/pkg.stof"), 2);
        }

        #[test]
        /// The directory cache keeps a file per hash.
        fn dir_cache() {
            use super::super::{DirImportCache, ImportCache};
            let dir = PackageDir::new();
            let cache = DirImportCache::new(dir.file("cache"));
            let hash = sha(b"bytes");
            assert!(cache.get(&hash).is_none());
            cache.put(&hash, &Bytes::from_static(b"bytes"));
            assert_eq!(cache.get(&hash).unwrap(), Bytes::from_static(b"bytes"));
            assert!(cache.get("../../etc/passwd").is_none(), "only hashes are file names");
            cache.put("not-a-hash", &Bytes::from_static(b"x"));
            assert!(!dir.file("cache/not-a-hash").exists());
        }

        #[test]
        /// A package parsed into the root runs only the #[load] functions it brought, not the document's others.
        fn package_runs_only_its_own_loads() {
            let server = Server::start_with(|base| {
                let pkg = "fn hello() -> str 'hello'\n#[load] fn own() { root.own_ran += 1; }\n".to_string();
                let _ = base;
                vec![("/loads.stof".to_string(), 200, "text/plain".to_string(), pkg.into_bytes())]
            });
            let pkg = sha(b"fn hello() -> str 'hello'\n#[load] fn own() { root.own_ran += 1; }\n");
            let mut graph = Graph::default();
            graph.quiet(true);
            graph.allow_http_hosts([server.host()]);
            graph.parse_stof_src(&format!(r#"
                own_ran: 0
                other_ran: 0
                #[load] fn other() {{ root.other_ran += 1; }}
                fn get() -> bool await import('{}/loads.stof', '{pkg}', '@pkg/loads')
                fn counts() -> str `${{self.own_ran}} ${{self.other_ran}}`
            "#, server.base), None, Profile::default()).unwrap();
            assert_eq!(call(&mut graph, "get"), Val::from(true));
            assert_eq!(call(&mut graph, "counts"), Val::from("1 0"));
        }

        /*****************************************************************************
         * Versions.
         *****************************************************************************/

        const LIB12: &str = "fn version() -> str '1.2'\n";
        const LIB14: &str = "fn version() -> str '1.4'\nfn added() -> bool true\n";
        const LIB14B: &str = "fn version() -> str '1.4 again'\n";
        const LIB20: &str = "fn version() -> str '2.0'\n";
        const LIB03: &str = "fn version() -> str '0.3'\n";
        const LIB04: &str = "fn version() -> str '0.4'\n";

        /// Versions of one package ("lib") to import: /lib12.stof, /lib14.stof, ... (and slow ones under /slow/).
        fn versioned() -> Server {
            let mut routes = Vec::new();
            for (name, src) in LIBS {
                routes.push((format!("/{name}.stof"), 200, "text/plain".to_string(), src.as_bytes().to_vec()));
                routes.push((format!("/slow/{name}.stof"), 200, "text/plain".to_string(), src.as_bytes().to_vec()));
            }
            Server::start_with(move |_| routes)
        }

        const LIBS: [(&str, &str); 6] = [("lib12", LIB12), ("lib14", LIB14), ("lib14b", LIB14B), ("lib20", LIB20), ("lib03", LIB03), ("lib04", LIB04)];

        fn lib_hash(name: &str) -> String {
            sha(LIBS.iter().find(|(lib, _)| *lib == name).unwrap().1.as_bytes())
        }

        /// A document with helpers for importing the versions: get('lib12', sha, '@lib@1.2.0') imports /lib12.stof
        /// there ({lib12} in the source is its hash).
        fn versions_doc(server: &Server, src: &str) -> Graph {
            let mut src = src.to_string();
            for (name, _) in LIBS { src = src.replace(&format!("{{{name}}}"), &lib_hash(name)); }
            doc(server, &format!(r#"
                async fn get(name: str, sha: str, into: str) -> bool await import(`{{base}}/${{name}}.stof`, sha, into)
                async fn slow(name: str, sha: str, into: str) -> bool await import(`{{base}}/slow/${{name}}.stof`, sha, into)
                fn attempt(name: str, sha: str, into: str) -> str {{
                    try {{ await self.get(name, sha, into); return 'imported'; }}
                    catch (e, msg) {{ return msg; }}
                }}
                fn which() -> str root.version()
                fn versions() -> str imported().map((dep: map): str => dep.get('version') ?? 'none').join(',')
                {src}
            "#))
        }

        fn get(graph: &mut Graph, name: &str, into: &str) -> Val {
            match graph.call("get", None, vec![Val::from(name), Val::from(lib_hash(name).as_str()), Val::from(into)]) {
                Ok(val) => val,
                Err(error) => panic!("get {name} {into} failed: {error}"),
            }
        }

        fn attempt(graph: &mut Graph, name: &str, into: &str) -> String {
            graph.call("attempt", None, vec![Val::from(name), Val::from(lib_hash(name).as_str()), Val::from(into)]).unwrap().to_string()
        }

        const PNG: [u8; 75] = [137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 2, 0, 0, 0, 3, 8, 2, 0, 0, 0, 54, 136, 73, 214, 0, 0, 0, 18, 73, 68, 65, 84, 120, 156, 99, 248, 207, 192, 240, 159, 129, 1, 76, 192, 105, 0, 62, 214, 5, 251, 118, 213, 141, 129, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130];

        #[test]
        /// Any file imports, as an `import` statement would take it: a format by its extension (data, text, images,
        /// documents), else by its content type, else text or bytes (another extension, or binary), else stof (no
        /// extension); into a path, or as a package (written to stof/ with its extension, Ex. stof/<dir>/rates.csv).
        fn any_format() {
            let graph_bytes = {
                let mut source = Graph::default();
                source.parse_stof_src("fn hi() -> str 'hi from bstf'\nn: 7", None, Profile::default()).unwrap();
                source.binary_export("bstf", None).unwrap().to_vec()
            };
            let files: Vec<(&str, &str, Vec<u8>)> = vec![
                ("/data.json", "application/octet-stream", br#"{"n": 1}"#.to_vec()),
                ("/data.toml", "text/plain", b"n = 2\n".to_vec()),
                ("/data.yaml", "text/plain", b"n: 3\n".to_vec()),
                ("/notes.txt", "text/plain", b"plain notes".to_vec()),
                ("/notes.md", "text/plain", b"# Notes".to_vec()),
                ("/page.html", "text/html", b"<p>hi</p>".to_vec()),
                ("/rates.csv", "text/csv", b"a,b\n1,2\n".to_vec()),
                ("/blob.dat", "application/octet-stream", vec![0, 159, 146, 150, 255]),
                ("/pixel.png", "application/octet-stream", PNG.to_vec()),
                ("/graph.bstf", "application/octet-stream", graph_bytes),
                ("/typed", "application/json", br#"{"n": 4}"#.to_vec()),
                ("/csv-typed", "text/csv", b"x,y\n".to_vec()),
                ("/binary", "application/octet-stream", vec![0, 255, 254]),
                ("/script", "text/plain", b"fn s() -> str 'stof'\n".to_vec()),
            ];
            let mut routes = Vec::new();
            let mut lines = String::new();
            for (i, (path, ctype, body)) in files.iter().enumerate() {
                routes.push((path.to_string(), 200, ctype.to_string(), body.clone()));
                lines.push_str(&format!("fn f{i}() -> bool await import('{{base}}{path}', '{}', 'root.f{i}')\n", sha(body)));
            }
            let server = Server::start_with(move |_| routes);
            let mut graph = doc(&server, &format!(r#"
                {lines}
                fn check() -> str [
                    root.f0.n, root.f1.n, root.f2.n, root.f3.text, root.f4.md, typeof root.f5, root.f6.text, root.f7.bytes.len(),
                    Image.width(root.f8.image), root.f9.hi(), root.f10.n, root.f11.text, root.f12.bytes.len(), root.f13.s()
                ].join('|')
            "#));
            for i in 0..files.len() {
                assert_eq!(call(&mut graph, &format!("f{i}")), Val::from(true), "{}", files[i].0);
            }
            let check = call(&mut graph, "check").to_string();
            assert_eq!(check, "1|2|3|plain notes|# Notes|obj|a,b\n1,2\n|5|2|hi from bstf|4|x,y\n|3|stof");

            // as packages, with the file system: written with their extensions, and found again without a fetch
            let dir = PackageDir::new();
            let mut files_graph = with_files(&server, &format!(r#"
                fn csv() -> bool await import('{{base}}/rates.csv', '{}', '@{{dir}}/rates')
                fn png() -> bool await import('{{base}}/pixel.png', '{}', '@{{dir}}/pixel')
            "#, sha(b"a,b\n1,2\n"), sha(&PNG)), &dir);
            assert_eq!(call(&mut files_graph, "csv"), Val::from(true));
            assert_eq!(call(&mut files_graph, "png"), Val::from(true));
            assert_eq!(std::fs::read(dir.file("rates.csv")).unwrap(), b"a,b\n1,2\n");
            assert_eq!(std::fs::read(dir.file("pixel.png")).unwrap(), PNG);
            let hits = server.hits("/rates.csv");
            let mut again = with_files(&server, &format!(r#"
                fn csv() -> str {{ await import('{{base}}/rates.csv', '{}', '@{{dir}}/rates'); root.text }}
            "#, sha(b"a,b\n1,2\n")), &dir);
            assert_eq!(call(&mut again, "csv"), Val::from("a,b\n1,2\n"));
            assert_eq!(server.hits("/rates.csv"), hits, "read from stof/");
        }

        #[test]
        /// The highest compatible version wins, in either order: a higher one is parsed over a lower one, and a lower one
        /// is skipped. The root records one version.
        fn versions_highest_wins() {
            let server = versioned();
            let mut graph = versions_doc(&server, "");
            assert_eq!(get(&mut graph, "lib12", "@lib@1.2.0"), Val::from(true));
            assert_eq!(call(&mut graph, "which"), Val::from("1.2"));
            assert_eq!(get(&mut graph, "lib14", "@lib@1.4.0"), Val::from(true), "parsed over 1.2");
            assert_eq!(call(&mut graph, "which"), Val::from("1.4"));
            assert_eq!(call(&mut graph, "versions"), Val::from("1.4.0"), "one record");
            assert_eq!(get(&mut graph, "lib12", "@lib@1.2.0"), Val::from(false), "1.4 stands in for 1.2");
            assert_eq!(server.hits("/lib12.stof"), 1, "a lower version isn't fetched");

            let mut reverse = versions_doc(&server, "");
            assert_eq!(get(&mut reverse, "lib14", "@lib@1.4.0"), Val::from(true));
            assert_eq!(get(&mut reverse, "lib12", "@lib@1.2.0"), Val::from(false));
            assert_eq!(call(&mut reverse, "which"), Val::from("1.4"));
            assert_eq!(server.hits("/lib12.stof"), 1);

            // 0.x: the minor version is the major one
            let mut zero = versions_doc(&server, "");
            assert_eq!(get(&mut zero, "lib03", "@lib@0.3.0"), Val::from(true));
            assert_eq!(get(&mut zero, "lib03", "@lib@0.3.1"), Val::from(false), "the same bytes");
            let msg = attempt(&mut zero, "lib04", "@lib@0.4.0");
            assert!(msg.contains("aren't compatible") && msg.contains("0.4.x") && msg.contains("0.3.x"), "{msg}");
        }

        #[test]
        /// Imports that run at the same time end with the highest version too: one started while a lower one fetches
        /// takes its place (the lower one does nothing), and a lower one started while a higher one fetches is skipped.
        fn versions_concurrent() {
            let server = versioned();
            let mut graph = versions_doc(&server, r#"
                fn race() -> str {
                    const low = self.slow('lib12', '{lib12}', '@lib@1.2.0');
                    const high = self.get('lib14', '{lib14}', '@lib@1.4.0');
                    `${await low} ${await high}`
                }
            "#);
            assert_eq!(call(&mut graph, "race"), Val::from("false true"), "1.2 was replaced while it fetched");
            assert_eq!(call(&mut graph, "which"), Val::from("1.4"));
            assert_eq!(call(&mut graph, "versions"), Val::from("1.4.0"));

            let mut other = versions_doc(&server, r#"
                fn race() -> str {
                    const high = self.slow('lib14', '{lib14}', '@lib@1.4.0');
                    const low = self.get('lib12', '{lib12}', '@lib@1.2.0');
                    `${await high} ${await low}`
                }
            "#);
            assert_eq!(call(&mut other, "race"), Val::from("true false"), "1.4 was loading");
            assert_eq!(call(&mut other, "which"), Val::from("1.4"));
            assert_eq!(server.hits("/lib12.stof"), 0, "the lower one isn't fetched");
        }

        #[test]
        /// Versions that can't share a document are errors, in either order: incompatible ones, and one version with
        /// different bytes. Nothing changes.
        fn versions_conflicts() {
            let server = versioned();
            let mut graph = versions_doc(&server, "");
            get(&mut graph, "lib14", "@lib@1.4.0");
            let msg = attempt(&mut graph, "lib20", "@lib@2.0.0");
            assert!(msg.contains("@lib@2.0.0") && msg.contains("conflicts with @lib@1.4.0 (loaded") && msg.contains("2.x and 1.x aren't compatible"), "{msg}");
            let msg = attempt(&mut graph, "lib14b", "@lib@1.4.0");
            assert!(msg.contains("different bytes than the same version loaded"), "{msg}");
            assert_eq!(call(&mut graph, "which"), Val::from("1.4"));

            let mut reverse = versions_doc(&server, "");
            get(&mut reverse, "lib20", "@lib@2.0.0");
            let msg = attempt(&mut reverse, "lib14", "@lib@1.4.0");
            assert!(msg.contains("1.x and 2.x aren't compatible"), "{msg}");

            // while it's loading
            let mut loading = versions_doc(&server, r#"
                fn race() -> str {
                    const first = self.slow('lib14', '{lib14}', '@lib@1.4.0');
                    const second = self.attempt('lib20', '{lib20}', '@lib@2.0.0');
                    await first;
                    second
                }
            "#);
            let msg = call(&mut loading, "race").to_string();
            assert!(msg.contains("(loading, from"), "{msg}");

            let msg = attempt(&mut graph, "lib14", "@lib@1.4");
            assert!(msg.contains("isn't a version"), "{msg}");
        }

        #[test]
        /// Without a version, a package uses whatever's there; a version replaces one imported without a version. The
        /// same bytes with a version are the same package (it learns the version).
        fn versions_and_none() {
            let server = versioned();
            let mut graph = versions_doc(&server, "");
            assert_eq!(get(&mut graph, "lib12", "@lib"), Val::from(true));
            assert_eq!(get(&mut graph, "lib14", "@lib"), Val::from(false), "whatever's there");
            assert_eq!(call(&mut graph, "which"), Val::from("1.2"));
            assert_eq!(get(&mut graph, "lib14", "@lib@1.4.0"), Val::from(true), "a version replaces it");
            assert_eq!(call(&mut graph, "which"), Val::from("1.4"));
            assert_eq!(get(&mut graph, "lib12", "@lib"), Val::from(false));

            let mut learns = versions_doc(&server, "");
            assert_eq!(get(&mut learns, "lib12", "@lib"), Val::from(true));
            assert_eq!(call(&mut learns, "versions"), Val::from("none"));
            assert_eq!(get(&mut learns, "lib12", "@lib@1.2.0"), Val::from(false), "the same bytes");
            assert_eq!(call(&mut learns, "versions"), Val::from("1.2.0"), "it learned the version");
            assert_eq!(get(&mut learns, "lib12", "@lib@1.1.0"), Val::from(false), "1.2 stands in for 1.1");
            let msg = attempt(&mut learns, "lib20", "@lib@2.0.0");
            assert!(msg.contains("aren't compatible"), "{msg}");
        }

        #[test]
        /// A failed import of a version leaves what was there: the version loaded stands, and nothing is left loading.
        fn versions_failures() {
            let server = versioned();
            let mut graph = versions_doc(&server, r#"
                fn missing() -> str {
                    try { await import('{base}/nope.stof', '{lib14}', '@lib@1.4.0'); return 'imported'; }
                    catch (e, msg) { return msg; }
                }
            "#);
            let msg = call(&mut graph, "missing").to_string();
            assert!(msg.contains("404"), "{msg}");
            assert_eq!(get(&mut graph, "lib12", "@lib@1.2.0"), Val::from(true), "nothing left loading");
            let msg = call(&mut graph, "missing").to_string();
            assert!(msg.contains("404"), "{msg}");
            assert_eq!(call(&mut graph, "which"), Val::from("1.2"));
            assert_eq!(get(&mut graph, "lib12", "@lib@1.0.0"), Val::from(false), "1.2 still stands");
            assert_eq!(get(&mut graph, "lib14", "@lib@1.4.0"), Val::from(true));
        }

        #[test]
        /// A call that's dropped while it imports or runs #[load] functions (Ex. a JavaScript host that stops stepping
        /// it), so they never finish or fail, leaves nothing behind: the next call imports the package (it isn't
        /// "loading" forever), and the #[load] function runs again. (Errors and time limits fail them on the way out.)
        fn abandoned_loads() {
            use std::sync::Arc;
            use crate::runtime::{Runtime, proc::Process, instruction::Instruction, instructions::call::FuncCall};
            let server = versioned();
            let mut graph = versions_doc(&server, "#[load] fn lib() { await import('{base}/slow/lib14.stof', '{lib14}', '@lib@1.4.0'); }");
            let start = |search: &str| -> Arc<dyn Instruction> { Arc::new(FuncCall { as_ref: false, cnull: false, stack: false, func: None, search: Some(search.into()), args: Default::default(), oself: None }) };
            {
                let mut runtime = Runtime::default();
                runtime.push_running_proc(Process::from(start("Std::load")), &mut graph);
                for _ in 0..10_000 {
                    if graph.packages.get("lib").is_some() { break; }
                    runtime.run_single_step(&mut graph);
                }
                assert!(graph.packages.get("lib").is_some_and(|state| !state.loaded), "loading when it's dropped");
                assert_eq!(graph.loading.len(), 1);
            } // dropped: its processes never finish
            assert_eq!(graph.load(None).unwrap(), Val::from(1), "the #[load] function runs again");
            assert_eq!(call(&mut graph, "which"), Val::from("1.4"), "and the package imports");

            // a time limit fails them, which cleans up as it goes
            let mut timed = versions_doc(&server, "#[load] fn lib() { await import('{base}/slow/lib12.stof', '{lib12}', '@lib@1.2.0'); }");
            timed.max_execution_time = Some(web_time::Duration::from_millis(100));
            assert!(timed.load(None).is_err(), "timed out");
            timed.max_execution_time = None;
            assert_eq!(timed.load(None).unwrap(), Val::from(1));
            assert_eq!(call(&mut timed, "which"), Val::from("1.2"));
        }

        #[test]
        /// A saved document remembers its packages' versions: its root's records.
        fn versions_saved() {
            let server = versioned();
            let mut graph = versions_doc(&server, "");
            get(&mut graph, "lib14", "@lib@1.4.0");
            let bytes = graph.binary_export("bstf", None).unwrap();

            let mut loaded = Graph::default();
            loaded.quiet(true);
            loaded.allow_http_hosts([server.host()]);
            loaded.binary_import("bstf", bytes, None, &Profile::default()).unwrap();
            assert!(loaded.packages.is_empty());
            assert_eq!(get(&mut loaded, "lib12", "@lib@1.2.0"), Val::from(false), "the record says 1.4");
            let msg = attempt(&mut loaded, "lib20", "@lib@2.0.0");
            assert!(msg.contains("conflicts with @lib@1.4.0 (loaded"), "{msg}");
            assert_eq!(server.hits("/lib12.stof"), 0);
        }

        #[test]
        /// Packages that need different versions of a package they share (through their #[load] functions) end with the
        /// highest one, fetched once each.
        fn versions_shared_dependency() {
            let server = Server::start_with(|base| {
                let a = format!("fn a() -> str 'a'\n#[load] fn lib() {{ await import('{base}/lib12.stof', '{}', '@lib@1.2.0'); }}\n", sha(LIB12.as_bytes()));
                let b = format!("fn b() -> str 'b'\n#[load] fn lib() {{ await import('{base}/lib14.stof', '{}', '@lib@1.4.0'); }}\n", sha(LIB14.as_bytes()));
                vec![
                    ("/a.stof".to_string(), 200, "text/plain".to_string(), a.into_bytes()),
                    ("/b.stof".to_string(), 200, "text/plain".to_string(), b.into_bytes()),
                    ("/lib12.stof".to_string(), 200, "text/plain".to_string(), LIB12.as_bytes().to_vec()),
                    ("/lib14.stof".to_string(), 200, "text/plain".to_string(), LIB14.as_bytes().to_vec()),
                ]
            });
            let a = format!("fn a() -> str 'a'\n#[load] fn lib() {{ await import('{}/lib12.stof', '{}', '@lib@1.2.0'); }}\n", server.base, sha(LIB12.as_bytes()));
            let b = format!("fn b() -> str 'b'\n#[load] fn lib() {{ await import('{}/lib14.stof', '{}', '@lib@1.4.0'); }}\n", server.base, sha(LIB14.as_bytes()));
            let mut graph = doc(&server, &format!(r#"
                #[load] fn a() {{ await import('{{base}}/a.stof', '{}', 'root.deps.a'); }}
                #[load] fn b() {{ await import('{{base}}/b.stof', '{}', 'root.deps.b'); }}
                fn which() -> str root.version()
            "#, sha(a.as_bytes()), sha(b.as_bytes())));
            graph.load(None).unwrap();
            assert_eq!(call(&mut graph, "which"), Val::from("1.4"));
            assert!(server.hits("/lib14.stof") == 1 && server.hits("/lib12.stof") <= 1);
        }

        #[test]
        /// Installing (vendor only) writes the highest version's file.
        fn versions_install() {
            let server = versioned();
            let dir = PackageDir::new();
            let mut graph = with_files(&server, &format!(r#"
                #[load] fn high() {{ await import('{{base}}/lib14.stof', '{}', '@{{dir}}/lib@1.4.0'); }}
                #[load] fn low() {{ await import('{{base}}/slow/lib12.stof', '{}', '@{{dir}}/lib@1.2.0'); }}
                fn parsed() -> bool root?.version != null
            "#, sha(LIB14.as_bytes()), sha(LIB12.as_bytes())), &dir);
            graph.import_vendor_only = true;
            graph.load(None).unwrap();
            assert_eq!(std::fs::read_to_string(dir.file("lib.stof")).unwrap(), LIB14);
            assert_eq!(call(&mut graph, "parsed"), Val::from(false), "nothing parsed");
        }

        /*****************************************************************************
         * Headers.
         *****************************************************************************/

        #[test]
        /// Headers go with the request; they must be a map.
        fn headers() {
            let server = served();
            let mut graph = doc(&server, r#"
                fn get() -> str {
                    await import('{base}/pkg.stof', '{pkg}', 'root.dep', null, 'prod', { 'X-Token': 'secret', 'Accept': 'text/plain' });
                    root.dep.hello()
                }
                fn bad() -> str { try { await import('{base}/pkg.stof', '{pkg}', 'root.bad', null, 'prod', 'x'); return 'imported'; } catch (e, msg) { return msg; } }
            "#);
            assert_eq!(call(&mut graph, "get"), Val::from("hello"));
            let head = server.head("/pkg.stof");
            assert!(head.contains("x-token: secret") && head.contains("accept: text/plain"), "{head}");
            let msg = call(&mut graph, "bad").to_string();
            assert!(msg.contains("headers must be a map"), "{msg}");
        }

        /*****************************************************************************
         * Packages (.pkg) and package files.
         *****************************************************************************/

        /// A .pkg file's bytes: (name, contents) files zipped.
        fn zip_files(files: &[(&str, String)]) -> Vec<u8> {
            let mut buf = std::io::Cursor::new(Vec::new());
            {
                let mut writer = zip::ZipWriter::new(&mut buf);
                for (name, contents) in files {
                    writer.start_file(*name, zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Bzip2)).unwrap();
                    writer.write_all(contents.as_bytes()).unwrap();
                }
                writer.finish().unwrap();
            }
            buf.into_inner()
        }

        #[test]
        /// A .pkg imported without a file system: read in memory, its files importing each other, and its pkg.stof's
        /// #[load] functions coming with it (so its own dependency loads too).
        fn pkg_in_memory() {
            let made: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
            let keep = made.clone();
            let server = Server::start_with(move |base| {
                let pkg = zip_files(&[
                    ("pkg.stof", format!("name: 'tools'\nversion: '1.0.0'\nimport: ['main.stof', {{ path: 'data/info.json', as: 'self.info' }}]\n#[load] fn dep() {{ await import('{base}/pkg.stof', '{}', '@dep'); }}\n", sha(PKG.as_bytes()))),
                    ("main.stof", "import './lib/util.stof'\nfn tool() -> str 'tool ' + self.util()\n".to_string()),
                    ("lib/util.stof", "fn util() -> str 'util'\n".to_string()),
                    ("data/info.json", r#"{"count": 3}"#.to_string()),
                ]);
                *keep.lock().unwrap() = pkg.clone();
                vec![
                    ("/tools.pkg".to_string(), 200, "application/octet-stream".to_string(), pkg),
                    ("/pkg.stof".to_string(), 200, "text/plain".to_string(), PKG.as_bytes().to_vec()),
                ]
            });
            let pkg = made.lock().unwrap().clone();
            let mut graph = doc(&server, &format!(r#"
                fn get() -> bool await import('{{base}}/tools.pkg', '{}', '@formata/tools@1.0.0')
                fn check() -> str `${{root.tool()}} ${{root.info.count}} ${{root.hello()}} ${{root.name ?? 'no fields'}}`
            "#, sha(&pkg)));
            assert_eq!(call(&mut graph, "get"), Val::from(true));
            assert_eq!(call(&mut graph, "check"), Val::from("tool util 3 hello no fields"), "pkg.stof's own fields stay out");
            assert_eq!(server.hits("/pkg.stof"), 1, "its dependency loaded");
            assert_eq!(graph.load(None).unwrap(), Val::from(1), "its #[load] function is in the document");
            assert_eq!(server.hits("/pkg.stof"), 1);
        }

        #[test]
        /// A package's imports stay in the package: a missing file, or a path outside it, is an error.
        fn pkg_stays_in_the_package() {
            let outside = zip_files(&[("pkg.stof", "import: 'main.stof'\n".to_string()), ("main.stof", "import '../secret.stof'\n".to_string())]);
            let rooted = zip_files(&[("pkg.stof", "import: 'main.stof'\n".to_string()), ("main.stof", "import '/etc/hosts.stof'\n".to_string())]);
            let missing = zip_files(&[("pkg.stof", "import: 'main.stof'\n".to_string())]);
            let no_pkg = zip_files(&[("main.stof", "x: 1\n".to_string())]);
            let not_zip = b"not a zip".to_vec();
            let server = Server::start(vec![
                ("/outside.pkg", 200, "application/octet-stream", outside.clone()),
                ("/rooted.pkg", 200, "application/octet-stream", rooted.clone()),
                ("/missing.pkg", 200, "application/octet-stream", missing.clone()),
                ("/no_pkg.pkg", 200, "application/octet-stream", no_pkg.clone()),
                ("/not_zip.pkg", 200, "application/octet-stream", not_zip.clone()),
            ]);
            let mut graph = Graph::default();
            graph.quiet(true);
            graph.allow_system(); // even with the file system
            graph.allow_http_hosts([server.host()]);
            graph.parse_stof_src(&format!(r#"
                fn attempt(name: str, sha: str) -> str {{
                    try {{ await import(`{}/${{name}}.pkg`, sha, 'root.p'); return 'imported'; }}
                    catch (e, msg) {{ return msg; }}
                }}
            "#, server.base), None, Profile::default()).unwrap();
            let mut attempt = |name: &str, bytes: &[u8]| graph.call("attempt", None, vec![Val::from(name), Val::from(sha(bytes).as_str())]).unwrap().to_string();
            let msg = attempt("outside", &outside);
            assert!(msg.contains("'../secret.stof' is outside the package"), "{msg}");
            let msg = attempt("rooted", &rooted);
            assert!(msg.contains("outside the package"), "{msg}");
            let msg = attempt("missing", &missing);
            assert!(msg.contains("main.stof' isn't in the package"), "{msg}");
            let msg = attempt("no_pkg", &no_pkg);
            assert!(msg.contains("needs a pkg.stof"), "{msg}");
            let msg = attempt("not_zip", &not_zip);
            assert!(msg.contains("not a package"), "{msg}");
        }

        #[test]
        /// A package in a package: its files come from the inner package, and its #[load] functions come along too.
        /// An untrusted .pkg can't bring #[load] functions (like any untrusted source).
        fn pkg_nested_and_untrusted() {
            let inner = zip_files(&[
                ("pkg.stof", "import: 'inner.stof'\n#[load] fn inner_dep() { self.inner_loaded = true; }\n".to_string()),
                ("inner.stof", "fn inner() -> str 'inner'\n".to_string()),
            ]);
            let mut outer_buf = std::io::Cursor::new(Vec::new());
            {
                let mut writer = zip::ZipWriter::new(&mut outer_buf);
                writer.start_file("pkg.stof", zip::write::SimpleFileOptions::default()).unwrap();
                writer.write_all(b"import: ['outer.stof', 'deps/inner.pkg']\n").unwrap();
                writer.start_file("outer.stof", zip::write::SimpleFileOptions::default()).unwrap();
                writer.write_all(b"fn outer() -> str 'outer'\n").unwrap();
                writer.start_file("deps/inner.pkg", zip::write::SimpleFileOptions::default()).unwrap();
                writer.write_all(&inner).unwrap();
                writer.finish().unwrap();
            }
            let outer = outer_buf.into_inner();
            let server = Server::start(vec![("/outer.pkg", 200, "application/octet-stream", outer.clone())]);
            let mut graph = doc(&server, &format!(r#"
                fn get() -> str {{ await import('{{base}}/outer.pkg', '{0}', 'root.p'); `${{root.p.outer()}} ${{root.p.inner()}} ${{root.p.inner_loaded}}` }}
                fn untrusted() -> str {{
                    try {{ await import('{{base}}/outer.pkg', '{0}', 'root.u', profile = 'untrusted'); return 'imported'; }}
                    catch (e, msg) {{ return msg; }}
                }}
            "#, sha(&outer)));
            assert_eq!(call(&mut graph, "get"), Val::from("outer inner true"));
            let msg = call(&mut graph, "untrusted").to_string();
            assert!(msg.contains("untrusted source can't have #[load] functions"), "{msg}");
        }

        #[test]
        /// A package's files can't unzip to more than the document's memory limit (Ex. a zip bomb).
        fn pkg_size_limit() {
            let big = zip_files(&[("pkg.stof", "import: 'big.stof'\n".to_string()), ("big.stof", format!("// {}\n", "0".repeat(200_000)))]);
            assert!(big.len() < 10_000, "it compresses");
            let server = Server::start(vec![("/big.pkg", 200, "application/octet-stream", big.clone())]);
            let mut graph = doc(&server, &format!(r#"
                fn attempt() -> str {{ try {{ await import('{{base}}/big.pkg', '{}', 'root.p'); return 'imported'; }} catch (e, msg) {{ return msg; }} }}
            "#, sha(&big)));
            graph.max_memory = Some(100_000);
            let msg = call(&mut graph, "attempt").to_string();
            assert!(msg.contains("unzip to more than 100000 bytes"), "{msg}");
        }

        #[test]
        /// A package your code imports from the package directory (`import '@name'`) is recorded, so a #[load] import of
        /// the package uses it, whatever its version (nothing fetched or parsed again). A package written as a .pkg is
        /// found without its extension.
        fn package_files_are_used() {
            let server = versioned();
            let dir = PackageDir::new();
            std::fs::create_dir_all(dir.file("")).unwrap();
            std::fs::write(dir.file("lib.stof"), LIB12).unwrap();
            let pkg = zip_files(&[("pkg.stof", "import: 'main.stof'\n".to_string()), ("main.stof", "fn tool() -> str 'tool'\n".to_string())]);
            std::fs::write(dir.file("tools.pkg"), &pkg).unwrap();
            std::fs::write(dir.file("page.html"), "<p>hi</p>").unwrap();
            std::fs::write(dir.file("rates.csv"), "a,b").unwrap();
            std::fs::write(dir.file("blob.dat"), [0u8, 255]).unwrap();

            let mut graph = Graph::default();
            graph.quiet(true);
            graph.allow_system();
            graph.allow_http_hosts([server.host()]);
            parse_into(&mut graph, &server, &format!(r#"
                import '@{{dir}}/lib'
                import '@{{dir}}/tools'
                import '@{{dir}}/page.html' as root.files.page
                import '@{{dir}}/rates.csv' as root.files.rates
                import '@{{dir}}/blob.dat' as root.files.blob
                fn files() -> str `${{root.files.page.text}} ${{root.files.rates.text}} ${{root.files.blob.bytes.len()}}`
                fn get() -> bool await import('{{base}}/lib14.stof', '{}', '@{{dir}}/lib@1.4.0')
                fn check() -> str `${{root.version()}} ${{root.tool()}}`
                fn packages() -> str imported().map((dep: map): str => dep.get('package')).join(',')
            "#, sha(LIB14.as_bytes())), &dir);
            assert_eq!(call(&mut graph, "get"), Val::from(false), "the file your code imported is used");
            assert_eq!(call(&mut graph, "check"), Val::from("1.2 tool"));
            assert_eq!(call(&mut graph, "files"), Val::from("<p>hi</p> a,b 2"), "any file: an export-only format's as text, another extension as text or bytes");
            assert_eq!(server.hits("/lib14.stof"), 0);
            let mut packages: Vec<String> = call(&mut graph, "packages").to_string().split(',').map(|name| name.to_string()).collect();
            packages.sort();
            let name = &dir.name;
            assert_eq!(packages, vec![format!("@{name}/blob"), format!("@{name}/lib"), format!("@{name}/page"), format!("@{name}/rates"), format!("@{name}/tools")], "named without their extensions");
        }
    }
}
