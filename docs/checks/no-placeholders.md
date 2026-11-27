# no-placeholders

**Invariant**:
No file in `.rules`, `docs/owner-guide.md`, `docs/agents/`, or `docs/checks/` holds a Kit placeholder: text in angle brackets that starts with a lowercase word, such as "main branch" written inside angle brackets.
Code samples inside fenced blocks are exempt only when the block is marked as a template.

Kit spec: [checks/no-placeholders.md](https://github.com/avidianity/factory/blob/v0.9.0/checks/no-placeholders.md).

## Implemented by

- Command: `cargo xtask no-placeholders`.
- It scans `.rules`, `docs/owner-guide.md`, `docs/agents/`, and `docs/checks/` with a Markdown parser.
- Write a variable that a reader fills per use in capitals, such as `<FILE>` or `<SKILL>`; the check accepts it.
- Fenced blocks marked `template` are exempt.

Runs in: CI, inside `cargo xtask repo-checks`.
