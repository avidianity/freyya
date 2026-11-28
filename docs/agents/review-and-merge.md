# Review and merge instructions

This page decides who approves a change before it merges, and what proof the change carries.
It separates two things that people often mix up:

- A **Decision** is a choice about product behavior, data, cost, security, or architecture.
  The Owner always makes it.
- **Code review** is reading a diff to judge it.
  The Review mode decides who does it.

## Mandatory: Decisions

The Owner makes every Decision, in every Review mode.
An agent never makes a Decision alone, and never hides one inside an implementation.

When a task needs a Decision:

1. Stop before you implement the part that depends on it.
2. Ask one question, with your recommendation and one reason.
3. Phrase it for the Owner's Review mode.
   In mode Never, use no code, and explain each new term in five words or fewer, in the same sentence.
4. Record the answer where it belongs: an ADR for architecture, the glossary for a term, the issue for task scope.

These are Decisions:

- what the product does, and for whom;
- which data is kept, for how long, and who can see it;
- anything that costs money to run or to call;
- authentication, authorization, isolation between users or tenants, and secrets;
- a new dependency, service, or architectural pattern;
- a change to a Mandatory rule, a check, or CI.

These are not Decisions:

- names, file layout, and code structure inside the rules;
- which test to write, and how;
- how to fix a failure that this change introduced;
- choices explicitly covered by an active Standing Decision.

### Standing Decisions

A **Standing Decision** is an Owner-approved boundary for routine choices, recorded in one ADR linked from `.rules`.
It starts empty at Bootstrap; the Owner adds entries only after seeing which recurring choices are safe.

Each entry states:

- what the agent may decide alone;
- concrete scope and numeric or policy limits;
- an expiry date, a review condition, or "until changed".

Before asking the Owner a routine Decision, check the Standing Decisions ADR.
When an entry covers the choice, follow it and cite the entry in the pull request body or decision log.
Editing the Standing Decisions ADR is always Tier 3; an agent never changes it alone.

Tier 3 and Tier 4 concerns are never eligible for Standing Decisions.
Authentication, authorization, isolation between users or tenants, money or billing, secrets, destructive migrations, CI, checks, branch protection, and production access always require an explicit Owner Decision.

## Mandatory: Tiers

Every change has a Tier from 0 to 4.
The tier check computes the Tier deterministically from the changed paths, reverse dependencies, and diff signals against the protected base revision.
An agent never sets or lowers the Tier.
The check specs [risk-tier](../checks/risk-tier.md) and [trusted-ci](../checks/trusted-ci.md) define the check and its trust boundary.

| Tier | Meaning | Examples |
| ---- | ------- | -------- |
| 0 | Mechanical | Formatting, generated code from an approved source, a document fix |
| 1 | Low | A local bug fix with a fail-first test, touching no risk area |
| 2 | Medium | Behavior across modules, a public API, a dependency change, background job behavior |
| 3 | High | Authentication, authorization, isolation between users or tenants, money, schema migrations, secrets, destructive jobs, CI and check configuration, Feedback budgets |
| 4 | Critical | Bulk data changes, production access changes, irreversible operations |

The Project's tier map lists the paths for each Tier.
Bootstrap writes it from the Survey and the Owner's risk areas.
A path in no list is Tier 2.

When a change touches paths in more than one Tier, the highest Tier applies.

## Mandatory: Review modes

The Project has one Review mode, recorded in `.rules` and an ADR.
The Owner can change it at any time with a new ADR.
The Owner can also override it for one pull request by saying so on that pull request.

| Tier | Always | Critical only | Never |
| ---- | ------ | ------------- | ----- |
| 0-2 | The Owner reviews the diff and approves | The agent merges on green CI and a complete Evidence bundle | The agent merges on green CI and a complete Evidence bundle |
| 3 | The Owner reviews the diff and approves | The Owner reviews the diff and approves | An Independent reviewer approves the code, then the Owner approves a Risk summary |
| 4 | The Owner approves; the agent never runs it alone | The Owner approves; the agent never runs it alone | The Owner approves a Risk summary; the agent never runs it alone |

**Always** fits an Owner who reads code and wants to see every change.

**Critical only** fits an Owner who reads code but wants agents to land routine work.

**Never** fits an Owner who does not read code.
The Owner still makes every Decision and approves every risky change, from a plain-language Risk summary.

"The agent never runs it alone" means: the agent prepares the change and the procedure, and the Owner starts it or watches it run.

## Mandatory: merge contract for agent merges

An agent merges a Tier 0 to 2 change only when all of these are true:

- [ ] The tier check ran from the protected base revision on the final revision and gave Tier 0, 1, or 2.
- [ ] Every required CI job passed on the exact revision that merges.
- [ ] The merged-result check in [git.md](./git.md#mandatory-check-the-merged-result-before-a-merge) passed.
- [ ] A behavior change has a fail-first test.
- [ ] No test, assertion, check, or CI job was removed, skipped, or weakened.
- [ ] The diff has no unexplained file and no secret.
- [ ] The Evidence bundle is complete in the pull request body.
- [ ] The Prove steps that [planning.md](./planning.md#mandatory-rigor-by-expected-tier) requires for the Tier ran, and no verified finding is open.
- [ ] The Project's active autonomy gate permits agent merges at this Tier; see [corrections.md](./corrections.md#mandatory-gates-read-the-records).

When one box is not checked, the agent does not merge.
It reports what is missing.

## Mandatory: Evidence bundle

The pull request body carries the evidence.
Use [templates/pr-evidence.md](../templates/pr-evidence.md).
It holds:

- the exact head commit SHA that the evidence covers;
- the Tier from the tier check, and why;
- the task and its acceptance criteria;
- the fail-first test: its name, and proof that it failed before the change;
- the checks that ran, backed by green forge required status checks on that exact revision;
- runtime evidence for a user-visible change, backed by revision-stamped verifier or CI artifacts, as [verification.md](./verification.md) says;
- from Tier 2: the Safety fact and its revision-stamped proof artifact, and from Tier 3: the verified `interrogate` findings and their fixes, as [planning.md](./planning.md#mandatory-proof-that-runs) says;
- known gaps, deferred coverage, and failures classified as pre-existing or environmental.

Remove secrets and personal data from evidence before you post it.

## Mandatory: Independent reviewer

An Independent reviewer is a fresh agent session that did not see the implementation.
Use a model from a different vendor when one is available, because agents from one model share blind spots.

Give the Independent reviewer:

- the task and its acceptance criteria;
- the exact revision and the diff;
- the check specs and task pages that apply;
- the prompt in [methods/review-prompts.md](https://github.com/avidianity/factory/blob/v0.9.0/methods/review-prompts.md#independent-verifier).

Do not give it the implementer's reasoning or conclusions, or the `interrogate` findings.
Its verdict is PASS, FAIL, or INCONCLUSIVE, with the commands it ran.
A FAIL or INCONCLUSIVE verdict blocks the merge.

## Mandatory: Risk summary

A Risk summary is for an Owner who will not read the code.
Use [templates/risk-summary.md](../templates/risk-summary.md).
It states, in plain words:

- what changes for people who use the product;
- what could go wrong, and how likely that is;
- how to undo it, and what cannot be undone;
- the evidence: tests, checks, and the Independent reviewer's verdict;
- your recommendation: yes or no.

It contains no code.
It asks one yes or no question.

## Mandatory: what an agent never does

- Merge a Tier 3 or 4 change without the approval this page requires.
- Change the tier map, the tier check, or the Review mode without an Owner Decision.
- Run a Tier 4 operation alone.
- Treat green CI, agent agreement, or a confident explanation as a substitute for the evidence above.

## Default: policy at each step

Approval is not only at merge time.
A change passes these steps, and a later step cannot repair a gap in an earlier one:

1. Task approval: is the goal authorized and measurable?
2. Execution approval: may the agent touch these paths and use these tools?
3. Pull request scope: are the changed files inside the task's scope?
4. Verification: does the evidence match the exact revision?
5. Merge approval: does the merge contract hold?
6. Deployment approval: is the target environment checked and the rollback ready?
7. Observation: did anything unexpected happen after release?
