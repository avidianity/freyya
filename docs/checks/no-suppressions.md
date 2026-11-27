# no-suppressions

**Invariant**:
No source file holds an inline directive that turns off a lint or type rule, except in a list of approved exceptions that changes only at Tier 3.

Kit spec: [checks/no-suppressions.md](https://github.com/avidianity/factory/blob/v0.9.0/checks/no-suppressions.md).

## Implemented by

- Command: `cargo xtask no-suppressions`.
- It finds `allow`, `expect`, and `warn` attributes at the token level, so it also sees them inside `cfg_attr` and inside macro bodies.
- It treats an item name with a leading underscore, such as `fn _unused()`, as a dead-code suppression; `const _` is allowed.
- Every member `Cargo.toml` must declare `[lints] workspace = true` and nothing else; `[workspace.lints]` must use only `deny` or `forbid`; `.cargo/config.toml` must pass no lint-weakening `rustflags`.
- Approved exceptions: `policy/suppressions.toml` (empty).
- The whole-Project lint levels live in `[workspace.lints]` in `Cargo.toml`; change them only through an ADR.

Runs in: hook and CI.
