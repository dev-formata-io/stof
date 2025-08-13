# Time Library (Time)
Functions for working with time. Requires the "system" feature flag to be enabled. Includes timestamps (Time.now()) as well as common time formats (like RFC-3339) that are used in APIs and across systems.

## Example Usage
```rust
#[main]
fn main() {
    const now = Time.now(); // default units are ms
    sleep(50ms);
    pln(Time.diff(now) as seconds); // having units is really nice
}
```

# Time.diff(prev: float) -> ms
Convenience function for getting the difference in milliseconds between a previous timestamp (takes any units, default ms) and the current time. Shorthand for (Time.now() - prev).
```rust
const ts = Time.now();
sleep(50ms);
const diff = Time.diff(ts);
assert(diff >= 50ms);
```


# Time.diff_ns(prev: float) -> ns
Convenience function for getting the difference in nanoseconds between a previous timestamp (takes any units, default ns) and the current time. Shorthand for (Time.now_ns() - prev).
```rust
const ts = Time.now_ns();
sleep(50ms);
const diff = Time.diff_ns(ts);
assert(diff >= 50ms);
```


# Time.from_rfc2822(time: str) -> ms
Returns a unix timestamp (milliseconds since Epoch) representing the given RFC-2822 string.
```rust
const ts = Time.from_rfc2822("Wed, 13 Aug 2025 16:24:12 +0000");
assert(ts < Time.now());
```


# Time.from_rfc3339(time: str) -> ms
Returns a unix timestamp (milliseconds since Epoch) representing the given RFC-3339 string.
```rust
const ts = Time.from_rfc3339("2025-08-13T16:22:43.028375200+00:00");
assert(ts < Time.now());
```


# Time.now() -> ms
Return the current time in milliseconds since the Unix Epoch (unix timestamp).
```rust
const ts = Time.now();
assert(Time.now() >= ts);
```


# Time.now_ns() -> ns
Return the current time in nanoseconds since the Unix Epoch (unix timestamp).
```rust
const ts = Time.now_ns();
assert(Time.now_ns() >= ts);
```


# Time.now_rfc2822() -> str
Returns a string representing the current time according to the RFC-2822 specefication.
```rust
const now = Time.now_rfc2822();
pln(now); // "Wed, 13 Aug 2025 16:24:12 +0000" when these docs were written
```


# Time.now_rfc3339() -> str
Returns a string representing the current time according to the RFC-3339 specefication.
```rust
const now = Time.now_rfc3339();
pln(now); // "2025-08-13T16:22:43.028375200+00:00" when these docs were written
```


# Time.sleep(time: float = 1000ms) -> void
Alias for Std.sleep, instructing this process to sleep for a given amount of time (default units are milliseconds).
```rust
const ts = Time.now();
Time.sleep(50ms); // units make life better here
const diff = Time.diff(ts);
assert(diff >= 50ms);
```


# Time.to_rfc2822(time: float) -> str
Returns a string representing the given timestamp according to the RFC-2822 specefication.
```rust
const now = Time.to_rfc2822(Time.now());
pln(now); // "Wed, 13 Aug 2025 16:24:12 +0000" when these docs were written
```


# Time.to_rfc3339(time: float) -> str
Returns a string representing the given timestamp according to the RFC-3339 specefication.
```rust
const now = Time.to_rfc3339(Time.now());
pln(now); // "2025-08-13T16:22:43.028375200+00:00" when these docs were written
```


