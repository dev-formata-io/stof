# String Library (Str)
Library for manipulating strings, automatically linked to the 'str' type.

## Example Usage
```rust
#[main]
fn main() {
    assert_eq("hello, world".split(", "), ['hello', 'world']);
}
```

# String At (index op)
Return a char (as a string) at the given index.

# String Contains
Return true if the string contains a given sequence.

# String Ends With
Return true if the string ends with a given sequence.

# First Char in String
Return the first char (as a string) in a string.

# Index Of
Return the index of a given squence (first char) if found, otherwise -1.

# Last Char in String
Return the last char (as a string) in a string.

# String Length
Return the length of a string.

# To Lowercase
Converts all chars to lowercase.

# Push
Pushes a string to the back of a string (concatination). Does not return anything.

# String Replace
Replace all instances of a find string with a replace string (defaults to an empty replace string, which removes all instances of the find string). Returns a new string without modifying the original.

# Split
Splits a string into a list at the given separator. Default separator is a single space.

# String Starts With
Return true if the string starts with a given sequence.

# Substring
Return a substring from a start index to an optional end index (up to, but not including). Default end index is the length of the string.

# Trim
Trims whitespace from the front and back of a string.

# Trim End
Trims whitespace from the end of a string.

# Trim Start
Trims whitespace from the start of a string.

# To Uppercase
Converts all chars to uppercase.

