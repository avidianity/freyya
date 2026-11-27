# no-suppressions

**Invariant**:
No source file holds an inline directive that turns off a lint or type rule, except in a list of approved exceptions that changes only at Tier 3.

Kit spec: [checks/no-suppressions.md](https://github.com/avidianity/factory/blob/v0.9.0/checks/no-suppressions.md).

## Implemented by

- Command: `cargo xtask no-suppressions`.
- It finds `allow`, `expect`, and `warn` attributes, including inside `cfg_attr`, with the `syn` parser.
- Approved exceptions: `policy/suppressions.toml` (empty).
- The whole-Project lint levels live in `[workspace.lints]` in `Cargo.toml`; change them only through an ADR.

Runs in: hook and CI.
