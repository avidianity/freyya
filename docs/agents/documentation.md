# Documentation instructions

This page applies when you write or change rules (`.rules`, `docs/agents/`, domain `RULES.md`), documents, `MAP.md`, `STATE.md`, `HOW-TO.md`, ADRs, research notes, glossaries, or changelogs.

## Mandatory: one home per fact

Each fact has one home.
Every other document links to that home instead of repeating the fact.
When you change behavior, contracts, topology, or operations, update the document that owns it in the same change.
The Earned check spec [doc-freshness](https://github.com/avidianity/factory/blob/v0.9.0/checks/doc-freshness.md) enforces doc updates when a pull request changes declared public boundary files.
Link `HOW-TO.md` and `STATE.md` claims to their owning source files where practical.
Then [doc-links](../checks/doc-links.md) fails automatically when referenced files move or are deleted.

| Kind of fact | Home |
| ------------ | ---- |
| Generalized project constraint or routing map | `.rules` (under 8,000 bytes) |
| Cross-cutting task-specific agent instruction | `docs/agents/<TASK>.md` (under 16,000 bytes) |
| Domain-specific agent constraint or boundary exception | That domain's `RULES.md` (co-located in its module folder, or `docs/domains/<DOMAIN>/RULES.md`) |
| How to use or operate a domain now | That domain's `HOW-TO.md` (and `how-to/<TOPIC>.md` when split) |
| Current state of a domain (what exists, live integrations, active limits) | That domain's `STATE.md` (and `state/<TOPIC>.md` when split) |
| System-wide ownership, topology, boundaries, and domain index | `MAP.md` |
| A decision and its reasons | An ADR in `docs/adr/` |
| The reason behind code | Names, ADRs, tests, commit messages, or a tagged constraint comment; see [engineering.md](./engineering.md#mandatory-no-code-comments) |
| A domain term | The `GLOSSARY.md` that owns it; see [domain.md](./domain.md) |
| A check's invariant | `docs/checks/<NAME>.md` |
| Which skill the Owner starts, what to type, and what to approve | `docs/owner-guide.md` |
| A user-visible feature's verification | Its Feature map; see [verification.md](./verification.md) |
| A finding from outside research | `docs/research/` |
| Release notes of a published package | That package's changelog |

## Mandatory: domain folders and lazy creation

Keep domain knowledge out of `.rules`.
Code domains co-locate their docs inside their own module or feature folder.
Cross-cutting or operational domains (such as deployment, observability, billing policy, or data boundaries) live in `docs/domains/<DOMAIN>/` with the same layout.

Each domain folder holds up to four single-purpose files, created lazily only when needed:

- `GLOSSARY.md`: domain terms and definitions only; no rules, state, or procedures.
- `HOW-TO.md`: how to call, use, or operate the domain now.
- `STATE.md`: current state of the domain.
- `RULES.md`: domain-specific agent constraints and boundary exceptions that apply only when working in that domain.

Do not create empty placeholder files for a domain.
Link every active domain folder, task page, and topic guide from `.rules`, `MAP.md`, `GLOSSARY-MAP.md`, or its parent index with a concrete read trigger; the check spec [doc-index](../checks/doc-index.md) enforces this.

## Mandatory: HOW-TO pages

A `HOW-TO.md` shows the current way to call a module or run a domain procedure.
Use [templates/how-to.md](../templates/how-to.md).

- Start with a literal sample that works, then state the rules.
- Keep only current contracts.
  Move history and reasons to an ADR.
- A HOW-TO beats an older ADR on call shape.
- When a `HOW-TO.md` approaches its size budget, split it into topic guides under `how-to/<TOPIC>.md` in the same folder.
  Turn `HOW-TO.md` into an index table (`Guide | Read before you change`) so agents load only the guide for the area they edit.

## Mandatory: MAP and STATE pages

`MAP.md` at the repository root records system-wide ownership, deployment topology, data and request boundaries, and the index of domains.
Use [templates/map.md](../templates/map.md).

A domain's `STATE.md` records how that domain stands right now: what is live, active integrations, active limits, and operational facts that do not belong in a call contract (`HOW-TO.md`) or decision history (`docs/adr/`).
Use [templates/state.md](../templates/state.md).

- Update `MAP.md` and `STATE.md` in place when current reality changes.
- Keep no changelog or timeline in `STATE.md`; move superseded facts to an ADR or `archive/`.
- When a `STATE.md` approaches its size budget, turn it into an index pointing to topic files under `state/<TOPIC>.md`.

## Mandatory: ADRs

Write an ADR when a decision is hard to reverse, would surprise a later reader, and came from a real trade-off.
Skip it when any of the three is missing.

Use [templates/adr.md](../templates/adr.md).
An ADR holds the decision, the reasons, the options you rejected and why, and the consequences.
Number ADRs in sequence; the check spec [record-numbers](../checks/record-numbers.md) keeps numbers unique.

To change an accepted decision, write a new ADR that names the one it supersedes.
Edit an accepted ADR in place only to fix a fact or add a small amendment, and mark the amendment with its date.

## Default: research notes

Put the result of outside research in `docs/research/<TOPIC>.md`.
Name each source and the date you checked it.
Research notes are reference, never instructions.

## Mandatory: changelogs

Only published packages have changelogs.
Do not edit a generated changelog by hand.

## Mandatory: size budget

Active docs stay under tiered byte budgets, enforced by [doc-size](../checks/doc-size.md):

- `.rules`: under 8,000 bytes.
- `docs/agents/*.md` and domain `RULES.md`: under 16,000 bytes.
- All other active docs (`MAP.md`, `GLOSSARY.md`, `GLOSSARY-MAP.md`, `HOW-TO.md`, `STATE.md`, `how-to/*.md`, `state/*.md`, `docs/domains/**/*.md`): under 32,000 bytes.

Never raise a budget.
Move domain detail out of `.rules`, split a `docs/agents/` page into sibling pages by sub-trigger, or split `HOW-TO.md` and `STATE.md` into an index and topic guides.

## Mandatory: archive

Move a document that only records history to an `archive/` folder beside its active docs.
Agents skip `archive/` unless the task is about history.
Files under `archive/` are exempt from `doc-size` and `doc-index`.

## Mandatory: writing style

- Write in Simplified Technical English: short sentences, active voice, present tense where true.
- One idea per sentence, and about 20 words at most.
- Use one word for one meaning; do not rotate synonyms.
- Write instructions as commands: "Run X", not "X should be run".
- Put each sentence on its own line in Markdown.
- Do not use the em dash.
- Use glossary terms as written.
- Show a literal sample (a command, an output, an error line) instead of describing one.

## Mandatory: updating rules and docs

Follow these steps whenever you add or change a rule or document:

1. Check scope first: does the rule apply to every task in the Project?
   If yes, add one short line to `.rules` and keep `.rules` under 8,000 bytes.
   If it applies to one workflow across the Project, put it in the owning `docs/agents/<TASK>.md` page.
   If it applies to one domain, put it in that domain's `RULES.md`, `HOW-TO.md`, or `STATE.md`.
2. Never put multi-paragraph procedures, feature exceptions, or current-state details in `.rules`; link to the owning page from the routing map instead.
3. Update the parent index (`.rules`, `MAP.md`, `GLOSSARY-MAP.md`, `HOW-TO.md`, or `STATE.md`) with a concrete read trigger whenever you add, move, or archive a doc, as [doc-index](../checks/doc-index.md) requires.
4. When a `docs/agents/` page nears 16,000 bytes, move sub-triggered rules to a sibling page and link it from the main page with the action that triggers reading it.
5. Delete a prose rule once a check enforces it, leaving only one line that links the check spec.
6. Do not add a rule for a mistake that happened once; a repeated mistake becomes a Correction; see [corrections.md](./corrections.md).
7. When you add, remove, or rename a skill, or change what the Owner types or approves, update `docs/owner-guide.md` in the same change; [skills-in-owner-guide](../checks/skills-in-owner-guide.md) enforces this.
8. Changes to `.rules`, `docs/agents/*.md`, and domain `RULES.md` are Tier 3 under [test-oracle-guard](../checks/test-oracle-guard.md) and [trusted-ci](../checks/trusted-ci.md); they require Owner approval.
