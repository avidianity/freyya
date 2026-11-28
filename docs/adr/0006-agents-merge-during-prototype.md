---
status: accepted
---

# Agents merge their own pull requests up to Tier 3 while freyya is a prototype

The Owner decided this on 2026-11-28: "i pre-approve everything until i say no ... including auto merge. this is still early and i need a working prototype as fast as possible."
It supersedes the merge part of [ADR 0004](0004-public-repo-single-account.md), which left every merge to the Owner at Gate 0.

## Context

The Kit starts every project at Gate 0, where the Owner merges every pull request, and opens agent merges only after records prove each step safe ([corrections.md](../agents/corrections.md#mandatory-gates-read-the-records)).
freyya is early: the first milestone merged on 2026-11-28, and waiting for the Owner to merge each pull request slowed work on a prototype.
One GitHub account serves the Owner and agents ([ADR 0004](0004-public-repo-single-account.md)), so GitHub cannot tell an agent merge from an Owner merge either way.

## Decision

**An agent merges its own pull request once every required check passes; at Tier 3 it also needs an Independent reviewer's PASS; Tier 4 still needs the Owner.**

- `policy/gate.toml` sets `active_gate = 3` and `agent_merge_max_tier = 3`; the tier check reads it from the protected base revision and reports who may merge.
- The Owner's standing yes covers Tier 3 Risk summaries; the agent still writes the Risk summary into the pull request.
- Branch protection on `main` stays: a pull request and the `rust`, `repo`, and `policy` checks are required for every merge.
- The Owner ends this at any time by saying no; the agent then sets `active_gate = 0` and removes `agent_merge_max_tier` in a pull request that the Owner merges. `active_gate = 0` stops all agent merges even if the field stays, and the code caps the field at Tier 3.

## Considered options

- **Stay at Gate 0.**
  Rejected: the Owner wants speed during the prototype.
- **Gate 1 or 2 only (agents merge Tier 0 to 2).**
  Rejected: most early changes touch CI, checks, or rules, which are Tier 3, so the Owner would still merge most pull requests.
- **Agents merge Tier 4 too.**
  Rejected: Tier 4 holds irreversible operations, which the Owner starts or watches.

## Consequences

- Work merges without waiting for the Owner, and the Owner reviews after the fact.
- The Kit's record-based gate climb (20 Owner-approved merges before Gate 1) is skipped; a bad merge is caught only by checks, the Independent reviewer at Tier 3, and the Owner's later review.
- An open Correction for a Tier 3 or 4 failure class, or an S1 or S2 escaped bug from an agent merge, still drops the project to Gate 0 at once.
