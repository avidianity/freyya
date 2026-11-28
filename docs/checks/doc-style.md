# doc-style

**Invariant**:
Markdown files contain no em dash character (U+2014), and pass the Project's Markdown linter.

Kit spec: [checks/doc-style.md](https://github.com/avidianity/factory/blob/v0.9.0/checks/doc-style.md).

## Implemented by

- Command: `cargo xtask doc-style` for the em dash rule.
- Tool: markdownlint-cli2 0.24.2 with `.markdownlint-cli2.jsonc`.

Runs in: hook and CI.
