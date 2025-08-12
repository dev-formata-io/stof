# Data Library (Data)
Library for working with opaque data pointers. If referenced explicitely, will work with custom data also.

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

# Attach To
Attach this data to an additional object.

# Drop
Remove data completely from the graph.

# Drop From
Remove data from a node in the graph (object). If this node is the only object referencing the data, the data will be removed completely from the graph.

# Data Exists?
Returns true if this data reference points to valid data in a graph.

# From Field Path
Create a data reference from a dot '.' separated field path.

# From ID
Create a data reference from a string ID.

# Data Id
String ID for this data reference.

# Data Library Name
The 'tagname' for this data reference. If the data points to a function, this will return 'Fn' for example. For custom data, like a PDF, this would return 'Pdf'.

# Move
Drop this data from an object and move it to another object.

# Data Objects
List of objects that this data is attached to.

