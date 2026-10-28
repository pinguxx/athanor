#!/usr/bin/env python3
"""
Download and configure ONNX Runtime binary for Athanor's RAG test suite.
Sets ORT_DYLIB_PATH for fastembed / ort-load-dynamic in GitHub Actions CI.
"""

import os
import sys
import platform
import urllib.request
import tarfile
import zipfile
import shutil

VERSION = "1.18.1"

def setup_onnxruntime():
    repo_root = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
    dest_dir = os.path.join(repo_root, ".onnxruntime")
    os.makedirs(dest_dir, exist_ok=True)

    plat = sys.platform
    arch = platform.machine().lower()

    if plat.startswith("linux"):
        asset = f"onnxruntime-linux-x64-{VERSION}.tgz"
        lib_name = "libonnxruntime.so"
    elif plat == "darwin":
        if "arm" in arch or "aarch" in arch:
            asset = f"onnxruntime-osx-arm64-{VERSION}.tgz"
        else:
            asset = f"onnxruntime-osx-x64-{VERSION}.tgz"
        lib_name = "libonnxruntime.dylib"
    elif plat in ("win32", "cygwin"):
        asset = f"onnxruntime-win-x64-{VERSION}.zip"
        lib_name = "onnxruntime.dll"
    else:
        print(f"[ERROR] Unsupported platform: {plat}", file=sys.stderr)
        return 1

    extracted_dir = os.path.join(dest_dir, asset.replace(".tgz", "").replace(".zip", ""))
    target_lib = os.path.join(extracted_dir, "lib", lib_name)

    if not os.path.exists(target_lib):
        archive_path = os.path.join(dest_dir, asset)
        url = f"https://github.com/microsoft/onnxruntime/releases/download/v{VERSION}/{asset}"
        print(f"[INFO] Downloading ONNX Runtime from {url}...")
        urllib.request.urlretrieve(url, archive_path)
        print(f"[INFO] Extracting {archive_path}...")

        if asset.endswith(".tgz"):
            with tarfile.open(archive_path, "r:gz") as tar:
                tar.extractall(path=dest_dir)
        elif asset.endswith(".zip"):
            with zipfile.ZipFile(archive_path, "r") as z:
                z.extractall(path=dest_dir)

    # Locate the library file
    found_lib = None
    for root, _, files in os.walk(extracted_dir):
        if lib_name in files:
            found_lib = os.path.join(root, lib_name)
            break
        # On Linux, libonnxruntime.so might be a symlink or named libonnxruntime.so.1.18.1
        for f in files:
            if f.startswith(lib_name):
                found_lib = os.path.join(root, f)
                break
        if found_lib:
            break

    if not found_lib or not os.path.exists(found_lib):
        print(f"[ERROR] Could not find {lib_name} in {extracted_dir}", file=sys.stderr)
        return 1

    lib_path = os.path.abspath(found_lib)
    lib_dir = os.path.dirname(lib_path)
    print(f"[SUCCESS] ONNX Runtime library located at: {lib_path}")

    # Copy to target/debug and target/debug/deps if target directory exists
    for sub in ["", "deps"]:
        t_dir = os.path.join(repo_root, "target", "debug", sub)
        try:
            os.makedirs(t_dir, exist_ok=True)
            shutil.copy2(lib_path, os.path.join(t_dir, lib_name))
            print(f"[INFO] Copied {lib_name} to {t_dir}")
        except Exception as e:
            print(f"[WARN] Could not copy to {t_dir}: {e}")

    # Export environment variables for GitHub Actions
    github_env = os.environ.get("GITHUB_ENV")
    if github_env:
        with open(github_env, "a", encoding="utf-8") as f:
            f.write(f"ORT_DYLIB_PATH={lib_path}\n")
            if plat.startswith("linux"):
                f.write(f"LD_LIBRARY_PATH={lib_dir}:{os.environ.get('LD_LIBRARY_PATH', '')}\n")
            elif plat == "darwin":
                f.write(f"DYLD_LIBRARY_PATH={lib_dir}:{os.environ.get('DYLD_LIBRARY_PATH', '')}\n")
        print(f"[ENV] Written ORT_DYLIB_PATH to GITHUB_ENV: {lib_path}")

    github_path = os.environ.get("GITHUB_PATH")
    if github_path:
        with open(github_path, "a", encoding="utf-8") as f:
            f.write(f"{lib_dir}\n")
        print(f"[PATH] Written {lib_dir} to GITHUB_PATH")

    return 0

if __name__ == "__main__":
    sys.exit(setup_onnxruntime())
