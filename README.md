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

A portable document format where validation, functions, and behavior live alongside the data they belong to, in one document, across any service, language, or runtime.

> Works with JSON, YAML, TOML, etc. out of the box for seamless interchange. Add sandboxed logic as data where you need it for immediate eval/transforms.

> Built with Stof: [Limitr](https://limitr.dev) is an open source pricing and enforcement engine. The entire policy - plans, credits, limits, validation logic - lives in a single Stof document.

## Define Data and Logic Together - Combine/Split as Needed
`npm i @formata/stof`
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
```bash
bun run transform.ts
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

## Units & Types
Rich type system for accuracy and consistency across environments + conversions (time, memory e.g. GB or MiB, temperature, etc.).

```typescript
import { stofAsync } from '../doc.ts';

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

## Self-Expanding Contexts
Documents evolve over time. Stof can even parse itself for immediate use in the same call.

Small surface area, always sandboxed (wasm + host libs), ultra portable, and quick.

```typescript
import { stofAsync } from '@formata/stof';

const doc = await stofAsync`
api: {}

fn load_api(stof: str) {
    parse(stof, self.api); // parses itself
}`;

const api = `
name: 'Stof'
fn message() -> str { 'Hello, ' + self.name ?? 'World' + '!!' }

#[main]
fn main() {
    pln(self.message());
}`;

doc.lib('Std', 'pln', (...args: unknown[])=>console.log(...args));
await doc.call('load_api', api);
await doc.run(); // calls #[main] funcs - see docs

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
``` toml
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
```typescript
import { initStof, StofDoc } from '@formata/stof';

// Initialize wasm once at startup
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
```typescript
await initStof();
const doc = new StofDoc();

// Register JS functions
doc.lib('console', 'log', (...args: unknown[]) => console.log(...args));
doc.lib('fetch', 'get', async (url: string) => {
    const res = await fetch(url);
    return await res.json();
}, true); // true = async function - full support

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

## License
Apache 2.0. See LICENSE for details.

## Feedback & Community
- Open issues or discussions on [GitHub](https://github.com/dev-formata-io/stof)
- Chat with us on [Discord](https://discord.gg/Up5kxdeXZt)
- Star the project to support future development!

> Reach out to info@stof.dev to contact us directly
