# Task contract template

````markdown
# Task: <the behavior or defect, in one line>

## Goal

<The outcome a user can observe, and why it matters.>

## Scope

- Allowed paths: <paths>
- Forbidden paths: <paths>
- Expected Tier: <0-4; the tier check decides the final Tier>

## Preconditions

<Test data, environment, and permissions that the task needs.>

## Acceptance criteria

- [ ] <The positive behavior.>
- [ ] <The negative behavior: an unauthorized or invalid path is refused.>
- [ ] <Error, retry, or concurrency behavior, where it applies.>

## Invariants

- <A rule that the change must never break.>

## Verification

<The exact commands that prove the criteria. Use only commands that exist.>

## Evidence

<What the pull request must show: tests, runtime evidence, stored-data checks.>

## Ask instead of guessing when

- <The intent is unclear.>
- <The task needs production access, a schema change, or a new dependency.>
- <A Decision is needed.>

## Budget

- Attempts per root cause: 2
- Model spend: <limit or "Owner's default">
- CI and runtime: <limit or "Owner's default">

## Done means

<The pull request is merged under review-and-merge.md, or a blocker is reported.>
````
