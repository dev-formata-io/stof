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

# Any?
Returns true if this object has at least one datum (fields, funcs, etc.)

# At (index operator)
Returns the field (tuple of name and value) on this object at the given index or null if out of bounds.

# Attributes.
Returns a map of attributes, either for this node or a field/func/obj at a given string path.

# Children
Returns a list of child objects on this object.

# Contains?
Returns true if this object contains some data with the given name.

# Create Type
Create a type from this object and a type name.

# Dump Graph
Debugging utility for dumping a complete graph. For a specific node, use dbg() on that object.

# Object Distance.
Returns the distance between two objects in the same graph.

# Empty?
Returns true if this object doesn't have any data (fields, funcs, etc.)

# Exists?
Returns true if this object exists in the graph (objects are just pointers into a graph).

# Fields
Returns a list of fields (tuple with name and value) on this object.

# Object from ID
Create an object reference from a string ID.

# Object from Map
Create a new object from a Map, using string keys to create fields on the new object.

# Functions
Returns a list of functions on this object, optionally filtering by attributes (string or list/tuple/set of strings).

# Get
Returns data on this object by name (field, fn, or opaque data reference).

# ID
Returns the id of an object as a string.

# Insert
Performs a 'set variable' instruction just like a normal field assignment, using this object as a starting context.

# Is an Instance of a Prototype?
Returns true if this object is an instance of a prototype.

# Is Parent?
Returns true if this object is a parent of another.

# Is Root?
Returns true if this object is a root object.

# Length (Number of Fields)
Returns the number of fields on this object.

# Move Object.
Move this object to a new parent. Destination/new parent must not be a child of this node (will return false and not be moved). Returns true if successfully moved.

# Move (or Rename) Field
Moves a field from a source path/name to a destination path/name. Returns true if successful.

# Name
Returns the name of an object as a string.

# Parent
Returns the parent object of a given object or null if the object is a root.

# Path
Returns the path of an object as a dot separated string of object names.

# Prototype
Returns the prototype of this object or null if a prototype doesn't exist.

# Remove
Performs a 'drop' just like the Std function, using this object as a starting context.

# Remove Prototype
Remove the prototype of this object.

# Root
Returns the root object that contains this object (or self if this object is a root).

# Run Object
Calls all #[run] functions with an optional order on this object, also going into fields, running sub-objects, etc.

# Schemafy
Applies all #[schema] fields from a schema object onto a target object, returning true if the target is determined to be valid according to the schema.

# Set Prototype
Set the prototype of this object to either another object or a typename.

# Map from Object Fields
Create a new map out of object fields.

# Upcast
Set the prototype of this object to the prototype of this objects prototype.

