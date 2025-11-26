<p align="center"><img src="./content/stof.png" height="150"></p>

----

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
Stof is a unified data format that works seamlessly with other formats to bridge the gap between static data and programmable documents.

It is based on an "Everything as Data" model, in which fields, functions, PDFs, images, binaries, or any other type of data are neatly combined, while keeping single-document simplicity and portability.

This provides many benefits:
- Write data once, use it everywhere, in any format
- Sandboxed logic + execution in your data (as data)
- Send functions over APIs
- Doesn't need a large ecosystem to work
- Format-agnostic (JSON, YAML, TOML, PDF, binary, etc.)

### Example Use-Cases
- Smart configs with validation and logic
- Prompts as human readable & maintainable data + code
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
    MiB ram: 2TB + 50GiB - 5GB
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

## Embedded Stof
Stof is written in Rust, but is meant to be used wherever you work.

Python bindings are planned. Message on Discord and get involved (seeking contributors).

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

### JavaScript/TypeScript
Stof is compiled to WebAssembly for embedding in JS, and a [JSR](https://jsr.io/@formata/stof) package is provided.

```typescript
import { StofDoc } from '@formata/stof';
const doc = await StofDoc.new();

doc.lib('Std', 'pln', (... vars: unknown[]) => console.log(...vars));
doc.lib('Example', 'nested', async (): Promise<Map<string, string>> => {
    const res = new Map();
    res.set('msg', 'hello, there');
    res.set('nested', await (async (): Promise<string> => 'this is a nested async JS fn (like fetch)')());
    return res;
}, true);

doc.parse(`
    field: 42
    fn main() -> int {
        const res = await Example.nested();
        pln(res);
        self.field
    }
`);
const field = await doc.call('main');
console.log(field);

/*
Map(2) {                                                                                                                                                                                                                       
  "msg" => "hello, there",
  "nested" => "this is a nested async JS fn (like fetch)"
}
42
*/
```

## License
Apache 2.0. See LICENSE for details.

## Feedback & Community
- Open issues or discussions on [GitHub](https://github.com/dev-formata-io/stof)
- Chat with us on [Discord](https://discord.gg/Up5kxdeXZt)
- Star the project to support future development!

> Reach out to info@stof.dev to contact us directly
