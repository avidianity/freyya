# merged-result

**Invariant**:
The whole-repository checks run on the tree that a merge will produce, right before each merge.

Kit spec: [checks/merged-result.md](https://github.com/avidianity/factory/blob/v0.9.0/checks/merged-result.md).

## Implemented by

- Command: `cargo xtask repo-checks`.
  It runs every whole-repository check: `agent-entry-links`, `skills-link`, `skills-in-owner-guide`, `doc-size`, `doc-index`, `doc-links`, `no-placeholders`, `doc-style`, `record-numbers`, `no-suppressions`, `test-scope`, `dependency-pinning`, and `no-dead-code`.
- Follow the steps in [git.md](../agents/git.md#mandatory-check-the-merged-result-before-a-merge).

Runs in: the merge step of a request to land, and in CI.
