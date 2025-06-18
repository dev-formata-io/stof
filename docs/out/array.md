
# Array
In Stof, arrays and vectors are the same thing.

## append(self:vec, other:vec) -> void
Appends another vector onto this array, leaving the other empty. If not boxed, "other" will be cloned
when this function is called, and the original vector maintains its values.
### Example
```rust
let a = [1, 2, 3, 4, 5];
let b = [6, 7, 8, 9]; // change to 'box([6, 7, 8, 9])' and see what happens
a.append(b);
assertEq(a, [1, 2, 3, 4, 5, 6, 7, 8, 9]);
assertEq(b, [6, 7, 8, 9]);
```
