---
name: stof
description: >
  Use this skill whenever the user wants to write, read, understand, debug, or generate Stof
  (.stof) documents. Stof is a portable data interchange format — JSON + functions — where data
  carries its own logic. Trigger this skill for: writing .stof files, understanding Stof syntax,
  converting JSON/YAML/TOML to Stof, designing self-validating configs with schemas, writing
  prototype types (#[type]), creating workflow pipelines with #[run], embedding logic in data,
  sending functions over APIs, or any question about Stof fields, functions, attributes, types,
  unit types, prototypes, schemas, async, imports, or the standard library. Also trigger when
  the user shows code using `self.`, `#[type]`, `#[main]`, `new X {}`, `schemafy`, prototype
  type-dispatch syntax, `typename`, or `funcs(attributes = ...)` syntax.
---

# Stof Skill

Stof (Standard Transformation and Organization Format) is a **data format first** — a superset
of JSON that adds functions, unit types, semantic versions, attributes, schemas, and prototypes.
A Stof document is a portable, sandboxed artifact that carries its own logic.

**Mental model**: A document is a graph of named **nodes** (objects), connected in a DAG. Each node points to N **data components** — fields, functions, images, PDFs, or any other serializable data. Internally this is a flat list of nodes and a flat list of data with pointers between them, so moving objects around involves no copies. Navigate nodes with dot-path syntax. `self` = current node, `super` = parent node, `root` = document root.

---

## Fields

Stof is a superset of JSON — valid JSON is always valid Stof.

```stof
// All equivalent declaration styles
"message": "value"           // JSON style
message: "value"             // shorthand (no quotes on key)
str message: "value"         // with explicit type
str message: "value",        // optional comma
str message: "value";        // optional semicolon (trailing ok)
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
private_field: 'secret'     // only visible to the object it's defined on (not children, not parents)
```

`#[readonly]` and `#[private]` can be combined with any other attributes.

**`const` at document level:**
```stof
const const_field: 'hello, there'   // same as const inside a function — throws 'AssignConst' on write
```

Note: `const` on a local variable (`const var = 'hello'`) does **not** make the field it's later assigned to const — `self.mydyfield = var` creates a normal mutable field.

**Typed union and tuple fields:**
```stof
bool | int union_field: true         // accepts bool or int; other types are cast to first matching type
(str, ver) tup_field: ('hi', 1.0.0)  // tuple field with explicit element types
```

When assigning a value outside the union, it is cast to the first matching type:
```stof
union_field = 42;     // fine — int is in the union
union_field = false;  // fine — bool is in the union
union_field = 'hi';   // 'hi' is not in union, cast to bool → true
```

**Absolute path access:**
```stof
// Fields and functions are reachable by their full dot-path from root
root.Lang.Fields.MyType.field.push_back('hello');

// <> shorthand also accepts dot-paths
<Fields.MyType>.field.push_back('bob');
```

Field values can be expressions, including unit arithmetic:
```stof
cm height: 6ft + 2in         // unit arithmetic, stored in declared unit
MiB memory: 2GB + 50GiB      // unit conversion
ms ttl: 300s                 // time unit
ms reset_inc: 30days         // days are valid time units
ver version: 0.5.24          // semantic version literal
ver version: 4.5.6-release+build   // semver with pre-release and build metadata
```

Nested objects:
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

## Functions

Functions are first-class data, attached to objects just like fields, and referenced via dot-path.
`self` = the object the function lives on; `super` = parent; `root` = document root.

```stof
// Standard declaration — return type uses ->
fn add(a: float, b: float) -> float {
    a + b   // last expression without ; is the implicit return value
}

// With explicit return
fn greet(name: str = "world") -> str {
    return `Hello, ${name}!`
}

// Optional parameter (? suffix — may be null/absent)
fn create(id: str!, expires?: ms) { }

// Union type parameter
fn allow(value: float | str = 0) -> bool {
    if ((typeof value) == 'str') value = value as float;
    value > 0
}
```

**Return value rules:**
- Last expression **without** `;` = implicit return
- `return expr;` = explicit early return
- A **void** function (no `->`) must not have a bare expression as its last line — add `;` to suppress: `assert(true)` is fine, but `42` alone errors
- A `-> str` function that ends with `42;` (semicolon present) errors — the `;` consumed the value so nothing was returned

```stof
fn no_stmt() -> str { 'hello, world' }         // implicit return ✓
fn ret_stmt() -> str { return 'hello, world'; } // explicit return ✓

fn void_fn() { assert(true) }  // fine — assert returns void

// Errors: void fn, but 42 is a bare expression (implicit return in void fn)
fn not_void() { 42 }

// Errors: -> str declared, but 42; has semicolon so nothing is returned
fn no_ret() -> str { 42; }
```

**Arrow functions:**
```stof
// Stored as a named field using fn keyword
fn func: (a: int, b: int) -> int => a + b;

// Stored as a plain field (no fn keyword)
double: (x: int): int => x * 2
stacked: () -> str => { super.exists }
async_fn: async (): void => {}

// Arrow function as a local variable
let func = (): int => { return 53; };
func()   // → 53

// Arrow functions capture self from their owning object
const created = new {
    visit: (): obj => self;   // self = created
};
created.visit()   // → created

// Immediately-invoked expression
const res = ((a: int): int => a + 42)(10);   // → 52

// .call() on an inline expression
const res = ((a: int): int => a + 42).call(10);
```

**Named parameters (keyword arguments):**
```stof
fn func(a: int, b: int) -> int { a + b }
func(b = 30, a = 12)   // → 42
```

**`this` — the function refers to itself; also used for recursion:**
```stof
fn fibonacci(n: int) -> int {
    if (n <= 1) { n }
    else { this.call(n - 1) + this(n - 2) }
}

fn quicksort(arr: list, low: int = 0, high: int = null) {
    if (high == null) high = arr.len() - 1;
    if (low < high) {
        this(arr, low, idx - 1);
        this(arr, idx + 1, high);
    }
}
```

**Async:**

Stof is **single-threaded but async by default**. Every function can use `await` regardless of
whether it's marked `async` — `await` on a non-promise is a passthrough. The `async` keyword
(and `#[async]` attribute — they're identical) simply spawns a new process.

`Promise<T>` is optional as a type annotation — it matches its inner type, so most code never
needs to write `Promise<str>` explicitly.

```stof
// async keyword and #[async] attribute are identical
async fn this_is_async() -> str { 'hello, async' }
#[async]
fn this_is_also_async() -> str { 'async is actually just an attribute' }

// ANY function can await — not just async ones
fn takes_promise_param(param: Promise<str>) -> str {
    await param   // fine in a non-async fn
}

// await is a passthrough on non-promises
await 'hello'   // → 'hello'
await 42         // → 42

// Call any expression as async — spawns a process and returns a promise
const promise = async self.some_regular_fn();
assert_eq(await promise, 578);

// Async block expression — returns a promise for the block's value
const res = async {
    return await async {
        let v = 7;
        v * 2
    };
};
assert_eq(typeof res, 'Promise<void>');  // unknown inner type = Promise<void>
assert_eq(await res, 14);

// Async block (fire and forget) — creates a new process, no handle needed
async {
    v.push_back('hello');
}

// Await a list of handles — returns a list of results
const results = await [self.fn_a(), self.fn_b()];
assert_eq(results, ['hello, async', 'async is also an attribute']);

// Collect handles and await all (void)
let handles = [];
for (const _ in 10) handles.push_back(async { let res = 10 * 10; });
await handles;

// Promise<T> type annotation — explicit but optional
async fn with_promise() -> Promise<str> { 'hello, promises' }

// Casting promises — cast the promise to change what it resolves to
let promise = (async { return '100'; }) as Promise<str>;
promise = promise as int;       // now resolves to int
assert_eq(await promise, 100);  // '100' cast to int on resolve

// Promise types match their inner type — async fns work where fn is expected
fn takes_fn(pointer: fn) -> int { pointer() }
let res = takes_fn(async (): int => 42);
assert_eq(await res, 42);
```

**Special function attributes:**
```stof
#[main]                  // run on document execution
#[main(42)]              // run with argument
#[init]                  // run once at end of parsing (parse context dropped)
#[test]                  // test function
#[test(expected_value)]  // parameterized test
#[errors]                // with #[test]: pass only if it throws
#[constructor]           // run automatically on new TypeName {}
#[dropped]               // run automatically when the object is drop()ed
#[run] / #[run(N)]       // ordered workflow execution via obj.run()
#[my_custom_attr]        // any string — pure metadata, readable at runtime
```

`#[init]` fires once when the parse context is dropped (end of file/import):
```stof
#[init]
fn initialization() {
    self.initialized = true;
}
```

**Attributes are just metadata — any attribute name is valid.** `#[static]` is a human convention
signalling that a function doesn't use `self`/`super`, but any function can always be called by
its dot-path regardless. Attributes are inspectable at runtime:
```stof
func.attributes()           // → {'custom': 42, 'test': null}
func.has_attribute('test')  // → true
this.attributes()           // read own attributes from inside the function
```

---

## Attributes

Attributes attach metadata to fields and functions. Custom string attributes serve as **event keys**.

```stof
#[main]                        // run on document execution
#[main(42)]                    // run with argument 42
#[test]                        // test function
#[test(expected_value)]        // parameterized test
#[errors]                      // combined with #[test]: pass if it throws
#[type]                        // marks an object as a prototype
#[extends(self.OtherProto)]    // prototype inheritance
#[static]                      // callable on the prototype itself (not instances)
#[no-export]                   // exclude from export/stringify
#[type_ignore]                 // ignore when type-casting
#[readonly]                    // field can be read anywhere, but not written (throws 'FieldReadOnlySet')
#[private]                     // field only visible to the object it's defined on
#[schema(fn_expr)]             // field-level validation function
#[schema_optional]             // field not required for schema validation
#[field]                       // expose a function as a named field
#[run] / #[run(N)]             // ordered workflow execution via obj.run()
#[run({'args': [42]})]         // run with arguments
#[run({'prototype': 'none'|'first'|'last'})]  // control prototype run behavior
#[custom({'key': true})]       // arbitrary metadata
#[my-event-key]                // custom string — used for event dispatch
```

Read a function's own attributes with `this.attributes()`:
```stof
fn my_func() {
    const attrs = this.attributes();  // e.g. {'test': null}
    const has = this.has_attribute('custom');
}
```

---

## Types

**Primitive types:** `bool`, `int`, `float`, `str`, `blob`, `obj`, `fn`, `null`, `unknown`, `void`

**Special types:**
- `ver` — semantic version (`0.5.24`, `4.5.6-release+build`)
- `prompt` — a tree of tagged strings for AI prompts; casts to/from `str`
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
(limit.value as float)   // cast mid-expression
'34GB' as float          // parses to 34000MB
'0xff' as int            // parses hex → 255
```

**Unit conversion:**
```stof
10kg.to_units('g')          // → 10000 (as g)
val.to_units('float')       // clone preserving existing units
val.to_units('int')         // convert type but keep units
val.to_units(50mg)          // use another value's units as target
?val.to_units(stof_units) ?? val   // null-safe conversion
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
let a = list();              // empty list
let a = list(1, 2, 3);
let m = map(('a', 0), ('b', 1));
let s = set();
let s = set(1, 2, 3);

// Raw string (no escape processing)
const stof = r#"
    fn hello() { 'hello' }
"#;

// XML helper
xml('hello, world', 'msg')  // → '<msg>hello, world</msg>'
```

---

## Prototypes (`#[type]`)

Prototypes are named object templates for type-casting, schema validation, and structured creation.

```stof
#[type]
Customer: {
    str! id: '';
    str! plan: '';
    list! refs: [];
    obj! meters: {};
    obj metadata: null;

    fn greeting() -> str {
        `Hello, ${self.id}`
    }
}
```

**`<TypeName>`** — path shortcut to the prototype object. Can be a short name, a dot-path, or a `self.super.`-relative path:
```stof
<Customer>.schemafy(obj)                  // validate obj against Customer schema
<Point2D>.add(1, 2)                       // call a function on the prototype
<Point2D>.field                           // read a static field from the prototype
<Point2D>.field = 'new'                   // set a static field on the prototype
<Geometry.Point>                          // dot-path to disambiguate
<self.super.Geometry.Point2D>.add(1, 2)   // relative path
<Lang.Objects.Types.Geometry.Point>       // full absolute path
```

**Typed fields on a prototype** — if a field is declared with a prototype type, assigning a plain `{}` auto-casts it:
```stof
#[type]
SuperType: {
    SubType sub: new SubType {}    // field has a prototype type
    str msg: ''
}

// When creating an instance, a plain new {} assigned to a typed field gets auto-cast
const o = new SuperType {
    msg: 'hi',
    sub: new { one: 'ONE' }       // auto-cast to SubType, merging defaults
};
assert_eq(typename o.sub, 'SubType');
assert_eq(o.sub.two, 'two')        // default from SubType filled in
```

**`#[constructor]`** — runs automatically when a new instance is created. If the prototype extends another, the base constructor runs first:
```stof
#[type]
Point2D: {
    float x: 0;
    float y: 0;

    #[constructor]
    fn init() {
        self.isapoint = true;       // runs on every new Point2D {}
    }
}

#[type]
#[extends('Point2D')]              // extends by name string also works
Point: {
    float z: 0;

    #[constructor]
    fn init() {
        // Point2D constructor already ran (base-first order)
        self.initialized = true;
    }
}

const p = new Point { x: 1, y: 2, z: 3 };
assert(p.isapoint);       // from Point2D constructor
assert(p.initialized);    // from Point constructor
```

**`#[dropped]`** — runs automatically when the object is dropped:
```stof
#[type]
Point: {
    #[dropped]
    fn on_drop() {
        super.point_dropped = true;   // notify parent before removal
    }
}

drop(point);                          // triggers #[dropped] functions
assert(parent.point_dropped);
```

**Calling a specific prototype's method on an instance** — `method<ProtoName>()`:
```stof
// Call Point's length() override instead of Point2D's
point.length<Point2D>()    // dispatches to Point2D.length() with point as self
```

**`#[extends]`** — prototype inheritance. Accepts an object reference or a name string:
```stof
#[type]
#[extends(self.Point2D)]   // object reference
SubProto: { ... }

#[type]
#[extends('Point2D')]      // name string also works
SubProto: { ... }
```

**`#[static]`** — function callable on the prototype itself (not instances). `self` inside refers to the prototype object:
```stof
#[type]
Helpers: {
    str! last_error: '';

    #[static]
    fn error(msg: str!) -> bool {
        self.last_error = msg;
        false
    }
}
// Usage: <Helpers>.error("oops")
```

**Constructor order with inheritance** — base constructor always runs before subtype constructor:
```stof
#[type]
Base: {
    #[constructor]
    fn init() { self.order = 1; }
}

#[type]
#[extends(self.Base)]
Sub: {
    #[constructor]
    fn init() {
        assert(self.order == 1);   // Base ran first
        self.order = 2;
    }
}
```

**Creating instances:**

There are two equivalent styles for typed instantiation at document/object level:

*Style A — `TypeName fieldname: { ... }` (preferred for clarity):*
```stof
// Declare the type before the field name; the object literal is auto-cast to that prototype.
// Nested typed fields work the same way — no `new` keyword needed anywhere.
roster: {
    const Character aurora: {
        name:  'Aurora',
        class: 'Mage',
        level: 5,
        stats: {
            Stat STR: { name: 'Strength', base: 7  };
            Stat DEX: { name: 'Dexterity', base: 14 };
        };
        Resources resources: {
            max_hp: 48, current_hp: 48,
            max_mp: 120, current_mp: 90,
        };
        // Items in lists can be cast inline with `as Type`
        items: [
            { name: 'Arcane Staff', rarity: 'Rare', power: 3, equipped: true } as Item,
            { name: 'Mana Crystal', rarity: 'Uncommon', power: 0 } as Item,
        ];
    };
}
```

*Style B — `new TypeName { ... }` / `const name: new TypeName { ... }`:*
```stof
// Document-level typed instance — use COLON (:), not equals (=)
const aurora: new Character { name: 'Aurora' };   // ✓ colon syntax at document level
// const aurora = new Character { ... };          // ✗ = syntax does NOT work at document level

// Inside a function, = works fine for local variables:
fn example() {
    const c = new Character { name: 'Test' };     // ✓ = works in function scope
}

// new ... on parent_obj — attach to a specific parent
const customer = new Customer {
    id, plan, refs: refs ?? [],
} on self.customers;
```

Both styles trigger `#[constructor]` and auto-fill prototype defaults. Style A (type-before-name) is generally cleaner for nested documents.

```stof
// Shorthand: variable name as field name (punning)
const var = 42;
const object = new { var };         // → { var: 42 }

// Cast a field's type inline in new {}
const object = new { str non_str }; // non_str is cast to str

// new {} on target — lives under target node
const plan = new {} on self.plans;

// Prototype as typed field on another object
Point pointed_field: { x: 42, y: 42, z: 42 }  // auto-cast to Point

// Pass prototype instances as typed parameters
fn add(first: Point, second: Point) -> Point {
    new Point { x: first.x + second.x; y: first.y + second.y }
}

// Parse then cast
const sub = new {} on self.customers;
parse(stof_string, sub, 'stof');
sub as Customer;

// Inline type cast on variable declaration
const ent: Entitlement = plan.entitlements.get(name);

// Cast inside a list
objects: [
    { value: 'first' } as NestedProto,
    { value: 'second' } as NestedProto,
]

// Programmatic prototype management
obj.create_type('MyType');           // register obj as a named prototype
obj.set_prototype('MyType');         // assign prototype by name
obj.remove_prototype();              // remove prototype relationship
obj.instance_of('MyType');           // check membership (returns bool)
obj.instance_of(proto_obj);          // check against an object reference
obj.prototype();                     // get the prototype object
```

**`new root {}`** — create a new document root node (not a child of anything):
```stof
// Inline syntax at document level — MyRoot is a root, not a field
root MyRoot {
    fn hello() -> str { 'hello' }
}
MyRoot.hello()   // → 'hello'

// Dynamic, guarded creation
if (!AnotherRoot) AnotherRoot = new root { root_obj: true };
assert(AnotherRoot.is_root());
assert_eq(AnotherRoot.parent(), null);
```

**Custom object IDs** — `(id_string)` after the opening brace sets the object's internal ID:
```stof
MyObj: { (my_custom_id)
    field: 42
}
assert_eq(MyObj.id(), 'my_custom_id');

root MyRoot { (custom_root_id)
    fn hello() -> str { 'hello' }
}
assert_eq(MyRoot.id(), 'custom_root_id');
```

---

## Schemas

`#[schema]` validates fields when `schemafy` is called. Receives `target_val` (the field value)
and optionally `target` (the containing object). Multiple schema attributes form a pipeline
(all must pass, short-circuited like `&&`):

> **⚠️ `#[schema]` must be placed on a specific *field*, not on the prototype body itself.**
> A `#[schema]` placed directly inside a `#[type]` block (not above a named field) will cause
> a syntax or runtime error. Each schema attribute decorates the field declared immediately below it:
> ```stof
> #[type]
> Limit: {
>     // ✓ Correct — #[schema] is on the field 'mode'
>     #[schema((v: str): bool => v == 'hard' || v == 'soft')]
>     str! mode: 'hard';
>
>     // ✗ Wrong — #[schema] floating in the prototype body, not on a field and must have type signatures
>     #[schema((v) => v.level >= 1 && v.level <= 20)]
>     // ...and then some other field follows — this is a parse error
> }
> ```

```stof
#[type]
Limit: {
    // Inline lambda
    #[schema((target_val: str): bool => {
        set('hard', 'soft', 'observe').contains(target_val) ? true :
        <LimitrValidation>.error(`Invalid mode: "${target_val}"`)
    })]
    str! mode: 'hard';

    // Two-argument form: target (the object) + target_val (the field value)
    #[schema((target: obj, target_val: float): bool => {
        target_val >= 0 ? true : <LimitrValidation>.error(`Negative amount on ${target.label}`)
    })]
    float! amount: 0;

    // Pipeline: multiple schema attributes applied in order
    #[schema((target_value: unknown): bool => (typeof target_value) == 'str')]
    #[schema((target_value: str): bool => target_value.contains('Dude'))]
    str last: 'Doe';

    // Reference a static helper function
    #[schema(<Helpers>.valid_credit_id)]
    str! credit: '';

    // Optional field: only validated if present (not null)
    #[schema((target_val: obj): bool => <Price>.schemafy(target_val))]
    #[schema_optional]
    Price price: null;
}
```

**Running schema validation:**
```stof
<Limit>.schemafy(my_obj)                                        // returns bool
<Schema>.schemafy(target, remove_invalid = true, remove_undefined = true)  // with options
assert(<Server>.schemafy(self))
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

    let mid = &a[1];
    mid = 42;

    // Swap via references
    swap(&a, &b);
    swap(&arr[i], &arr[j]);

    // copy() — deep copy that breaks reference
    let copy_of_a = copy(a);

    drop a                         // free variable
    drop self.field                // delete field from document
    drop(some_obj)                 // drop object node entirely
    drop('self.field1', 'self.field2')  // drop multiple by path string; returns [bool]
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
obj! meters: {}

// Null-coalescing (??) — returns left side if not null, else right
let x = null ?? "default"              // → "default"
let x = null ?? null ?? null ?? 'hi'  // → 'hi' (chains until non-null)
let x = false ?? 'hi'                 // → false  (?? only skips null, not falsy!)

// ?? with cast
let x = null as kg ?? 43 as g         // → 43000mg

// ? prefix — null-safe call chain (returns null instead of throwing)
const result = ?self.func_dne()        // null if func doesn't exist
const field  = self?.field?.another?.other  // null if any link is null
let result   = ?self.subobj.hello()   // null if subobj or hello() doesn't exist
let result   = ?&self.subobj.hellooo()  // for ref syntax, ? goes first

// Optional chaining with both styles
result = ?self?.subobj?.hello()        // both ? and ?. can be combined

// Ternary — falsy: 0, false, null all take the else branch
let y = x > 5 ? true : false
let x = 'hi' ? 42 : 50               // → 42 (non-empty string is truthy)
let x = 0 ? 'nope' : 'yup'           // → 'yup' (0 is falsy)

// Nested ternary + ?? in branch
let x = 0 ? 30 : true as int ? null ?? 76 : 'man'   // → 76
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

// Single-expression branches (no braces required)
if (x > 5) pln("big");
else pln("small");

// Falsy: 0, false, null are all falsy
if (0) { /* not reached */ }
if (!dev && !sub) { /* both falsy */ }

// Double-negation coerces to bool
assert(!!sub);
assert_eq(str(!!!!56), 'true');
```

> **⚠️ `if` is a statement, not an inline expression.** For inline conditional values, use the
> ternary operator `?:` — NOT `if`:
> ```stof
> // ✓ Correct — ternary for inline values
> let sign = mod >= 0 ? '+' : '';
> let eq   = self.equipped ? ' [EQUIPPED]' : '';
>
> // ✗ Wrong — if cannot be used as an inline value expression
> let sign = if (mod >= 0) { '+' } else { '' };   // syntax error
> ```

### Ternary

```stof
let y = x > 5 ? true : false;

// Nested ternary (right-associative)
let x = 0 ? 30 : true as int ? null ?? 76 : 'man';  // → 76

// ?? inside ternary branch
let val = condition ? null ?? fallback : other;
```

### switch

```stof
// Basic switch — no implicit fallthrough between cases
switch (value) {
    case 'a': pln("got a");
    case 'b': {
        pln("got b");
    }
    default: pln("other");
}

// Multiple values fall into the same case (empty case = fallthrough to next)
switch ('d') {
    case 'a':
    case 'hello': { res = false; }
    case 'd':
    default: { res = true; }   // 'd' and default both run this
}

// switch as an expression — returns a value (no semicolon on return values)
let res = switch ('yo') {
    case 'yo': 42
    case 'other': 100
    default: 500
};    // → 42

// switch on mixed types
switch (value) {
    case null:   result -= 10;
    case true:   result += 10;
    case 42:     result += 22;
    case 'hello': result += 5;
    default:     result -= 100;
}
```

### for-in

```stof
// Iterates values; built-in loop vars: first, last, index
for (const user in self.users) {
    if (first) pln("first:", user.name);
    if (last)  pln("last:", user.name);
    pln(index, user.name);
}

// Cast loop variable on the fly
for (const val: kg in self.iterator) total += val;  // each val cast to kg

// Typed loop variable from prototype
for (const sub: Container in self.subs) {
    const i: str = index + 1;
    text.push(`${i}. ${sub.out()}\n`);
}

// for-in over obj.fields() gives (key, value) pairs
for (const pair in self.plans.fields()) {
    const name = pair[0];
    const plan: Plan = pair[1];
}

// for-in over a number iterates 0..N
for (let i in 10) pln(i);    // 0, 1, ..., 9

// for-in with ref (in-place mutation)
for (let val in &list) val += 2;   // mutates each element

// C-style for loop
for (let j = 0; j < high; j += 1) { ... }

// return from inside for-in
async fn find_last() -> int {
    for (const val in self.iterator) {
        if (last) return val;
    }
    -1
}
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

### Range

```stof
let arr = 0..10|2;   // [0, 2, 4, 6, 8]  (start..end|step)
```

### while / loop

```stof
while (condition) { ... }

loop {
    if (done) break;
}
```

### break / continue / return in loops

```stof
// break / continue work in all loop types
while (i < 200) {
    if (i > 100) break;
    i += 1;
}

for (let i = 0; i < 10; i += 1) {
    if (i < 1) continue;
    // ...
}

// return from inside a loop exits the function
fn loop_ret() -> int {
    let i = 0;
    while (i < 10) {
        if (i > 5) return i;
        i += 1;
    }
    return -1;
}
```

### Tagged break / continue (labeled loops)

Use `^label` to break or continue an outer loop from an inner one:

```stof
^outer while (i < 10) {
    let j = 0;
    while (j < 10) {
        if (i >= 5 && j >= 5) break ^outer;   // exits the outer loop
        j += 1;
    }
    i += 1;
}

^outer while (i < 10) {
    i += 1;
    let j = 0;
    while (j < 10) {
        if (j > 5) continue ^outer;   // skips to next outer iteration
        j += 1;
        total += i + j;
    }
}

// Also works on for-in loops
^loop for (const val in self.iterator) {
    for (let i = 10; i > 0; i -= 1) {
        if (i < 4) { res = true; break ^loop; }
    }
}
```

---

## Error Handling

```stof
// try/catch — catch any error
try { risky_call(); }
catch { /* any error, no value */ }

// Catch a typed value
try throw(42);
catch (val: int) res = val;

try throw({42, 78, 'hi'});
catch (error: unknown) {
    assert_eq(error, {42, 78, 'hi'});
}

// Single-expression try/catch (no braces)
try assert(false);
catch { caught = true; }

// try as an expression — the value of the taken branch is the result
const func = (): int => {
    try { 42 }
    catch { 72 }
};   // → 42

// Nested try/catch — rethrow by throwing in the catch block
try {
    try { throw('hello') }
    catch (e: str) { throw(e + ', world'); }
} catch (e: str) {
    caught = e;   // → 'hello, world'
}

// return from inside try works normally
const func = (): int => {
    try { return 42; }
    catch { return 100; }
};

// throw any value — not just strings
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

---

## Object Operations (Obj library)

```stof
// Navigation
self.name()                         // name of this object ("MyObj")
self.path()                         // full path ("root.Parent.MyObj")
self.id()                           // unique internal ID string
self.parent()                       // parent object
self.root()                         // document root
self.is_root()                      // bool
self.children()                     // list of child objects
self.dist(other_obj)                // distance in the DAG
self.is_parent(child_obj)           // bool — is self a direct/indirect parent of child?
Obj.is_parent(parent, child)        // static form
self.exists()                       // bool — does this node still exist in the graph?

// Move (reparent)
obj.move(new_parent)                // reparent obj under new_parent (pointer update, no copy)

// Fields / functions
self.fields()                       // list of (key, value) pairs
self.funcs()                        // list of fn values on this obj
self.funcs('my_attr')               // list of fns with matching attribute
self.contains('field_name')         // bool
self.len()                          // number of fields
self.empty() / self.any()

// Insert / remove
self.insert('key', value)           // insert field by name
self.insert('sub.nested.key', val)  // creates sub-objects as needed
self.remove('key')                  // remove field (shallow by default)
self.remove('key', shallow = false) // remove field + its subtree

// Move / rename fields
self.move_field('old_name', 'new_name')         // rename field
self.move_field('self.src.field', 'dest.moved') // move between objects

// Attributes
self.attributes()                   // map of attributes on this object
self.attributes('field_name')       // map of attributes on a specific field

// Convert
self.to_map()                       // obj → map (field values, string keys only)
Obj.from_map(map)                   // map → obj
Obj.from_id(id_str)                 // look up object by internal ID

// Prototype management
self.create_type('TypeName')
self.set_prototype('TypeName')
self.remove_prototype()
self.instance_of('TypeName') / self.instance_of(proto_obj)
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
list[idx]                   // index access
&list.front()               // mutable reference

// Modification
list.push_back(item)
list.push_front(item)
list.pop_back()             // removes and returns last
list.pop_front()
list.append(other)          // extend with another list
list.insert(idx, item)
list.replace(idx, item)
list.remove(idx)            // returns removed item
list.remove_first(val)      // remove first occurrence of value
list.remove_last(val)
list.remove_all(val)        // remove all occurrences; returns bool
list.clear()
list.reverse()
list.reversed()             // returns reversed copy

// Query
list.contains(val)
list.index_of(val)          // returns -1 if not found
list.empty() / list.any()
list.is_uniform()           // true if all elements are same type

// Sort
list.sort()
list.sort_by((a, b): int => ...)   // custom comparator; return -1/0/1

// Other
list.join(', ')             // join to string
list.to_uniform('kg')       // convert all elements to target units
```

---

## Map Operations

```stof
// Maps use {'key': value} syntax
let m = {'a': 0, 'b': 1};
m.get('key')
m.insert('key', val)        // returns old value if replaced
m.remove('key')             // returns removed value
m.contains('key')
m.keys()                    // returns a set
m.values()                  // returns a list
m.len()
m.empty() / m.any()
m.first() / m.last()        // returns (key, value) tuple
m.at(idx)                   // returns (key, value) tuple
m.pop_first() / m.pop_last()
m.append(other)
m.clear()
// Iterate as (key, value) pairs:
for (const pair in m) { pair[0]; pair[1]; }
// Or: for (const pair in obj.fields()) { ... }
```

---

## Set Operations

```stof
// Sets use {val1, val2} syntax (curly, no colons)
let s = {1, 2, 3};
s.insert(val)               // returns true if newly inserted
s.remove(val)               // returns removed value
s.contains(val)
s.len()
s.empty() / s.any()
s.first() / s.last()        // sets are ordered
s.at(idx)
s.pop_first() / s.pop_last()
s.append(other)
s.clear()
s.split(val)                // returns (before, after) tuple
s.union(other)
s.difference(other)
s.intersection(other)
s.symmetric_difference(other)
s.disjoint(other)           // true if no overlap
s.subset(other) / s.superset(other)
s.is_uniform()
s.to_uniform('kg')
```

---

## String Operations

```stof
str.len()
str.at(idx)                 // character at index
str.first() / str.last()
str.push("append")
str.contains("sub")
str.starts_with("prefix")
str.ends_with("suffix")
str.index_of("sub")         // -1 if not found
str.replace("old", "new")   // returns new string, original unchanged
str.split(".")              // returns list
str.upper() / str.lower()
str.trim() / str.trim_start() / str.trim_end()
str.substring()             // clone
str.substring(start)        // from index to end
str.substring(start, end)   // slice [start, end)
str.matches(regex)          // bool — does it match the regex?
str.find_matches(regex)     // list of (match, start, end) tuples
// String can be iterated char by char:
for (const c in my_str) { ... }
```

> **⚠️ No `str.repeat()`** — the String library does NOT have a `repeat` method. To repeat a
> character or string N times, write a manual loop or a helper function:
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
Num.sqrt(x)                 // static form (also works as method)
val.floor() / val.ceil() / val.trunc() / val.fract()
val.signum()                // 1 or -1
val.exp() / val.exp2()
val.ln() / val.log()
val.pow(n)
val.round(decimals)
Num.atan2(y, x)

val.to_string()
val.hex() / val.oct() / val.bin()

// Units
val.has_units()             // bool
val.is_angle() / val.is_temp() / val.is_length() / val.is_mass() / val.is_time() / val.is_memory()
val.remove_units()          // strips units, leaves float
val.to_units('g')           // converts between compatible units
val.to_units('float')       // preserves units but strips type wrapper
val.to_units('int')         // converts to int (keep units)

// Numbers are iterable: for (const i in 10) → 0..9
// val.len() → val itself (as int); val.at(idx) → min(idx, val)
```

---

## Tuple Operations

```stof
let t = (32, 43, true, 'hi');
t.len()
t[0] / t.at(0)              // index access
let inner = &t[1][0];       // mutable reference into nested tuple
inner = 'changed';
```

---

## Semantic Version (`ver`) Operations

```stof
let v = 4.5.6-release+build;
v.major() / v.minor() / v.patch()
v.release()    // → 'release'
v.build()      // → 'build'
v.set_major(n) / v.set_minor(n) / v.set_patch(n)
v.set_release('str') / v.set_build('str')
v.clear_release() / v.clear_build()
```

---

## Prompt Type

`prompt` is a tree of tagged strings, ideal for structured AI prompt management.
It passes by reference (like a collection), and casts to/from `str`.

```stof
// Creation
const p = prompt();                      // empty
const p: prompt = 'hello';              // from string
const p = prompt('hello', 'greet');     // text + tag → <greet>hello</greet>
const p = prompt(tag = 'outer');        // tag only
const p = prompt('', 'outer',          // nested tree
    prompt('first', 'a'),
    prompt('second', 'b')
);

// As string
p as str                                // → '<greet>hello</greet>'
const s: str = p;                       // cast to str

// Mutation
p.push('more text')                     // append text
p.push('text', 'tag')                   // append tagged sub-prompt
p.push(other_prompt, 'tag')             // append prompt with tag override
p += prompt('more', 'tag')              // += works
p.pop()                                 // remove and return last
p.insert(idx, 'text')
p.replace(idx, 'text')
p.remove(idx)
p.reverse()
p.clear()                               // remove all children (keeps tag)

// Inspection
p.str()                                 // same as p as str
p.text()                                // raw text content (no tags)
p.tag()                                 // the tag string
p.prompts()                             // list of sub-prompts
p.len()                                 // number of sub-prompts
p.any() / p.empty()
p[idx]                                  // get sub-prompt by index
p.set_text('new text')
p.set_tag('new_tag')
```

---

## Standard Library (`Std` / global)

All std functions work without a prefix (or with `Std.`):

```stof
// Output
pln(...)                    // print with newline
print(...)                  // print without newline
err(...)                    // print to stderr
dbg(...)                    // debug print

// Asserts (snake_case — current standard)
assert(cond)
assert_eq(a, b)
assert_neq(a, b)
assert_not(cond)
assert_null(val)

// Type inspection
typeof val                  // underlying primitive type name
typename val                // full type name (unit type or prototype name)
str(val)                    // cast any value to string (same as val as str)

// Serialization
stringify(format, obj)      // serialize to string ('json', 'toml', 'yaml', 'stof', 'stof:human', 'text', 'md', 'urlencoded', 'bytes')
blobify(format, obj)        // serialize to binary blob ('bstf', 'bytes')
parse(str_or_blob, obj, format)  // deserialize into object
parse(str)                  // parse into current context
format('stof')              // bool — is this format available?
Std.formats()               // set of available format names
format_content_type('stof') // → 'application/stof'

// Object creation
new {}
new TypeName { field: val } on parent
copy(val)                   // deep copy

// IDs
nanoid()                    // 21-char unique ID
nanoid(14)                  // N-char unique ID
Std.graph_id()              // internal graph ID

// Swap
swap(&a, &b)                // swap two values by reference
swap(a, b)                  // swap containers (no & needed for collections)

// Environment
env("KEY")                  // get env var
set_env("KEY", "val")       // set env var
remove_env("KEY")
env_vars()                  // map of all env vars

// Process control
sleep(100ms)
exit()                      // exit current process
exit(handle1, handle2)      // exit other async processes

// Libraries & formats
lib('Http')                 // bool — is this library available?
libs()                      // set of available library names

// Functions by attribute
funcs('my_attr')            // list of all fns in doc with matching attribute
funcs(attributes = key)     // same, key can be str/list/set

// Other
min(a, b, ...) / max(a, b, ...)
xml('text', 'tag')          // wrap text in XML tag → '<tag>text</tag>'
Std.callstack()             // list of currently executing functions
```

---

## Md Library

The `Md` library provides utilities for working with Markdown content beyond basic format import/export.

```stof
Md.html(markdown_str)   // convert Markdown to HTML string
Md.json(markdown_str)   // convert Markdown to AST as JSON string (mdast format)
```

---

## Time Library

```stof
Time.now()                  // current timestamp as ms
Time.now_ns()               // current timestamp as ns
Time.diff(start_ms)         // elapsed ms since start
Time.diff_ns(start_ns)      // elapsed ns
Time.sleep(20ms)            // same as global sleep()

// RFC formatting
Time.now_rfc3339()          // current time as RFC 3339 string
Time.from_rfc3339(str)      // parse RFC 3339 → ms timestamp
Time.to_rfc3339(ms)         // ms → RFC 3339 string
Time.now_rfc2822()
Time.from_rfc2822(str)
Time.to_rfc2822(ms)

// Arithmetic
time_a + 30days
time_a - time_b             // → ms difference
while ((now - started) > reset_inc) started += reset_inc;
```

---

## Http Library

```stof
if (lib('Http')) {
    const resp = await Http.fetch('https://api.example.com/data');

    Http.success(resp)          // bool — 2xx status
    Http.client_error(resp)     // bool — 4xx
    Http.server_error(resp)     // bool — 5xx
    Http.text(resp)             // response body as string
    Http.size(resp)             // response size (castable to KiB, etc.)
    Http.parse(resp, new {})    // parse response body into an object

    // Parallel requests
    let handles = [];
    for (let i in 10) handles.push_back(Http.fetch(url));
    for (const response in await handles) { ... }
}
```

---

## Blob Library

```stof
let b: blob = 'hello, world'    // assign string as blob
let b = |104, 101, 108, 108, 111|;  // raw byte literal (pipe-delimited)

b.len()                         // byte count
b.size()                        // as memory unit (e.g. 12bytes)
b[idx]                          // byte value at index

b.utf8()                        // blob → UTF-8 string
Blob.from_utf8('hello')         // string → blob

b.base64()                      // blob → base64 string
Blob.from_base64(str)           // base64 → blob

b.url_base64()                  // URL-safe base64
Blob.from_url_base64(str)

// Iterate bytes
for (const byte in b) { ... }
```

---

## Age Encryption Library

```stof
if (lib('Age')) {
    const identity = Age.generate()            // generate key pair
    const public_key = identity.public()       // → str

    // Encrypt
    const bin = Age.blobify(public_key, 'stof', payload_obj)
    // or multiple recipients:
    const bin = Age.blobify([pub1, identity2], 'stof', payload_obj)

    // Decrypt
    const success = Age.parse(identity, bin, dest_obj, 'stof')

    drop(identity)                             // destroy key pair
}
```

---

## Fn Library (Function Introspection)

Functions are values — you can store them, inspect them, call them dynamically, and rebind them.

```stof
// Get a function reference
const func = self.my_function;

// Introspection
func.id()                    // unique ID string
func.name()                  // → 'my_function'
func.data()                  // underlying Data handle
func.params()                // list of (name, type) tuples: [('a', 'int'), ('b', 'int')]
func.return_type()           // → 'int'
func.has_attribute('test')   // bool
func.attributes()            // map of all attributes
func.obj()                   // the primary object this fn is attached to
func.objs()                  // list of all objects this fn is attached to
func.is_async()              // bool

// Calling
func(5, 5)                   // direct call
func.call(5, 6)              // call method
Fn.call(func, 9, 2)          // static call via Fn lib
func.call_expanded([30, 12]) // expand a list as positional args

// Binding — rebind self to a different object
const f = (): str => self.msg ?? 'dne';
f.bind(other_obj);           // now self inside f = other_obj

// this — inside a function, refers to the function itself
fn my_func() {
    pln(this.name());        // → 'my_func'
    pln(this.attributes());  // → {'test': null}
    // this(...) = recursive call
}
```

---

## Data Library (Low-Level Data Handles)

Every field and function in Stof is backed by a `Data` handle — a portable binary artifact.
The `Data` library gives direct access to these handles for advanced use cases like sharing
fields between objects, binary serialization, cache invalidation, and cross-object attachment.

```stof
// Get a data handle
const field_data = Data.field('self.my_field')   // by path string
const func_data  = (self.my_func).data()         // from a function reference
const data       = Data.from_id(func.id())       // by ID string

// Check existence
data.exists()               // bool — is this data still live?
Data.exists(identity)       // static form — works on any value (e.g. Age identity)

// Inspect
data.id()                   // ID string
data.objs()                 // list of objects this data is attached to
Data.id(val)                // static form — get ID of any data value
Data.libname(val)           // get the library name for a Data<Lib> component (e.g. 'Pdf', 'Image')

// Blob serialization — serialize a field/fn to binary and back
const blob = field_data.blob()                   // Data → blob
const new_data = Data.load_blob(blob, dest_obj)  // load blob into dest_obj (returns Data)
// Cast result to the right type:
const func = Data.load_blob(blob, dest) as fn;

// Attach / move / drop
data.attach(other_obj)       // attach this data to another object (now on both)
data.drop()                  // remove data from all objects (deletes it)
data.drop_from(obj)          // remove data from one specific object only
data.move(from_obj, to_obj)  // move attachment: detach from_obj, attach to_obj

// Invalidation / validation — for cache-invalidation patterns
data.invalidate('cache_key')
data.validate('cache_key')   // returns bool — true if valid
?Data.field('self.x').invalidate('key')  // null-safe (field may not exist)

// Outright inline binary data in a document (field embedded as raw bytes)
// "data" prefix is optional; attaches to current object on parse
data@v1 |20, 0, 0, 0, ...|
```

---

## Formats & Interop

Stof's underlying structure is an **entity-component system over a DAG**: nodes (objects) are
entities, and data components (fields, functions, images, PDFs, etc.) are the components attached
to them. Formats and libraries are just different lenses on that same flat structure — a PDF
format loads binary data as a `Data<Pdf>` component on a node; the `Pdf` library then provides
Stof functions to operate on it. Everything is self-contained and sandboxed. The host (Rust)
implementation registers both formats and libraries via simple traits.

**Format I/O is potentially lossy.** Each format can only represent what its underlying structure supports. The round-trip fidelity depends entirely on the format:

| Format | Functions | Attributes | Unit types | Prototypes | Binary data |
|--------|-----------|------------|------------|------------|-------------|
| `stof` / `stof:human` | ✓ | ✓ | ✓ | ✓ | — |
| `bstf` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `json` | ✗ | ✗ | ✗ | ✗ | — |
| `toml` | ✗ | ✗ | ✗ | ✗ | — |
| `yaml` | ✗ | ✗ | ✗ | ✗ | — |
| `text` / `md` | ✗ | ✗ | ✗ | ✗ | — |
| `urlencoded` | ✗ | ✗ | ✗ | ✗ | — |

Use `stof:human` or `bstf` when full fidelity matters. Use `json`/`toml`/`yaml` for interop with external systems, accepting that functions, attributes, and Stof-specific types won't survive. The exact behaviour of any format is determined by its host implementation — custom formats registered by the host may have different trade-offs.



```stof
// stringify — serialize an object to a string
stringify('json', obj)          // → JSON string
stringify('toml', obj)          // → TOML string
stringify('yaml', obj)          // → YAML string
stringify('stof', obj)          // → compact Stof
stringify('stof:human', obj)    // → human-readable Stof (preserves attributes, types, functions)
stringify('md', obj)            // → Markdown (reads obj.md field)
stringify('text', obj)          // → plain text (reads obj.text field)
stringify('urlencoded', obj)    // → URL-encoded form data (nested = bracket notation)
stringify('bytes', obj)         // → UTF-8 string from obj.bytes blob field

// blobify — serialize to binary blob
blobify('bstf', obj)            // → blob (binary Stof — preserves types, functions, attributes)
blobify('bytes', obj)           // → blob (raw bytes from obj.bytes field)

// parse — deserialize into an existing object
parse(str_or_blob, dest, 'json')
parse(str_or_blob, dest, 'toml')
parse(str_or_blob, dest, 'yaml')
parse(str_or_blob, dest, 'stof')
parse(str_or_blob, dest, 'bstf')
parse(str_or_blob, dest, 'bytes')   // string auto-converts to UTF-8 blob
parse(str_or_blob, dest, 'md')      // stores content in dest.md
parse(str_or_blob, dest, 'text')    // stores content in dest.text
parse(str_or_blob, dest, 'urlencoded') // or 'www-form' (alias)
parse(stof_string)                  // parse into current context (no dest needed)
```

**Format notes:**
- `'stof'` vs `'stof:human'` — compact omits whitespace; `stof:human` is readable and preserves functions, attributes, and prototype definitions. Use `stof:human` for full roundtrips.
- `'bstf'` — binary Stof. Fully roundtrips types, prototypes, functions, and attributes as a blob. Use this for in-memory or over-the-wire transfer where readability isn't needed.
- `'bytes'` — the `obj.bytes` field must be a `blob`. `stringify` UTF-8 decodes it; `blobify` returns it raw. Parsing a string auto-converts via UTF-8.
- `'text'` — reads/writes `obj.text` as a plain string.
- `'md'` — reads/writes `obj.md` as a Markdown string.
- `'urlencoded'` / `'www-form'` — nested objects encode as bracket notation (`sub[val]=42`).

### Imports

```stof
// Path-only (format inferred from file extension)
import './config.stof' as self.Config
import './data.json' as self.Data
import './test.json'               // no alias — parsed into self directly

// Explicit format override
import json './test.json' as self.Imported
import toml './test.toml' as self.Imported
import yaml './test.yaml' as self.Imported
import text './test.json' as self.Raw   // imports raw file text into dest.text
import pkg './path'                    // import a package (@ prefix = stof/ directory)

// Binary / rich formats — loaded as data components
import './file.pdf'                 // → self.pdf as Data<Pdf> (if Pdf lib available, else bytes)
import './image.png'                // → self.image as Data<Image>
import './doc.docx' as self.Doc     // → self.Doc (if docx format available)

// Runtime parse
parse(str_or_blob, target_obj, 'stof')
parse(stof_string)                   // parse into current context
```

### Rich data components (`Data<Lib>`)

When a format loads binary data (images, PDFs, etc.), the result is a `data` value typed as
`Data<LibName>`. These are first-class values that can be stored in fields, copied, dropped, and
operated on via the corresponding library:

```stof
// Check library availability before using
if (lib('Pdf')) {
    assert_eq(typename self.pdf, 'Data<Pdf>');
    assert_eq(Data.libname(self.pdf), 'Pdf');

    const text = self.pdf.extract_text();
    const images = self.pdf.extract_images();  // list of maps with height, width, etc.
}

if (lib('Image')) {
    assert_eq(typename self.image, 'Data<Image>');

    // Data components can be deep-copied
    const clone = copy(self.image);
    assert(Data.exists(clone));

    // Library functions as both methods and static calls
    clone.width()               // → 1200
    Image.width(clone)          // same, static form
    clone.resize(500, 500)      // mutates the data component
    clone.bmp()                 // → blob

    // Construct from blob
    const img = Image.from_blob(bmp_blob);
    drop(img);
    assert_not(Data.exists(img));
}

if (format('docx')) {
    const text = self.Doc.text();   // extract text from Word document
}

// Cast to a typed data handle
let dta: data = self.pdf as Data<Pdf>;
assert_eq(Data.libname(dta), 'Pdf');
```

### Stof export/import notes

- `stringify('stof:human', obj)` + `parse(str, dest, 'stof')` fully roundtrips including functions and prototype definitions.
- Attributes that hold function values (e.g. `#[schema(...)]`) roundtrip correctly — the function is preserved in the exported Stof.
- **Avoid export/import within the same graph** — IDs may collide. Use `copy(obj)` for in-graph duplication instead.
- Package imports: `import pkg './@geo'` resolves `@` to the `stof/` directory.

---

## `#[run]` Workflow Pattern

```stof
workflow: {
    #[run(1)]
    step_one: {
        #[run]
        fn execute() { pln("step 1") }
    }

    #[run(2)]
    step_two: {
        #[run]
        fn execute() { pln("step 2") }
    }

    // Run with arguments
    #[run({'args': [42]})]
    fn setup(val: int) { self.configured = val; }

    // Run on lists works too — each element is run
    #[run]
    list_run: [
        () => { self.done = true; },
        { #[run] fn inner() { super.sub_done = true; } }
    ]
}

#[main]
fn main() {
    self.workflow.run()
}
```

---

## Common Idioms & Patterns

**Guard-and-return:**
```stof
const policy = self.get();
if (policy == null) return;
```

**Null-safe dynamic dispatch:**
```stof
?self.get().valid() ?? false
?App.event_handler(key, val)
```

**Dynamic object insertion:**
```stof
const item = new {} on parent_obj;
try {
    parse(stof_str, item, 'stof');
} catch {
    drop(item);
    return null;
}
const id = item.id ?? nanoid();
item.id = id;
parent_obj.remove(id, shallow = false);
parent_obj.insert(id, item as MyType);
```

**Attribute-driven event dispatch:**
```stof
// Dispatch: find all fns tagged with key, call them
for (const func in funcs(attributes = key)) {
    const count = func.params().len();
    if (count == 1) func(value);
    else if (count < 1) func();
}

// Register handlers with matching attribute
#[my-event]
fn on_event(value: obj) { ... }
```

**Traverse parent chain to find a typed ancestor:**
```stof
fn get_root_policy(child: obj) -> Policy {
    if (child.instance_of('Policy')) return child;
    let parent = child.parent();
    loop {
        if (parent == null) break;
        if (parent.instance_of('Policy')) return parent;
        parent = parent.parent();
    }
    null
}
```

**Block expression for initialization:**
```stof
value = {
    if (condition) "case_a"
    else "case_b"
};
```

**send_tmp pattern (fire event then drop temp object):**
```stof
await <Event>.send_tmp('plan-changed', new {
    previous: prev_plan,
    current: new_plan,
});
// object auto-dropped after send
```

---

## Key Corrections / Things to Know

- **Function return type syntax:** always `-> type`, e.g. `fn url() -> str`. The old `fn url(): str` colon syntax is no longer valid (that was a prior version).
- **All std assertions are snake_case:** `assert_eq`, `assert_neq`, `assert_not`, `assert_null`. The old `assertEq`, `assertNull` forms are outdated.
- **`typeof` vs `typename`:** `typeof 54kg` → `'float'`; `typename 54kg` → `'kg'`. Use `typename` when you need unit type or prototype name.
- **`shallow = false` on remove:** required to recursively remove a subtree, not just the field reference.
- **`move()` reparents nodes:** call after inserting to ensure an object lives under the right parent.
- **Prompts pass by reference:** `prompt` is not a value type — it behaves like a collection.
- **`#[run]` args:** `#[run({'args': [42]})]` passes arguments. `#[run({'prototype': 'none'|'first'|'last'})]` controls prototype `#[run]` priority.
- **`this` inside a function** refers to the function data itself. `this(...)` is how you recurse.
- **`copy(val)`** performs a deep copy breaking references; bare assignment of collections copies the reference.
- **`const` fields are enforced at runtime:** assigning to a `const` field throws `'AssignConst'`. Catch it if you need to handle it: `try obj.hello = 'x'; catch (e: str) { assert_eq(e, 'AssignConst'); }`
- **`#[constructor]` order:** when using `#[extends]`, the base prototype's `#[constructor]` always runs before the subtype's constructor.
- **`#[dropped]` runs on `drop()`:** any functions marked `#[dropped]` on a prototype fire when an instance of that type is dropped. Useful for cleanup or notifying parents.
