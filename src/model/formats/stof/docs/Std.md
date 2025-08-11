# Standard Library (Std)
Functions in the 'Std' library are foundational to Stof and therefore do not requre one to explicitly reference 'Std' when calling. Within the standard library, you'll find functions for asserting values, printing to the console, throwing errors, putting processes to sleep, etc. Note for advanced users that it is possible to extend or modify this library as needed.

## Example Usage
```rust
#[main]
fn main() {
    Std.pln('printing a line');
    pln('printing another line'); // no 'Std' needed for this library
}
```

# Make an assertion
Used in testing and to assert truthiness.

# Make a equal assertion
Used in testing and to assert that two values are equal.

# Make a not equals assertion
Used in testing and to assert that two values are not equal.

# Make a Falsy assertion
Used in testing and to assert that a value is falsy.

# Blobify
Export a portion of this graph as a binary blob in the desired format. The default format is JSON (as UTF-8 bytes).

# Callstack
Return the current callstack as a list of functions (last function is 'this').

# Deep Copy
Copy a value completely.

# Print Debug to Standard Output
Will print N arguments (using a debug format) to the standard output stream.

# Trace Stack
Prints the current stack (debug mode).

# Drop
Drop fields, functions, objects, and data from the graph.

# Print to Error Output
Will print N arguments to the error output stream.

# Exit a process
Immediately terminates this (or another) process. Pass a promise into this function to terminate that process.

# Has Format?
Return true if a given format is available in this graph.

# Format Content Type
Returns the requested format's content type, or null if the format is not available. Ex. assert_eq(format_content_type('json'), 'application/json')

# Formats
Returns a set of all available formats (for parse, stringify, blobify, etc.).

# Functions
Get all functions within this graph, optionally specifying attributes as a filter (single string, or a list/tuple/set of strings).

# Graph ID
Return this graph's unique ID.

# Has Library?
Return true if a given library is available in this graph.

# Libs
Returns a set of all available libraries.

# List Constructor
Create a new list.

# Map Constructor
Create a new map.

# Maximum Value
Return the maximum value for all parameters given (unbounded). If a list or set is provided, this will contemplate the max value in that collection. Will consider units if provided as well.

# Minimum Value
Return the minimum value for all parameters given (unbounded). If a list or set is provided, this will contemplate the min value in that collection. Will consider units if provided as well.

# Nano ID
Generate a new nanoid string (URL safe). Default lenght is 21 characters.

# Parse
Parse additional data into the document, using any format available to the graph (stof, json, images, pdfs, etc.). The default format used is Stof (.stof) and the default context is 'self' (the calling object).

# Print to Standard Output
Will print N arguments to the standard output stream.

# Set Constructor
Create a new set.

# Shallow Drop
Drop fields, functions, objects, and data from the graph. If removing a field and the field points to some data or an object, don't drop that object or additional data (shallow).

# Put Process to Sleep
Instruct this process to sleep for an amount of time. Use time units for specificity (Ex. sleep(200ms)).

# Create a String
Will print N arguments into a string and return it.

# Stringify
Export a portion of this graph as a string in the desired format. The default format is JSON.

# Swap
Swap the memory of two values.

# Throw an error
Used to force an error anywhere inside Stof.

# Trace
Trace this location in the current process.

