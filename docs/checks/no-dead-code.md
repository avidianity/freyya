# no-dead-code

**Invariant**:
A pull request adds no dead symbol.
A dead symbol is a file, function, type, constant, export, or dependency that nothing reachable from a Project entry point uses.
The exported API of a published package is an entry point.
A symbol in application code that only tests use is dead.

The check compares the dead symbols on the pull request head with those on the base branch.
It fails when the head has a dead symbol that the base branch lacks, wherever that symbol is.

Kit spec: [checks/no-dead-code.md](https://github.com/avidianity/factory/blob/v0.9.0/checks/no-dead-code.md).

## Implemented by

- The compiler: `[workspace.lints]` in `Cargo.toml` sets `dead_code`, `unused`, and all warnings to deny.
- Command: `cargo clippy --workspace --locked` without test targets, so a symbol that only tests use is dead.
- Open: unused dependencies and orphan files that no module declares have no tool yet.

Runs in: CI.
