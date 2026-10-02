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

<h3 align="center">Run AI-generated logic safely, inside your app.</h3>

<p align="center">
    Stof is a sandboxed runtime for logic that lives in data. Let a model write a rule, a transform, or a tool, then run it in-process in under a millisecond, with access to nothing but the data and functions you give it. Stof documents combine and split like JSON, and run like code, in Rust, JavaScript, and Python.
</p>

## Your agent can write the logic. Where do you run it?

Models are good at writing small pieces of logic: a routing rule, a validation check, a data transform, a tool. Running that logic is the hard part, because the code came from a model, not from your team.

| Option | The catch |
|---|---|
| Run generated Python or JS in a container or microVM | Heavy, slow to start, and more infrastructure to run and secure |
| `eval` it in your app | The code can reach everything your app can |
| Function calling / JSON tool calls | The model can only call functions you already wrote; it can't add new logic |
| Rule DSLs (logic encoded as JSON) | Limited, and awkward to read and write, for people and models alike |
| WebAssembly | Needs a compiler toolchain, binaries are large to send, and the logic lives apart from the data |

Stof is the missing option: a small language made to be written by models, sent as data, and run in-process in a sandbox.

## How it works

A model writes this. Stof is JSON plus functions, and the tag `#[triage]` is just a label the model and your app agree to:

```stof
#[triage]
fn route() {
    if (self.ticket.text.lower().contains('refund')) self.ticket.team = 'billing';
    else if (self.ticket.priority > 2) self.ticket.team = 'urgent';
    else self.ticket.team = 'general';
}

#[triage]
fn flag() {
    self.ticket.needs_human = self.ticket.priority > 2;
}
```

Your app runs everything tagged `#[triage]`, without requiring what the model named its functions or how many it wrote:

```typescript
import { StofDoc } from '@formata/stof';

const doc = await StofDoc.new(modelOutput);
doc.parse({ ticket: { text: 'Where is my refund?', priority: 1 } });

await doc.run('triage');
doc.get('ticket.team');   // 'billing'
```

And the document can't reach anything you didn't give it:

```typescript
const doc = await StofDoc.new(`fn run() -> str { fs::read_string('/etc/passwd') }`);
await doc.call('run');   // throws: this host hasn't given documents file system access
```

To let it do more, you hand it specific functions: `doc.lib('App', 'notify', notify)` makes `App::notify(...)` available, and nothing else. File system, environment, and network access stay off unless you turn them on (`doc.allowHttp()` here, `allow_system()` and `allow_http()` in Rust and Python).

## Logic that moves like data

Documents combine like data. Logic and data from different places merge into one document, and everything tagged `#[triage]` runs together:

```typescript
const doc = await StofDoc.new(teamPolicy);        // your team's rules (Stof)
doc.parse(modelOutput);                           // + rules a model wrote (Stof)
doc.parse(apiResponse, 'json');                   // + data from an API (JSON)

await doc.run('triage');                          // run all the rules together
doc.stringify('json');                            // and hand back plain JSON
```

```text
team policy (Stof) ─┐
model rules (Stof) ─┼─▶ one document ─▶ run('triage') ─▶ JSON out
API data    (JSON) ─┘
```

They split like data, too. Any part of a document can be sent on its own, functions included, as compact binary: `doc.blobify('bstf', part)`. Store it, version it, diff it, send it over HTTP or a WebSocket, and run it wherever it lands. The same document gives the same result in Rust, Node.js, the browser, and Python.

## Where it fits

Stof is a glue layer between the formats you already have and your code:

```text
  where things come from          the glue (often temporary)       your app
  ──────────────────────          ──────────────────────────       ────────
  model output     Stof  ─┐
  API responses    JSON  ─┤
  config files     YAML  ─┼─▶   Stof document: data + logic   ─▶   run it, read the results
  settings         TOML  ─┤                                   ─▶   export JSON / YAML / TOML
  user input       Stof  ─┘
```

Load data from anywhere, add logic, run it, and keep the document or throw it away. The runtime is a library (npm, crate, or pip) inside your app: no server, container, or extra process, and documents can only call the functions you expose.

**Reach for Stof when:**

- A model, a user, or a customer gives you logic you need to run: rules, filters, transforms, scoring, tool behavior.
- The same logic has to run in more than one place (browser, server, worker) and give the same answer.
- You want to change behavior by sending or storing a new document, not by shipping a deploy.
- Logic should travel with the data it belongs to, through APIs, queues, and databases.

**Use something else when** the code is your own application logic, it's compute-heavy, or it's a long-running service. Write that in your main language and expose it to Stof documents as functions.

## Why Stof

- **Safe by default.** A document can only read and change itself and call the functions you give it.
- **Fast and in-process.** No containers or cold starts: parsing and running a small document takes under a millisecond, and each call after that takes microseconds.
- **Small on the wire.** Documents, functions included, serialize to compact binary: the rules above are under 1KB.
- **It starts as JSON.** Any JSON document is valid Stof, and Stof imports and exports JSON, YAML, TOML, and more.
- **Models already write it.** Stof reads like JSON, so models write it without a lot of special training, and errors give the exact line and a plain explanation, so a model can fix its own mistakes.
- **Proven in production.** Stof is the policy engine behind [Limitr](https://limitr.dev), where documents are pushed to running apps and evaluated in-process.

## A few things you'll like

- **Units built in:** `5min + 30s` == `330s`, and `2GB` converts to `2000MB` exactly, when needed for comparisons, etc.
- **Types and schemas:** objects can have types with inheritance, and schemas that validate other objects.
- **Documents that grow:** a running document can parse new fields and functions into itself.
- **Tests inside the document:** mark functions with `#[test]` and run them with `stof test`.
- **Clear errors:** parse and runtime errors point to the file, line, and column, so a model can fix its own mistakes.

## Quickstart

Try it in your browser at the [Stof Playground](https://stof.dev/playground), or install it:

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

**Teaching a model Stof:** give it [content/SKILL.md](https://github.com/dev-formata-io/stof/blob/main/content/SKILL.md), a short guide to the language with the library references in [content/stof.zip](https://github.com/dev-formata-io/stof/blob/main/content/stof.zip).

## FAQ

**Is it really safe to run Stof a model wrote?**
A document runs in a sandbox: it can see itself and the functions your app provides, nothing else. File system, environment variables, file imports, and network access are all off by default, in every language; a host turns them on explicitly with `allow_system()` or `allow_http()`. The `stof` command-line tool turns them on, since it runs your own files.

**Why not just run the model's Python in a container?**
You can, but it means infrastructure, startup time, and a lot of access to lock down. Stof runs inside your process, starts instantly, and has no access by default.

**Can models actually write Stof?**
Yes. It reads like JSON and JavaScript, which every model already knows. For better results, give your agent the [Stof skill](https://github.com/dev-formata-io/stof/blob/main/content/SKILL.md): a short guide to the language and its common mistakes. When something's wrong, errors give the line and a plain explanation the model can act on.

**Why not WebAssembly?**
WebAssembly needs a compiler toolchain, and its binaries are large to send. Stof is readable, small, and travels with the data it works on. (The Stof runtime itself is compiled to WebAssembly for browsers.)

**Do I have to replace JSON, YAML, or TOML?**
No. Stof imports and exports them, so you can add logic to the data you already have.

## Learn more

- [stof.dev](https://stof.dev): docs, the standard library, and examples
- [Playground](https://stof.dev/playground): try Stof in the browser
- [Limitr](https://limitr.dev): Stof in production
- [Discord](https://discord.gg/Up5kxdeXZt): questions and discussion
- [Changelog](https://github.com/dev-formata-io/stof/blob/main/CHANGELOG.md) and [issues](https://github.com/dev-formata-io/stof/issues)

## License

Apache 2.0. See [LICENSE](https://github.com/dev-formata-io/stof/blob/main/LICENSE).
