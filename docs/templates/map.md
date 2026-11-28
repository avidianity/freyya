# MAP template

Save as `MAP.md` at the Project root once the Project has more than one domain or external system boundary.

````markdown
# System map

<One or two sentences: what the system hosts and how it fits alongside external systems.>

## Systems and ownership

| System | Responsibility | Ownership |
| ------ | -------------- | --------- |
| <system> | <What it owns> | <Team or repo> |

## Runtime topology

<Concise overview of processes, data stores, and trust boundaries.>

## Domains

| Domain | Folder | Read when you |
| ------ | ------ | ------------- |
| <domain> | [<path>](<path>) | <Exact trigger for entering this domain> |

## Boundaries

- <Data wall, import restriction, or cross-domain call rule.>
````

Update `MAP.md` in place when system topology or domain boundaries change.
Keep it under 32,000 bytes; move per-domain detail into each domain's `STATE.md`, `HOW-TO.md`, or `RULES.md`.
