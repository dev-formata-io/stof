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

# Absolute Value
Return the absolute value of a number.

# Arc Cosine
ACosine function (returns Radians).

# Inverse Hyperbolic Cosine
ACosH function.

# Arc Sine
ASine function (return Radians).

# Inverse Hyperbolic Sine
ASinH function.

# Index of Number
Used when iterating over the integers up to a number (single value range). For example, if the number is 45.at(5), the result will be 5. Or if the number is 45.at(50), the result will be 45.

# Arc Tangent
ATangent function (return Radians).

# ATan2 Function
ATan2 function.

# Inverse Hyperbolic Tangent
ATanH function.

# Binary String
Return this numbers binary string (integer).

# Cube Root
Return the cube root of a number.

# Ceil
Return the smallest integer greater than or equal to self.

# Cosine
Cosine function.

# Hyperbolic Cosine
CosH function.

# Exponential Function
e^(self).

# Exponential 2
2^(self).

# Floor
Return the largest integer less than or equal to self.

# Fract
Return the fractional part of self.

# Has Units?
Return true if this number has units.

# Hex String
Return this numbers hex string (integer).

# Infinity?
Returns true if this number is infinity.

# Is Angle?
Return true if this number has angle units.

# Is Length?
Return true if this number has units of length.

# Is Mass?
Return true if this number has units of mass.

# Is Temperature?
Return true if this number has temperature units.

# Is Time?
Return true if this number has units of time.

# Length of a Number
Used when iterating over the integers up to a number (single value range).

# Natural Log
ln(self).

# Log
Log function with a base (default of 10).

# Maximum Value
Return the maximum value for all parameters given (unbounded). If a list or set is provided, this will contemplate the max value in that collection. Will consider units if provided as well.

# Minimum Value
Return the minimum value for all parameters given (unbounded). If a list or set is provided, this will contemplate the min value in that collection. Will consider units if provided as well.

# Not a Number?
Returns true if this is value is NaN.

# Oct String
Return this numbers octal string (integer).

# Power
Raise this number to a power (default is to square).

# Remove Units
Remove units if this number has any.

# Round
Round this number, optionally specifying the number of places to be rounded to.

# Sign Number
Return the sign number of self (-1 or 1).

# Sine
Sine function.

# Hyperbolic Sine
SinH function.

# Square Root
Return the square root of a number.

# Tangent
Tangent function.

# Hyperbolic Tangent
TanH function.

# To String
Return this number as a string (like print).

# Trunc
Return the integer part of self.

