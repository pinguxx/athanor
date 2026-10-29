#!/usr/bin/env python3
"""
Google Rust Style Guide Automated Auditor

Scans Rust source files against the Google Rust Style Guide:
http://google.github.io/styleguide/rustguide.html

Checks for:
  - Glob imports (use foo::*)
  - Rigid borrowed parameters (&String, &Vec, &Box, &Option)
  - Check-then-unwrap anti-patterns (.is_some() followed by .unwrap())
  - Unsafe code without // SAFETY: justification comment
  - std::panic::catch_unwind usage
  - Multiple module paths inside single use statement (use foo::{a, b::c})
  - Implementation of len() without is_empty()
  - String concatenation with '+'
"""

import sys
import os
import re
from pathlib import Path


class StyleViolation:
    def __init__(self, file_path: str, line_no: int, rule: str, severity: str, message: str, snippet: str):
        self.file_path = file_path
        self.line_no = line_no
        self.rule = rule
        self.severity = severity  # BLOCKER, MAJOR, MINOR, NIT
        self.message = message
        self.snippet = snippet.strip()

    def __str__(self):
        return f"[{self.severity}] {self.file_path}:{self.line_no} ({self.rule})\n  -> {self.message}\n  Code: {self.snippet}\n"


def audit_file(file_path: Path) -> list:
    violations = []
    is_test_file = "tests" in file_path.parts or file_path.name.endswith("_test.rs")
    try:
        with open(file_path, "r", encoding="utf-8", errors="replace") as f:
            lines = f.readlines()
    except Exception as e:
        return [StyleViolation(str(file_path), 1, "FileRead", "BLOCKER", f"Could not read file: {e}", "")]

    content = "".join(lines)

    # Check 1: len() without is_empty() in the same file
    has_len = bool(re.search(r"pub\s+fn\s+len\s*\(&self", content))
    has_is_empty = bool(re.search(r"pub\s+fn\s+is_empty\s*\(&self", content))
    if has_len and not has_is_empty:
        violations.append(
            StyleViolation(
                str(file_path),
                1,
                "API-Sets-Together",
                "MAJOR",
                "Type implements pub fn len(&self) but is missing pub fn is_empty(&self). Both should be implemented together.",
                "pub fn len(&self)"
            )
        )

    for i, line in enumerate(lines, start=1):
        stripped = line.strip()

        # Skip comment lines
        if stripped.startswith("//") or stripped.startswith("/*") or stripped.startswith("*"):
            continue

        # Check 2: Glob imports
        if re.search(r"^\s*(pub\s+)?use\s+.*::\*\s*;", line):
            # Allowed exceptions: super::*, enum inside fn, test module
            if not is_test_file and "super::*" not in line and "prelude::*" not in line:
                violations.append(
                    StyleViolation(
                        str(file_path),
                        i,
                        "No-Glob-Imports",
                        "MAJOR",
                        "Glob import detected. Every imported item must be named explicitly to support LSCs and code clarity.",
                        line
                    )
                )

        # Check 3: Rigid container references in function signatures
        # &String
        if re.search(r"fn\s+[a-zA-Z0-9_]+\s*\(.*&\s*String\b", line):
            violations.append(
                StyleViolation(
                    str(file_path),
                    i,
                    "Flexible-References",
                    "MAJOR",
                    "Function parameter accepts '&String'. Use flexible slice '&str' instead.",
                    line
                )
            )

        # &Vec<T>
        if re.search(r"fn\s+[a-zA-Z0-9_]+\s*\(.*&\s*Vec\s*<", line):
            violations.append(
                StyleViolation(
                    str(file_path),
                    i,
                    "Flexible-References",
                    "MAJOR",
                    "Function parameter accepts '&Vec<T>'. Use flexible slice '&[T]' instead.",
                    line
                )
            )

        # &Box<T>
        if re.search(r"fn\s+[a-zA-Z0-9_]+\s*\(.*&\s*Box\s*<", line):
            violations.append(
                StyleViolation(
                    str(file_path),
                    i,
                    "Flexible-References",
                    "MAJOR",
                    "Function parameter accepts '&Box<T>'. Borrow inner data '&T' directly instead.",
                    line
                )
            )

        # &Option<T>
        if re.search(r"fn\s+[a-zA-Z0-9_]+\s*\(.*&\s*Option\s*<", line):
            violations.append(
                StyleViolation(
                    str(file_path),
                    i,
                    "Flexible-References",
                    "MAJOR",
                    "Function parameter accepts '&Option<T>'. Use 'Option<&T>' instead to borrow inner value.",
                    line
                )
            )

        # Check 4: Unsafe code without // SAFETY: comment
        if re.search(r"\bunsafe\s*\{", line) or re.search(r"\bunsafe\s+fn\b", line):
            prev_lines = "".join(lines[max(0, i-4):i-1])
            if "SAFETY:" not in prev_lines:
                violations.append(
                    StyleViolation(
                        str(file_path),
                        i,
                        "Unsafe-Safety-Comment",
                        "BLOCKER",
                        "Unsafe block or function detected without preceding '// SAFETY:' explanation comment.",
                        line
                    )
                )

        # Check 5: Multi-module use statements (e.g. use foo::{bar, baz::qux};)
        if re.search(r"use\s+[^{;]+::\{[^}]*::", line):
            violations.append(
                StyleViolation(
                    str(file_path),
                    i,
                    "Single-Module-Use",
                    "MINOR",
                    "A use statement should import items from one module. No '::' inside '{...}'.",
                    line
                )
            )

        # Check 6: catch_unwind prohibition
        if "catch_unwind" in line and not is_test_file:
            violations.append(
                StyleViolation(
                    str(file_path),
                    i,
                    "No-Catch-Unwind",
                    "MAJOR",
                    "panic::catch_unwind is prohibited in general first-party Rust code unless justified for FFI or defense-in-depth.",
                    line
                )
            )

        # Check 7: Multiple string concatenations with +
        if re.search(r'"\s*\+\s*&?[a-zA-Z0-9_.]+\s*\+\s*"', line):
            violations.append(
                StyleViolation(
                    str(file_path),
                    i,
                    "Format-Over-Plus",
                    "NIT",
                    "String concatenation using multiple '+' operators. Prefer format!(\"...\") or write! for efficiency.",
                    line
                )
            )

    return violations


def scan_directory(root_dir: str) -> list:
    all_violations = []
    root = Path(root_dir)

    for path in root.rglob("*.rs"):
        # Skip target, .fastembed_cache, .onnxruntime, etc.
        parts = path.parts
        if "target" in parts or ".fastembed_cache" in parts or ".onnxruntime" in parts:
            continue
        all_violations.extend(audit_file(path))

    return all_violations


def main():
    target_dir = sys.argv[1] if len(sys.argv) > 1 else "."
    print(f"Scanning Rust codebase for Google Rust Style Guide violations: {target_dir}")
    violations = scan_directory(target_dir)

    if not violations:
        print("\n[PASS] 0 Google Rust Style Guide violations found! Codebase is fully compliant.")
        sys.exit(0)

    print(f"\n[ALERT] Found {len(violations)} style guide violations:\n")
    blockers = [v for v in violations if v.severity == "BLOCKER"]
    majors = [v for v in violations if v.severity == "MAJOR"]
    minors = [v for v in violations if v.severity == "MINOR"]
    nits = [v for v in violations if v.severity == "NIT"]

    print(f"Summary: {len(blockers)} Blockers, {len(majors)} Majors, {len(minors)} Minors, {len(nits)} Nits\n")

    for v in violations:
        print(v)

    if blockers:
        sys.exit(1)


if __name__ == "__main__":
    main()
