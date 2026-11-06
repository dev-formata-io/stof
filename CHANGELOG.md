# Changelog

## 0.11.1

Fixes found building the first real use of 0.11's seals (a signed, sealed module in Limitr), and a note on units.

### Fixed

- **`copy(obj)` makes one copy of each object inside, at every depth.** Below the top level, a field holding a
  child object was copied a second time: the copy had two copies of the child, and the field pointed at one while
  a path (`copied.a.b`) found the other, so a change through one didn't show through the other.
- **Digests of objects in lists.** An object inside a list (or map, set, or tuple) was digested by its path, and
  objects parsed into a list (Ex. from JSON) have random names, so the same data never had the same digest, and a
  seal that checked one could never match. They're now digested by what they hold. Digests of anything without
  objects in lists are unchanged.
- **An object that holds only removable seals can be dropped** (Ex. a plugin's container), and the removable parts
  go with it. It was refused, as if they were sealed for good.
- **A refused change never surfaces in another call.** A refusal made while a process failed (Ex. a `using` value
  that a seal kept, dropped as the error unwound) stayed pending, and the next instruction to run, in any process,
  failed with it. It now stays with the call that made it.
- **`List.insert` and `List.replace` on an empty list no longer panic in debug builds.** The index check subtracted
  one from the length, which overflows for an empty list (release builds, including the npm and PyPI packages,
  wrapped around and did the right thing). Found running Limitr's suite with a debug build of stof-cli.

### Docs

- **Units and plain numbers.** The skill and the site (Types and Units, Units Reference) now say what math between
  units gives (the larger unit: `Time::now() + 1hr` is in hours) and that a plain number compared with a unit value
  compares bare numbers, so a time read from JSON should be given its units first (`const at: ms = record.expires;`).

### Tests

- Stof suite (3 new): `copy()` at every depth, digests of objects in lists, removable containers; and the empty
  list `insert`/`replace` cases. `tests/seal.rs` (1 new): a refusal stays with its call.

## 0.11.0

Trust, built into documents. Keys and signatures in Stof itself, one signing scheme everywhere, and seals that can
confine code from somewhere else to what it's given, or let it be removed when it's no longer wanted. A minor
version because it changes the signing functions 0.10.10 added and the digests they sign (see Upgrading).

### Added

- **The `Key` library: keys and signatures in documents.** A signature proves who wrote something and that it
  hasn't changed.
  - `Key::generate()`: a new secret key (32 bytes from the system's secure random source, base64).
  - `Key::public(secret)`: its public key, to share. It checks signatures; it can't make them.
  - `Key::sign(secret, context, message)`: sign the exact bytes of a message (a string's UTF-8, or a blob).
  - `Key::verify(public, context, message, signature)`: true only for exactly that message, context, and key. A
    malformed key or signature is simply false.
  - The context names what a signature is for (Ex. "orders/v1"). A signature only verifies under the context it was
    made with, so one made for one purpose can't be passed off as another.
  - Keys and signatures are plain base64 strings (hex is read too): they travel in JSON, environment variables, and
    headers, and need nothing dropped.
- **One signing scheme everywhere.** Hosts have the same functions, with the same names and argument order, and
  signatures made anywhere verify everywhere:
  - JS: `StofDoc.generateKey()`, `StofDoc.publicKey(secret)`, `StofDoc.sign(secret, context, message)`,
    `StofDoc.verify(public, context, message, signature)` (a string or Uint8Array).
  - Python: `Doc.generate_key()`, `Doc.public_key(..)`, `Doc.sign(..)`, `Doc.verify(..)` (str or bytes).
  - Rust: `stof::model::seal::{generate_key, public_key, sign, verify}`.
  - A seal signature is a signature over the part's digest under the context "stof/seal"
    (`seal::SEAL_CONTEXT`): `Key::sign(secret, 'stof/seal', digest(part))` in Stof, or
    `StofDoc.sign(secret, 'stof/seal', doc.digest(path))`. `doc.seal(path, { publicKey, signature })` checks it.
- **Confined seals: code that reaches only what it's given.** For code you didn't write that runs in your
  document (a plugin, a rule a model wrote). Seal it with `reads` and/or `hosts`, and while its code runs (and
  everything it calls):
  - **Reads:** it finds only the sealed part and the objects in `reads` (and what's under them). A path to
    anything else finds nothing (`root.secrets.key` and `super.secrets` are null), and a type it isn't given isn't
    found, made (`new Type {}`), or cast to (which would read its defaults). `reads` are paths (or `<Type>`s), looked up each time the part is entered, so an object replaced at a
    path is the one it reads. Objects given from Stof are kept as their paths.
  - **Calls:** calling a function elsewhere fails, even through a function value it was handed. Functions it may
    read (Ex. a `<Helpers>` type) can be called, and run confined too.
  - **Changes:** it changes only its own part (its fields, objects it makes). What it reads is read-only to it, and
    it can't make roots or seal anything outside itself.
  - **Libraries:** it can't hand an object outside its reach to a library function (`stringify`, `Obj`, your app's
    functions) or get one back (Ex. `Obj.parent(self)`, `Obj.from_id(..)`, `funcs()`). Exporting everything
    (`stringify('json')` with no object), or an object that holds one out of reach, fails in every format.
  - **Hosts:** `Http.fetch` reaches only the listed hosts, checked again after redirects (natively on the final
    URL, like the document's own allowlist; in JavaScript on every hop). The document's allowlist still applies: a
    host must be on both. `[]` means no network.
  - Objects out of its reach stay opaque, even ones it's handed (an argument, or a field you put in it): it can
    hold one and hand it back, but not look inside, change, print, or export it. Give it values (strings, numbers,
    lists, maps), or list the objects in `reads`.
  - A failed lookup finds nothing (null, like a missing field), so code can't probe for what it can't see; a
    refused change or call fails with a `sealed: ...` error, so nothing is refused silently.
  - Confinements stack: sealing again with `reads` adds one (the part runs under both), so a document can narrow
    what a host gave a part but never widen it. Confined code that calls another confined part runs under both.
  - The rest of the document isn't confined, including code that calls into a confined part and keeps going once
    it returns (or fails). Host calls (get, set, call) never are.
  - Rust: `SealOptions { reads, hosts, .. }` with `graph.seal_path_with(path, &options, digest)`,
    `seal_signed_with(..)`, `seal_with(node, ..)`. JS: `doc.seal(path, { reads: ['inputs'], hosts: [..] })`.
    Python: `doc.seal(path, reads=[..], hosts=[..])`. Stof: `seal(obj, reads = [self.inputs], hosts = [..])`.
  - Cost: nothing for documents without confined parts (a few empty checks). Confined code pays a parent walk on
    lookups and library calls, and `reads` are looked up once per entry into the part.
- **Removable seals.** `removable` seals an object that can still be dropped whole, by Stof (`drop(obj)`) or the
  host (`doc.remove(path)`); nothing in it can change, and it can't be moved or renamed. Not when another seal
  covers it (one around it, or one inside it that isn't removable). Sealing again without `removable` makes it stay.
  The seal goes with the object.
- **Seeing how something is sealed.** `sealed(obj)` in Stof (a map), `doc.sealed(path)` in JS (an object),
  `doc.sealed(path)` in Python (a dict), `graph.seal_info(node)` in Rust: null when it isn't sealed, otherwise
  `part` (the path of the object sealed: this one or one around it), `fields`, `removable`, and `confined` (one
  `{ reads, hosts }` per confinement its code runs under).
- **Removing allowed hosts.** `graph.disallow_http_hosts([..])` (Rust), `doc.disallow_http_hosts([..])` (Python),
  `doc.disallowHttp([..])` (JS). Removing every listed host leaves none allowed, not any host; with no list (any
  host) there's nothing to remove and it returns false.
- **Age takes secret keys as strings.** `Age::parse` and `Age::public` accept the secret key as a string
  (`AGE-SECRET-KEY-1...`, Ex. passed in by the host) as well as a `Data<Age>`. Age's docs now cover how it works
  with `Key`: Age encrypts (only the reader can open it), Key signs (the reader knows who sent it).
- `Error::Key` ("key: ...") for keys that aren't keys.

### Changed

- **Signing functions (breaking).** The 0.10.10 seal-only `sign(digest, secret)` / `verify(digest, public,
  signature)` (JS `StofDoc`, Python `Doc`, Rust `seal`) are now the general `sign(secret, context, message)` /
  `verify(public, context, message, signature)`. Seal signatures are made under "stof/seal".
- **Digests travel: the same part has the same digest in any document (breaking for stored digests).** A digest no
  longer includes the part's own name (where it's put decides that), names objects by the fields that hold them
  (objects made with `new` have random names), and leaves out the attributes Stof keeps for itself on field
  objects (so data imported as JSON digests the same as the same data written in Stof, before and after it's read).
- **JS: `allowHttp([..])` adds hosts** (it replaced the list), as it does in Rust and Python.
- `Std::seal` takes `removable`, `reads` (objects or paths), and `hosts` (named, after `fields`). Calls written
  before still work.

### Fixed

- **Dropping a field whose object a seal keeps no longer removes the field.** `drop(self.x)`, `Obj.remove(..)`, and
  `doc.remove(path)` removed the field even when a seal refused to drop the object behind it (the call failed, but
  the object was left without its field). Now nothing changes.
- **Sealed and confined fields can no longer be changed in place.** A list, map, set, or tuple read from a field
  is shared with the field, so `self.frozen.tags.push_back('x')` changed a field sealed with `fields: true` (since
  0.10.10), and a confined part could do the same to an object it only reads (`push_back`, `insert`, `swap`,
  `&field`). Values read from a field the running code may not change are now copies: changing the copy never
  changes the field. Assigning to such a field (`=`, `+=`, `Obj.set`, the host's `set`) is refused before anything
  is written (a list could be written in place before the seal check refused it). Fields nothing seals read and
  change exactly as before, with no copying.
- **Destructuring respects confinement and private fields.** `const { key } = o` read fields of an object a
  confined part may not reach, and read `#[private]` fields from outside their object. Both now behave like
  `o.key`: refused out of reach, and null for a private field. A library call on a private field from outside
  (Ex. `o.hidden.len()`) now sees null too.

### Upgrading from 0.10.10 or 0.10.11

| Before | Now |
|---|---|
| `StofDoc.sign(digest, secret)` | `StofDoc.sign(secret, 'stof/seal', digest)` |
| `StofDoc.verify(digest, public, signature)` | `StofDoc.verify(public, 'stof/seal', digest, signature)` |
| `Doc.sign(digest, secret)` / `Doc.verify(..)` | `Doc.sign(secret, 'stof/seal', digest)` / `Doc.verify(public, 'stof/seal', digest, signature)` |
| `seal::sign(digest, secret)` / `seal::verify(..)` | `seal::sign(secret, seal::SEAL_CONTEXT, digest.as_bytes())` / `seal::verify(..)` |
| A digest or seal signature you stored | Compute the digest again with 0.11 and sign it again |
| JS `allowHttp([...])` twice to replace the list | `disallowHttp([...])` the hosts to drop |

### Tests

- `tests/confine.rs` (21 with full features): confinement (reads, reads that follow paths, calls, changes,
  libraries, `super`, async, function values, opaque objects, stacking and narrowing, hosts and the document's
  allowlist, redirects natively against a local server, every export format, unwinding after errors, seals not exported), removable seals,
  `seal_info`, signatures (contexts, exact bytes, hex keys, errors, host and document interchangeable), seal
  signatures made in Stof and checked by the host, host lists, digests across documents, and in-place changes
  (copies of guarded fields, refused assignments, destructuring).
- Stof suite: `libs/key.stof` (keys, signatures, a fixed signature that must keep verifying, errors, seal
  signatures), `libs/age.stof` (secret key strings, sign and encrypt), and `security/sealing.stof` (confined
  reads, types and methods, parts calling parts, narrowing, reads that follow paths, `sealed()`, hosts, removable
  seals and the drop fix, digests through JSON, in-place changes, destructuring and private fields).
- The JS checks (`tests/js/check.mjs`) and Python smoke test cover the same APIs in those builds, including the
  fixed signature (the same scheme in every build) and a redirect away from a confined part's host.

## 0.10.11

### Fixed

- **`exit(..)` on a process that's already gone no longer panics.** Exiting a process that had already finished or
  exited (Ex. `exit(p, p)`, or `exit(p)` after `await p`) panicked in the runtime (`swap_remove` out of range); it's
  now nothing to do. Whether it panicked depended on timing, which is why the robustness test only caught it in CI.

## 0.10.10

### Security

- **Memory limit: a document can no longer run its host out of memory.** Running out of memory can't be recovered
  from: in WebAssembly a one-line loop that doubles a string (`s = s + s`) trapped the engine and, on an async call
  (`await doc.call(..)`, which a `try` around it couldn't catch), ended the Node.js process; natively it aborted the
  process. The execution time limit didn't help: memory ran out in under 4 seconds. Now a call fails with a memory
  limit error once it has grown memory by more than 256 MB, and the document keeps working.
  - Set it per document: `graph.max_memory = Some(bytes)` (Rust), `doc.set_max_memory(bytes)` (Python),
    `doc.setMaxMemory(bytes)` (JS). `None`/`null` for no limit.
  - Growth is counted by a counting allocator (`runtime::memory::StofAlloc`, per thread, no atomics), checked before
    every instruction, and summed over a call's steps (an async call that resumes on another thread is still counted
    right). The JavaScript and Python builds install it; a Rust host can install it as its `#[global_allocator]`.
    Measured cost: within noise (the check) and about 1 to 2% (the allocator, natively).
  - Steps that can make a value many times bigger in one instruction check the size first, with or without the
    allocator: `'a' * n`, `Str.replace`, `Str.split`, `List.join`, blob/string/prompt to list or set, and `parse`.
  - Like the time limit, the error ends the call: a `catch` can't keep it going while the memory is still held.
  - Memory a call keeps (Ex. fields it sets) counts against that call only; the next call starts from zero.
- **JavaScript: a crash in the engine fails the call instead of ending the process.** An async call
  (`doc.call`, `callSource`, `run`) was driven by Rust futures polled from promise callbacks, so a WebAssembly trap
  there escaped where no `try` could catch it and Node.js exited. doc.ts now runs each call one step at a time (each
  step a synchronous call into the engine), with the same call gate, the same yield to the event loop every 20 ms,
  and the same host promises and call-backs (`await doc.call(..)` from a library function runs between steps, as
  before). If the engine traps (Ex. out of memory with no memory limit), the call rejects and the document is marked
  crashed: `doc.crashed` says why, and every later call on it throws "the Stof document crashed ... create a new
  document" (instead of "busy" forever). Other documents keep working.
  - Rust: `Runtime::async_*_with_gate` are replaced by `JsTask` (`start_call`, `start_call_source`, `start_run`,
    `step`, `fail`, `result`, `report` on the wasm `Stof`).
- **Network: any host means any public host.** `allow_http()` / `doc.allowHttp()` with no host list no longer reaches
  this machine or private networks: loopback, private ranges (10/8, 172.16/12, 192.168/16, carrier-grade NAT),
  link-local (169.254/16, where cloud metadata services live), IPv6 loopback/unique-local/link-local (and IPv4 written
  as IPv6), and the names `localhost`, `*.localhost`, `*.local`, `*.internal`, and `metadata`. A document allowed
  every host could otherwise read cloud credentials from the metadata service or call internal services from inside
  the host's network.
  - Native builds check the URL, every address a name resolves to, and every redirect before it's sent. JavaScript
    checks the URL as written and, on servers, every redirect before it's sent (fetch follows them in browsers);
    it can't see what a name resolves to, so list the hosts when that matters.
  - To reach them anyway: list them (`allow_http_hosts(["localhost:8080"])`, which works as before), or
    `graph.allow_private_network()` (Rust), `doc.allow_private_network()` (Python), `doc.allowPrivateNetwork()` (JS).
    The `stof` CLI should call it to keep allowing everything.
- **HTML: URL attributes are an allowlist, and `style`, `action`, and `formaction` are opt-in.** URL attributes
  (`href`, `src`, `srcset`, `action`, `poster`, ...) are written only when relative or `http`, `https`, `mailto`, or
  `tel` (and raster `data:` images), instead of dropping only `javascript:`, `vbscript:`, and other `data:`. `style`
  (which can restyle or cover the page) and form `action`/`formaction` (which can post elsewhere) are dropped unless
  the host allows them: `graph.allow_html_attributes(["style"])` (Rust), `doc.allow_html_attributes([...])` (Python),
  `doc.allowHtmlAttributes([...])` (JS).

### Added

- **Sealing: lock part of a document so it can be called but not changed.** A host seals an object (a path, or a
  type as `<Name>`) and everything under it: `graph.seal_path("<Engine>", false, None)` (Rust), `doc.seal('<Engine>')`
  (Python, JS). From then on its functions, prototypes, and the fields that hold functions or sealed objects can't
  be added, removed, renamed, replaced, or moved, sealed objects can't be removed, moved, renamed, or re-attributed,
  and no other type can take a sealed type's name (it would be found first). That holds for the document's own code
  (an agent's function, a customer's rule) and for host calls alike, until the host unseals (`unseal`; documents
  can't). A refused change fails the instruction or host call that made it with a `sealed: ...` error, and nothing
  changes. Before, any code in a document could replace another part's functions (Ex.
  `parse("fn price() -> int { 0 }", <Engine>)`).
  - A document can seal parts of itself too: `seal(<Engine>)` or `seal(self.rules, true)` (Ex. in an `#[init]`),
    returning the digest, and `digest(obj)`. It's one-way: documents can't unseal, only the host can.
  - Sealed code keeps working: its arrow functions, calls, and state (fields still change; seal with `fields` to
    freeze them too: `doc.seal(path, { fields: true })`, `fields=True`). A host's `callSource` on a sealed object
    still works.
  - Enforced where the document changes, so calls cost nothing more: the Limitr engine's suite (281 tests) runs as
    fast with its whole spec sealed.
  - What it guarantees: the sealed part's code doesn't change. Functions are found by name when called, so a sealed
    function calling into an unsealed part calls whatever is there: seal everything a call relies on.
- **Attributes can change at runtime.** `obj.set_attribute(name, value = null, path = null)` and
  `obj.remove_attribute(name, path = null)` (on the object, or the field, function, or object at the path, the same
  targets as `attributes`), and `func.set_attribute(name, value = null)` / `func.remove_attribute(name)`. Changes take
  effect at once: `funcs(attributes = ..)`, running by attribute, `#[readonly]`/`#[private]`, and exports all see them.
  Before, attributes were set only by the source. Attributes are behavior, so on a sealed object they never change,
  even when its fields may (`sealed: ...`). `#[type]` and `#[init]` act when a document is parsed, so setting them
  later doesn't register a type or run anything. Documents compiled to BSTF by earlier versions still load (the new
  instructions are added after the existing ones).
- **Seal digests and signatures.** `doc.digest(path)` is a SHA-256 over the part's functions (names, parameters,
  return types, attributes, instructions) and structure, plus its fields' values with `fields`. It's the same
  whether the document was parsed or loaded from BSTF (about 15 ms for the whole Limitr spec, natively), so a host
  can check what it's about to trust: `doc.seal(path, { digest })` seals only if the digest matches, and
  `doc.seal(path, { publicKey, signature })` only if an Ed25519 signature over the digest verifies. Sign at build
  time with `StofDoc.sign(digest, secretKey)` (JS), `Doc.sign(...)` (Python), or `stof::model::seal::sign` (Rust);
  keys and signatures are base64 (or hex), and a secret key is 32 random bytes (Ex. `openssl rand -base64 32`).
  - Compiling the same source now gives the same instructions: block and switch expressions share one return tag
    (they had a random one each; a return still goes to the nearest), and while a digest is computed, arrow function
    IDs are left out and attributes are sorted (BSTF is unchanged).
  - New dependencies: `sha2` and `ed25519-dalek` (pure Rust; `curve25519-dalek` was already in the js and full builds).

- **The npm package ships the Stof skill** in `skill/stof` (`node_modules/@formata/stof/skill/stof`): SKILL.md and the
  library references, copied from `content/SKILL.md` and `docs/libs` at publish (`npm run skill`, which also refreshes
  `plugin/skills/stof`). Packages built on Stof (Ex. Limitr) can point agents at it instead of shipping their own copy.
  `npm run readme` copies the README and LICENSE into `web` the same way.

### Changed

- **Library references use `Lib::func(..)`.** Every library function's docs (and so `docs/libs`, the skill's
  references, and the stof.dev library pages) now write calls as `Num::abs(x)`, `Http::fetch(url)`, and so on, instead
  of `Num.abs(x)`. Both work, but a `.` is a path that looks for an object named `Num` first; models learn from the
  references, so they now write the form that always means the library. The skill says so too.

### Fixed

- **Python: no more panics in the binding** (49 `unwrap`s): library functions take any number of arguments (was at
  most 9), a set holding values Python can't hash (Ex. lists) comes back as a list, and a map key Python can't hash
  becomes its text, instead of a panic.

### Tests

- `tests/js/check.mjs` (build `web/dist`, then `node tests/js/check.mjs`): calls and call-backs, host promises,
  concurrent calls, the memory limit, the network guard, html, and a crash. `tests/python/smoke.py` (install the
  wheel, then run it). CI runs both: the JavaScript job builds the engine with the wasm-bindgen CLI that matches the
  crate (its version comes from `project.stof`), and the Python job builds the wheel with maturin.
- `stof run project.stof` now writes `pyproject.toml` too, with `[tool.maturin]` naming the module (`pystof`) and its
  `py` feature, so `maturin build` and `pip install .` work without changing `Cargo.toml`.

## 0.10.9

### Fixed

- **Crashes and hangs found by a new robustness test are fixed.** A panic kills the whole engine in WebAssembly, and
  hosts run text people and models write. The test (below) parses and runs about 60,000 broken and edge-case inputs
  on every run (238,000 at `STOF_FUZZ_SCALE=4`); every one now ends in a value or an error. What it found:
  - `nanoid(length)` with a length of 0, a negative one, or a huge one looped forever (the execution time limit
    can't interrupt a single call). The length must be from 1 to 4096, else the error says so.
  - `Str.at` on an empty string crashed. It returns `''`.
  - Integer overflow, which a debug build (as in CI) stops on: `Time.add_months` with a huge count (in release it
    looped for seconds; now an error), `Time.add_days` with `-Infinity` days (an error), and `List.remove_last` on an
    empty list or with no match (null, as before). Integer arithmetic itself was already safe: it becomes a float.
  - `Str.at`, `first`, `last`, and `substring` on text that isn't ASCII returned part of a character, or crashed when
    an offset fell inside one. Offsets are still bytes, like `len` and `index_of`; an offset inside a character means
    that character (`'日本'.at(1)` is `'日'`, and `substring` starts at its first byte).
- **JavaScript: a synchronous call back into a running document throws, instead of crashing the engine.** A
  `doc.lib(..)` function can still call back with the async methods (`await doc.call(..)`), which run after Stof's
  current step, as before. A synchronous method (`get`, `set`, `sync_call`, an export) called during the step
  (before the function's first await) found the document borrowed and aborted the WebAssembly module; it now throws
  "the document is busy running a step ..." with what to do instead.
- **JavaScript: a call gate that throws or rejects ends the calls with an error**, instead of crashing the engine.
  Where `setTimeout` isn't available, a long call skips its yield to the event loop instead of crashing.
- A type name that isn't a type expression (from a host or a library) is an object type's name, instead of a crash.
- **Infinity and NaN in every format.** Each value comes back as the number it was, or is written the way the format
  writes numbers:
  - JSON has no infinity or NaN: it wrote `null` (the value was lost). It writes the strings `"Infinity"`,
    `"-Infinity"`, and `"NaN"` (valid JSON; JavaScript's `Number()` reads them), and JSON import reads exactly those
    strings as the numbers. Other words (`"inf"`, `"nan"`, `"infinity and beyond"`) stay strings.
  - YAML wrote `null` too. It writes YAML's own `.inf`, `-.inf`, and `.nan`, and reads them (and the JSON words).
  - urlencoded reads `Infinity`, `-Infinity`, and `NaN` as numbers, like JSON (other words stay text, as before).
  - TOML (`inf`, `nan`), BSTF, and Stof already round-tripped.
  - Printed numbers (`str(x)`, templates, `pln`, urlencoded, XML, HTML, `stof:human`) say `Infinity`, `-Infinity`,
    and `NaN` instead of Rust's `inf`, so printed text reads back as Stof.
- **Dropped objects no longer stay in memory.** A graph kept every removed object and field in its deadpools, and
  nothing emptied them, so every object created and dropped (`using`, `drop`, `remove`, `call_source`) stayed in
  memory for the life of the document: about 1.5 to 9 KB each. A Limitr `allow()` left about 14 KB behind per call.
  Deadpools are now off by default, which is how the CLI has always run, and the same calls stay flat (0 bytes per
  call after warm-up). Affects every embedding: JavaScript and WebAssembly (`StofDoc`), Python (`Doc`), and Rust
  (`Graph::default()`).
  - Rust code that reads `node_deadpool` or `data_deadpool` (Ex. for `flush_join`) turns them on with
    `graph.set_deadpools_enabled(true)` and empties them with `flush`, `gc_table(.., true)`, or the clear functions.
- **XML export of an unnamed object failed about one time in six.** `stringify('xml', new { a: 1 })` named the root
  element after the object's random ID, which isn't an XML name when it starts with a digit. An object without a
  name of its own is written as `<root>`.
- `roundtrip_bstf_types` compared a timestamp with `<` and failed when the test ran within one millisecond; it
  compares with `<=`.

- **A passing `cargo test` reads as passing.** With `--nocapture` (as CI ran it) a green run printed hundreds of
  lines that looked like failures: errors and warnings from tests that check errors, and the readable-export test's
  re-run of the suite, whose object ID tests fail by design. Those graphs are quiet now (`Graph::quiet`, below), and
  the run prints the suite's own report and the tests' intended output.
- A name for a library that only some builds have (`Age`, `Http`, `fs`, `Image`, `Pdf`, `Md`) no longer warns
  "unknown name ... always null" in a build without it. The same document can run in a host that has it, and code
  guards it with `lib('Age')`.
- The Http tests (all features) failed whenever GitHub's limit on requests without a token was reached (403), as on
  shared CI runners. A limited response still counts as a response.
- CI runs the tests with all features too, and checks the JavaScript build. It no longer passes `--nocapture`, so a
  run shows output only for a test that fails.
- The JavaScript build no longer warns about an unused import (`imbl::vector` in the Http library).

### Added

- **Messages on asserts.** `assert`, `assert_not`, `assert_eq`, `assert_neq`, `assert_null`, and `assert_not_null`
  take an optional last argument, written first in the error: `assert_eq(total, 42, 'total after the refund')`
  fails with "total after the refund: '43' does not equal '42'". Without one, errors read as before.
- **`Infinity`, `-Infinity`, and `NaN` literals** (whole words: `Infinity_cap` and `NaNcy` are still names). The
  readable `stof` export writes them (it wrote `inf()` and `nan()`), and keeps units on an infinite value
  (`Infinity as kg`, which it used to drop).
- **The message of a caught error.** `catch (e, message)` names the readable message too (Ex. `e` is
  `'FuncArgs'` and `message` is "nanoid length must be from 1 to 4096 (got 0)"). `e` is unchanged, so code that
  compares codes keeps working, and documents compiled before 0.10.9 still run.
- **A clearer warning for a map key that's a name.** `{ b: 1 }` inside an expression is a map whose key is the
  variable `b`, not an object. When `b` isn't a variable, the warning says so and shows `new { b: .. }` (an object)
  and `{ 'b': .. }` (a text key).
- **`quiet`: print nothing to the console.** `graph.quiet(true)` (Rust), `doc.quiet()` (JS and Python) turns off
  parse warnings, test progress, init failures, and `pln`/`err`/`dbg` output, for hosts that log their own way.
  Results and errors still come back: `test` returns its report, and calls return their errors.
- `tests/robustness.rs`: every `.stof` file in the repo cut off and randomly edited, every library function called
  with edge values, and every format parsing garbage and writing odd objects. It fails on any panic, and a watchdog
  fails it when one input runs for 15 seconds. `STOF_FUZZ_SCALE=N` runs more.
- `tests/memory.rs`: calls 13 kinds of functions (objects with `using`, nested objects, arenas, `drop`, typed
  objects with constructors, `call_source`, lists, maps, try and catch) 20,000 times each, and fails if the graph
  or the heap grows (live bytes, counted by the test's own allocator).

### Changed

- The README and the package descriptions lead with what Stof is for: software that behaves differently for each
  user, starting with agents. Each customer's rules, tools, and state live in one document that an agent can edit, a
  person can review, and the app runs. The first example is a per-customer refund tool.

## 0.10.8

### Added

- **Edit Stof text in place.** `outline(text)` lists a text's fields and functions with their paths and where they
  are, from the parser itself. `edit(text, edits)` sets, adds, removes, and renames fields and functions by path
  and changes nothing else: comments, formatting, and every other definition stay exactly as they were. It's for apps
  and agents that change documents people also write by hand (Ex. a no-code editor saving into a hand-written
  policy).
  - Edits: `{ set: path, value }` (a JSON value), `{ set: path, stof }` (Stof source), `{ set_fn: path, stof }` (a
    function: an existing one keeps its attributes and doc comments unless the source starts with its own),
    `{ remove: path }`, and `{ rename: path, to }`. A path is a list of names, or names separated by dots.
  - The outline gives each definition's path, whether it's an object, its value (a field's value, or a function from
    `fn`), and its source (with its attributes and the comments directly above it).
  - A set on a field that isn't there adds it, and any objects on its path, in the style of the object it goes
    into: one field per line or on one line, its separators, and its indent. Added text uses the text's line
    endings.
  - A remove takes the definition's attributes, the comments directly above it (not past a blank line), its
    separator, and a comment at the end of its line.
  - Each edit is checked: the text has to parse before and after, a path has to be defined once, and only objects
    written inline have children. A failed edit changes nothing.
  - Nothing in the text runs: field expressions and attribute values aren't evaluated, and init functions aren't
    called. So any text is safe to outline or edit, including one a user or an agent sent.
  - Typed fields don't need their types (Ex. `Limitr policy: {..}` outlines without the Limitr engine).
  - Rust: `stof::parser::edit::{outline, edit, Edit}`; JS: `StofDoc.outline(text)` and `StofDoc.edit(text, edits)`;
    Python: `Doc.outline(text)` and `Doc.edit(text, edits)`.
- **Readable Stof export.** `stringify('stof', obj)` (and host exports in the `stof` format) now writes Stof a person
  or a model can read and edit, instead of the exact graph form:
  - objects with their types (`Plan pro: { .. }`), attributes, and doc comments
  - values as literals: units (`1GB`), versions, lists, tuples, and blobs, with `set(..)` and `map((k, v), ..)`
  - functions as their source text, when the host asked the parser to keep it (see below)
  - objects in lists as `new Type { .. }` when they only hold values, else as `#[no-field]` objects before the field
  - references to other objects as paths (`self.a`, `super.b`), and type definitions first so they're defined before
    they're used

  Function source is kept only when the host turns it on before parsing: `graph.keep_source(true)` in Rust,
  `doc.keepSource()` in JS, `doc.keep_source()` in Python. It's off by default, so documents don't hold the text of
  every function. A function without source text (not kept, or loaded from BSTF) is written as a `data@v1` block, and
  nothing is lost.
  Comments, formatting, and object IDs aren't kept. The Stof test suite, written this way and parsed back, passes
  except the tests of declared object IDs.
- **`stof:graph`** is the exact export the `stof` format used to write: IDs, references, and compiled functions as
  data blocks. `stof:human` (the same with comments) is unchanged.
- **`Obj.set(name, value)` and `Obj.unset(name, shallow = false)`:** set, create, or remove the field with exactly this
  name, never a path, so a name can have dots. They're the counterparts of `Obj.get`. `insert` and `remove` still
  take paths.
- Parse info (for editor tooling) records byte spans for fields and functions (`Def::span`).

### Changed

- **`a ?? b` costs less.** It's the value and one check now, instead of a jump to a generated tag.
- **`Age.blobify(.., 'stof', ..)` encrypts readable Stof** (the `stof` export). Decrypting with `Age.parse` reads it
  the same way. Pass `'stof:graph'` for the exact form.
- **A variable declared without a type takes no type from its value.** `let x = self.price` (a `float` field) and
  then `x = 'text'` used to fail with "cannot cast str to float". A declared type (`let x: float = ..`) still holds.
  The field itself is never changed by assigning to the variable.
- **Documents use about half the memory.** An empty document went from about 2.2 MB to 0.24 MB, and the Limitr
  engine from about 7.4 MB to 3.5 MB. BSTF is unchanged: documents written by 0.10.7 load in 0.10.8, and the reverse.
  - Function parameters (`Func::params`, `LibFunc::params`) are `Params` (`Arc<[Param]>`), sized to the parameters.
    The `imbl::Vector` they used allocated room for 64 parameters each. Rust hosts building a `LibFunc` write
    `params: params![..]` instead of `vector![..]` (or any `Arc<[Param]>`).
  - Each instruction takes less room: the few large, rarely used instructions are shared or boxed, since every
    instruction is as large as the largest.
- Resolving a call no longer copies a library function just to check that it exists (`Graph::has_libfunc(lib, name)`).

## 0.10.7

### Added

- **HTML and XML formats.** `stringify('html', obj)` and `stringify('xml', obj)` write objects as elements, from three
  attributes (plus the existing `#[no-export]`):
  - `#[tag('name')]` names the element. It can go on the field, the object, or a type (instances inherit it);
    without one, the element is named after its field.
  - `#[attr]` (or `#[attr('data-id')]`) makes a field or a function an attribute.
  - `#[text]` writes a value as escaped text inside the element. A `#[text]` function can return objects, which are
    written as elements.
  - Child objects are child elements, lists are one element per item, and other values are `<name>value</name>`, all
    in field order. Lists, maps, and objects in attributes are JSON. Null values and unmarked functions aren't
    written.
  - **HTML is safe to put in a page**, whatever the document holds (Ex. a view an AI model wrote):
    - text and attribute values are always escaped
    - `script`, `style`, `iframe`, `frame`, `object`, `embed`, `applet`, `base`, `link`, `meta`, `noscript`, and
      `template` are never written, with their content
    - `on*` attributes, `srcdoc`, and `javascript:`/`vbscript:` URLs are dropped, and so are `data:` URLs other
      than images
    - names that aren't element or attribute names are never written
    - only the standard HTML elements and custom elements (with a `-` in the name) are written by default. A host
      can allow exactly a list instead (`graph.allow_html_tags(..)` in Rust, `doc.allowHtmlTags([..])` in JS,
      `doc.allow_html_tags([..])` in Python, Ex. its own components), and documents can't change it.
    - An html export of an object without a tag is a fragment (its content, with no element around it).
  - **XML** starts with a declaration, writes empty elements as `<name/>` and booleans as `true`/`false`, and fails
    on names that aren't valid XML. Attribute values keep newlines and tabs as character references.
  - **XML import** (`parse(src, obj, 'xml')`; the new `xml` feature, in `full` and `js`, so in the CLI, Python, and
    JavaScript) is the reverse of the export, so an exported document parses back to the same document: attributes become `#[attr]` string fields, an element with
    only text becomes a string field, repeated elements become a list, and text next to attributes or elements
    becomes a `#[text]` field named `text`. Values stay strings. The root element's content goes into the object
    being parsed into, which gets `#[tag]` when the names differ. Elements with different names that alternate come
    back grouped by name.
- **Computed values in exports.** A format can ask for functions to be called before it writes (Ex. `#[attr]`
  functions and fields holding functions in html and xml). `stringify` and `blobify` call them in the running process,
  so async functions are awaited and errors are reported like any call. A computed value can be an object with
  computed values of its own. Host exports (`Graph::stringify`/`blobify` in Rust, `stringExport`/`binaryExport` in
  JS, `string_export`/`binary_export` in Python) call them too, with a runtime of their own, so an async host function
  can't finish there; call `stringify` from Stof for those. Formats implement `Format::computed` and
  `string_export_computed` (defaults: nothing to compute, the plain export).
- **`List.map(func)`, `List.filter(func)`, and `List.any(func)`.** Each calls the function for every item, in order,
  in the running process. `map` of an async function returns its promises (await the list). `any()` without a
  function still checks for values.

### Fixed

- **`Obj.run` awaits async steps.** An `async fn` step's promise was dropped, so the step finished after `run()`
  returned (and after the steps that came next). Each step now finishes before the next one starts.
- **`funcs(attributes)`, `obj.funcs(..)`, and running functions by attribute (`doc.run('attr')`, `#[main]`,
  `#[test]`) go in definition order.** They came from a hash set, so their order changed between runs. Functions on an
  object are in the order they're written; across a document, in the order of the objects that hold them.
- **`List.sort_by` is a stable sort, and an error in the compare function fails it.** Equal values could change
  order, and a compare function that threw (Ex. a parameter type that doesn't match the items) was read as "equal",
  leaving the list unsorted without an error.
- **A cast that isn't possible says so.** It failed with "not implemented"; it's now "cannot cast (int, str) to
  tup" (the value's type and the target). Ex. a parameter typed `tup`, which is an object type named `tup`: tuples
  are typed by their items, `(int, str)`.

## 0.10.6

### Fixed

- **Evaluation order.** Binary operators (`+`, `-`, `*`, `<`, `==`, `&`, ...) evaluated their right side first, and
  template strings their parts from last to first, so side effects ran out of order: on `[1, 2, 3]`,
  `items.len() + items.pop_back()` was `5` (len after the pop) instead of `6`. Everything now runs left to right, like
  calls, lists, and maps already did. Compound assignments (`x += f()`) read the variable first. Results without side
  effects are unchanged.
  - New `Base::Binary(Op)` instruction (appended; BSTF-compatible) for operands evaluated left to right. Documents
    exported before keep the original instructions and run as they did.
- **A call to a function that returns nothing gives null where its value is used.** `const x = self.log();`,
  `[1, self.log()]`, `f(self.log())`, `self.log() ?? 'default'`, and `await` on an async function that returns
  nothing (alone or in a list: `await [a(), b()]`) failed with "internal runtime error (StackError)". They're null
  now. A function that ends with a call to one that returns nothing (`fn f() { self.log() }`) still returns nothing.
  - New `ValueIns` instruction and `Base::EnsureValue` (appended; BSTF-compatible), only around calls and awaits in
    those positions.
- **Arrow functions keep the variables where they were made.** An arrow passed to another function, or stored and
  called later, lost the variables around it (`self.apply(100, (x: float): float => x * (1 + tax))` failed with a null
  error). Arrows now capture the outer variables their body uses when they're evaluated. They still see the variables
  of the function they're called from first, so code that worked before sees the same values. Assigning to a captured
  variable updates the original (`let total = 0; self.each(list, (x: int) => { total += x; });`). Captures are
  runtime only: an arrow exported to BSTF captures again when it's evaluated. An arrow made in a loop keeps the
  variables of its latest evaluation.
- **Schema rules with one parameter get the value being checked**, whatever it's named. `(v: str): bool => ...` got the
  field's name, and `(value: int): bool => ...` got the type's default, so those rules quietly checked the wrong thing.
  A rule with more parameters, or one named `schema`, `target`, `field`, or `schema_val`, is matched by name as before.
- Method calls on template strings: `` `hi ${name}`.upper() `` was a parse error.
- `Fn` library calls on a function by its path: `self.price.params()` was "function not found" (it only worked
  through a variable).
- Doc comments (`///`, `/** */`) before `root Name { }` were a parse error.
- An unknown escape in a string (Ex. `'\.'` in a regular expression) reported "expected ')'". It now says
  `unknown escape '\.' in a string`, at the escape, and suggests `\\` or a raw string (`r#"..."#`).
- **`#[run]` steps from a type run once, on the instance.** A type's object step ran twice on each instance and once
  on the type itself (changing the type's defaults), and a subtype's step ran alongside the step it was meant to
  replace. See the `Obj.run` changes below.
- **Parsing into an object again merges into its object fields** instead of replacing them. Parsing the same source
  twice (or `parse("nested: { x: 5 }", obj)`) left the old object behind as an unreachable child and dropped fields the
  new source didn't mention. Now the parse continues into the existing object, the way it does for the object it
  started in: new fields are added and existing ones updated. This applies to Stof, JSON, YAML, and TOML, and to
  objects declared twice in one document (`#[type]` objects included, so a type is no longer registered twice).
  A field that refers to an object somewhere else is still replaced.

### Added

- **Call a function from source without adding it to the document.** For one-off logic, like a check or a script
  from a model, that used to need a `drop(this);` at the end. The source's first function is called as if it were
  defined on the object (the root by default): `self` is that object, and the source's other functions are helpers
  it can call through `self`. Everything the source declares is removed after the call, even when it fails. Changes
  the function makes to the document stay. A source function with the same name as one the object already has is
  an error rather than replacing it.
  - JavaScript: `await doc.callSource(src, args = [], node = null)` and `doc.sync_callSource(...)`.
  - Python: `doc.call_source(src, args = None, node = None)`.
  - Rust: `Runtime::call_source(&mut graph, src, context, args)` (and `async_call_source`).
- **Remove a field, function, or object by path from the host**, like `drop(path)` in Stof:
  `doc.remove('pricing.old_total')` (JavaScript and Python) and `graph.remove(path, shallow)` (Rust). With
  `shallow`, a field that holds an object is removed without dropping the object. Rust also gets
  `graph.drop_path(path, context, shallow)`, which `drop` now uses.

### Changed

- **`Obj.run` steps.** Every field and function with `#[run]` is a named step (its name, or
  `#[run({'name': 'x'})]`), and:
  - Steps run in the order they're written. Before, unnumbered fields ran before unnumbered functions, and
    functions in any order.
  - A type's steps run on each instance, before the instance's new steps (base type first). Before, the instance's
    steps ran first unless the object had `#[run({'prototype': 'first'})]`.
  - A step with the same name as a type's step replaces it, in its place, so each step runs once. `#[run] step: null`
    removes a type's step.
  - A number orders every step together, like a z-index: unnumbered steps are 0, `#[run(-1)]` runs before them and
    `#[run(1)]` after. A replacing step without a number keeps the number of the step it replaces.
  - `obj.run(prototype = 'first' | 'last' | 'none' | 'only', steps = ['a', 'b'])`: which steps run (`'none'` only
    the object's own steps, `'only'` only the type's), or just the named steps in that order. An object can set
    either with its own attribute: `#[run({'prototype': 'none'})]`, `#[run({'steps': [...]})]`.
  - Numbered steps, `args`, nested objects, and lists of steps (pipelines) work as before.
- Faster number parsing: a number not followed by a unit (most of them) skips the unit lookup, which was a large
  share of parse time in function-heavy documents. Parsing stays as fast as 0.10.5 with this release's other changes.
- **`pln`, `dbg`, and `err` write to the console in JavaScript** (`console.log` and `console.error`), with the same
  formatting as the native runtime. Before, their output was lost in WebAssembly unless the host replaced `Std.pln`.
- JavaScript `doc.stringify(format, node)`, `doc.blobify(format, node)`, and `doc.record(node)`, and Python
  `Doc.string_export` and `Doc.binary_export`, take an object's path (Ex. `'config.production'`) as well as its ID,
  like `get`, `set`, and `call`.

### Agent skill

- `#[private]` limits which functions can read a field; it doesn't keep the field out of exports (`#[no-export]`
  does). The skill said otherwise.
- Notes on one-parameter schema rules and arrow function captures.
- `#[run]` steps: definition order, type steps and overrides, numbers, and `run(steps = ..)`.
- Host `callSource`/`call_source` and `remove` in the embedding examples.

## 0.10.5

### Added

- **Parse info for editor tooling** (`parser::info`, used by the Stof language server, `stof lsp`).
  `parse_with_info(graph, format, path, overlay, profile)` parses a Stof file, or a whole package (`"pkg"`) so a
  file is seen with the types and fields of its other files, and records where every root, field, function, and
  import is defined (with doc comments), plus warnings with locations. `overlay` texts are parsed instead of the
  files at those paths (Ex. unsaved editor buffers). Init functions don't run. Normal parsing is unchanged: nothing
  is recorded unless a parse asks for it.
- **Destructuring declarations.** Take values apart into variables with `let` or `const`:
  - `const { name, age: years, role = 'member', address: { city } } = user;` fields of an object (or string keys
    of a map), with renames (`age: years`), defaults for missing or null values (`role = 'member'`), and nesting.
  - `const [first, _, third] = list;` and `const (key, value) = pair;` items of a list or tuple by position
    (`_` skips one). Either bracket works for lists and tuples.
  - In loops: `for (const (key, value) in map)`, `for (const { name, age } in users)`.

  A missing field or item is null (or its default). Destructuring a value of the wrong kind (Ex. null) is an error.

## 0.10.4

### Added

- **Execution time limit for hosts.** A call fails with an execution timeout after 2 minutes (as before); hosts can
  now change it: `graph.max_execution_time = Some(duration)` (Rust), `doc.set_max_execution_time(seconds)`
  (Python), `doc.setMaxExecutionTime(ms)` (JS). `None`/`null` removes the limit.
- **HTTP host allowlist.** `graph.allow_http_hosts(["api.example.com"])` (Rust), `doc.allow_http_hosts([...])`
  (Python), `doc.allowHttp(['api.example.com'])` (JS) give the `Http` library access to those hosts only: a request
  to another host throws, and a response redirected to another host returns an error instead. An entry is a host
  (any port) or `host:port`. `allow_http()` is unchanged (any host).

### Fixed

- Parsing BSTF into an object of a document that already has some of its objects (Ex. diffing a policy with its own
  export, `parse(blobify('bstf', obj), copy, 'bstf')`) overwrote the existing objects: they moved into the import
  and were dropped with it. Objects and data whose IDs are taken now get new IDs, so the import is a copy; other
  IDs are kept, as before. (`Graph::reassign_ids_used_in`. Importing at the root is unchanged.)
- `Obj.diff` compared objects inside lists by ID, so a list of objects (Ex. price tiers parsed from JSON) always
  showed as changed. Objects now match by their fields, including inside lists, tuples, and maps. A list with any
  difference is still kept whole in the result.
- `Obj.upcast()` set the object's prototype to a data ID instead of the prototype's own prototype, so the object
  lost its type (`typename` was an ID and its functions were gone). It now moves up one type, as documented.
- Moving an object to the parent it already has (`move_node` with the current parent, Ex. `obj.move(parent)`)
  detached it: it was added to the parent (no change), then removed from its "old" parent (the same one), leaving
  an object no field could reach and nothing dropped. It's now a no-op that succeeds.

### Performance

These keep every result the same; they remove work that grew with the size of the document.

- `drop(obj)` (and `using` cleanup) removes fields anywhere that hold the dropped object or one of its children.
  That searched all of the document's data once per dropped node. Objects now track the field that holds them
  (`Node::holders`: none, one, or many/unknown), so a drop removes that field directly; only an object that more
  than one field may hold (Ex. one assigned to a variable or a second field) still searches, once per drop.
- `funcs(attributes)` (Ex. `funcs(attributes = 'my-event')`) read every node's data recursively. It's now one pass
  over the document's data.
- Prototype lookups (method calls on objects, type checks, ...) scanned all of the object's data for its prototype.
  They now go straight to it (`__proto__`, now `PROTOTYPE_DATA_NAME`), so a call on an object with many fields
  (Ex. an object of customers) no longer depends on its size.
- Loops trim their execution history in place, and method lookups split paths without allocating.

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
