# Stof Language Fundamentals

Detailed reference for fields, types, variables, null handling, control flow, and error handling.

---

## Fields

Stof is a superset of JSON — valid JSON is always valid Stof.

```stof
// Declaration styles
"message": "value"           // JSON style
message: "value"             // shorthand (no quotes on key)
str message: "value"         // with explicit type
const str message: "value"   // const (immutable) field
list! alt_ids: []            // not-null type — throws on null assign
obj metadata: null           // nullable field
```

**String literals:**
```stof
'single quotes'
"double quotes"
`backtick template: ${expr} and ${self.field}`   // string interpolation
r#"raw string — no escape processing, great for embedded Stof/JSON"#
```

**Field access modifiers:**
```stof
#[readonly]
read_only_field: 42         // can be read anywhere, throws 'FieldReadOnlySet' on write

#[private]
private_field: 'secret'     // only visible to the object it's defined on
```

**`const` at document level:**
```stof
const const_field: 'hello, there'   // throws 'AssignConst' on write
```

Note: `const` on a local variable (`const var = 'hello'`) does **not** make the field it's later assigned to const — `self.mydyfield = var` creates a normal mutable field.

**Typed union and tuple fields:**
```stof
bool | int union_field: true         // accepts bool or int; other types cast to first match
(str, ver) tup_field: ('hi', 1.0.0)  // tuple field with explicit element types
```

**Absolute path access:**
```stof
root.Lang.Fields.MyType.field.push_back('hello');
<Fields.MyType>.field.push_back('bob');   // <> shorthand
```

**Unit arithmetic in field values:**
```stof
cm height: 6ft + 2in         // unit arithmetic, stored in declared unit
MiB memory: 2GB + 50GiB      // unit conversion
ms ttl: 300s                 // time unit
ver version: 4.5.6-release+build   // semver with pre-release and build metadata
```

**Nested objects:**
```stof
server: {
    port: 8080
    address: "localhost"

    fn url() -> str {
        `https://${self.address}:${self.port}`
    }
}
```

---

## Types

**Primitive types:** `bool`, `int`, `float`, `str`, `blob`, `obj`, `fn`, `null`, `unknown`, `void`

**Special types:**
- `ver` — semantic version (`0.5.24`, `4.5.6-release+build`)
- `prompt` — tree of tagged strings for AI prompts; casts to/from `str`
- `ms`, `ns` — time durations; also `s`, `min`, `hr`, `days`
- Unit types (length): `m`, `cm`, `km`, `ft`, `in`, `mi`
- Unit types (mass): `g`, `kg`, `mg`, `lb`, `oz`
- Unit types (memory): `bytes`, `KB`, `MB`, `GB`, `TB`, `KiB`, `MiB`, `GiB`, `TiB`
- Unit types (angle): `deg`, `rad`
- Unit types (temperature): `F`, `C`, `K`

**Collections:** `list` / `vec` (array), `map`, `set`, `tuple`

**Union types:** `float | str` — field or parameter can hold either type.

**Not-null modifier (`!`):** `str! id: ''` — throws if null is ever assigned.

**`typeof` vs `typename`:**
```stof
typeof 54kg      // → 'float'   (the underlying primitive type)
typename 54kg    // → 'kg'      (the full type name including units/prototype)
typeof self.obj  // → 'obj'
typename self.obj // → 'MyProto' if the obj has a prototype
```

**Casting:**
```stof
value as float           // cast to float (works with unit strings like '12kg')
my_obj as Server         // cast to prototype
'34GB' as float          // parses to 34000MB
'0xff' as int            // parses hex → 255
```

**Unit conversion:**
```stof
10kg.to_units('g')          // → 10000 (as g)
val.to_units('float')       // clone preserving existing units
val.to_units(50mg)          // use another value's units as target
```

**Blob literal:**
```stof
const msg = |104, 101, 108, 108, 111|;   // pipe-delimited raw bytes
```

---

## Literals & Constructors

```stof
// Collections
let a = [1, 2, 3];           // list
let m = {'a': 0, 'b': 1};    // map
let s = {1, 2, 3};           // set (curly + no colons)
let t = (32, true, 'hi');    // tuple

// Constructor functions
let a = list(1, 2, 3);
let m = map(('a', 0), ('b', 1));
let s = set(1, 2, 3);

// Raw string
const stof = r#"
    fn hello() { 'hello' }
"#;

// XML helper
xml('hello, world', 'msg')  // → '<msg>hello, world</msg>'
```

---

## Variables & References

```stof
fn example() {
    let a = 32                     // implicit type
    let b: int = 43                // explicit type (casts if needed)
    const c = "immutable"

    // References — write-back on assignment (& on both sides)
    let &ref_a = &self.some_field
    ref_a = "new value"            // updates self.some_field

    // Reference to collection element
    let first = &a.front();
    first = 'yo';                  // mutates the list in place

    // Swap via references
    swap(&a, &b);
    swap(&arr[i], &arr[j]);

    // copy() — deep copy that breaks reference
    let copy_of_a = copy(a);

    drop a                         // free variable
    drop self.field                // delete field from document
    drop(some_obj)                 // drop object node entirely
    drop('self.field1', 'self.field2')  // drop multiple by path string
}
```

**In-place mutation using `&` in for-in:**
```stof
for (let i in &self.data) i *= 2    // mutates self.data in place
```

---

## Null & Initialization

```stof
// Not-null types (throw on null assign)
str! id: ''
list! alt_ids: []

// Null-coalescing (??) — returns left side if not null, else right
let x = null ?? "default"              // → "default"
let x = false ?? 'hi'                 // → false  (?? only skips null, not falsy!)

// ? prefix — null-safe call chain (returns null instead of throwing)
const result = ?self.func_dne()        // null if func doesn't exist
const field  = self?.field?.another?.other  // null if any link is null

// Ternary — falsy: 0, false, null all take the else branch
let y = x > 5 ? true : false
let x = 0 ? 'nope' : 'yup'           // → 'yup' (0 is falsy)
```

---

## Control Flow

### if / else

```stof
if (condition) {
    // ...
} else if (other) {
    // ...
} else {
    // ...
}
```

> **`if` is a statement, not an inline expression.** For inline conditional values, use the ternary operator `?:`:
> ```stof
> let sign = mod >= 0 ? '+' : '';   // ✓ ternary
> let sign = if (mod >= 0) { '+' } else { '' };   // ✗ syntax error
> ```

### switch

```stof
// Basic switch — no implicit fallthrough between cases
switch (value) {
    case 'a': pln("got a");
    case 'b': { pln("got b"); }
    default: pln("other");
}

// switch as an expression — returns a value
let res = switch ('yo') {
    case 'yo': 42
    case 'other': 100
    default: 500
};    // → 42
```

### for-in

```stof
// Iterates values; built-in loop vars: first, last, index
for (const user in self.users) {
    if (first) pln("first:", user.name);
    pln(index, user.name);
}

// Cast loop variable on the fly
for (const val: kg in self.iterator) total += val;

// for-in over obj.fields() gives (key, value) pairs
for (const pair in self.plans.fields()) {
    const name = pair[0];
    const plan: Plan = pair[1];
}

// for-in over a number iterates 0..N
for (let i in 10) pln(i);    // 0, 1, ..., 9

// C-style for loop
for (let j = 0; j < high; j += 1) { ... }
```

### Custom iterators

Any object with `len()` and `at(index)` methods can be iterated with for-in:

```stof
iterator: {
    len: (): int => 10
    at: (index: int): int => index
}
for (let x in self.iterator) { pln(x); }   // 0..9
```

### Range, while, loop

```stof
let arr = 0..10|2;   // [0, 2, 4, 6, 8]  (start..end|step)

while (condition) { ... }

loop {
    if (done) break;
}
```

### Tagged break / continue (labeled loops)

```stof
^outer while (i < 10) {
    let j = 0;
    while (j < 10) {
        if (i >= 5 && j >= 5) break ^outer;   // exits the outer loop
        j += 1;
    }
    i += 1;
}
```

---

## Error Handling

```stof
// try/catch
try { risky_call(); }
catch { /* any error */ }

// Catch a typed value
try throw(42);
catch (val: int) res = val;

// try as an expression
const func = (): int => {
    try { 42 }
    catch { 72 }
};   // → 42

// throw any value
throw('message');
throw(42);
throw({42, 78, 'hi'});

// Assertions
assert(condition)
assert_eq(a, b)
assert_neq(a, b)
assert_not(condition)
assert_null(val)
```
