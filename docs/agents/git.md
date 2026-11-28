# Git and workflow instructions

This page applies when you commit, push, open or merge a pull request, or build a Spec on an Integration branch.

## Mandatory: authorization and completion

| Request | Authorized completion |
| ------- | --------------------- |
| Ordinary implementation request | Edits and checks only; commit and publishing need an explicit request |
| Explicit `implement` | Implementation, checks, review, and a local commit on the current branch |
| Explicit `implement-spec` | Every Ticket of the Spec, merged into one Integration branch; push and a draft pull request when the tracker closes work through pull requests |
| Explicit `wayfinder` | Research subagents push `research/<NAME>` branches; no pull request, no merge |
| Explicit request to land, publish, or merge | Push, pull request, CI gate, merged-result check, and merge when [review-and-merge.md](./review-and-merge.md) allows it |

Merge authority also depends on the Tier and the Review mode.
[review-and-merge.md](./review-and-merge.md) decides who approves; this table decides what you may do without being asked again.

A requested Code review pauses integration, push, and merge until approval.

A push alone never completes a request to land.
Landing is complete when the pull request is merged, or when a blocker or a pending review is reported.

## Default: branches

The Project has one long-lived branch, `main`.
Each task gets one short-lived branch from the updated main branch and one pull request back to it.
Name branches `<TYPE>/<ISSUE>-<SLUG>`, or `<TYPE>/<SLUG>` without an issue.

A Spec that `implement-spec` builds gets one Integration branch, named for the Spec issue.
Each Ticket gets its own branch from the Integration branch, and merges back into it, never into `main`.
The Integration branch is short-lived, and lands as one pull request to `main`.

When the Project deploys from more than one long-lived branch, Bootstrap records the branch model in an ADR and replaces this section.

Never push directly to a long-lived branch.
Never delete a long-lived branch.

## Mandatory: before you change branches

- Inspect existing changes before you switch or synchronize.
  Preserve unrelated work.
- Synchronize the branch with a fast-forward pull when it tracks a remote.
- Then pin the review baseline to an exact commit.
  Do not pin before you pull, or upstream changes become part of the task review.

## Mandatory: commits

- Inspect the status, the diff, and recent history before you commit.
  Stage only the files you intend.
- Use Conventional Commits: `<TYPE>(<SCOPE>): <SUMMARY>`.
  Allowed types and scopes live in `policy/layout.toml`; the scopes are `app`, `saves`, `catalog`, `store`, `advisor`, `ui`, `checks`, `ci`, `deps`, `docs`, `research`, `rules`, `skills`, and `bootstrap`.
  The [commit-message](../checks/commit-message.md) check enforces this.
- Never add co-author lines or agent attribution.
- Keep unrelated fixes out of this branch; if you directly encounter a real bug outside the task, note it in one short line at handoff instead of bundling an unrequested commit.
  When an in-scope prerequisite cleanup is unavoidable, isolate it in its own commit before the main commit.
- For issue work, put `Closes #<ISSUE>` in the feature commit body and the pull request body.
- Never skip a hook.
  Fix the failure, then commit again.
- After each commit, confirm that `git log --oneline -1` moved, because a failed hook step can still print a passing summary.

## Mandatory: pull requests

- Before you open a pull request, review the full diff from the base, every commit, and the remote tracking state.
- Write the pull request body with the [Evidence bundle](./review-and-merge.md#mandatory-evidence-bundle).
- Always name the base branch.
- Merge with squash unless the Project's ADR says otherwise, and delete only the pull request's own head branch.

## Mandatory: CI gate

Local hooks check changed files.
CI owns the whole-tree checks.

Before you merge a pull request that changes code, every required CI job on the current revision must pass.
Watch CI at an interval of 60 seconds or slower.
A failed or cancelled required job blocks the merge.

A change to documents only runs the document checks.

## Mandatory: check the merged result before a merge

Two pull requests can each pass CI and still break the base together, because each was checked without the other.
So right before each merge, check the tree that the merge will produce:

1. In the checkout that has the branch, fetch the base.
2. Merge the base into the branch.
   Do not rebase, so the reviewed commits stay the same and the push needs no force.
3. Run `cargo xtask repo-checks`.
   It runs the whole-repository checks, such as unique record numbers and generated-file freshness.
4. When the merge made a commit, run the change's relevant tests again, then push.
5. Merge only after that.

Never check out the base branch for this step.
When another pull request merges into the same base first, do the step again.

When the forge offers a merge queue or a required "up to date" rule, the Project may use that instead, recorded in an ADR.

## Default: stacked pull requests

Stack pull requests when the Owner asks for a stack, or when tasks build on each other.

- Each branch in a stack starts from the tip of the branch below it.
- Land a stack bottom-up, or as a whole with the forge's stack tool.
- Run the merged-result check on the top branch, because it holds every change in the stack.
