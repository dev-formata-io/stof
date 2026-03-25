# Standard Library Reference

Complete reference for Stof's built-in library functions. Each type has linked methods that can be called on values of that type directly (e.g., `my_list.len()`) or via the library namespace (e.g., `List.len(my_list)`).

For full API documentation with examples, see the [auto-generated library docs](../../docs/libs/).

---

## Global Functions

```stof
pln(...)                          // print with newline
print(...)                        // print without newline
err(...)                          // print to stderr
typeof val                        // primitive type as string
typename val                      // full type name (units/prototype)
str(val)                          // convert to string
stringify(format, obj)            // serialize: 'json', 'yaml', 'toml', 'stof:human', 'bstf'
parse(source, dest, format)       // deserialize into dest object
blobify(format, obj)              // serialize to binary blob
copy(val)                         // deep copy (breaks references)
nanoid()                          // generate unique ID
swap(&a, &b)                      // swap by reference
env("KEY") / set_env("KEY", "v")  // environment variables
sleep(100ms)                      // sleep duration
exit()                            // exit process
min(a, b) / max(a, b)            // math
lib('Http') / libs()             // check available libraries
funcs('my_attr')                 // find functions by attribute
funcs(attributes = key)          // find functions with specific attribute
xml(content, tag)                // XML helper: xml('hello', 'msg') → '<msg>hello</msg>'
```

---

## Obj (Object Library)

Linked to the `obj` type. Core methods for inspecting and manipulating objects.

| Method | Signature | Description |
|--------|-----------|-------------|
| `any` | `() -> bool` | Has any data attached |
| `at` | `(index: int) -> (str, unknown)` | Field (name, value) at index |
| `attributes` | `(path?: str) -> map` | Get attributes map |
| `children` | `() -> list` | List of child objects |
| `contains` | `(name: str) -> bool` | Has data with given name |
| `create_type` | `(typename: str) -> void` | Register as prototype programmatically |
| `drop` | `(shallow?: bool) -> void` | Remove object from graph |
| `fields` | `() -> list` | List of (name, value) tuples |
| `funcs` | `(attributes?: str) -> list` | Functions, optionally filtered by attribute |
| `get` | `(path: str) -> unknown` | Get field by dot-path |
| `has` | `(path: str) -> bool` | Check if path exists |
| `instance_of` | `(typename: str) -> bool` | Check prototype membership |
| `len` | `() -> int` | Number of data components |
| `name` | `() -> str` | Object name |
| `parent` | `() -> obj` | Parent object |
| `path` | `() -> str` | Full dot-path from root |
| `remove` | `(name: str, shallow?: bool) -> void` | Remove data by name |
| `rename` | `(name: str) -> void` | Rename this object |
| `run` | `() -> unknown` | Execute `#[run]` workflow |
| `schemafy` | `(obj) -> void` | Validate against this prototype's schema |
| `set` | `(path: str, value) -> void` | Set field by dot-path |

---

## List / Vec (Array Library)

Linked to the `list` type.

| Method | Signature | Description |
|--------|-----------|-------------|
| `any` | `() -> bool` | Contains any values |
| `append` | `(other: list) -> void` | Append another list |
| `at` | `(index: int) -> unknown` | Get value at index (supports `&` reference) |
| `back` | `() -> unknown` | Last element |
| `clear` | `() -> void` | Remove all values |
| `contains` | `(value) -> bool` | Check if value exists |
| `empty` | `() -> bool` | Is empty |
| `filter` | `(fn) -> list` | Filter by predicate |
| `find` | `(fn) -> unknown` | Find first match |
| `flat` | `() -> list` | Flatten nested lists |
| `front` | `() -> unknown` | First element |
| `iter` | `(fn) -> void` | Iterate with function |
| `join` | `(sep: str) -> str` | Join elements as string |
| `len` | `() -> int` | Length |
| `map` | `(fn) -> list` | Transform elements |
| `pop_back` | `() -> unknown` | Remove and return last |
| `pop_front` | `() -> unknown` | Remove and return first |
| `push_back` | `(value) -> void` | Add to end |
| `push_front` | `(value) -> void` | Add to beginning |
| `reduce` | `(fn, init) -> unknown` | Reduce to single value |
| `remove` | `(index: int) -> unknown` | Remove at index |
| `reverse` | `() -> void` | Reverse in place |
| `sort` | `(fn?) -> void` | Sort (optional comparator) |
| `unique` | `() -> list` | Remove duplicates |

---

## Map Library

Linked to the `map` type.

| Method | Signature | Description |
|--------|-----------|-------------|
| `any` | `() -> bool` | Contains any entries |
| `at` | `(key) -> unknown` | Get value by key |
| `clear` | `() -> void` | Remove all entries |
| `contains` | `(key) -> bool` | Check if key exists |
| `empty` | `() -> bool` | Is empty |
| `get` | `(key) -> unknown` | Get value (null if missing) |
| `insert` | `(key, value) -> void` | Insert or update |
| `keys` | `() -> list` | List of keys |
| `len` | `() -> int` | Number of entries |
| `remove` | `(key) -> unknown` | Remove and return |
| `values` | `() -> list` | List of values |

---

## Set Library

Linked to the `set` type.

| Method | Signature | Description |
|--------|-----------|-------------|
| `any` | `() -> bool` | Contains any values |
| `clear` | `() -> void` | Remove all |
| `contains` | `(value) -> bool` | Check membership |
| `empty` | `() -> bool` | Is empty |
| `insert` | `(value) -> void` | Add value |
| `len` | `() -> int` | Size |
| `remove` | `(value) -> bool` | Remove value |

---

## Str (String Library)

Linked to the `str` type.

| Method | Signature | Description |
|--------|-----------|-------------|
| `at` | `(index: int) -> str` | Character at index |
| `contains` | `(seq: str) -> bool` | Contains substring |
| `ends_with` | `(seq: str) -> bool` | Ends with |
| `find_matches` | `(regex: str) -> list` | Regex matches |
| `len` | `() -> int` | Length |
| `lines` | `() -> list` | Split into lines |
| `lower` | `() -> str` | Lowercase |
| `replace` | `(from, to) -> str` | Replace all occurrences |
| `split` | `(sep: str) -> list` | Split by separator |
| `starts_with` | `(seq: str) -> bool` | Starts with |
| `trim` | `() -> str` | Trim whitespace |
| `upper` | `() -> str` | Uppercase |

---

## Num (Number Library)

Static math functions.

| Method | Signature | Description |
|--------|-----------|-------------|
| `Num.abs` | `(n) -> float` | Absolute value |
| `Num.ceil` | `(n) -> int` | Ceiling |
| `Num.floor` | `(n) -> int` | Floor |
| `Num.max` | `(a, b) -> float` | Maximum |
| `Num.min` | `(a, b) -> float` | Minimum |
| `Num.pow` | `(base, exp) -> float` | Power |
| `Num.random` | `(min?, max?) -> float` | Random number |
| `Num.round` | `(n) -> int` | Round |
| `Num.sqrt` | `(n) -> float` | Square root |
| `.pow` | `(exp) -> float` | Instance power |

---

## Http (Network Library)

Requires the "http" feature flag. Adds a thread pool for async HTTP requests.

| Method | Signature | Description |
|--------|-----------|-------------|
| `Http.fetch` | `async (url, method?, body?, headers?, timeout?, query?, bearer?) -> Promise<map>` | Make HTTP request |
| `Http.text` | `(response: map) -> str` | Extract text body |
| `Http.blob` | `(response: map) -> blob` | Extract binary body |
| `Http.parse` | `(response: map, context?: obj) -> obj` | Parse response into object |
| `Http.success` | `(response: map) -> bool` | Status 200-299 |
| `Http.client_error` | `(response: map) -> bool` | Status 400-499 |
| `Http.server_error` | `(response: map) -> bool` | Status 500-599 |
| `Http.size` | `(response: map) -> bytes` | Response body size |

---

## Other Libraries

### Ver (Semantic Version)
Methods: `major()`, `minor()`, `patch()`, `pre()`, `build()`, `compatible(other)`, `increment(part)`.

### Time
Methods: `Time.now()`, `Time.elapsed(start)`, `Time.millis()`, `Time.stamp()`, formatting functions.

### Tuple
Methods: `at(index)`, `len()`, `set(index, value)`.

### Prompt (AI Prompt)
Tree of tagged strings. Casts to/from `str`. Methods: `push(tag, content)`, `get(tag)`, `to_str()`.

### Blob (Binary Data)
Methods: `len()`, `at(index)`, `slice(start, end)`, `append(other)`.

### Data (Rich Data Components)
For embedded PDFs, images, and other binary content: `Data.get(obj, type)`, `Data.set(obj, type, blob)`.

### Fn (Function Library)
Methods: `call(args...)`, `attributes()`, `has_attribute(name)`, `name()`.

### Age (Duration)
Time duration library for working with age/elapsed time values.

### Fs (File System)
File operations (when enabled): `fs.read(path)`, `fs.write(path, content)`, `fs.exists(path)`.
