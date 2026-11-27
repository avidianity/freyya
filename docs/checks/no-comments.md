# no-comments

**Invariant**:
No line that a change adds to a source file holds a comment, except a comment on the allow-list.
The allow-list is:

- a shebang line;
- a license header that the Project's license requires;
- a directive that the language, compiler, or build tool reads, such as an encoding declaration or a build constraint;
- a doc comment on an exported symbol of a published package, when the package's API reference is generated from it;
- a tagged constraint comment that explains a non-obvious technical constraint that names, types, or a test cannot express, prefixed with `INVARIANT: <WHY>`, `SAFETY: <WHY>`, or `WORKAROUND(<REF>): <WHY>` (where `<REF>` names an issue, ADR, or URL).

Generated files are exempt.
A suppression directive is not on the allow-list; [no-suppressions](./no-suppressions.md) owns it.
Build this check with a syntax-tree parser or linter, never regular expressions over raw source text.

Kit spec: [checks/no-comments.md](https://github.com/avidianity/factory/blob/v0.9.0/checks/no-comments.md).

## Implemented by

- Command: `cargo xtask no-comments --base <BASE-REV> --head <HEAD-REV>`; the hook uses `--base HEAD --head :index`.
- It reads comments with `ra-ap-rustc_lexer`, the Rust compiler's own lexer, and checks only the lines that the change adds.

Runs in: hook and CI.

## Allow-list

A change to this list is Tier 3.

- A tagged constraint comment: `// INVARIANT: <WHY>`, `// SAFETY: <WHY>`, or `// WORKAROUND(<REF>): <WHY>`, where `<REF>` is `#<ISSUE>`, `ADR <NNNN>`, or a URL.
- A multi-line constraint uses one tagged block comment, such as `/* SAFETY: <WHY> */`; each `//` line needs its own tag.
- A shebang line.
- Generated files: paths under a `generated/` folder, or files whose first comment, in the first three lines, contains `@generated`.

Freyya publishes no package, so doc comments (`///` and `//!`) are not on the list.
