# doc-links

**Invariant**:
Every relative link in a Markdown file resolves to an existing file, and every heading anchor in a link exists in its target.

Kit spec: [checks/doc-links.md](https://github.com/avidianity/factory/blob/v0.9.0/checks/doc-links.md).

## Implemented by

- Tool: lychee 0.24.2 with `lychee.toml`: offline, local files and heading anchors only.
- Command: `lychee --config lychee.toml <FILES>`.

Runs in: CI, and in the hook for changed Markdown files.
