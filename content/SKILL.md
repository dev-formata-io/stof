---
name: stof
description: >
  Write, read, debug, and embed Stof (.stof) documents. Stof is a data runtime: a superset of JSON where
  documents carry their own functions, types, units, schemas, and tests, and run sandboxed anywhere
  (Rust, WebAssembly/TypeScript, Python). Use for any .stof file, Stof syntax or error question,
  converting JSON/YAML/TOML to Stof, self-validating configs (#[schema]), prototype types (#[type]),
  #[main]/#[test]/#[run] functions, the `stof` CLI, or embedding Stof with StofDoc (JS), Doc (Python),
  or Graph/Runtime (Rust). Also use when code shows `self.`, `#[type]`, `new X {}`, `schemafy`,
  `Lib::func`, `funcs(attributes = ...)`, or `stof test`.
---

# Stof

Stof is a **data runtime**: a document is data plus the logic that belongs with it. Every JSON document is
already valid Stof. On top of JSON, Stof adds functions, types, units (`5km`, `2GB`, `30s`), semantic versions,
attributes, prototypes, schemas, imports, and tests. Documents run sandboxed: they can only see themselves and the
libraries the host gives them.

This file is the language guide. Each standard library is documented in `references/libs/<Lib>.md`; read the one
you need before using functions not shown here (list at the end).

## Mental model

- A document is a **graph of objects**. Each object holds **fields** (named values) and **functions**.
- The top-level body of a file is the **main root** object. `root Name { ... }` declares another root. Imports
  can create roots (`import './x.stof' as X;`).
- Inside a function, `self` is the object the function lives on, `super` is its parent, and `root` is the
  root above it.
- Functions are values on objects (`self.fn_name`), not globals. Call them with a path: `self.total()`.
- Fields are created by declaring them in the document or by assigning (`self.count = 1`). Variables (`let`,
  `const`) live only inside a function.

## Rules that trip people up

Read these before writing Stof. Each one is a real mistake that produces wrong results or an error.

1. **Paths start at a root, a variable, `self`, `super`, or `root`.** Inside a function, `config.port` means
   "the root named `config`". For a field, write `self.config.port`. The parser warns about bare names that
   can't resolve: `warning: unknown name 'config' ... (use self.config for a field)`.
2. **Functions only see their own parameters and locals.** A named function can't read its caller's variables;
   pass values as arguments. Arrow functions (`(x: int): int => x + offset`) do see the variables of the
   function they're written in.
3. **Arithmetic with null is an error.** `null * 3` throws `cannot multiply null: the left value is null or
   missing (use ?? to give a default)`. Give defaults with `??`: `(self.rate ?? 0) * 3`. Initialize
   accumulators: `let total = 0;`. Comparisons with null don't throw (`null < 10` is `true`), so check for null first.
4. **`{ ... }` inside an expression is a map or a set, not an object.** `{ b: 2 }` is a map whose key is the
   value of the variable `b`. Create objects with `new`: `new { a: 1, nested: new { b: 2 } }`. Inside
   `new { ... }` and in the document body, `{ ... }` is an object.
5. **`return x;` needs the semicolon.** The last expression of a block without a `;` is its value:
   `fn area() -> float { self.w * self.h }`.
6. **Number methods return a new value.** `x.round(2)` doesn't change `x`; write `x = x.round(2)`.
7. **Constructors run only with `new`.** An object declared in the document with a type (`Rect unit: {...}`)
   gets the type's fields and functions, but its `#[constructor]` doesn't run.
8. **Lists, maps, sets, and objects are shared on assignment.** `let a = l;` aliases `l`; use `copy(l)` for
   an independent copy.
9. **`async { expr }` with a single expression is an async set literal.** Use a statement:
   `async { return 1 + 1; }`.
10. **Template strings are always `str`.** `` `${a}${b}` `` concatenates text even for numbers; null prints
    `null`. `\$` escapes a dollar sign.
11. **Catch values are error codes.** `catch (e: str)` receives `'AssignConst'`, `FuncDne("Num.split")`, or the
    value passed to `throw(...)`. Don't match on the human-readable message.
12. **Arguments are cast to the parameter type.** `fn f(v: int)` called with `'5'` receives `5`; an impossible
    cast throws (`CastVal(...)`).
13. **`int / int` is integer division.** `7 / 2 == 3`. Use a float operand (`7 / 2.0`, `(n as float) / 2`) for
    a fractional result. Wrap casts in parentheses inside larger expressions: `(n as float) / 2`, not
    `n as float / 2` (a parse error).
14. **Tests run concurrently on one shared document.** Processes switch at every loop iteration and Stof function call
    (and `await`/`sleep`); code between those points runs without interruption. So tests that change the same
    fields (`self.api.timeout = 2min` in one, `self.api.replicas = 0` in another) interleave and break each other.
    Give each test its own data (`new { ... }`, or fields only that test uses) and treat shared fields as
    read-only. Sharing is intended for tests that coordinate on purpose.

**Answers to common questions:**
- A missing field reads as null (`self.nope` is null, no error); calling a missing function throws (`FuncDne`)
  unless you use `?.`.
- `{ ... }` inside a list in the document body is an object (`items: [{ qty: 2 }]`); JSON arrays of objects
  parse into lists of objects (`o.users[0].name`).
- `int` and `float` compare by value (`85.0 == 85`); `str(2.0)` is `'2'`; casts with `as` work between numbers,
  strings, and units (`'3.5' as float`, `42 as str`, `512MiB as GiB`).
- Typed locals: `let total: float = 0;`. `const` stops reassignment, not mutation (`const l = []; l.push_back(1);`
  is fine).
- `new { ... }` inside a function creates a child of `self` that lives until dropped. `drop(o)` temporary objects,
  or create them `on` a scratch object.
- `obj.fields()` is a list of `(name, value)` tuples (no functions).
- A function value keeps its own `self`: `handler(x)` runs with `self` = the object the handler is defined on.
- `pln(a, b)` prints its arguments concatenated with no separator.

## Fields and values

```stof
// JSON works as-is (quotes, commas, and semicolons are all optional in Stof)
"name": "Stof"
version: 1.2.0                 // ver (semantic version)
str label: 'hi'                // typed field: the value is cast to the type
const str fixed: 'locked'      // const: assigning it throws 'AssignConst'
list! tags: []                 // `!` = not null
cm height: 6ft + 2in           // units convert: 187.96cm
MiB memory: 2GB
ms ttl: 5min                   // 300000ms
server: {                      // nested object
    port: 8080
    address: 'localhost'
    fn url() -> str { `https://${self.address}:${self.port}` }
}
#[readonly] ro: 42             // attributes annotate fields and functions
#[private] secret: 'x'         // private fields aren't exported to JSON etc.
root Settings { theme: 'dark' }
```

**Types:** `null`, `bool`, `int`, `float`, `str`, `ver`, `blob`, `list`, `map`, `set`, tuples (`(int, str)`),
`obj`, `fn`, `data`, `Promise<T>`, `unknown`, unions (`int | str`), units, and custom types (`Rect`,
`Geometry.Point`). `typeof v` gives the base type (`'obj'`, `'float'`); `typename v` gives the specific one
(`'Rect'`, `'cm'`).

**Units:** length `km hm dcm m dm cm mm um nm mi yd ft in`; time `day hr min s ms us ns`; temperature `K C F`;
mass `Gt Mt t kg g mg ug ng pg Ton lb oz`; data `bit byte KB MB GB TB PB … KiB MiB GiB TiB …`; angles
`rad deg`. Literals carry units (`5km`), math converts (`1km + 500m == 1.5km`), `as` converts (`1km as m`),
and `v.to_units('GB')` converts by name.

**Literals:** `'single'`, `"double"`, `` `template ${expr}` ``, `r#"raw"#`, `[list]`, `{'map': 1}`, `{1, 2}`
(set), `(1, 'two')` (tuple), `1_000`, `0xFF`, `1e3`, `1.2.3` (version).

## Functions

```stof
fn add(a: float, b: float = 1) -> float { a + b }                 // default parameter
fn greet(name: str = 'world') -> str { return `Hello, ${name}!`; }
fn maybe(id: str, limit?: int) -> int { limit ?? 10 }            // optional parameter (null when omitted)
fn fib(n: int) -> int { if (n <= 1) return n; this(n - 1) + this(n - 2) }   // `this` = current function
```

```stof
self.add(1, 2)              // 3
self.add(b = 5, a = 1)      // named arguments
const double = (x: int): int => x * 2;    // arrow function (sees enclosing variables)
const f = self.add;         // functions are values
f(2, 3); f.call(2, 3);
```

**Library calls:** `Num::abs(-3)` always calls the library; `(-3).abs()` calls it as a method through the
value's type library; `Lib?::func()` returns null when the library or function doesn't exist (Ex. an optional
host library). Standard library functions (`pln`, `assert_eq`, `parse`, `copy`, ...) are called bare.

**Null-safe access:** `self.a?.b`, `self.obj?.func()`, `?self.maybe_missing()` return null instead of throwing.
`a ?? b` gives `b` only when `a` is null (`false ?? 'x'` is `false`).

## Control flow and errors

```stof
let total = 0;
for (const x in [1, 2, 3]) total += x * index;      // `index` is the loop position
for (let i in 3) total += i;                        // 0, 1, 2
for (const pair in self.server.fields()) pln(pair[0], pair[1]);
for (let v in &list) v *= 10;                       // `&` iterates by reference (modifies the list)
while (cond) { if (done) break; continue; }
const kind = switch (i) { case 4: 'four' default: 'other' };
const sign = i > 0 ? '+' : '-';
^outer for (const a in 3) { for (const b in 3) { if (a * b == 2) break ^outer; } }   // labeled loops

try { throw('bad input'); }
catch (e: str) { pln(e); }          // typed catch; `catch { }` and `catch (e) { }` work too
try { throw(new { code: 42 }); } catch (e: obj) { pln(e.code); }
```

**References:** `let first = &l[0]; first = 0;` writes into the list; `swap(&a, &b)` swaps variables.

## Objects, types, and prototypes

```stof
#[type]
Shape: {
    str name: 'shape'
    fn area() -> float { 0 }
    fn describe() -> str { `${self.name}: ${self.area()}` }
    #[constructor]
    fn created() { self.made = true; }       // runs on `new`
}

#[type]
#[extends('Shape')]
Rect: {
    str name: 'rect'
    float w: 1
    float h: 1
    fn area() -> float { self.w * self.h }
    #[static]
    fn square(side: float) -> Rect { new Rect { w: side, h: side } }
}

Rect unit: { w: 2, h: 3 }          // typed object in the document (no constructor call)
shapes: {}
```

```stof
self.unit.area()                    // 6, Rect's function
self.unit.describe()                // 'rect: 6', inherited from Shape
self.unit.instance_of('Shape')      // true
self.unit.area<Shape>()             // 0, call the Shape version explicitly
<Rect>.square(3)                    // static call on a type
const r = new Rect { w: 4, h: 5 } on self.shapes;   // `on` sets the parent (default: self)
new Geometry.Point { x: 1 }        // types nested in objects or imports: qualified name
drop(r); r.exists();                // false: objects live until dropped

const o = new { a: 1 };
o.c = 3;                            // assignment creates fields
o.insert('d.e', 4);                 // creates intermediate objects
o.contains('a'); o.fields(); o.parent(); o.id();
```

## Schemas

A type's `#[schema(...)]` attributes validate (and can transform) another object's fields.

```stof
#[type]
Server: {
    #[schema((target_val: int): bool => target_val > 0 && target_val < 65536)]
    int port: 8080
    #[schema((target_val: str): bool => target_val.len() > 0)]
    str host: 'localhost'
    #[schema_optional]
    str note: null
}
config: { port: 70000, host: 'example.com' }
```

`<Server>.schemafy(self.config)` returns `false` (port out of range) and leaves `self.config` unchanged; after
`self.config.port = 443` it returns `true`. With `remove_invalid = true` it deletes failing fields instead (and
returns `true`); `remove_undefined = true` deletes fields the type doesn't declare. A missing field is validated
as null: `#[schema_optional]` skips validation for null or missing values; otherwise make required fields reject
null explicitly (`target_val != null && ...`), since comparisons with null don't throw (`null < 10` is `true`) and
a method call on null (`target_val.len()`) throws out of `schemafy`. See `references/libs/Obj.md` (`schemafy`).

## Attributes and running functions

- `#[main]`: run by `stof run` and the host `run()`. `stof run -a <attr>` (and `doc.run('attr')`) runs
  functions with another attribute instead.
- `#[test]`, `#[test(expected)]`, `#[errors]` (the test passes only if it throws): run by `stof test`.
- `#[run]`, `#[run(order)]`: `obj.run()` calls these on an object, lowest order first (pipelines).
- `#[async]` or `async fn`: calling it returns a `Promise<T>`; `await` it.
- Custom attributes are data: `#[on-save] fn audit(doc: obj) {...}` and
  `for (const handler in funcs(attributes = 'on-save')) handler(self);` dispatch events.
- `#[run({'args': [5]})]`: pass arguments to a run step.
- `#[init]`: runs once after the document is parsed (setup).
- `#[constructor]` / `#[dropped]` (in a type): run on `new` and on `drop(instance)`.
- On fields: `#[readonly]`, `#[private]`, `#[no-export]` (left out of `stringify`), `#[schema(...)]`, plus anything
  custom (read with `self.attributes('field')`).
- `#[type_ignore]` on a type's field keeps it on the type only (not copied into instances; read it as
  `<Type>.FIELD`). Use it with `const` for shared constants, so instances don't each carry a copy.

## Imports and formats

```stof
import './lib/geo.stof' as Geo;           // relative to the importing file; `as` makes a root
import json './data.json' as self.data;   // explicit format, into a field of this object
import './config.yaml';                    // the file extension picks the format; into the main root
```

Formats: `stof`, `json`, `yaml`, `toml`, `urlencoded`, `text`, `md`, `bstf` (binary Stof: the whole document
including functions), and with features `pdf`, `docx`, and image formats. `@pkg/name` imports a package from
`stof/pkg/name` under the working directory.

```stof
stringify('json', o)                 // export an object (functions and private fields are skipped)
const back = new {}; parse('{"x": [1, 2]}', back, 'json');   // import into an object
const bin = blobify('bstf', o);      // binary, with functions; parse(bin, target, 'bstf') restores it
parse('fn hi() -> str { "hi" }', self);   // documents can add code to themselves
```

## Async

```stof
async fn slow(x: int) -> int { sleep(5ms); x * 2 }

const b = self.slow(5);                       // Promise<int>
const a = async { return 1 + 1; };            // async block
await a;                                      // 2
await [b, self.slow(1)];                      // [10, 2]: awaiting a list awaits each
await 'plain';                                // non-promises pass through
```

## Testing and the CLI

```stof
#[test]
fn totals() {
    assert_eq(self.add(1, 2), 3);
    assert(self.unit.instance_of('Shape'));
    assert_null(self.nope?.deeper);
}

#[test(55)]
fn expected_value() -> int { self.fib(10) }

#[test]
#[errors]
fn must_fail() { throw('expected'); }
```

```bash
stof test file.stof            # run #[test] functions (exit code 1 on failure)
stof test ./dir                # a package directory
stof run file.stof             # run #[main] functions
stof run file.stof -a nightly  # run #[nightly] functions instead
stof pkg ./dir                 # build a .pkg from a directory with pkg.stof
```

Errors print the message, the failing line with a caret, and the Stof call stack:

```text
error: cannot multiply null: the left value is null or missing (use ?? to give a default)
  --> main.stof:31:5
    |
 31 |     self.rate * units
    |     ^
  at root.total (main.stof:31:5)
  at root.main (main.stof:28:5)
```

Fix parse warnings (`unknown name ...`) before debugging behavior; they usually mean a missing `self.`.
Debug with `pln(...)`, `dbg(...)`, and `callstack()`.

## Embedding

Hosts parse documents, call functions by path, read and write values, register host functions as libraries,
and export. Paths in `get`/`set`/`call` are relative to the main root (`'config.port'`, `'total'`); roots and
`<Types>` work too.

**TypeScript / JavaScript** (`npm i @formata/stof`):

```typescript
import { StofDoc, stofAsync } from '@formata/stof';

const doc = await StofDoc.new(`
    rate: 2
    fn total(units: int) -> float { units * self.rate }
`);
await doc.call('total', 21);                     // 42
doc.get('rate'); doc.set('rate', 3);
doc.lib('Host', 'now', () => Date.now());        // Stof calls Host::now()
doc.parse({ extra: true });                      // add more (Stof source, a JS object, or bytes)
await doc.run();                                 // #[main] functions
doc.stringify('json'); doc.blobify('bstf'); doc.record();   // export (record = plain JS object)

const quick = await stofAsync`fn hi() -> str { 'hi' }`;
```

**Python** (`pip install stof`):

```python
from pystof import Doc

doc = Doc()
doc.lib('Host', 'name', lambda first, last: first + ' ' + last)
doc.parse("rate: 2\nfn total(units: int) -> float { units * self.rate }")
doc.call('total', [21])        # args: a list, a single value, or none (doc.call('fn'))
doc.get('rate'); doc.set('rate', 3)   # optional start object id as a last argument
doc.run()
doc.string_export('json'); doc.binary_export()   # bstf of the main root
```

**Rust** (`cargo add stof`):

```rust
use stof::model::Graph;
use stof::runtime::{Runtime, Val};

let mut graph = Graph::default();
graph.parse_stof_src("rate: 2\nfn total(units: int) -> float { units * self.rate }", None, Default::default())?;
let total = Runtime::call(&mut graph, "total", vec![Val::from(21i64)])?;   // 42
Runtime::run(&mut graph, None, true)?;                                     // #[main]
let json = graph.string_export("json", None)?;
```

**Sandbox:** embedded documents have no file system, environment, or network access. Hosts opt in:
`graph.allow_system()` / `graph.allow_http()` (Rust), `doc.allow_system()` / `doc.allow_http()` (Python),
`doc.allowHttp()` (JS: adds `Http.fetch` on the JS fetch API; there's no file system in the browser build). Without `allow_system()`, `import` statements
and `fs::*` calls fail. The `stof` CLI allows everything.

Host errors from `call`/`run` include the Stof call stack. Parse with the test profile (`'test'` in JS/Python,
`stof::model::Profile::test()` in Rust) to get statement-level line numbers in runtime errors; the default production profile
keeps documents smaller.

## Standard library reference

Read `references/libs/<file>` for full signatures and examples.

| Library | File | For |
|---|---|---|
| Std | `Std.md` | Called bare: `pln`, `assert*`, `throw`, `parse`, `stringify`, `blobify`, `copy`, `drop`, `swap`, `funcs`, `sleep`, `env`, `format`, `nanoid`, `min`/`max` |
| Obj | `Obj.md` | Objects: `fields`, `insert`, `get`, `contains`, `parent`, `children`, `schemafy`, `instance_of`, `run`, `to_map`, `move` |
| Str | `Str.md` | Strings: `len`, `split`, `lower`, `upper`, `trim`, `replace`, `contains`, `starts_with`, `substring` |
| Num | `Num.md` | Numbers and units: `abs`, `round`, `floor`, `sqrt`, `pow`, `min`, `max`, `to_units`, `has_units` |
| List, Map, Set, Tup | `List.md` `Map.md` `Set.md` `Tup.md` | Collections |
| Fn | `Fn.md` | Functions as values: `call`, `params`, `attributes`, `obj`, `bind` |
| Time | `Time.md` | `Time::now()`, RFC-3339/2822, date math |
| Ver | `Ver.md` | Semantic versions |
| Blob | `Blob.md` | Binary data |
| Data | `Data.md` | Opaque data components (PDFs, images, custom host data) |
| Prompt | `Prompt.md` | Prompt trees for AI workflows |
| Md | `Md.md` | Markdown to HTML |
| Http | `Http.md` | HTTP fetch (`http` feature) |
| fs | `fs.md` | File system (`system` feature; hosts can remove it) |
| Image, Pdf, Age | `Image.md` `Pdf.md` `Age.md` | Images, PDFs, age encryption (features) |
