# doc-size

**Invariant**:
Every active documentation and guidance file stays under its tiered byte budget:

- `.rules`: under 8,000 bytes.
- Each `docs/agents/*.md` task page and each domain `RULES.md`: under 16,000 bytes.
- Every other active documentation file (`MAP.md`, `GLOSSARY.md`, `GLOSSARY-MAP.md`, `HOW-TO.md`, `STATE.md`, `how-to/*.md`, `state/*.md`, `docs/domains/**/*.md`): under 32,000 bytes.

Files under an `archive/` folder are exempt.

Kit spec: [checks/doc-size.md](https://github.com/avidianity/factory/blob/v0.9.0/checks/doc-size.md).

## Implemented by

- Command: `cargo xtask doc-size`.
- `docs/adr/` and `docs/research/` are records and reference, so they have no budget; `archive/` folders are exempt.

Runs in: hook and CI.
