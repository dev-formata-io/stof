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

# Set.any(set: set) -> bool
Does this set contain any values?
```rust
const set = {1, 2, 3};
assert(set.any());
```


# Set.append(set: set, other: set) -> void
Append another set to this one.
```rust
const set = {1, 2, 3};
set.append({3, 4});
assert_eq(set, {1, 2, 3, 4});
```


# Set.at(set: set, index: int) -> unknown
Return the Nth (index) element in this ordered set, or null if the index is out of bounds.
```rust
const set = {1, 2, 3};
assert_eq(set[1], 2);
```


# Set.clear(set: set) -> void
Clear all values from the set.
```rust
const set = {1, 2, 3};
set.clear();
assert_eq(set, {});
```


# Set.contains(set: set, val: unknown) -> bool
Returns true if the set contains the value.
```rust
const set = {1, 2, 3};
assert(set.contains(3));
```


# Set.difference(set: set, other: set) -> set
Perform a difference between two sets, returning a new set (everything in this set that is not in other).
```rust
const set = {1, 2, 3};
assert_eq(set.difference({2, 3}), {1});
```


# Set.disjoint(set: set, other: set) -> bool
Returns true if there is no overlap between the two sets (empty intersection).
```rust
const set = {1, 2, 3};
const other = {4, 5};
assert(set.disjoint(other));
```


# Set.empty(set: set) -> bool
Is this set empty?
```rust
const set = {1, 2, 3};
assert_not(set.empty());
```


# Set.first(set: set) -> unknown
Return the first (minimum) value in the set, or null if the set is empty.
```rust
const set = {1, 2, 3};
assert_eq(set.first(), 1);
```


# Set.insert(set: set, val: unknown) -> bool
Insert the value into the set, returning true if the value was not previously in the set (newly inserted).
```rust
const set = {1, 2};
assert(set.insert(3));
```


# Set.intersection(set: set, other: set) -> set
Perform an intersection between two sets, returning a new set (only elements found in both sets).
```rust
const set = {1, 2, 3};
assert_eq(set.intersection({3, 4}), {3});
```


# Set.is_uniform(set: set) -> bool
Returns true if all values in this set are of the same specific type.
```rust
const set = {2, 3};
assert(set.is_uniform());
```


# Set.last(set: set) -> unknown
Return the last (maximum) value in the set, or null if the set is empty.
```rust
const set = {1, 2, 3};
assert_eq(set.last(), 3);
```


# Set.len(set: set) -> int
Return the size of this set (cardinality).
```rust
const set = {1, 2, 3};
assert_eq(set.len(), 3);
```


# Set.pop_first(set: set) -> unknown
Remove and return the first (minimum) value in the set.
```rust
const set = {1, 2, 3};
assert_eq(set.pop_first(), 1);
assert_eq(set, {2, 3});
```


# Set.pop_last(set: set) -> unknown
Remove and return the last (maxiumum) value in the set.
```rust
const set = {1, 2, 3};
assert_eq(set.pop_last(), 3);
assert_eq(set, {1, 2});
```


# Set.remove(set: set, val: unknown) -> unknown
Remove and return the value if found in the set, otherwise null.
```rust
const set = {1, 2, 3};
assert_eq(set.remove(2), 2);
assert_eq(set, {1, 3});
```


# Set.split(set: set, val: unknown) -> (set, set)
Split the set into a smaller set (left) and larger set (right) at the given value (not included in resulting sets).
```rust
const set = {1, 2, 3};
assert_eq(set.split(2), ({1}, {3}));
```


# Set.subset(set: set, other: set) -> bool
Returns true if all values in this set exist within another set.
```rust
const set = {2, 3};
const other = {2, 3, 4};
assert(set.subset(other));
```


# Set.superset(set: set, other: set) -> bool
Returns true if all values in another set exist within this set.
```rust
const set = {2, 3};
const other = {2, 3, 4};
assert(other.superset(set));
```


# Set.symmetric_difference(set: set, other: set) -> set
Perform a symmetric difference between two sets, returning a new set (values in this set that do not exist in other unioned with the values in other that do not exist in this set).
```rust
const set = {1, 2, 3};
assert_eq(set.symmetric_difference({2, 3, 4}), {1, 4});
```


# Set.to_uniform(set: set, type: str) -> void
Try casting all set values to a single type. Type parameter is a string, just like you'd specify a type in Stof.
```rust
const set = {2000m, 3km};
set.to_uniform("km");
assert_eq(set, {2km, 3km});
```


# Set.union(set: set, other: set) -> set
Union two sets, returning a new set.
```rust
const set = {1, 2, 3};
const other = {4, 5};
assert_eq(set.union(other), {1, 2, 3, 4, 5});
```


