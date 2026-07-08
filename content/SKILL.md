---
name: stof
description: "Use this skill whenever the user wants to write, read, understand, debug, or generate Stof (.stof) documents. Stof is a portable data interchange format — JSON + functions — where data carries its own logic. Trigger this skill for: writing .stof files, understanding Stof syntax, converting JSON/YAML/TOML to Stof, designing self-validating configs with schemas, writing prototype types (#[type]), creating workflow pipelines with #[run], embedding logic in data, sending functions over APIs, or any question about Stof fields, functions, attributes, types, unit types, prototypes, schemas, async, imports, or the standard library. Also trigger when the user shows code using `self.`, `#[type]`, `#[main]`, `new X {}`, `schemafy`, prototype type-dispatch syntax, `typename`, or `funcs(attributes = ...)` syntax."
---

# Stof Skill

Stof (Standard Transformation and Organization Format) is a **data format first** — a superset of JSON that adds functions, unit types, semantic versions, attributes, schemas, and prototypes. A Stof document is a portable, sandboxed artifact that carries its own logic.

**Mental model**: A document is a graph of named **nodes** (objects), connected in a DAG. Each node holds **data components** — fields, functions, images, PDFs, or any serializable data. Navigate with dot-path syntax: `self` = current node, `super` = parent, `root` = document root.

## Quick Reference

| Concept | Syntax | Example |
|---------|--------|---------|
| Field | `type name: value` | `str name: "Alice"` |
| Function | `fn name(params) -> ret { }` | `fn greet() -> str { 'hi' }` |
| Prototype | `#[type] Name: { }` | `#[type] Point: { float x: 0 }` |
| Instance | `new TypeName { }` | `new Point { x: 5 }` |
| Attribute | `#[attr]` | `#[main]`, `#[test]`, `#[run(1)]` |
| Import | `import 'path' as target` | `import './data.json' as self.Data` |
| Unit type | `unit name: value` | `cm height: 6ft + 2in` |
| Null-safe | `?expr` / `??` | `?self.func()` / `x ?? "default"` |
| Async | `async fn` / `await` | `const p = async self.work(); await p` |
| Schema | `schemafy(obj)` | `<MyType>.schemafy(data)` |

**Reference docs** (detailed syntax, examples, and API):
- **[Language Fundamentals](references/language-fundamentals.md)** — Fields, types, literals, variables, null handling, control flow, error handling
- **[Functions & Prototypes](references/functions-and-prototypes.md)** — Function declaration, async, attributes, prototypes, schemas
- **[Standard Library](references/standard-library.md)** — Obj, List, Map, Set, Str, Num, Tuple, Ver, Prompt, Std, Time, Http, Blob, Age, Fn, Data libraries
- **[Formats & Patterns](references/formats-and-patterns.md)** — Format I/O, imports, rich data components, `#[run]` workflows, common idioms

---

## Fields

Stof is a superset of JSON — valid JSON is always valid Stof.

```stof
message: "value"             // shorthand (no quotes on key)
str message: "value"         // with explicit type
const str message: "value"   // const (immutable) — throws 'AssignConst' on write
list! alt_ids: []            // not-null type — throws on null assign
```

**String literals:** `'single'`, `"double"`, `` `template: ${expr}` ``, `r#"raw string"#`

**Modifiers:** `#[readonly]` (read anywhere, throws on write), `#[private]` (only visible to owning object). Can be combined with other attributes.

Note: `const` on a local variable does **not** make the field it's later assigned to const — `self.myfield = var` creates a normal mutable field.

**Unit arithmetic in field values:**
```stof
cm height: 6ft + 2in         // unit arithmetic, stored in declared unit
ms ttl: 300s                 // time unit conversion
ver version: 0.5.24          // semantic version literal
```

**Nested objects with functions:**
```stof
server: {
    port: 8080
    address: "localhost"
    fn url() -> str { `https://${self.address}:${self.port}` }
}
```

See [Language Fundamentals](references/language-fundamentals.md) for union/tuple types, path access, and field access modifiers.

---

## Functions

Functions are first-class data, attached to objects like fields, referenced via dot-path. `self` = owning object; `super` = parent; `root` = document root.

```stof
fn add(a: float, b: float) -> float { a + b }

fn greet(name: str = "world") -> str { return `Hello, ${name}!` }

fn create(id: str!, expires?: ms) { }   // optional param with ? suffix
```

**Return rules:** last expression without `;` = implicit return. `return expr;` = explicit. Void functions must not end with a bare expression.

**Arrow functions:**
```stof
fn func: (a: int, b: int) -> int => a + b;
double: (x: int): int => x * 2
let func = (): int => { return 53; };
```

**Named parameters:** `func(b = 30, a = 12)` — keyword arguments in any order.

**Recursion with `this`:** `this` refers to the function itself; `this(n - 1)` = recursive call.

**Async:** Stof is single-threaded but async by default. Any function can `await`. The `async` keyword spawns a new process.

```stof
async fn fetch_data() -> str { 'hello, async' }
const promise = async self.some_fn();   // call anything as async
const results = await [self.fn_a(), self.fn_b()];   // await list of handles
```

See [Functions & Prototypes](references/functions-and-prototypes.md) for arrow function details, async patterns, and special attributes.

---

## Attributes

Attributes attach metadata to fields and functions. Custom string attributes serve as event keys.

```stof
#[main]                        // run on document execution
#[test] / #[test(expected)]    // test function
#[errors]                      // with #[test]: pass if it throws
#[type]                        // marks object as a prototype
#[extends(self.OtherProto)]    // prototype inheritance
#[constructor]                 // run on new TypeName {}
#[dropped]                     // run when object is drop()ed
#[run] / #[run(N)]             // ordered workflow execution via obj.run()
#[readonly] / #[private]       // field access modifiers
#[schema(fn_expr)]             // field-level validation
#[init]                        // run once at end of parsing
#[custom({'key': true})]       // arbitrary metadata
```

Read attributes at runtime: `func.attributes()`, `func.has_attribute('test')`, `this.attributes()`.

---

## Types

**Primitive types:** `bool`, `int`, `float`, `str`, `blob`, `obj`, `fn`, `null`, `unknown`, `void`

**Special types:** `ver` (semantic version), `prompt` (AI prompt tree), time (`ms`, `s`, `min`, `hr`, `days`), length (`m`, `cm`, `km`, `ft`, `in`, `mi`), mass (`g`, `kg`, `lb`), memory (`bytes`, `KB`, `MB`, `GB`, `KiB`, `MiB`, `GiB`), angle (`deg`, `rad`), temperature (`F`, `C`, `K`)

**Collections:** `list`/`vec`, `map`, `set`, `tuple`

**Union types:** `float | str` — field or parameter can hold either. **Not-null:** `str! id: ''` — throws on null assign.

```stof
typeof 54kg      // → 'float'   (primitive type)
typename 54kg    // → 'kg'      (full type name including units/prototype)
value as float   // casting
10kg.to_units('g')   // → 10000 (unit conversion)
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
    fn greeting() -> str { `Hello, ${self.id}` }
}
```

**`<TypeName>` shortcut:** `<Customer>.schemafy(obj)`, `<Point2D>.add(1, 2)`, `<Geometry.Point>` (dot-path).

**Creating instances:**
```stof
// Style A — type before field name (preferred)
const Character aurora: { name: 'Aurora', class: 'Mage' };

// Style B — new keyword
const aurora: new Character { name: 'Aurora' };   // colon syntax at document level
let aurora = new Character { name: 'Aurora' };     // = syntax inside functions
```

**Inheritance:** `#[extends(self.Base)]` or `#[extends('Base')]`. Base constructor always runs first.

**`#[constructor]`** runs on `new TypeName {}`. **`#[dropped]`** runs on `drop(obj)`. **`#[static]`** makes function callable on prototype itself.

See [Functions & Prototypes](references/functions-and-prototypes.md) for inheritance chains, schema validation, and type-dispatch.

---

## Control Flow

```stof
if (cond) { ... } else if (other) { ... } else { ... }
let sign = mod >= 0 ? '+' : '';                        // ternary

for (const user in self.users) { pln(index, user.name); }   // built-in: first, last, index
for (let i in 10) pln(i);       // 0..9
while (cond) { ... }
loop { if (done) break; }
^outer while (i < 10) { while (j < 10) { break ^outer; } }  // labeled loops
```

---

## Null Handling

```stof
str! id: ''                  // not-null type
let x = null ?? "default"   // null-coalescing (?? only skips null, not falsy)
let r = ?self.func_dne()    // null-safe call
let f = self?.a?.b?.c       // optional chaining
```

---

## Error Handling

```stof
try { risky_call(); } catch { /* any error */ }
try throw(42); catch (val: int) res = val;
throw('message');
assert(cond) / assert_eq(a, b) / assert_neq(a, b)
```

---

## Standard Library Essentials

```stof
pln(...) / print(...) / err(...)        // output
typeof val / typename val / str(val)    // type inspection
stringify('json', obj) / parse(s, dest, 'json')   // serialization
copy(val) / nanoid()                    // utility
env("KEY") / set_env("KEY", "val")      // environment
sleep(100ms) / exit()                   // process control
funcs('my_attr')                       // find functions by attribute
```

See [Standard Library reference](references/standard-library.md) for complete Obj, List, Map, Set, Str, Num, Tuple, Ver, Prompt, Time, Http, Blob, Age, Fn, and Data library APIs.

---

## Formats & Interop

| Format | Full fidelity | Use case |
|--------|:---:|---------|
| `stof:human` | Yes | Readable roundtrip with functions, attributes, prototypes |
| `bstf` | Yes | Binary transfer (includes binary data) |
| `json` / `toml` / `yaml` | No | External system interop |
| `text` / `md` | No | Plain text / Markdown content |

```stof
import './config.stof' as self.Config
import './data.json' as self.Data
import './file.pdf'                 // → Data<Pdf> component
stringify('stof:human', obj)        // full fidelity serialize
parse(json_str, dest, 'json')       // deserialize
```

See [Formats & Patterns reference](references/formats-and-patterns.md) for rich data components, import variations, and export/import notes.

---

## `#[run]` Workflow Pattern

```stof
workflow: {
    #[run(1)]
    step_one: { #[run] fn execute() { pln("step 1") } }
    #[run(2)]
    step_two: { #[run] fn execute() { pln("step 2") } }
}
#[main]
fn main() { self.workflow.run() }
```

---

## Common Idioms

```stof
// Guard-and-return
const policy = self.get();
if (policy == null) return;

// Dynamic object insertion with validation
const item = new {} on parent;
try { parse(stof_str, item, 'stof'); } catch { drop(item); return null; }

// Attribute-driven event dispatch
for (const func in funcs(attributes = key)) { func(value); }

// Traverse parent chain
let parent = child.parent();
loop { if (parent == null || parent.instance_of('Target')) break; parent = parent.parent(); }
```

---

## Key Corrections

- **Function return type:** always `-> type` (not `: type`)
- **Assertions are snake_case:** `assert_eq`, `assert_neq`, `assert_not`, `assert_null`
- **`typeof` vs `typename`:** `typeof` gives primitive type, `typename` gives unit/prototype name
- **`shallow = false` on remove:** required to recursively remove a subtree
- **`this` inside a function** refers to the function data itself; `this(...)` = recursion
- **`copy(val)`** deep copies breaking references; bare assignment copies the reference
- **`const` fields enforced at runtime:** assigning throws `'AssignConst'`
- **`#[constructor]` order:** base prototype constructor always runs first
- **`#[schema]` placement:** must be on a specific field, not floating in the prototype body
- **Prompts pass by reference** like collections, not value types
