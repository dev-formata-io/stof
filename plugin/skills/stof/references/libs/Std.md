# Standard Library (Std)
Functions in the 'Std' library are foundational to Stof and therefore do not requre one to explicitly reference 'Std' when calling them. Within the standard library, you'll find functions for asserting values, printing to the console, throwing errors, putting processes to sleep, etc. Note for advanced users that it is possible to extend or modify this library as needed.

## Example Usage
```rust
#[main]
fn main() {
    Std::pln("printing a line");
    pln("printing another line"); // no 'Std' needed for this library
}
```

# Std::assert(value: unknown = false, message: str = null) -> void
Throw an error if the given value is not truthy. An optional message comes first in the error.
```rust
assert(true);
assert(false); // errors
assert(self.items.len() > 0, 'items should not be empty'); // errors with the message
```


# Std::assert_eq(first: unknown, second: unknown, message: str = null) -> void
Throw an error if the first value does not equal the second. An optional message comes first in the error.
```rust
assert_eq('a', 'a');
assert_eq(43, 42); // errors
assert_eq(total, 42, 'total after the refund'); // errors: "total after the refund: '43' does not equal '42'"
```


# Std::assert_neq(first: unknown, second: unknown, message: str = null) -> void
Throw an error if the first value equals the second. An optional message comes first in the error.
```rust
assert_neq('a', 'b');
assert_neq(34, 34); // errors
assert_neq(new_id, old_id, 'a copy gets a new id'); // errors with the message
```


# Std::assert_not(value: unknown = true, message: str = null) -> void
Throw an error if the given value is truthy. An optional message comes first in the error.
```rust
assert_not(false);
assert_not(true); // errors
assert_not(self.deleted, 'the record was deleted'); // errors with the message
```


# Std::assert_not_null(value: unknown, message: str = null) -> void
Throw an error if the given value is null (or void). An optional message comes first in the error.
```rust
assert_not_null(42);
assert_not_null(null); // errors
assert_not_null(self.customer, 'customer was not created'); // errors with the message
```


# Std::assert_null(value: unknown, message: str = null) -> void
Throw an error if the given value is not null (void, the result of a function without a return value, counts as null). An optional message comes first in the error.
```rust
assert_null(null);
assert_null(42); // errors
```


# Std::blobify(format: str = "json", context: obj = null) -> blob
Use a loaded format to export a binary blob from the given context (or entire graph/document). The default format is json, and the standard implementation only exports object fields. Export results will vary depending on the format, some support more than others (it is up to the format implementation to decide how it exports data). You can always create your own to use.
```rust
const object = new { x: 3.14km, y: 42m };
const export = blobify("json", object); // json string like "stringify", but as a utf8 blob
assert(export.len() > 0);
```


# Std::callstack() -> list
Return the current callstack as a list of function pointers (last function is 'this').
```rust
// inside a function call
for (const func in callstack()) {
    pln(func.obj().path(), ".", func.name());
}
```


# Std::copy(val: unknown) -> unknown
Deep copy the given value. If this value is an object (or contains one), recursively deep copy the object (all fields, funcs, & data).
```rust
const a = {1, 2, 3};
const b = copy(a);
b.clear();
assert_neq(a, b);
```


# Std::dbg(..) -> void
Prints all arguments as debug output to the standard output stream.
```rust
dbg("hello, world");
```


# Std::dbg_tracestack() -> void
Print a snapshot of the current stack.


# Std::digest(obj: obj, fields: bool = false) -> str
A SHA-256 (hex) over an object's functions and structure (its functions' names, parameters, return types, attributes, and instructions, and the objects, types, and fields that hold functions or objects under it), plus every field's value when `fields` is true. The same for the same document, whether it was parsed or loaded from BSTF, and wherever the object is (its own name isn't part of it), so an object sent to another document has the same digest there. It identifies exactly the logic (and with `fields`, the data) it covers. Hosts check it when they seal (`doc.seal(path, { digest })`) or verify a signature over it.
```rust
const before = digest(self.rules);
assert_eq(digest(self.rules), before);
```


# Std::drop(..) -> bool | list
Drop fields (by str path), functions (path or fn), objects (path or obj), and data from the graph. Objects will have their #[dropped] functions called when dropped. When dropping multiple values at once, this will return a list of booleans indicating a successful removal or not for each value.
```rust
const func = () => {};
const object = new {};
const results = drop("self.field", func, object);
assert_eq(results, [true, true, true]);
```


# Std::env(var: str) -> str
Get an environment variable by name. Requires the "system" feature flag.
```rust
const var = env("HOST");
```


# Std::env_vars() -> map
Get a map of the current environment variables (str, str). Requires the "system" feature flag.
```rust
const vars: map = env_vars();
```


# Std::err(..) -> void
Prints all arguments to the error output stream.
```rust
err("hello, world");
```


# Std::exit(..) -> void
Immediately terminates this (or another) Stof process. Pass a promise into this function to terminate it's processes execution.
```rust
const promise = async {
    sleep(10s);
};
exit(promise);
```


# Std::format(format: str) -> bool
Is the given format loaded/available to use?
```rust
assert(format("json"));
assert_not(format("step"));
```


# Std::format_content_type(format: str) -> str
Returns the available format's content type (HTTP header value), or null if the format is not available. All formats are required to give a content type, even if it doesn't apply to that format.
```rust
assert_eq(format_content_type("json"), "application/json");
```


# Std::formats() -> set
A set of all available formats, available to use with parse, stringify, and blobify.
```rust
const loaded = formats();
assert(loaded.contains("json"));
```


# Std::funcs(attributes: str | list | set = null) -> list
Get a list of all functions in this graph, optionally filtering by attributes (single string, list of strings, set of strings, or tuple of strings). Functions in a confined part (see `seal`) aren't found: call them through the part.
```rust
for (const func in funcs({"test", "main"})) {
    // all test and main functions in the graph
    // call them or whatever you need
}
```


# Std::graph_id() -> str
Return this graph's unique string ID.
```rust
assert(graph_id().len() > 10);
```


# Std::inf() -> float
Positive infinity (negate it for negative infinity). To check whether a number is infinite, use Num::inf(val) (Ex. val.inf()).
```rust
const limit = inf();
assert(limit > 1e308);
assert(-inf() < -1e308);
assert(limit.inf());
```


# Std::lib(lib: str, func?: str) -> bool
Is the given library (and optional function) loaded/available to use?
```rust
assert(lib("Std")); // standard library is loaded
assert_not(lib("Render")); // no "Render" library loaded

assert(lib("Num", "abs")); // Num::abs(..) lib function is available
```


# Std::libs() -> set
Set of all available libraries. This will most likely include standard libraries like Std, Fn, Set, List, etc.
```rust
assert(libs().superset({"Std", "Fn", "Num", "Set"}));
```


# Std::list(..) -> list
Construct a new list with the given arguments.
```rust
assert_eq(list(1, 2, 3), [1, 2, 3]);
```


# Std::log_debug(..) -> void
Logs all arguments as debug info using the "log" crate.
```rust
log_debug("this is what just happened, in case you need to debug me");
```


# Std::log_error(..) -> void
Logs all arguments as an error using the "log" crate.
```rust
log_error("we have a problem");
```


# Std::log_info(..) -> void
Logs all arguments as info using the "log" crate.
```rust
log_info("we just did something cool");
```


# Std::log_trace(..) -> void
Logs all arguments as a trace using the "log" crate.
```rust
log_trace("we have a problem");
```


# Std::log_warn(..) -> void
Logs all arguments as a warnging using the "log" crate.
```rust
log_warn("we encountered something, but are handling it");
```


# Std::map(..) -> map
Construct a new map with the given arguments (tuples of key & value). Helpful as a way to create an empty map.
```rust
assert_eq(map(("a", 1), ("b", 2)), {"a": 1, "b": 2});
```


# Std::max(..) -> unknown
Return the maximum value of all given arguments. If an argument is a collection, the max value within the collection will be considered only.
```rust
assert_eq(max(1km, 2m, 3mm), 1km);
```


# Std::min(..) -> unknown
Return the minimum value of all given arguments. If an argument is a collection, the min value within the collection will be considered only.
```rust
assert_eq(min(1km, 2m, 3mm), 3mm);
```


# Std::nan() -> float
Not a number (NaN). NaN never equals anything, including itself: check for it with Num::nan(val) (Ex. val.nan()).
```rust
const value = nan();
assert(value.nan());
assert_neq(value, value);
```


# Std::nanoid(length: int = 21) -> str
Generate a URL safe random string ID, using the nanoid algorithm with a specified length (default is 21 characters, from 1 to 4096). Probability of a collision is very low, and inversely proportional to ID length.
```rust
assert_neq(nanoid(), nanoid(33));
```


# Std::parse(source: str | blob, context: str | obj = self, format: str = "stof", profile: str = "prod") -> bool
Parse data into this document/graph at the given location (default context is the calling object), using the given format (default is Stof). Formats are extensible and replaceable in Stof, so use whichever formats you have loaded (json, stof, images, pdfs, docx, etc.).

Profiles: "prod" (the default; #[test]s left out), "test", "docs", "prod_docs", and "untrusted": Stof source from somewhere else, declarations only. Nothing runs while it's parsed (field and attribute values must be literals; no #[init] or #[dropped] functions), it imports nothing, and it declares no #[type]s, roots, or binary data. Parse it into a fresh object (drop it if the parse fails), then seal that object confined before calling anything in it. Confined code's own parse is always "untrusted", and can't parse "bstf" or "pkg".
```rust
parse("fn hello() -> str { \"hello\" }");
assert_eq(self.hello(), "hello"); // can now call it

using tool = new {};
parse(source_from_elsewhere, tool, 'stof', 'untrusted');
seal(tool, removable = true, reads = [], hosts = ['api.example.com'], libs = ['Std', 'Str', 'Http'], time = 2s);
```


# Std::peek(..) -> void
Trace this location within your code execution. Will print out your arguments plus process debug information and the next instructions on the instruction stack. If the last argument given is an integer value, that number of (future) instructions will be shown (very helpful for deeper debugging).
```rust
peek("Getting here"); // will print "Getting here", then output a trace of the current process info and next 10 instructions to be executed
peek(70); // next 70 instructions
```


# Std::pln(..) -> void
Prints all arguments to the standard output stream.
```rust
pln("hello, world");
```


# Std::prof(name: str) -> bool
Is/was this graph parsed last with the given profile name?
```rust
// is the current profile named "test"?
const test = prof('test');
```


# Std::prompt(text: str = '', tag?: str) -> prompt
A helper function to create a prompt.
```rust
let p = prompt(tag = 'instruction');
p += prompt('do a thing', 'sub');
p += prompt('another thing', 'sub');
assert_eq(p as str, '<instruction><sub>do a thing</sub><sub>another thing</sub></instruction>');
```


# Std::remove_env(var: str) -> void
Remove an environment variable by name. Requires the "system" feature flag.
```rust
remove_env("HOST");
```


# Std::seal(obj: obj, fields: bool = false, removable: bool = false, reads: list = null, hosts: list = null, libs: list = null, time: ms = null, calls: list = null) -> str
Seal an object (Ex. `self.rules`, or a type: `<Engine>`) and everything under it: from now on its functions can be called but not added, removed, renamed, replaced, or moved, and its attributes can't change, by this document's code or by the host. Its fields still change unless `fields` is true. Returns its digest (see `digest`). It's one-way: a document can't unseal, only the host can. A change a seal refuses fails with a `sealed: ...` error and changes nothing.

- `removable`: the object can still be dropped whole (nothing in it can change, and it can't be moved or renamed). For code that comes and goes, Ex. a provider's capabilities.
- `reads` (a list of objects or paths): confine the part's code. While it runs (and everything it calls), it finds only the sealed object and these objects (a path to anything else finds nothing, calling a function elsewhere fails), changes only the sealed object, and can't hand an object elsewhere to a library function (Ex. `stringify`). `[]` means only itself. They're kept as paths and looked up each time the part is entered, so an object replaced at a path is the one it reads.
- `hosts` (a list of host names): `Http.fetch` from the part's code reaches only these hosts (the document's own allowlist still applies). `[]` means no network.
- `libs` (a list of library names, Ex. `['Std', 'Str', 'Num', 'List', 'Map', 'Http']`): the part's code calls only these libraries, the host's own libraries included (Ex. one a host added to sign with its key), so code from somewhere else can't use what the host gave the document. Method calls count as their library's (`text.split('/')` is `Str`). `[]` means no library calls at all. Even with `Std`, confined code can't call `exit` (it would end the code that called the part too) or the host's environment (`env`, `env_vars`, `set_env`, `remove_env`).
- `time` (Ex. `500ms`): the part's code runs for at most this long each time it's entered (and everything it calls), on top of the call's own limit. It's wall-clock time from entering, waits included (Ex. an `Http.fetch`), like the call's own limit. Once it's past it, only code outside the part can catch the error (or any other), its gates can't be called, and its sleep wakes at the limit. And it runs only while it's called: a process its code spawns and doesn't await ends (with no value) when the call into the part returns.
- `calls` (a list of functions outside the part, Ex. `[self.api.request]`): gates, the part's only way to anything else. Its code may call them, and they run unconfined, as themselves: an API you choose for it (Ex. one that signs a request for it, which it can't do itself). Hand them to it as values (a field in the part holding the function, set before sealing: `tool.api = new { sign: self.keys.sign } on tool`, made `on` the part so it's within its reach; or an argument), since a path to them finds nothing from inside. A gate runs with everything the document has, so it must check what it's given. A gate must be outside the part, and a call is a gate's only when every confinement it's called under lists it, so the part can't make one of its own. The functions themselves are kept, not their paths.

Sealing again only adds to a seal: it never unfreezes fields, makes an object removable that wasn't, or lifts a confinement. Confined code can only seal what it may change.
```rust
#[type] Engine: { fn price(units: int) -> int { units * 50 } }

#[init]
fn lock() { seal(<Engine>); }

fn later() {
    parse("fn price(units: int) -> int { 0 }", <Engine>); // error: sealed
}

fn add_tool(source: str) {
    const tool = new {} on self.tools;
    parse(source, tool, 'stof', 'untrusted');           // code from elsewhere: declarations only
    tool.api = new { sign: self.keys.sign_for_tool } on tool; // its gate, handed in (it calls self.api.sign(text))
    seal(tool, removable = true, reads = [self.inputs], hosts = ['api.weather.com'], libs = ['Std', 'Str', 'Http'],
        time = 2s, calls = [self.keys.sign_for_tool]);  // once, with everything: every confinement must list a gate
}
```


# Std::sealed(obj: obj) -> map | null
How an object is sealed, or null when it isn't: a map with `part` (the path of the object `seal` was called on, which may be this one or one around it), `fields` (its fields are frozen), `removable` (it can be dropped whole), and `confined` (a list of `{ reads, hosts, libs, time, calls }`, one per confinement its code runs under; empty when it isn't confined; `calls` as function paths).
```rust
fn check() {
    seal(self.tool, removable = true, reads = [self.inputs]);
    const info = sealed(self.tool);
    assert(info.get('removable'));
    assert_eq(info.get('confined').len(), 1);
    assert_null(sealed(self.inputs));
}
```


# Std::set(..) -> set
Construct a new set with the given arguments.
```rust
assert_eq(set(1, 2, 3), {1, 2, 3});
```


# Std::set_env(var: str, value: str) -> void
Set an environment variable by name with a value. Requires the "system" feature flag.
```rust
set_env("HOST", "localhost");
```


# Std::sha256(data: str | blob) -> str
The SHA-256 of text (its UTF-8 bytes) or a blob, as lowercase hex: the same as any other SHA-256 of those bytes, so another system can check it (Ex. a hash a protocol sends). For a part of the document (its logic and structure), use `digest`.
```rust
assert_eq(sha256('abc'), 'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad');
```


# Std::shallow_drop(..) -> bool | list
Operates the same way Std::drop(..) does, however, if dropping a field and the field points to an object or data, only remove the field and not the associated object/data. This is used instead of drop in instances where multiple fields might point to the same object and you'd like to remove the field without removing the object.
```rust
const object = self.field; // field is an obj value
assert(shallow_drop("self.field"));
assert_not(self.field); // note: this will still work if the objects name is "field"
assert(object.exists()); // object was kept around
```


# Std::sleep(time: ms) -> void
Instruct this process to sleep for an amount of time, while others continue executing. Use time units for specificity, but don't expect this to be very accurate (guaranteed it will sleep for at least this long, but maybe longer). Default unit is milliseconds.
```rust
sleep(1s); // sleep for 1 second
```


# Std::str(..) -> str
Prints all arguments to a string, just like it would be to an output stream.
```rust
assert_eq(str("hello, world"), "hello, world");
```


# Std::stringify(format: str = "json", context: obj = null) -> str
Use a loaded format to export a string from the given context (or entire graph/document). The default format is json, and the standard implementation only exports object fields. Export results will vary depending on the format, some support more than others (it is up to the format implementation to decide how it exports data). You can always create your own to use.
```rust
const object = new { x: 3.14km, y: 42.5m };
assert_eq(stringify("json", object), "{\"x\":3.14,\"y\":42.5}"); // lossy as json doesn't have a units concept
```

Some formats call functions for their values first (computed values), like `#[attr]` and `#[text]` functions in the html and xml formats. They're called in this process, so async functions are awaited before the export is written.
```rust
// in the document:
// #[tag('p')] note: { #[attr('class')] kind: 'hint'; #[text] fn message() -> str { 'Saved' } }
assert_eq(stringify('html', self.note), '<p class="hint">Saved</p>');
```


# Std::swap(first: unknown, second: unknown) -> void
Swap the memory addresses of any two values.
```rust
const a = 42;
const b = -55;
swap(&a, &b); // '&' because int is a value type (not automatically a reference)
assert_eq(a, -55);
assert_eq(b, 42);
```


# Std::throw(value: unknown = "Error") -> void
Throw an error with an optional value. Optionally catch this value within a try-catch block. Otherwise, this process will immediately hault executing with the given error.
```rust
throw("error message");
```


# Std::trace(..) -> void
Trace this location within your code execution. Will print out your arguments plus process debug information and the current instruction stack. If the last argument given is an integer value, that number of executed instruction stack instructions will be shown (very helpful for deeper debugging).
```rust
trace("Getting here"); // will print "Getting here", then output a trace of the current process info and last 10 executed instructions
trace(70); // last 70 executed instructions (most recent on bottom and numbered)
```


# Std::xml(text: str, tag: str) -> str
A helper function to create an XML-tagged string.
```rust
assert_eq(xml("hello, world", "msg"), "<msg>hello, world</msg>");
```


