//
// Copyright 2024 Formata, Inc. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

use std::sync::Arc;
use imbl::vector;
use crate::{model::{prompt::{PROMPT_ANY, PROMPT_AT, PROMPT_CLEAR, PROMPT_EMPTY, PROMPT_INSERT, PROMPT_LEN, PROMPT_LIB, PROMPT_POP, PROMPT_PROMPTS, PROMPT_PUSH, PROMPT_REMOVE, PROMPT_REPLACE, PROMPT_REVERSE, PROMPT_SET_TAG, PROMPT_SET_TEXT, PROMPT_STR, PROMPT_TAG, PROMPT_TEXT}, LibFunc, Param}, runtime::{instruction::Instructions, NumT, Type}};


pub fn prompt_str() -> LibFunc {
    LibFunc {
        library: PROMPT_LIB.clone(),
        name: "str".into(),
        is_async: false,
        docs: r#"# Prompt.str(prompt: prompt) -> str
Convert this prompt into a string, just like a cast to 'str' would do.
```rust
const p = prompt('hello', 'greet');
assert_eq(p.str(), '<greet>hello</greet>');
```"#.into(),
        params: vector![
            Param { name: "prompt".into(), param_type: Type::Prompt, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(PROMPT_STR.clone());
            Ok(instructions)
        })
    }
}

pub fn prompt_text() -> LibFunc {
    LibFunc {
        library: PROMPT_LIB.clone(),
        name: "text".into(),
        is_async: false,
        docs: r#"# Prompt.text(prompt: prompt) -> str
Get the text portion of this prompt (ignoring any sub-prompts & tag).
```rust
const p = prompt('hello', 'greet');
assert_eq(p.text(), 'hello');
```"#.into(),
        params: vector![
            Param { name: "prompt".into(), param_type: Type::Prompt, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(PROMPT_TEXT.clone());
            Ok(instructions)
        })
    }
}

pub fn prompt_tag() -> LibFunc {
    LibFunc {
        library: PROMPT_LIB.clone(),
        name: "tag".into(),
        is_async: false,
        docs: r#"# Prompt.tag(prompt: prompt) -> str
Get the string tag for this prompt, or null if not present.
```rust
const p = prompt('hello', 'greet');
assert_eq(p.tag(), 'greet');
```"#.into(),
        params: vector![
            Param { name: "prompt".into(), param_type: Type::Prompt, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(PROMPT_TAG.clone());
            Ok(instructions)
        })
    }
}

pub fn prompt_prompts() -> LibFunc {
    LibFunc {
        library: PROMPT_LIB.clone(),
        name: "prompts".into(),
        is_async: false,
        docs: r#"# Prompt.prompts(prompt: prompt) -> list
Return this prompts list of sub-prompts.
```rust
const p = prompt('hello', 'greet', prompt('a thing', 'sub'));
assert_eq(p.str(), '<greet>hello<sub>a thing</sub></greet>');
assert_eq(p.prompts(), [prompt('a thing', 'sub')]);
```"#.into(),
        params: vector![
            Param { name: "prompt".into(), param_type: Type::Prompt, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(PROMPT_PROMPTS.clone());
            Ok(instructions)
        })
    }
}

pub fn prompt_set_text() -> LibFunc {
    LibFunc {
        library: PROMPT_LIB.clone(),
        name: "set_text".into(),
        is_async: false,
        docs: r#"# Prompt.set_text(prompt: prompt, text: str) -> void
Set the text portion of this prompt.
```rust
const p = prompt('hello', 'greet');
p.set_text('hello, world');
assert_eq(p.str(), '<greet>hello, world</greet>');
```"#.into(),
        params: vector![
            Param { name: "prompt".into(), param_type: Type::Prompt, default: None },
            Param { name: "text".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(PROMPT_SET_TEXT.clone());
            Ok(instructions)
        })
    }
}

pub fn prompt_set_tag() -> LibFunc {
    LibFunc {
        library: PROMPT_LIB.clone(),
        name: "set_tag".into(),
        is_async: false,
        docs: r#"# Prompt.set_tag(prompt: prompt, tag: str) -> void
Set the tag portion of this prompt. Set to null to clear the tag.
```rust
const p = prompt('hello', 'greet');
p.set_tag('msg');
assert_eq(p.str(), '<msg>hello, world</msg>');
```"#.into(),
        params: vector![
            Param { name: "prompt".into(), param_type: Type::Prompt, default: None },
            Param { name: "tag".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(PROMPT_SET_TAG.clone());
            Ok(instructions)
        })
    }
}

pub fn prompt_len() -> LibFunc {
    LibFunc {
        library: PROMPT_LIB.clone(),
        name: "len".into(),
        is_async: false,
        docs: r#"# Prompt.len(prompt: prompt) -> int
The number of sub-prompts contained within this prompt.
```rust
const p = prompt('hello', 'greet', prompt('hello, world'));
assert_eq(p.len(), 1);
```"#.into(),
        params: vector![
            Param { name: "prompt".into(), param_type: Type::Prompt, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(PROMPT_LEN.clone());
            Ok(instructions)
        })
    }
}

pub fn prompt_at() -> LibFunc {
    LibFunc {
        library: PROMPT_LIB.clone(),
        name: "at".into(),
        is_async: false,
        docs: r#"# Prompt.at(prompt: prompt, index: int) -> prompt
Get the sub-prompt at a given index (like a list). Will return null if the prompt does not have any sub-prompts.
```rust
const p = prompt('hello', 'greet', prompt('hello there'));
const sub = p[0];
assert_eq(sub, prompt('hello there'));
```"#.into(),
        params: vector![
            Param { name: "prompt".into(), param_type: Type::Prompt, default: None },
            Param { name: "index".into(), param_type: Type::Num(NumT::Int), default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(PROMPT_AT.clone());
            Ok(instructions)
        })
    }
}

pub fn prompt_any() -> LibFunc {
    LibFunc {
        library: PROMPT_LIB.clone(),
        name: "any".into(),
        is_async: false,
        docs: r#"# Prompt.any(prompt: prompt) -> bool
Does this prompt have any sub-prompts?
```rust
const p = prompt('hello', 'greet');
assert(!p.any());
```"#.into(),
        params: vector![
            Param { name: "prompt".into(), param_type: Type::Prompt, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(PROMPT_ANY.clone());
            Ok(instructions)
        })
    }
}

pub fn prompt_empty() -> LibFunc {
    LibFunc {
        library: PROMPT_LIB.clone(),
        name: "empty".into(),
        is_async: false,
        docs: r#"# Prompt.empty(prompt: prompt) -> bool
Returns true if the prompt does not have any sub-prompts.
```rust
const p = prompt('hello', 'greet');
assert(p.empty());
```"#.into(),
        params: vector![
            Param { name: "prompt".into(), param_type: Type::Prompt, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(PROMPT_EMPTY.clone());
            Ok(instructions)
        })
    }
}

pub fn prompt_push() -> LibFunc {
    LibFunc {
        library: PROMPT_LIB.clone(),
        name: "push".into(),
        is_async: false,
        docs: r#"# Prompt.push(prompt: prompt, other: prompt | str) -> void
Push a sub-prompt to this prompt.
```rust
const p = prompt('hello', 'greet');
p.push(', world');
assert_eq(p as str, '<greet>hello, world</greet>');
```"#.into(),
        params: vector![
            Param { name: "prompt".into(), param_type: Type::Prompt, default: None },
            Param { name: "other".into(), param_type: Type::Union(vector![Type::Str, Type::Prompt]), default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(PROMPT_PUSH.clone());
            Ok(instructions)
        })
    }
}

pub fn prompt_pop() -> LibFunc {
    LibFunc {
        library: PROMPT_LIB.clone(),
        name: "pop".into(),
        is_async: false,
        docs: r#"# Prompt.pop(prompt: prompt) -> prompt
Pop a sub-prompt from the end of the sub-prompt list.
```rust
const p = prompt('hello', 'greet');
p.push(', world');
p.pop();
assert_eq(p as str, '<greet>hello</greet>');
```"#.into(),
        params: vector![
            Param { name: "prompt".into(), param_type: Type::Prompt, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(PROMPT_POP.clone());
            Ok(instructions)
        })
    }
}

pub fn prompt_clear() -> LibFunc {
    LibFunc {
        library: PROMPT_LIB.clone(),
        name: "clear".into(),
        is_async: false,
        docs: r#"# Prompt.clear(prompt: prompt) -> void
Clear all sub-prompts from this prompt.
```rust
const p = prompt('hello', 'greet');
p.push(', world');
p.clear();
assert_eq(p as str, '<greet>hello</greet>');
```"#.into(),
        params: vector![
            Param { name: "prompt".into(), param_type: Type::Prompt, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(PROMPT_CLEAR.clone());
            Ok(instructions)
        })
    }
}

pub fn prompt_reverse() -> LibFunc {
    LibFunc {
        library: PROMPT_LIB.clone(),
        name: "reverse".into(),
        is_async: false,
        docs: r#"# Prompt.reverse(prompt: prompt) -> void
Reverse all sub-prompts in this prompt.
```rust
const p = prompt(tag = 'greet');
p.push(', world');
p.push('hello');
p.reverse();
assert_eq(p as str, '<greet>hello, world</greet>');
```"#.into(),
        params: vector![
            Param { name: "prompt".into(), param_type: Type::Prompt, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(PROMPT_REVERSE.clone());
            Ok(instructions)
        })
    }
}

pub fn prompt_remove() -> LibFunc {
    LibFunc {
        library: PROMPT_LIB.clone(),
        name: "remove".into(),
        is_async: false,
        docs: r#"# Prompt.remove(prompt: prompt, index: int) -> prompt
Remove a sub-prompt at the given index.
```rust
const p = prompt(tag = 'greet');
p.push('hello');
p.push(', world');
p.remove(1);
assert_eq(p as str, '<greet>hello</greet>');
```"#.into(),
        params: vector![
            Param { name: "prompt".into(), param_type: Type::Prompt, default: None },
            Param { name: "index".into(), param_type: Type::Num(NumT::Int), default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(PROMPT_REMOVE.clone());
            Ok(instructions)
        })
    }
}

pub fn prompt_insert() -> LibFunc {
    LibFunc {
        library: PROMPT_LIB.clone(),
        name: "insert".into(),
        is_async: false,
        docs: r#"# Prompt.insert(prompt: prompt, index: int, other: prompt) -> void
Insert a sub-prompt into the given index.
```rust
const p = prompt(tag = 'greet');
p.push('hello');
p.insert(1, ', world');
assert_eq(p as str, '<greet>hello, world</greet>');
```"#.into(),
        params: vector![
            Param { name: "prompt".into(), param_type: Type::Prompt, default: None },
            Param { name: "index".into(), param_type: Type::Num(NumT::Int), default: None },
            Param { name: "other".into(), param_type: Type::Prompt, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(PROMPT_INSERT.clone());
            Ok(instructions)
        })
    }
}

pub fn prompt_replace() -> LibFunc {
    LibFunc {
        library: PROMPT_LIB.clone(),
        name: "replace".into(),
        is_async: false,
        docs: r#"# Prompt.replace(prompt: prompt, index: int, other: prompt) -> void
Replace a sub-prompt at the given index.
```rust
const p = prompt(tag = 'greet');
p.push('hello');
p.replace(0, 'yo');
assert_eq(p as str, '<greet>yo</greet>');
```"#.into(),
        params: vector![
            Param { name: "prompt".into(), param_type: Type::Prompt, default: None },
            Param { name: "index".into(), param_type: Type::Num(NumT::Int), default: None },
            Param { name: "other".into(), param_type: Type::Prompt, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(PROMPT_REPLACE.clone());
            Ok(instructions)
        })
    }
}
