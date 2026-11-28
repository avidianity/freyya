# Risk summary template

Use this for an Owner who does not read code.
Write in plain words, with no code.
Explain each new term in five words or fewer, in the same sentence.

````markdown
# Approval needed: <what changes, in plain words>

## What changes for people

<Who notices, and what they notice. One short paragraph.>

## What could go wrong

- <A failure, in plain words.>
  How likely: <low / medium / high>, because <reason>.
- <A second failure, if there is one.>

## How to undo it

<How to undo it, and how long that takes.>
<What cannot be undone, if anything: sent messages, payments, deleted data.>

## Evidence

- Tests: <how many new tests, and that they pass>
- Independent check: <PASS / FAIL / INCONCLUSIVE> by a fresh reviewer that did not write the change
- Tried for real: <what was run against the working product, and the result>

## Recommendation

<Yes or no>, because <one reason>.

**Do you approve? Reply yes or no.**
````
