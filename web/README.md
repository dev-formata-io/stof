# STOF: Data that carries its own logic

<p align="center">
    <a href="https://docs.stof.dev" style="margin: 3px"><img src="https://img.shields.io/badge/docs-docs.stof.dev-purple?logo=gitbook&logoColor=white"></a>
    <a href="https://github.com/dev-formata-io/stof" style="margin: 3px"><img src="https://img.shields.io/github/stars/dev-formata-io/stof"></a>
    <a href="https://github.com/dev-formata-io/stof/actions" style="margin: 3px"><img src="https://img.shields.io/github/actions/workflow/status/dev-formata-io/stof/rust.yml"></a>
    <a href="https://crates.io/crates/stof" style="margin: 3px"><img src="https://img.shields.io/crates/d/stof?label=crate%20downloads&color=aqua"></a>
    <a href="https://crates.io/crates/stof" style="margin: 3px"><img src="https://img.shields.io/crates/l/stof?color=maroon"></a>
</p>

### Standard Transformation and Organization Format

- [Docs](https://docs.stof.dev)
- [Playground](https://play.stof.dev)
- [GitHub](https://github.com/dev-formata-io/stof)
- [Discord](https://discord.gg/Up5kxdeXZt)
- [Install](https://docs.stof.dev/book/installation)

<br/>

![Alt](https://repobeats.axiom.co/api/embed/efbc3324d289ccfb6d7825c840491d10ea1d5260.svg "Repobeats analytics image")

## Overview
Send functions + data over APIs, write configs that validate themselves, build data pipelines where transformations travel with the data, store logic + data in a database, etc.

> Works with JSON, YAML, TOML, etc. - no migration needed.

> Add/import logic only where required.

Treats everything uniformly - fields, functions, PDFs, images, binaries, etc. - as data that can be combined in a single portable document.

### Benefits
- Write data + logic once, use it everywhere (JS, Rust, Python, anywhere your app lives)
- Format-agnostic I/O (works with JSON, YAML, TOML, PDF, binaries, etc.)
- Sandboxed logic + execution in your data (as data)
- Send functions over APIs
- Doesn't need a large ecosystem to work

### Example Use-Cases
- Smart configs with validation and logic
- Data interchange with sandboxed execution
- Prompts as human-readable & maintainable data + code
- AI/LLM workflows and model configs
- Data pipelines with built-in processing
- Integration glue between systems
- Self-describing datasets
- ... basically anywhere data meets logic

### Sample Stof
Check out the online [playground](https://play.stof.dev) for examples you can play with yourself.
```rust
#[attributes("optional exec control | metadata | meta-logic")]
// A field on the doc "root" node.
field: 42

// JSON-like data & function organization
stats: {
    // Optional field types & expressions
    prompt context: prompt("trees of strings", tag="optional-xml-tag",
        prompt("behaves like a tree for workflows & functions"),
        prompt("just cast to/from str anywhere strings are needed")
        // Std.prompt(..) can take N prompts as sub-prompts
    );
    
    // Units as types with conversions & casting
    cm height: 6ft + 2in
    MiB memory: 2MB + 50GiB - 5GB + 1TB
    ms ttl: 300s
}

#[main]
/// The CLI (and other envs) use the #[main] attribute for which fns to call on run.
fn do_something() {
    // Dot separated path navigation of the document (self is the current node/obj)
    let gone = self.self_destruction();
    assert(gone);

    // async functions, blocks, and expressions always available
    async {
        const now = Time.now();
        loop {
            sleep(20ms);
            if (Time.diff(now) > 2s) break;
        }
    }

    // partial I/O with any format
    pln(stringify("toml", self.stats));
}

/**
 * A function that removes itself from this document when executed.
 */
fn self_destruction() -> bool {
    pln(self.field); // Std.pln(..) print line function
    drop(this);      // "this" is always the last fn on the call stack
    true             // "return" keyword is optional (no ";")
}
```

## CLI
See [installation docs](https://docs.stof.dev/book/installation) for CLI instructions and more information.

```rust
#[main]
fn say_hi() {
    pln("Hello, world!");
}
```
```
> stof run example.stof
Hello, world!
```

## Embedded Stof
Stof is written in Rust, and is meant to be used wherever you work. Join the project [Discord](https://discord.gg/Up5kxdeXZt) to get involved.

### Rust
``` toml
[dependencies]
stof = "0.8.*"
```
```rust
use stof::model::Graph;

fn main() {
    let mut graph = Graph::default();
    
    graph.parse_stof_src(r#"
        #[main]
        fn main() {
            pln("Hello, world!");
        }
    "#, None).unwrap();

    match graph.run(None, true) {
        Ok(res) => println!("{res}"),
        Err(err) => panic!("{err}"),
    }
}
```

### Python
Stof is available on [PyPi](https://pypi.org/project/stof). Just `pip install stof` and import the `pystof` module to get started.

> A few examples are located in the *src/py/examples* folder.

```python
from pystof import Doc

STOF = """
#[main]
fn main() {
    const name = Example.name('Stof,', 'with Python');
    pln(`Hello, ${name}!!`)
}
"""

def name(first, last):
    return first + ' ' + last

def main():
    doc = Doc()
    doc.lib('Example', 'name', name)
    doc.parse(STOF)
    doc.run()

if __name__ == "__main__":
    main()

# Output:
# Hello, Stof, with Python!!
```

### JavaScript/TypeScript

#### Installation
```bash
npm install @formata/stof
```

#### Initialization
Stof uses WebAssembly, so make sure to initialize it once.

```typescript
// Node.js, Deno, & Bun - Auto-detects and loads WASM
import { initStof } from '@formata/stof';
await initStof();

// Browser with bundler - Pass WASM explicitly
import { initStof } from '@formata/stof';
import stofWasm from '@formata/stof/wasm';
await initStof(await stofWasm());
```

#### Quick Start
```typescript
import { initStof, StofDoc } from '@formata/stof';

// Initialize once at startup
await initStof();

// Create and parse documents
const doc = new StofDoc();
doc.parse(`
    name: "Alice"
    age: 30
    fn greet() -> string {
        "Hello, " + self.name
    }
`);

// Call functions and access values
const greeting = await doc.call('greet');
console.log(greeting); // "Hello, Alice"
console.log(doc.get('age')); // 30
```

#### JavaScript Interop

Call JavaScript functions from Stof:
```typescript
await initStof();
const doc = new StofDoc();

// Register JS functions
doc.lib('console', 'log', (...args: unknown[]) => console.log(...args));
doc.lib('fetch', 'get', async (url: string) => {
    const res = await fetch(url);
    return await res.json();
}, true); // true = async function

doc.parse(`
    fn main() {
        const data = await fetch.get("https://api.example.com/data");
        console.log(data);
    }
`);

await doc.call('main');
```

#### Parse & Export
```typescript
// Parse from JSON
doc.parse({ name: "Bob", age: 25 });

// Export to different formats
const json = doc.stringify('json');
const obj = doc.record(); // JavaScript object
```

**Supports**: Node.js, Browser, Deno, Bun, Edge runtimes

## Research & Exploration
Stof explores several research areas:

- Practical code mobility at scale with modern type systems
- Security models for distributed computation-as-data
- Performance characteristics of serializable computation vs traditional RPC
- Formal semantics for "code as data" in distributed systems
- Edge computing, data pipelines, and collaborative systems

## License
Apache 2.0. See LICENSE for details.

## Feedback & Community
- Open issues or discussions on [GitHub](https://github.com/dev-formata-io/stof)
- Chat with us on [Discord](https://discord.gg/Up5kxdeXZt)
- Star the project to support future development!

> Reach out to info@stof.dev to contact us directly
