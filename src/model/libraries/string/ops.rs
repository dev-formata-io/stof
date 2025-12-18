//
// Copyright 2025 Formata, Inc. All rights reserved.
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
use arcstr::literal;
use imbl::vector;
use crate::{model::{LibFunc, Param, string::{AT, CONTAINS, ENDS_WITH, FIND_ALL, FIRST, INDEX_OF, IS_MATCH, LAST, LEN, LOWER, PUSH, REPLACE, SPLIT, STARTS_WITH, STR_LIB, SUBSTRING, TRIM, TRIM_END, TRIM_START, UPPER}}, runtime::{Num, NumT, Type, Val, instruction::Instructions, instructions::Base}};


/// Len.
pub fn str_len() -> LibFunc {
    LibFunc {
        library: STR_LIB.clone(),
        name: "len".into(),
        is_async: false,
        docs: r#"# Str.len(val: str) -> int
Returns the length (number of characters) in this string.
```rust
assert_eq("hello".len(), 5);
```
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(LEN.clone());
            Ok(instructions)
        })
    }
}

/// At.
pub fn str_at() -> LibFunc {
    LibFunc {
        library: STR_LIB.clone(),
        name: "at".into(),
        is_async: false,
        docs: r#"# Str.at(val: str, index: int) -> str
Returns a character at the given index within the string, or the last character if the index is out of bounds.
```rust
const val = "hello";
assert_eq(val[1], "e");
```
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Str, default: None },
            Param { name: "index".into(), param_type: Type::Num(NumT::Int), default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(AT.clone());
            Ok(instructions)
        })
    }
}

/// First.
pub fn str_first() -> LibFunc {
    LibFunc {
        library: STR_LIB.clone(),
        name: "first".into(),
        is_async: false,
        docs: r#"# Str.first(val: str) -> str
Return the first char (as a string) in this string.
```rust
const val = "hello";
assert_eq(val.first(), "h");
```
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(FIRST.clone());
            Ok(instructions)
        })
    }
}

/// Last.
pub fn str_last() -> LibFunc {
    LibFunc {
        library: STR_LIB.clone(),
        name: "last".into(),
        is_async: false,
        docs: r#"# Str.last(val: str) -> str
Return the last char (as a string) in this string.
```rust
const val = "hello";
assert_eq(val.last(), "o");
```
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(LAST.clone());
            Ok(instructions)
        })
    }
}

/// Starts with?
pub fn str_starts_with() -> LibFunc {
    LibFunc {
        library: STR_LIB.clone(),
        name: "starts_with".into(),
        is_async: false,
        docs: r#"# Str.starts_with(val: str, seq: str) -> bool
Does this string start with the given string sequence?
```rust
const val = "hello";
assert(val.starts_with("he"));
```
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Str, default: None },
            Param { name: "seq".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(STARTS_WITH.clone());
            Ok(instructions)
        })
    }
}

/// Ends with?
pub fn str_ends_with() -> LibFunc {
    LibFunc {
        library: STR_LIB.clone(),
        name: "ends_with".into(),
        is_async: false,
        docs: r#"# Str.ends_with(val: str, seq: str) -> bool
Does this string end with the given string sequence?
```rust
const val = "hello";
assert(val.ends_with("llo"));
```
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Str, default: None },
            Param { name: "seq".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(ENDS_WITH.clone());
            Ok(instructions)
        })
    }
}

/// Push.
pub fn str_push() -> LibFunc {
    LibFunc {
        library: STR_LIB.clone(),
        name: "push".into(),
        is_async: false,
        docs: r#"# Str.push(val: str, other: str) -> void
Pushes another string to the back of this string, leaving the other string unmodified.
```rust
const val = "hello";
val.push(", world");
assert_eq(val, "hello, world");
```
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Str, default: None },
            Param { name: "other".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(PUSH.clone());
            Ok(instructions)
        })
    }
}

/// Contains?
pub fn str_contains() -> LibFunc {
    LibFunc {
        library: STR_LIB.clone(),
        name: "contains".into(),
        is_async: false,
        docs: r#"# Str.contains(val: str, seq: str) -> bool
Return true if the sequence is found at least once anywhere in this string.
```rust
const val = "hello, world";
assert(val.contains(", w"));
```
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Str, default: None },
            Param { name: "seq".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(CONTAINS.clone());
            Ok(instructions)
        })
    }
}

/// Index Of.
pub fn str_index_of() -> LibFunc {
    LibFunc {
        library: STR_LIB.clone(),
        name: "index_of".into(),
        is_async: false,
        docs: r#"# Str.index_of(val: str, seq: str) -> int
Find the first occurrance of the given sequence in this string, returning the index of the first char. If not found, returns -1.
```rust
const val = "hello, world";
assert_eq(val.index_of(", w"), 5);
```
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Str, default: None },
            Param { name: "seq".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(INDEX_OF.clone());
            Ok(instructions)
        })
    }
}

/// Replace.
pub fn str_replace() -> LibFunc {
    LibFunc {
        library: STR_LIB.clone(),
        name: "replace".into(),
        is_async: false,
        docs: r#"# Str.replace(val: str, find: str, replace: str = "") -> str
Replace all occurrances of a find string with a replace string (default removes all occurrances). This will return a new string, without modifying the original.
```rust
const val = "hello john";
assert_eq(val.replace(" ", ", "), "hello, john");
```
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Str, default: None },
            Param { name: "find".into(), param_type: Type::Str, default: None },
            Param { name: "replace".into(), param_type: Type::Str, default: Some(Arc::new(Base::Literal(Val::Str(literal!(""))))) }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(REPLACE.clone());
            Ok(instructions)
        })
    }
}

/// Split.
pub fn str_split() -> LibFunc {
    LibFunc {
        library: STR_LIB.clone(),
        name: "split".into(),
        is_async: false,
        docs: r#"# Str.split(val: str, sep: str = " ") -> list
Splits a string into a list at the given separator.
```rust
const val = "hello, world";
assert_eq(val.split(", "), ["hello", "world"]);
```
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Str, default: None },
            Param { name: "sep".into(), param_type: Type::Str, default: Some(Arc::new(Base::Literal(Val::Str(" ".into())))) }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(SPLIT.clone());
            Ok(instructions)
        })
    }
}

/// Upper.
pub fn str_upper() -> LibFunc {
    LibFunc {
        library: STR_LIB.clone(),
        name: "upper".into(),
        is_async: false,
        docs: r#"# Str.upper(val: str) -> str
Return a new string with all characters converted to uppercase.
```rust
const val = "hello";
assert_eq(val.upper(), "HELLO");
```
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(UPPER.clone());
            Ok(instructions)
        })
    }
}

/// Lower.
pub fn str_lower() -> LibFunc {
    LibFunc {
        library: STR_LIB.clone(),
        name: "lower".into(),
        is_async: false,
        docs: r#"# Str.lower(val: str) -> str
Return a new string with all characters converted to lowercase.
```rust
const val = "HELLO";
assert_eq(val.lower(), "hello");
```
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(LOWER.clone());
            Ok(instructions)
        })
    }
}

/// Trim.
pub fn str_trim() -> LibFunc {
    LibFunc {
        library: STR_LIB.clone(),
        name: "trim".into(),
        is_async: false,
        docs: r#"# Str.trim(val: str) -> str
Return a new string with the whitespace (newlines, tabs, and space characters) removed from the front and back.
```rust
const val = "\n\thello\t\n";
assert_eq(val.trim(), "hello");
```
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(TRIM.clone());
            Ok(instructions)
        })
    }
}

/// Trim start.
pub fn str_trim_start() -> LibFunc {
    LibFunc {
        library: STR_LIB.clone(),
        name: "trim_start".into(),
        is_async: false,
        docs: r#"# Str.trim_start(val: str) -> str
Return a new string with the whitespace (newlines, tabs, and space characters) removed from the front only.
```rust
const val = "\n\thello\t\n";
assert_eq(val.trim_start(), "hello\t\n");
```
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(TRIM_START.clone());
            Ok(instructions)
        })
    }
}

/// Trim end.
pub fn str_trim_end() -> LibFunc {
    LibFunc {
        library: STR_LIB.clone(),
        name: "trim_end".into(),
        is_async: false,
        docs: r#"# Str.trim_end(val: str) -> str
Return a new string with the whitespace (newlines, tabs, and space characters) removed from the back only.
```rust
const val = "\n\thello\t\n";
assert_eq(val.trim_end(), "\n\thello");
```
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Str, default: None }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(TRIM_END.clone());
            Ok(instructions)
        })
    }
}

/// Substring.
pub fn str_substr() -> LibFunc {
    LibFunc {
        library: STR_LIB.clone(),
        name: "substring".into(),
        is_async: false,
        docs: r#"# Str.substring(val: str, start: int = 0, end: int = -1) -> str
Return a new string that is the substring of the given value from a start index up to, but not including an end index. Default start is the beginning of the string and the default end is the entire length of the string.
```rust
const val = "hello, world";
assert_eq(val.substring(), "hello, world");
assert_eq(val.substring(7), "world");
assert_eq(val.substring(3, 8), "lo, w");
```
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Str, default: None },
            Param { name: "start".into(), param_type: Type::Num(NumT::Int), default: Some(Arc::new(Base::Literal(Val::Num(Num::Int(0))))) },
            Param { name: "end".into(), param_type: Type::Num(NumT::Int), default: Some(Arc::new(Base::Literal(Val::Num(Num::Int(-1))))) }
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(SUBSTRING.clone());
            Ok(instructions)
        })
    }
}

/// Is match?
pub fn str_matches() -> LibFunc {
    LibFunc {
        library: STR_LIB.clone(),
        name: "matches".into(),
        is_async: false,
        docs: r#"# Str.matches(val: str, regex: str) -> bool
Return true if this string matches the provided regex string.
```rust
const val = "I categorically deny having triskaidekaphobia.";
const regex = "\\b\\w{13}\\b";
assert(val.matches(regex));
```
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Str, default: None },
            Param { name: "regex".into(), param_type: Type::Str, default: None },
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(IS_MATCH.clone());
            Ok(instructions)
        })
    }
}

/// Find all matches.
pub fn str_find_matches() -> LibFunc {
    LibFunc {
        library: STR_LIB.clone(),
        name: "find_matches".into(),
        is_async: false,
        docs: r#"# Str.find_matches(val: str, regex: str) -> list
Return a list of tuples "(content: str, start: int, end: int)" that represent all matches of the regex in the string vlaue.
```rust
const val = "I categorically deny having triskaidekaphobia.";
const regex = "\\b\\w{13}\\b";
assert_eq(val.find_matches(regex), [("categorically", 2, 15)]);
```
"#.into(),
        params: vector![
            Param { name: "val".into(), param_type: Type::Str, default: None },
            Param { name: "regex".into(), param_type: Type::Str, default: None },
        ],
        return_type: None,
        unbounded_args: false,
        args_to_symbol_table: false,
        func: Arc::new(|_as_ref, _arg_count, _env, _graph| {
            let mut instructions = Instructions::default();
            instructions.push(FIND_ALL.clone());
            Ok(instructions)
        })
    }
}
