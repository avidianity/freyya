# risk-tier

**Invariant**:
Every pull request gets a Tier from 0 to 4, computed deterministically from three inputs against the protected base revision (see [trusted-ci](./trusted-ci.md)):

1. **Changed paths**: each changed path matched against the Project's tier map on the base revision.
   A path in no list is Tier 2.
2. **Reverse dependencies**: any changed file that a Tier 3 or Tier 4 path transitively imports or depends on inherits that Tier, computed from the Project's dependency graph.
   When a stack lacks a static dependency graph, mark shared foundation folders Tier 3 in the tier map.
3. **Diff signals**: dependency manifest or lockfile changes are at least Tier 2; schema migrations, new external network endpoints, secret reads, and authentication or authorization guards outside known risk folders are at least Tier 3; bulk data change scripts and irreversible data-dropping operations (such as dropping populated tables or columns or truncating tables) are Tier 4.
   [test-oracle-guard](./test-oracle-guard.md) adds Tier 3 signals for weakened tests, checks, or policy files.

The highest Tier across all three inputs applies.
The check applies the `tier-<N>` label and replaces any `tier-*` label that someone else set.
When the Project uses Autonomy gates, this check (or a companion required check reading the protected base revision) compares the computed Tier against the Project's active gate policy file and blocks merge without the required approvals whenever the Tier exceeds the active gate.
Never gate the Tier on an agent or a language model; semantic impact analysis belongs in `blast-radius` from Tier 2 onward.

Kit spec: [checks/risk-tier.md](https://github.com/avidianity/factory/blob/v0.9.0/checks/risk-tier.md).

## Implemented by

- Command: `cargo xtask risk-tier --base <BASE-REV> --head <HEAD-REV>`.
  It prints the Tier and each path, rule, or signal that set it.
- Tier map: `policy/tier-map.toml`, read from the base revision.
  The highest matching rule wins; a path that matches no rule is Tier 2.
- Reverse dependencies: the workspace crate graph from `cargo metadata`.
  A change inside a crate that a Tier 3 or 4 crate depends on takes that Tier.
  Put each risk area in its own crate and list the crate folder in the tier map, so that the graph sees it.
- Diff signals, found with the `syn` Rust parser by comparing the base and head versions of each changed file:
  a dependency manifest or lockfile change (Tier 2); a new `http`, `https`, or `wss` endpoint string, a new environment read, or a new file write or delete (Tier 3); a new `DELETE FROM`, `DROP TABLE`, or `TRUNCATE TABLE` string (Tier 4); and the [test-oracle-guard](./test-oracle-guard.md) signals.

Runs in: CI, in `.github/workflows/policy.yml`, which applies the `tier-<N>` label.
Agents also run it locally to read the Expected Tier of a change before they open a pull request.
The active Autonomy gate is `policy/gate.toml` (Gate 0).
