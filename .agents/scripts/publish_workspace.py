#!/usr/bin/env python3
"""
Automated topological publishing script for the Athanor workspace crates to crates.io.

Order is determined by dependency graph:
  1. athanor-core (foundation: zero internal workspace dependencies)
  2. athanor-macros (proc-macro: dev-depends on athanor-core)
  3. athanor-crucible (depends on athanor-core)
  4. athanor-alembic (depends on athanor-core)
  5. athanor-mcp (depends on athanor-core)
  6. athanor-rag (depends on athanor-core)
  7. athanor-opus (depends on athanor-core, athanor-crucible)
  8. athanor-atelier (depends on athanor-core, athanor-opus, athanor-crucible)
  9. athanor-evals (depends on athanor-core, athanor-macros)
 10. athanor (umbrella crate: facade aggregating all sub-crates)
"""

import subprocess
import sys
import time
import os

PUBLISH_ORDER = [
    "athanor-core",
    "athanor-macros",
    "athanor-crucible",
    "athanor-alembic",
    "athanor-mcp",
    "athanor-rag",
    "athanor-opus",
    "athanor-atelier",
    "athanor-evals",
    "athanor",
]

INDEX_PROPAGATION_DELAY_SECS = 25


def check_auth():
    # Check credentials file or CARGO_REGISTRY_TOKEN
    cred_file_toml = os.path.expanduser("~/.cargo/credentials.toml")
    cred_file = os.path.expanduser("~/.cargo/credentials")
    has_token = (
        bool(os.environ.get("CARGO_REGISTRY_TOKEN"))
        or os.path.isfile(cred_file_toml)
        or os.path.isfile(cred_file)
    )
    return has_token


def publish_crate(crate_name: str, dry_run: bool = False) -> bool:
    cmd = ["cargo", "publish", "-p", crate_name]
    if dry_run:
        cmd.append("--dry-run")

    print(f"\n=======================================================")
    print(f"[{'DRY-RUN' if dry_run else 'PUBLISHING'}] {crate_name}")
    print(f"Command: {' '.join(cmd)}")
    print(f"=======================================================")

    res = subprocess.run(cmd)
    return res.returncode == 0


def main():
    dry_run = "--dry-run" in sys.argv

    if not dry_run and not check_auth():
        print("\n[ERROR] No crates.io credentials found!")
        print("Please authenticate using one of the following methods:")
        print("  1. Run: cargo login <your-crates-io-token>")
        print("  2. Or set: $env:CARGO_REGISTRY_TOKEN = \"<your-crates-io-token>\"")
        print("\nYou can generate a token at: https://crates.io/settings/tokens")
        sys.exit(1)

    print(f"Beginning topological publication of {len(PUBLISH_ORDER)} Athanor crates...")

    for i, crate in enumerate(PUBLISH_ORDER, start=1):
        print(f"\n({i}/{len(PUBLISH_ORDER)}) Preparing to publish: {crate}")
        success = publish_crate(crate, dry_run=dry_run)
        if not success:
            print(f"\n[ABORT] Publishing failed for {crate}. Halting pipeline.")
            sys.exit(1)

        if not dry_run and i < len(PUBLISH_ORDER):
            print(f"Waiting {INDEX_PROPAGATION_DELAY_SECS}s for crates.io index propagation...")
            time.sleep(INDEX_PROPAGATION_DELAY_SECS)

    print("\n=======================================================")
    print(f"SUCCESS! All {len(PUBLISH_ORDER)} Athanor crates published to crates.io!")
    print("=======================================================")


if __name__ == "__main__":
    main()
