# Correction template

Use this body for an issue labelled `correction`.

````markdown
# Correction: <the failure class>

## Failures seen

- <Link to a pull request, issue, or session, with a short description.>
- <A second occurrence. A Correction needs at least two.>

## Impact

<Who is affected, how badly, and how likely.>

## Root cause

<A missing invariant, a confusing API, an unclear requirement, or missing tooling.>

## Rule

<The allowed and the forbidden behavior, stated so that a machine could decide it.>

## Enforcement

Lowest layer that can stop it: <simplify design | database | type or API | static rule | test | runtime verification | model checking | instruction>

Why not a lower layer: <reason>

## Cases

- Failing case: <an input the check must reject>
- Passing case: <an input the check must accept>

## Where it runs

<Hook, CI, or both, and the check spec name.>

## Cost

- Affected modules: <list>
- CI time added: <estimate>
- Existing violations: <count, and the plan to fix them>

## Removal

<When this check could be relaxed or deleted.>

## Done when

The check has merged, the failing case fails, and the passing case passes.
````
