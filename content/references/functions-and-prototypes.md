# Stof Functions, Attributes, Prototypes & Schemas

Detailed reference for function declaration, async patterns, attributes, prototype types, and schema validation.

---

## Functions

Functions are first-class data, attached to objects just like fields, and referenced via dot-path. `self` = the object the function lives on; `super` = parent; `root` = document root.

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

**Return value rules:**
- Last expression **without** `;` = implicit return
- `return expr;` = explicit early return
- A **void** function (no `->`) must not have a bare expression as its last line — add `;` to suppress
- A `-> str` function that ends with `42;` (semicolon present) errors — the `;` consumed the value

```stof
fn no_stmt() -> str { 'hello, world' }         // implicit return ✓
fn ret_stmt() -> str { return 'hello, world'; } // explicit return ✓
fn void_fn() { assert(true) }                   // fine — assert returns void
fn not_void() { 42 }                            // ✗ error: bare expression in void fn
fn no_ret() -> str { 42; }                      // ✗ error: semicolon consumed return value
```

**Arrow functions:**
```stof
fn func: (a: int, b: int) -> int => a + b;     // named field
double: (x: int): int => x * 2                  // plain field
async_fn: async (): void => {}                   // async arrow

let func = (): int => { return 53; };            // local variable
func()   // → 53

// Arrow functions capture self from their owning object
const created = new { visit: (): obj => self; };
created.visit()   // → created

// Immediately-invoked expression
const res = ((a: int): int => a + 42)(10);   // → 52
```

**Named parameters (keyword arguments):**
```stof
fn func(a: int, b: int) -> int { a + b }
func(b = 30, a = 12)   // → 42
```

**`this` — the function refers to itself; used for recursion:**
```stof
fn fibonacci(n: int) -> int {
    if (n <= 1) { n }
    else { this.call(n - 1) + this(n - 2) }
}
```

### Async

Stof is **single-threaded but async by default**. Every function can use `await` regardless of whether it's marked `async` — `await` on a non-promise is a passthrough. The `async` keyword (and `#[async]` attribute — they're identical) spawns a new process.

```stof
async fn this_is_async() -> str { 'hello, async' }

// ANY function can await — not just async ones
fn takes_promise_param(param: Promise<str>) -> str {
    await param   // fine in a non-async fn
}

// await is a passthrough on non-promises
await 'hello'   // → 'hello'

// Call any expression as async — spawns a process and returns a promise
const promise = async self.some_regular_fn();
assert_eq(await promise, 578);

// Async block expression
const res = async { let v = 7; v * 2 };
assert_eq(await res, 14);

// Await a list of handles
const results = await [self.fn_a(), self.fn_b()];

// Casting promises
let promise = (async { return '100'; }) as Promise<str>;
promise = promise as int;       // now resolves to int
assert_eq(await promise, 100);
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
#[init]                        // run once at end of parsing
#[type]                        // marks an object as a prototype
#[extends(self.OtherProto)]    // prototype inheritance
#[static]                      // callable on the prototype itself (not instances)
#[constructor]                 // run automatically on new TypeName {}
#[dropped]                     // run automatically when the object is drop()ed
#[no-export]                   // exclude from export/stringify
#[type_ignore]                 // ignore when type-casting
#[readonly]                    // field can be read anywhere, but not written
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

**Any attribute name is valid** — `#[static]` is a human convention. Attributes are inspectable at runtime:
```stof
func.attributes()           // → {'custom': 42, 'test': null}
func.has_attribute('test')  // → true
this.attributes()           // read own attributes from inside the function
```

`#[init]` fires once when the parse context is dropped (end of file/import):
```stof
#[init]
fn initialization() { self.initialized = true; }
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
    obj metadata: null;

    fn greeting() -> str { `Hello, ${self.id}` }
}
```

**`<TypeName>`** — path shortcut to the prototype object:
```stof
<Customer>.schemafy(obj)                  // validate against schema
<Point2D>.add(1, 2)                       // call function on prototype
<Geometry.Point>                          // dot-path to disambiguate
<self.super.Geometry.Point2D>.add(1, 2)   // relative path
```

**Typed fields on a prototype** — assigning a plain `{}` auto-casts it:
```stof
#[type]
SuperType: {
    SubType sub: new SubType {}    // field has a prototype type
}

const o = new SuperType { sub: new { one: 'ONE' } };
assert_eq(typename o.sub, 'SubType');   // auto-cast
```

**`#[constructor]`** — runs automatically on instance creation. Base constructor runs first with inheritance:
```stof
#[type]
Point2D: {
    float x: 0; float y: 0;
    #[constructor]
    fn init() { self.isapoint = true; }
}

#[type]
#[extends('Point2D')]
Point: {
    float z: 0;
    #[constructor]
    fn init() { self.initialized = true; }  // Point2D init already ran
}
```

**`#[dropped]`** — runs automatically when the object is dropped:
```stof
#[type]
Point: {
    #[dropped]
    fn on_drop() { super.point_dropped = true; }
}
drop(point);   // triggers #[dropped]
```

**`#[extends]`** — prototype inheritance:
```stof
#[type]
#[extends(self.Point2D)]   // object reference
SubProto: { ... }

#[type]
#[extends('Point2D')]      // name string also works
SubProto: { ... }
```

**`#[static]`** — function callable on the prototype itself:
```stof
#[type]
Helpers: {
    str! last_error: '';
    #[static]
    fn error(msg: str!) -> bool { self.last_error = msg; false }
}
// Usage: <Helpers>.error("oops")
```

**Calling a specific prototype's method on an instance:**
```stof
point.length<Point2D>()    // dispatches to Point2D.length()
```

### Creating instances

**Style A — `TypeName fieldname: { ... }` (preferred for clarity):**
```stof
roster: {
    const Character aurora: {
        name: 'Aurora', class: 'Mage', level: 5,
        Resources resources: { max_hp: 48, current_hp: 48 };
        items: [
            { name: 'Arcane Staff', rarity: 'Rare' } as Item,
        ];
    };
}
```

**Style B — `new TypeName { ... }`:**
```stof
// Document-level — use COLON (:), not equals (=)
const aurora: new Character { name: 'Aurora' };   // ✓

// Inside a function, = works fine
fn example() {
    const c = new Character { name: 'Test' };     // ✓
}

// new ... on parent_obj — attach to a specific parent
const customer = new Customer { id, plan } on self.customers;
```

**Other creation patterns:**
```stof
const object = new { var };         // punning: { var: 42 }
const plan = new {} on self.plans;  // create under target node
new root MyRoot { fn hello() -> str { 'hello' } }  // new root node

// Programmatic prototype management
obj.create_type('MyType');
obj.set_prototype('MyType');
obj.instance_of('MyType');
obj.prototype();
```

**Custom object IDs:**
```stof
MyObj: { (my_custom_id) field: 42 }
assert_eq(MyObj.id(), 'my_custom_id');
```

---

## Schemas

`#[schema]` validates fields when `schemafy` is called. Receives `target_val` (the field value) and optionally `target` (the containing object). Multiple schema attributes form a pipeline (all must pass).

> **`#[schema]` must be placed on a specific *field*, not on the prototype body itself.**

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
        target_val >= 0 ? true : <LimitrValidation>.error(`Negative amount`)
    })]
    float! amount: 0;

    // Pipeline: multiple schema attributes
    #[schema((target_value: unknown): bool => (typeof target_value) == 'str')]
    #[schema((target_value: str): bool => target_value.contains('Dude'))]
    str last: 'Doe';

    // Reference a static helper
    #[schema(<Helpers>.valid_credit_id)]
    str! credit: '';

    // Optional field: only validated if present
    #[schema((target_val: obj): bool => <Price>.schemafy(target_val))]
    #[schema_optional]
    Price price: null;
}
```

**Running schema validation:**
```stof
<Limit>.schemafy(my_obj)                                        // returns bool
<Schema>.schemafy(target, remove_invalid = true, remove_undefined = true)
```
