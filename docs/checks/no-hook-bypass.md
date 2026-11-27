# no-hook-bypass

**Invariant**:
Every check that a hook runs also runs in CI on the same files.
Agent sessions refuse commands that skip hooks, such as a no-verify flag or a hooks-path override.

Kit spec: [checks/no-hook-bypass.md](https://github.com/avidianity/factory/blob/v0.9.0/checks/no-hook-bypass.md).

## Implemented by

- CI parity: `.github/workflows/ci.yml` runs every command in `lefthook.yml` on the whole pull request:
  `rustfmt`, `gitleaks`, `no-comments`, `no-suppressions`, `test-scope`, `dependency-pinning`, `agent-entry-links`, `doc-size`, `doc-index`, `doc-style`, `doc-links`, `markdownlint-cli2`, and `commit-message`.
- Claude Code guard: `.claude/settings.json` runs `cargo xtask guard` before every Bash and PowerShell command.
  It blocks `--no-verify` and its abbreviations outside a commit message, `git commit -n`, `core.hooksPath` overrides (quotes, backticks, and carets removed first), `LEFTHOOK` values that turn hooks off, `lefthook uninstall`, and edits under `.git/hooks` with either slash or a `cd .git` first.
  The guard fails open: when `cargo xtask guard` itself cannot run, Claude Code does not block the command, and CI parity remains the backstop.
- Codex and Antigravity: no command guard yet; CI parity is the backstop.

Runs in: CI for parity; the Claude Code guard for refusal.
