# Number Library (Num)
Library for manipulating and using numbers, automatically linked to the number types (int, float, & units).

## Example Usage
```rust
#[main]
fn main() {
    assert_eq(Num.abs(-23), 23);

    const val = -5;
    assert_eq(val.abs(), 5);
}
```

# Num.abs(val: int | float) -> int | float
Return the absolute value of the given number.
```rust
const v = -2;
assert_eq(v.abs(), 2);
```

# Num.acos(val: int | float) -> rad
Arc Cosine function (returns a float with radian units).


# Num.acosh(val: int | float) -> float
Inverse hyperbolic Cosine function.


# Num.asin(val: int | float) -> rad
Arc Sine function (returns a float with radian units).


# Num.asinh(val: int | float) -> float
Inverse hyperbolic Sine function.


# Num.at(val: int | float, index: int) -> int
Index into this number (helpful for iteration of single value ranges).
```rust
assert_eq((10).at(5), 5);
assert_eq((10).at(20), 10);
```


# Num.atan(val: int | float) -> rad
Arc Tangent function (returns a float with radian units).


# Num.atan2(y: int | float, x: int | float) -> rad
Computes the four quadrant arctangent of self (y) and other (x) in radians.
```rust
assert_eq((Num.atan2(1, 2) as deg).round(), 27deg);
```


# Num.atanh(val: int | float) -> float
Inverse hyperbolic Tangent function.


# Num.bin(val: int) -> str
Returns this number represented as a binary string.
```rust
assert_eq((10).bin(), "1010");
```


# Num.cbrt(val: int | float) -> float
Return the cube root of a number.
```rust
const v = 8;
assert_eq(v.cbrt(), 2);
```

# Num.ceil(val: int | float) -> int | float
Return the smallest integer greater than or equal to the given value.
```rust
const v = 2.4;
assert_eq(v.ceil(), 3);
```

# Num.cos(val: int | float) -> float
Cosine function.


# Num.cosh(val: int | float) -> float
Hyperbolic Cosine function.


# Num.exp(val: int | float) -> float
Exponential function (e^(val)).
```rust
assert_eq((1).exp().round(3), 2.718);
```

# Num.exp2(val: int | float) -> float
Exponential 2 function (2^(val)).
```rust
assert_eq((2).exp2(), 4);
```

# Num.floor(val: int | float) -> int | float
Return the largest integer less than or equal to the given value.
```rust
const v = 2.4;
assert_eq(v.floor(), 2);
```

# Num.fract(val: int | float) -> int | float
Return the fractional part of this number.
```rust
const v = 2.4;
assert_eq(v.trunc(), 0.4);
```

# Num.has_units(val: int | float) -> bool
Returns true if the given number has units.
```rust
const val = 10kg;
assert(val.has_units());
```


# Num.hex(val: int) -> str
Returns this number represented as a hexidecimal string.
```rust
assert_eq((10).hex(), "A");
```


# Num.inf(val: int | float) -> bool
Return true if this value is infinity.
```rust
assert_not((14).inf());
```

# Num.is_angle(val: int | float) -> bool
Returns true if the given number has angular units (degrees or radians).
```rust
const val = 10deg;
assert(val.is_angle());
```


# Num.is_length(val: int | float) -> bool
Returns true if the given number has length units.
```rust
const val = 10m;
assert(val.is_length());
```


# Num.is_mass(val: int | float) -> bool
Returns true if the given number has units of mass.
```rust
const val = 10kg;
assert(val.is_mass());
```


# Num.is_temp(val: int | float) -> bool
Returns true if the given number has temperature units.
```rust
const val = 10F;
assert(val.is_temp());
```


# Num.is_time(val: int | float) -> bool
Returns true if the given number has units of time.
```rust
const val = 10s;
assert(val.is_time());
```


# Num.len(val: int | float) -> int
Length of this number (helpful for iteration).
```rust
assert_eq((10).len(), 10);
```


# Num.ln(val: int | float) -> float
Natural log.
```rust
assert_eq((1).ln(), 0);
```

# Num.log(val: int | float, base: int | float = 10) -> float
Log function with a given base value.
```rust
assert_eq((2).log().round(3), 0.301);
```


# Num.max(..) -> unknown
Return the maximum value of all given arguments. If the argument is a collection, this will get the maximum value within that collection for comparison with the others. Will consider units if provided as well.
```rust
assert_eq(Num.max(12, 23, 10, 42, 0), 42);
```


# Num.min(..) -> unknown
Return the minimum value of all given arguments. If the argument is a collection, this will get the minimum value within that collection for comparison with the others. Will consider units if provided as well.
```rust
assert_eq(Num.min(12, 23, 10, 42, 0), 0);
```


# Num.nan(val: int | float) -> bool
Return true if this value is NaN.
```rust
assert_not((14).nan());
```

# Num.oct(val: int) -> str
Returns this number represented as an octal string.
```rust
assert_eq((10).oct(), "12");
```


# Num.pow(val: int | float, to: int | float = 2) -> float
Returns the given value raised to the given power.
```rust
const val = 10;
assert_eq(val.pow(to = 2), 100);
assert_eq(val.pow(), 100);
```


# Num.remove_units(val: int | float) -> int | float
Removes the units (if any) on this number.
```rust
const val = 10kg;
assert_eq(typeof val.remove_units(), "float");
```


# Num.round(val: int | float, places: int = 0) -> int | float
Round the given number to the given number of places. If value is an integer, do nothing.
```rust
const val = 10.348;
assert_eq(val.round(2), 10.35);
assert_eq(val.round(), 10);
```


# Num.signum(val: int | float) -> int | float
Return a number representing the sign of this value (-1 or 1).
```rust
assert_eq((42).signum(), 1);
assert_eq((-42).signum(), -1);
```

# Num.sin(val: int | float) -> float
Sine function.


# Num.sinh(val: int | float) -> float
Hyperbolic Sine function.


# Num.sqrt(val: int | float) -> float
Return the square root of a number.
```rust
const v = 4;
assert_eq(v.sqrt(), 2);
```

# Num.tan(val: int | float) -> float
Tangent function.


# Num.tanh(val: int | float) -> float
Hyperbolic Tangent function.


# Num.to_string(val: int | float) -> str
Returns this number represented as a string (like print).
```rust
assert_eq((10).to_string(), "10");
assert_eq(str(10), "10"); // prefer Std.str(..)
```


# Num.trunc(val: int | float) -> int | float
Return the integer part of the given value.
```rust
const v = 2.4;
assert_eq(v.trunc(), 2);
```

