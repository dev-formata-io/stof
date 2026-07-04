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

<p align="center">
    <em><b>Stof</b> is a <b>Data Runtime</b> built on top of Rust & WebAssembly. Designed to <b>combine data and logic</b> together in a portable & lightweight context that runs anywhere, <b>sandboxed</b>.</em>
</p>

<p align="center">
  <a href="https://stof.dev">
    <img src="https://raw.githubusercontent.com/dev-formata-io/stof/main/content/stof_gif.gif" alt="Stof: JSON to Stof" />
  </a>
</p>

## Install

```bash
npm i @formata/stof     # TypeScript / JavaScript
cargo add stof          # Rust
pip install stof        # Python
```

Full setup for every language, the complete standard library, and worked end-to-end examples: **[stof.dev](https://stof.dev)**

## Quickstart

Every JSON document you have is already a Stof document. Start with the data you have, add logic where you need it, and export back to JSON - or YAML, or TOML - any time.

```typescript
import { stofAsync } from '@formata/stof';

const doc = await stofAsync`
{
    "name": "Stof"
    "hello": ():str => \`Hello, \${self.name}!\`
}`;

console.log(await doc.call('hello')); // Hello, Stof!
```

```typescript
import { StofDoc } from '@formata/stof';

const doc = await StofDoc.parse({
    first: 'John',
    last: 'Doe',
    domain: 'example.com'
});

doc.parse(`
fn fill(domain: str = 'example.com') {
    self.domain = domain;
    self.email = \`\${self.first.lower()}.\${self.last.lower()}@\${self.domain}\`;
}`);

await doc.call('fill', 'stof.dev');
console.log(doc.record());

/*
{
    first: 'John',
    last: 'Doe',
    domain: 'stof.dev',
    email: 'john.doe@stof.dev',
}
*/
```

## What Stof Is

Stof is a data runtime — not a data format, not a programming language, not a database.

The same way a JavaScript runtime executes JavaScript, a data runtime executes data. A Stof document doesn't just describe something; it validates itself, transforms itself, and acts, wherever the runtime lands.

Concretely: valid JSON is already valid Stof. You're not migrating to a new format — you're handing your existing data a place to keep the logic that used to live somewhere else.

## Why it's different

- **Data and logic belong together.** Splitting them across a data file and a separate codebase creates schema drift, versioning hell, and is a limitation for modern distributed systems. Stof documents carry both, as one unit.
- **Portable means portable.** The same document runs native, in WebAssembly, or embedded in another host, with the same behavior every time. If it doesn't run everywhere, it isn't portable — it's just convenient sometimes.
- **Sandboxed by default.** A document can only see and manipulate itself unless you explicitly hand it more. Logic that arrives over the wire has to be safe to run without a review cycle first.
- **Documents should be able to grow.** A running document parsing new fields, new types, even new functions into itself isn't an edge case — it's the model working as intended. Data that can only be replaced, never extended, is still just a snapshot.
- **Interop with all data formats.** A superset of JSON that is compatible with every other data format, even ones that don't exist yet — binary, images, PDFs, DOCX, YAML, TOML, JSON, etc.

## Where This Came From

Stof comes out of a decade spent building parametric and geometry formats for CAD and graphics systems. That's where the core architectural insights come from: represent everything as a flat graph of nodes and data components (ECS + DAG), connected by pointers rather than nested copies, and moving or transforming part of the document stops being expensive. Everything — fields, functions, PDFs, images, binaries — become data components on a graph of nodes. Allow function components to manipulate the graph, and you have Stof.

Stof started as a solo project while trying to send wasm over the wire, and it's been running in production since — including as the policy engine underneath [Limitr](https://limitr.dev), where every plan, credit limit, and validation rule is a live Stof document.

## Learn More

- [stof.dev](https://stof.dev) — install docs, the full standard library, and examples
- [Discord](https://discord.gg/Up5kxdeXZt) — talk to the team and community
- [GitHub Issues](https://github.com/dev-formata-io/stof/issues) — bugs and feature requests

## License

Apache 2.0. See LICENSE for details.
