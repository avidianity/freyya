# behavior-has-test

**Invariant**:
A pull request that changes files under the Project's declared application source paths also changes or adds a test file under the declared application test paths, or its body says "No behavior change" with a reason.
Changes outside declared application source paths, including root or tooling configuration files, repository and development scripts, generated files, and documentation, do not require a test file.

Kit spec: [checks/behavior-has-test.md](https://github.com/avidianity/factory/blob/v0.9.0/checks/behavior-has-test.md).

## Implemented by

- Command: `cargo xtask behavior-has-test --base <BASE-REV> --head <HEAD-REV> --body-file <FILE>`.
- Application source paths: `crates/*/src/**`, from `policy/layout.toml`.
- A change counts as tested when it changes a file under `crates/*/tests/`, or adds or changes a `#[test]` function in a changed source file, found with `syn`; see [ADR 0002](../adr/0002-rust-test-layout.md).

Runs in: CI, in `.github/workflows/policy.yml`.
