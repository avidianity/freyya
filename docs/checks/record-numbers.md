# record-numbers

**Invariant**:
Sequential numbered records, such as ADRs and sequential migrations when the Project uses sequential migration numbers, have unique numbers with no gaps in their sequence.
For SQL migration identifier ordering (sequential or timestamped) and immutability of applied migrations, see [migration-history](https://github.com/avidianity/factory/blob/v0.9.0/checks/migration-history.md).

Kit spec: [checks/record-numbers.md](https://github.com/avidianity/factory/blob/v0.9.0/checks/record-numbers.md).

## Implemented by

- Command: `cargo xtask record-numbers`.
- It checks `docs/adr/<NNNN>-<SLUG>.md` for duplicate numbers and gaps.

Runs in: CI, and in the merged-result check.
