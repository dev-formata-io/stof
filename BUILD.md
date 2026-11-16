# Building and publishing Stof

Stof is one Rust crate with a host binding per language. Each binding is a Cargo feature, and every package is built
from this repo:

| Package | Registry | Feature | Built with | Source |
| --- | --- | --- | --- | --- |
| `stof` | crates.io | `basic` (default), `full` | `cargo` | `src/` |
| `@formata/stof` | npm | `js` | `wasm-pack` + `tsc` | `src/js/`, `web/` |
| `stof` (import `pystof`) | PyPI | `py` | `maturin` | `src/py/` |

`project.stof` is the source of truth for versions, dependencies, and CI. It writes `Cargo.toml`, `pyproject.toml`,
and `.github/`, so change those in `project.stof` and regenerate, never by hand:

```sh
stof run project.stof
```

A new language binding (Ex. Go) gets a row in the table above, a feature in `project.stof`, a section below, and
jobs in the CI and release workflows (`github.workflows` in `project.stof`).

## Release checklist

1. Set `version` in `project.stof`, then `stof run project.stof`.
2. Set the same version in `web/package.json` (and `web/package-lock.json`), `web/doc.ts` (`StofDoc.VERSION`), and
   `plugin/.claude-plugin/plugin.json`. The release workflow checks they all match.
3. Move the CHANGELOG's `Unreleased` section under the new version.
4. Update the Stof skill (`content/SKILL.md`). The npm package ships it, so this comes before publishing.
5. Commit and push, then run the release workflow (below). To publish by hand instead, test everything and publish
   in this order: crates.io, npm, PyPI.
6. Update stof-cli and stof.dev to the new version. stof-cli releases with its own workflow, which builds the
   `stof` binaries and installers (see "Building and releasing" in its README).

## Release workflow

`.github/workflows/release.yml` (written by `project.stof`, like the CI workflow) publishes a release in one go. It
runs only when started by hand: Actions > Release > Run workflow, on `main`.

1. **Check and build everything.** The versions must match (`Cargo.toml`, `pyproject.toml`, `web/package.json`,
   `web/doc.ts`, the plugin, and a CHANGELOG section), and then it runs `cargo test --features full`, builds and
   checks the npm package, and builds and smoke-tests the Python wheels (Windows x64, Linux x64 and arm64, macOS
   universal2) and the source package.
2. **Publish, only if all of that passed:** crates.io, then npm, then PyPI. A version that's already out is skipped,
   so a release that failed partway can be run again.

The publishing jobs run in the `release` environment and use **trusted publishing**: each registry trusts this repo's
`release.yml` in that environment and hands it a short-lived token when it publishes, so there are no tokens to
store or rotate. Set it up once:

1. **GitHub** (Settings > Environments > `release`):
   - Required reviewers: yourself, so every run waits for your approval before it publishes anything. Leave
     "Prevent self-review" off, or you can't approve your own runs.
   - Deployment branches and tags: "Selected branches and tags", with `main`.
   - No secrets.
2. **crates.io** (crates.io/crates/stof > Settings > Trusted Publishing > Add): GitHub, owner `dev-formata-io`,
   repository `stof`, workflow `release.yml`, environment `release`.
3. **npm** (npmjs.com/package/@formata/stof > Settings > Trusted Publisher > GitHub Actions): organization or user
   `dev-formata-io`, repository `stof`, workflow `release.yml`, environment `release`. Once a release has gone out
   this way, set Publishing access to "Require two-factor authentication and disallow tokens".
4. **PyPI** (pypi.org/manage/project/stof/settings/publishing > Add a new publisher > GitHub): owner
   `dev-formata-io`, repository `stof`, workflow `release.yml`, environment `release`.

The names must match exactly. A mismatch fails at publish time (npm reports it as a misleading 404 or "access token
expired"), after everything has built.

## Tests

```sh
cargo test                     # basic features
cargo test --features full     # every library
cargo check --features js      # the JavaScript binding compiles
```

The JavaScript and Python checks run against a built package: `node tests/js/check.mjs` after the npm build, and
`python tests/python/smoke.py` after installing a wheel.

## Rust (crates.io)

```sh
cargo publish --dry-run
cargo publish
```

## JavaScript (npm)

wasm-pack builds the engine with only the `js` feature and optimizes it with wasm-opt. Either pass
`--no-default-features --features js` (as below, and as the CI and release workflows do), or run
`stof run project.stof -a javascript` first, which makes `js` the default, and `stof run project.stof` after to put
`Cargo.toml` back. It fetches the wasm-bindgen CLI that matches the `wasm-bindgen` crate itself. `web/pkg` is
committed, so commit it after building.

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack --locked

wasm-pack build --target web --release --out-dir web/pkg -- --no-default-features --features js

cd web && npm ci && npm run build && cd ..
node tests/js/check.mjs

cd web
npm publish                    # prepublishOnly copies the skill, README, and LICENSE, then builds
```

## Python (PyPI)

`pyproject.toml` names the module (`pystof`) and turns on the `py` feature, so plain `maturin` commands work without
touching `Cargo.toml`. Install the tools once:

```sh
pip install "maturin[zig]" twine
rustup target add x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu
```

### What to build

pip installs a wheel only when its tags match the user's machine. Anyone without a matching wheel builds Stof from the
source package, which takes minutes and needs a Rust toolchain.

Stof's wheels use Python's stable ABI (pyo3's `abi3-py38`), so **one wheel per OS and CPU covers every CPython from
3.8 on**, including versions released later. And Stof links no system libraries (its HTTP uses rustls, with the OS's
root certificates), so Linux wheels cross-compile from any machine with zig. The wheels turn on the `mimalloc`
feature, which compiles a little C: zig does that for Linux, and the usual C compiler for Windows (Visual Studio's
build tools) and macOS (the Xcode command line tools). A release is five files:

| Wheel | Rust target | Built on |
| --- | --- | --- |
| Windows x64 | `x86_64-pc-windows-msvc` | Windows |
| Linux x64 | `x86_64-unknown-linux-gnu` | anywhere (zig) |
| Linux arm64 | `aarch64-unknown-linux-gnu` | anywhere (zig) |
| macOS (Intel + Apple silicon) | `universal2-apple-darwin` | a Mac only (or the release workflow) |
| source package | | anywhere |

The stable ABI doesn't cover PyPy or free-threaded CPython (Ex. `3.14t`): those install from the source package.

```sh
maturin sdist                                                  # source package
maturin build --release                                        # this machine (Ex. Windows x64)
maturin build --release --zig --target x86_64-unknown-linux-gnu --compatibility manylinux2014
maturin build --release --zig --target aarch64-unknown-linux-gnu --compatibility manylinux2014
```

On a Mac:

```sh
rustup target add x86_64-apple-darwin aarch64-apple-darwin
maturin build --release --target universal2-apple-darwin
```

`--compatibility manylinux2014` builds against glibc 2.17, so the wheel installs on any Linux from 2014 on. Without
`--zig`, a wheel built on Linux is tagged for that machine's glibc (Ex. `manylinux_2_38`), and pip on an older distro
(Ex. Ubuntu 22.04) won't install it.

The file names show what each wheel covers: `stof-<version>-cp38-abi3-<platform>.whl` is CPython 3.8 and later. A
wheel tagged with one Python version (Ex. `cp314-cp314`) means the stable ABI wasn't on.

### Check and upload

Test a wheel before uploading it (any Python 3.8+ on that platform):

```sh
pip install --force-reinstall target/wheels/<wheel>.whl
python tests/python/smoke.py
```

Then upload everything at once:

```sh
twine check target/wheels/*
twine upload target/wheels/*
```

PyPI won't take a file name twice. To add a missing wheel to a release that's already out, build and upload just that
wheel (Ex. the macOS one, from the release workflow's `python-macos-latest-universal2-apple-darwin` artifact).
