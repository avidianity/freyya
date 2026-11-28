# Feature map template

Save as `docs/verification/features/<feature>.md`, and add it to the index.

````markdown
# Feature: <what a user can do>

## Entry point

<The route, screen, command, or API that a user starts from.>

## Preconditions

- Build: the revision under test, checked by Doctor
- Role: <the user role>
- Test data: <fixture name and version>

## User actions

1. <Action, with the stable control name.>
2. <Action.>

## Success invariants

- Visible: <what the user sees>
- Authoritative: <the stored record or external effect, checked directly>
- Network: <the request, its method, and its status>

## Negative and edge paths

- <Unauthorized role: refused, and no effect.>
- <Empty, error, loading, or retry state.>

## Evidence

<Which screenshots, traces, and assertions to keep, and where.>

## Cleanup

<What the run removes, and what stays.>

## Gotchas

<Known traps, such as slow loading or a control that moves.>
````
