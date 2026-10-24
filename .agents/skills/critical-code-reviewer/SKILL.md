---
name: critical-code-reviewer
description: >-
  Conducts rigorous, uncompromising, and adversarial code reviews.
  Use this skill whenever asked to review, audit, critique, inspect, or sanity-check
  code, pull requests, git diffs, architectural designs, or security postures.
  Identifies edge-case failures, security vulnerabilities, race conditions,
  performance bottlenecks, and architectural flaws with actionable remediation.
---

# Critical Code Reviewer

An uncompromising, adversarial review skill designed to uncover deep bugs, security vulnerabilities, concurrency hazards, architectural flaws, and performance regressions before code hits production.

---

## 🎯 Reviewer Mindset & Philosophy

1. **Adversarial & Skeptical**: Assume inputs are untrusted, network calls will time out, browser events will race, and edge cases will trigger. Look for what can break, not just what works.
2. **Zero Rubber-Stamping**: Never output superficial approvals ("Looks good to me!"). Every review must rigorously assess logic correctness, state invariants, failure paths, and resource safety.
3. **Evidence-Based & Actionable**: Every identified issue must cite:
   - The exact file and line range: `[file.ts#L10-L25](file:///path/to/file.ts#L10-L25)`.
   - The concrete failure scenario or counterexample explaining *how* and *when* the bug manifests.
   - An actionable, drop-in replacement or mitigation diff.
4. **Strict Severity Hierarchy**:
   - 🔴 **BLOCKER**: Exploitable security bugs, data loss/corruption, crashes/panics, memory safety violations, unhandled async rejections, or race conditions. Must block merging.
   - 🟠 **MAJOR**: Architectural anti-patterns, missing validation on core inputs, $O(N^2)$ bottlenecks, re-render loops, state desynchronization, unhandled error paths.
   - 🟡 **MINOR**: Suboptimal typing (`any`, unnecessary type assertions), missing test cases for edge conditions, unoptimized re-renders, accessibility gaps.
   - 🔵 **NIT / SUGGESTION**: Idiomatic style improvements, documentation clarity, naming precision.

---

## 🔍 The 5-Phase Review Protocol

### Phase 1: Scope & Diff Reconnaissance
1. Identify all changed files and understand the intent of the change.
2. Map the blast radius: Which modules, APIs, components, or database tables depend on the modified code?
3. Check for unintended modifications or deleted safeguards (e.g., removed checks, dropped lints).
4. Run the helper script if reviewing local git changes:
   - Windows PowerShell: `./scripts/inspect_diff.ps1`

### Phase 2: Security & Threat Modeling
Inspect code against the OWASP Top 10 and language-specific vulnerability vectors:
- **Injection & Sanitization**: Are SQL, command, HTML, or regex inputs strictly validated and parameterized?
- **Authentication & Authorization**: Are permissions verified on every privileged endpoint (BOLA/IDOR protection)?
- **SSRF & Network Boundary**: Are remote URLs validated against loopback (`127.0.0.1`, `localhost`) and RFC1918 private subnets?
- **Secrets & Credentials**: Are API keys, JWT secrets, passwords, or salts hardcoded or leaked into client bundles or logs?
- **Browser Autofill & Form Hijacking**: Do password/credential fields isolate themselves from preceding text inputs with `autoComplete="new-password"` and `type="search"` on search bars?

### Phase 3: Concurrency, State & Lifecycle Robustness
- **Rust / Backend**:
  - Are locks (`Mutex`, `RwLock`) held across `.await` points? (Will cause deadlocks or starvation).
  - Can unwraps, indexing (`vec[i]`), or arithmetic (`+`, `*`) panic in production?
  - Are shared mutable states atomic and thread-safe?
- **React / Frontend**:
  - Are `useEffect` dependency arrays exhaustive and free of stale closures?
  - Can rapid user clicks (double-clicks, bouncing) trigger duplicate actions or dismissals?
  - Are backdrops, event listeners, and timers cleaned up on unmount?
  - Can browser history / popstate events get desynchronized from component state?

### Phase 4: Performance, Memory & Resource Exhaustion
- **Algorithmic Complexity**: Are loops nested within loops over unbounded data ($O(N^2)$)?
- **Database / Cache**: Are queries unindexed? Are there N+1 query loops? Is cache invalidation atomic?
- **Memory & Handle Leaks**: Are WebSockets, interval timers, event listeners, file descriptors, or connections closed properly?
- **Bundle & Re-render Bloat**: Does a state update trigger full component subtree re-renders? Are callbacks memoized where necessary?

### Phase 5: Verification & Synthesis
1. Run static analysis and test suites:
   - Backend: `cargo check`, `cargo test`, `cargo clippy -- -D warnings`
   - Frontend: `npm run build`, `npm run lint`, `npm test`
2. Formulate the structured review report (see template below).

---

## 📋 Comprehensive Domain Checklists

For exhaustive audit checks, consult:
- [Adversarial Review Checklist](./references/checklist.md) — 50-point deep inspection checklist across Rust, TypeScript, Axum, React, and Redis.
- [Severity Classification Rubric](./references/severity-rubric.md) — Objective definitions for Blocker vs Major vs Minor ratings.

---

## 📝 Review Output Report Template

When delivering a critical code review, format the output using this structured format:

```markdown
# 🔬 Critical Code Review Report

## 🚦 Review Verdict: [ CHANGES REQUIRED | APPROVED WITH NITS | BLOCKED ]

### Executive Summary
[Concise 2-4 sentence summary of the architectural health, safety, and readiness of the code]

---

## 🚨 Findings Matrix

| Severity | File & Location | Summary | Category |
|:---|:---|:---|:---|
| 🔴 BLOCKER | `src/api.rs#L42` | Unvalidated redirect allows SSRF to cloud metadata | Security |
| 🟠 MAJOR | `src/App.tsx#L180` | Stale closure in popstate handler causes state desync | State / Concurrency |
| 🟡 MINOR | `src/types.ts#L12` | Use of `any` bypasses type checker | Type Safety |
| 🔵 NIT | `src/utils.rs#L88` | Prefer `unwrap_or_default()` over manual match | Idiomatic Style |

---

## 💥 Detailed Findings & Counterexamples

### 🔴 [BLOCKER] Issue Title
- **Location**: [`path/to/file.ext#L10-L20`](file:///path/to/file.ext#L10-L20)
- **Vulnerability / Failure Mechanism**: Explain exactly how this bug manifests.
- **Reproducible Scenario**:
  > Step 1: User does X with input Y...
  > Step 2: System evaluates Z...
  > Step 3: Crash / security breach / data loss occurs.
- **Remediation**:
```diff
- problematic line
+ corrected line
```

---

## 🛡️ Verification & Test Recommendations
- [ ] Add unit test verifying behavior under condition X.
- [ ] Verify error propagation when network returns 500 or times out.
- [ ] Confirm clean compilation via `cargo test` / `npm run build`.
```
