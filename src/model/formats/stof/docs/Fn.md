# Function Library (Fn)
Library for working with and calling functions, linked to the 'fn' type.

## Example Usage
```rust
#[main]
fn main() {
    const f = ():str => 'hello';
    assert_eq(Fn.call(f), 'hello');
    assert_eq(f.call(), 'hello');
    assert_eq(f(), 'hello');
}
```

# Attributes
Returns a map of this functions attributes (with values).

# Call Function
Will call this function with the provided arguments.

# Expanded Call Function
Will call this function but will expand the arguments out if they are containers. For example, providing a list of values here will result in each individual list value as a separate function argument.

# Data
Converts a function reference into a generic data reference.

# Has Attribute?
Returns true if this function has an attribute with the requested name.

# ID
Returns the ID of a function reference.

# Is Async?
Returns true if this function is an async function (has an 'async' attribute).

# Name
Returns the name of a function.

# Object
Returns the first object reference found that this function is attached to.

# Objects
Returns a list of object references that this function is attached to.

# Parameters
Returns a list of tuples containing the name and type of each expected parameter.

# Return Type
Returns a string (typeof) for this functions return type.

