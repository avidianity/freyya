# doc-links

**Invariant**:
Every relative link in a Markdown file resolves to an existing file, and every heading anchor in a link exists in its target.

Kit spec: [checks/doc-links.md](https://github.com/avidianity/factory/blob/v0.9.0/checks/doc-links.md).

## Implemented by

- Command: `cargo xtask doc-links`.
- It parses every active Markdown file and `.rules` with `pulldown-cmark`, resolves each relative link, and checks each heading anchor with GitHub slug rules.
- External URLs are out of scope.

Runs in: CI, and in the hook for changed Markdown files.
