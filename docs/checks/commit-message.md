# commit-message

**Invariant**:
Each commit message follows Conventional Commits with an allowed scope.
It holds no co-author line, no agent attribution, and no marker that tells CI to skip the commit.

Kit spec: [checks/commit-message.md](https://github.com/avidianity/factory/blob/v0.9.0/checks/commit-message.md).

## Implemented by

- Command: `cargo xtask commit-message --file <FILE>` in the hook, and `cargo xtask commit-message --range <BASE>..<HEAD>` in CI.
- Types and scopes live in `policy/layout.toml`.
- Merge commits are skipped.

Runs in: the commit-message hook, and CI for every commit in a pull request.
