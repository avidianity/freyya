# Freyya

Freyya reads a single-player Victoria 3 campaign and tells the Owner what is happening in its economy and what to consider doing.
These terms come from the Owner's purpose statement at Bootstrap; `grill-with-docs` refines them.

## Language

**Save**:
A file that Victoria 3 writes to record one moment of a **Campaign**.
_Avoid_: savegame, save file (in prose)

**Campaign**:
One playthrough of Victoria 3, from its start date onward, across all of its **Saves**.
_Avoid_: game, run, playthrough

**Game files**:
The rule definitions that the installed Victoria 3 version ships, such as goods, buildings, and production methods.
_Avoid_: static data, game data, defines (for the whole set)

**Snapshot**:
The facts that freyya extracted from one **Save**, stamped with its game date and game version.
_Avoid_: state dump, record

**Observation**:
A value read directly from a **Snapshot**, such as a treasury balance or a market price.
_Avoid_: fact, metric (when the source matters)

**Estimate**:
A value that freyya computes from **Observations** and **Game files** under stated assumptions, never an exact game calculation.
_Avoid_: prediction, forecast, projection

**Candidate action**:
A player action that freyya checked as allowed in the current **Snapshot**, such as building one more level of a building in a state.
_Avoid_: move, option, suggestion

**Advice**:
A ranked set of **Candidate actions** with the **Observations** and **Estimates** behind each one.
_Avoid_: recommendation (alone), tip

**Objective**:
The Owner's confirmed aim for one **Campaign**, stated as priorities and limits; broad success across the economy, finances, industry, stability, technology, military, living standards, and influence when none is set.
_Avoid_: goal, strategy, win condition

## Relationships

- A **Campaign** has one or more **Saves**.
- Each **Snapshot** comes from exactly one **Save**.
- An **Estimate** depends on one **Snapshot** and the **Game files** of that **Snapshot**'s game version.
- **Advice** cites the **Snapshot** it came from.
- A **Campaign** has one **Objective** at a time, and **Advice** ranks **Candidate actions** against it.

## Flagged ambiguities

- "Recommendation" was used for both one **Candidate action** and the whole ranked set.
  Resolved: the whole ranked set is **Advice**.
