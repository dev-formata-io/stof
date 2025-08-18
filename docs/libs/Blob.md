# Blob Library (Blob)
Library for working with binary blobs (Vec\<u8> or Uint8Array), linked to the 'blob' type. Useful when working with web APIs, raw binary data, and in exchange scenarios between formats (see "blobify" in the Std library for more details).

# Blob.at(bytes: blob, index: int) -> int
Byte at a specific index within this blob.
```rust
const bytes: blob = "hello";
assert_eq(bytes[1], 101); // or '.at(1)' or 'Blob.at(bytes, 1)'
```

# Blob.base64(bytes: blob) -> str
Transform this blob into a string using Base64 encoding.
```rust
const bytes: blob = "hello";
assert_eq(bytes.base64(), "aGVsbG8=");
```

# Blob.from_base64(val: str) -> blob
Transform a string into a blob, using Base64 encoding.
```rust
const bytes: blob = Blob.from_base64("aGVsbG8=");
assert_eq(bytes as str, "hello");
```

# Blob.from_url_base64(val: str) -> blob
Transform a string into a blob, using URL-safe Base64 encoding.
```rust
const bytes: blob = Blob.from_url_base64("aGVsbG8=");
assert_eq(bytes as str, "hello");
```

# Blob.from_utf8(val: str) -> blob
Transform a string into a blob, using standard UTF-8 encoding (default for normal casts too).
```rust
const bytes: blob = "hello";
assert_eq(bytes, Blob.from_utf8("hello"));
```

# Blob.len(bytes: blob) -> int
Size of this binary blob (integer number of bytes).
```rust
const bytes: blob = "hello";
assert_eq(bytes.len(), 5);
```

# Blob.size(bytes: blob) -> bytes
Size of this binary blob (in units of bytes).
```rust
const bytes: blob = "hello";
assert_eq(bytes.size(), 5bytes);
```

# Blob.url_base64(bytes: blob) -> str
Transform this blob into a string using URL-safe Base64 encoding.
```rust
const bytes: blob = "hello";
assert_eq(bytes.url_base64(), "aGVsbG8=");
```

# Blob.utf8(bytes: blob) -> str
Transform this blob into a string using UTF-8 (default conversion for casts also).
```rust
const bytes: blob = "hello";
assert_eq(bytes.utf8(), "hello");
assert_eq(bytes as str, "hello");
```

