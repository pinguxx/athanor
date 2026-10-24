# 🔬 50-Point Adversarial Review Checklist

This checklist provides deep-inspection questions across five mission-critical engineering domains.

---

## 1. Rust & Backend Systems (10 Points)

1. **Panic Freedom**: Does the code contain any unvalidated `.unwrap()`, `.expect()`, `panic!()`, or unreachable branches that could be triggered by malformed user input?
2. **Bounds & Slicing**: Are array indices (`slice[i]`) guarded against `IndexOutOfBounds`? Are range operations (`slice[start..end]`) checked?
3. **Integer Arithmetic**: Can arithmetic (`+`, `-`, `*`) overflow or divide by zero under extreme values? (Use `checked_add`, `saturating_add`, or explicit bounds checking).
4. **Lock Scope & Concurrency**: Are `std::sync::Mutex` or `tokio::sync::RwLock` guards held across `.await` points or over lengthy I/O operations?
5. **Async Runtime Blocking**: Are CPU-intensive loops or synchronous I/O executed inside an async task without `spawn_blocking`?
6. **Error Propagation**: Are errors properly surfaced using `?` and typed error enums (`Result<T, AppError>`), or are errors swallowed silently with `let _ = ...`?
7. **Resource Cleanup**: Are file descriptors, database connections, and sockets guaranteed to release on early-return or error exit?
8. **Clone & Allocation Overhead**: Are heavy structures cloned unnecessarily in hot loops instead of borrowing (`&str`, `&[T]`, `Cow`)?
9. **Deadlock Vulnerability**: If multiple locks are acquired, are they consistently acquired in a fixed, global order across all threads?
10. **Trait Invariants**: Do custom implementations of `PartialEq`, `Eq`, `Hash`, or `Ord` uphold mathematical equivalence invariants?

---

## 2. Security & Threat Modeling (10 Points)

11. **BOLA / IDOR**: Does every handler fetching or modifying an object by ID verify that the requesting user has explicit authorization to access *that specific* object?
12. **Injection Defense**: Are SQL queries strictly parameterized? Are user inputs passed to system commands, regex engines, or shell interpreters?
13. **SSRF Neutralization**: Are URLs supplied by users validated to forbid loopback (`127.0.0.1`, `localhost`, `[::1]`), cloud metadata services (`169.254.169.254`), and private networks (`10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`)?
14. **Cross-Site Scripting (XSS)**: Is untrusted HTML, markdown, or SVG sanitized with strict tag/attribute allowlists before rendering?
15. **Cross-Site Request Forgery (CSRF)**: Are mutation endpoints protected by CSRF tokens or `SameSite=Strict/Lax` cookies with `Authorization` headers?
16. **Token & Session Hygiene**: Are JWTs signed with robust algorithms (e.g., HS256/RS256), validated for expiration (`exp`), and revoked on logout?
17. **Rate Limiting & DoS**: Are expensive endpoints (authentication, file upload, search, AI synthesis) protected by sliding-window rate limiters?
18. **Payload Limits**: Are request body sizes clamped to prevent memory exhaustion from oversized payloads?
19. **Information Leakage**: Do error responses reveal stack traces, internal database schema details, or server directory paths?
20. **Autofill Bleed**: Do modal password fields isolate themselves with `autoComplete="new-password"`, unique names, and `type="search"` on search bars to prevent browser credential contamination?

---

## 3. React & Frontend State Management (10 Points)

21. **Stale Closures**: In `useEffect`, `useCallback`, and event handlers, are all referenced state and props present in the dependency array?
22. **Uncontrolled Re-render Cascades**: Does state update at the root trigger unnecessary re-renders of the entire feed or heavy component trees?
23. **Modal & Backdrop Event Isolation**: Does opening a modal debounce backdrop clicks to prevent double-click self-dismissal? Do modal buttons have `type="button"` and `e.stopPropagation()`?
24. **History & Navigation Sync**: When using URL hashes or history states (`pushState`, `popstate`), can navigating back drop the user into an unintended view or desynchronize UI filters?
25. **Memory Leak Hygiene**: Are all event listeners, intervals (`setInterval`), timeouts (`setTimeout`), and WebSocket connections cleanly removed in `useEffect` cleanup return functions?
26. **Accessibility (a11y)**: Do interactive elements have appropriate ARIA attributes (`role="tab"`, `aria-selected`, `aria-label`), keyboard navigation (`Escape`, `Tab`), and visible focus rings?
27. **Input Debouncing**: Are live search boxes, autosuggestions, and filter queries debounced to avoid hammering state updates or API backends?
28. **Type Safety & `any`**: Are TypeScript types strictly defined without falling back to `any` or blind type assertions (`as unknown as T`)?
29. **Error Boundaries**: Are subtrees (especially complex widgets, markdown parsers, or charts) isolated inside React Error Boundaries so a component render crash doesn't unmount the whole application?
30. **Offline & Degraded States**: Does the frontend gracefully inform the user when offline or when a backend service returns a 503 Maintenance state?

---

## 4. API Design & Data Persistence (10 Points)

31. **Idempotency**: Can mutating actions (payments, voting, publishing, status changes) be safely retried without creating duplicate records or billing?
32. **Pagination & Limits**: Are list endpoints explicitly paginated with sensible default and maximum page sizes (e.g., max 50 items)?
33. **Schema Migrations**: Are database schema migrations backwards-compatible with running application versions (expand/contract pattern)?
34. **Index Coverage**: Are query filters (`WHERE`, `ORDER BY`, `JOIN`) backed by database indexes to prevent table scans?
35. **Cache Invalidation**: When an entity is updated, are all associated cache keys (Redis, CDN, in-memory) invalidated atomically?
36. **Serialization Consistency**: Do serialization names (`snake_case` vs `camelCase`) match across frontend TypeScript interfaces and backend Rust structs?
37. **Timezone Uniformity**: Are all timestamps stored and transferred in UTC ISO-8601 format (`YYYY-MM-DDTHH:mm:ssZ`)?
38. **Atomic Transactions**: Are multi-step database mutations wrapped inside transactions to prevent partial updates upon failure?
39. **Stateless Scalability**: Does the application server store session state in external stores (Redis/DB) rather than in-memory process globals?
40. **Health & Readiness Probes**: Are `/health` and `/ready` endpoints lightweight and validating actual downstream connectivity?

---

## 5. Testing & Verification Rigor (10 Points)

41. **Failure Path Coverage**: Do automated tests assert error responses (e.g., 400 Bad Request, 401 Unauthorized, 403 Forbidden, 404 Not Found), not just 200 OK happy paths?
42. **Boundary Testing**: Are tests verifying edge values: 0, 1, empty strings, max integer, strings with unicode/emojis, whitespace-only?
43. **Concurrency Stress**: Are race conditions tested with concurrent requests (e.g., double-voting, rapid sequential checkout)?
44. **Mock Authenticity**: Do test mocks accurately reflect real service behaviors, status codes, and error formats?
45. **Deterministic Results**: Are tests free of race conditions, hardcoded sleep timeouts (`sleep(500)`), or dependencies on external live network services?
46. **Sanitization Testing**: Are XSS polyglots and malicious strings included in test suites to verify escaping?
47. **Regression Guard**: Is every bug fix accompanied by a regression test replicating the exact reported failure?
48. **Lint & Formatting Zero-Tolerance**: Does the PR pass `cargo clippy -- -D warnings` and `npm run lint` with zero warnings?
49. **Clean Build Verification**: Does `npm run build` and `cargo test` complete cleanly with zero build errors?
50. **No Prohibited Terminology**: Does the codebase strictly maintain branding, compliance, and terminology constraints without forbidden artifacts?
