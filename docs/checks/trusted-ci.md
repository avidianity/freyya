# trusted-ci

**Invariant**:
Policy-enforcing CI checks execute their code and read their policy files from the protected base revision, never from the pull request head.
Policy-enforcing checks include [risk-tier](./risk-tier.md), [test-oracle-guard](./test-oracle-guard.md), and [evidence-bundle](./evidence-bundle.md).
Policy files include the tier map, Feedback budgets, allow-lists, and check configuration.

Those checks read the pull request's changed file list and diff strictly as untrusted data.
They never execute code or install dependencies from the pull request head in a privileged base-branch context.

Forge-enforced ownership and branch protection rules require approval from a non-agent Owner identity for any pull request that changes CI workflows, check scripts, policy files, or the Project's rules files.
Agent credentials cannot approve pull requests or bypass branch protection.

Kit spec: [checks/trusted-ci.md](https://github.com/avidianity/factory/blob/v0.9.0/checks/trusted-ci.md).

## Implemented by

- `.github/workflows/policy.yml` runs on `pull_request_target`.
  It checks out the protected base revision, builds `xtask` from it, and fetches the pull request head only as data: it never builds, runs, or installs anything from the head.
- `.github/CODEOWNERS` names the Owner for every protected path.
- Branch protection on `main` requires a pull request and the required checks, for administrators too.

Runs in: CI, on every pull request event.

Known gap: one GitHub account serves the Owner and agents, so GitHub cannot require a non-agent approval; see [ADR 0004](../adr/0004-public-repo-single-account.md).
