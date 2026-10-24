---
name: security-auditor
description: >-
  Audits the security posture of the codebase, APIs, and live site.
  Use this skill whenever asked to review security, conduct a vulnerability assessment,
  pentest, audit authentication, inspect SSRF/XSS/SQLi defenses, check security headers,
  or evaluate data privacy, cryptographic integrity, and OWASP compliance.
---

# Security Auditor

A specialized, offensive-and-defensive application security auditing skill designed to discover, demonstrate, and remediate vulnerabilities across backend APIs, frontend clients, authentication flows, cryptographic protocols, and live web deployments.

---

## 🎯 Security Auditor Philosophy & Approach

1. **Defense-in-Depth & Zero Trust**: Assume the perimeter is breached, inputs are crafted maliciously, client tokens can be tampered with, and external services might be spoofed. Validate and sanitize at every layer.
2. **Comprehensive Attack Surface Mapping**: Evaluate vulnerabilities not just in code syntax, but in business logic, authorization boundaries, cryptographic invariants, network boundary transitions (SSRF), and browser security controls (CORS, CSP, cookies).
3. **Exploitability & Concrete PoCs**: Accompany security findings with concrete proof-of-concept payloads or attack scenarios explaining *how* an adversary would exploit the flaw and *what* the blast radius is.
4. **Actionable Remediation**: Provide production-ready, hardened code diffs with defense-in-depth mitigations (e.g., parameterization, strict allowlists, defense headers, cryptographic binding).

---

## 🛡️ Core Vulnerability Vectors & Checklists

### 1. Injection & Input Sanitization
- **SQL / Database Injection**:
  - Ensure all database queries utilize parameterized queries or SQLx prepared statements (`$1`, `$2`).
  - Verify that dynamic sorting, column names, or table names are strictly checked against compile-time allowlists.
- **Cross-Site Scripting (XSS)**:
  - Verify all user inputs (takes, counters, comments, usernames, bios, assembly titles, report descriptions) are encoded before HTML rendering.
  - In React, ensure zero usage of `dangerouslySetInnerHTML` unless passed through an allowlist-based sanitizer like DOMPurify.
  - In Rust/Backend, ensure output strings passed to RSS, ActivityPub, or JSON APIs are properly escaped.
  - Test with polyglot XSS vectors: `"><svg/onload=confirm(1)>`, `javascript:/*--></title></style></textarea></script></xmp><svg/onload='+/"/+/onmouseover=1/+/[*/[]/+alert(1)//'>`.
- **Command & Process Injection**:
  - Ensure no user input is ever passed to a shell (`sh -c`, `cmd.exe /c`, `powershell`). Use fixed argument arrays without shell interpolation.

### 2. Broken Object-Level Authorization (BOLA / IDOR)
- **Ownership Verification**:
  - On any mutation (`POST /api/posts/:id/counters`, `POST /api/reports`, `POST /api/assemblies/:id/vote`), verify whether the authenticated subject has legitimate authority to act upon the target object.
  - Ensure users cannot vote on behalf of other user IDs by simply altering JSON bodies or URL route parameters.
  - Verify that private or draft assemblies, unlisted proposals, or quarantined posts cannot be accessed by unauthorized IDs.

### 3. Server-Side Request Forgery (SSRF)
- **Network Boundary Protection**:
  - Remote image, media, or webhook URLs must be validated before the server initiates an outbound HTTP request.
  - Block loopback ranges: `127.0.0.0/8`, `::1`, `localhost`.
  - Block RFC1918 private subnets: `10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`.
  - Block link-local and cloud metadata endpoints: `169.254.169.254` (AWS, GCP, Azure, DigitalOcean instance metadata).
  - Enforce explicit scheme allowlists: only `http` or `https`.
  - Prevent DNS rebinding attacks by validating resolved IP addresses post-lookup or pinning DNS resolutions.

### 4. Authentication, Sessions & JWT Security
- **Token Integrity & Validation**:
  - Verify JWT expiration (`exp`), not-before (`nbf`), and issuer (`iss`).
  - Enforce strict signature algorithm verification (e.g., reject algorithm confusion attacks such as `alg: "none"` or HMAC with RSA public keys).
  - Verify secret key entropy: JWT signing keys and password salts must have sufficient cryptographic entropy ($\ge 256$ bits).
- **Password & Credential Storage**:
  - Use modern, memory-hard key derivation functions (Argon2id, PBKDF2 with $\ge 100,000$ iterations) with unique per-user salts.
  - Ensure passwords, salts, and JWT secrets are never logged in plaintext, printed in tracing output, or serialized into public API responses.
- **Cookie Security**:
  - Sensitive authentication cookies (such as refresh tokens) must be marked with `HttpOnly; Secure; SameSite=Strict` (or `Lax`).

### 5. Media Upload & Magic Bytes Inspection
- **File Upload Defenses**:
  - Do not trust user-supplied `Content-Type` headers or file extensions (`.png`, `.jpg`).
  - Inspect magic bytes (first 8 to 16 bytes of file buffers) to confirm legitimate image/media formats (PNG: `89 50 4E 47`, JPEG: `FF D8 FF`, WebP: `52 49 46 46 ... 57 45 42 50`).
  - Block executable scripts, SVG with embedded `<script>` or `<foreignObject>` tags, polyglot PDFs, or macro-enabled documents.
  - Enforce maximum payload size limits (e.g., 5 MB limit in Axum request body).

### 6. Rate Limiting, Brute Force & Anti-Sybil
- **Resource Exhaustion & Flood Defense**:
  - Verify rate limiters protect brute-force targets: `/api/auth/login`, `/api/auth/register`, `/api/assemblies/:id/join`, `/api/reports`.
  - Check for sliding window or token bucket implementations in Redis/memory.
  - Ensure failed authentication attempts trigger progressive delays or lockouts.
  - Verify Sybil damping formulas: ensure coordinated sockpuppet or cartel voting clusters have their influence mathematically mitigated.

### 7. HTTP Security Headers & Web Posture
- Check that the server emits hardened response headers:
  - `Content-Security-Policy`: Restricts allowed sources of scripts, styles, objects, and connect endpoints.
  - `X-Content-Type-Options: nosniff`: Prevents MIME-type sniffing.
  - `X-Frame-Options: DENY` (or `SAMEORIGIN`): Prevents clickjacking attacks.
  - `Strict-Transport-Security`: Enforces TLS on all subsequent connections.
  - `Referrer-Policy: strict-origin-when-cross-origin`: Prevents leakage of internal URL paths or tokens.
  - `Permissions-Policy`: Restricts camera, microphone, geolocation if unused.

---

## 🛠️ Step-by-Step Security Audit Workflow

When asked to audit security:

1. **Run Automated Security Scanner**:
   Execute the automated security auditing script in PowerShell:
   ```powershell
   ./scripts/audit_security.ps1
   ```
2. **Static Code Analysis**:
   - Backend: Search for `unsafe`, raw SQL strings, unhandled `unwrap()`, unconstrained body streams, or SSRF gaps.
   - Frontend: Search for `dangerouslySetInnerHTML`, insecure `localStorage` storage of sensitive tokens, or unescaped dynamic attributes (`href={userInput}`).
3. **Live Endpoint Security Verification**:
   - Query the running backend server (`http://127.0.0.1:4000/api/health` or other routes) to verify security headers and rate limiting.
4. **Review Checklists**:
   Consult [Security Checklist](./references/security_checklist.md) and [Vulnerability Catalog](./references/vulnerability_catalog.md).
5. **Generate Audit Report**:
   Produce a structured report categorizing findings by CVSS / OWASP severity with actionable diffs.

---

## 📋 Security Audit Report Template

```markdown
# 🛡️ Security Audit & Vulnerability Assessment Report

## 🎯 Executive Summary
- **Overall Posture**: [ SECURE | ATTENTION NEEDED | CRITICAL VULNERABILITIES IDENTIFIED ]
- **Summary**: [Concise 2-3 sentence overview of findings]

## 🚨 Vulnerabilities & Attack Vectors

### 1. [VULN-01] Title of Vulnerability
- **Severity**: 🔴 Critical / 🟠 High / 🟡 Medium / 🔵 Low / ℹ️ Info
- **CWE / OWASP Category**: e.g., CWE-918 (SSRF) / OWASP A10:2021
- **Affected File/Route**: `[file.rs#L50-L75](file:///path/to/file.rs#L50-L75)`
- **Vulnerability Description**: [Detailed explanation of the flaw]
- **Exploit Scenario / Attack Vector**:
  ```bash
  curl -X POST http://localhost:4000/api/... -d '{"url":"http://169.254.169.254/latest/meta-data/"}'
  ```
- **Impact & Blast Radius**: [Confidentiality, Integrity, Availability impact]
- **Remediation & Hardened Code**:
  ```diff
  - vulnerable_code()
  + secure_hardened_code()
  ```

## 🔒 Security Posture Checklist Verification
- [x] OWASP Top 10 Injection & XSS Mitigations Verified
- [x] SSRF Loopback & RFC1918 Private IP Range Blocking Verified
- [x] BOLA/IDOR Ownership Checks Verified
- [x] HTTP Security Headers Verified
- [x] Rate Limiting & DoS Defenses Verified
- [x] Password Hashing & Secret Entropy Verified
```
