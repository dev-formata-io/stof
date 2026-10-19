# Stof

Stof is JSON with functions. One document holds data and the logic that works on it, and your app runs it in a sandbox, in-process, from Rust, TypeScript (WebAssembly), or Python. This plugin teaches Claude the Stof language, its standard libraries, and the host APIs, so it can write, review, and debug Stof for you.

## What it adds

One skill, `stof`. Claude loads it when you work with a `.stof` file, ask about Stof syntax or an error, convert JSON, YAML, or TOML to Stof, write self-validating configs with `#[schema]`, define prototype types with `#[type]`, write `#[main]`, `#[test]`, or `#[run]` functions, use the `stof` CLI, or embed Stof with `StofDoc` (JavaScript), `Doc` (Python), or `Graph` and `Runtime` (Rust).

- `skills/stof/SKILL.md` is the language guide: the mental model, the rules that trip people up, and worked examples.
- `skills/stof/references/libs/` holds one reference page per standard library (`Std`, `Str`, `List`, `Map`, `Obj`, `Time`, `Http`, and the rest). Claude reads only the page it needs.

## Install

On claude.ai or in Cowork, add Stof from the plugin directory. In Claude Code:

```
/plugin install stof --marketplace dev-formata-io/stof
```

## Use it

Ask Claude things like:

- "Turn this JSON config into Stof with a schema that validates it."
- "Why does this Stof function return null?"
- "Write tests for this document and run them with `stof test`."
- "Embed this document in my Node app and call its functions."

## Data

The plugin is only Markdown instructions. It runs no code, connects to no services, and sends or stores no data. Anything Claude runs while using it (for example the `stof` CLI) runs on your machine with your permission.

## Links

- Docs and playground: https://stof.dev
- Source: https://github.com/dev-formata-io/stof
- Issues: https://github.com/dev-formata-io/stof/issues

Stof is open source under the Apache 2.0 license, made by Formata, Inc.
