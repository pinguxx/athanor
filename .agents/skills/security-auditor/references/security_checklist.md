# Security Audit Checklist & Verification Guide

This checklist provides a systematic methodology for verifying application security across both frontend React/TypeScript code and backend Rust/Axum services.

---

## 1. Authentication & Session Security

- [ ] **Password Strength & Storage**:
  - Minimum length $\ge 8$ characters enforced on registration.
  - Passwords hashed using Argon2id or PBKDF2 with unique cryptographic salt per user.
  - Password hashes and salts never included in API responses (check `User` serde attributes: `#[serde(skip_serializing)]`).
- [ ] **JWT Validation & Cryptographic Binding**:
  - JWT signatures verified on every protected request.
  - Expiration timestamp (`exp`) strictly enforced with appropriate token lifetimes (e.g., 15 minutes for access tokens).
  - Refresh tokens rotated upon use and stored securely in `HttpOnly; Secure; SameSite=Strict` cookies.
  - `alg: "none"` or unexpected algorithm transitions strictly rejected.
- [ ] **Account Recovery & Verification**:
  - Verification codes generated with cryptographically secure PRNG (`rand::rngs::OsRng`).
  - Codes have short TTLs (e.g., 15 minutes) and are invalidated immediately upon successful verification.
  - Rate limiting on verification attempts to prevent brute-forcing 6-digit codes ($10^6$ space).

---

## 2. Authorization & Access Control (BOLA / IDOR)

- [ ] **Object Ownership Checks**:
  - User ID from authenticated session compared against resource owner for modifications or deletions.
  - Voting operations confirm voter cannot vote for their own post if prohibited by business rules.
  - Follow/unfollow prevents self-action and confirms caller's identity matches authenticated token.
- [ ] **Privilege Separation**:
  - Moderator / Admin actions require explicit role verification, not merely token presence.
  - Domain-gated assemblies strictly enforce campus email domain matching or verified credentials.
  - Secret passcodes for private assemblies never exposed in public listing payloads.

---

## 3. Network Boundary & SSRF Protection

- [ ] **Outbound URL Fetching**:
  - URL scheme restricted to `http` or `https` (reject `file://`, `gopher://`, `ftp://`).
  - IP resolution performed before request dispatch; IP checked against denylist:
    - `127.0.0.0/8`, `::1` (Loopback)
    - `10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16` (Private IPv4)
    - `169.254.0.0/16`, `fe80::/10` (Link-local & Cloud Metadata)
    - `0.0.0.0/8` (Current network)
    - `fc00::/7` (Unique Local IPv6)
  - Timeouts enforced on all outbound HTTP requests ($\le 5$ seconds).
  - Maximum response body size limits enforced to prevent outbound compression bombs or memory exhaustion.

---

## 4. Input Sanitization & Content Security

- [ ] **Cross-Site Scripting (XSS)**:
  - User text rendered as plain text nodes, not HTML.
  - Markdown/highlighting parsers do not evaluate raw HTML or `<script>` tags.
  - External links use `rel="noopener noreferrer"`.
  - Image URLs validated to be valid web URLs (`https://`) before setting `img.src`.
- [ ] **File & Media Uploads**:
  - File extension not solely relied upon; magic bytes inspected (`image/png`, `image/jpeg`, `image/webp`).
  - SVG uploads sanitized or rejected to prevent SVG-embedded script execution.
  - Maximum upload size strictly enforced by server middleware.

---

## 5. DoS, Rate Limiting & Resource Exhaustion

- [ ] **Rate Limiting Coverage**:
  - High-value endpoints (`/api/auth/login`, `/api/auth/register`, `/api/vote`, `/api/reports`) have per-IP and per-user rate limiters.
  - Redis or in-memory stores track burst limits and sliding windows.
- [ ] **Payload Limits**:
  - Axum body extractors enforce maximum byte limits (e.g., 2MB for general JSON, 10MB for media uploads).
  - Unbounded database queries enforce pagination (`limit`, `offset`) and default caps ($\le 100$ items).

---

## 6. HTTP Headers & Transport Layer

- [ ] **Security Headers on All Responses**:
  - `X-Content-Type-Options: nosniff`
  - `X-Frame-Options: DENY`
  - `Content-Security-Policy: default-src 'self'; ...`
  - `Strict-Transport-Security: max-age=31536000; includeSubDomains`
  - `Referrer-Policy: strict-origin-when-cross-origin`
- [ ] **CORS Configuration**:
  - `Access-Control-Allow-Origin` not set to wildcard `*` on endpoints using credentials.
  - Allowed methods and headers strictly enumerated.
