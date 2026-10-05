# Changelog

## 0.10.3

### Added

- **`using` declarations** for temporary objects: `using tmp = new { .. };` is a const variable whose object is
  dropped when its block or function ends, on every exit (end of block, `return`, `break`/`continue`, a caught or
  uncaught error), exactly like `drop(tmp)` (`#[dropped]` functions run; not after an uncaught error, where the
  process can't run anything). A plain `{ ... }` block is the scope, so `{ using scratch = new {}; .. new {} on scratch .. }`
  is an arena. Values that aren't objects, functions, or data are ignored, and dropping by hand first is fine.
  `using` is only a keyword before a name (`using = 3;` still assigns a variable named `using`).
- **`using` expressions** for temporaries with no name: `self.handle(using new { id })` drops the object when the
  enclosing block or function ends, the same as a `using` declaration.
- **`stof test --leaks`** (`Runtime::test_leaks`, `Graph::test_leaks`): runs tests one at a time and fails any test
  that leaves objects behind (created, still in the document, and not referenced by any field, directly or inside a
  list, map, set, or tuple). The report says where they are.
- Trailing commas in call arguments and parameter lists (`f(a, b,)`, `fn f(a: int, b: int,)`, arrow functions),
  matching lists, maps, sets, and tuples. Comments are allowed anywhere inside the parentheses. Before, a trailing
  comma was a parse error, or in some positions (Ex. a multi-line `map(...)` as a function's last expression)
  mis-parsed with only an "unknown name" warning.

### Rust API

- `SymbolTable::pop()` returns the values of the scope's `using` variables (to drop) instead of a bool.
- New `Base::MarkUsing` and `Base::UsingValue` instructions (appended; BSTF-compatible).

### Fixed

- `Num.round(x, places)` and `x.round(places)` with more than 9 places: the scale was an integer that overflowed,
  so `Num::round(0.0105, 12)` gave `0.010500000461931886` (and debug builds panicked). Places past f64
  precision now leave the number as-is.

## 0.10.2

### Fixed

- `typeof` and `typename` bind to the value right after them, like `!` and `-`: `typeof x == 'obj'` is
  `(typeof x) == 'obj'`. Before, they took the whole rest of the expression, so `typeof x == 'obj'` was `'bool'`
  (always truthy) and the ternary in `typeof x == 'obj' ? a : b` was typed instead of chosen. Code that meant the
  type of a larger expression needs parentheses: `typeof (a + b)`. `(typeof x) == 'obj'` still works.
- `typeof` and `typename` are whole words: `typeofx` is a name, not `typeof x`.

## 0.10.1

Package metadata only (no code changes): updated descriptions, keywords, and categories on crates.io, npm, and PyPI
to match the new README. See 0.10.0 for the release notes.

## 0.10.0

A correctness, safety, and developer-experience release. Most programs run unchanged, but several long-standing
behaviors that silently produced wrong results are now fixed or reported, and some of those fixes are breaking.
Read **Breaking changes** before upgrading.

### Highlights

- **Readable errors with locations.** Parse errors show `file:line:col` and the offending line with a caret.
  Runtime errors show a plain-English message, the line that failed (test profile), and the Stof call stack.
- **Warnings for unknown names.** A bare name that can't be a variable, parameter, root, or standard library
  function is reported at parse time (it would always be null).
- **Lexical function scopes.** Functions no longer see their caller's variables.
- **Safety.** No known input crashes or hangs the runtime anymore: a fuzzing sweep found and fixed parser hangs,
  stack overflows, panics, and an out-of-memory abort in binary (BSTF) decoding.
- **Faster.** The Limitr spec (5,000 lines) parses 60x faster (2.9s → 0.05s); function calls are ~2x faster.

### Breaking changes

| Change | Before | Now | Migrate |
|---|---|---|---|
| **Paths start at a root** | `a.b` found any object named `a` anywhere in the graph (hash order) | `a.b` requires `a` to be a graph root (`root a {}`, `import .. as a`) | Use `self.a.b`, `super.a.b`, or `root.a.b` for fields |
| **Function scopes** | A function could read (and a typo could bind to) its caller's local variables | Named functions only see their own parameters and locals; arrow functions still see the function they're defined in | Pass values as arguments |
| **Arithmetic with null** | `3 * null == 3`, `null + 1 == 1`, `let t; t += 1` worked | Error: `cannot multiply null: the right value is null or missing` | Use `??` for defaults: `(self.rate ?? 0) * 3`; initialize accumulators: `let t = 0;` |
| **Argument evaluation** | Arguments ran inside the callee after earlier params were bound (`f(1, q)` could see the callee's `q`) | Arguments are evaluated in the caller's scope before the call; defaults are evaluated in the callee (`fn f(a, b = a * 2)` works) | None, unless code relied on the old order |
| **Number methods don't mutate** | `x.abs()`, `x.round(2)` etc. rewrote `x` (even consts) | They return a new value; `x` is unchanged | Assign the result: `x = x.round(2)` |
| **Template strings are strings** | `` `${a}${b}` `` could add numbers; a null printed as empty | Always a `str`; each part prints like `str(v)`; null prints `null`; `\$` escapes a dollar | Format numbers explicitly if needed |
| **Relative imports** | Relative to the working directory | Relative to the importing file; `@pkg/x` is `stof/pkg/x` in the working directory | Fix paths that assumed the working directory |
| **Import formats** | An unknown explicit format (`import foo '..'`) silently did nothing; the "extension" was everything after the first `.` | Unknown explicit formats are errors; the real file extension picks the format; unknown extensions import as text (or bytes) | |
| **Statements** | `return x` without `;` silently mis-parsed; `letter = 1` could parse as `let ter = 1` | `return x` needs `;`; keywords must be whole words | Add the `;` |
| **Integer overflow and /0** | `i64::MAX + 1` wrapped negative; integer `/ 0` and `% 0` panicked | Overflow becomes a float; `/ 0` is `inf`, `0 / 0` and `% 0` are `NaN` | |
| **Unit conversions** | Imperial mass constants were approximate (`1lb` = 453.592g); conversions drifted | Exact factors (`1lb` = 453.59237g, `1lb → oz` is exactly 16); mixed-unit `==` compares within 1e-14 | Update exact expectations |
| **JSON non-finite numbers** | Exporting `inf`/`NaN` to JSON panicked | Exported as `null` (like `JSON.stringify`) | |
| **Sandboxed by default** | `Graph::default()` (Rust) and Python `Doc()` registered the `fs` library, `env` functions, and `Http` (Python, JS); documents could read and write files, read environment variables, import files from disk, and make network requests | None of these unless the host opts in; file and package imports are refused with a clear error | Call `graph.allow_system()` / `graph.allow_http()` (Rust), `doc.allow_system()` / `doc.allow_http()` (Python), `doc.allowHttp()` (JS). The `stof` CLI enables everything |
| **Catch values unchanged** | | `catch (e: str)` still receives the error code form (Ex. `'AssignConst'`, `FuncDne("Num.split")`) | |

**Host APIs (JS, Python, Rust):**
- `doc.get/set/call` paths are relative to the start object (the main root by default) when the first name is
  there (`doc.get('config.port')`, `doc.call('hello')`); roots, `<Types>`, and libraries work as before.
- Errors from `call`/`run` include the Stof call stack (Ex. `cannot call 'round' on null ...\n  at root.main (main.stof:12:5)`).

**Python API:** `Doc.get(path, start=None)`, `Doc.set(path, value, start=None)`, `Doc.call(path, args=None)` (a list,
a single value, or nothing), and `Doc.binary_export(format='bstf', node=None)` have defaults; passing `None`
explicitly still works.

**Rust API:**
- `Runtime::call`/`eval` errors are wrapped in the new `Error::Located(inner, stack)`; use `error.inner()` to match variants.
- `Error` displays human-readable messages (`error.code()` gives the variant name, `error.catch_value()` what a catch block sees).
- `StofParseError`: `file_path` and `location` are methods (`error.file_path()`, `error.location()`); details are in `error.info`.
- `Func` has a `src: Option<SrcLoc>` (where it's defined; not serialized).
- New `Base::Src(line, col)` instruction (debug profiles only, see below) and `Error` variants `Located`, `NullArithmetic`, `FuncArgsInfo` (appended; BSTF-compatible).

### Added

- **`allow_system()` / `allow_http()`** (Rust, Python; `allowHttp()` in JS): opt in to file system/environment and network access for documents. In JS, `allowHttp()` also adds `Http.fetch` (the native runtime's signature and response map, backed by the JS fetch API), so apps no longer need their own.
- **`Lib::func(args)`**: always calls the library, never an object or variable of that name. `Lib?::func()` is null
  when the library or function doesn't exist (Ex. optional host libraries).
- **Null-safe calls anywhere in a path:** `self.obj?.func()` (before, only a leading `?` worked for calls).
- **Qualified type names:** `<Geometry.Point>` is the nearest type `Point` under `Geometry`, from anywhere. Type
  lookup is deterministic (nearest wins, then by path).
- **Assignment onto computed targets:** `self.customer(id).type = 'x'`, `list[0].name = 'a'`.
- **Std:** `assert_null(v)`, `assert_not_null(v)`, `inf()`, `nan()`.
- **Parse warnings:** unknown bare names and functions (with location); roots, params, or locals named like a library.
- **Error reports:** `error:` message, `note:` for common mistakes (Ex. `'self.config' has no field 'rat'`), the source
  line with a caret, and `at <fn> (file:line:col)` per call. `STOF_TRACE=1` adds the low-level instruction trace.
- **Statement locations** (`file:line:col` of the failing statement) when `Profile.debug_info` is on (the test
  profile, and `stof run`). Production profiles record only where each function is defined, so BSTF documents keep
  their size (markers add ~11%).
- **Release profile:** LTO (binary 30MB → 19MB).
- **New agent skill** (`content/SKILL.md`, packaged with the library references in `content/stof.zip`): rewritten
  as a concise, verified guide (mental model, common mistakes, syntax, testing, embedding).
- **`stacker` feature** (on in `basic`, `full`, `py`; not `js`): the parser and BSTF decoder grow the stack instead
  of overflowing on deeply nested input.

### Fixed

- Test and main reports skipped functions that contained loops (their output lines vanished).
- Parsing a string with no target nested every object after the first into it (Rust `parse_stof_src`).
- `?.` in a call path threw instead of returning null.
- `void`/`null` compared unequal to themselves (a set could hold two nulls); sorting with `NaN` panicked; `min`/`max`
  with `NaN` depended on argument order.
- Integer literals above 2^53 lost precision; huge literals saturated; `0x10-1` and oversized hex literals panicked.
- wasm: integers were converted to JS as 32-bit (timestamps were mangled).
- `Http.fetch` sent no User-Agent, so APIs that require one (Ex. GitHub) rejected every request; it now sends `stof/<version>` unless the request sets its own.
- `Http.parse` (and any import by content type) didn't recognize content types with parameters, so `application/json; charset=utf-8` was imported as raw bytes instead of JSON.
- JS: when an async library function's promise rejected (Ex. a failed fetch), the error came back as the function's return value instead of being thrown, so `try`/`catch` never saw it. Rejections now throw the error message.
- JS: library functions can declare parameter names (`doc.lib('App', 'f', f, false, ['a', 'b'])`) so Stof can call them with named arguments; before, named arguments landed in the wrong position.
- JS: plain objects passed to Stof (Ex. `doc.call('f', { a: 1 })`) were converted to bytes; they're maps now, like Python dicts.
- Exceptions inside a call didn't restore the caller's scopes in the catch block.
- `stof run` / `stof test` (and `Runtime::run`/`test`) dropped errors raised before a function's call started (Ex. a
  `#[main]` or `#[test]` function with a required parameter): the run failed with no message and no function name.
- Argument errors say what's wrong: `missing argument 'units' for total(units: int, rate?: float)`, `too many
  arguments (3 given, 2 expected) ...`, `unknown argument 'x' ...` (was "invalid arguments for this function
  call"). Catch blocks still receive `'FuncArgs'`. New `Error::FuncArgsInfo` (appended; BSTF-compatible).
- Library doc examples: `Num.fract`, `Prompt.set_tag`, `Std.prompt`, and `Std.stringify` showed wrong results.
- A catch variable leaked into the enclosing scope, so a second `catch (e)` in the same function failed with
  "a variable with this name already exists". It is now scoped to its catch block.
- Import: the `../` handling, spaces in paths, double imports of one file, and a panic on relative imports without a base.
- URL-encoded parsing panicked on `nan`/`inf` words and on non-UTF-8 escapes.

### Safety and robustness

- **Hangs:** `a: new {`, `a: { b: 1`, and `root X {` at the end of input looped forever. Nested parentheses parsed in
  exponential time (12 levels took seconds).
- **Crashes:** deep nesting overflowed the stack (now an error at 128 levels, the same limit as serde_json). BSTF
  with corrupt lengths could request exabytes of memory (an uncatchable abort); decoding now never trusts lengths
  and bounds nesting (128). Data too deep to decode is an error instead of being silently dropped.
- **Panics → errors:** number literals of only `_`, blob bytes over 255, a failing switch case value, object
  `#[init]` errors, invalid age keys or data, invalid HTTP header values, out-of-range dates, JS library calls
  (any argument count now), host strings containing `_pr:_ms`, and error reporting for dropped objects.
- Verified with a mutation fuzzer over the test suite: 40k parsed documents, 1.5k executed, 20k corrupted BSTF
  blobs, with no panics or hangs.

### Performance

- Parsing: errors are recorded without allocating and formatted once; keyword statements dispatch directly; nested
  parentheses parse once. Limitr spec: 2.9s → 0.05s.
- Runtime: instruction queues are plain deques, the clock is checked every 256 instructions, internal tags are
  counters. 200k function calls: 2.9s → 1.5s; 1M loop iterations: 4.1s → 3.0s.

### Dependencies

- Security: all 13 `cargo audit` vulnerabilities fixed (bytes, crossbeam-epoch, h2, rustls, rustls-webpki, time,
  imbl-sized-chunks, lopdf, pyo3). `npm audit` (web): 0.
- Upgraded: imbl 7, lopdf 0.45, pyo3 0.29, wasm-bindgen 0.2.129 (the wasm-bindgen CLI must match), and patch updates.
- Added: `stacker` (optional, native only).
