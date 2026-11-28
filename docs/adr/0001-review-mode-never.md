---
status: accepted
---

# Review mode is Never

The Owner chose Review mode Never during Bootstrap on 2026-11-28, after first choosing Critical only.

## Context

The Owner does not want to read code.
The Kit offers three Review modes: Always, Critical only, and Never.
See [review-and-merge.md](../agents/review-and-merge.md#mandatory-review-modes).
Freyya is a single-user, local, read-only tool, so a bad change rarely reaches anyone but the Owner.

## Decision

**Review mode is Never.**

- Tier 0 to 2: the agent may merge on green CI and a complete Evidence bundle, once an Autonomy gate allows agent merges.
  At Gate 0, the Owner merges every pull request.
- Tier 3: an Independent reviewer approves the code, then the Owner approves a plain-language Risk summary.
- Tier 4: the Owner approves a Risk summary, and the agent never runs the change alone.

The Owner still makes every Decision.
Agents phrase each Decision with no code, and explain each new term in the same sentence.

## Considered options

- **Always.**
  Rejected: the Owner does not want to read every diff.
- **Critical only.**
  Rejected: the Owner first picked it, then chose not to read code at all.

## Consequences

- Every Tier 3 and 4 change needs an Independent reviewer and a Risk summary before the Owner's yes.
- Agents never show the Owner code to approve.
- The Owner can change the mode at any time with a new ADR.
