# File System Library (fs)
Functions for working with the file system. Requires the "system" feature to automatically be added, otherwise, remove this library to further sandbox your environment.

# fs.read(path: str) -> blob
If available, will read a file from a path into a binary blob.
```rust
const bytes = fs.read("src/lib.rs");
```

# fs.read_string(path: str) -> str
If available, will read a file from a path into a string.
```rust
const content = fs.read_string("src/lib.rs");
```

# fs.write(path: str, content: str | blob) -> void
If available, will write content into a file at the given path. Will throw an error if the directory doesn't exist and will overwrite the file if it already exists.
```rust
fs.write("src/text.txt", "testing");
```

