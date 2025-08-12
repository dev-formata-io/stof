# Map Library (Map)
Library linked to the 'map' type. Maps are ordered by keys.

## Example Usage
```rust
#[main]
fn main() {
    const a = {1: 'hi'};
    assert_eq(a.len(), 1);
    assert_eq(Map.len(a), 1);
}
```

# Map.any(this: map) -> bool
Does this map contain any key-value pairs?
```rust
const map = map();
assert_not(map.any());
```

# Map.append(this: map, other: map) -> void
Append the items of another map onto this map.
```rust
const map = {"a": 1};
map.append({"b": 2});
assert_eq(map, {"a": 1, "b": 2});
```

# Map.at(this: map, index: int) -> (unknown, unknown)
The key-value pair at the given index in this sorted map.
```rust
const map = {"a": 1, "b": 2};
assert_eq(map.at(1), ("b", 2));
```

# Map.clear(this: map) -> void
Clear this map of all items.
```rust
const map = {"a": 1};
map.clear();
assert(map.empty());
```

# Map.contains(this: map, key: unknown) -> bool
Returns true if this map contains a key that equals the given value.
```rust
const map = {"a": 1};
assert(map.contains("a"));
```

# Map.empty(this: map) -> bool
Is this map empty?
```rust
const map = map();
assert(map.empty());
```

# Map.first(this: map) -> (unknown, unknown)
Return the first key-value pair in this ordered map, or null if the map is empty. Optionally return the value as a reference with the '&' operator.
```rust
const map = {"a": 1};
assert_eq(map.first(), ("a", 1));
```

# Map.get(this: map, key: unknown) -> unknown
Return a value for the given key in this map, optionally by reference.
```rust
const map = {"a": 1};
assert_eq(map.get("a"), 1);
```

# Map.insert(this: map, key: unknown, value: unknown) -> unknown
Insert a key-value pair into this map, returning the old value if the key was already present, or null otherwise.
```rust
const map = {"a": 1};
assert_eq(map.insert("a", 3), 1);
assert_eq(map, {"a": 3});
```

# Map.keys(this: map) -> set
A set of this map's keys.
```rust
const map = {"a": 1, "b": 2};
assert_eq(map.keys(), {"a", "b"});
```

# Map.last(this: map) -> (unknown, unknown)
Return the last key-value pair in this ordered map, or null if the map is empty. Optionally return the value as a reference with the '&' operator.
```rust
const map = {"a": 1, "b": 3};
assert_eq(map.last(), ("b", 3));
```

# Map.len(this: map) -> int
The number of key-value pairs in this map.
```rust
const map = {"a": 1, "b": 2};
assert_eq(map.len(), 2);
```

# Map.pop_first(this: map) -> (unknown, unknown)
Remove the smallest key-value pair from this map and return it.
```rust
const map = {"a": 1, "b": 2};
assert_eq(map.pop_first(), ("a", 1));
```

# Map.pop_last(this: map) -> (unknown, unknown)
Remove the largest key-value pair from this map and return it.
```rust
const map = {"a": 1, "b": 2};
assert_eq(map.pop_last(), ("b", 2));
```

# Map.remove(this: map, key: unknown) -> unknown
Remove the value with the given key and return it, or null if the key isn't present.
```rust
const map = {"a": 1, "b": 2};
assert_eq(map.remove("b"), 2);
assert_eq(map, {"a": 1});
```

# Map.values(this: map) -> list
A list of this map's values.
```rust
const map = {"a": 1, "b": 2};
assert_eq(map.values(), [1, 2]);
```

