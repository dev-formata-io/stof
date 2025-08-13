# Semantic Version Library (Ver)
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

# Ver.build(ver: ver) -> str
Return the build portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
assert_eq(ver.build(), "build");
```


# Ver.clear_build(ver: ver) -> void
Clear the build portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
ver.clear_build();
assert_eq(ver, 1.2.3-release);
```


# Ver.clear_release(ver: ver) -> void
Clear the release portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
ver.clear_release();
assert_eq(ver, 1.2.3+build);
```


# Ver.major(ver: ver) -> int
Return the major portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
assert_eq(ver.major(), 1);
```


# Ver.minor(ver: ver) -> int
Return the minor portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
assert_eq(ver.minor(), 2);
```


# Ver.patch(ver: ver) -> int
Return the patch portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
assert_eq(ver.patch(), 3);
```


# Ver.release(ver: ver) -> str
Return the release portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
assert_eq(ver.release(), "release");
```


# Ver.set_build(ver: ver, val: str) -> void
Set the build portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
ver.set_build("modified");
assert_eq(ver, 1.2.3-release+modified);
```


# Ver.set_major(ver: ver, val: int) -> void
Set the major portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
ver.set_major(4);
assert_eq(ver, 4.2.3-release+build);
```


# Ver.set_minor(ver: ver, val: int) -> void
Set the minor portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
ver.set_minor(4);
assert_eq(ver, 1.4.3-release+build);
```


# Ver.set_patch(ver: ver, val: int) -> void
Set the patch portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
ver.set_patch(4);
assert_eq(ver, 1.2.4-release+build);
```


# Ver.set_release(ver: ver, val: str) -> void
Set the release portion of this semantic version.
```rust
const ver = 1.2.3-release+build;
ver.set_release("modified");
assert_eq(ver, 1.2.3-modified+build);
```


