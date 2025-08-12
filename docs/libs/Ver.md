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

# Build
Return the build portion of this version.

# Clear Build
Removes the build portion of this version (does not return anything).

# Clear Release
Removes the release portion of this version (does not return anything).

# Major
Return the major portion of this version.

# Minor
Return the minor portion of this version.

# Patch
Return the patch portion of this version.

# Release
Return the release portion of this version.

# Set Build
Set the build portion of this version (does not return anything).

# Set Major
Set the major portion of this version (does not return anything).

# Set Minor
Set the minor portion of this version (does not return anything).

# Set Patch
Set the patch portion of this version (does not return anything).

# Set Release
Set the release portion of this version (does not return anything).

