# Delegation and handoff instructions

This page applies when one agent session starts other agent sessions (subagents) inside this Project, reviews their work, or integrates it.
The operating model for many independent sessions across Projects is in the Kit's `fleet/` folder, not here.

## Mandatory: permission and ownership

Work directly unless the Owner asks for delegation or invokes a skill whose workflow delegates.
A skill that loads by itself does not give permission to delegate.
Permission to delegate covers the stages described here, not unrelated work.

The session that delegates is the parent.
The parent owns integration, verification, and publishing.
The parent may give one merge into a local Integration branch to a fresh merger subagent, and then verifies the result.
Subagents never push, open pull requests, or merge a pull request.
The one exception is a research subagent of `wayfinder`: it pushes its own `research/<NAME>` branch, which never merges and gets no pull request.

## Mandatory: fresh sessions

Every dispatch starts a fresh session, including follow-up questions, reviews, and fixes.
Never resume or reuse an earlier subagent session.
The parent keeps continuity through exact handoffs, not through subagent memory.

## Mandatory: isolated checkouts

Every subagent works in its own checkout, including read-only exploration and review.
Use a worktree or a leased checkout from a pool; [adapters/tools.md](https://github.com/avidianity/factory/blob/v0.9.0/adapters/tools.md) lists tools.
Never share a checkout between agents, and never switch the parent's checkout for a subagent.

The subagent's first repository action is to get its checkout.
It then checks out the exact baseline commit and verifies `HEAD`.
It reads the rules file in that checkout.

When the Project has one shared resource that parallel runs fight over, such as one test database or one port, give each subagent its own copy or run them one at a time.
Two agents on one database, port, or credential set can corrupt each other's results.

## Mandatory: exact handoffs

Every dispatch includes:

- the scope, the expected output, the task or spec, and the allowed edits;
- the exact files, folders, and task pages the subagent needs, so it does not scan unrelated parts of the repository;
- the exact baseline commit;
- a snapshot of relevant uncommitted work, including untracked files and deletions;
- earlier findings, accepted and rejected feedback, and checks already run;
- the known-red baseline, so the subagent does not run it again;
- what the subagent may commit, and any Code review checkpoint.

Apply snapshots only inside the new checkout.
Verify that the snapshot matches, for example with a file list and content hashes.
`git diff` alone leaves out new files.

## Default: the implementer and reviewer pipeline

For delegated implementation:

1. An implementer writes the change and its tests, and commits locally.
2. Two fresh read-only reviewers check it in parallel:
   - **Standards**: does it follow the Project's rules and task pages?
   - **Spec**: does it do what the task asked, and nothing else?
3. Each fix goes to a fresh implementer, and the result goes to fresh reviewers.

Under `implement-spec`, each Frontier Ticket goes to one fresh implementer subagent, which builds it test-first and merges the Integration branch tip into its own branch.
One `code-review` of the whole Integration branch follows the last Ticket.
[planning.md](./planning.md#default-build-a-whole-spec) says when to use it.

## Mandatory: blocked tasks

Dispatch only tasks on the Frontier.
Dispatch a blocked task when every task that blocks it reports a clean result.
Its baseline is the Integration branch tip after those merges, or the tip of the one branch it builds on.

## Mandatory: requested Code review

When the Owner asks to review before publishing, pause integration and publishing.
Give the Owner:

- the checkout path and branch;
- the diff summary and the files changed;
- the checks and their results;
- the open questions.

Keep the checkout until the Owner approves.

## Mandatory: integration and cleanup

- Verify each subagent's report against its checkout: the commits exist, the diff stays in scope, and the checks are clean.
- Re-run the relevant tests after you merge the moved base into a branch.
- Return or remove each checkout after its pull request merges.
- Keep the checkout of every unmerged, blocked, or review-pending task, and say why.
