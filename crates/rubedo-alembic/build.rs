//! Build script for `rubedo-alembic`.
//!
//! On Windows x86_64 MSVC targets, ensures that Wasmtime 25's C longjmp helper
//! clears the frame pointer to bypass Windows `RtlUnwindEx` security cookie aborts
//! during trap and fuel exhaustion handling.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    #[cfg(target_os = "windows")]
    patch_wasmtime_longjmp_for_windows();
}

#[cfg(target_os = "windows")]
fn patch_wasmtime_longjmp_for_windows() {
    let cargo_home = std::env::var("CARGO_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            let user_profile = std::env::var("USERPROFILE").unwrap_or_default();
            std::path::PathBuf::from(user_profile).join(".cargo")
        });

    let src_dir = cargo_home.join("registry").join("src");
    if !src_dir.exists() {
        return;
    }

    if let Ok(entries) = std::fs::read_dir(src_dir) {
        for entry in entries.flatten() {
            if entry.file_name().to_string_lossy().starts_with("index.crates.io") {
                if let Ok(crates) = std::fs::read_dir(entry.path()) {
                    for crate_dir in crates.flatten() {
                        let name = crate_dir.file_name().to_string_lossy().to_string();
                        if name.starts_with("wasmtime-25.") {
                            let helpers_c = crate_dir.path().join("src").join("runtime").join("vm").join("helpers.c");
                            if helpers_c.exists() {
                                if let Ok(content) = std::fs::read_to_string(&helpers_c) {
                                    if !content.contains("*(uint64_t*)&buf = 0;") {
                                        let target = "  if (platform_setjmp(buf) != 0) {\n    return 0;\n  }\n";
                                        let patch = "  if (platform_setjmp(buf) != 0) {\n    return 0;\n  }\n#if defined(CFG_TARGET_OS_windows) && defined(CFG_TARGET_ARCH_x86_64)\n  *(uint64_t*)&buf = 0;\n#endif\n";
                                        let patched = content.replace(target, patch);
                                        let _ = std::fs::write(&helpers_c, patched);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
