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

[Stof](https://stof.dev) is the simplest way to create, store, share, and transform unified data.

- [Docs](https://docs.stof.dev)
- [Discord](https://discord.gg/Up5kxdeXZt)
- [Contact](https://stof.dev/contact-us)

## Example
> Stof also parses normal JSON object syntax

> Check out the examples (and test 'src/model/formats/stof/tests') folder, with real examples for you to play with

``` rust
const list users: [              // optional type and const specification for fields
    {
        name: "Joe Schmo",       // commas or semi-colons accepted, but optional
        cm height: 6ft + 1in     // Stof adds units and declarations are expressions
        age: +32;                // trailing commas or semi-colons are okay
    },                           // trailing commas in arrays are okay
]

fn joe() -> obj {                // functions are document data, just like fields
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
        let body = stringify("toml", joe); // any loaded format (binary & parse too)
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

## Motivations
1. If you've ever wished you could put different data formats into a singular document and manipulate it all at once with a standard (and modern) interface, you're in the right spot.

2. If you've wanted a singular API that could be used across servers and within different environments, well now you can.

3. If you've been looking for that YAML or JSON replacement/addition that adds a simple programming layer and type system that just makes sense, keep reading.

4. If you have an embedded environment where you need to run user sent or untrusted code (especially over the wire), you're pretty good at looking for solutions because you're here.

5. If you've ever thought to yourself "code is just data itself, so why can't I work with it as such?", then you're also in the right place.

## How
Stof can look and feel like a familiar programming interface, but it's actually just a document of data that comes with a runtime for manipulating itself in a sandbox you control.

Because it's just a normal document of data, it can be shared, stored, combined, split, and otherwise transformed, and seamlessly works with other formats (both binary and text import/export).

We say "data" instead of "field" (like other formats) because a field is just one type of data component (functions are another). Stof is built like an Entity Component System, where components (data) can be anything you'd like, even large PDF documents, 3D models, or binary voice data. All can then be organized cleanly and worked with via Stof functions at the same time.

## Contributing
We have an awesome and growing community, so jump in and consider supporting the project.

Any fellow programming nerds are probably filled with a lot of good ideas at this point, so lets chat (and please get involved)! Email info@stof.dev or find me via GitHub profile.

## License
Apache 2.0. See LICENSE for details.
