# Functions & Prototypes Reference

Detailed reference for function declaration, async patterns, attributes, prototypes, and schemas in Stof.

---

## Function Declaration

Functions are first-class data attached to objects, referenced via dot-path. `self` = owning object, `super` = parent, `root` = document root.

```stof
// Standard declaration — return type uses ->
fn add(a: float, b: float) -> float {
    a + b   // last expression without ; is the implicit return value
}

// With explicit return and default parameter
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

### Return Value Rules

- Last expression **without** `;` = implicit return
- `return expr;` = explicit early return
- A **void** function (no `->`) must not have a bare expression as its last line — add `;` to suppress
- A `-> str` function that ends with `42;` (semicolon present) errors — the `;` consumed the value

```stof
fn no_stmt() -> str { 'hello, world' }         // implicit return ✓
fn ret_stmt() -> str { return 'hello, world'; } // explicit return ✓
fn void_fn() { assert(true) }                   // fine — assert returns void
// fn not_void() { 42 }                         // ERROR: bare expression in void fn
// fn no_ret() -> str { 42; }                   // ERROR: semicolon consumed value
```

### Arrow Functions

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
```

### Named Parameters (Keyword Arguments)

```stof
fn func(a: int, b: int) -> int { a + b }
func(b = 30, a = 12)   // → 42
```

### `this` — Self-Reference and Recursion

`this` inside a function refers to the function data itself. `this(...)` = recursive call.

```stof
fn fibonacci(n: int) -> int {
    if (n <= 1) { n }
    else { this.call(n - 1) + this(n - 2) }
}
```

---

## Async

Stof is **single-threaded but async by default**. Every function can use `await` regardless of whether it's marked `async` — `await` on a non-promise is a passthrough. The `async` keyword (and `#[async]` attribute — they're identical) spawns a new process.

`Promise<T>` is optional as a type annotation — it matches its inner type.

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

// Call any expression as async — spawns a process and returns a promise
const promise = async self.some_regular_fn();
assert_eq(await promise, 578);

// Async block expression — returns a promise for the block's value
const res = async { let v = 7; v * 2 };
assert_eq(await res, 14);

// Await a list of handles — returns a list of results
const results = await [self.fn_a(), self.fn_b()];

// Casting promises — cast the promise to change what it resolves to
let promise = (async { return '100'; }) as Promise<str>;
promise = promise as int;
assert_eq(await promise, 100);   // '100' cast to int on resolve
```

---

## Special Function Attributes

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

**Attributes are just metadata — any attribute name is valid.** `#[static]` is a human convention signalling that a function doesn't use `self`/`super`. Attributes are inspectable at runtime:
```stof
func.attributes()           // → {'custom': 42, 'test': null}
func.has_attribute('test')  // → true
this.attributes()           // read own attributes from inside the function
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

### `<TypeName>` Path Shortcut

Can be a short name, dot-path, or `self.super.`-relative path:
```stof
<Customer>.schemafy(obj)                  // validate obj against Customer schema
<Point2D>.add(1, 2)                       // call a function on the prototype
<Geometry.Point>                          // dot-path to disambiguate
<self.super.Geometry.Point2D>.add(1, 2)   // relative path
```

### Typed Fields on Prototypes

If a field is declared with a prototype type, assigning a plain `{}` auto-casts it:
```stof
#[type]
SuperType: {
    SubType sub: new SubType {}    // field has a prototype type
    str msg: ''
}

const o = new SuperType {
    msg: 'hi',
    sub: new { one: 'ONE' }       // auto-cast to SubType, merging defaults
};
assert_eq(typename o.sub, 'SubType');
```

### Constructor and Dropped

**`#[constructor]`** runs automatically when a new instance is created. With inheritance, base constructor runs first:
```stof
#[type]
Point2D: {
    float x: 0; float y: 0;
    #[constructor]
    fn init() { self.isapoint = true; }
}

#[type]
#[extends(self.Point2D)]
Point: {
    float z: 0;
    #[constructor]
    fn init() {
        assert(self.isapoint);   // Point2D constructor already ran
        self.initialized = true;
    }
}
```

**`#[dropped]`** runs automatically when the object is dropped:
```stof
#[type]
Point: {
    #[dropped]
    fn on_drop() { super.point_dropped = true; }
}
drop(point);   // triggers #[dropped] functions
```

### Inheritance (`#[extends]`)

Accepts an object reference or a name string:
```stof
#[type]
#[extends(self.Point2D)]   // object reference
SubProto: { ... }

#[type]
#[extends('Point2D')]      // name string also works
SubProto: { ... }
```

### `#[static]` Functions

Callable on the prototype itself (not instances). `self` inside refers to the prototype object:
```stof
#[type]
Helpers: {
    str! last_error: '';
    #[static]
    fn error(msg: str!) -> bool { self.last_error = msg; false }
}
// Usage: <Helpers>.error("oops")
```

### Type-Dispatch

Call a specific prototype's method on an instance:
```stof
point.length<Point2D>()    // dispatches to Point2D.length() with point as self
```

### Creating Instances

Two equivalent styles for typed instantiation:

**Style A — `TypeName fieldname: { }` (preferred):**
```stof
roster: {
    const Character aurora: {
        name: 'Aurora',
        class: 'Mage',
        level: 5,
        stats: {
            Stat STR: { name: 'Strength', base: 7  };
            Stat DEX: { name: 'Dexterity', base: 14 };
        };
    };
}
```

**Style B — `new TypeName { }` / `const name: new TypeName { }`:**
```stof
const aurora: new Character { name: 'Aurora' };   // colon at document level
let aurora = new Character { name: 'Aurora' };     // = syntax inside functions
```

**Important:** at document level, use `:` (colon), not `=` (equals) for `new` assignments.

### Schema Validation

```stof
#[type]
Config: {
    str! name: '';
    #[schema((v) => v > 0 && v < 65536)]
    int! port: 8080;
    #[schema_optional]
    str description: null;
}

// Validate an object against the schema
<Config>.schemafy(my_obj);   // throws if validation fails
```
