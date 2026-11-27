# Runtime verification instructions

This page applies when you change a user-visible feature, or when you create or maintain the Project's runtime verification.
Tests prove code paths; runtime verification proves that the running product does what a user needs.

## Mandatory: when it starts

Bootstrap does not create runtime verification.
Create the Verification contract and the first Feature map with the first user-visible feature.
After that, every user-visible feature gets a Feature map in the same change that adds it.

## Mandatory: the Verification contract

The Verification contract is one page, `docs/verification/README.md`, plus the commands it names.
It answers six questions:

| Operation | Must answer |
| --------- | ----------- |
| Launch | How do I start an isolated copy of the application with test credentials and test data? |
| Doctor | How do I know that this copy is the right build, healthy, and mine, before I drive it? |
| Drive | How do I perform one user journey, through the real user-facing path? |
| Assert | Which machine-checkable result means the behavior is correct, including the authoritative stored data? |
| Evidence | Where do the commands, results, traces, screenshots, and logs go? |
| Cleanup | What is removed, and what evidence stays? |

Rules for the contract:

- Every command in it is executable and documented.
  Never rely on a setup step that only lives in someone's memory.
- Doctor is read-only.
  It checks the build revision, the port owner, the credentials, and the health of the copy.
  It refuses a stale copy or a copy that another agent or person started.
- Drive uses the stack's existing tools for browsers, HTTP, CLIs, or terminals.
  Prefer stable accessible names or identifiers over pixel positions.
- Assert checks both what the user sees and the authoritative effect, such as the stored record or the sent message.
  A screenshot alone never proves an authorization rule, a balance, or a sent message.
- Cleanup stops only what this run started, and it keeps the evidence.
  Running Cleanup twice is safe.
- State whether two copies can run at the same time.
  When they cannot, say so; parallel agents then take turns.

## Mandatory: Feature maps

Keep `docs/verification/features/README.md` as an index, and one page per user-visible feature.
Use [templates/feature-map.md](../templates/feature-map.md).
Start with the three to five features that matter most.

A Feature map covers more than the golden path.
Include the loading, empty, error, unauthorized, and retry states that matter for that feature.

## Mandatory: proving the verifier

A verifier that nobody has run is a draft.
Before the first Feature map merges, a fresh agent session that did not write it must:

1. Start from a clean checkout with only the documented access.
2. Run Launch, then Doctor, and confirm that Doctor refuses a stale or foreign copy.
3. Drive the mapped feature through the real user-facing path.
4. Check both the visible result and the authoritative effect.
5. Break the feature on purpose, and confirm that the verifier fails for the right reason.
6. Restore the feature, and confirm that the verifier passes.
7. Run Cleanup twice, and confirm that the evidence still exists.
8. Try a second copy at the same time, and confirm the contract's answer.

Record the result in the pull request's Evidence bundle.

## Mandatory: evidence for a user-visible change

For each user-visible change, keep:

- the build revision, the role used, the test data version, and the device or browser;
- the route and the stable control names;
- the user actions;
- the network requests: method, status, and a sanitized response shape;
- the authoritative effect, before and after;
- positive, negative, and failure-recovery assertions;
- screenshots or traces after meaningful state changes;
- what Cleanup removed;
- the verdict (PASS, FAIL, or INCONCLUSIVE), the reason, and the command to reproduce it.

Evidence from a different revision is INCONCLUSIVE.
Remove secrets and personal data before you store or post evidence.

## Mandatory: keeping verification current

When you maintain verification, the result is one of three states:

- **Clean**: every mapped feature passes from source and from a live run; nothing changes.
- **Changed**: one pull request updates the map or the commands after you proved drift with a live run.
- **Blocked**: a named prerequisite or failure prevents a reliable pass.

Maintenance changes verification only.
Never edit product code to make an old expected result look right.
A broken product is a product bug: report it, and do not hide it with a map change.

## Mandatory: who may weaken an expected result

A change that weakens an expected result, an authorization fixture, or a negative path in a Feature map is Tier 3.
The implementer of a feature cannot approve a weaker expected result for that same feature.
