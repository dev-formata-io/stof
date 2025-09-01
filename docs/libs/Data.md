# Data Library (Data)
Library for working with opaque data pointers. If referenced explicitely, will work with custom data also, like PDFs, Images, etc.

## Example Usage
```rust
fn hi() -> str { 'hi' }

#[test]
fn main() {
    const o = new {};
    
    const func = self.hi;
    const dta = func.data();
    dta.attach(o);
    
    assert_eq(o.hi(), 'hi');
}
```

# Data.attach(ptr: data, obj: obj) -> bool
Attach this data to an additional object. This data will now be accessible using the same name from the object.
```rust
const func: fn = self.hi;
const other = new {};
assert(func.data().attach(other));
assert_eq(other.hi, func);
```

# Data.blob(ptr: data) -> blob
Uses bincode serialization to serialize the data (name, attributes, value, etc.), turning it into a blob.
```rust
const func: fn = self.hi;
const bin = func.data().blob(); // entire function as a blob
```

# Data.drop(ptr: data) -> bool
Drop this data from the document, returning true if the data existed and was removed.
```rust
const func: fn = self.hi;
assert(func.data().drop());
```

# Data.drop_from(ptr: data, obj: obj) -> bool
Drop this data from a specific object. If the given object was the only reference, the data will be dropped completely from the document.
```rust
const func: fn = self.hi;
assert(func.data().drop_from(self));
```

# Data.exists(ptr: data) -> bool
Does this data pointer point to existing data? Will be false if the data has been dropped from the document.
```rust
const func: fn = self.hi;
const ptr = func.data();
drop(func);
assert_not(ptr.exists());
```

# Data.field(path: str) -> data
Create a data pointer to a field, using a path/name from the current object context.
```rust
const ptr = Data.field('myfield'); // self.myfield
assert(ptr.exists());
```

# Data.from_id(id: str) -> data
Create a new data pointer with a string ID.
```rust
const func: fn = self.hi;
const id = func.data().id();
assert_eq(Data.from_id(id), func.data());
```

# Data.id(ptr: data) -> str
Get the id for this data pointer, which can be used to later construct another reference.
```rust
const func: fn = self.hi;
const id = func.data().id();
assert_eq(Data.from_id(id), func.data());
```

# Data.libname(ptr: data) -> str
Get the library name for this data pointer, if applicable.
```rust
const func: fn = self.hi;
assert_eq(func.data().libname(), "Fn");
```

# Data.load_blob(bytes: blob, context: obj | str = self) -> data
Uses bincode to deserialize the data blob (name, attributes, value, etc.), adding it to the desired context object.
```rust
const func: fn = self.hi;
const bin = func.data().blob(); // entire function as a blob

const other = new {};
const dref = Data.load_blob(bin, other); // copy of the function is now on "other"
```

# Data.move(ptr: data, from: obj, to: obj) -> bool
Combines a drop and attach, removing this data from an object and placing it on another.
```rust
const func: fn = self.hi;
const other = new {};
assert(func.data().move(self, other));
assert_not(self.hi); // func is now on other
```

# Data.objs(ptr: data) -> list
List of objects that this data is attached to (will always have at least one).
```rust
const func: fn = self.hi;
assert_eq(func.data().objs().front(), self);
```

