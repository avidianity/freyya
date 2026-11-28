# Findings: real Saves and Game files

Research date: 2026-11-28.
Spec: [#7](https://github.com/avidianity/freyya/issues/7).
Game: Victoria 3 1.15.2, no DLC, no mods, on the Owner's PC.
This report grows one section per Ticket; the Owner approves it as a whole.
It is the findings report of [#18](https://github.com/avidianity/freyya/issues/18) and covers the checklist of [#4](https://github.com/avidianity/freyya/issues/4).
Two topics have their own notes, and this report only sums them up: the [telemetry mod](telemetry-mod.md) and the [AI cost estimate](ai-cost-estimate.md).

## Summary for the Owner

- Freyya reads the Owner's real binary Saves in under 1 second each, through the Rakaly program and the Jomini library.
- Treasury, expenses, construction, building levels, staffing, and employment match the game screen.
- Income, balance, market prices, and market orders differ from the screen by 1 to 3 percent, or more for orders.
  The Save holds last week's ledger; the screen seems to recalculate live.
- The Save stores no pop demand per good and no construction capacity.
  Freyya must estimate both from Observations and Game files.
- Reliable first Advice: money reserve, construction queue, building staffing, and unemployment.
  Market shortage Advice needs more work first.
- The telemetry mod is not worth a test now.
- AI is cheap: the chosen starting model costs about 0.01 USD per day at normal use.
- Next step: join Snapshot and Game files for Estimates, then history and alerts.

## Save reader

Ticket: [#8](https://github.com/avidianity/freyya/issues/8).

### Choice: Rakaly CLI decodes, Jomini parses

Freyya pipes each Save through Rakaly CLI 0.8.26 (`rakaly melt --to-stdout --unknown-key error --format vic3`) and parses the melted text with the Jomini 0.39.0 Rust library.
Before decoding, Jomini checks the Save header, finds the zip end, and verifies the gamestate CRC32.

Evidence:

- A 1.15.2 Save is binary even outside Ironman.
  The manual Save header is `SAV0105...`: kind 5, a zip holding a binary `gamestate` and `meta`.
- Binary Saves store field names as 16-bit tokens.
  Jomini can parse binary data only with a token list, and the `vic3save` crate's README says: "Per PDS counsel, the data to construct such a TokenResolver is not distributed here" ([rakaly/jomini `crates/vic3save/README.md`](https://github.com/rakaly/jomini/tree/master/crates/vic3save), read 2026-11-28).
  `vic3save` is also not published on crates.io.
- Rakaly CLI embeds its token lists at build time ([rakaly/cli `src/tokens.rs`](https://github.com/rakaly/cli/blob/master/src/tokens.rs), read 2026-11-28), so it is the only public decoder for binary Saves.
  Its changelog lists "Update to support VIC3 1.15.0 saves" in 0.8.23; 0.8.26 (2026-11-22) is the latest release ([CHANGELOG](https://github.com/rakaly/cli/blob/master/CHANGELOG.md), read 2026-11-28).
- Freyya therefore follows the Spec's fallback: Jomini alone cannot read the Owner's Saves, so the Rakaly CLI decodes them.

### 1.15.2 coverage

| Save | Result |
| ---- | ------ |
| `sweden_1837_08_16.v3` (manual, committed in `fixtures/saves/`) | Melts with `--unknown-key error`: no unknown token. Version 1.15.2, date 1837-08-16, Sweden (SWE). |
| `autosave.v3`, `autosave_1.v3` to `autosave_4.v3`, `autosave_exit.v3` | All read. All belong to a Philippines (PHI) Campaign, 1837-02-01 to 1837-06-01, not to the Sweden Campaign. |

The manual Save is 12.5 MB; its `gamestate` unpacks to 87.6 MB and melts to 120.6 MB of text.
A release build reads it and prints the result in about 0.8 seconds on the Owner's PC.
See [Measurements](#measurements) for time and memory.
Later on 2026-11-28 the Owner played on, and the autosaves moved to 1838 (see [Fixtures](#fixtures)).

### Refusals

`freyya_core::read_save` returns a Snapshot or one typed refusal:

| Refusal | When |
| ------- | ---- |
| Damaged | The header is not a Save header, the gamestate CRC32 or size does not match, Rakaly fails, or the text lacks a player or date. |
| Incomplete | The header says zip but no zip end exists, as in a cut or half-written file. |
| UnsupportedVersion | `meta_data.version` is not a checked version; only `1.15.2` is checked. |
| Modded | `meta_data.mods` lists one or more mods. |
| ReaderUnavailable | The Rakaly CLI cannot be started. |

Each Damaged and ReaderUnavailable refusal keeps the original error as its cause, so `std::error::Error::source` returns it.

The `mods` field name comes from Pdx-Unlimiter's Victoria 3 editor adapter, which reads `meta_data.mods` and `meta_data.dlcs` ([crschnick/pdx_unlimiter](https://github.com/crschnick/pdx_unlimiter), read 2026-11-28).
The Owner's vanilla Saves have neither field, so a hand-made text Save covers the modded case.

### How to repeat

1. Download `rakaly-0.8.26-x86_64-pc-windows-msvc.zip` from the [0.8.26 release](https://github.com/rakaly/cli/releases/tag/v0.8.26); its SHA-256 is `717aee5d3a2bd67455f77fed6f26ec2aaa6866832fb15baaced0161a00e604dd`.
2. Put `rakaly.exe` on `PATH`, or pass `--rakaly <PATH>`.
   `freyya-cli` stops at startup with a clear message when it cannot find Rakaly or the game folder.
3. Run:

```text
> cargo run -q -p freyya-cli -- read-save fixtures/saves/sweden_1837_08_16.v3
Save:           fixtures/saves/sweden_1837_08_16.v3
Game version:   1.15.2
Game date:      1837-08-16
Player country: Sweden (SWE)
```

A refusal prints one line to standard error and exits with code 65:

```text
Refused Cargo.lock: the Save is damaged: not a Victoria 3 Save: Invalid header
```

### Risks

- Every game patch can add tokens; freyya then waits for a Rakaly release.
  Until then `--unknown-key error` makes Rakaly fail, and freyya reports a Damaged refusal naming the unknown token, not UnsupportedVersion, because the version sits behind the same tokens.
- The application tests need `rakaly` on `PATH`; CI installs 0.8.26 and checks its SHA-256.

## What freyya reads

Tickets: [#9](https://github.com/avidianity/freyya/issues/9) money, [#10](https://github.com/avidianity/freyya/issues/10) goods market, [#11](https://github.com/avidianity/freyya/issues/11) buildings and jobs, [#12](https://github.com/avidianity/freyya/issues/12) construction, [#13](https://github.com/avidianity/freyya/issues/13) Game files.
Source Save: `sweden_1837_08_16.v3`, Sweden (country id 24), read on 2026-11-28.
Each Observation carries a unit and its source field in the Save.
A value that the Save lacks shows as "missing", with a reason when freyya knows one, never as 0.
Run `cargo run -q -p freyya-cli -- read-save <SAVE>` to print every Observation.

### Money

| Observation | Unit | Source in the Save | Manual Save |
| ----------- | ---- | ------------------ | ----------- |
| Treasury | £ | `budget.money` | 355,998.74 |
| Income | £ per week | sum of the 16 `budget.weekly_income` lines | 22,482.04 |
| Expenses | £ per week | sum of the 22 `budget.weekly_expenses` lines | 18,602.32 |
| Debt | £ | `budget.principal` | missing |

- All sources sit under `country_manager.database.24`.
- The Save writes `principal` only for a country in debt; 94 countries have it.
  A country in debt has `money=0`.
- Income minus expenses equals `balance_trend.current` (3,879.72) exactly, so the Save agrees with itself.
- In the Save but not shown yet: credit limit (`credit`, 4,461,981), investment pool (`investment_pool`, 31,009.86), and per-building incomes and expenses (`country_building_budget`).

### Goods market

| Observation per good | Unit | Source in the Save |
| -------------------- | ---- | ------------------ |
| Price | £ per unit | `budget.current_price_report.goods.<GOOD>.value` of the market owner |
| Sell orders from buildings | units per week | sum of `output_goods` of buildings in the market's states |
| Buy orders from buildings | units per week | sum of `input_goods` of buildings in the market's states |
| Buy orders from pops | | always missing |
| Net exports | trade volume | sum of `trade.goods.<GOOD>.value` of the market's states |

- Sweden's market 8 holds Sweden and Norway, states 129 to 133 and 483 to 485.
- `market_manager.database` holds only the owner; it has no prices and no orders.
- The Save has no total of buy orders or sell orders per market.
- The Save stores pop spending as money (`weekly_budget`), not as goods, so pop demand is missing.
- A good with no entry in any building's goods list or any state's trade list shows its orders or net exports as missing, not as 0.
- The Save names each good by a number: its position in `common/goods/00_goods.txt` (0 ammunition, 7 grain, 25 iron, 35 tools; 54 goods).

### Buildings

- For each player state: building id, key, readable name, levels, production methods, staffing, and employees.
- Source: `building_manager.database.<ID>`, with employees summed from `pops.database.*.workforce` by `workplace`.
- `staffing` is in staffed levels: employees divided by jobs per level.
  A rye farm with 3 levels and 10,799 employees has staffing 2.70.
- Jobs per level come from Game files, not from the Save.
- A building under construction has `levels=0` and no staffing, so staffing shows missing.
- In the Save but not shown yet: `goods_cost`, `profit_after_reserves`, `salary_rate`, and owners.

### Construction

| Observation | Unit | Source in the Save |
| ----------- | ---- | ------------------ |
| Government and private queue, item by item | | `government_queue` and `private_queue` `.construction_elements` |
| Building and state per item | | `type`, `state` |
| Points left per item | construction points | `construction_left` |
| Speed of items under construction | construction points per week | `base_construction_speed` |
| Construction in use | construction points per week | sum of `base_construction_speed` |

- Each item is one level.
- An item not started shows its full cost as points left (ranch 200, iron mine 400, arms industry 600).
- The Save has no construction capacity field.
  Construction in use equals capacity only while both queues have an item under construction.
- A country with nothing queued has no queue key; freyya reads that as an empty queue.
- Sweden builds in three Norwegian states (483 to 485) that belong to country 23.

### Jobs

| Observation | Unit | Source in the Save | Manual Save |
| ----------- | ---- | ------------------ | ----------- |
| Salaried workforce | people | `pop_statistics.population_salaried_workforce` | 438,670 |
| Subsisting workforce | people | `population_subsisting_workforce` times 100,000 | 378,501 |
| Unemployed workforce | people | `population_unemployed_workforce` | 36,823 |
| Vacancies | people | `vacancies_new_culture` times 100,000 | 105 |

- Some `pop_statistics` fields store people in units of 100,000.
  Proof: salaried plus subsisting plus unemployed equals the total workforce, 853,994.
- The Save leaves out a key when its value is 0; freyya shows it as missing.
- `vacancies_new_culture` is not all unfilled jobs.
  A rye farm 0.3 levels short of staff (about 1,200 jobs) has no vacancies key.
- Each state has the same fields, and the states add up to the country.

### Game files catalog

Run `cargo run -q -p freyya-cli -- check-game-files` on the Owner's PC.
Output on 2026-11-28, about 1 second:

```text
version             pass  1.15.2
English names       pass  108217 names
goods               pass  54 definitions, all named
buildings           pass  122 definitions, all named
production methods  pass  450 definitions, all named
technology          pass  179 definitions, all named
laws                pass  148 definitions, all named
pop needs           pass  15 definitions, all named
defines             pass  3591 defines
state regions       pass  782 definitions, all named
Catalog:    version 1.15.2
  goods               54
  buildings           122
  production methods  450
  technologies        179
  laws                148
  pop needs           15
  state regions       782
  defines             3591
```

- The catalog reads the version from `launcher/launcher-settings.json` (`rawVersion`).
- It holds keys, English names, and defines (game constants).
  It does not hold values yet, such as base prices, production method inputs and outputs, or jobs per level.
- Jomini accepts some broken files silently, such as an unclosed `{`, so "pass" does not prove a file is well formed.
- Defines that use script math, such as `@[1/starvation_threshold]`, stay as raw text.

## UI check

Ticket: [#14](https://github.com/avidianity/freyya/issues/14).
The Owner took 15 screenshots of the paused manual Save on 2026-11-28.
The screenshots stay outside the repository.
The game rounds what it shows, such as "£355K".

| Value | Freyya | Screen | Result | Explanation |
| ----- | ------ | ------ | ------ | ----------- |
| Treasury | 355,998.74 | £355K | match | |
| Expenses | 18,602.32 | 18.59K | match | |
| Income | 22,482.04 | about 22.29K | mismatch, unexplained | Same 8 lines; against the screen, freyya's tolls are 146 higher, dividends 31 lower, and tariffs 10 higher. |
| Balance | +3,879.72 | +3.73K | mismatch, unexplained | Follows income. |
| Debt | missing | none | match | |
| Construction per week | 15.00 | 15.0 | match after fix | Was 25, an AI planning value in the Save. |
| Queue item 1 speed | 7.50 | 7.5 | match after fix | Was 7.77 from `construction_speed`; its meaning is unknown. |
| Queue item 1 time left | 25.35 points | 4 weeks | match | 25.35 / 7.5 = 3.4 weeks. |
| Government queue | 23 items | 1 + 22 | match | |
| Private queue | 1 item | 1 + 0 | match | |
| Iron price | 40.09 | £39.5 | mismatch, unexplained | The only price in the Save is last week's report. |
| Iron sell orders | 314.60 | 311 | small mismatch | |
| Iron buy orders | 143.01 | 307 | mismatch, partly explained | The screen adds trade center exports, 150: trade volume 30 times Game files `traded_quantity` 5. Building orders still differ by about 9 percent. |
| Svealand building levels | listed | listed | match | |
| Port employees (Svealand) | 895 | 895 | match | |
| Salaried, subsisting, unemployed | 438,670; 378,501; 36,823 | 438K; 378K; 36.8K | match | Confirms the 100,000 scale. |
| Vacancies | 105 | not shown | not checked | |

The likely cause of the open mismatches: the screen recalculates income, prices, and orders live, and the Save holds last week's values.
One test would settle it: a Save and screenshots right after a weekly tick, then again one game day later.

## Measurements

Ticket: [#15](https://github.com/avidianity/freyya/issues/15).
Command: `freyya-cli measure-save <SAVE>...`, release build, Rakaly 0.8.26, 12 logical CPUs, 2026-11-28.
Other builds ran at the same time, so expect about 25 percent noise.

| Save | Size | Game date | Check | Decode (Rakaly) | Parse | Total |
| ---- | ---- | --------- | ----- | --------------- | ----- | ----- |
| `sweden_1837_08_16.v3` | 12.5 MB | 1837-08-16 SWE | 82 ms | 553 ms | 80 ms | 721 ms |
| `autosave_4.v3` | 13.5 MB | 1838-07-01 PHI | 90 ms | 575 ms | 88 ms | 757 ms |
| `autosave_exit.v3` | 13.8 MB | 1838-11-06 PHI | 90 ms | 595 ms | 88 ms | 777 ms |

- Memory: about 210 MB for freyya and 28 MB for Rakaly during one read.
  Most of it is the 120 MB melted text; streaming it would cut this.
- Snapshot size as compact JSON: 89 KB (Sweden) and 51 KB (Philippines).
  Hundreds of Snapshots on disk cost only tens of MB.
- Freyya recognises a Save it has already read by a hash (a short fingerprint) of its content.
  This takes about 3 ms, against about 750 ms for a full read.
- No late-game Save exists yet, so late-game cost is not measured.

## Telemetry mod

Ticket: [#17](https://github.com/avidianity/freyya/issues/17); details in the [telemetry mod note](telemetry-mod.md).

- A mod can write numbers to `debug.log` once per game month; 1.15.2 has no daily or weekly script pulse.
- The mod changes the game checksum, so it disables achievements on an existing Campaign.
- Under Owner decision 8, a modded Campaign gets no Advice.
- Both linked mods are unmaintained, and the Workshop mod is removed and flagged incompatible.
- Conclusion: a hands-on test is not worth it now.
  Ask again only if Saves leave a real gap; if so, test on a throwaway Campaign.

## AI cost and the AI-free option

Ticket: [#16](https://github.com/avidianity/freyya/issues/16); method, prices, and tables in the [AI cost estimate](ai-cost-estimate.md).

- Compact summaries of real Saves are 65 to 1,155 tokens; the full `read-save` output is 14,201 tokens.
- A cheap model costs 0.003 to 0.11 USD per day; a capable model costs 0.12 to 2.50 USD per day.
- The 0.50 USD per day default covers normal use with a cheap model or an 80/20 mix.

The Owner chose DeepSeek Flash on OpenRouter as the starting model on 2026-11-28 ([product brief](product-brief.md#part-2-owner-decisions-from-the-grill-2026-11-28)).
OpenRouter listed `deepseek/deepseek-v4.1-flash` at 0.051 USD per million input tokens and 0.567 USD per million output tokens.
With the token assumptions of the AI cost estimate and no caching, this report calculates:

| Use | Questions per day | Cost per day |
| --- | ----------------: | -----------: |
| Light | 5 | 0.003 USD |
| Normal | 15 | 0.013 USD |
| Heavy | 40 | 0.064 USD |

DeepSeek uses its own tokenizer (the rule that cuts text into tokens), so add about 20 percent as a safe margin.
A settings page lets the Owner change the model, and later the provider if freyya supports more than OpenRouter.

An AI-free freyya keeps data, alerts, Candidate actions, Estimates, and history.
It needs about 17 to 25 Tickets for the five first areas; only 3 to 6 of them exist because AI is absent.
It loses plain-language Objectives, free-form questions, and explanations that join several areas.
Recommendation: build the deterministic core first, because both options need it.
Add AI as a thin, optional layer that calls the same tools.

## Which Advice is reliable

"Reliable" means the inputs match the game screen in the UI check.

| Area | Reliable now | Not reliable yet, and why |
| ---- | ------------ | ------------------------- |
| Money | Treasury and expenses; in debt or not; weeks of reserve at the current balance, within about 4 percent. | Exact income and balance (about 150 £ per week off). Credit Advice: the credit limit is not shown or checked. |
| Goods markets | Which goods cost more or less than base price, roughly; which buildings sell a good. | Shortage size and buy minus sell: pop demand is missing and exports need Game files. Exact price (about 1.5 percent off). |
| Buildings | Levels, production methods, staffing, understaffed buildings. | Profit and "build more of X": profit fields are not checked, and gains need Estimates. |
| Construction | Queue contents, order, points left, speed of items under construction, time to finish item 1. | Capacity when a queue is empty; time to finish later items; idle capacity. |
| Jobs | Employed, peasants, unemployed per country and per state. | Vacancies: the only Save field misses most unfilled jobs and no checked screen shows it. |

## Numbers that need Estimates

An Estimate is a value freyya computes from Observations and Game files under stated assumptions.

| Estimate | Formula and assumptions |
| -------- | ----------------------- |
| Construction capacity | 10 base plus 5 per staffed level of a construction sector with iron frame buildings, from Game files. Assumes no other modifiers. |
| Time to finish an item | Points left divided by the share of capacity it gets. Assumes capacity and the 50 percent private share stay the same. |
| Item cost | Full construction cost of the building from Game files. |
| Exports and imports in goods | Trade volume times the good's `traded_quantity` from Game files. Exact for iron in the UI check. |
| Pop demand per good | From pop needs in Game files and pop money budgets. Assumes the game's buy packages; unchecked. |
| Unfilled jobs | Levels times jobs per level from Game files, minus employees. |
| Output and input of one more level | Production method inputs and outputs from Game files, times throughput. Assumes the same prices and staffing. |
| Profit of one more level | Output value minus input value minus wages, at current prices. Assumes prices do not move. |
| Weeks of reserve or credit | Treasury, or credit left, divided by the weekly deficit. Assumes a constant deficit. |
| Price after a change | Base price times (1 + 0.75 times (buy minus sell) divided by the smaller of the two). Fits iron and grain on screen; unconfirmed. |

The catalog must first read values (base prices, costs, production methods, jobs per level, `traded_quantity`), not only keys.

## Numbers only the engine knows

The game calculates these and does not write them to the Save:

- The live income, balance, and prices that the screen shows between weekly ticks.
- Construction capacity as one number, and the full list of modifiers behind it.
- Pop consumption per good.
- How tolls split by source, such as strait tolls.
- What `construction_speed` (7.77, against a base of 7.5) means.
- AI decisions, private investment choices, and future events.
- The exact price formula and modifier stacks; script docs show parts, not the whole.

## Architecture

- `freyya-core` is the headless core: a library with no screen code.
  It holds Save reading, the Game files catalog, Snapshot areas, and the "already read" check.
- Public entry points: `read_save` (Save to Snapshot or a typed refusal), `SaveReader` (the same call, which also recognises a Save already read), `read_game_files`, and `check_game_files`.
- The area modules are private. The core exports only the Snapshot types and one `Observation` type.
  Each Observation has a `Reading` (a read value, or missing with an optional reason), one `Unit`, and its source in the Save.
  The CLI shows every Observation through the same display.
- `freyya-cli` drives the core from the command line for every investigation step.
- `freyya` is the gpui window (gpui is the screen toolkit).
  It stays a thin screen that calls the core and never reads the Save itself.
- Rakaly runs as a separate program; freyya pipes the Save in and reads text out.

Add these layers to the core, in this order, each behind a small public call:

1. Estimates that join one Snapshot with the catalog of the same game version.
2. Alert rules and Candidate actions that use Observations and Estimates only.
3. Snapshot history in the local app-data folder, not OneDrive.
4. An optional AI adapter in its own crate (a Rust package) that calls the same tools.
   The gpui settings page picks model and provider.

## Unresolved concerns and maintenance risks

- **Rakaly and withheld tokens.**
  Paradox does not publish the token list, so only Rakaly decodes binary Saves (see [Risks](#risks)).
  No other way to read binary Saves is verified.
- **Modded-Save check unverified.**
  Freyya refuses a Save that lists `meta_data.mods`, but this key comes from another tool and was tested only on a hand-made Save.
- **Live screen values.**
  Income, balance, prices, and market orders differ from the screen; the cause is a guess until a second UI check.
- **One early Save only.**
  Late-game size, time, memory, and debt cases are not tested on a real Save.
- **Vacancies** have no checked source.
- **Memory**: one read holds about 210 MB; streaming would cut this.
- **Saves already read** stay in memory without a limit; a long-running watcher needs one.
- **Jomini is lenient** with broken Game files, so the catalog check can pass on a damaged file.

## Implementation plan

Each step is one small Ticket that the Owner can approve alone.

1. Read values into the catalog: base prices, construction costs, production methods, jobs per level, `traded_quantity`.
2. Market view: join Snapshot and catalog for exports, imports, and price against base.
3. Show credit limit, investment pool, and building profit fields; check them on screen.
4. Second UI check after a weekly tick, plus a vacancies screen and one Philippines Save.
5. Commit the chosen fixtures and assert their checked values in tests.
6. Snapshot history in the local app-data folder.
7. Alert rules for money and construction, the two areas that match the screen.
8. Estimates: construction capacity, time to finish, unfilled jobs.
9. Alert rules for jobs, buildings, and goods markets.
10. Candidate actions for "build one more level", with Estimates and a fixed ranking.
11. gpui screens over the same calls.
12. Save watcher, with Owner permission to run it.
13. Optional AI adapter with DeepSeek Flash on OpenRouter, and the settings page.

Decisions left to the Owner after this report: the AI budget, model routing, where the AI key lives, and feature order.

## Fixtures

The committed fixture is `fixtures/saves/sweden_1837_08_16.v3` (12.5 MB).
The Owner's active Campaign is now Philippines 1838 in the autosaves, each about 13.7 MB.
All are under the 50 MB limit of [ADR 0005](../adr/0005-commit-real-saves.md).
The game overwrites autosaves in turn, so a copy must come from a stable file.

Recommendation:

- Keep the Sweden manual Save; the UI check values belong to it.
- Add one Philippines Save at 1838: the Owner makes a manual Save, and the agent copies it on request.
  It adds a second country, a later date, and a different market.
- Add a later Save when the Campaign reaches the mid-game, if it stays under 50 MB, to measure late-game cost.
- Keep hand-made text Saves for an unknown version, a damaged Save, and a modded Save.

## Checklist of #4

| Item of #4 | Section |
| ---------- | ------- |
| Values from real Saves, with units and missing fields | [What freyya reads](#what-freyya-reads) |
| Game files catalog | [Game files catalog](#game-files-catalog) |
| Compare with screenshots | [UI check](#ui-check) |
| Read time, memory, Snapshot size | [Measurements](#measurements) |
| Telemetry mod on paper | [Telemetry mod](#telemetry-mod) |
| AI cost estimate and AI-free option | [AI cost and the AI-free option](#ai-cost-and-the-ai-free-option) |
| Reliable Advice | [Which Advice is reliable](#which-advice-is-reliable) |
| Architecture, concerns, plan | [Architecture](#architecture), [concerns](#unresolved-concerns-and-maintenance-risks), [plan](#implementation-plan) |
| Fixtures | [Fixtures](#fixtures) |
