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

# Fn.attributes(func: fn) -> map
Get a map of attributes (name & value) that this function has, if any.
```rust
const func: fn = self.hi; // #[hi] fn hi() {}
assert_eq(func.attributes(), {"hi": null});
```

# Fn.call(func: fn, ..) -> unknown
Call this function, using any arguments given after the function itself (some library functions can take N arguments, this is one of them).
```rust
const func: fn = (name: str):str => "Hi, " + name;
assert_eq(func.call("Bob"), "Hi, Bob");
```

# Fn.call_expanded(func: fn, ..) -> unknown
Call this function, using any arguments given after the function itself. However, if an argument is a collection (ex. list), expand the list values out as arguments themselves.
```rust
const func: fn = (name: str):str => "Hi, " + name;
assert_eq(func.call_expanded(["Bob"]), "Hi, Bob");
```

# Fn.data(func: fn) -> data
Get the data pointer for this function.
```rust
const func: fn = self.hi;
assert(func.data().exists());
```

# Fn.has_attribute(func: fn, name: str) -> bool
Returns true if the given function has an attribute with the given name.
```rust
const func: fn = self.hi; // #[hi] fn hi() {}
assert(func.has_attribute("hi"));
```

# Fn.id(func: fn) -> str
Get the data ID for this function (shorthand for "func.data().id()").
```rust
const func: fn = self.hi;
assert_eq(func.id(), func.data().id());
```

# Fn.is_async(func: fn) -> bool
Is this function async? This is just shorthand for checking if an "async" attribute exists (what makes a func async).
```rust
const func: fn = self.hi; // async fn hi() {}
assert(func.is_async());
```

# Fn.name(func: fn) -> str
Get the name of this function.
```rust
const func: fn = self.hi; // fn hi() {}
assert_eq(func.name(), "hi");
```

# Fn.obj(func: fn) -> obj
Get the first object found that references this function.
```rust
const func: fn = self.hi;
assert_eq(func.obj(), self);
```

# Fn.objs(func: fn) -> list
Get a list of all objects that this function is attached to.
```rust
const func: fn = self.hi;
assert_eq(func.objs(), [self]);
```

# Fn.params(func: fn) -> list
Get a list of expected parameters for this function (tuple containing the name and type).
```rust
const func: fn = self.hi; // fn hi(a: int) {}
assert_eq(func.params(), [("a", "int")]);
```

# Fn.return_type(func: fn) -> str
Get the return type for the given function.
```rust
const func: fn = self.hi; // fn hi() -> int { 42 }
assert_eq(func.return_type(), "int");
```

