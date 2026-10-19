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

# Time.add_days(ts: ms, n: int) -> ms
Adds n calendar days to the given UTC timestamp. n may be negative.
```rust
const tomorrow = Time.add_days(Time.now(), 1);
assert(tomorrow > Time.now());
const yesterday = Time.add_days(Time.now(), -1);
assert(yesterday < Time.now());
```


# Time.add_months(ts: ms, n: int) -> ms
Adds n calendar months to the given UTC timestamp.
The day of month is clamped to the last day of the target month if necessary
(e.g. Jan 31 + 1 month = Feb 28/29).
```rust
const next = Time.add_months(Time.now(), 1);
assert(next > Time.now());
```


# Time.day_of_month(ts: ms) -> int
Returns the day of the month (1–31) for the given UTC timestamp.
```rust
const dom = Time.day_of_month(Time.now());
assert(dom >= 1 && dom <= 31);
```


# Time.day_of_week(ts: ms) -> int
Returns the ISO day of the week for the given UTC timestamp.
0 = Monday, 1 = Tuesday, ..., 6 = Sunday.
```rust
const dow = Time.day_of_week(Time.now());
assert(dow >= 0 && dow <= 6);
```


# Time.days_in_month(ts: ms) -> int
Returns the number of days in the month containing the given UTC timestamp.
Correctly handles leap years for February.
```rust
const dim = Time.days_in_month(Time.now());
assert(dim >= 28 && dim <= 31);
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


# Time.hour(ts: ms) -> int
Returns the UTC hour (0–23) for the given timestamp.
```rust
const h = Time.hour(Time.now());
assert(h >= 0 && h <= 23);
```


# Time.minute(ts: ms) -> int
Returns the UTC minute (0–59) for the given timestamp.
```rust
const m = Time.minute(Time.now());
assert(m >= 0 && m <= 59);
```


# Time.month(ts: ms) -> int
Returns the UTC month (1–12) for the given timestamp.
```rust
const m = Time.month(Time.now());
assert(m >= 1 && m <= 12);
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


# Time.second(ts: ms) -> int
Returns the UTC second (0–59) for the given timestamp.
```rust
const s = Time.second(Time.now());
assert(s >= 0 && s <= 59);
```


# Time.sleep(time: float = 1000ms) -> void
Alias for Std.sleep, instructing this process to sleep for a given amount of time (default units are milliseconds).
```rust
const ts = Time.now();
Time.sleep(50ms); // units make life better here
const diff = Time.diff(ts);
assert(diff >= 50ms);
```


# Time.start_of_day(ts: ms) -> ms
Returns the UTC midnight timestamp for the day containing the given timestamp.
```rust
const today = Time.start_of_day(Time.now());
assert(today <= Time.now());
```


# Time.start_of_month(ts: ms) -> ms
Returns the UTC midnight timestamp for the first day of the month containing the given timestamp.
```rust
const som = Time.start_of_month(Time.now());
assert(som <= Time.now());
assert(Time.day_of_month(som) == 1);
```


# Time.start_of_period(ts: ms, schedule: str) -> ms
Returns the start of the current billing/reset period for the given UTC timestamp
and schedule expression. Returns null if the schedule is invalid.

Schedule formats:
  "monthly:N"             — Nth day of every month (1–31, clamped to month length)
  "monthly:last"          — Last day of every month
  "weekly:mon"            — Every Monday (mon|tue|wed|thu|fri|sat|sun)
  "nth_weekday:N:mon"     — Nth occurrence of weekday in the month (N = 1–4)
  "yearly:M-D"            — Month M, day D of every year (e.g. "yearly:1-1" for Jan 1)
  "quarterly:D"           — Day D of the first month of each quarter (Jan/Apr/Jul/Oct)

All times are UTC. The returned timestamp is always midnight UTC on the period start day.

```rust
// Resets on the 1st of every month
const period = Time.start_of_period(Time.now(), 'monthly:1');
assert(period <= Time.now());

// Resets every Monday
const week_start = Time.start_of_period(Time.now(), 'weekly:mon');
assert(week_start <= Time.now());

// Resets on the first Tuesday of every month
const billing = Time.start_of_period(Time.now(), 'nth_weekday:1:tue');
assert(billing <= Time.now());

// Resets every January 1st
const annual = Time.start_of_period(Time.now(), 'yearly:1-1');
assert(annual <= Time.now());

// Resets on the 1st of each quarter (Jan, Apr, Jul, Oct)
const quarterly = Time.start_of_period(Time.now(), 'quarterly:1');
assert(quarterly <= Time.now());
```


# Time.start_of_week(ts: ms, start_day: int = 0) -> ms
Returns the UTC midnight timestamp for the start of the week containing the given timestamp.
start_day follows ISO convention: 0 = Monday (default), 6 = Sunday.
```rust
const sow = Time.start_of_week(Time.now());       // week starting Monday
assert(sow <= Time.now());

const sun = Time.start_of_week(Time.now(), 6);    // week starting Sunday
assert(sun <= Time.now());
```


# Time.start_of_year(ts: ms) -> ms
Returns the UTC midnight timestamp for January 1st of the year containing the given timestamp.
```rust
const soy = Time.start_of_year(Time.now());
assert(soy <= Time.now());
assert(Time.month(soy) == 1);
assert(Time.day_of_month(soy) == 1);
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


# Time.year(ts: ms) -> int
Returns the UTC year for the given timestamp.
```rust
const y = Time.year(Time.now());
assert(y >= 2025);
```


