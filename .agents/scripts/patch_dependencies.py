#!/usr/bin/env python3
"""
Patch upstream dependencies that hit compiler recursion/depth limits or runtime bugs.
- lance-0.17.0: Fixes layout computation query depth limit overflow in rustc.
- hf-hub-0.3.2: Fixes RelativeUrlWithoutBase on HuggingFace 302/307 redirects in sync client.
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

    for path in matches:
        try:
            with open(path, 'r', encoding='utf-8') as f:
                content = f.read()
            if '#![recursion_limit' not in content:
                with open(path, 'w', encoding='utf-8') as f:
                    f.write('#![recursion_limit = "512"]\n' + content)
                print(f"[PATCH] Injected #![recursion_limit = \"512\"] into {path}")
            else:
                print(f"[OK] {path} already has recursion_limit defined.")
        except Exception as e:
            print(f"[ERROR] Failed to patch {path}: {e}", file=sys.stderr)
            return 1

    return 0

def patch_hf_hub():
    cargo_home = os.environ.get('CARGO_HOME') or os.path.expanduser('~/.cargo')
    pattern = os.path.join(cargo_home, 'registry', 'src', '**', 'hf-hub-0.3.2', 'src', 'api', 'sync.rs')
    matches = glob.glob(pattern, recursive=True)
    if not matches:
        print(f"[INFO] hf-hub-0.3.2 not found in {cargo_home}. Run 'cargo fetch' first if compiling.")
        return 0

    for path in matches:
        try:
            with open(path, 'r', encoding='utf-8') as f:
                content = f.read()

            modified = False

            # Hunk 1: Disable automatic redirect following on client so manual redirect logic handles it
            target_builder = "let client = HeaderAgent::new(ureq::builder().build(), headers.clone());"
            replacement_builder = "let client = HeaderAgent::new(ureq::builder().redirects(0).build(), headers.clone());"
            if target_builder in content:
                content = content.replace(target_builder, replacement_builder)
                modified = True

            # Hunk 2: Handle relative Location headers in metadata()
            target_meta = """            self.client
                .get(response.header(LOCATION).unwrap())
                .set(RANGE, "bytes=0-0")
                .call()
                .map_err(Box::new)?"""

            replacement_meta = """            let loc = response.header(LOCATION).unwrap();
            let loc_url = if loc.starts_with('/') {
                format!("https://huggingface.co{loc}")
            } else if !loc.starts_with("http") {
                format!("{}/{loc}", self.endpoint)
            } else {
                loc.to_string()
            };
            self.client
                .get(&loc_url)
                .set(RANGE, "bytes=0-0")
                .call()
                .map_err(Box::new)?"""

            if target_meta in content:
                content = content.replace(target_meta, replacement_meta)
                modified = True

            # Hunk 3: Handle relative Location headers in download_tempfile()
            target_down = "let response = self.client.get(url).call().map_err(Box::new)?;"
            replacement_down = """let mut curr_url = url.to_string();
        let mut response = self.client.get(&curr_url).call().map_err(Box::new)?;
        while (300..400).contains(&response.status()) {
            if let Some(loc) = response.header(LOCATION) {
                curr_url = if loc.starts_with('/') {
                    format!("https://huggingface.co{loc}")
                } else {
                    loc.to_string()
                };
                response = self.client.get(&curr_url).call().map_err(Box::new)?;
            } else {
                break;
            }
        }"""

            if target_down in content:
                content = content.replace(target_down, replacement_down)
                modified = True

            if modified:
                with open(path, 'w', encoding='utf-8') as f:
                    f.write(content)
                print(f"[PATCH] Patched redirect handling in {path}")
            else:
                print(f"[OK] {path} already patched with redirect handling.")
        except Exception as e:
            print(f"[ERROR] Failed to patch {path}: {e}", file=sys.stderr)
            return 1

    return 0

if __name__ == '__main__':
    rc1 = patch_lance()
    if rc1 != 0:
        sys.exit(rc1)
    rc2 = patch_hf_hub()
    if rc2 != 0:
        sys.exit(rc2)
    sys.exit(0)
