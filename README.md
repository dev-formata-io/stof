# Stof
### One document, any runtime. Send functions over the wire. Documents that validate themselves.

<p align="left">
    <a href="https://docs.stof.dev" style="margin: 3px"><img src="https://img.shields.io/badge/docs-docs.stof.dev-purple?logo=gitbook&logoColor=white"></a>
    <a href="https://github.com/dev-formata-io/stof" style="margin: 3px"><img src="https://img.shields.io/github/stars/dev-formata-io/stof"></a>
    <a href="https://github.com/dev-formata-io/stof/actions" style="margin: 3px"><img src="https://img.shields.io/github/actions/workflow/status/dev-formata-io/stof/rust.yml"></a>
    <a href="https://www.npmjs.com/package/@formata/stof"><img src="https://img.shields.io/npm/d18m/%40formata%2Fstof?label=npm%3A%40formata%2Fstof&color=orange"></a>
    <a href="https://crates.io/crates/stof" style="margin: 3px"><img src="https://img.shields.io/crates/d/stof?label=crate%20downloads&color=aqua"></a>
    <a href="https://crates.io/crates/stof" style="margin: 3px"><img src="https://img.shields.io/crates/l/stof?color=maroon"></a>
</p>

[Playground](https://play.stof.dev) · [Docs](https://docs.stof.dev) · [Discord](https://discord.gg/Up5kxdeXZt) · [Site](https://stof.dev)

---

Stof is a portable document format where functions, validation, and behavior live alongside the data they belong to. It's a superset of JSON - your existing data works as-is. Add logic only where you need it.

```typescript
server: {
    str host: "0.0.0.0"
    int port: 8080
    MiB memory: 2GB

    fn url() -> str { `https://${self.host}:${self.port}` }
    fn valid() -> bool { self.memory > 200MB }
}
```

Built in Rust. Runs natively, in WASM (Node, Deno, Bun, browser), and in Python. Sandboxed execution - safe to run untrusted Stof from external sources.

## Why Stof?

Interoperability is the reality of modern software. JSON alone no longer cuts it - too much ambiguity, too much brittleness, and every API has its own flavor of interchange format or DSL. There is no single correct way in distributed systems.

Stof doesn't try to replace them. It's the layer that works with all of them - parse JSON, YAML, TOML, STOF, binary, etc. into one document, add the logic that belongs with the data (functions), and send it anywhere. Export to any format as needed internally.

Here's what that looks like in practice:

```typescript
import { stofAsync } from '@formata/stof';

const doc = await stofAsync`
#[type]
Server: {
    port: 8080
    host: 'localhost'
    secure: false
    MiB memory: 500GiB

    fn url() -> str {
        let url = self.secure ? 'https://' : 'http://';
        url += self.host + ':' + self.port;
        url
    }
}`;

// Parse STOF, JSON, YAML, binary, etc. into the same document
doc.parse(`Server "prod": {
    "host": "prod.example.com",
    "port": 443,
    "secure": true,
    "memory": "2GB"
}`);

console.log(await doc.call('prod.url'));     // https://prod.example.com:443
console.log(doc.get('prod.memory'));         // ~1907 MiB (auto-converted from GB)
```

## Runtime Self-Assembly

Stof documents can parse new Stof into themselves at runtime, receiving code over the network and executing it immediately. The document grows while it runs, always sandboxed.

```typescript
import { stofAsync } from '@formata/stof';

const doc = await stofAsync`
    fn loaded() -> str {
        const stof = await Ext.fetch();
        parse(stof, self);
        self.say_hello()
    }
`;

doc.lib('Ext', 'fetch', async () => {
    return `fn say_hello() -> str { 'Hello, world!' }`;
});

console.log(await doc.call('loaded'));      // Hello, world!
```

This is how [Limitr](https://limitr.dev) dynamically assembles API capabilities across services, a document receives new code, parses it in, and executes it with full type safety.

## Units & Types

Rich type system with automatic unit conversions — time, distance, memory, temperature, and more.

```typescript
import { stofAsync } from '@formata/stof';

const doc = await stofAsync`
#[type]
Point: {
    meters x: 0
    meters y: 0
    
    fn dist(other: Point) -> m {
        Num.sqrt((other.x - self.x).pow(2) + (other.y - self.y).pow(2))
    }
}

Point reference: {
    x: 1ft
    y: 2ft
}

fn distance(imported_json: str) -> inches {
    const imported = new {};
    parse(imported_json, imported, 'json');
    (self.reference.dist(imported) as inches).round(2)
}
`;

const dist = await doc.call('distance', '{ "x": 3, "y": 4 }');
console.log(dist); // 170.52
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

## Get Started

Stof is written in Rust, but use it where you work. Join the project [Discord](https://discord.gg/Up5kxdeXZt) to contribute.

### Rust

```toml
[dependencies]
stof = "0.9.*"
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

`pip install stof`

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

`npm i @formata/stof`

#### Initialization

Stof uses WebAssembly, so make sure to initialize it once.

```typescript
// Node.js, Deno, & Bun - Auto-detects and loads WASM
import { initStof } from '@formata/stof';
await initStof();

// Vite
import { initStof } from '@formata/stof';
import stofWasm from '@formata/stof/wasm?url';
await initStof(stofWasm);

// Browser with bundler - Pass WASM explicitly (e.g. @rollup/plugin-wasm)
import { initStof } from '@formata/stof';
import stofWasm from '@formata/stof/wasm';
await initStof(await stofWasm());
```

#### Usage

```typescript
import { initStof, StofDoc } from '@formata/stof';

await initStof();

const doc = new StofDoc();
doc.parse(`
    name: "Alice"
    age: 30
    fn greet() -> str {
        'Hello, ' + self.name
    }
`);

const greeting = await doc.call('greet');
console.log(greeting); // "Hello, Alice"
console.log(doc.get('age')); // 30
```

#### JavaScript Interop

```typescript
await initStof();
const doc = new StofDoc();

doc.lib('console', 'log', (...args: unknown[]) => console.log(...args));
doc.lib('fetch', 'get', async (url: string) => {
    const res = await fetch(url);
    return await res.json();
});

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
doc.parse({ name: "Bob", age: 25 });

const json = doc.stringify('json');
const obj = doc.record();
```

**Supports**: Node.js, Browser, Deno, Bun, Edge runtimes


## Learn More

- [Docs](https://docs.stof.dev) - core concepts, standard library, and format reference
- [GitHub Issues](https://github.com/dev-formata-io/stof) - bugs and feature requests
- [Discord](https://discord.gg/Up5kxdeXZt) - talk to the team and community
- [Playground](https://play.stof.dev) - try Stof in your browser, no install required

VS Code extension available - search "Stof" in your editor's extension marketplace.

## License

Apache 2.0. See LICENSE for details.
