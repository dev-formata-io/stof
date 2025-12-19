# String Library (Str)
Library for manipulating strings, automatically linked to the 'str' type.

## Example Usage
```rust
#[main]
fn main() {
    assert_eq("hello, world".split(", "), ['hello', 'world']);
}
```

# Str.at(val: str, index: int) -> str
Returns a character at the given index within the string, or the last character if the index is out of bounds.
```rust
const val = "hello";
assert_eq(val[1], "e");
```


# Str.contains(val: str, seq: str) -> bool
Return true if the sequence is found at least once anywhere in this string.
```rust
const val = "hello, world";
assert(val.contains(", w"));
```


# Str.ends_with(val: str, seq: str) -> bool
Does this string end with the given string sequence?
```rust
const val = "hello";
assert(val.ends_with("llo"));
```


# Str.find_matches(val: str, regex: str) -> list
Return a list of tuples "(content: str, start: int, end: int)" that represent all matches of the regex in the string vlaue.
```rust
const val = "I categorically deny having triskaidekaphobia.";
const regex = "\\b\\w{13}\\b";
assert_eq(val.find_matches(regex), [("categorically", 2, 15)]);
```


# Str.first(val: str) -> str
Return the first char (as a string) in this string.
```rust
const val = "hello";
assert_eq(val.first(), "h");
```


# Str.index_of(val: str, seq: str) -> int
Find the first occurrance of the given sequence in this string, returning the index of the first char. If not found, returns -1.
```rust
const val = "hello, world";
assert_eq(val.index_of(", w"), 5);
```


# Str.last(val: str) -> str
Return the last char (as a string) in this string.
```rust
const val = "hello";
assert_eq(val.last(), "o");
```


# Str.len(val: str) -> int
Returns the length (number of characters) in this string.
```rust
assert_eq("hello".len(), 5);
```


# Str.lower(val: str) -> str
Return a new string with all characters converted to lowercase.
```rust
const val = "HELLO";
assert_eq(val.lower(), "hello");
```


# Str.matches(val: str, regex: str) -> bool
Return true if this string matches the provided regex string.
```rust
const val = "I categorically deny having triskaidekaphobia.";
const regex = "\\b\\w{13}\\b";
assert(val.matches(regex));
```


# Str.push(val: str, other: str) -> void
Pushes another string to the back of this string, leaving the other string unmodified.
```rust
const val = "hello";
val.push(", world");
assert_eq(val, "hello, world");
```


# Str.replace(val: str, find: str, replace: str = "") -> str
Replace all occurrances of a find string with a replace string (default removes all occurrances). This will return a new string, without modifying the original.
```rust
const val = "hello john";
assert_eq(val.replace(" ", ", "), "hello, john");
```


# Str.split(val: str, sep: str = " ") -> list
Splits a string into a list at the given separator.
```rust
const val = "hello, world";
assert_eq(val.split(", "), ["hello", "world"]);
```


# Str.starts_with(val: str, seq: str) -> bool
Does this string start with the given string sequence?
```rust
const val = "hello";
assert(val.starts_with("he"));
```


# Str.substring(val: str, start: int = 0, end: int = -1) -> str
Return a new string that is the substring of the given value from a start index up to, but not including an end index. Default start is the beginning of the string and the default end is the entire length of the string.
```rust
const val = "hello, world";
assert_eq(val.substring(), "hello, world");
assert_eq(val.substring(7), "world");
assert_eq(val.substring(3, 8), "lo, w");
```


# Str.trim(val: str) -> str
Return a new string with the whitespace (newlines, tabs, and space characters) removed from the front and back.
```rust
const val = "\n\thello\t\n";
assert_eq(val.trim(), "hello");
```


# Str.trim_end(val: str) -> str
Return a new string with the whitespace (newlines, tabs, and space characters) removed from the back only.
```rust
const val = "\n\thello\t\n";
assert_eq(val.trim_end(), "\n\thello");
```


# Str.trim_start(val: str) -> str
Return a new string with the whitespace (newlines, tabs, and space characters) removed from the front only.
```rust
const val = "\n\thello\t\n";
assert_eq(val.trim_start(), "hello\t\n");
```


# Str.upper(val: str) -> str
Return a new string with all characters converted to uppercase.
```rust
const val = "hello";
assert_eq(val.upper(), "HELLO");
```


