# agent-entry-links

**Invariant**:
Each agent entry file that [adapters/agents.md](https://github.com/avidianity/factory/blob/v0.9.0/adapters/agents.md) lists for the Project's agents is a symbolic link to `.rules`, and `.rules` exists.

Kit spec: [checks/agent-entry-links.md](https://github.com/avidianity/factory/blob/v0.9.0/checks/agent-entry-links.md).

## Implemented by

- Command: `cargo xtask agent-entry-links`.
- The entry files are `CLAUDE.md` and `AGENTS.md` (Codex and Antigravity), listed in `policy/layout.toml`.
- On Windows, create a link with `cmd /c mklink CLAUDE.md .rules`; Developer Mode must be on, and Git needs `core.symlinks=true`.

Runs in: hook and CI.
