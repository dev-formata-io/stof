# Object Library (Obj)
Library that is linked to the 'obj' type.

## Example Usage
```rust
#[main]
fn main() {
    const o = new {};
    assert_eq(Obj.parent(o), self);
    assert_eq(o.parent(), self);
}
```

# Obj.any(obj: obj) -> bool
Returns true if this object has any data attached to it.
```rust
const obj = new { x: 0, y: 0 };
assert(obj.any());
```


# Obj.at(obj: obj, index: int) -> (str, unknown)
Field (name, value) on this object at the given index, or null if the index is out of bounds.
```rust
const obj = new { x: 0, y: 0 };
assert_eq(obj[1], ("y", 0));
```


# Obj.attributes(obj: obj, path: str = null) -> map
Returns a map of attributes, either for this object if the path is null, or for the field/func/obj at the given path.
```rust
assert_eq(self.attributes(), {"a": null}); // if self was defined as a field with the attribute #[a]
```


# Obj.children(obj: obj) -> list
Returns a list containing this objects children.
```rust
const obj = new {};
assert_eq(self.children(), [obj]);
```


# Obj.contains(obj: obj, name: str) -> bool
Return true if this object contains data with the given name.
```rust
const obj = new { x: 0, y: 0 };
assert(obj.contains("y"));
```


# Obj.create_type(obj: obj, typename: str) -> void
Add a typename reference to the graph, pointing to this object. Programmatic version of #[type("typename")] attribute.
```rust
const obj = new { float x: 0, float y: 0 };
obj.create_type("MyType");

const ins = new MyType {};
assert_eq(ins.x, 0);
assert_eq(ins.y, 0);
assert_eq(typename ins, "MyType");
assert_eq(ins.prototype(), obj);
```


# Obj.dbg_graph() -> void
Utility function for dumping the complete graph, helpful for some debugging cases. To dump a specific node, use Std.dbg(..) with the desired object(s).


# Obj.dist(obj: obj, other: obj) -> int
Get the distance between two objects (number of edges that separate them).
```rust
const obj = new { x: 0, y: 0 };
assert_eq(obj.dist(self), 1);
```


# Obj.empty(obj: obj) -> bool
Returns true if this object doesn't have any data attached to it.
```rust
const obj = new { x: 0, y: 0 };
assert_not(obj.empty());
```


# Obj.exists(obj: obj) -> bool
Returns true if this object reference points to an existing object. This is false if the object has been dropped from the document.
```rust
const obj = new {};
assert(obj.exists());
```


# Obj.fields(obj: obj) -> list
Returns a list of fields (tuples with name and value each) on this object.
```rust
const obj = new { x: 0, y: 0 };
assert_eq(obj.fields(), [("x", 0), ("y", 0)]);
```


# Obj.from_id(id: str) -> obj
Create a new object reference from an ID. Objects in Stof are references just like data.
```rust
const obj = new { x: 0, y: 0 };
const ptr = Obj.from_id(obj.id());
assert_eq(ptr, obj);
```


# Obj.from_map(map: map) -> obj
Get the distance between two objects (number of edges that separate them).
```rust
const map = { "x": 0, "y": 0 };
const obj = Obj.from_map(map);
assert_eq(obj.x, 0);
```


# Obj.funcs(obj: obj, attributes: str | list | set = null) -> list
Returns a list of functions on this object, optionally filtering by attributes (str, list of str, set of str, tuple of str).
```rust
// #[myfunc] fn func() {}
assert_eq(self.funcs("myfunc"), [self.func]);
```


# Obj.get(obj: obj, name: str) -> unknown
Get data on this object by name (field value, fn, or data pointer).
```rust
const obj = new { x: 0, y: 0 };
assert_eq(obj.get("x"), 0);
```


# Obj.id(obj: obj) -> str
Return the ID of this object.
```rust
const obj = new {};
assert(obj.id().len() > 0);
```


# Obj.insert(obj: obj, path: str, value: unknown) -> void
Either creates or assigns to a field, just like a normal field assignment, using this object as a starting context.
```rust
const obj = new { x: 0, y: 0 };
obj.insert("z", 9);
assert_eq(obj.z, 9);
```


# Obj.instance_of(obj: obj, proto: str | obj) -> bool
Returns true if this object is an instance of a prototype.
```rust
const obj = new MyType {};
assert(obj.instance_of("MyType"));
```


# Obj.is_parent(obj: obj, other: obj) -> bool
Returns true if this object is a parent of another.
```rust
const obj = new {};
assert(self.is_parent(obj));
```


# Obj.is_root(obj: obj) -> bool
Returns true if this object is a root.
```rust
assert(self.is_root()); // if self is a root
```


# Obj.len(obj: obj) -> int
Number of fields on this object.
```rust
const obj = new { x: 0, y: 0 };
assert_eq(obj.len(), 2);
```


# Obj.move(obj: obj, dest: obj) -> bool
Move this object to a new parent destination. Parent destination cannot be a child of this object (node detachment).
```rust
const obj = new { x: 0, y: 0 };
const other = new {};
obj.move(other);
assert_eq(obj.parent(), other);
```


# Obj.move_field(obj: obj, source: str, dest: str) -> bool
Move or rename a field from a source path/name to a destination path/name (like "mv" in bash), returning true if successfully moved.
```rust
const obj = new { x: 0, y: 0 };
obj.move_field("x", "dude");
assert_eq(obj.dude, 0);
assert_not(obj.x);
```


# Obj.name(obj: obj) -> str
Return the name of this object.
```rust
const obj = new {};
assert(obj.name().len() > 0);
```


# Obj.parent(obj: obj) -> obj
Return the parent of this object, or null if this object is a root.
```rust
const obj = new {};
assert_eq(obj.parent(), self);
```


# Obj.path(obj: obj) -> str
Return the path of this object as a dot '.' separated string.
```rust
assert_eq(self.path(), "root.TestObject"); // if self is "TestObject" and it's parent is "root"
```


# Obj.prototype(obj: obj) -> obj
Returns the prototype object for this object or null if this object doesn't have one.
```rust
assert_not(self.prototype()); // no prototype
```


# Obj.remove(obj: obj, path: str, shallow: bool = false) -> bool
Performs a "drop" operation, just like the Std.drop(..) function, using this object as a starting context. Use this to remove fields, functions, data, etc.

## Shallow
If shallow is true and the path references an object field, drop the field, but don't drop the object from the graph. Default behavior is to drop objects.

```rust
const obj = new { x: 0, y: 0 };
assert(obj.remove("x"));
assert_not(obj.x);
```


# Obj.remove_prototype(obj: obj) -> void
Remove an object's prototype.
```rust
const obj = new MyType {};
obj.remove_prototype();
assert_eq(typename obj, "obj");
```


# Obj.root(obj: obj) -> obj
Returns the root object that contains this object (or self if this object is a root).
```rust
const obj = new {};
assert_eq(obj.root(), self); // if self is a root
```


# Obj.run(obj: obj) -> void
Run an object (like calling a function, but for the entire object as a task). This will execute all fields and functions with a #[run] attribute, optionally with an order #[run(3)]. Any sub objects encountered will also get ran recursively. Arrays act like pipelines, unlocking serious functionality.

## Motivation
This concept enables data-driven abstractions above function calls. An example would be setting some fields on an object that already has some #[run] functions defined, ready to utilize the values in those fields. With prototypes, you can probably see how this is a powerful tool.

### Concrete Example
```rust
#[type]
Request: {
    str name: "europe"

    #[run]
    fn execute() {
        self.result = await Http.fetch("https://myawesomeendpoint/" + self.name);
    }
}

#[main]
fn example() {
    const req = new Request { name: "usa" };
    req.run();
    // now work with req.result as needed
}
```


# Obj.schemafy(schema: obj, target: obj, remove_invalid: bool = false, remove_undefined: bool = false) -> bool
Applies all #[schema] fields from a schema object onto a target object, manipulating the target's fields accordingly and returning true if the target is determined to be valid (matches the schema).

## Use Cases
- filtering & renaming fields as a batch
- validation
- structured transformations (to/from APIs, etc.)
- access control

```rust
schema: {
    #[schema((target_value: str): bool => target_value.len() > 2)]
    first: 'John'

    #[schema(( // pipelines are big AND filters, applied in order and short circuited like &&
        (target_value: unknown): bool => (typeof target_value) == 'str',
        (target_value: str): bool => target_value.contains('Dude'),
    ))]
    last: 'Doe'
}

target: {
    first: 'aj'
    last: 'Dude'
    undefined: 'blah'
}

#[test]
fn schemafy_obj() {
    assert(self.schema.schemafy(self.target, remove_invalid = true, remove_undefined = true));
    assert_eq(str(self.target), "{\"last\":\"Dude\"}");
}
```


# Obj.set_prototype(obj: obj, proto: obj | str) -> void
Set the prototype of this object.
```rust
const proto = new {};
const obj = new {};
obj.set_prototype(proto);
assert_eq(obj.prototype(), proto);
```


# Obj.to_map(obj: obj) -> map
Create a new map out of this object's fields.
```rust
const obj = new { x: 3km, y: 5.5m };
const map = obj.to_map();
assert_eq(map.get("x"), 3km);
```


# Obj.upcast(obj: obj) -> bool
Set the prototype of this object to the prototype of this objects existing prototype.
```rust
const obj = new SubType {};
assert(obj.upcast());
assert_eq(typename obj, "SuperType");
```


