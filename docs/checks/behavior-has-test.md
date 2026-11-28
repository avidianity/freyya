# behavior-has-test

**Invariant**:
A pull request that changes files under the Project's declared application source paths also changes or adds a test file under the declared application test paths, or its body says "No behavior change" with a reason.
Changes outside declared application source paths, including root or tooling configuration files, repository and development scripts, generated files, and documentation, do not require a test file.

Kit spec: [checks/behavior-has-test.md](https://github.com/avidianity/factory/blob/v0.9.0/checks/behavior-has-test.md).

## Implemented by

- Command: `cargo xtask behavior-has-test --base <BASE-REV> --head <HEAD-REV> --body-file <FILE>`.
- Application source paths: `crates/*/src/**`, from `policy/layout.toml`.
- A "No behavior change" line counts only in visible text, not inside an HTML comment.
- A change counts as tested when it adds or changes a `#[test]` function in a changed Rust file under `crates/*/src/` or `crates/*/tests/`, found with `syn`; any other file under `tests/` does not count; see [ADR 0002](../adr/0002-rust-test-layout.md).

Runs in: CI, in `.github/workflows/policy.yml`.
