# Google Rust Style Guide: Anti-Patterns & Idiomatic Equivalents

Direct "Good vs Bad" comparative code examples extracted directly from the [Google Rust Style Guide](http://google.github.io/styleguide/rustguide.html).

---

## 1. Imports and Module Calls

### Glob Imports
```rust
// ❌ BAD: Glob import creates namespace pollution and makes LSCs difficult
use std::collections::*;
use crate::models::*;

// ✅ GOOD: Explicit item imports
use std::collections::{HashMap, HashSet};
use crate::models::{User, Account};
```

### Module Granularity in Use Statements
```rust
// ❌ BAD: Multiple nested module paths in a single use block
use foo::{Foo, bar::{Bar, Baz}};

// ✅ GOOD: One use statement per module
use foo::Foo;
use foo::bar::{Bar, Baz};
```

### Qualify Calls to Top-Level Functions
```rust
// ❌ BAD: Calling foreign free function unqualified
use std::mem::transmute;
let x = transmute(y);

// ✅ GOOD: Qualify top-level function with its module name
use std::mem;
let x = mem::transmute(y);
```

### Unqualify Type Names
```rust
// ❌ BAD: Type referenced with full module path in method bodies
fn parse(s: &str) -> regex::Regex {
    regex::Regex::new(s).unwrap()
}

// ✅ GOOD: Import the type and reference it unqualified
use regex::Regex;
fn parse(s: &str) -> Regex {
    Regex::new(s).unwrap()
}
```

---

## 2. Function Parameter and Return Types

### Flexible Reference Types
```rust
// ❌ BAD: Demanding specific container references
fn find_user(name: &String, ids: &Vec<u64>, cfg: &Box<Config>, opt: &Option<Tag>) {}

// ✅ GOOD: Flexible borrowed slices and options
fn find_user(name: &str, ids: &[u64], cfg: &Config, opt: Option<&Tag>) {}
```

### Prefer Enums over Bools
```rust
// ❌ BAD: Mysterious boolean flags
fn render(text: &str, truncate: bool, highlight: bool) {}
render("hello", true, false); // What does true mean?

// ✅ GOOD: Explicit descriptive enums
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Truncation {
    Truncate,
    KeepFull,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Highlight {
    Enabled,
    Disabled,
}

fn render(text: &str, trunc: Truncation, hl: Highlight) {}
render("hello", Truncation::Truncate, Highlight::Disabled);
```

### Error Reporting: Option vs Result vs Panic
```rust
// ✅ GOOD: Option when cause is obvious and no details exist
pub fn checked_sub(a: u32, b: u32) -> Option<u32> {
    if a >= b { Some(a - b) } else { None }
}

// ✅ GOOD: Result for recoverable operational/IO failures with context
pub fn load_state(path: &Path) -> Result<State, StorageError> { ... }

// ✅ GOOD: Panic only for invariant violations / developer bugs
pub fn get_element(&self, index: usize) -> &Element {
    if index >= self.len() {
        panic!("index {index} out of bounds for collection of length {}", self.len());
    }
    &self.items[index]
}
```

---

## 3. Type Definitions and Structures

### Named Fields over Tuples
```rust
// ❌ BAD: Multi-value tuple structs with confusing .0, .1 indexing
pub struct Coordinates(pub f64, pub f64, pub f64);
let x = coord.0;
let z = coord.2;

// ✅ GOOD: Named fields
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
}
```

### Default on Options Structs
```rust
// ❌ BAD: Config struct without Default forces manual initialization of all fields
pub struct ClientConfig {
    pub timeout_ms: u64,
    pub retries: u32,
    pub user_agent: String,
}

// ✅ GOOD: Implements Default for functional record update syntax
#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub timeout_ms: u64,
    pub retries: u32,
    pub user_agent: String,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 5000,
            retries: 3,
            user_agent: "Athanor/0.1.0".to_string(),
        }
    }
}

// Call site can cleanly customize only specific fields:
let cfg = ClientConfig {
    retries: 5,
    ..Default::default()
};
```

---

## 4. Traits and Abstractions

### Standard Library Traits instead of Ad-Hoc Methods
```rust
// ❌ BAD: Ad-hoc method duplicating standard trait
impl IpAddress {
    pub fn from_string(s: &str) -> Result<Self, ParseError> { ... }
}

// ✅ GOOD: Standard FromStr implementation
impl std::str::FromStr for IpAddress {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> { ... }
}
```

### Extension Trait Naming and Suffix
```rust
// ❌ BAD: Extension trait without ~Ext suffix
pub trait StringHelper {
    fn sanitize(&self) -> String;
}

// ✅ GOOD: Trait clearly identified with ~Ext suffix
pub trait StringExt {
    fn sanitize(&self) -> String;
}
impl StringExt for str {
    fn sanitize(&self) -> String { ... }
}
```

---

## 5. Implementation Idioms

### Pattern Matching over Check-Then-Unwrap
```rust
// ❌ BAD: Check-then-unwrap
if result.is_some() {
    process(result.unwrap());
}

// ✅ GOOD: Pattern matching
if let Some(val) = result {
    process(val);
}
```

### Expect over Bare Unwrap
```rust
// ❌ BAD: Bare unwrap with no diagnostic context
let port: u16 = std::env::var("PORT").unwrap().parse().unwrap();

// ✅ GOOD: Expect documenting why the value must exist
let port_str = std::env::var("PORT")
    .expect("PORT environment variable must be set by container orchestrator");
let port: u16 = port_str
    .parse()
    .expect("PORT must be a valid 16-bit unsigned integer");
```

### Matching Non-Exhaustive Enums
```rust
#[non_exhaustive]
pub enum Event {
    Start,
    Stop,
    Pause,
}

// ❌ BAD: Wildcard without documenting known non-handled variants
match event {
    Event::Start => handle_start(),
    _ => (),
}

// ✅ GOOD: Group unhandled known variants or document wildcard
match event {
    Event::Start => handle_start(),
    Event::Stop | Event::Pause | _ => (),
}
```

### Context Strings in `assert!`
```rust
// ❌ BAD: Bare assert or assertion that merely repeats the condition
assert!(items.is_empty());
assert!(items.is_empty(), "items should be empty");

// ✅ GOOD: Explaining the invariant or showing actual values
assert!(
    items.is_empty(),
    "items queue must be drained prior to shutdown, but contained {} items",
    items.len()
);
```

### Combinator Chains vs `for` Loops
```rust
// ❌ BAD: Unreadable combinator chain mutating state across closures
let mut failure_count = 0;
let valid_records: Vec<_> = entries
    .into_iter()
    .map(validate_entry)
    .map(|res| {
        if res.is_err() { failure_count += 1; }
        res
    })
    .filter_map(Result::ok)
    .collect();

// ✅ GOOD: Clear and readable for loop
let mut failure_count = 0;
let mut valid_records = Vec::with_capacity(entries.len());
for entry in entries {
    match validate_entry(entry) {
        Ok(record) => valid_records.push(record),
        Err(_) => failure_count += 1,
    }
}
```

### String Formatting vs Ad-Hoc Concatenation
```rust
// ❌ BAD: Repeated string allocations with +
let msg = prefix + ": " + &topic + " -> " + &payload;

// ✅ GOOD: format! with single allocation
let msg = format!("{prefix}: {topic} -> {payload}");
```
