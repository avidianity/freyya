# Corrections instructions

This page applies when an agent mistake repeats, when a bug escapes after merge, or when you decide how much autonomy agents get.
Its goal: each repeated mistake becomes a check at the lowest layer that can stop it.

## Mandatory: when a mistake becomes a Correction

A mistake becomes a Correction when it happens a second time, in any session.
A single mistake is fixed in its own change and needs no record.

Open one issue labelled `correction` per failure class, with [templates/correction.md](../templates/correction.md).
Before you open one, search for an open Correction of the same class and add to it instead.

A Correction names:

- the failure: a reproducible example and the area it affects;
- the impact on users: severity and likelihood;
- the root cause: a missing invariant, a confusing API, an unclear requirement, or missing tooling;
- the lowest layer that can stop it, and why no lower layer can;
- a failing example and a passing example;
- where the check runs: hook, CI, or both.

A Correction closes only when its check or better API has merged and its failing example fails.
A rule in prose does not close a Correction, unless no lower layer can decide the case.

## Mandatory: decision order

For each Correction, take the first layer that can stop the mistake:

```text
Can we remove the confusing API or the invalid state?   -> simplify the design
Can the database enforce the rule?                      -> constraint or transaction
Can the type system rule it out?                        -> type or API shape
Can a linter or dependency rule detect it?              -> static rule
Can an automated test reproduce it?                     -> regression or property test
Can an agent drive the running product to prove it?     -> runtime verification
Is it a hard concurrency or ordering property?          -> consider model checking
Only judgment or policy is left?                        -> one short instruction, or review
```

[methods/trust-layers.md](https://github.com/avidianity/factory/blob/v0.9.0/methods/trust-layers.md) explains why this order holds.

## Mandatory: no rule inflation

- One rule lives in one place.
  Do not keep a rule both as a long instruction and as a check, unless each does a different job.
- When a check ships, shrink the prose rule to one line that links the check spec.
- Each check's error text names the rule and the approved alternative, so the agent can fix the mistake without reading the docs.

## Mandatory: Escaped defects

When someone finds a bug after its change merged:

1. Label the bug's issue `escaped`.
2. Link the pull request that caused it.
3. Record its severity: S1 critical, S2 major, S3 moderate, or S4 minor.
4. Record which check or review should have caught it.
5. When the same kind of bug escapes again, open a Correction.

## Mandatory: gates read the records

Agent autonomy grows in steps, and every Project starts at Gate 0.
A Project never auto-merges on merge zero; each gate opens only when the records prove the previous step is safe.
Enforce the active gate mechanically through forge permissions and a protected policy file read from the protected base revision, never through agent self-restraint alone.
Changes to the active gate file are Tier 3 under [trusted-ci](../checks/trusted-ci.md) and [test-oracle-guard](../checks/test-oracle-guard.md).

### Gate 0: assisted

Agents propose changes.
The Owner approves and merges every pull request.
Enforce Gate 0 at the forge: give agent credentials no merge permission, or require non-agent Owner approval on every pull request through branch protection.

Move to Gate 1 when all of these hold:

- Every Day-1 check is built and proven both ways.
- The tier check labels every pull request from the protected base revision.
- 20 Owner-approved agent pull requests have merged, each with a complete Evidence bundle.
- No open Correction exists for a failure class that touches a Tier 3 or 4 area.

### Gate 1: agents merge Tier 0 and 1

Agents merge Tier 0 and Tier 1 changes alone under the merge contract.
The Owner approves Tier 2 and above.
A required CI check running from the protected base revision reads the active gate file and blocks merges without non-agent Owner approval whenever the computed Tier exceeds Gate 1.

Move to Gate 2 when all of these hold:

- 30 agent merges at Tier 0 and 1 have landed.
- No `escaped` bug of severity S1 or S2 came from them.
- Every `escaped` bug of severity S3 or S4 has a Correction or a merged check.
- No open Correction exists for a failure class that touches a Tier 3 or 4 area.

### Gate 2: agents merge Tier 2 with an Independent reviewer

Agents merge Tier 2 changes after an Independent reviewer passes them.
The Owner approves Tier 3 and 4.
The required CI check from the protected base revision blocks Tier 2 merges until an Independent reviewer verdict is recorded and blocks Tier 3 and 4 without non-agent Owner approval.

Move to Gate 3 when all of these hold:

- 30 agent merges at Tier 2 have landed.
- No `escaped` bug of severity S1 or S2 came from them.
- Every `escaped` bug of severity S3 or S4 has a Correction or a merged check.
- The Independent reviewer caught at least one real problem in a planted test.

### Gate 3: steady state

Agents merge Tier 0 to 2 alone under the merge contract.
Tier 3 and Tier 4 always need the approval that [review-and-merge.md](./review-and-merge.md) requires, enforced by branch protection and forge ownership rules.

### Gate limits and regression

The Review mode caps these gates: under Review mode `Always`, the Project stays at Gate 0 whatever the records say.
An open Correction for a failure class touching a Tier 3 or 4 area, or an S1 or S2 `escaped` bug from an agent-merged change in the last 30 merges, closes agent merges and drops the Project to Gate 0 at once by updating the protected gate policy file (and revoking agent merge permission at the forge).
When dropped to Gate 0, the Project climbs again with fresh counts.
The Owner can change these thresholds with an ADR.

## Mandatory: stop triggers

Stop all agent merges at once, and tell the Owner, when any of these happens:

- An agent crosses a data boundary between users or tenants, or leaks a secret.
- An agent changes a test to hide a failing behavior.
- Evidence is false or cannot be reproduced, more than once.
- Flaky CI or runtime verification prevents a trustworthy result.
- An agent-merged change causes a critical incident or an unauthorized change of data.
- Cost exceeds the limit the Owner set.
- An agent works outside its approved repository, branch, scope, or permissions.

Merges resume only after the Owner decides.

## Default: fuller measurement

The records above are light on purpose: one label per record, and evidence in pull request bodies.
For cost, cycle time, and controlled comparisons, see the Kit's `fleet/measurement.md`.
