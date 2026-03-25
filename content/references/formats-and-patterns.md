# Stof Formats, Interop & Patterns

Detailed reference for format I/O, imports, rich data components, the `#[run]` workflow pattern, and common idioms.

---

## Formats & Interop

Stof's underlying structure is an **entity-component system over a DAG**: nodes (objects) are entities, and data components (fields, functions, images, PDFs, etc.) are the components attached to them. Formats and libraries are just different lenses on that same flat structure.

**Format I/O is potentially lossy.** Round-trip fidelity depends on the format:

| Format | Functions | Attributes | Unit types | Prototypes | Binary data |
|--------|-----------|------------|------------|------------|-------------|
| `stof` / `stof:human` | Yes | Yes | Yes | Yes | No |
| `bstf` | Yes | Yes | Yes | Yes | Yes |
| `json` | No | No | No | No | No |
| `toml` | No | No | No | No | No |
| `yaml` | No | No | No | No | No |
| `text` / `md` | No | No | No | No | No |
| `urlencoded` | No | No | No | No | No |

Use `stof:human` or `bstf` for full fidelity. Use `json`/`toml`/`yaml` for interop with external systems.

### Serialization

```stof
// stringify — serialize an object to a string
stringify('json', obj)          // → JSON string
stringify('stof:human', obj)    // → human-readable Stof (preserves all)
stringify('md', obj)            // → Markdown (reads obj.md field)
stringify('urlencoded', obj)    // → URL-encoded form data

// blobify — serialize to binary blob
blobify('bstf', obj)            // fully roundtrips types, functions, attributes

// parse — deserialize into an existing object
parse(str_or_blob, dest, 'json')
parse(str_or_blob, dest, 'stof')
parse(stof_string)              // parse into current context
```

**Format notes:**
- `'stof'` vs `'stof:human'` — compact omits whitespace; `stof:human` is readable with full fidelity
- `'bstf'` — binary Stof for over-the-wire transfer
- `'bytes'` — `obj.bytes` field must be a `blob`
- `'urlencoded'` / `'www-form'` — nested objects encode as bracket notation (`sub[val]=42`)

### Imports

```stof
// Path-only (format inferred from extension)
import './config.stof' as self.Config
import './data.json' as self.Data
import './test.json'               // parsed into self directly

// Explicit format override
import json './test.json' as self.Imported
import text './test.json' as self.Raw   // imports raw text into dest.text
import pkg './path'                    // import a package (@ prefix = stof/ directory)

// Binary / rich formats — loaded as data components
import './file.pdf'                 // → self.pdf as Data<Pdf>
import './image.png'                // → self.image as Data<Image>

// Runtime parse
parse(str_or_blob, target_obj, 'stof')
```

### Rich data components (`Data<Lib>`)

When a format loads binary data, the result is a `data` value typed as `Data<LibName>`:

```stof
if (lib('Pdf')) {
    assert_eq(typename self.pdf, 'Data<Pdf>');
    const text = self.pdf.extract_text();
    const images = self.pdf.extract_images();
}

if (lib('Image')) {
    const clone = copy(self.image);
    clone.width()               // → 1200
    clone.resize(500, 500)
    clone.bmp()                 // → blob
    const img = Image.from_blob(bmp_blob);
}

// Cast to a typed data handle
let dta: data = self.pdf as Data<Pdf>;
assert_eq(Data.libname(dta), 'Pdf');
```

### Stof export/import notes

- `stringify('stof:human', obj)` + `parse(str, dest, 'stof')` fully roundtrips including functions and prototype definitions
- Attributes that hold function values (e.g. `#[schema(...)]`) roundtrip correctly
- **Avoid export/import within the same graph** — IDs may collide. Use `copy(obj)` instead
- Package imports: `import pkg './@geo'` resolves `@` to the `stof/` directory

---

## `#[run]` Workflow Pattern

```stof
workflow: {
    #[run(1)]
    step_one: {
        #[run]
        fn execute() { pln("step 1") }
    }

    #[run(2)]
    step_two: {
        #[run]
        fn execute() { pln("step 2") }
    }

    // Run with arguments
    #[run({'args': [42]})]
    fn setup(val: int) { self.configured = val; }

    // Run on lists — each element is run
    #[run]
    list_run: [
        () => { self.done = true; },
        { #[run] fn inner() { super.sub_done = true; } }
    ]
}

#[main]
fn main() { self.workflow.run() }
```

---

## Common Idioms & Patterns

**Guard-and-return:**
```stof
const policy = self.get();
if (policy == null) return;
```

**Null-safe dynamic dispatch:**
```stof
?self.get().valid() ?? false
?App.event_handler(key, val)
```

**Dynamic object insertion:**
```stof
const item = new {} on parent_obj;
try {
    parse(stof_str, item, 'stof');
} catch {
    drop(item);
    return null;
}
const id = item.id ?? nanoid();
item.id = id;
parent_obj.remove(id, shallow = false);
parent_obj.insert(id, item as MyType);
```

**Attribute-driven event dispatch:**
```stof
for (const func in funcs(attributes = key)) {
    const count = func.params().len();
    if (count == 1) func(value);
    else if (count < 1) func();
}

#[my-event]
fn on_event(value: obj) { ... }
```

**Traverse parent chain to find a typed ancestor:**
```stof
fn get_root_policy(child: obj) -> Policy {
    if (child.instance_of('Policy')) return child;
    let parent = child.parent();
    loop {
        if (parent == null) break;
        if (parent.instance_of('Policy')) return parent;
        parent = parent.parent();
    }
    null
}
```

**send_tmp pattern (fire event then drop temp object):**
```stof
await <Event>.send_tmp('plan-changed', new {
    previous: prev_plan,
    current: new_plan,
});
```
