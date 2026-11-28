# skills-link

**Invariant**:
The Project has one skills folder.
Each other skills folder that an agent of the Project reads is a symbolic link to it.
Each skill in the folder is listed in the skills lock file or is a Project skill.

Kit spec: [checks/skills-link.md](https://github.com/avidianity/factory/blob/v0.9.0/checks/skills-link.md).

## Implemented by

- Command: `cargo xtask skills-link`.
- The one skills folder is `.agents/skills`; `.claude/skills` links to it.
- Each skill is pinned in `skills-lock.json`, or listed as a Project skill in `policy/layout.toml` (now `correction`).

Runs in: CI, inside `cargo xtask repo-checks`.
