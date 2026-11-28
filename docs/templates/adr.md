# ADR template

Save as `docs/adr/<NNNN>-<slug>.md`, with the next free number.

````markdown
---
status: <proposed | accepted | superseded by NNNN>
---

# <The decision, as a short statement>

<One or two sentences: who decided, when, and what prompted it.>

## Context

<The forces at play, the facts, and the constraints. Name each external source and the date you checked it.>

## Decision

<What we do. Bold the key choice.>

## Considered options

- **<Option>.**
  Rejected: <reason>.
- **<Option>.**
  Rejected: <reason>.

## Consequences

- <What becomes easier.>
- <What becomes harder.>
- <What must now stay true.>
````

Write an ADR only when the decision is hard to reverse, would surprise a later reader, and came from a real trade-off.
