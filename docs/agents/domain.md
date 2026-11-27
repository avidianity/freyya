# Domain language instructions

This page applies before you add or change a domain term, or when a word in a task is unclear.

## Mandatory: glossaries

The Project's domain language lives in glossaries named `GLOSSARY.md`.
A glossary defines terms; it holds no implementation detail, no plan, and no decision.

A Project with one domain has one `GLOSSARY.md` at the root.
A Project with several domains has `GLOSSARY-MAP.md` at the root.
It names each domain, where its `GLOSSARY.md` lives, and which glossary owns each shared term.

Use [templates/glossary.md](../templates/glossary.md) for the format.
The pinned `domain-modeling` skill reads and writes glossaries and ADRs under these names.

## Mandatory: create lazily

Create a glossary when the first term is settled, not before.
Create `docs/adr/` when the first ADR is needed.

## Mandatory: use the terms

- Read the owning `GLOSSARY.md` before you use or introduce a domain term.
- Use each term exactly as written, in code names, documents, issues, and messages.
- Do not use a word that the glossary lists under "Avoid".

## Mandatory: settle unclear terms

When a task uses a term that conflicts with the glossary, say so before you continue.
For example: "The glossary defines a Cancellation as the whole order. You seem to mean one item. Which one?"

When a task uses a vague or overloaded word, propose one precise term, with a short example that shows the boundary.
The meaning of a term is a Decision; the Owner settles it.
Add the settled term to the glossary in the same change.

## Mandatory: conflicts with ADRs

When a term conflicts with an accepted ADR, flag it.
Do not change the ADR or the term alone.
