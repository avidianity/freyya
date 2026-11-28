# Testing and quality instructions

This page applies when you implement behavior, fix a bug, run checks, or defer tests.

## Mandatory: behavior verification

Every behavior change ships with an automated test in the same change, unless the Owner defers test work as the exception below says.

A bug fix starts with a reproduction of the failure that the user sees.
Reproduce it end to end, as close as possible to how a user meets it.
Then write a regression test that fails before the fix and passes after it.
This is the fail-first test that the [Evidence bundle](./review-and-merge.md#mandatory-evidence-bundle) records.

Test through the real boundary that the change sits on: the HTTP route, the authorization check, the database, the job queue, or the user interface.
Keep pure unit tests for pure functions.
Do not lower the boundary to make a test pass.

Automated tests do no network I/O beyond the local machine.
Inject deterministic adapters for external services and AI models.

Test behavior, not implementation.
A good test fails when a rule that a user can see breaks, not when a function is renamed.
[methods/testing-by-defect-class.md](https://github.com/avidianity/factory/blob/v0.9.0/methods/testing-by-defect-class.md) helps you choose the kind of test.

## Mandatory: what does not earn a test

Never add automated tests for files that carry no application behavior or that a lower layer already validates:

- declarative tool and build configuration files (linter, formatter, commit-message, compiler, bundler, database-tool, and container configs); validate them by running the tool or type checker;
- low-risk developer and repository scripts (such as local setup helpers or formatting wrappers); verify them by running the script when it changes;
- type-only declarations, static constants, generated files, and thin framework wiring; types, the compiler, and boundary tests already cover them;
- tautology tests that mirror configuration values or repeat implementation lines as assertions.

Base script exceptions on risk, never on change frequency.
High-risk scripts (such as deployment, backup, data-migration, or custom policy check scripts) earn targeted tests or rehearsals in a separate non-application test path.
Run that non-application test path when those scripts, their check or policy configuration, or lockfile dependencies change, never inside the application Feedback loops.
The check spec [test-scope](../checks/test-scope.md) enforces the declared application test paths and rejects config or low-risk script tests in the application suite.

## Mandatory: coverage scope

Coverage metrics are optional; a Project adopts coverage floors, complexity-coverage scores, or mutation targets only when they protect real risk.
When enabled, collect coverage only on files that carry application behavior, using an explicit include allow-list inside the declared application source paths.
Never collect coverage repository-wide by default.

Always exclude from coverage, complexity-coverage, and mutation targets:

- root, tooling, build, database-tool, and container configuration files;
- repository, build, and development scripts;
- generated code, schema artifacts, and database migrations;
- type-only declaration files, static constant tables, and pure re-export barrels;
- thin entry points and framework bootstrap wiring;
- test files, fixtures, fakes, and test helpers.

Never write a filler test to satisfy a coverage floor on a file that belongs on this exclusion list.
Fix the coverage scope configuration instead; that change is Tier 3 under [test-oracle-guard](../checks/test-oracle-guard.md).
The check spec [test-scope](../checks/test-scope.md) checks that coverage and mutation configurations stay inside the declared application source paths.

## Mandatory: the check ladder

| Tier | When | Command |
| ---- | ---- | ------- |
| Baseline | Once, before the first edit | `cargo test --workspace --exclude xtask --locked` |
| Relevant | Every implementation loop, and at handoff | `cargo test --locked --package <CRATE>` for each changed crate and each crate that depends on it |
| Full | Only when the Owner asks | `cargo test --workspace --exclude xtask --all-targets --locked` |
| Handoff | Once, when the work is done | Relevant tests, then `lefthook run pre-commit` |

Do not run static checks during iteration.
Run the scoped hook checks once at handoff.
Hooks and CI are the gate; do not reproduce CI locally.
After a tier passes, repeat it only for new changes, new failures, or a concern you can name.

A change to documents only needs the document checks, not application tests.

## Mandatory: Feedback budget

Each Feedback loop has a Feedback budget.
Code growth must never make a loop slower than its budget.

| Feedback loop | What it runs | Feedback budget |
| ------------- | ------------ | --------------- |
| Git hook | The hook checks on the changed files | 30 seconds |
| Relevant tier | The Relevant command above, on the changed crates | 2 minutes |
| CI gate | Every required CI job on one revision, wall clock | 10 minutes |

The numbers are Defaults.
Change them only by an ADR; such a change is Tier 3.
Never raise a budget to make a loop pass.

CI measures each loop on its standard runner, as the check spec [feedback-budget](https://github.com/avidianity/factory/blob/v0.9.0/checks/feedback-budget.md) says.
Local times are not evidence.

- At 80% of a budget, the check opens one `budget` issue for that loop.
  Work that issue before the loop reaches 100%.
- Over the budget, the loop fails.
  The change that crosses the budget brings the loop back under it, in the same pull request, even when older growth caused most of the time.
  This failure is Introduced, not Pre-existing.
- One run over the budget is a failure.
  Do not rerun to get a pass.

## Default: staying inside the Feedback budget

- The hook checks only the staged files.
- The Relevant tier selects affected tests from the dependency graph.
- CI caches dependencies and build outputs.
- CI shards tests across parallel jobs.
- Each check runs once per commit; no two workflows run the same job.
- A change to documents only skips application tests in CI.
- Keep declarative config files and low-risk repository scripts out of the application test suite, as [test-scope](../checks/test-scope.md) requires.
- Run non-application check-script and ops-script tests when those scripts, their configuration, or lockfile dependencies change, outside the application Relevant tier.
- Less code is less to check.
  The check specs [no-dead-code](../checks/no-dead-code.md) and [no-duplicate-code](https://github.com/avidianity/factory/blob/v0.9.0/checks/no-duplicate-code.md) stop code that adds time and no value.

## Mandatory: baseline and failure classes

Run the Baseline tier before your first edit.
Keep its failing tests as the known-red baseline.

When the baseline is red because of infrastructure, such as a stopped database, no network, or a missing binary, that is an environment fault.
Stop, report it to the Owner, and do no task work until it is fixed.
Any other red baseline is recorded as known red, and work continues.

Classify every failure before you try to fix it:

- **Introduced**: not in the known-red baseline, and reachable from this change.
  It is this task's work.
- **Pre-existing**: in the known-red baseline.
  Record it; never repair it inline.
- **Environmental**: caused by infrastructure, or shared by failures in unrelated files.
  Record it and report it at once; never repair it inline.

## Mandatory: stopping rule

An attempt is one edit made to fix a failure, followed by a re-run.
Failures that share one root cause count as one issue.

Stop after two failed attempts on one root cause.
Record it, report it, and continue the task.
Stopping is not a question: do not ask permission and do not wait for an answer.

The rule applies to writing a test too.
After two failed attempts to make a boundary test work, record the coverage gap, report it, and continue.

## Mandatory: recording

- An environment fault goes in the handoff and to the Owner at once, because it blocks work.
- A pre-existing failure gets one tracker issue labelled `needs-triage` per root cause, never one per failing test.
  [issue-tracker.md](./issue-tracker.md) gives the format.

## Mandatory: flaky tests

A flaky test is a test whose result changes across runs of the same commit with no code change.
A deterministic gate failure is not a flake.
An environment fault is not a flake.

A flake that your change introduced is your bug.
A flake that you find and did not introduce gets a `needs-triage` issue with the commit, the command, and the run count.
Never mark a flake as passing because a rerun passed.

## Mandatory: test integrity

Never make a check pass by weakening it.
These are forbidden without an Owner Decision:

- deleting, skipping, or renaming a failing test so that it does not run;
- changing an assertion to match the new output instead of the specified output;
- hardcoding a result for a test fixture;
- mocking a security check that runs in production;
- raising a timeout to hide a hang;
- adding a suppression directive;
- moving code to a path that a check does not scan.

The check spec [test-oracle-guard](../checks/test-oracle-guard.md) raises the Tier of a change that edits tests, check configuration, or CI.
Removing forbidden config, script, or tautology tests is an Owner-approved cleanup, not test weakening; open a dedicated cleanup pull request so the Owner approves the removal at Tier 3.

## Mandatory exception: deferred tests

When the Owner says **defer tests**, skip writing and running tests for the current task.
**Do not run tests** defers running only.
Deferral ends with the current task unless the Owner names a longer scope.
Record the missing coverage in the handoff, and never claim that deferred tests passed.
Git hooks stay on, even when a hook runs tests.

## Default: stronger tests

- Use property-based tests where the input space is large, and keep each minimized counterexample as a fixed test.
- Use targeted mutation testing on high-risk code: break the rule on purpose and confirm that a test fails for the right reason.
- Map implicit state machines before you add a formal model.
  [methods/formal-methods.md](https://github.com/avidianity/factory/blob/v0.9.0/methods/formal-methods.md) gives the admission rule.
