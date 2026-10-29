---
name: google-rust-styleguide
description: >-
  Authoritative specialist and auditor for the official Google Rust Style Guide (http://google.github.io/styleguide/rustguide.html).
  Use this skill whenever asked to review, audit, verify, refactor, inspect, or compare Rust code against the Google Rust Style Guide.
  Knows all rules by heart, including code organization, imports, ownership, function signatures, types, traits, concurrency, naming, documentation, and implementation idioms.
---

# Google Rust Style Guide: Expert Auditor & Reviewer

An authoritative, exacting reviewer that knows the official [Google Rust Style Guide](http://google.github.io/styleguide/rustguide.html) by heart and systematically audits, compares, and refactors Rust codebases to align with Google's engineering standards.

---

## 🎯 Core Philosophy & Guiding Principles

1. **Readability & Maintainability over Writeability**: Code is read orders of magnitude more often than it is written. Extra keystrokes during authoring are well worth it if they clarify intent for future readers and maintainers.
2. **Facilitate Large-Scale Changes (LSCs)**: Google operates massive monorepos where codebase-spanning refactorings (LSCs) and automated codemods are routine. Rules like prohibiting glob imports (`use foo::*`) and qualifying free function calls exist primarily to ensure automated tooling and code janitors can safely refactor symbols.
3. **Consistency with the Broader Rust Ecosystem**: Google adopts standard public Rust conventions (the official Rust Style Guide and Rust API Guidelines) unless an explicit Google-specific rule overrides them.
4. **Rules Must Pull Their Weight**: Avoid gratuitous rules; every guideline has a tangible maintainability, safety, or automation rationale.

---

## 📖 The 10 Essential Domains of the Google Rust Style Guide

### 1. Code Organization & Imports
- **Prefer Small Crates**: Design small, single-purpose crates focused on one primary abstraction.
- **Name Files After Primary Item**: If a leaf module defines `struct Foo`, name the file `foo.rs` (`mod foo;`).
- **No APIs in Non-Leaf Modules**: Non-leaf modules must only contain `mod` declarations and `pub use` re-exports. Keep business logic in leaf modules.
- **Forbid Glob Imports (`use foo::*`)**: Every imported item must be explicitly named to prevent namespace pollution and breakages during upstream additions.
  - *Exceptions*: `use super::*` in test submodules; `use MyEnum::*` inside enum method bodies; `pub use submodule::*` in non-leaf facade modules.
- **Single-Module Use Statements**: Exactly one module per `use` statement (`imports_granularity = Module`). Forbid `::` inside `{}` blocks (e.g., `use foo::{bar, baz::qux}` is forbidden; write separate `use foo::bar;` and `use foo::baz::qux;`).
- **Locally-Scoped Trait Imports**: Import traits inside function bodies where they are used (e.g., `use std::str::FromStr;`), unless the trait methods are used extensively throughout the file.
- **Qualify Free Function Calls**: Top-level/free functions defined in external modules must be qualified by their module name (e.g., `mem::transmute(x)`, `fs::read(path)`). Unqualified lowercase function calls (`foo()`) signal local module functions.
- **Unqualify Type Names**: Types and traits should be imported and referenced unqualified (e.g., `Regex`, not `regex::Regex`), except when disambiguating common names (like `io::Result`).

### 2. Code Formatting & Aesthetics
- **Standard Formatting**: Enforce `rustfmt` with project defaults (`max_width = 100`, `hex_literal_case = Lower`).
- **Strict `#[rustfmt::skip]` Restrictions**: Never skip formatting for personal aesthetic choices. Only permitted for manual alignment (tables, matrices) or code generators.

### 3. Ownership & Borrowing
- **Avoid `Rc` and `Arc`**: Default to unique ownership (`T`, `Box<T>`) and borrowed references (`&T`, `&mut T`). `Arc`/`Rc` introduce runtime overhead and cyclic complexity.
  - *Exception*: Multi-threaded async state sharing (e.g. Tokio tasks) or inherently shared cyclic object graphs.
- **Prefer Borrows over Deep Copies**: Do not use `.clone()` as a lazy workaround for borrowing. Reserve `.clone()` for genuine needs for independent copies. (Cloning cheap `Arc`/`Rc` handles is expected).

### 4. Function Inputs and Outputs
- **Accept Flexible Reference Types**: Always borrow the underlying data, not wrapper containers:
  - `&str` instead of `&String`
  - `&[T]` instead of `&Vec<T>`
  - `&T` instead of `&Box<T>`
  - `Option<&T>` instead of `&Option<T>`
- **Iterators vs Collections in Return Types**: Reusable libraries should prefer returning iterators (`impl Iterator<Item = T>`) so callers can choose how to allocate or chain. (Application code may return `Vec<T>` if clearer).
- **Iterators vs Slices in Parameters**: Reusable libraries should accept `impl IntoIterator<Item = T>` if only iterating.
- **Accepting `impl Into<T>`**: Use `impl Into<T>` sparingly on library APIs where call-site convenience outweighs monomorphization cost. Application code should avoid `impl Into<T>`.
- **Enums over Booleans**: Replace boolean arguments with descriptive two-variant enums (`Mode::Async` vs `Mode::Sync`). If using `bool`, document with inline comment `/* is_recursive= */ true`.
- **Error Reporting Hierarchy**:
  - `Option<T>`: Simple, self-evident failure (e.g. `checked_add`, missing key).
  - `Result<T, E>`: Recoverable operational failures with rich context (I/O, network, parse).
  - `panic!`: Avoidable bugs and violated internal invariants.

### 5. Types & Invariants
- **Newtypes over Type Aliases**: Use a newtype (`struct UserId(u64)`) for semantic distinction, type safety against mix-ups, or types needing their own API. Use plain type aliases only for reducing syntactic verbosity.
- **Named Fields over Tuples**: Multi-element structs must use named fields (`struct Point { x: f64, y: f64 }`), never anonymous tuples or tuple structs with `.0`, `.1`.

### 6. Traits & Abstractions
- **Implement Known Standard Traits**: Implement standard traits (`FromStr`, `Default`, `Display`, `Debug`, `Hash`, `PartialEq`, `Ord`) instead of inventing custom method names (`from_string()`).
- **Prefer `#[derive]`**: Avoid manual trait implementations when `#[derive]` suffices.
- **Generics over `dyn Trait`**: Prefer `T: Trait` or `impl Trait` over dynamic dispatch `dyn Trait` in library APIs to allow compiler inlining and devirtualization.
- **Inherent vs Trait Methods**: Define traits ONLY for generic bounds, dynamic dispatch, or extension traits. Do NOT use traits purely to group methods.
- **Extension Traits (`~Ext`)**: Extension traits must carry the suffix `Ext` (`ForeignTypeExt`). Only the owning team should define extension traits unless foundational.
- **Pair `From` with Constructors**: If a type implements `From<T>`, pair it with an inherent constructor method (`new_from_foo()` or `from_foo()`).
- **`Default` on Option Structs**: Structs with public fields used as a "bag of options" must implement `Default` to allow functional record update syntax (`Config { timeout: 10, ..Default::default() }`).

### 7. Concurrency
- **Handling Poison**: Never reach inside a poisoned `Mutex` to salvage state. A panic while holding a lock means invariants are broken.

### 8. Macros
- **Metaprogramming Preference Hierarchy**:
  1. Regular Rust functions, types, and traits (always prefer safe, typed code).
  2. `macro_rules!` (syntactic AST transformations).
  3. External code generation (build scripts, generators).
  4. `proc_macro` (use only when declarative macros cannot express the abstraction).

### 9. Naming & Documentation
- **Method Conventions**: `as_...()` for borrowed views (`&self -> &T`), `to_...()` for expensive copies/conversions (`&self -> T`), `into_...()` for consuming conversions (`self -> T`).
- **Initialization Naming**: `new()`, `new_with_foo()`. Fallible constructors MUST be named `try_new()` or `try_new_with_foo()`.
- **Descriptive Lifetimes**: Use descriptive lifetime names (`'ctx`, `'scope`) rather than generic single-character `'a` when multiple lifetimes exist.
- **Avoid Abbreviations**: Do not invent project-specific abbreviations. Write full words (`request`, `response`, `transaction`).
- **Public API Documentation**: All public APIs must have `///` comments explaining behavior, panic conditions (`# Panics`), and safety invariants (`# Safety`).
- **Module Documentation**: Use `//!` at the top of files to document crates and modules.

### 10. Advanced Features & Implementation Concerns
- **Unsafe Code Containment**: Unsafe code must be encapsulated in safe abstractions. Every `unsafe` block must have a `// SAFETY:` comment explicitly detailing why the preconditions are met.
- **No Unstable Features**: Forbid `#![feature(...)]` on stable toolchains.
- **Getters & Setters**: `foo(&self) -> &T`, `foo_mut(&mut self) -> &mut T`, `set_foo(&mut self, T)`, `take_foo(&mut self) -> Option<T>`, `with_foo(&self, impl FnOnce(&T) -> R) -> R`.
- **Implement Expected API Sets Together**: If implementing `len()`, always implement `is_empty()`.
- **No `#[doc(hidden)]` Access**: Never call external APIs marked `#[doc(hidden)]`.
- **`#[non_exhaustive]` on Public Enums & Structs**: Mark evolving public enums and structs with public fields `#[non_exhaustive]`.
- **`#[must_use]` on Pure Functions & Types**: Mark pure functions without side effects and critical types with `#[must_use]`.
- **No Code Before `main()`**: Never execute global constructors before `main()`.
- **Limit Type Inference**: Provide explicit types on complex iterator chains and generic method calls (`.into()`, `.parse()`).
- **Pattern Matching over Check-Then-Unwrap**: Replace `if opt.is_some() { opt.unwrap() }` with `if let Some(x) = opt { ... }`.
- **`expect` over `unwrap`**: In tests or proven invariants, use `.expect("descriptive context")` instead of bare `.unwrap()`.
- **Context Strings in `assert!`**: Always provide meaningful explanations in assertions (`assert!(cond, "expected X because Y, got {val}")`).
- **Avoid Complex Combinator Chains**: If an iterator chain requires mutating external state or complex branching, use a readable `for` loop.
- **Prohibit `catch_unwind`**: Never use `std::panic::catch_unwind` in standard business logic. (Only valid for FFI boundaries or host defense-in-depth).
- **`format!` / `write!` for Strings**: Use `format!("{a}{b}{c}")` or `write!` rather than multiple string concatenations with `+`.

---

## 🔬 The 5-Phase Comparison & Audit Protocol

When asked to compare code against the Google Rust Style Guide, execute the following protocol:

### Phase 1: Automated Static Scan
Run the automated style scanner script on the target files or crates:
```powershell
python .agents/skills/google-rust-styleguide/scripts/audit_google_style.py <target-crate-or-dir>
```
Collect all identified AST violations (glob imports, rigid references, missing `is_empty()`, `catch_unwind`, etc.).

### Phase 2: Imports & Module Organization Audit
Inspect imports and file structure:
- Are there any glob imports (`use foo::*`)?
- Does any `use` statement span multiple modules?
- Are top-level free functions qualified by module (`mem::transmute`, `io::stdout`)?
- Are types imported unqualified?

### Phase 3: Signatures, Ownership & Error Types Audit
Inspect all function signatures:
- Are parameters using `&str`, `&[T]`, `Option<&T>` instead of `&String`, `&Vec`, `&Option`?
- Are boolean flags used where an enum would be clearer?
- Are `Rc`/`Arc` avoided in favor of unique ownership or borrowed references?
- Are error mechanisms properly categorized (`Option` vs `Result` vs `panic!`)?

### Phase 4: Types, Traits & Concurrency Audit
Inspect structs, enums, and trait implementations:
- Are standard traits (`FromStr`, `Default`, `Display`) used instead of custom methods?
- If `len()` is present, is `is_empty()` implemented?
- Are public options structs implementing `Default`?
- Are public evolving enums marked `#[non_exhaustive]`?
- Are pure functions annotated with `#[must_use]`?

### Phase 5: Implementation Idioms & Findings Matrix
Inspect function bodies:
- Is pattern matching used instead of check-then-unwrap?
- Do assertions provide context strings?
- Are `format!` / `write!` used instead of string `+` concatenations?
- Are unsafe blocks accompanied by `// SAFETY:` justifications?
- Synthesize the report using the template below.

---

## 📝 Comparison Report Template

```markdown
# 🏛️ Google Rust Style Guide Compliance Audit: [TARGET NAME]

## 🚦 Compliance Verdict: [ FULLY COMPLIANT | ACTIONABLE REFACTORING REQUIRED | CRITICAL DEVIATIONS ]

### Executive Summary
[Concise 2-4 sentence summary of the codebase's adherence to the Google Rust Style Guide, highlighting major strengths and primary deviations.]

---

## 📊 Compliance Scorecard

| Category | Status | Primary Findings |
|:---|:---:|:---|
| **1. Code Organization & Imports** | 🟢 PASS / 🟡 WARN / 🔴 FAIL | Glob imports, module qualifications, use granularity |
| **2. Formatting & Aesthetics** | 🟢 PASS / 🟡 WARN / 🔴 FAIL | rustfmt conformity, column width, skip attributes |
| **3. Ownership & Borrowing** | 🟢 PASS / 🟡 WARN / 🔴 FAIL | Rc/Arc minimization, borrow vs clone |
| **4. Function Inputs & Outputs** | 🟢 PASS / 🟡 WARN / 🔴 FAIL | Flexible references (&str, &[T]), enums vs bools |
| **5. Types & Invariants** | 🟢 PASS / 🟡 WARN / 🔴 FAIL | Newtypes vs aliases, named fields |
| **6. Traits & Abstractions** | 🟢 PASS / 🟡 WARN / 🔴 FAIL | Standard traits, derive, generic vs dyn |
| **7. Concurrency** | 🟢 PASS / 🟡 WARN / 🔴 FAIL | Poisoned mutex handling, async ergonomics |
| **8. Macros & Metaprogramming** | 🟢 PASS / 🟡 WARN / 🔴 FAIL | Macro justification, AST cleanliness |
| **9. Naming & Documentation** | 🟢 PASS / 🟡 WARN / 🔴 FAIL | as_/to_/into_, try_new, public docs |
| **10. Implementation Concerns** | 🟢 PASS / 🟡 WARN / 🔴 FAIL | Pattern matching vs unwrap, #[must_use], assert context |

---

## 🚨 Deviations & Actionable Remediations

### Finding 1: [Short Title]
- **Rule**: [Google Style Guide Section & Rule Name]
- **Location**: [`path/to/file.rs#L10-L15`](file:///absolute/path/to/file.rs#L10-L15)
- **Severity**: 🔴 BLOCKER / 🟠 MAJOR / 🟡 MINOR / 🔵 NIT
- **Current Code**:
  ```rust
  // Current non-compliant snippet
  ```
- **Recommended Refactoring (Compliant)**:
  ```rust
  // Drop-in Google Rust Style Guide compliant replacement
  ```
- **Rationale**: [Explanation directly quoting or citing the Google Rust Style Guide]

---

## 🎯 Next Steps & Suggested Refactoring Plan
1. [Prioritized actionable step]
2. [Prioritized actionable step]
```

---

## 📚 Reference Links & Resources
- [Rules Cheat Sheet](./references/rules-cheat-sheet.md) — Exhaustive matrix of all rules and pass/fail criteria.
- [Anti-Patterns & Idiomatic Equivalents](./references/anti-patterns.md) — Good vs Bad code examples for every rule.
- [Automated Audit Script](./scripts/audit_google_style.py) — Python AST scanner checking workspace code.
