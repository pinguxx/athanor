# Security Policy

## Supported Versions

We provide security updates and critical fixes for the following versions of Athanor:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

---

## Reporting a Vulnerability

The Athanor project takes memory safety, runtime isolation, and agent boundary integrity seriously. If you discover a security vulnerability, execution escape, or memory safety bypass, please report it privately.

**Please DO NOT open a public GitHub issue.**

### Reporting Procedure
1. Send an email to **security@athanor-rs.org** (or use GitHub Private Vulnerability Reporting via the Security tab).
2. Include a detailed description of the vulnerability, including:
   - Impacted crate(s) (e.g. `athanor-alembic`, `athanor-mcp`).
   - Reproduction steps or proof-of-concept code.
   - Potential impact and affected configurations.
3. You will receive an initial response acknowledging your report within 48 hours.
4. We will coordinate a remediation release and give appropriate attribution in the release notes.

---

## Security Model & Guarantees

Athanor enforces strict execution containment:

- **WASM Sandboxing (`athanor-alembic`)**: Untrusted code executed in WebAssembly has zero access to the host filesystem, network sockets, environment variables, or system time.
- **Instruction Fuel Watchdogs**: Infinite loops or malicious iterations are deterministically halted upon fuel exhaustion in $<15\text{ ms}$.
- **Memory Limits**: Guest linear memory is constrained to a hard ceiling (default 64MB) to prevent host memory exhaustion (OOM).
- **Safe Rust**: All production crates forbid unsafe blocks (`#![forbid(unsafe_code)]` or `#![deny(unsafe_code)]`) with audited safety boundaries.
- **Zero Panic Tolerance**: Production crates strictly avoid `.unwrap()` and `.expect()` to prevent denial-of-service panics.
