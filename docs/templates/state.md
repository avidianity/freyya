# STATE template

Save as `STATE.md` in the domain's folder (co-located with the module, or under `docs/domains/<domain>/STATE.md`).

````markdown
# <Domain> current state

<One or two sentences: what this domain owns today.>

## What is live

- <Implemented capability, active rollout state, or supported flow.>

## Active boundaries and integrations

- <External system, upstream contract, or internal domain boundary it depends on.>

## Known limits and gaps

- <Current constraint, temporary limit, or deferred scope, with issue or ADR link.>

## Topic state guides

<Use this section only when `STATE.md` splits into `./state/<topic>.md`.>

| Guide | Read before you change |
| ----- | ---------------------- |
| [<topic>](./state/<topic>.md) | <Exact trigger or area> |
````

Update this page in place when current state changes.
Keep no timeline or changelog here; move superseded facts to an ADR or `archive/`.
