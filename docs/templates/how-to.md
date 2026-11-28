# HOW-TO template

Save as `HOW-TO.md` in the domain's or module's folder.

## Single-file form

````markdown
# How to use <domain or module>

## Sample

```<language>
<A short call or command that works today, copied from a passing test or live run.>
```

## Rules

- <A rule a caller or operator must follow.>
- <What the domain guarantees, and what it refuses.>

## Errors

| Error | Meaning | What the caller does |
| ----- | ------- | -------------------- |
| <name> | <meaning> | <action> |
````

## Index form (when `HOW-TO.md` nears its size budget)

Split topic guides into `./how-to/<topic>.md`, and replace `HOW-TO.md` with an index table:

````markdown
# <Domain or module>

<One or two sentences: what this domain owns.>
Read this index, then only the guides for the area you change.

| Guide | Read before you change |
| ----- | ---------------------- |
| [<topic>](./how-to/<topic>.md) | <Exact trigger or area that needs this guide> |
````

Keep only current contracts and procedures.
Move history and reasons to an ADR.
