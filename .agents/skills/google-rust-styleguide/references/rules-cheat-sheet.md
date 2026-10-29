# Google Rust Style Guide: Rules Cheat Sheet

This cheat sheet indexes all normative rules from the [Google Rust Style Guide](http://google.github.io/styleguide/rustguide.html) with objective pass/fail inspection criteria.

---

## 1. Code Organization

| Rule | Requirement | Exceptions / Details |
|:---|:---|:---|
| **Small Crates** | Prefer small, focused crates (ideally focusing on one primary abstraction). | Coherence rules forcing types together, or foundational libraries. |
| **Flat File Layout** | Source files within a single crate directory should avoid unnecessary deep nesting. | Cargo standard crates with separate modules. |
| **Module Naming** | Name source files and leaf modules after the primary user-defined item (`mod foo;` / `foo.rs` for `struct Foo`). | Modules with multiple co-equal abstractions. |
| **No Non-Leaf APIs** | Do not implement API logic in non-leaf modules; non-leaf modules should only contain `mod` declarations and re-exports (`pub use`). | Minor gluing/integration code (<10-20 LOC). |
| **No Glob Imports** | Forbid `use foo::*`. Every imported symbol must be named explicitly. | `use super::*` in `tests`; `use MyEnum::*` inside methods dealing with `MyEnum`; non-leaf `pub use submodule::*`. |
| **Single-Module `use`** | A `use` statement must import items from exactly one module (`imports_granularity = Module`). No `::` inside `{}` blocks. | None. |
| **Locally-Scoped Traits** | Import traits inside function bodies where they are used (`use std::str::FromStr;`), unless used broadly throughout the entire file. | Universally used traits like `Iterator`, `Serialize`, `Deserialize`. |
| **Qualify Top-Level Functions**| Qualify calls to free functions with their module name (`mem::transmute()`, `io::stdout()`, `fs::read()`). | C++ FFI imports or highly disambiguated local functions. |
| **Unqualify Type Names** | Types and traits should be imported and used unqualified (use `Regex`, not `regex::Regex`). | Disambiguating names like `io::Result` vs `std::result::Result`, or macro expansions. |

---

## 2. Formatting & Aesthetics

| Rule | Requirement | Exceptions / Details |
|:---|:---|:---|
| **Automatic Formatter** | Code must be formatted using `rustfmt` with project-standard configuration. | None. |
| **`max_width`** | Standard 100 columns width limit. | URLs or long doc literals. |
| **Hex Literals** | Use lowercase for hexadecimal literals (`0xab12_3cdef` instead of `0xAB12_3CDEF`). | None. |
| **`#[rustfmt::skip]`** | Only permitted for manual alignment (matrices, tabular data) or machine-generated blocks. | Strictly forbidden for personal aesthetic preferences. |

---

## 3. Ownership and Borrowing

| Rule | Requirement | Exceptions / Details |
|:---|:---|:---|
| **Avoid `Rc` and `Arc`** | Prefer unique ownership (`T`, `Box<T>`) and borrowed references (`&T`, `&mut T`). | Asynchronous multi-threaded shared state (e.g. Tokio tasks) or cyclic graphs where borrowing is unreasonably complex. |
| **Prefer Borrows over Copies**| Do not use `.clone()` as a lazy substitute for proper borrowing and lifetime design. | Pragmatic cloning when borrowing adds excessive lifetime complexity, or cloning cheap `Arc`/`Rc` pointers. |

---

## 4. Function Inputs and Outputs

| Rule | Requirement | Exceptions / Details |
|:---|:---|:---|
| **Flexible References** | Pass references to data itself rather than wrappers: `&str` instead of `&String`, `&[T]` instead of `&Vec<T>`, `&T` instead of `&Box<T>`, `Option<&T>` instead of `&Option<T>`. | Non-linear character buffers (rope/cord). |
| **Return Iterators in Libraries**| Prefer returning `impl Iterator<Item = T>` or specific iterator types instead of `Vec<T>` in reusable libraries. | Application code where callers always need a `Vec`, or random indexing is required. |
| **Accept Iterators vs Slices**| Reusable library functions should accept `impl IntoIterator<Item = T>` or iterators if they only need to iterate over items. | Application code where `&[T]` provides simpler ergonomics. |
| **Accepting `impl Into<T>`** | Use `impl Into<T>` sparingly on library APIs where call-site ergonomics are paramount (e.g. `impl Into<String>`). | Do not overuse in application code due to monomorphization bloat. |
| **Enum over Bool** | Replace boolean flags with expressive two-variant enums (`Mode::Async` vs `Mode::Sync`, `Visibility::Public` vs `Visibility::Private`). | Self-explanatory conditions, or document with inline comment `/* is_recursive= */ true`. |
| **Error Reporting Hierarchy**| - `Option<T>`: Obvious failure where reason is self-evident (e.g., `checked_add`, missing key).<br>- `Result<T, E>`: Recoverable, unavoidable failures with rich context (I/O, network, parse).<br>- `panic!`: Avoidable program bugs and violated internal invariants. | Test assertions. |

---

## 5. Types & Invariants

| Rule | Requirement | Exceptions / Details |
|:---|:---|:---|
| **Newtypes vs Aliases** | Use a newtype (`struct UserId(u64)`) whenever the type has semantic differences, domain logic, or requires type safety against mix-ups. | Plain type aliases (`type Result<T> = ...`) for reducing syntactic noise without semantic distinction. |
| **Named Fields over Tuples** | Structs with multiple fields must use named fields instead of tuples or multi-element tuple structs (`struct Point { x: f64, y: f64 }`). | Single-element newtypes (`struct Kilograms(f64)`). |

---

## 6. Traits & Abstractions

| Rule | Requirement | Exceptions / Details |
|:---|:---|:---|
| **Standard Library Traits** | Implement standard traits (`FromStr`, `Display`, `Debug`, `Default`, `Hash`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`) instead of custom ad-hoc method names (`to_string()`, `from_str()`). | Adding an inherent method alongside the trait for discoverability. |
| **Use `#[derive]`** | Prefer `#[derive(...)]` over manual trait implementations whenever possible. | Specialized custom logic required. |
| **Generics vs `dyn Trait`** | Prefer static generics (`T: Trait` or `impl Trait`) over dynamic dispatch (`dyn Trait`) in library APIs for inlining and zero-cost abstraction. | Heterogeneous collections, or strict binary size constraints. |
| **Inherent vs Trait Methods**| Define traits ONLY to enable generic constraints (`T: Trait`), `impl Trait`/`dyn Trait`, or extension traits. Do not use traits merely to group methods. | None. |
| **Extension Traits (`~Ext`)** | Extension traits must carry the suffix `Ext` (`ForeignTypeExt`). Only the owning team of the type should define extension traits unless foundational. | None. |
| **Pair `From` with Constructors** | If a type implements `From<T>`, pair it with an inherent constructor method (`new_from_foo()` or `from_foo()`). | None. |
| **`Default` on Option Structs** | Configuration structs with public fields used as a "bag of options" must implement `Default` to enable functional record update (`Struct { a: 1, ..Default::default() }`). | None. |

---

## 7. Concurrency

| Rule | Requirement | Exceptions / Details |
|:---|:---|:---|
| **Never Access Poisoned Mutex**| Never recover data from inside a poisoned `Mutex` (`lock().map_err(...)` or unchecked unwraps of poisoned state). | None. |

---

## 8. Macros

| Rule | Requirement | Exceptions / Details |
|:---|:---|:---|
| **Metaprogramming Preference** | Prefer: Regular Functions/Types > `macro_rules!` > Code Generation > `proc_macro`. | DSLs or complex boilerplate reduction where functions are impossible. |

---

## 9. Naming & Documentation

| Rule | Requirement | Exceptions / Details |
|:---|:---|:---|
| **Method Conventions** | `as_...()` borrows (`&self -> &T`), `to_...()` expensive conversions (`&self -> T`), `into_...()` consuming conversions (`self -> T`). | None. |
| **Descriptive Lifetimes** | Use meaningful lifetime names (`'ctx`, `'scope`, `'conn`) rather than single-character `'a`, `'b` (except in tiny generic helpers with $\le 2$ lifetimes). | Anonymous lifetimes `'_` where clear. |
| **Generic Parameter Naming** | Short names indicative of trait bound (`T`, `R` for return, `E` for error, `K` for key, `V` for value, `D` for database). | Single-letter letters corresponding to primary bound. |
| **Initialization Functions** | `new()` or `new_with_foo()`. Fallible constructors MUST be named `try_new()` or `try_new_with_foo()`. Value conversions: `from_foo()`. | None. |
| **Avoid Abbreviations** | Do not invent project-specific abbreviations. Write words in full. | Universally recognized acronyms (`HTTP`, `ID`, `URL`, `IP`). |
| **Public API Documentation** | Public APIs must have `///` doc comments detailing behavior, panic preconditions, and safety invariants. | Truly self-evident methods. |
| **Document Panics & Invariants**| Functions that may panic on invalid inputs must have a `# Panics` section. | None. |
| **Module Documentation** | Use `//!` at the top of files to document crates and modules. | None. |

---

## 10. Advanced Features & Implementation Concerns

| Rule | Requirement | Exceptions / Details |
|:---|:---|:---|
| **Unsafe Encapsulation** | Unsafe code must be encapsulated in safe abstractions with strict boundary checks. Every `unsafe` block must have a `// SAFETY:` comment justifying correctness. | 100% safe Rust code is strongly preferred. |
| **No Unstable Features** | Never use `#![feature(...)]` on stable toolchains. | Target-specific embedded platforms with explicit architectural approval. |
| **Getters & Setters** | `foo(&self) -> &T`, `foo_mut(&mut self) -> &mut T`, `set_foo(&mut self, T)`, `take_foo(&mut self) -> Option<T>`, `with_foo(&self, impl FnOnce(&T) -> R) -> R`. | None. |
| **Co-implement API Sets** | If `len(&self)` is implemented, `is_empty(&self) -> bool` MUST also be implemented. | None. |
| **No `#[doc(hidden)]` Access**| Never access external types or functions marked `#[doc(hidden)]`. | None. |
| **`const` Function Caution** | Mark public functions `const` only if guaranteed to remain `const` permanently. | None. |
| **No Non-Math Operator Overloads**| Overload `+`, `-`, `*`, `/` only for true mathematical operations. Never use operators as syntax sugar. | None. |
| **`#[non_exhaustive]` on Enums**| Public enums expected to evolve across boundaries must be marked `#[non_exhaustive]`. | Fixed ontologies (`Color::Red`, `Color::Green`, `Color::Blue`). |
| **`#[non_exhaustive]` on Structs**| Public structs with all-public fields should be marked `#[non_exhaustive]` unless intended for literal construction. | Plain data bags intended for functional record update with `Default`. |
| **`#[must_use]` on Pure Functions**| Pure functions without side effects (`get()`, `calculate()`, `build()`) and critical types (Error/Results) must be marked `#[must_use]`. | If return type is already `#[must_use]`. |
| **No Code Before `main()`** | Do not use global lifecycle hooks (e.g. `ctor` crate) that run before `main()`. | None. |
| **Limit Type Inference** | Add explicit type annotations on complex generic return values (`.into()`, `.parse()`) and long iterator results. | Obvious single-type assignments. |
| **Pattern Match over Unwrap** | Replace `if x.is_some() { x.unwrap() }` with `if let Some(val) = x { ... }` or `let Some(val) = x else { return; };`. | None. |
| **`expect` over `unwrap`** | In test code or proven invariants, use `.expect("Context reason")` instead of bare `.unwrap()`. | In non-test code, avoid both. |
| **Non-Exhaustive Match Handling**| When matching `#[non_exhaustive]` enums, list known variants explicitly and use a wildcard arm `_ => ...` for unknown future variants. | `if let` when only one variant matters. |
| **Context in `assert!`** | Assertions must provide contextual failure messages: `assert!(x > 0, "Counter must be positive, got {x}")`. | Trivial self-evident conditions. |
| **Combinators vs Loops** | Do not write unreadable multi-line combinator chains with state mutation or complex conditionals. Use standard `for` loops. | Simple, linear map/filter/collect pipelines. |
| **Prohibit `catch_unwind`** | Prohibit `std::panic::catch_unwind` in standard application and library logic. | Foreign FFI boundaries or defense-in-depth isolation in host runners. |
| **`format!` / `write!` for Strings**| Use `format!("{a}{b}{c}")` or `write!` instead of repeated string concatenations with `+` or ad-hoc templates. | Exactly two string slices where `+` is clear. |
