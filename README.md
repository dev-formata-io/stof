<p align="center"><img src="./content/stof.png" height="150"></p>
<p align="center">
    <a href="https://docs.stof.dev"><img src="https://img.shields.io/badge/docs-docs.stof.dev-purple?logo=gitbook&logoColor=white"></a>
    <a href="https://discord.gg/Up5kxdeXZt"><img src="https://img.shields.io/discord/1319468398169686016?logo=discord&logoColor=white"></a>
</p>
<p align="center">
    <a href="https://github.com/dev-formata-io/stof/actions"><img src="https://img.shields.io/github/actions/workflow/status/dev-formata-io/stof/rust.yml"></a>
    <a href="https://crates.io/crates/stof"><img src="https://img.shields.io/crates/d/stof"></a>
    <a href="https://crates.io/crates/stof"><img src="https://img.shields.io/crates/l/stof"></a>
    <a href="https://github.com/dev-formata-io/stof/commits/main/"><img src="https://img.shields.io/github/commit-activity/m/dev-formata-io/stof"></a>
</p>

----

<br/>
<p align="center"><img src="./content/overview.png" height="200"></p>
<br/>

[Stof](https://stof.dev) is a simple data runtime for all of your stuff. It unifies data handling and makes working with data much more enjoyable.

- [Docs](https://docs.stof.dev)
- [Discord](https://discord.gg/Up5kxdeXZt)
- [Contact Us](https://stof.dev/contact-us)
- [Introduction & Design](https://docs.stof.dev/book/introduction-and-design)
- [More Information](https://docs.stof.dev/resources-and-information)

<br/>
<br/>
<p align="center"><img src="./content/stof.gif"></p>
<br/>

## Why
Stof drastically decreases the amount of application code it takes to manipulate data by moving logic into the data that needs manipulating (typically done the other way around), creating a standard interface that can be used for cheaper and higher-quality access, while improving security, developer experience, and governance.

## Getting Started
### CLI
[CLI](https://docs.stof.dev/reference/cli) is a standalone way to use Stof outside of an embedded environment.
```bash
cargo install stof-cli
```
### Rust
The Stof [Rust Crate](https://crates.io/crates/stof) is the most fully-featured way to embed and work with Stof. However, we are adding as many languages as possible to this list.
```bash
cargo add stof
```
### TypeScript (JSR)
It is currently possible to use Stof from the browser and in JavaScript host environments through WebAssembly. In this context, Stof is not yet fully featured.
[Stof JSR](https://jsr.io/@formata/stof)

## Example
> Note: Stof is also a superset of JSON
``` rust
users: [
    {
        name: "Joe Schmo",       // commas or semi-colons accepted, but optional
        cm height: 6ft + 1in     // Stof adds units and declarations are expressions
        age: +32;                // trailing commas or semi-colons are okay
    },                           // trailing commas in arrays are okay
]

fn getJoe(): obj {               // Stof adds data types (casting, etc..)
    for (user in self.users) {
        if (user.name.toLower().contains("joe")) return user;
    }
    return null;
}

#[main]
fn main() {
    let joe = self.getJoe();
    pln(stringify(joe, 'toml')); // any format loaded into the doc (parse too)
}
```
``` bash
> stof run example.stof
age = 32
height = 185.42
name = "Joe Schmo"
```

## Why Use Stof?
Use Stof for just about any data interchange use-case. It's particularly helpful for data unification and sending APIs (logic) over the wire (and combining APIs dynamically).

1. Data Unification
    - Use different types of data together at once (PDFs, JSON, YAML, XML, Images, DOCX, etc).
2. Remote Execution
    - Send Stof over networks as a lightweight and sandboxed way to execute logic remotely.
3. Data Validation
    - Use Stofs type system to quickly and easily filter/validate data according to your needs.
4. Data Transformation & Connectors
    - Connect data to your application through Stof for easy restructuring & access.

## Contributing
Theres a lot to do, so please jump in and consider supporting the project.

## License
Apache 2.0. See LICENSE for details.
