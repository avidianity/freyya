# Planning and building instructions

This page applies when you turn an idea into work, design a change, or build and prove a Ticket.
It names skills that the Project pins in its skills lock file.
Each skill owns its own steps; this page decides when each one runs and what it must produce.

## Mandatory: how agents run pinned skills

Most pinned skills set `disable-model-invocation: true`.
Never start such a skill on your own, even in an agent that ignores that key.
The Owner starts a stage by asking for it, or by naming its skill.
That request permits every skill that this page assigns to the stage and to the Expected Tier.
To run such a skill, read its `SKILL.md` in the Project's skills folder and follow it.

## Mandatory: the pipeline

Work moves through these stages in order.

| Stage | Question it answers | Skill | Output |
| ----- | ------------------- | ----- | ------ |
| 1. Decide | What do we build, and what does each word mean? | `grill-with-docs` | Decisions in ADRs and the issue, terms in the glossary |
| 2. Chart | What must we decide before a Spec is possible? | `wayfinder` | A Map with decision tickets |
| 3. Spec | What is the whole change, from the user's side? | `to-spec` | One Spec issue |
| 4. Split | Which thin slices, and which slice blocks which? | `to-tickets` | Tickets with Blocking edges |
| 5. Shape | Which types, signatures, and module boundaries? | `architect`, and `arena` for a contested shape | A Design sketch |
| 6. Build | Does the code do what the Ticket says? | `implement` for one Ticket, `implement-spec` for a whole Spec | Commits on a branch |
| 7. Prove | What does the change break, and can an adversary break it? | `code-review`, `blast-radius`, `interrogate` | Verified findings and a Safety fact |
| 8. Land | May it merge? | none; follow [git.md](./git.md) and [review-and-merge.md](./review-and-merge.md) | A merged pull request |

Skip a stage only in these cases:

- Stage 2 runs only when the work is too big for one session, or when open questions hide the way to a Spec.
- A request that fits one Ticket and needs no Decision starts at Stage 4 with one Ticket, or at Stage 5 when the Ticket exists.
- An incoming report goes through `triage` first.
  An issue that `triage` marks ready for an agent enters at Stage 4.
- Stages 5 and 7 scale with the Expected Tier, as the next section says.

## Mandatory: rigor by Expected Tier

The tier check needs a diff, so it cannot run before code exists.
Until then, each Ticket states an Expected Tier, read from the Project's tier map.

| Expected Tier | Shape | Prove |
| ------------- | ----- | ----- |
| 0 | None | `code-review` |
| 1 | None | `code-review` |
| 2 | `architect` | `code-review`, `blast-radius` |
| 3 | `architect`; `arena` when two shapes stay close | `code-review`, `blast-radius`, `interrogate`, and the Code review that the Review mode requires |
| 4 | `arena` | The same as Tier 3; the Owner starts the change or watches it run |

The tier check on the final revision decides the Tier.
When it gives a higher Tier than the Expected Tier, run the missing Shape and Prove steps before the merge.

## Mandatory: Owner gates

- Stage 1 and every grilling or prototype ticket of a Map are conversations with the Owner.
  The agent asks; it never answers its own questions.
- The Owner approves the Ticket breakdown before `to-tickets` publishes it.
  The breakdown shows the Expected Tier of each Ticket, and each published Ticket body has an "Expected Tier" section.
- Approval of a Spec, a Map, or a Design sketch is not permission to build it.
  Stage 6 starts only when the Owner asks to build: an ordinary request for one Ticket, `implement`, or `implement-spec`.
  For Tier 0 to 2 work, the Owner may grant one upfront go-ahead that covers Spec, Ticket breakdown, and Build (`to-spec` through `implement-spec`).
  Even under one upfront go-ahead, stop and ask the Owner if any Ticket reaches Expected Tier 3 or 4, if the tier check raises a change to Tier 3 or 4, or if work needs a Decision outside the Standing Decisions ADR.
  An ordinary request that fits one Ticket and needs no Decision may skip publishing the Ticket to the tracker.
- A choice that an experiment can settle is not a Decision: prototype it, and recommend the option that the result supports.
  A choice about product behavior or preference is a Decision for the Owner.

## Mandatory: Tickets

- Each Ticket is one thin slice through every layer that the change touches.
  It can be shown or verified on its own, and it fits one fresh agent session.
- Each Ticket names its Blocking edges with the tracker's native relationship, as [issue-tracker.md](./issue-tracker.md#mandatory-specs-tickets-and-maps) says.
- Each Ticket states its Expected Tier.
- A wide mechanical change, such as a rename across the codebase, is split as expand, migrate in batches, contract.
  Each batch is its own Ticket.
- Start only Tickets on the Frontier.
  The width of the Frontier is the most agents that can help at one time; more agents than Frontier Tickets add queue time, not finished work.

## Mandatory: design in code

- A Design sketch is code: types, signatures, and module boundaries, with no bodies.
  It is not a prose design document.
- Put the Design sketch in a comment on the Ticket, or in the first commit of the Ticket's branch.
- `architect` sketches two different shapes and picks one in which each piece has one owner and callers need to know the least.
- `arena` runs several agents on the same shape question, picks a base, and takes the strongest parts of the others.
- A Design sketch that needs a new dependency, service, or pattern stops for a Decision.

## Default: build a whole Spec

`implement-spec` builds every Ticket of a Spec on one Integration branch.
It starts one fresh implementer agent for each Frontier Ticket, merges each result into the Integration branch, starts new Frontier Tickets as their blockers close, and runs `code-review` on the whole branch at the end.

- The Integration branch follows [git.md](./git.md#default-branches), and lands as one pull request.
- That pull request takes the highest Tier of its Tickets.
  When a Spec mixes Tier 3 or 4 Tickets with lower ones, build the high-tier Tickets with `implement` on their own branches.
  Then the low-tier work does not wait for a high-tier approval.
- [delegation.md](./delegation.md) applies to every agent that `implement-spec` starts.

## Mandatory: proof that runs

A finding or a safety claim counts only when a run shows it: a failing test, a script that calls the real code, or command output.
A convincing explanation is not proof.

- `code-review` reviews the diff on two axes, Standards and Spec.
  Fix each finding, then review again.
- `blast-radius` finds what the change breaks outside the diff, and names the Safety fact.
  It proves the Safety fact with a script or test that runs the real code.
- `interrogate` starts adversaries, ideally on models from several vendors, that try to break the change.
  It reports only findings that a rerun confirms, and it fixes nothing.
- Put the Safety fact and the verified findings in the Evidence bundle, as [review-and-merge.md](./review-and-merge.md#mandatory-evidence-bundle) says.
- A verified `interrogate` finding blocks the merge until a fix lands and the rerun no longer shows it.
- At Tier 3 and 4, an unproven Safety fact blocks the merge.
  At Tier 2, list it under Gaps.
- `interrogate` and `arena` read the Owner's model settings from the Owner's own agent setup.
  The Project never commits them.
- When the Project opts out of `pstack` in an ADR (see [skills/README.md](https://github.com/avidianity/factory/blob/v0.9.0/skills/README.md#opting-out-of-pstack)), Shape writes a direct Design sketch in code (types, signatures, and module boundaries with no bodies), and Prove runs `code-review`, fail-first and runtime evidence, and at Tier 3 and 4 an Independent reviewer using [methods/review-prompts.md](https://github.com/avidianity/factory/blob/v0.9.0/methods/review-prompts.md#independent-verifier).

## Default: briefs and long runs

- Before an agent starts a Ticket of Expected Tier 2 or higher, run `sharpen` on its prompt.
  `sharpen` makes the task deeper, not bigger: keep the Ticket's scope and acceptance criteria as written.
- An unattended run of more than one Ticket keeps a decision log with `show-me-your-work`.
  Link the log from the pull request when a reviewer needs it.

## Default: helper skills

The pipeline skills load these skills themselves.
Do not start them as separate stages.

| Helper | Loaded by |
| ------ | --------- |
| `grilling`, `domain-modeling` | `grill-with-docs`, `wayfinder`, `triage` |
| `research`, `prototype` | `wayfinder` |
| `tdd` | `implement`, `implement-spec` |
| `codebase-design` | `tdd` |
| `in-your-own-words` | `interrogate` |
| `how`, `why`, `unslop` | `blast-radius` |

`grill-me` grills the Owner on any plan without the glossary.
`wait-what` explains a confusing message or change in plain words.
