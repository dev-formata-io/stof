<p align="center"><img src="./content/stof.png" height="180"></p>
<p align="center">
    <a href="https://docs.stof.dev"><img src="https://img.shields.io/badge/docs-docs.stof.dev-purple?logo=gitbook&logoColor=white"></a>
    <a href="https://discord.gg/Up5kxdeXZt"><img src="https://img.shields.io/discord/1319468398169686016?logo=discord&logoColor=white"></a>
</p>
<p align="center">
    <a href="https://github.com/dev-formata-io/stof/actions"><img src="https://img.shields.io/github/actions/workflow/status/dev-formata-io/stof/rust.yml"></a>
    <a href="https://crates.io/crates/stof"><img src="https://img.shields.io/crates/d/stof"></a>
    <a href="https://crates.io/crates/stof"><img src="https://img.shields.io/crates/l/stof"></a>
    <a href="https://github.com/dev-formata-io/stof/commits/main/"><img src="https://img.shields.io/github/commit-activity/m/dev-formata-io/stof"></a>
    <a href="https://crates.io/crates/stof"><img src="https://img.shields.io/crates/size/stof"></a>
</p>

----

[Stof](https://stof.dev) is efficient, governable, and accessible data that is much simpler to use, offering fine-grained control and sandboxed manipulation between computer systems without the need for additional application code, servers, and dependencies.

- [Docs](https://docs.stof.dev)
- [Discord](https://discord.gg/Up5kxdeXZt)
- [Contact Us](https://stof.dev/contact-us)
- [Introduction & Design](https://docs.stof.dev/book/introduction-and-design)
- [More Information](https://docs.stof.dev/resources-and-information)

<br/>

![alt text](./content/stof.gif)

## Why
The logic and cost involved in deciphering, unifying, and validating data entering and leaving computer systems are growing exponentially, becoming a bottleneck and limiting factor for next-generation federated applications and AI.

Stof drastically decreases the amount of application code it takes to do this by moving this logic into the data layer, creating a standard interface that can be used by all systems today for cheaper and higher-quality access to data, while improving application-level security, developer experience, and data governance.

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
> For declaring data, Stof is a superset of JSON - valid JSON will always be valid Stof.
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
### Accessibility & Unification
Simplify data access, making it more straightforward to interact with diverse data sources and destinations. Stof works with other data formats instead of competing, meaning no matter what data format or schema you're interfacing with, Stof can help.

### Code as Data
Add custom logic to Stof documents, leveraging their data for a more capable and robust data interface. Offer a safe way to send logic over the network, tying systems together at the data layer.

### Validation
Ensure integrity by validating data using schemas, defined in Stof, reducing errors and maintaining consistency across applications.

### Transformation & Structure
Dynamically manipulate data, enabling seamless transformations to meet specific application needs, such as converting units or restructuring data formats.

### Orchestration
Stof is a logical layer between data sources and applications, capable of unification, transformation, validation, structuring, access control, and orchestration between systems, replacing fragile application code typically responsible for these tasks.

## Contributing
Take a look first at the Stof test suite in `src/tests/tests.stof`, this will give a good jumping off point, but there's a lot to do still overall. We'll update this section as the project progresses with more details on how you can specifically get involved.

There are lots of opportunities to contribute and use Stof. If you have ideas or want to get involved, just reach out and we'll make it happen.

Please also consider sponsoring and supporting this project as it's the only way we'll make strides towards our vision of a world where data is more secure, easily governed, and readily available - one with efficient and user-friendly access to high-quality data that is simple and native to the federated and decentralized world we are quickly moving towards.

## License
Apache 2.0. See LICENSE for details.
