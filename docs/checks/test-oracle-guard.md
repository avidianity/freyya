# test-oracle-guard

**Invariant**:
A pull request that deletes a test, skips a test, removes an assertion, adds a lint or type suppression, raises a test timeout, or changes CI, policy, or check configuration is at least Tier 3.
Policy and check configuration includes the tier map, the active Autonomy gate file, the Feedback budget file, the test-scope configuration, the comment allow-list, the duplicate-code thresholds and allow-list, the Standing Decisions ADR, `.rules`, task pages under `docs/agents/`, and domain `RULES.md` files.

Kit spec: [checks/test-oracle-guard.md](https://github.com/avidianity/factory/blob/v0.9.0/checks/test-oracle-guard.md).

## Implemented by

- Part of `cargo xtask risk-tier`; it compares the base and head versions of each changed Rust file with the `syn` parser.
- Tier 3 signals: a deleted test function or test file, a new `#[ignore]`, fewer assertion macros, or a new lint suppression.
- The tier map makes every protected policy path Tier 3: `.rules`, `docs/agents/`, `docs/checks/`, `policy/`, `xtask/`, `.github/`, `lefthook.yml`, the lint and toolchain files, and the Review mode ADR.

Runs in: CI, from the base revision, as an input to [risk-tier](./risk-tier.md).
