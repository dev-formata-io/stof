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

### A declarative runtime & data format for modern workflows
- [Docs](https://docs.stof.dev)
- [GitHub](https://github.com/dev-formata-io/stof)
- [Discord](https://discord.gg/Up5kxdeXZt)
- [Install](https://docs.stof.dev/book/installation)


Stof is a unified data format that works seamlessly **with** other formats to bridge the gap between static data and programmable documents. It is a lightweight, embeddable, and portable data logic format & platform for AI, infra-as-code, and config-heavy workflows. It's built to support:

- Data-Mesh, Integration, & Orchestration **glue-layer**
- Universal LLM & AI workflows, tools, & **intersystem data**
- Smart configs with logic, types, units, schemas, & **self-validation**
- Asynchronous **validation & transformation**

> Think of Stof as a foundation for building robust and declarative data flows, config systems, or backend models.

## Core Stof principle: Everything as Data
Using data in whatever form it is defined should not be difficult. Stof is the glue-layer & interface for working with any type of data as a singular unified, portable, and embeddable document.

Stof accomplishes this by treating every piece of data as a component in a general graph (document) of containers. Whether it's functions, fields, PDFs, binaries, or anything else, Stof organizes it neatly and provides an interface for it.

The Stof runtime is a thin, embeddable runtime that allows a Stof document to manipulate itself through calling the functions it contains. Functions are just pieces of data like a field, so they can even operate on themselves.

Libraries are the only way a Stof function can operate outside of the document (Ex. HTTP, filesystem, etc.), which are not saved with the document and are controlled by the host system. This sandboxed behavior is advantageous for sending logic + data over networks or in any untrusted environments.

## When to use Stof?
Modern software (especially AI/ML, infra, cloud, CI/CD, and workflows) increasingly relies on structured data that needs to be:
- Human-readable
- Machine-validatable
- Extendable with logic
- Executable safely
- Translatable between formats
- Transported between systems
- Versioned and inspectable

But the tools we have for this are *primitive and fragmented*:
- JSON/YAML/TOML carry structure, but rely on other tools for behavior, types, units, schemas, or validations.
- External tools create complexity between systems and often require additional configuration & maintenence.
- Configs drift and break across environments.
- Runtime logic is scattered across codebases, devops scripts, and data definitions.

> Stof unifies **structure + validation + behavior** into one **coherent, inspectable, portable artifact**.

*Note: if you're doing simple config loading or small and static data modeling, learning Stof might feel like overkill. You can replicate most of what Stof does using JSON + code + libraries; it just takes more effort and lacks formality, organization, single-file simplicity, unification, etc. (also a big pain for cross-boundary systems, like APIs, teams, and services).*

## JSR Example
``` typescript
import { StofDoc } from 'jsr:@formata/stof';
const doc = await StofDoc.new();

// Stof Std pln function mapped to console.log
doc.lib('Std', 'pln', (... vars: unknown[]) => console.log(...vars));

// My example nested function that is async, mapped to an async Stof lib fn
doc.lib('Example', 'nested', async (): Promise<Map<string, string>> => {
    const res = new Map();
    res.set('msg', 'hello, there');
    res.set('nested', await (async (): Promise<string> => 'this is a nested async JS fn (like fetch)')());
    return res;
}, true);

// Add some Stof
doc.parse(`
    fn main() {
        const res = await Example.nested();
        pln(res);
    }
`);
await doc.call('main');

/* OUTPUT
Map(2) {
  "msg" => "hello, there",
  "nested" => "this is a nested async JS fn (like fetch)"
}
*/
```

## Example/Tour
> Located in `examples/readme` for you to try yourself.
``` rust
/*
 * True data interop!
 * Import/export any format - our goal is to work seamlessly WITH other formats.
 * Organize data, APIs, configs, connectors, pipelines, AI, etc. with objects & paths, as a graph.
 */
import "old_crusty_config.toml" as OldCrusty; // Creates a new root object named "OldCrusty"
import "norse.png" as self.NorseImage;        // Creates a child object with path "root.NorseImage"


/*
 * Define objects and fields in a JSON-like way.
 */
{
    "json-like": "json objects & fields can be parsed by Stof natively",
    str "just stof": "it's all Stof, so use types, etc."
}

field: 42              // Optionally end field declarations with a comma or semi-colon
str typed: "types!"    // Field types for consistency, readability, and reliability
const bool cool: true; // Constant fields for immutable data (with or without type)

MiB unit-types: OldCrusty.server.RAM; // Unit types for conversions on casts, operations, etc.
cm height: 6ft + 1in;                 // Expressions and a full runtime always available


/*
 * Functions & logic as data. Manipulate this document from within.
 *
 * - Sandboxed (host defines system access, if any)
 * - Portable (send and execute over a network)
 * - Simple (familiar, approachable, and intuative, without looking like a blob of JSON)
 */
fn hello_world() -> str {
    "hello, world"
}


/*
 * Use objects to organize every type of data, including logic & APIs.
 * Keyword "self" always refers to the current object.
 * Keyword "super" always refers to the parent object (if any).
 * Navigate the graph of data via dot separated paths (both down and up, including other roots).
 * Use "?" in front of function calls to return null if the function doesn't exist.
 */
Api: {
    import "api"; // default ".stof" extension if not provided
    str config-name: ?self.modular_apis(OldCrusty); // "Old Crusty"
}


/*
 * Attributes for metadata & control (both fields & functions).
 *
 * - Used for main, test, and async functions
 * - Reference objects, functions, and data (validation, access control, schemas, etc.)
 * - Always available programmatically for metadata access
 */
#[my_attribute]
#[attribute_with_val(42days)]
#[metadata_map({"config": true})]
#[metadata_obj(new { stof_object: true })]
#[validation((value: unknown): bool => true)]
metadata_field: "we have attributes"


/*
 * Foundational async behavior for modern networking and concurrency.
 * No threads by default, but host system can implement async as single or multi-threaded.
 *
 * Note: "async fn" just adds #[async] as a function attribute.
 *
 * - HTTP requests & APIs
 * - Database connections
 * - Event driven systems
 * - Declarative UI + logic
 */
async fn doing_something_async() {
    // "Std" library, an extensible & complete standard library
    pln("Hello, Stof!");

    // Turn any expression into an async expression
    const handle = async 42;
    assert_eq(await handle, 42);

    // Async block expressions (and regular block expressions)
    const result = await async {
        let a = 5;
        let b = 5;
        return a + b;
    };
    assert_eq(result, 10);
}


/*
 * Prototypes for modern, complex data models & interfaces.
 * See docs for details.
 *
 * - Schemas
 * - Modular APIs applied to static data
 * - Pipelines & workflows
 * - Inheritance via #[extends(obj | str)]
 */
#[type]
Config: {
    /**
     * Example use with imported OldCrusty TOML data.
     * ```
     * const config = OldCrusty as Config;
     * assert_eq(config.description(), "Left to drift & die, but is required somehow");
     * ```
     */
    fn description() -> str {
        self.description ?? "no description"
    }
}


/**
 * Prompt primitive type for AI workflows.
 * - Trees of optionally structured prompts (strings with optional XML tags)
 * - Acts like a collection when you need it and a string when you don't
 * - A better, more maintainable way to create modern AI apps & agents
 */
fn create_prompt() -> prompt {
    const llm_prompt = prompt();

    // newlines just for the example...
    const add_to_llm = (pmt: prompt, llm: prompt) => {
        llm.push(pmt);
        llm.push("\n");
    };

    const data = prompt(tag="data");
    data.push("seamless str <-> prompt casting");
    add_to_llm(data, llm_prompt);

    const format = prompt(tag="format");
    format.push("1. first thing. ");
    format.push("2. second thing.");
    add_to_llm(format, llm_prompt);

    const instructions = prompt(
        text="LLMs are good at textual data, humans are not. Stof helps.",
        tag="instructions"
    );
    add_to_llm(instructions, llm_prompt);
    llm_prompt.pop(); // pop final newline prompt (yes, there's a lib)

    llm_prompt
}


#[main]
/**
 * Stof CLI uses #[main] attributes by default to tell which functions to run with `stof run`.
 * Change this with the -a option, allowing you to use attributes instead of multiple scripts.
 *
 * Run with CLI via `stof run readme.stof` - you'll see `Hello, Stof!` printed to your console.
 * Run with `stof -d run readme.stof` - you'll now see YAML output in addition.
 *
 * Logging (log_debug, log_trace, log_info, log_warn, log_error) built in:
 * - `stof run readme.stof` - only log_warn & log_error
 * - `stof -d run readme.stof` - log_info in addition to warn & error
 * - `stof -dd run readme.stof` - all logs available
 * - uses "log" Rust crate for flexibility
 */
fn main() {
    self.doing_something_async();

    const yaml = stringify('yaml', self);
    log_info(yaml); // use -d

    const prompt = self.create_prompt();
    log_debug(prompt as str); // use -dd
}
```
``` bash
> cd examples/readme
> stof run readme.stof
Hello, Stof!
```

## Installation
Add Stof to your `Cargo.toml`:
```toml
[dependencies]
stof = "0.8.*"
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
