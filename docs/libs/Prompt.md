# Prompt Library (Prompt)
Library for the prompt type, which is a tree of strings that is helpful when working with AI workflows.

## Example Usage
```rust
#[main]
fn main() {
    const p = prompt("hello, world", "msg");
    assert_eq(p as str, "<msg>hello, world</msg>");
}
```

# Prompt.any(prompt: prompt) -> bool
Does this prompt have any sub-prompts?
```rust
const p = prompt('hello', 'greet');
assert(!p.any());
```

# Prompt.at(prompt: prompt, index: int) -> prompt
Get the sub-prompt at a given index (like a list). Will return null if the prompt does not have any sub-prompts.
```rust
const p = prompt('hello', 'greet', prompt('hello there'));
const sub = p[0];
assert_eq(sub, prompt('hello there'));
```

# Prompt.clear(prompt: prompt) -> void
Clear all sub-prompts from this prompt.
```rust
const p = prompt('hello', 'greet');
p.push(', world');
p.clear();
assert_eq(p as str, '<greet>hello</greet>');
```

# Prompt.empty(prompt: prompt) -> bool
Returns true if the prompt does not have any sub-prompts.
```rust
const p = prompt('hello', 'greet');
assert(p.empty());
```

# Prompt.insert(prompt: prompt, index: int, other: prompt) -> void
Insert a sub-prompt into the given index.
```rust
const p = prompt(tag = 'greet');
p.push('hello');
p.insert(1, ', world');
assert_eq(p as str, '<greet>hello, world</greet>');
```

# Prompt.len(prompt: prompt) -> int
The number of sub-prompts contained within this prompt.
```rust
const p = prompt('hello', 'greet', prompt('hello, world'));
assert_eq(p.len(), 1);
```

# Prompt.pop(prompt: prompt) -> prompt
Pop a sub-prompt from the end of the sub-prompt list.
```rust
const p = prompt('hello', 'greet');
p.push(', world');
p.pop();
assert_eq(p as str, '<greet>hello</greet>');
```

# Prompt.prompts(prompt: prompt) -> list
Return this prompts list of sub-prompts.
```rust
const p = prompt('hello', 'greet', prompt('a thing', 'sub'));
assert_eq(p.str(), '<greet>hello<sub>a thing</sub></greet>');
assert_eq(p.prompts(), [prompt('a thing', 'sub')]);
```

# Prompt.push(prompt: prompt, other: prompt | str) -> void
Push a sub-prompt to this prompt.
```rust
const p = prompt('hello', 'greet');
p.push(', world');
assert_eq(p as str, '<greet>hello, world</greet>');
```

# Prompt.remove(prompt: prompt, index: int) -> prompt
Remove a sub-prompt at the given index.
```rust
const p = prompt(tag = 'greet');
p.push('hello');
p.push(', world');
p.remove(1);
assert_eq(p as str, '<greet>hello</greet>');
```

# Prompt.replace(prompt: prompt, index: int, other: prompt) -> void
Replace a sub-prompt at the given index.
```rust
const p = prompt(tag = 'greet');
p.push('hello');
p.replace(0, 'yo');
assert_eq(p as str, '<greet>yo</greet>');
```

# Prompt.reverse(prompt: prompt) -> void
Reverse all sub-prompts in this prompt.
```rust
const p = prompt(tag = 'greet');
p.push(', world');
p.push('hello');
p.reverse();
assert_eq(p as str, '<greet>hello, world</greet>');
```

# Prompt.set_tag(prompt: prompt, tag: str) -> void
Set the tag portion of this prompt. Set to null to clear the tag.
```rust
const p = prompt('hello', 'greet');
p.set_tag('msg');
assert_eq(p.str(), '<msg>hello, world</msg>');
```

# Prompt.set_text(prompt: prompt, text: str) -> void
Set the text portion of this prompt.
```rust
const p = prompt('hello', 'greet');
p.set_text('hello, world');
assert_eq(p.str(), '<greet>hello, world</greet>');
```

# Prompt.str(prompt: prompt) -> str
Convert this prompt into a string, just like a cast to 'str' would do.
```rust
const p = prompt('hello', 'greet');
assert_eq(p.str(), '<greet>hello</greet>');
```

# Prompt.tag(prompt: prompt) -> str
Get the string tag for this prompt, or null if not present.
```rust
const p = prompt('hello', 'greet');
assert_eq(p.tag(), 'greet');
```

# Prompt.text(prompt: prompt) -> str
Get the text portion of this prompt (ignoring any sub-prompts & tag).
```rust
const p = prompt('hello', 'greet');
assert_eq(p.text(), 'hello');
```

