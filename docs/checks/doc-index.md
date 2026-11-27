# doc-index

**Invariant**:
Every active `docs/agents/*.md` task page, every domain folder doc (`GLOSSARY.md`, `HOW-TO.md`, `STATE.md`, `RULES.md`), and every topic guide under `how-to/` or `state/` is transitively reachable by links with read triggers starting from `.rules`.
For example, `.rules` links `MAP.md` or `GLOSSARY-MAP.md`, which links each domain folder, whose `HOW-TO.md` or `STATE.md` links its topic guides.
Files under `archive/`, `docs/adr/`, and `docs/research/` are exempt from `.rules` reachability, because agents open ADRs by slug and skip `archive/` unless asked about history.

Kit spec: [checks/doc-index.md](https://github.com/avidianity/factory/blob/v0.9.0/checks/doc-index.md).

## Implemented by

- Command: `cargo xtask doc-index`.
- It follows Markdown links from `.rules` and reports every task page or domain doc that no chain reaches.

Runs in: hook and CI.
