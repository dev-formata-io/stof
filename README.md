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

[Stof](https://stof.dev) is a simple and embeddable data runtime. Expanding on hypermedia concepts, Stof documents make working with data much more enjoyable by unifying many data types and APIs to be used together from one place.

- [Docs](https://docs.stof.dev)
- [Discord](https://discord.gg/Up5kxdeXZt)
- [Contact Us](https://stof.dev/contact-us)
- [Introduction & Design](https://docs.stof.dev/book/introduction-and-design)
- [More Information](https://docs.stof.dev/resources-and-information)

## Why
> Unified data that manipulates itself is the foundation for portable, composable, secure, and universal APIs.

Combining APIs and manipulating the data to, from, and within them is a pain, made worse by the plethora of programming languages and SDKs that are used.

Stof is a glue format for putting all of this together and a standard interface from your language of choice (or standalone) for working with data and APIs.

If you're sending APIs and data over a network, combining many APIs, or are using a lot of different data types at once, Stof might be a good choice for your project.

## Example
> Note: Stof documents are also a superset of JSON
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

<br/>
<br/>
<p align="center"><img src="./content/stof.gif"></p>
<br/>

## Why Use Stof?
Use Stof for just about any data interchange use-case. It's particularly helpful for data unification, sending APIs (logic) over the wire, and combining APIs dynamically.

As an example, it would require many SDKs, dependencies, and a lot of application dependant logic to use the Google Drive API to get document data (images, PDF, DOCX, JSON, etc.), send unified data to an LLM provider, then post a response in Slack or Discord.

All of this plus the user configuration, conversation data, and auth can be contained within a singular Stof document, sent between servers, persisted in databases or to disk (even partially), and be executed anywhere Stof runs (most places due to WebAssembly).

1. Data Unification
    - Use different types of data together at once (PDFs, JSON, YAML, XML, Images, DOCX, etc).
2. Remote Execution
    - Send Stof over networks as a lightweight and sandboxed way to execute logic remotely.
3. Data Validation
    - Use Stofs type system to quickly and easily filter/modify/validate data according to your needs.
4. Data Transformation & Connectors
    - Upgrade and downgrade data to and from Stof using the formats of your choice for interoperability, transformations, translations, and data connectors.

## Contributing
Theres a lot to do, so please jump in and consider supporting the project.

## License
Apache 2.0. See LICENSE for details.
