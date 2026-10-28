#!/usr/bin/env python3
"""
Patch upstream dependencies that hit compiler recursion/depth limits.
Specifically fixes lance-0.17.0 layout computation overflow in rustc.
"""

import glob
import os
import sys

def patch_lance():
    cargo_home = os.environ.get('CARGO_HOME') or os.path.expanduser('~/.cargo')
    pattern = os.path.join(cargo_home, 'registry', 'src', '**', 'lance-0.17.0', 'src', 'lib.rs')
    matches = glob.glob(pattern, recursive=True)
    if not matches:
        print(f"[INFO] lance-0.17.0 not found in {cargo_home}. Run 'cargo fetch' first if compiling.")
        return 0

    patched_count = 0
    for path in matches:
        try:
            with open(path, 'r', encoding='utf-8') as f:
                content = f.read()
            if '#![recursion_limit' not in content:
                with open(path, 'w', encoding='utf-8') as f:
                    f.write('#![recursion_limit = "512"]\n' + content)
                print(f"[PATCH] Injected #![recursion_limit = \"512\"] into {path}")
                patched_count += 1
            else:
                print(f"[OK] {path} already has recursion_limit defined.")
        except Exception as e:
            print(f"[ERROR] Failed to patch {path}: {e}", file=sys.stderr)
            return 1

    return 0

if __name__ == '__main__':
    sys.exit(patch_lance())
