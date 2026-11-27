---
status: accepted
---

# Tests follow the Rust layout

The Owner settled this stack-convention conflict during Bootstrap on 2026-11-28.

## Context

The Kit checks [test-scope](../checks/test-scope.md) and [behavior-has-test](../checks/behavior-has-test.md) assume that tests live in separate test files.
Rust puts unit tests in a `#[cfg(test)]` module at the bottom of the source file they test.
Rust puts integration tests, which see only the public API, in a crate's `tests/` folder.
Forcing every test into separate files would make internal items public only so that tests can reach them.

## Decision

**Follow the Rust layout.**

- Unit tests live in `#[cfg(test)]` modules inside `crates/*/src/**`.
- Integration tests live in `crates/*/tests/**`.
- Check-script tests live in `xtask/**`, outside the application suite.
- `test-scope` accepts a test inside an application source file.
- `behavior-has-test` counts a change as tested when it adds or changes a `#[test]` function, found with a Rust syntax-tree parser (`syn`), or changes a file under `crates/*/tests/`.

The paths live in `policy/layout.toml`.

## Considered options

- **The Kit's separate-file layout.**
  Rejected: it fights Rust's privacy rules and widens public APIs.

## Consequences

- "Every behavior change has a test" stays as strict as the Kit requires.
- The checks parse Rust source with `syn`, never with regular expressions.
- A change to `policy/layout.toml` is Tier 3.
