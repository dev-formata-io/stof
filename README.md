# Stof

[Stof](https://docs.stof.dev) is a unified data interface and interchange format for creating, sharing, and manipulating data. As a data format, it can replace other interchange formats like JSON, TOML, YAML, etc... As a data interface and runtime, it can remove the fragile and cumbersome parts of combining and using data from your application.

Created for and used in the [Formata](https://formata.io) platform, Stof is useful for creating distributed systems, API development, system integration, configurations, and data organization in general.

Stof is a dutch/german word for cloth, stuff, or fabric. It makes a good pun "just use some Stof" and a good file extension ".stof". Also makes sense since Stof weaves data together (if meaning matters to you). Personally, I like the pun and it hasn't gotten old yet.

## Getting Started
- Take a look at the docs [here](https://docs.stof.dev).
- See Stof in action with [Formata](https://formata.io).
- [Contact us](https://formata.io/contact-us) for more information.

## Adding Stof to your Project
### Rust
```sh
cargo add stof
```
### TypeScript (JSR)
[Stof JSR](https://jsr.io/@formata/stof)

We're adding languages and ways to use Stof all the time - please help if your interested.

## Why Use Stof?

As a programmer, the code, SDKs, and APIs required just to get data into and out of an application is a huge pain. It leads to a lot of system fragility and difficult to maintain software - costing time, frustration, and lots of dread for engineers.

Whether you create your own services/APIs to handle this logic (microservice), an iPaaS platform, or an SDK/embedded solution, dealing with this at the application layer always requires some special logic, parsing, or manipulation to be useful. This is because we have application interfaces and data formats, but a lack of data interfaces. The burden of making data useful falls on the application (programmer) using it, often requiring a lot of custom middleware or additional dependencies.

Stof solves this, allowing you to create the data interface that makes sense for your application/use-case and move the complexity of combining, parsing, and structuring data into Stof. With Stof, the data molds itself to your use cases instead of the application having to wrangle the data over and over again to use it.

[Formata](https://formata.io) takes this concept to the next level by offering a hosted solution for Stof logic that combines many distributed systems and APIs the way you need them. [Contact us](https://formata.io/contact-us) if Formata sounds like it could be a good fit for your needs.

## Example
``` rust
/**
 * We're already in the 'root' object scope, so outer braces are optional.
 * Field names don't need quotes, but can be double or single-quoted.
 * We don't need commas to separate fields.
 *     - Optional commas or semi-colons to end declarations.
 *     - Trailing commas or semi-colons are allowed.
 *     - Trailing commas in arrays are allowed.
 * Stof adds types, which are optional when declaring a field.
 * Field declarations are expressions.
 *     - Call functions, do some math, etc...
 * Stof numbers can be an "int", "float", or some units (a variant of float).
 *     - Unit conversions are made for you on casts and operations.
 *
 * Take a look at the Stof docs for a complete overview of features.
 */
users: [
    {
        first: 'Bob'
        last: 'Smith'
        user_for: +12days
    },
    {
        first: 'Jane'
        last: 'Doe'
        user_for: 20hrs + 33min
    },
    {
        first: 'Jerry'
        last: 'Smith'
        user_for: 300hrs
    }
]

fn oldest(): obj {
    let time = 0;
    let oldest = null;
    for (user in self.users) {
        if (user.user_for > time) {
            time = user.user_for;
            oldest = user;
        }
    }
    return oldest;
}

// Fields and functions can have attributes. Here, this function is marked
// as a test, testing equality between the return value and given expression.
#[test("Jerry Smith has been a user for 300hr")]
fn get_oldest(): str {
    let oldest = self.oldest();
    return `${oldest.first} ${oldest.last} has been a user for ${oldest.user_for as hours}`;
}
```
This test can be found in the Stof test suite in `src/tests/practical/docs.stof`. Stof tests can be run with `cargo test stof_test_suite -- --nocapture`.

## Features
### Data Unification
Currently, Stof has implementations for many common data formats out of the box. Stof can upgrade these formats into Stof, unifying and merging the data so it can all be worked with at once.

Formats are pluggable in Stof - you can replace formats, add your own, etc... Stof was designed with very complex data in mind, allowing in many cases for more efficient and much more capable data representation.

We're adding formats all the time, so submit an issue or reach out if you need a specific format added.

### Logic, Types, and Data Interfaces
Stof adds functions and types to your data, as data. A Stof document can use the functions it contains to manipulate itself in its own sandboxed environment. The app/system calling into Stof has complete control over Stof's access to its internals and to the outside world. By default, Stof can only interact with certain types of data it contains (fields & functions). You can also put permissions on what can be accessed/modified by functions.

Interfaces to the Stof data in the form of types and functions can be parsed into or removed from the document at any time (just like fields or any other data). This makes it possible for interfaces to be dynamically combined when and where you need them.

### No Additional Tooling or Complex Build Steps
Stof is written in Rust, but we're making it available via WebAssembly and should be available in all languages, enabling it to be embedded wherever you like to work with data. Stof documents are their own environments, able to parse additional Stof, import other formats, export data, manipulate itself, etc... Therefore, there is nothing to build - just start using it.

### Over-The-Wire
Stof was initially created to solve the problem of sending data and logic together over the wire efficiently in distributed systems, with the idea that logic as data is much less expensive to send than large amounts of data. Also makes things more secure and efficient by keeping data in place as much as possible.

You can think of Stof as just a JSON request body over the wire, except instead of just fields, you can safely share logic (functions) between systems as well. The system on the other side has complete control for what Stof has access to, with only the data it contains being accessible by default.

### Human Readability
Stof is very human readable in its text form (".stof"). As a superset of JSON, valid JSON will always be valid Stof. However, it's been improved on quite a bit.

- Both line comments and block comments are allowed
- Field names do not need quotes, but can have double or single quotes if preferred
- Whitespace doesn't really matter, allowing you to format fields the way they can be most understood
- Braces are used to denote object scopes, which help us programmers quickly see where things are at (no small dashes, etc... that require focus to read)
- Use commas, semi-colons, or nothing at all to separate field declarations
- Use types to explicitly show humans and the computer what type a field should be (and cast where needed)
    - Stof even adds unit types and does unit conversions for you (No more "was this seconds or milliseconds?")
- Field values are expressions - you can do math, cast types, call functions, init data, etc...
- If you like a different format better, it's Stof so just import/parse it into the document and keep going
    - Very helpful for legacy stuff - can import/parse into a specific Stof location for use

## Contributing
This is a brand new project, so we could use all the help we can get. Take a look first at the Stof test suite in `src/tests/tests.stof`, this will give a good jumping off point, but there's a lot to do still overall. We'll update this section as the project progresses with more details on how you can specifically get involved.

## License
Apache 2.0. See LICENSE.md for details.
