# ⚖️ Severity Classification Rubric

Use this standardized rubric to ensure consistent, objective classification of code review findings.

---

## 🔴 BLOCKER (P0 / Critical)
**Criteria**: Code that compromises security, causes data corruption, crashes the service, or introduces non-recoverable regressions. **Merge is blocked until resolved.**

### Concrete Examples:
- SQL injection, command injection, unescaped XSS vulnerability.
- Server-Side Request Forgery (SSRF) allowing internal network or cloud metadata access.
- Authentication or authorization bypass (e.g., IDOR / BOLA allowing any user to edit another user's post).
- Panic on untrusted input in Rust (`.unwrap()`, out-of-bounds slice indexing).
- Unhandled Promise rejection or React component render crash that takes down the entire application.
- Data race or deadlock condition under concurrent access.
- Plaintext credential, secret, or private key hardcoded into source or committed to Git.
- Violation of non-negotiable project commitments or core quality gates (e.g., failing backend tests, prohibited terminology).

---

## 🟠 MAJOR (P1 / High)
**Criteria**: Flaws in architecture, performance, error handling, or state synchronization that impair system reliability, scalability, or user experience. **Resolution strongly required prior to release.**

### Concrete Examples:
- Unindexed database queries or $O(N^2)$ in-memory loops over potentially unbounded collections.
- Stale closures or missing dependencies in `useEffect` causing state desynchronization.
- Missing input validation on public endpoints (e.g., negative duration, title > 10,000 chars).
- Unhandled network error states or silent error swallowing (`catch {}` without logging or UI notification).
- Browser autofill credential bleed (e.g., password manager injecting passwords into search inputs).
- Memory leak hazards: Event listeners, intervals, or WebSocket connections not removed on component unmount.
- Lack of transaction boundaries across multi-table mutating operations.

---

## 🟡 MINOR (P2 / Medium)
**Criteria**: Code smells, non-critical edge cases, type safety degradation, or missing test coverage that does not break the core user flow but reduces maintainability and robustness.

### Concrete Examples:
- TypeScript `any` or loose type assertions (`as unknown as TargetType`).
- Missing unit tests for edge-case boundaries (e.g., empty lists, single-item collections).
- Suboptimal re-rendering of large lists due to missing `React.memo` or unstable callback references.
- Accessibility gaps (e.g., missing `aria-label` on icon-only buttons, poor contrast on secondary badges).
- Repetitive boilerplate that should be abstracted into a reusable utility.
- Missing docstrings on public APIs or non-obvious algorithms.

---

## 🔵 NIT / OBSERVATION (P3 / Low)
**Criteria**: Stylistic enhancements, minor idiomatic conventions, or micro-optimizations that can be accepted or deferred at the author's discretion.

### Concrete Examples:
- Minor variable or function naming improvements for enhanced clarity.
- Using idiomatic language constructs (e.g., `if let Some(x)` vs `match`, or optional chaining `?.`).
- Typo corrections in internal comments or log messages.
- Ordering of import statements or structuring of CSS classes.
