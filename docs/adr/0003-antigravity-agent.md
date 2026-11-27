---
status: accepted
---

# Antigravity joins Claude Code and Codex

The Owner named Claude Code, Codex, and Antigravity (`agy`) as the Project's agents during Bootstrap on 2026-11-28.

## Context

The Kit's agent adapter lists Claude Code, Codex, and OpenCode.
Antigravity is not in it.
Its official docs, read on 2026-11-28, say:

- it reads `AGENTS.md` in the workspace automatically, with a 24,000-byte limit per file ([rules](https://antigravity.google/docs/rules));
- it reads workspace skills from `.agents/skills` ([skills](https://antigravity.google/docs/skills));
- the Owner starts a skill by typing `/<SKILL-NAME>` in the CLI or the app;
- project tool servers live in `.agents/mcp_config.json` ([MCP](https://antigravity.google/docs/mcp)).

The docs do not say whether Antigravity follows symbolic links, or whether it honors `disable-model-invocation`.

## Decision

**Support Antigravity through the Kit's one-source layout, with no extra files.**
`AGENTS.md` links to `.rules`, and `.agents/skills` is the one skills folder.
The rule in [planning.md](../agents/planning.md#mandatory-how-agents-run-pinned-skills) still forbids an agent to start an Owner-only skill on its own.

## Considered options

- **A separate Antigravity rules file under `.agents/rules/`.**
  Rejected: two copies of the rules drift apart.

## Consequences

- No Antigravity command guard exists yet, so CI is the only backstop for [no-hook-bypass](../checks/no-hook-bypass.md) in Antigravity.
- If Antigravity does not follow the `AGENTS.md` link, the first Antigravity session will show it; fix the layout then.
