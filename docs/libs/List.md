# List Library (List)
Library linked to the 'list' type.

## Example Usage
```rust
#[main]
fn main() {
    const array = [1, 2, 3];
    assert_eq(array.len(), 3);
    assert_eq(List.len(array), 3);
}
```

# List.any(array: list) -> bool
Does this list contain any values?
```rust
const array = [1];
assert(array.any());
```

# List.append(array: list, other: list) -> void
Append another list to this list, leaving other unmodified.
```rust
const array = [1, 2, 3];
array.append([4, 5]);
assert_eq(array, [1, 2, 3, 4, 5]);
```

# List.at(array: list, index: int) -> unknown
Get the value at the given index, optionally by reference. 
```rust
const array = [1, 2, 3];
let v = &array[1]; // &List.at(array, 1);
v = 5;
assert_eq(array, [1, 5, 3]);
```

# List.back(array: list) -> unknown
Get the value at the back of this list, optionally by reference.
```rust
const array = [1, 2];
assert_eq(array.back(), 2);
```

# List.clear(array: list) -> void
Clear all values from this list.
```rust
const array = [1, 2, 3];
array.clear();
assert_eq(array, []);
```

# List.contains(array: list, value: unknown) -> bool
Does this list contain the given value?
```rust
const array = [1];
assert(array.contains(1));
```

# List.empty(array: list) -> bool
Is this list empty?
```rust
const array = [1];
assert_not(array.empty());
```

# List.front(array: list) -> unknown
Get the value at the front of this list, optionally by reference.
```rust
const array = [1];
assert_eq(array.front(), 1);
```

# List.index_of(array: list, v: unknown) -> int
If the list contains the given value, return the index of the first matched value. Returns -1 if the list does not contain the given value.
```rust
const array = [1, 2, 3];
assert_eq(array.index_of(2), 1);
```

# List.insert(array: list, index: int, val: unknown) -> void
Insert a value into this list at the given index.
```rust
const array = [2, 1];
array.insert(1, 3);
assert_eq(array, [2, 3, 1]);
```

# List.is_uniform(array: list) -> bool
Returns true if every value in this list has the same specific type (does not account for object prototype inheritance).
```rust
const array = ["hi", true];
assert_not(array.is_uniform());
```

# List.join(array: list, sep: str) -> str
Join the values in this array together into a single string.
```rust
const array = ["hello", "world"];
assert_eq(array.join(", "), "hello, world");
```

# List.len(array: list) -> int
Return the length of this list.
```rust
const array = [1, 2, 3];
assert_eq(array.len(), 3);
```

# List.pop_back(array: list) -> unknown
Remove a single value from the back of this list and return it. 
```rust
const array = [1, 2, 3];
assert_eq(array.pop_back(), 3);
assert_eq(array, [1, 2]);
```

# List.pop_front(array: list) -> unknown
Remove a single value from the front of this list and return it. 
```rust
const array = [1, 2, 3];
assert_eq(array.pop_front(), 1);
assert_eq(array, [2, 3]);
```

# List.push_back(array: list, ..) -> void
Push N values to the back of this list.
```rust
const array = [1, 2, 3];
array.push_back(4, 5);
assert_eq(array, [1, 2, 3, 4, 5]);
```

# List.push_front(array: list, ..) -> void
Push N values to the front of this list.
```rust
const array = [1, 2, 3];
array.push_front(4, 5);
assert_eq(array, [5, 4, 1, 2, 3]);
```

# List.remove(array: list, index: int) -> unknown
Remove a value at the given index and return it. Returns null if index is out of bounds.
```rust
const array = [1];
assert_eq(array.remove(0), 1);
assert(array.empty());
```

# List.remove_all(array: list, val: unknown) -> bool
Remove all occurrances of a value in this array (equals) and return true if any were removed.
```rust
const array = [2, 1, 1, 2];
assert(array.remove_all(2));
assert_eq(array, [1, 1]);
```

# List.remove_first(array: list, val: unknown) -> unknown
Remove the first occurrance of a value in this array (equals) and return it.
```rust
const array = [2, 1, 1, 2];
assert_eq(array.remove_first(2), 2);
assert_eq(array, [1, 1, 2]);
```

# List.remove_last(array: list, val: unknown) -> unknown
Remove the last occurrance of a value in this array (equals) and return it.
```rust
const array = [2, 1, 1, 2];
assert_eq(array.remove_last(2), 2);
assert_eq(array, [2, 1, 1]);
```

# List.replace(array: list, index: int, val: unknown) -> unknown
Replace/set the value at the given index with a new value, returning the old.
```rust
const array = [2, 1];
assert_eq(array.replace(1, 4), 1);
assert_eq(array, [2, 4]);
```

# List.reverse(array: list) -> void
Reverses this list in-place.
```rust
const array = [1, 2, 3];
array.reverse();
assert_eq(array, [3, 2, 1]);
```

# List.reversed(array: list) -> list
Return a new list that is reversed, leaving this list unmodified.
```rust
const array = [1, 2, 3];
const other = array.reversed();
assert_eq(array, [1, 2, 3]);
assert_eq(other, [3, 2, 1]);
```

# List.sort(array: list) -> void
Sort the values in this array according to their already defined ordering.
```rust
const array = [2, 1, 4, 3];
array.sort();
assert_eq(array, [1, 2, 3, 4]);
```

# List.sort_by(array: list, func: fn) -> void
Sort the values in this array according to a function that takes two list arguments and returns an integer (< 0 for less, > 0 for greater, and 0 for equal).
```rust
const array = [2, 1, 4, 3];
array.sort_by((a: int, b: int): int => {
    if (a < b) 1
    if (a > b) -1
    0
});
assert_eq(array, [4, 3, 2, 1]);
```

# List.to_uniform(array: list, type: str) -> void
Try casting all values in this list to the given type (given as a string like you would in a Stof file). Will throw an error if a value cannot be cast.
```rust
const array = [1, "hi", true];
array.to_uniform("str");
assert_eq(array, ["1", "hi", "true"]);
```

