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

<h3 align="center">Software that behaves differently for every user.</h3>

<p align="center">
    Start with your agent. Each customer's rules, tools, and state go in one document: the agent edits it, you review the diff, and your app runs it. Stof is JSON with functions, run in a sandbox inside your app, in Rust, JavaScript, or Python.
</p>

## One customer, one document

This is how a support agent handles refunds for one customer, Acme. It's a Stof document: the data looks like JSON,
and the function next to it is a tool the agent can call.

```rust
// acme.stof
auto_approve_under: 50
monthly_cap: 500
refunded_this_month: 0

/// Refund an order, or escalate it to a person.
fn refund(order: str, amount: float) -> str {
    if (amount >= self.auto_approve_under) return "escalated";
    if (self.refunded_this_month + amount > self.monthly_cap) return "escalated";
    Payments::refund(order, amount);
    self.refunded_this_month += amount;
    "refunded"
}

#[test]
fn large_refunds_need_a_person() {
    assert_eq(self.refund("A-1", 120), "escalated");
}
```

Your app loads Acme's document and hands it the tools Acme's agent may use:

```typescript
import { StofDoc } from '@formata/stof';

const acme = await StofDoc.new();
acme.lib('Payments', 'refund', (order: string, amount: number) => payments.refund(order, amount));
acme.parse(source);                    // acme.stof

await acme.call('refund', 'A-1', 40);  // 'refunded', and refunded_this_month is 40
await acme.call('refund', 'A-2', 120); // 'escalated': over the auto-approve limit
```

Then Acme's account manager says Acme's refunds up to $150 can go through without a ticket. The agent proposes a
change to Acme's document, the test in the document runs, and you approve a one-line diff:

```diff
-auto_approve_under: 50
+auto_approve_under: 150
```

The same $120 refund now goes through, for Acme only, with no deploy. That's the idea: the agent changed one line in
this customer's document, and your app behaves differently for them.

**The document can only use what your app hands it.** `Payments::refund` works because you registered it for Acme.
A call to anything else (Ex. `Payments::export_customers()`) fails, and a document has no file system, network, or
environment unless you turn them on.

## Why a document

Teams building agents end up hand-building each customer's rules out of prompts, JSON, and glue code. The usual
options each break somewhere:

| Instead of a document | What breaks |
|---|---|
| Let the model write code and run it in a sandbox | The diff is hard to review, the sandbox is more infrastructure, and the code doesn't travel with the customer |
| JSON tool calls | Safe, but JSON can't hold logic ("this month", "under $50"), so a rules engine grows around it |
| Put the rules in the prompt | A rule in a prompt is a suggestion the model may ignore |
| Feature flags and settings | The settings are per customer, but the logic is shared and changes with a deploy |

A Stof document is all of the things you need at once: a model can write it, a person can read and diff it, it has
its own tests, and your app runs it deterministically, in-process, in a sandbox. The same document gives the same result in
Rust, Node.js, the browser, and Python.

## Why it works: everything is data

Stof has functions, but it isn't a language for writing apps. It's a document, and every part of it is data. Acme's
document is an object, and the function sits on it the same way the fields do:

```text
root (acme.stof)
├── auto_approve_under: 50          field
├── monthly_cap: 500                field
├── refunded_this_month: 0          field: state, updated as refunds go through
├── refund()                        function: data on the object, like a field
└── large_refunds_need_a_person()   test
```

Fields, functions, types, and docs are all pieces of data attached to objects (an entity-component system, if you
know game engines). That's why a document can hold a customer's behavior and not only their settings:

- **Anything you can do with data, you can do with logic.** Merge two documents, pull one object out with its
  functions, diff two versions, store it in a database, or send it over the wire.
- **A document can change while it runs.** It can parse new fields and functions into itself, or drop them, with
  no build step.
- **It grows to fit your domain.** Your app adds its own host functions to safely bridge contexts, types and schemas live in the
  document, and in Rust you can add your own formats and kinds of data.
- **There's very little to learn.** If you can read JSON and a bit of JavaScript, you can read Stof, and so can a
  model.

## Logic that moves like data

Documents combine like data. Logic and data from different places merge into one document, and every function
tagged `#[check]` runs together:

```typescript
const doc = await StofDoc.new(teamRules);         // rules every customer gets (Stof)
doc.parse(acmeRules);                             // + rules Acme's agent added (Stof)
doc.parse(order, 'json');                         // + data from your API (JSON)

await doc.run('check');                           // run all the #[check] rules together
doc.stringify('json');                            // and hand back plain JSON
```

```text
team rules (Stof) ─┐
Acme rules (Stof) ─┼─▶ one document ─▶ run('check') ─▶ JSON out
order data (JSON) ─┘
```

They split like data, too. Any part of a document can be sent on its own, functions included, as compact binary: `doc.blobify('bstf', part)`. Store it, version it, diff it, send it over HTTP or a WebSocket, and run it wherever it lands. The same document gives the same result in Rust, Node.js, the browser, and Python.

## Where it fits

Stof isn't an agent framework. Your agent, your model, and your app stay yours; Stof is the document each customer's
behavior lives in, and the runtime that runs it. It also works as a glue layer between the formats you already have
and your code:

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

- Each customer (or user, or tenant) needs its own rules, tools, limits, or state, and an agent or a person needs to
  change them safely.
- A model, a user, or a customer gives you logic you need to run: rules, filters, transforms, scoring, tool behavior.
- You're about to invent a config format, rules engine, or DSL for your product.
- The same logic has to run in more than one place (browser, server, worker) and give the same answer.
- You want to change behavior by sending or storing a new document, not by shipping a deploy.
- Logic should travel with the data it belongs to, through APIs, queues, and databases.

**Use something else when** the code is your own application logic, it's compute-heavy, or it's a long-running service. Write that in your main language and expose it to Stof documents as functions.

## Why Stof

- **Safe by default.** A document can only read and change itself and call the functions you give it. Each call has time and memory limits, and you can seal the parts you trust so the rest of the document can call them but not change them.
- **Fast and in-process.** Parsing and running a small document takes under a millisecond, and each call after that takes microseconds.
- **Small on the wire.** A document travels as text or as binary (`bstf`), functions and state included: Acme's document is about 600 bytes of text, or 2KB as binary.
- **It starts as JSON.** Any JSON document is valid Stof, and Stof imports and exports JSON, YAML, TOML, and more.
- **Models already write it.** It reads like JSON and JavaScript, and errors give the exact line and a plain explanation, so a model can fix its own mistakes.
- **More than a rules engine.** Types and schemas, units (`5min + 30s == 330s`, `2GB` to `MB` exactly), destructuring, async, and tests inside the document (`#[test]`, run with `stof test`).
- **Proven in production.** Stof is the engine behind [Limitr](https://limitr.dev), where each customer's pricing (plans, limits, credits, and the rules around them) is a Stof document, pushed to running apps and evaluated in-process.

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

**Teaching a model Stof:** give it [content/SKILL.md](https://github.com/dev-formata-io/stof/blob/main/content/SKILL.md), a short guide to the language with the library references in [content/stof.zip](https://github.com/dev-formata-io/stof/blob/main/content/stof.zip). The npm package ships the same skill, references included, in `node_modules/@formata/stof/skill/stof`.

## FAQ

**Is it safe to run a document an agent wrote or changed?**
A document runs in a sandbox: it can see itself and the functions your app provides, nothing else. File system, environment variables, file imports, and network access are all off by default, in every language; a host turns them on explicitly with `allow_system()` or `allow_http()` (or allows only some hosts with `allow_http_hosts`). Network access never reaches your machine or private network (localhost, internal services, the cloud metadata service) unless you list those hosts. Each call also has a time limit (2 minutes by default) and a memory limit (256 MB by default), both set by the host, so a runaway document fails its call instead of taking your app down. The `stof` command-line tool turns access on, since it runs your own files.

**Can an agent's function change the rest of the document?**
By default a document can change itself, including its own functions. Seal the parts you trust (`doc.seal('<Engine>')`) and they can still be called, but not replaced, removed, or added to, by the document or anyone else, until you unseal them. `doc.digest(path)` fingerprints a part's functions, and `doc.seal(path, { publicKey, signature })` seals it only if that fingerprint was signed by your key, so you know the logic you're running is the logic you shipped.

For code that comes from somewhere else (a plugin, a rule an agent wrote), parse it as untrusted (`doc.parse(src, 'stof', 'plugins.weather', 'untrusted')`: declarations only, nothing runs while it's parsed) and seal it confined: `doc.seal('plugins.weather', { reads: ['inputs'], hosts: ['api.weather.example'], libs: ['Std', 'Str', 'Http'], time: 2000 })`. While it runs, it sees only itself and what you list, changes only itself, reaches only those hosts, calls only those libraries, and stops after that long; the rest of the document stays out of its reach. Give it an API of your own with gates (`calls`): functions it may call that run as themselves, its only way to anything else. Seal it `removable` and you can drop it whole when it's no longer wanted.

**Can a document check where data came from?**
Yes: the `Key` library signs and verifies (`Key::sign(secret, 'orders/v1', order)`, `Key::verify(public, 'orders/v1', order, signature)`), and your app has the same functions (`StofDoc.sign`, `StofDoc.verify`), so a signature made in one place checks in any other. Each signature names what it's for (its context), so it can't be reused for anything else.

**Why not let the model write Python and run it in a container?**
You can, but it means infrastructure, startup time, and a lot of access to lock down, and the change a person reviews is a program, not a one-line diff. Stof runs inside your process, starts instantly, and has no access by default.

**Where do the documents live?**
Wherever you keep customer data: a database row per customer, object storage, or a repo if you want history and reviews. State the document updates (like `refunded_this_month`) is part of the document: save it with `doc.blobify('bstf')` and load it back with `StofDoc.new(bytes, 'bstf')`.

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
