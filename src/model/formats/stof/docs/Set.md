# Set Library (Set)
Library linked to the 'set' type.

## Example Usage
```rust
#[main]
fn main() {
    const a = {1, 2, 3};
    assert_eq(a.len(), 3);
    assert_eq(Set.len(a), 3);
}
```

# Any?
Returns true if the set contains at least one value.

# Append
Appends another set to this one (returns nothing).

# At
Returns the value at the given index or null if the index if out of bounds.

# Clear
Clear all values from a set (returns nothing).

# Contains?
Returns true if the set contains a given value.

# Difference
Performs a difference between two sets, returning a new set (everything in this set that is not in other).

# Disjoint?
Returns true if there is no overlap between two sets (empty intersection).

# Empty?
Returns true if the set is empty.

# First
Returns the first (minimum) value in the set.

# Insert
Insert a value into this set, returning true if the value was newly inserted.

# Intersection
Performs an intersection between two sets, returning a new set.

# Uniform Types?
Returns true if all values in this set are of the same type.

# Last
Returns the last (maximum) value in the set.

# Length
Returns the size of the set.

# Pop First
Removes and returns the first (minimum) value in the set.

# Pop Last
Removes and returns the last (maximum) value in the set.

# Remove
Removes and returns a value from the set or null if the value doesn't exist.

# Split
Split this set into two sets - one that contains all smaller values and one that contains all larger values. Will return a tuple containing the two sets, with the smaller values at index 0 (Tup(smaller set, larger set)).

# Subset?
Returns true if all values in this set exist within another set.

# Superset?
Returns true if all values in another set exist within this set.

# Symmetric Difference
Performs a symmetric difference between two sets, returning a new set (values in this set that do not exist in other unioned with the values in other that do not exist in this set).

# To Uniform
Casts all values in the set to the same type.

# Union
Performs a union between two sets, returning a new set.

