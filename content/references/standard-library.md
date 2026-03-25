# Stof Standard Library Reference

Complete reference for all built-in library operations: Obj, List, Map, Set, Str, Num, Tuple, Ver, Prompt, Std, Md, Time, Http, Blob, Age, Fn, and Data.

---

## Object Operations (Obj library)

```stof
// Navigation
self.name()                         // name of this object
self.path()                         // full path ("root.Parent.MyObj")
self.id()                           // unique internal ID string
self.parent()                       // parent object
self.root()                         // document root
self.is_root()                      // bool
self.children()                     // list of child objects
self.dist(other_obj)                // distance in the DAG
self.is_parent(child_obj)           // bool
self.exists()                       // bool — does this node still exist?

// Move (reparent)
obj.move(new_parent)                // pointer update, no copy

// Fields / functions
self.fields()                       // list of (key, value) pairs
self.funcs()                        // list of fn values on this obj
self.funcs('my_attr')               // list of fns with matching attribute
self.contains('field_name')         // bool
self.len()                          // number of fields
self.empty() / self.any()

// Insert / remove
self.insert('key', value)
self.insert('sub.nested.key', val)  // creates sub-objects as needed
self.remove('key')                  // shallow by default
self.remove('key', shallow = false) // remove field + subtree

// Move / rename fields
self.move_field('old_name', 'new_name')
self.move_field('self.src.field', 'dest.moved')

// Attributes
self.attributes()                   // map of attributes on this object
self.attributes('field_name')       // map of attributes on a specific field

// Convert
self.to_map()                       // obj → map
Obj.from_map(map)                   // map → obj
Obj.from_id(id_str)                 // look up by ID

// Prototype management
self.create_type('TypeName')
self.set_prototype('TypeName')
self.remove_prototype()
self.instance_of('TypeName')
self.prototype()

// Object.run()
self.run()                          // run all #[run]-annotated fields/fns
```

---

## List Operations

```stof
// Access
list.len()
list.at(idx)
list.front() / list.back()
list[idx]
&list.front()               // mutable reference

// Modification
list.push_back(item)
list.push_front(item)
list.pop_back() / list.pop_front()
list.append(other)
list.insert(idx, item)
list.replace(idx, item)
list.remove(idx)
list.remove_first(val) / list.remove_last(val)
list.remove_all(val)
list.clear()
list.reverse()
list.reversed()             // returns reversed copy

// Query
list.contains(val)
list.index_of(val)          // returns -1 if not found
list.empty() / list.any()
list.is_uniform()

// Sort
list.sort()
list.sort_by((a, b): int => ...)   // custom comparator; return -1/0/1

// Other
list.join(', ')
list.to_uniform('kg')
```

---

## Map Operations

```stof
let m = {'a': 0, 'b': 1};
m.get('key')
m.insert('key', val)        // returns old value if replaced
m.remove('key')
m.contains('key')
m.keys()                    // returns a set
m.values()                  // returns a list
m.len()
m.empty() / m.any()
m.first() / m.last()        // returns (key, value) tuple
m.at(idx)
m.pop_first() / m.pop_last()
m.append(other)
m.clear()
// Iterate: for (const pair in m) { pair[0]; pair[1]; }
```

---

## Set Operations

```stof
let s = {1, 2, 3};          // curly, no colons
s.insert(val)               // returns true if newly inserted
s.remove(val)
s.contains(val)
s.len()
s.empty() / s.any()
s.first() / s.last()        // sets are ordered
s.at(idx)
s.pop_first() / s.pop_last()
s.append(other)
s.clear()
s.split(val)                // returns (before, after) tuple
s.union(other) / s.difference(other)
s.intersection(other) / s.symmetric_difference(other)
s.disjoint(other)
s.subset(other) / s.superset(other)
s.is_uniform()
s.to_uniform('kg')
```

---

## String Operations

```stof
str.len()
str.at(idx)
str.first() / str.last()
str.push("append")
str.contains("sub")
str.starts_with("prefix") / str.ends_with("suffix")
str.index_of("sub")         // -1 if not found
str.replace("old", "new")   // returns new string
str.split(".")              // returns list
str.upper() / str.lower()
str.trim() / str.trim_start() / str.trim_end()
str.substring(start, end)   // slice [start, end)
str.matches(regex)          // bool
str.find_matches(regex)     // list of (match, start, end) tuples
// Iterable: for (const c in my_str) { ... }
```

> **No `str.repeat()`** — write a manual loop:
> ```stof
> fn repeat(char: str, times: int) -> str {
>     let out = '';
>     for (let _ in times) out += char;
>     out
> }
> ```

---

## Number Operations

```stof
val.abs()
val.sqrt() / val.cbrt()
val.floor() / val.ceil() / val.trunc() / val.fract()
val.signum()
val.exp() / val.exp2()
val.ln() / val.log()
val.pow(n)
val.round(decimals)
Num.atan2(y, x)

val.to_string()
val.hex() / val.oct() / val.bin()

// Units
val.has_units()
val.is_angle() / val.is_temp() / val.is_length() / val.is_mass() / val.is_time() / val.is_memory()
val.remove_units()
val.to_units('g')
```

---

## Tuple Operations

```stof
let t = (32, 43, true, 'hi');
t.len()
t[0] / t.at(0)
let inner = &t[1][0];       // mutable reference into nested tuple
```

---

## Semantic Version (`ver`) Operations

```stof
let v = 4.5.6-release+build;
v.major() / v.minor() / v.patch()
v.release() / v.build()
v.set_major(n) / v.set_minor(n) / v.set_patch(n)
v.set_release('str') / v.set_build('str')
v.clear_release() / v.clear_build()
```

---

## Prompt Type

`prompt` is a tree of tagged strings for structured AI prompt management. Passes by reference (like a collection), casts to/from `str`.

```stof
// Creation
const p = prompt();                      // empty
const p: prompt = 'hello';              // from string
const p = prompt('hello', 'greet');     // text + tag → <greet>hello</greet>
const p = prompt('', 'outer',          // nested tree
    prompt('first', 'a'),
    prompt('second', 'b')
);

// As string
p as str                                // → '<greet>hello</greet>'

// Mutation
p.push('text', 'tag')
p += prompt('more', 'tag')
p.pop() / p.insert(idx, 'text') / p.replace(idx, 'text')
p.remove(idx) / p.reverse() / p.clear()

// Inspection
p.str() / p.text() / p.tag()
p.prompts()                             // list of sub-prompts
p.len() / p.any() / p.empty()
p[idx]
p.set_text('new text') / p.set_tag('new_tag')
```

---

## Standard Library (`Std` / global)

All std functions work without a prefix (or with `Std.`):

```stof
// Output
pln(...) / print(...) / err(...) / dbg(...)

// Asserts (snake_case)
assert(cond) / assert_eq(a, b) / assert_neq(a, b)
assert_not(cond) / assert_null(val)

// Type inspection
typeof val                  // underlying primitive type
typename val                // full type name (unit or prototype)
str(val)                    // cast to string

// Serialization
stringify(format, obj)      // 'json', 'toml', 'yaml', 'stof', 'stof:human', 'text', 'md', 'urlencoded', 'bytes'
blobify(format, obj)        // 'bstf', 'bytes'
parse(str_or_blob, obj, format)
format('stof')              // bool — is format available?
Std.formats()               // set of available formats

// Object creation
new {} / new TypeName { field: val } on parent
copy(val)                   // deep copy

// IDs
nanoid() / nanoid(14)
Std.graph_id()

// Swap
swap(&a, &b)

// Environment
env("KEY") / set_env("KEY", "val") / remove_env("KEY") / env_vars()

// Process control
sleep(100ms) / exit()

// Libraries & formats
lib('Http') / libs()

// Functions by attribute
funcs('my_attr') / funcs(attributes = key)

// Other
min(a, b, ...) / max(a, b, ...)
xml('text', 'tag')
Std.callstack()
```

---

## Md Library

```stof
Md.html(markdown_str)   // convert Markdown to HTML string
Md.json(markdown_str)   // convert Markdown to AST as JSON string (mdast format)
```

---

## Time Library

```stof
Time.now()                  // current timestamp as ms
Time.now_ns()               // as ns
Time.diff(start_ms)         // elapsed ms since start
Time.diff_ns(start_ns)
Time.sleep(20ms)

// RFC formatting
Time.now_rfc3339() / Time.from_rfc3339(str) / Time.to_rfc3339(ms)
Time.now_rfc2822() / Time.from_rfc2822(str) / Time.to_rfc2822(ms)

// Arithmetic
time_a + 30days
time_a - time_b             // → ms difference
```

---

## Http Library

```stof
if (lib('Http')) {
    const resp = await Http.fetch('https://api.example.com/data');
    Http.success(resp)          // bool — 2xx
    Http.client_error(resp)     // bool — 4xx
    Http.server_error(resp)     // bool — 5xx
    Http.text(resp)             // response body as string
    Http.size(resp)             // response size
    Http.parse(resp, new {})    // parse body into object

    // Parallel requests
    let handles = [];
    for (let i in 10) handles.push_back(Http.fetch(url));
    for (const response in await handles) { ... }
}
```

---

## Blob Library

```stof
let b: blob = 'hello, world'
let b = |104, 101, 108, 108, 111|;  // raw byte literal

b.len()                         // byte count
b.size()                        // as memory unit
b[idx]

b.utf8() / Blob.from_utf8('hello')
b.base64() / Blob.from_base64(str)
b.url_base64() / Blob.from_url_base64(str)
// Iterable: for (const byte in b) { ... }
```

---

## Age Encryption Library

```stof
if (lib('Age')) {
    const identity = Age.generate()
    const public_key = identity.public()

    // Encrypt
    const bin = Age.blobify(public_key, 'stof', payload_obj)
    // or: Age.blobify([pub1, identity2], 'stof', payload_obj)

    // Decrypt
    const success = Age.parse(identity, bin, dest_obj, 'stof')

    drop(identity)
}
```

---

## Fn Library (Function Introspection)

```stof
const func = self.my_function;

// Introspection
func.name()                  // → 'my_function'
func.params()                // [('a', 'int'), ('b', 'int')]
func.return_type()           // → 'int'
func.has_attribute('test')
func.attributes()
func.obj()                   // primary object attached to
func.is_async()

// Calling
func(5, 5)
func.call(5, 6)
Fn.call(func, 9, 2)
func.call_expanded([30, 12]) // expand list as positional args

// Binding — rebind self to a different object
func.bind(other_obj);

// this — inside a function, refers to the function itself
fn my_func() {
    pln(this.name());        // → 'my_func'
    this(...)                // recursive call
}
```

---

## Data Library (Low-Level Data Handles)

Every field and function is backed by a `Data` handle — a portable binary artifact.

```stof
// Get a data handle
const field_data = Data.field('self.my_field')
const func_data  = (self.my_func).data()
const data       = Data.from_id(func.id())

// Check / inspect
data.exists()
data.id()
data.objs()                 // objects this data is attached to
Data.libname(val)            // library name for Data<Lib> component

// Blob serialization
const blob = field_data.blob()
const new_data = Data.load_blob(blob, dest_obj)

// Attach / move / drop
data.attach(other_obj)
data.drop() / data.drop_from(obj)
data.move(from_obj, to_obj)

// Invalidation / validation
data.invalidate('cache_key')
data.validate('cache_key')

// Inline binary data in a document
data@v1 |20, 0, 0, 0, ...|
```
