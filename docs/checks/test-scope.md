# test-scope

**Invariant**:
Every test file in the application test suite lives under the Project's declared application test paths.
No test file in the application test suite targets a repository root config file, a declarative tool config file, or a low-risk developer or repository script.
Automated self-tests for custom check scripts and high-risk operational scripts (such as deployment, backup, or data-migration scripts) live in a separate non-application test path that runs when those scripts, their check or policy configuration, or lockfile dependencies change, never inside the application Feedback loops.
When the Project uses coverage, complexity-coverage, or mutation tools, their configuration collects only an explicit allow-list inside the declared application source paths and excludes config files, repository scripts, generated files, migrations, type-only files, thin entry points, and test helpers.

Kit spec: [checks/test-scope.md](https://github.com/avidianity/factory/blob/v0.9.0/checks/test-scope.md).

## Implemented by

- Command: `cargo xtask test-scope`.
- Scope file: `policy/layout.toml`.
  Application tests live in `#[cfg(test)]` modules under `crates/*/src/**` and in `crates/*/tests/**`; check-script tests live in `xtask/**`; see [ADR 0002](../adr/0002-rust-test-layout.md).
- An application test may not name a tool config or repository file, such as `Cargo.toml` or `lefthook.yml`.
- The Project has no coverage or mutation tool; the check fails when a coverage config appears.
- The `xtask` tests run in CI when `xtask/`, `policy/`, or `Cargo.lock` change, outside the application Feedback loops.

Runs in: hook and CI.
