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

# At
Return an element at the given index (or null if out of bounds).

# Length
Returns the size of this tuple.

