---
status: accepted
---

# Public repository, one GitHub account for the Owner and agents

The Owner made both choices during Bootstrap on 2026-11-28.

## Context

The Kit's [trusted-ci](../checks/trusted-ci.md) check relies on forge branch protection, and on approval from a non-agent Owner identity for protected paths.
GitHub's docs, read on 2026-11-28, say that a GitHub Free personal account gets protected branches and rulesets only on public repositories ([about protected branches](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/about-protected-branches)).
Agents act through the Owner's own `gh` login, so GitHub cannot tell an agent from the Owner.
GitHub also never lets an account approve its own pull request.

## Decision

**Make `avidianity/freyya` public, and keep one GitHub account for the Owner and agents.**

- Branch protection on `main` requires a pull request and the required CI checks.
- It cannot require an approval, because the only account cannot approve its own pull requests.
- `CODEOWNERS` names the Owner for protected paths, as documentation; GitHub cannot enforce it here.
- At Gate 0, the Owner merges every pull request.
  Agents follow this as a rule in `.rules`; GitHub does not enforce it.

## Considered options

- **GitHub Pro on a private repository.**
  Rejected: the Owner chose not to pay.
- **Private repository with no enforcement.**
  Rejected: the Owner chose public.
- **A second GitHub account for agents.**
  Rejected: the Owner chose one account.

## Consequences

- Required CI checks still block a merge on red.
- Non-agent approval and Gate 0 merge limits rest on agent rules, not on GitHub.
  This is a known gap against `trusted-ci` and [corrections.md](../agents/corrections.md#mandatory-gates-read-the-records).
- Everything in the repository, including commit author emails, is public.
- Close the gap later with a separate agent account, or a paid plan with a private repository.
