# Tuple Library (Tup)
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

# Tup.at(tup: (..), index: int) -> unknown
Return the value (optionally by reference) at the given index in the tuple.
```rust
const tup = ("hi", 42, true);
assert_eq(&tup[1], 42);
```


# Tup.len(tup: (..)) -> int
Return the length of this tuple.
```rust
const tup = ("hi", 42, true);
assert_eq(tup.len(), 3);
```


