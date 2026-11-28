---
status: accepted
---

# Commit chosen real Saves as test fixtures

The Owner decided this during the spec grill on 2026-11-28, after the Bootstrap rule had banned committing any Save.

## Context

The Bootstrap rule said to never commit game files or Saves, and to test with synthetic fixtures or copies outside the repository.
The Owner said their Saves hold no sensitive information, and the current campaign is not a serious one.
Checks that run on real Saves catch parser and extraction faults that hand-made fixtures miss, and they can run in CI only if the Saves are in the repository.
The repository is public ([ADR 0004](0004-public-repo-single-account.md)).
GitHub rejects files over 100 MB and warns above 50 MB ([about large files on GitHub](https://docs.github.com/en/repositories/working-with-files/managing-large-files/about-large-files-on-github), read on 2026-11-28).
A committed file stays in the history after it is deleted.
Game files belong to Paradox, so redistributing them is a different question from the Owner's own Saves.

## Decision

**Commit a chosen set of the Owner's real vanilla Saves, each under 50 MB, in `fixtures/saves/`.**

- Every check, including CI, can run against these real Saves.
- Tiny hand-made fixtures cover only the cases that the real Saves do not contain, such as an unknown game version or a damaged Save.
- Game files stay out of the repository; checks that need them run only on the Owner's machine.
- The read-only rule toward the game is unchanged: freyya copies a Save into `fixtures/saves/` only when the Owner asks, and never writes to the save folder.

## Considered options

- **Synthetic fixtures only, real Saves outside the repository.**
  Rejected: CI could not test real Saves, and the Owner sees no privacy cost.
- **Commit every Save.**
  Rejected: late-game Saves grow large, and each one stays in the history.
- **Git LFS.**
  Rejected for now: it adds a storage quota and a tool for a handful of files.

## Consequences

- CI tests real Save parsing on every pull request.
- Repository history grows by each committed Save; choose them on purpose.
- A Save over 50 MB needs a new decision before it is committed.
- No hook or check enforces the 50 MB limit or the Owner's request yet; agents follow it as a rule, and GitHub's 100 MB limit is the only hard stop.
