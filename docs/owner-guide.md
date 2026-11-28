# Working with agents in freyya

This page is for you, the Owner.
It tells you which skill to start, what to type, what you get back, and what you approve.
Agents follow `.rules` and `docs/agents/`; you do not need to read them.

## Your job and the agents' job

- You make every Decision: a choice about product behavior, data, cost, security, or architecture.
  An agent asks; it never answers its own questions.
  You may pre-approve bounded routine choices in one Standing Decisions ADR; Tier 3 and Tier 4 choices are never delegated.
- Agents find facts, write the code, run the checks, and prove that the change works.
- The Review mode is Never.
  You never read code: for a risky change, a fresh reviewer agent checks the code first, then you read a short plain-language Risk summary and answer yes or no.
  You can change it at any time; ask an agent to record the new mode in an ADR.

## How to start a skill

A skill is a written procedure that an agent follows.
Most skills here start only when you ask for one, so an agent never starts them alone.
Starting a skill is your permission for the work it describes, and for nothing more.

| Agent | Start a skill | See all skills |
| ----- | ------------- | -------------- |
| Claude Code | Type `/<SKILL>`, then your request on the same line | Type `/skills`, or type `/` |
| Codex | Type `$<SKILL>` anywhere in the message | Type `/skills`, or type `$` |
| Antigravity (`agy`) | Type `/<SKILL>` in the CLI or the app; in the IDE, ask for the skill by name | Type `/skills` in the CLI |

Put your request after the skill name, on the same line:

```text
/grill-with-docs Show me which goods are short in my main market, and why.
```

When a command does not work, ask in plain words, for example "Run the grill-with-docs skill."
The rules tell the agent to read the skill and follow it.

## Pick a skill

| When you | Start | You get |
| -------- | ----- | ------- |
| Have an idea for a feature or a change | `grill-with-docs` | Questions until every Decision is settled; ADRs and glossary terms |
| Have an idea too big for one session, or too foggy to describe | `wayfinder` | A Map issue that lists the Decisions to make, one per session |
| Got a bug report or a feature request | `triage` | A labelled issue; `ready-for-agent` when an agent can do it alone |
| Settled the Decisions in this session | `to-spec` | One Spec issue |
| Have a Spec | `to-tickets` | A Ticket breakdown to approve, then Tickets in the tracker |
| Want one Ticket built | `implement` | Commits on a branch and a pull request |
| Want a whole Spec built | `implement-spec` | Every Ticket built on one branch, and one pull request |
| Want a design for a change before any code | `architect` | A Design sketch: types and signatures with no bodies |
| See two designs that stay close | `arena` | Several agents try the design; you get the best one, with parts of the others |
| Want to know what a change can break | `blast-radius` | What breaks outside the change, and the Safety fact, proven by a run |
| Want a change attacked before it merges | `interrogate` | Only the problems that a run proved; it fixes nothing |
| Want a diff reviewed against the Spec and the rules | `code-review` | Findings on two axes: the Project's rules, and the Spec |
| Think a task description is too thin | `sharpen` | A deeper version of the same task, with the same scope |
| Start a long run that you will not watch | `show-me-your-work` | A decision log: one line per choice, with its reason and proof |
| Want your plan questioned, with no documents written | `grill-me` | Questions until the plan holds up |
| Did not understand an agent's last message | `wait-what` | The same message again, in plain words |
| Saw the same agent mistake a second time | `correction` | A `correction` issue, and later a check that stops the mistake |

A small, clear change needs no skill.
Ask for it in plain words, for example "Rename the Export button to Download."

## Walkthrough: from idea to merged change

This example follows one idea through every step.
Each step shows what you type and what happens next.

### 1. Decide

```text
/grill-with-docs Show me which goods are short in my main market, and why.
```

The agent asks questions, each with its recommended answer.
Reply "yes" to take the recommendation, or answer in your own words.
"I do not know, what do you recommend?" is a good answer.
The agent finds facts in the code itself; it asks you only for Decisions.
It records each Decision in an ADR or the issue, and each new word in `GLOSSARY.md`.
It stops when nothing is left to decide, and asks you to confirm.

### 2. Write the Spec

Stay in the same session, because `to-spec` writes from the conversation so far.

```text
/to-spec
```

The agent asks you to confirm where the tests touch the code.
Then it publishes one Spec issue and gives you its link.
Approving a Spec does not start the build.

### 3. Split it into Tickets

```text
/to-tickets https://github.com/avidianity/freyya/issues/12
```

The agent shows the breakdown: each Ticket, its Expected Tier (the risk class from 0 to 4), and which Ticket blocks which.
Ask for changes until the breakdown is right, then approve it.
The agent publishes the Tickets.

### 4. Build

For the whole Spec:

```text
/implement-spec https://github.com/avidianity/freyya/issues/12
```

For one Ticket:

```text
/implement https://github.com/avidianity/freyya/issues/13
```

This request is your permission to build.
The agent runs the design and proof skills that each Ticket's Expected Tier needs, so you do not start them yourself.
When a Ticket needs a new Decision outside your Standing Decisions ADR, the agent stops and asks you.
For Tier 0 to 2 work, you may also give one upfront go-ahead from Spec through Build (for example "Write the Spec, split it, and build it if every Ticket stays Tier 2 or lower"); the agent still stops if any Ticket rises to Tier 3 or needs a new Decision.

### 5. Land

The agent opens a pull request with an Evidence bundle: the proof that the change works.
What you do depends on the change's risk level, from 0 (lowest) to 4 (highest):

- **Levels 0 to 2 (routine):** the agent tells you that the checks passed and the proof is attached.
  For now, you click Merge yourself; later, when the records show agents are safe, agents merge these alone.
- **Level 3 (risky):** a fresh reviewer agent checks the code first.
  Then you get a short Risk summary in plain words and answer yes or no.
  The agent merges only after your yes.
A Tier 4 change never runs alone: you start it, or you watch it run.

## Other ways in

### Work too big for one session

```text
/wayfinder Add advice for laws and institutions, not only the economy.
```

The first session charts a Map and stops.
Each later session settles one Decision on the Map:

```text
/wayfinder https://github.com/avidianity/freyya/issues/20
```

When the Map has no open Decisions, run `to-spec` in that session.

### A bug report or a feature request

```text
/triage https://github.com/avidianity/freyya/issues/31
```

With no link, the agent shows the issues that wait for triage.
An issue marked `ready-for-agent` goes straight to `implement`.

### A repeated agent mistake

```text
/correction Agents run the full test suite before every commit. See https://github.com/avidianity/freyya/pull/40 and https://github.com/avidianity/freyya/pull/44.
```

The agent opens a `correction` issue with one example that the new check must reject and one it must accept.
Ask it to build the check when you want the mistake to stop for good.

## Skills that you do not start

The skills above load these helpers themselves, so you never start them:

- `grilling`, `domain-modeling`, `research`, and `prototype`
- `tdd` and `codebase-design`
- `in-your-own-words`, `how`, `why`, and `unslop`
- `principle-separate-before-serializing-shared-state`, `principle-redesign-from-first-principles`, `principle-prove-it-works`, `principle-laziness-protocol`, and `principle-encode-lessons-in-structure`

## What you approve, and when

- Each Decision outside your Standing Decisions ADR, during `grill-with-docs`, `wayfinder`, or a build that stops to ask.
- Any change to your Standing Decisions ADR (always Tier 3).
- The Ticket breakdown before `to-tickets` publishes it, and the start of each build by starting `implement` or `implement-spec` (or one upfront go-ahead for Tier 0 to 2 Spec through Build).
- Each merge that the Review mode and active autonomy gate give to you; during the prototype agents merge up to Tier 3, and Tier 4 stays with you ([ADR 0006](adr/0006-agents-merge-during-prototype.md)).
- Each Tier 4 operation, which you start or watch.

GitHub will not merge into `main` until a pull request passes the `rust`, `repo`, and `policy` checks.
This also applies to your own account, and no approval is needed ([ADR 0004](adr/0004-public-repo-single-account.md)).

## Setup on your machine

- On a new machine, restore the Project's skills with `npx skills@1.9.0 experimental_install --frozen-lockfile --agent codex`.
- On a new machine, turn on the Git hooks once with `lefthook install`.
  The tools the hooks call are Rust (through `rustup`), Node.js, `lefthook`, and `gitleaks`.
- `arena` and `interrogate` start agents on several models.
  To choose those models, run pstack's `setup-pstack` skill once in your own agent setup.
  Without it, they use their built-in defaults.

## When something goes wrong

- An agent says "run setup-matt-pocock-skills".
  Do not run it.
  Tell the agent: "Read the Agent skills block in `.rules`."
- An agent says it cannot start a skill.
  Tell it: "Read `.agents/skills/<SKILL>/SKILL.md` and follow it."
- An agent's message makes no sense.
  Start `wait-what`.
- To move to a newer Kit version, tell an agent: "Upgrade this project from <https://github.com/avidianity/factory>. Read UPGRADE.md there and follow it."
