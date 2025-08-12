# Map Library (Map)
Library linked to the 'map' type.

## Example Usage
```rust
#[main]
fn main() {
    const a = {1: 'hi'};
    assert_eq(a.len(), 1);
    assert_eq(Map.len(a), 1);
}
```

# Any?
Returns true if this map is not empty.

# Append
Appends another map to this one (returns nothing).

# At
Returns a key-value pair at an index within this map.

# Clear
Clears a map of all keys and values (returns nothing).

# Contains Key?
Returns true if a map contains a key.

# Empty?
Returns true if this map is empty.

# First
Returns the minimum key-value pair in this map.

# Get
Get a value in this map by key.

# Insert
Insert a key-value pair into this map.

# Keys
Returns a set of keys in this map.

# Last
Returns the maximum key-value pair in this map.

# Length (size)
Returns the size of this map.

# Pop First (min)
Removes and returns the first key-pair in this map (min) or null if the map is empty.

# Pop Last (max)
Removes and returns the last key-pair in this map (max) or null if the map is empty.

# Remove
Removes a value in this map by key or returns null if the key isn't found.

# Values
Returns a list of values in this map.

