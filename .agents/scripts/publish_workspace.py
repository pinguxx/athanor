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
import re
import email.utils
import datetime

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


def publish_crate(crate_name: str, dry_run: bool = False, max_attempts: int = 10) -> bool:
    cmd = ["cargo", "publish", "-p", crate_name]
    if dry_run:
        cmd.append("--dry-run")

    for attempt in range(1, max_attempts + 1):
        print(f"\n=======================================================")
        print(f"[{'DRY-RUN' if dry_run else 'PUBLISHING'}] {crate_name} (Attempt {attempt}/{max_attempts})")
        print(f"Command: {' '.join(cmd)}")
        print(f"=======================================================")

        proc = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
        print(proc.stdout, end="")

        if proc.returncode == 0:
            print(f"[SUCCESS] {crate_name} published successfully!")
            return True

        if "already exists on crates.io" in proc.stdout:
            print(f"[ALREADY PUBLISHED] {crate_name} is already live on crates.io. Proceeding.")
            return True

        if dry_run:
            return False

        # Check for crates.io 429 Too Many Requests rate limit
        if "429 Too Many Requests" in proc.stdout:
            match = re.search(r"try again after (.*?) and see", proc.stdout)
            if match:
                date_str = match.group(1).strip()
                try:
                    target_time = email.utils.parsedate_to_datetime(date_str)
                    now = datetime.datetime.now(datetime.timezone.utc)
                    wait_secs = max(5, int((target_time - now).total_seconds()) + 10)
                    print(f"\n[RATE-LIMIT] Crates.io new crate rate limit hit. Resets at {target_time} UTC.")
                    print(f"Sleeping {wait_secs}s until rate limit window resets...")
                    time.sleep(wait_secs)
                    continue
                except Exception as e:
                    print(f"Could not parse rate limit date '{date_str}': {e}")

            print("[RATE-LIMIT] 429 hit, sleeping default 300s...")
            time.sleep(300)
            continue

        if attempt < max_attempts:
            wait_time = 15 * attempt
            print(f"[RETRY] Attempt {attempt} failed for {crate_name}. Waiting {wait_time}s before retry...")
            time.sleep(wait_time)

    return False


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
