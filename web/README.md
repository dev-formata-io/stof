# Stof: Data + Logic, Anywhere
### Standard Transformation and Organization Format

<p align="left">
    <a href="https://docs.stof.dev" style="margin: 3px"><img src="https://img.shields.io/badge/docs-docs.stof.dev-purple?logo=gitbook&logoColor=white"></a>
    <a href="https://github.com/dev-formata-io/stof" style="margin: 3px"><img src="https://img.shields.io/github/stars/dev-formata-io/stof"></a>
    <a href="https://github.com/dev-formata-io/stof/actions" style="margin: 3px"><img src="https://img.shields.io/github/actions/workflow/status/dev-formata-io/stof/rust.yml"></a>
    <a href="https://www.npmjs.com/package/@formata/stof"><img src="https://img.shields.io/npm/d18m/%40formata%2Fstof?label=npm%3A%40formata%2Fstof&color=orange"></a>
    <a href="https://crates.io/crates/stof" style="margin: 3px"><img src="https://img.shields.io/crates/d/stof?label=crate%20downloads&color=aqua"></a>
    <a href="https://crates.io/crates/stof" style="margin: 3px"><img src="https://img.shields.io/crates/l/stof?color=maroon"></a>
</p>

- [Site](https://stof.dev)
- [Docs](https://docs.stof.dev)
- [Playground](https://play.stof.dev)
- [GitHub](https://github.com/dev-formata-io/stof)
- [Discord](https://discord.gg/Up5kxdeXZt)

<br/>

## Overview

Data and logic have always been separate. That makes things hard. Stof puts them together.

A portable document format where validation, functions, and behavior live alongside the data they belong to — in one document, across any service, language, or runtime.

- **Superset of JSON** — valid JSON is always valid Stof. Works with YAML, TOML, and more out of the box.
- **Sandboxed execution** — logic runs in a secure, isolated runtime. Safe to execute untrusted code from external sources.
- **Built in Rust, runs everywhere** — native crate, WebAssembly for JS/TS (Node, Deno, Bun, browser), and Python bindings via PyPI.

> Used in production: [Limitr](https://limitr.dev)'s pricing policy engine — plans, credits, limits, validation logic — runs entirely on Stof.


## Data That Does Things

Stof starts where JSON ends. Add functions right next to the data they operate on.

```typescript
import { stofAsync } from '@formata/stof';

const doc = await stofAsync`
    name: "Alice"
    age: 30
    
    fn greet() -> str {
        'Hello, ' + self.name + '!'
    }
    
    fn can_rent_car() -> bool {
        self.age >= 25
    }
`;

console.log(await doc.call('greet'));        // Hello, Alice!
console.log(await doc.call('can_rent_car')); // true
```

No separate schema file. No external validator. The data knows its own rules.


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


## Format Interop

Combine JSON, YAML, TOML, and Stof in a single document. Parse one format, transform it, export as another.

```typescript
import { stofAsync } from '@formata/stof';

const doc = await stofAsync`{
    json: '{"plans":{"pro":{"label":"Pro","price":{"amount":20},"entitlements":{"ai_chat":{"description":"AI Chat Feature","limit":{"credit":"chat-token","value":100000,"resets":true,"reset_inc":1.0}}}}}}'
    yaml: ''
    
    fn transform() {
        const policy = new {};
        parse(self.json, policy, 'json');
        
        policy.plans.pro.price.amount = 50;
        const entitlements = policy.plans.pro.entitlements;
        entitlements.ai_chat.limit.value *= 2;
        
        self.yaml = stringify('yaml', policy);
        Std.pln(self.yaml);
    }
}`;
doc.lib('Std', 'pln', (...args: unknown[])=>console.log(...args));

await doc.call('transform');
```
```
plans:
  pro:
    label: Pro
    price:
      amount: 50
    entitlements:
      ai_chat:
        description: AI Chat Feature
        limit:
          credit: chat-token
          value: 200000
          resets: true
          reset_inc: 1.0
```


## Self-Expanding Contexts

This is the capability that changes everything.

Stof documents can parse new Stof into themselves at runtime — receiving code over the network and immediately executing it. The program grows while it runs, always sandboxed.

```typescript
import { stofAsync } from '@formata/stof';

const doc = await stofAsync`
api: {}

fn load_api(stof: str) {
    parse(stof, self.api);
}`;

// Imagine this arriving over HTTP, from another service, or from an agent
const api = `
name: 'Stof'
fn message() -> str { 'Hello, ' + self.name ?? 'World' + '!!' }

#[main]
fn main() {
    pln(self.message());
}`;

doc.lib('Std', 'pln', (...args: unknown[])=>console.log(...args));
await doc.call('load_api', api);
await doc.run(); // calls #[main] funcs

// Hello, Stof
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


## Embedded

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
doc.parse({ name: "Bob", age: 25 });

const json = doc.stringify('json');
const obj = doc.record();
```

**Supports**: Node.js, Browser, Deno, Bun, Edge runtimes


## License

Apache 2.0. See LICENSE for details.


## Feedback & Community

- Open issues or discussions on [GitHub](https://github.com/dev-formata-io/stof)
- Chat with us on [Discord](https://discord.gg/Up5kxdeXZt)
- Star the project to support future development!

> Reach out to info@stof.dev to contact us directly
