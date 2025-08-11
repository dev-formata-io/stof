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

# Any
Returns true if this list is not empty.

# Append
Appends another list to this one (returns nothing).

# At
Return an element at the given index (or null if out of bounds).

# Back
Returns the value at the back of this list or null if the list is empty.

# Clear
Clears the list of all values.

# Contains
Returns true if this list contains the given value.

# Empty
Returns true if this list is empty.

# Front
Returns the value at the front of this list or null if the list is empty.

# Index Of
Returns the first index of the given value if found or -1 if not found.

# Insert
Insert a value into this list at the given index, pushing values right.

# Is Uniform?
Returns true if the list contains only a singular type of value.

# Join
Joins the values in this list into a string, separated by a given separator (default is an empty space char).

# Length
Returns the length of this list.

# Pop Back
Removes a value from the back of this list, returning that value or null if the list is empty.

# Pop Front
Removes a value from the front of this list, returning that value or null if the list is empty.

# Push Back
Pushes arguments to the back of this list.

# Push Front
Pushes arguments to the front of this list (in order).

# Remove
Remove a value at the given index, returning it if found or null if the index is out of bounds.

# Remove All
Remove all occurrances of a value, returning true if any were found and removed.

# Remove First
Remove the first occurrance of a value, returning it if found or null otherwise.

# Remove Last
Remove the last occurrance of a value, returning it if found or null otherwise.

# Replace
Replace a value into this list at the given index, returning the old value.

# Reverse
Reverses the list in place.

# Reversed
Returns a new list in the reverse order.

# Sort
Sorts this list given the default value ordering.

# To Uniform Type
Casts each value in this list to a given type.

