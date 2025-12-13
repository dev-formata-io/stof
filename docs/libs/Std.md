# Standard Library (Std)
Functions in the 'Std' library are foundational to Stof and therefore do not requre one to explicitly reference 'Std' when calling them. Within the standard library, you'll find functions for asserting values, printing to the console, throwing errors, putting processes to sleep, etc. Note for advanced users that it is possible to extend or modify this library as needed.

## Example Usage
```rust
#[main]
fn main() {
    Std.pln("printing a line");
    pln("printing another line"); // no 'Std' needed for this library
}
```

# Std.assert(value: unknown = false) -> void
Throw an error if the given value is not truthy.
```rust
assert(true);
assert(false); // errors
```


# Std.assert_eq(first: unknown, second: unknown) -> void
Throw an error if the first value does not equal the second.
```rust
assert_eq('a', 'a');
assert_eq(43, 42); // errors
```


# Std.assert_neq(first: unknown, second: unknown) -> void
Throw an error if the first value equals the second.
```rust
assert_neq('a', 'b');
assert_neq(34, 34); // errors
```


# Std.assert_not(value: unknown = true) -> void
Throw an error if the given value is truthy.
```rust
assert_not(false);
assert_not(true); // errors
```


# Std.blobify(format: str = "json", context: obj = null) -> blob
Use a loaded format to export a binary blob from the given context (or entire graph/document). The default format is json, and the standard implementation only exports object fields. Export results will vary depending on the format, some support more than others (it is up to the format implementation to decide how it exports data). You can always create your own to use.
```rust
const object = new { x: 3.14km, y: 42m };
const export = blobify("json", object); // json string like "stringify", but as a utf8 blob
assert(export.len() > 0);
```


# Std.callstack() -> list
Return the current callstack as a list of function pointers (last function is 'this').
```rust
// inside a function call
for (const func in callstack()) {
    pln(func.obj().path(), ".", func.name());
}
```


# Std.copy(val: unknown) -> unknown
Deep copy the given value. If this value is an object (or contains one), recursively deep copy the object (all fields, funcs, & data).
```rust
const a = {1, 2, 3};
const b = copy(a);
b.clear();
assert_neq(a, b);
```


# Std.dbg(..) -> void
Prints all arguments as debug output to the standard output stream.
```rust
dbg("hello, world");
```


# Std.dbg_tracestack() -> void
Print a snapshot of the current stack.


# Std.drop(..) -> bool | list
Drop fields (by str path), functions (path or fn), objects (path or obj), and data from the graph. Objects will have their #[dropped] functions called when dropped. When dropping multiple values at once, this will return a list of booleans indicating a successful removal or not for each value.
```rust
const func = () => {};
const object = new {};
const results = drop("self.field", func, object);
assert_eq(results, [true, true, true]);
```


# Std.env(var: str) -> str
Get an environment variable by name. Requires the "system" feature flag.
```rust
const var = env("HOST");
```


# Std.env_vars() -> map
Get a map of the current environment variables (str, str). Requires the "system" feature flag.
```rust
const vars: map = env_vars();
```


# Std.err(..) -> void
Prints all arguments to the error output stream.
```rust
err("hello, world");
```


# Std.exit(..) -> void
Immediately terminates this (or another) Stof process. Pass a promise into this function to terminate it's processes execution.
```rust
const promise = async {
    sleep(10s);
};
exit(promise);
```


# Std.format(format: str) -> bool
Is the given format loaded/available to use?
```rust
assert(format("json"));
assert_not(format("step"));
```


# Std.format_content_type(format: str) -> str
Returns the available format's content type (HTTP header value), or null if the format is not available. All formats are required to give a content type, even if it doesn't apply to that format.
```rust
assert_eq(format_content_type("json"), "application/json");
```


# Std.formats() -> set
A set of all available formats, available to use with parse, stringify, and blobify.
```rust
const loaded = formats();
assert(loaded.contains("json"));
```


# Std.funcs(attributes: str | list | set = null) -> list
Get a list of all functions in this graph, optionally filtering by attributes (single string, list of strings, set of strings, or tuple of strings).
```rust
for (const func in funcs({"test", "main"})) {
    // all test and main functions in the graph
    // call them or whatever you need
}
```


# Std.graph_id() -> str
Return this graph's unique string ID.
```rust
assert(graph_id().len() > 10);
```


# Std.lib(lib: str) -> bool
Is the given library loaded/available to use?
```rust
assert(lib("Std")); // standard library is loaded
assert_not(lib("Render")); // no "Render" library loaded
```


# Std.libs() -> set
Set of all available libraries. This will most likely include standard libraries like Std, Fn, Set, List, etc.
```rust
assert(libs().superset({"Std", "Fn", "Num", "Set"}));
```


# Std.list(..) -> list
Construct a new list with the given arguments.
```rust
assert_eq(list(1, 2, 3), [1, 2, 3]);
```


# Std.log_debug(..) -> void
Logs all arguments as debug info using the "log" crate.
```rust
log_debug("this is what just happened, in case you need to debug me");
```


# Std.log_error(..) -> void
Logs all arguments as an error using the "log" crate.
```rust
log_error("we have a problem");
```


# Std.log_info(..) -> void
Logs all arguments as info using the "log" crate.
```rust
log_info("we just did something cool");
```


# Std.log_trace(..) -> void
Logs all arguments as a trace using the "log" crate.
```rust
log_trace("we have a problem");
```


# Std.log_warn(..) -> void
Logs all arguments as a warnging using the "log" crate.
```rust
log_warn("we encountered something, but are handling it");
```


# Std.map(..) -> map
Construct a new map with the given arguments (tuples of key & value). Helpful as a way to create an empty map.
```rust
assert_eq(map(("a", 1), ("b", 2)), {"a": 1, "b": 2});
```


# Std.max(..) -> unknown
Return the maximum value of all given arguments. If an argument is a collection, the max value within the collection will be considered only.
```rust
assert_eq(max(1km, 2m, 3mm), 1km);
```


# Std.min(..) -> unknown
Return the minimum value of all given arguments. If an argument is a collection, the min value within the collection will be considered only.
```rust
assert_eq(min(1km, 2m, 3mm), 3mm);
```


# Std.nanoid(length: int = 21) -> str
Generate a URL safe random string ID, using the nanoid algorithm with a specified length (default is 21 characters). Probability of a collision is very low, and inversely proportional to ID length.
```rust
assert_neq(nanoid(), nanoid(33));
```


# Std.parse(source: str | blob, context: str | obj = self, format: str = "stof", profile: str = "prod") -> bool
Parse data into this document/graph at the given location (default context is the calling object), using the given format (default is Stof). Formats are extensible and replaceable in Stof, so use whichever formats you have loaded (json, stof, images, pdfs, docx, etc.).
```rust
parse("fn hello() -> str { \"hello\" }");
assert_eq(self.hello(), "hello"); // can now call it
```


# Std.peek(..) -> void
Trace this location within your code execution. Will print out your arguments plus process debug information and the next instructions on the instruction stack. If the last argument given is an integer value, that number of (future) instructions will be shown (very helpful for deeper debugging).
```rust
peek("Getting here"); // will print "Getting here", then output a trace of the current process info and next 10 instructions to be executed
peek(70); // next 70 instructions
```


# Std.pln(..) -> void
Prints all arguments to the standard output stream.
```rust
pln("hello, world");
```


# Std.prof(name: str) -> bool
Is/was this graph parsed last with the given profile name?
```rust
// is the current profile named "test"?
const test = prof('test');
```


# Std.prompt(text: str = '', tag?: str) -> prompt
A helper function to create a prompt.
```rust
const prompt = prompt(tag = 'instruction');
prompt += prompt('do a thing', 'sub');
prompt += prompt('another thing', 'sub');
assert_eq(prompt as str, '<instruction><sub>do a thing</sub><sub>another thing</sub></instruction>');
```


# Std.remove_env(var: str) -> void
Remove an environment variable by name. Requires the "system" feature flag.
```rust
remove_env("HOST");
```


# Std.set(..) -> set
Construct a new set with the given arguments.
```rust
assert_eq(set(1, 2, 3), {1, 2, 3});
```


# Std.set_env(var: str, value: str) -> void
Set an environment variable by name with a value. Requires the "system" feature flag.
```rust
set_env("HOST", "localhost");
```


# Std.shallow_drop(..) -> bool | list
Operates the same way Std.drop(..) does, however, if dropping a field and the field points to an object or data, only remove the field and not the associated object/data. This is used instead of drop in instances where multiple fields might point to the same object and you'd like to remove the field without removing the object.
```rust
const object = self.field; // field is an obj value
assert(shallow_drop("self.field"));
assert_not(self.field); // note: this will still work if the objects name is "field"
assert(object.exists()); // object was kept around
```


# Std.sleep(time: ms) -> void
Instruct this process to sleep for an amount of time, while others continue executing. Use time units for specificity, but don't expect this to be very accurate (guaranteed it will sleep for at least this long, but maybe longer). Default unit is milliseconds.
```rust
sleep(1s); // sleep for 1 second
```


# Std.str(..) -> str
Prints all arguments to a string, just like it would be to an output stream.
```rust
assert_eq(str("hello, world"), "hello, world");
```


# Std.stringify(format: str = "json", context: obj = null) -> str
Use a loaded format to export a string from the given context (or entire graph/document). The default format is json, and the standard implementation only exports object fields. Export results will vary depending on the format, some support more than others (it is up to the format implementation to decide how it exports data). You can always create your own to use.
```rust
const object = new { x: 3.14km, y: 42m };
assert_eq(stringify("json", object), "{\"x\":3.14,\"y\":42}"); // lossy as json doesn't have a units concept
```


# Std.swap(first: unknown, second: unknown) -> void
Swap the memory addresses of any two values.
```rust
const a = 42;
const b = -55;
swap(&a, &b); // '&' because int is a value type (not automatically a reference)
assert_eq(a, -55);
assert_eq(b, 42);
```


# Std.throw(value: unknown = "Error") -> void
Throw an error with an optional value. Optionally catch this value within a try-catch block. Otherwise, this process will immediately hault executing with the given error.
```rust
throw("error message");
```


# Std.trace(..) -> void
Trace this location within your code execution. Will print out your arguments plus process debug information and the current instruction stack. If the last argument given is an integer value, that number of executed instruction stack instructions will be shown (very helpful for deeper debugging).
```rust
trace("Getting here"); // will print "Getting here", then output a trace of the current process info and last 10 executed instructions
trace(70); // last 70 executed instructions (most recent on bottom and numbered)
```


# Std.xml(text: str, tag: str) -> str
A helper function to create an XML-tagged string.
```rust
assert_eq(xml("hello, world", "msg"), "<msg>hello, world</msg>");
```


