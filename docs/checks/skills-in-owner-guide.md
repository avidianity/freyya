# skills-in-owner-guide

**Invariant**:
The name of each skill in the Project's skills folder appears in `docs/owner-guide.md`.

Kit spec: [checks/skills-in-owner-guide.md](https://github.com/avidianity/factory/blob/v0.9.0/checks/skills-in-owner-guide.md).

## Implemented by

- Command: `cargo xtask skills-in-owner-guide`.
- Each skill folder name must appear in `docs/owner-guide.md` in backticks.

Runs in: CI, inside `cargo xtask repo-checks`.
