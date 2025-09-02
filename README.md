<p align="center"><img src="./content/stof.png" height="150"></p>
<p align="center">
    <a href="https://docs.stof.dev"><img src="https://img.shields.io/badge/docs-docs.stof.dev-purple?logo=gitbook&logoColor=white"></a><span style="margin: 0px 4px;"></span>
    <a href="https://github.com/dev-formata-io/stof"><img src="https://img.shields.io/github/stars/dev-formata-io/stof"></a>
</p>
<p align="center">
    <a href="https://github.com/dev-formata-io/stof/actions"><img src="https://img.shields.io/github/actions/workflow/status/dev-formata-io/stof/rust.yml"></a>
    <a href="https://crates.io/crates/stof"><img src="https://img.shields.io/crates/d/stof?label=Stof%20downloads&color=aqua"></a>
    <a href="https://crates.io/crates/stof-cli"><img src="https://img.shields.io/crates/d/stof-cli?label=CLI%20downloads&color=darkgray"></a>
    <a href="https://crates.io/crates/stof"><img src="https://img.shields.io/crates/l/stof?color=maroon"></a>
    <a href="https://github.com/dev-formata-io/stof/commits"><img src="https://img.shields.io/github/commit-activity/m/dev-formata-io/stof"></a>
</p>

----

<br/>

**A smart, declarative runtime for data workflows**
- [Docs](https://docs.stof.dev)
- [GitHub](https://github.com/dev-formata-io/stof)
- [Discord](https://discord.gg/Up5kxdeXZt)

## What is Stof?
Stof works **with** other data formats to bridge the gap between static data and programmable documents. It is a lightweight, embeddable, and portable data logic format & platform for AI, infra-as-code, and config-heavy workflows. It's built to support:

- Data-Mesh, Integration, & Orchestration **glue-layer**
- Universal LLM & AI workflows, tools, & **intersystem data**
- Smart configs with logic, types, units, schemas, & **self-validation**
- Asynchronous **validation & transformation**

> Think of it as a foundation for building robust and declarative data flows, config systems, or backend models.

## Quick Example
``` rust
const list users: [              // optional type and const specification for fields
    {
        name: "Joe Schmo",       // commas or semi-colons accepted, but optional
        cm height: 6ft + 1in     // Stof adds units and declarations are expressions
        age: +32;                // trailing commas or semi-colons are okay
    },                           // trailing commas in arrays are okay
]

{
    "json": "normal json data supported when you need it",
    str "enhanced": "but also with types, units, logic, functions, etc."

    child: {
        MiB space: 2GiB  // unit types perform conversions when cast (time, temp, mass, etc.)
        fn apis() -> str { "sections of your document now become APIs" }
    }
}

async fn joe() -> obj {                // functions are document data, just like fields
    for (const user in self.users) {
        if (user.name.lower().contains("joe")) return user;
    }
    null
}

#[main]                          // main attribute to mark this func for 'run'
#[custom({'ex': true})]          // metadata values (funcs, maps, objs, etc.)
fn main() {
    const joe = self.joe();
    assert(this.attributes().get("custom").get("ex"));
    
    async {                                // async at the core (funcs & exprs too)
        let body = stringify("toml", await joe); // any loaded format (binary & parse too)
        body.push("stof = true\n");
        pln(body);
    }
}
```
``` bash
> stof run example.stof
age = 32
height = 185.42
name = "Joe Schmo"
stof = true
```

## Installation
Add Stof to your `Cargo.toml`:
```toml
[dependencies]
stof = 0.8.*
```

See [installation docs](https://docs.stof.dev/book/installation) for CLI instructions and more information.

## Documentation
- [Hello, World](https://docs.stof.dev/book/hello-world)
- [Roadmap](https://docs.stof.dev/roadmap)
- [Install](https://docs.stof.dev/book/installation)
- [GitHub](https://github.com/dev-formata-io/stof)

## Status
Stof is currently in **early development**, we welcome feedback and contributions. The core is stable for experimentation, and is actively being used in production at [Formata](https://formata.io).

New features are being added weekly, so hop into the Discord and get involved!

## License
Apache 2.0. See LICENSE for details.

## Feedback & Community
- Open issues or discussions on [GitHub](https://github.com/dev-formata-io/stof)
- Chat with us on [Discord](https://discord.gg/Up5kxdeXZt)
- Star the project to support future development!

> Reach out to info@stof.dev to contact us directly
