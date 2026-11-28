# Semantic Version Library (Ver)
Library for working with semantic versioning. Versions are a base type in Stof (ver), and they follow semver.org:

- **Order:** comparisons use semver precedence: major, minor, and patch as numbers, then a pre-release before its release (`1.0.0-rc.1 < 1.0.0`), its dot-separated identifiers compared numbers as numbers, below words. Build metadata doesn't count for `<`, `>`, `<=`, and `>=`; `==` also compares it.
- **Strings:** a string compares with a version as a version (`1.2.3 == '1.2.3'`), and casts to one (`'v1.2.3' as ver`: a leading `v` is fine, as tags have it).
- **Wildcards:** a part can be `*` (`'1.2.*' as ver`, or `x`), which compares equal to any number.
- **Requirements:** `ver.matches('^1.2')`, as Cargo's (`^`, `~`, `=`, `>`, `>=`, `<`, `<=`, `*`, separated by commas); `ver.compatible(other)`; and `bump_major`, `bump_minor`, `bump_patch`.

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

# Ver::build(ver: ver) -> str
Return the build portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
assert_eq(ver.build(), "build");
```


# Ver::bump_major(ver: ver) -> void
Bump this version to its next major version (minor and patch to 0; the release and build cleared). A pre-release of a major version becomes that version.
```rust
const ver = 1.2.3-rc.1+build;
ver.bump_major();
assert_eq(ver, 2.0.0);
const pre = 2.0.0-rc.1;
pre.bump_major();
assert_eq(pre, 2.0.0);
```


# Ver::bump_minor(ver: ver) -> void
Bump this version to its next minor version (patch to 0; the release and build cleared). A pre-release of a minor version becomes that version.
```rust
const ver = 1.2.3;
ver.bump_minor();
assert_eq(ver, 1.3.0);
const pre = 1.3.0-rc.1;
pre.bump_minor();
assert_eq(pre, 1.3.0);
```


# Ver::bump_patch(ver: ver) -> void
Bump this version to its next patch version (the release and build cleared). A pre-release becomes its version.
```rust
const ver = 1.2.3;
ver.bump_patch();
assert_eq(ver, 1.2.4);
const pre = 1.2.4-rc.1;
pre.bump_patch();
assert_eq(pre, 1.2.4);
```


# Ver::clear_build(ver: ver) -> void
Clear the build portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
ver.clear_build();
assert_eq(ver, 1.2.3-release);
```


# Ver::clear_release(ver: ver) -> void
Clear the release portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
ver.clear_release();
assert_eq(ver, 1.2.3+build);
```


# Ver::compatible(ver: ver, other: ver) -> bool
Can the newer of these versions stand in for the older (as `^older` matches it)? The same major version, or for 0.x the same minor version (0.0.x: the same version). A pre-release goes only with its own version's pre-releases and the releases from that version on.
```rust
const ver = 1.2.0;
assert(ver.compatible(1.9.3));
assert(!ver.compatible(2.0.0));
```


# Ver::is_prerelease(ver: ver) -> bool
Is this version a pre-release (it has a release part, Ex. `1.0.0-rc.1`)?
```rust
const pre = 1.0.0-rc.1;
assert(pre.is_prerelease());
const release = 1.0.0+build;
assert(!release.is_prerelease());
```


# Ver::major(ver: ver) -> int
Return the major portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
assert_eq(ver.major(), 1);
```


# Ver::matches(ver: ver, req: str) -> bool
Does this version match a requirement, as Cargo's: `^1.2.3` (or a bare `1.2.3`: compatible versions), `~1.2` (patch updates), `=1.2.3`, `>1.2`, `>=1.2`, `<2`, `<=2.1`, `1.2.*`, or `*`, separated by commas (all must match). A pre-release only matches a requirement with a pre-release of its own version.
```rust
const ver = 1.4.2;
assert(ver.matches('^1.2'));
assert(!ver.matches('>=2.0.0, <3.0.0'));
const pre = 1.5.0-rc.2;
assert(!pre.matches('^1.4'));
assert(pre.matches('>=1.5.0-rc.1'));
```


# Ver::minor(ver: ver) -> int
Return the minor portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
assert_eq(ver.minor(), 2);
```


# Ver::patch(ver: ver) -> int
Return the patch portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
assert_eq(ver.patch(), 3);
```


# Ver::release(ver: ver) -> str
Return the release portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
assert_eq(ver.release(), "release");
```


# Ver::set_build(ver: ver, val: str) -> void
Set the build portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
ver.set_build("modified");
assert_eq(ver, 1.2.3-release+modified);
```


# Ver::set_major(ver: ver, val: int) -> void
Set the major portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
ver.set_major(4);
assert_eq(ver, 4.2.3-release+build);
```


# Ver::set_minor(ver: ver, val: int) -> void
Set the minor portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
ver.set_minor(4);
assert_eq(ver, 1.4.3-release+build);
```


# Ver::set_patch(ver: ver, val: int) -> void
Set the patch portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
ver.set_patch(4);
assert_eq(ver, 1.2.4-release+build);
```


# Ver::set_release(ver: ver, val: str) -> void
Set the release portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
ver.set_release("modified");
assert_eq(ver, 1.2.3-modified+build);
```


