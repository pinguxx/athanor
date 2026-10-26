## Description
Briefly describe the purpose of this PR and what changes were made.

## Related Issues
Closes #[issue number]

## Checklist
- [ ] Code follows Athanor architectural tenets (Zero Lifetimes, `'static + Send + Sync`).
- [ ] No `.unwrap()` or `.expect()` calls in non-test crate sources.
- [ ] Added unit and integration tests under `crates/[crate_name]/tests/`.
- [ ] `cargo test --workspace` passes with 0 failures.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes with 0 warnings.
- [ ] `cargo fmt --all -- --check` passes cleanly.
- [ ] Documentation updated in `crates/[crate_name]/README.md` and inline docstrings.
