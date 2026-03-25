# Formats & Patterns Reference

Detailed reference for format I/O, imports, rich data components, `#[run]` workflows, and common idioms in Stof.

---

## Supported Formats

| Format | Full fidelity | Use case |
|--------|:---:|---------|
| `stof:human` | Yes | Readable roundtrip with functions, attributes, prototypes |
| `bstf` | Yes | Binary transfer (includes binary data components) |
| `json` | No | External system interop — data only, no functions |
| `toml` | No | Configuration files |
| `yaml` | No | Configuration files |
| `text` / `md` | No | Plain text / Markdown content |

**Full fidelity** means the format preserves functions, attributes, prototypes, and binary data. Non-fidelity formats export data only.

---

## Import

```stof
// Import Stof files
import './config.stof' as self.Config
import './other.stof' as self.Other

// Import JSON/YAML/TOML — parsed into objects
import './data.json' as self.Data
import './config.yaml' as self.Config
import './settings.toml' as self.Settings

// Import binary/rich data — creates Data<T> components
import './file.pdf'                 // → Data<Pdf> component on current object
import './image.png'                // → Data<Image> component
import './document.pdf' as self.Doc // → Data<Pdf> on self.Doc

// Import from URL
import 'https://example.com/data.json' as self.Remote
```

### Import Behavior

- `import` merges the imported content into the target object
- If the target already has fields with the same names, they are overwritten
- `as self.Path` controls where the imported data lands in the document graph
- Without `as`, content is imported into the current object
- Rich data (PDFs, images) become `Data<Type>` components accessible via the Data library

---

## Serialization & Deserialization

```stof
// Serialize to string
stringify('json', obj)            // JSON string
stringify('yaml', obj)            // YAML string
stringify('toml', obj)            // TOML string
stringify('stof:human', obj)      // Full fidelity Stof string

// Serialize to binary
blobify('bstf', obj)              // Binary Stof blob

// Deserialize from string
parse(json_str, dest, 'json')     // Parse JSON into dest object
parse(yaml_str, dest, 'yaml')     // Parse YAML into dest
parse(stof_str, dest, 'stof')     // Parse Stof into dest

// Roundtrip example
const serialized = stringify('stof:human', self.data);
const restored = new {};
parse(serialized, restored, 'stof');
```

---

## Rich Data Components

Stof documents can contain binary data components alongside fields and functions:

```stof
// Import creates data components
import './diagram.pdf'              // Data<Pdf>
import './photo.png'                // Data<Image>

// Access via Data library
const pdf_data = Data.get(self, 'Pdf');
const img_data = Data.get(self, 'Image');

// Set data programmatically
Data.set(self, 'Pdf', pdf_blob);
```

Data components travel with objects — when you serialize with `bstf` (binary Stof), all data components are included.

---

## Export Control

```stof
// Exclude from export/stringify
#[no-export]
internal_config: {
    secret_key: 'abc123'
}

// Only this object is excluded — children can still be exported individually
```

---

## `#[run]` Workflow Pattern

The `#[run]` attribute creates ordered, composable workflow pipelines within objects.

### Basic Workflow

```stof
workflow: {
    #[run(1)]
    step_one: {
        #[run]
        fn execute() { pln("step 1: initialize") }
    }
    #[run(2)]
    step_two: {
        #[run]
        fn execute() { pln("step 2: process") }
    }
    #[run(3)]
    step_three: {
        #[run]
        fn execute() { pln("step 3: finalize") }
    }
}

#[main]
fn main() {
    self.workflow.run()   // executes steps in order: 1, 2, 3
}
```

### Run with Arguments

```stof
#[run({'args': [42]})]
fn step(value: int) {
    pln(`Processing: ${value}`)
}
```

### Prototype Run Behavior

Control whether prototype `#[run]` functions execute on instances:

```stof
#[run({'prototype': 'none'})]    // skip prototype's run functions
#[run({'prototype': 'first'})]   // run prototype's functions first
#[run({'prototype': 'last'})]    // run prototype's functions last
```

---

## Common Patterns

### Guard-and-Return

```stof
fn process() {
    const policy = self.get();
    if (policy == null) return;
    // ... continue with valid policy
}
```

### Dynamic Object Insertion with Validation

```stof
fn safe_insert(stof_str: str) -> obj {
    const item = new {} on parent;
    try {
        parse(stof_str, item, 'stof');
    } catch {
        drop(item);
        return null;
    }
    item
}
```

### Attribute-Driven Event Dispatch

```stof
// Find and call all functions with a specific attribute
fn dispatch(key: str, value: unknown) {
    for (const func in funcs(attributes = key)) {
        func(value);
    }
}
```

### Traverse Parent Chain

```stof
fn find_ancestor(child: obj, target_type: str) -> obj {
    let parent = child.parent();
    loop {
        if (parent == null || parent.instance_of(target_type)) break;
        parent = parent.parent();
    }
    parent
}
```

### HTTP Fetch and Parse

```stof
fn fetch_json(url: str) -> obj {
    const resp = await Http.fetch(url);
    if (!Http.success(resp)) throw(`HTTP error`);
    const data = new {};
    Http.parse(resp, data);
    data
}
```

### Configuration with Validation

```stof
#[type]
AppConfig: {
    str! name: '';
    #[schema((v) => v > 0 && v < 65536)]
    int! port: 8080;
    str! env: 'development';

    fn is_production() -> bool { self.env == 'production' }
}

// Validate external config
const raw = new {};
parse(config_json, raw, 'json');
<AppConfig>.schemafy(raw);   // throws if invalid
```
