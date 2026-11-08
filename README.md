<h1 align="center">
    <a href="https://stof.dev">
        <picture>
            <source height="125" media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/dev-formata-io/stof/main/content/stof.png">
            <source media="(prefers-color-scheme: light)" srcset="https://raw.githubusercontent.com/dev-formata-io/stof/main/content/image_dark.png">
            <img height="125" alt="Stof" src="https://raw.githubusercontent.com/dev-formata-io/stof/main/content/image_dark.png">
        </picture>
    </a>
    <br>
    <a href="https://stof.dev"><img src="https://img.shields.io/badge/docs-stof.dev-purple?logo=gitbook&logoColor=white"></a>
    <a href="https://github.com/dev-formata-io/stof"><img src="https://img.shields.io/github/stars/dev-formata-io/stof"></a>
    <a href="https://github.com/dev-formata-io/stof/actions"><img src="https://img.shields.io/github/actions/workflow/status/dev-formata-io/stof/rust.yml"></a>
    <a href="https://www.npmjs.com/package/@formata/stof"><img src="https://img.shields.io/npm/d18m/%40formata%2Fstof?label=npm%3A%40formata%2Fstof&color=orange"></a>
    <a href="https://crates.io/crates/stof"><img src="https://img.shields.io/crates/d/stof?label=crate%20downloads&color=aqua"></a>
    <a href="https://crates.io/crates/stof"><img src="https://img.shields.io/crates/l/stof?color=maroon"></a>
</h1>

<h3 align="center">JSON that runs.</h3>

<p align="center">
    Stof is JSON with functions. Put the logic next to the data, and your app runs the document in a sandbox that can only touch what you hand it. In Rust, JavaScript, and Python.
</p>

<p align="center">
    <a href="https://stof.dev/playground">Try it in your browser</a> · <a href="https://stof.dev">Docs</a> · <a href="https://discord.gg/Up5kxdeXZt">Discord</a>
</p>

## JSON, plus a function

```rust
// doc.stof
{
    "values": { "a": 42, "b": 24 },

    fn sum() -> int {
        self.values.a + self.values.b
    }
}
```

```typescript
import { StofDoc } from '@formata/stof';

const doc = await StofDoc.new(source);   // doc.stof
await doc.call('sum');                    // 66
```

Any JSON is already a Stof document. Add a function where the logic belongs, and your app calls it the way it reads a
field. The document runs inside your process: no server, no container, and it can't reach anything you didn't give it.

## What else a document can hold

```rust
// server.stof
ms timeout: 5min + 30s        // 330000
MiB memory: 2GB               // about 1907.35

#[type]
Server: {
    #[schema((v: int): bool => v > 0 && v < 65536)]
    int port: 8080
}

config: { port: 70000 }

fn valid() -> bool { <Server>.schemafy(self.config) }

#[test]
fn config_is_valid() { assert(self.valid()); }
```

- **Units** are part of the number, and math between them converts. No more `"timeout": "5m"` for someone to parse later.
- **Types** carry rules for their fields. Point one at any object to check it: this config fails, because 70000 isn't a port.
- **Tests** live next to what they check. `stof test server.stof` fails here until the port is fixed.

There's more: destructuring, async, attributes, and imports and exports for JSON, YAML, TOML, and compact binary.

## It can only touch what you hand it

```typescript
const doc = await StofDoc.new(`
    fn greet(name: str) -> str { Host::greet(name) }
    fn leak() -> str { fs::read('/etc/passwd') }
`);

doc.lib('Host', 'greet', (name) => `hello, ${name}`);

await doc.call('greet', 'world');   // 'hello, world'
await doc.call('leak');
// throws: 'fs::read' isn't available: this host hasn't given documents file system access
```

A document sees itself and the functions your app registers. There's no file system, network, or environment unless
you turn them on, and network access can be limited to the hosts you list (`doc.allowHttp(['api.example.com'])`).
Every call has a time and memory limit. You can also seal the parts you trust, so nothing can change them, and confine
code from somewhere else so it sees only what you list. See [Security and the Sandbox](https://stof.dev/start/security).

## What it's for

- **Logic someone else writes.** A model, a user, or a customer gives you rules, filters, transforms, scoring, or
  agent tools. Check them, test them, and run them in-process on every request.
- **Behavior per customer.** One document per customer holds their rules, limits, and state. An agent can edit it, a
  person reviews a one-line diff, and your app behaves differently for that customer with no deploy.
- **Config that checks itself.** Validate and compute values in the config, and still export plain JSON, YAML, or TOML.
- **The rules engine or DSL you were about to build.** Describe your domain in Stof instead.
- **Logic that has to run everywhere.** Send a document, functions included, as text or compact binary, and get the
  same answer in the browser, on a server, or in a worker.

Each one has a runnable example in the [playground](https://stof.dev/playground). Use something else for your own
application code, heavy compute, or long-running services: write those in your main language and give them to
documents as functions.

Stof is the engine behind [Limitr](https://limitr.dev), where every customer's pricing is a Stof document, evaluated
in-process on every call.

## Install

```bash
npm i @formata/stof     # JavaScript / TypeScript
cargo add stof          # Rust
pip install stof        # Python
cargo install stof-cli  # command line: stof run, stof test
```

**TypeScript**

```typescript
import { stofAsync } from '@formata/stof';

const doc = await stofAsync`
    "name": "Stof"
    fn hello() -> str { \`Hello, \${self.name}!\` }
`;
console.log(await doc.call('hello')); // Hello, Stof!
```

**Python**

```python
from pystof import Doc

doc = Doc()
doc.parse('rate: 2\nfn total(units: int) -> float { units * self.rate }')
print(doc.call('total', 21))  # 42.0
```

**Rust**

```rust
use stof::model::Graph;
use stof::runtime::{Runtime, Val};

let mut graph = Graph::default();
graph.parse_stof_src("rate: 2\nfn total(units: int) -> float { units * self.rate }", None, Default::default())?;
let total = Runtime::call(&mut graph, "total", vec![Val::from(21i64)])?; // 42
```

**Teaching a model Stof:** give it [content/SKILL.md](https://github.com/dev-formata-io/stof/blob/main/content/SKILL.md),
a short guide to the language, with the library references in
[content/stof.zip](https://github.com/dev-formata-io/stof/blob/main/content/stof.zip). The npm package ships the same
skill in `node_modules/@formata/stof/skill/stof`.

## FAQ

**Is it safe to run a document a model wrote?**
That's what it's built for. A document can only see itself and the functions your app gives it. File system,
environment, file imports, and network are off by default in every language, and network access never reaches
localhost or your private network unless you list those hosts. Each call has a time limit (2 minutes by default) and a
memory limit (256 MB by default), so a runaway document fails its call instead of your app. For code from somewhere
else, parse it as untrusted (nothing runs while it's parsed) and seal it confined to what you list. The `stof` CLI
turns access on, since it runs your own files.

**Can models write Stof?**
Yes. It reads like JSON and JavaScript, which every model already knows, and errors give the exact line and a plain
explanation, so a model can fix its own mistakes. The [skill](https://github.com/dev-formata-io/stof/blob/main/content/SKILL.md)
helps.

**Why not have the model write Python and run it in a container?**
You can, but that's infrastructure, startup time, and a lot of access to lock down, and what a person reviews is a
program. Stof runs inside your process, starts instantly, has no access by default, and a change is a diff to a document.

**Why not WebAssembly?**
WebAssembly needs a compiler toolchain, and its binaries are large to send. Stof is readable, small, and travels with
the data it works on. (The Stof runtime itself is compiled to WebAssembly for browsers.)

**How fast is it?**
A small document parses and runs in under a millisecond, and calls after that take microseconds.

**Do I have to replace JSON, YAML, or TOML?**
No. Stof imports and exports them, so you can add logic to the data you already have.

## Learn more

- [stof.dev](https://stof.dev): docs and the library reference
- [Playground](https://stof.dev/playground): try Stof in the browser
- [Limitr](https://limitr.dev): Stof in production
- [Discord](https://discord.gg/Up5kxdeXZt): questions and discussion
- [Changelog](https://github.com/dev-formata-io/stof/blob/main/CHANGELOG.md) and [issues](https://github.com/dev-formata-io/stof/issues)

## License

Apache 2.0. See [LICENSE](https://github.com/dev-formata-io/stof/blob/main/LICENSE).
