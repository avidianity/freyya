# Victoria 3 AI Advisor — feasibility and research

Research date: 2026-11-28. Target: single-player, read-only, local companion for Victoria 3 1.15.x. No game memory access or automatic gameplay. This is a proposed scope, not a confirmed specification. No actual save or installed game files were available for testing.

## Conclusion

A snapshot-based advisor is feasible. A faithful simulator or globally optimal planner is a separate, much larger undertaking. Start with verified observations, constrained candidate actions and transparent estimates. Do not attempt to reproduce the game engine. Before estimating the complete build, test the current save format against real campaigns and reconcile extracted values with the in-game UI.

## 1. Public resources and reusable work

| Resource | Reusable capability | Limitation |
|---|---|---|
| [Rakaly CLI](https://github.com/rakaly/cli) / [releases](https://github.com/rakaly/cli/releases) | Decode binary saves; save watching | Version-specific token resolution. Changelog reports support for 1.15.0 in 0.8.23; exact 1.15.2 compatibility untested. |
| [Jomini](https://github.com/rakaly/jomini) | Rust parser for Paradox text and binary data; compressed save envelopes | Low-level parser, not a semantic Victoria 3 data model. Binary input requires correct token resolver. |
| [Garibaldi](https://github.com/OakenTrader/Garibaldi) | Save watcher, construction, innovation, research, demographics, treasury, debt and historical charts | Existing workflow archives then processes saves; not proof of interactive latency. Limited mod support documented. |
| [V3 Save Analyzer](https://github.com/JacksonWeber/v3-save-analyzer) | GDP, population, standard of living, literacy, budgets, state GDP, technology and goods production | Dashboard, not an advisor; verify exact field definitions and version coverage. |
| [Victoria 3 Economy Analyzer](https://github.com/deadcrusade/Victoria-3-economy-analyzer) | Played and world market prices, profitability signals; effective modded goods catalog | Documented target 1.13.9. Capture and analysis separated. |
| [Victoria 3 Analysis](https://github.com/ShabbyGayBar/victoria3-analysis) / [documentation](https://shabbygaybar.github.io/victoria3-analysis/) | Production coefficients, goods flows, optimization, household demand, equilibrium, resource limits | Approximate model, not save-driven engine parity. Inspect license before copying or distributing code/data. |
| [V3 Reference](https://github.com/alcaras/v3ref) / [site](https://alcaras.github.io/v3ref/) | Node/Jomini pipeline extracting structured static game definitions and localization | Requires game data. Not live campaign state. |
| [Vic3 Tabula](https://vic3tabula.com/en/) / [rules](https://vic3tabula.com/en/rules) | Searchable game definitions and official in-game help; rules page identifies 1.15.2 and 687 entries | Fan-hosted, derived data. Dynamic game-state conditions not evaluated. |
| [Modding Digests](https://github.com/Victoria-3-Modding-Co-op/Modding-Digests) | Generated scripting documentation and patch-level file changes | Not a stable application API. |

Current game reference: [official Steam announcements](https://steamcommunity.com/app/529340/announcements/) report 1.15.2. The [Rakaly changelog mirror](https://opencode.terrychan.org/rakaly/cli/blob/master/CHANGELOG.md) reports 1.15.0 support; [release 0.8.23](https://github.com/rakaly/cli/releases/tag/v0.8.23) exists. Do not infer patch compatibility without tests.

## 2. Two distinct data sources

**Runtime save:** current country, date, markets, states, buildings, employment, finances, technology and political state. A save is a serialized game state, not necessarily every UI metric or engine intermediate. Existing tools demonstrate some fields are extractable, but the exhaustive schema, exact field units and completeness are unverified. [Save format reference](https://vic3.paradoxwikis.com/wiki/Save-game_editing); [Jomini envelope documentation](https://github.com/rakaly/jomini).

**Static game files:** installed version-specific rules, goods, buildings, production methods, technologies, professions, pop needs, laws, modifiers, resource caps and localized names. Start with `game/common/{buildings,building_groups,goods,production_methods,production_method_groups,technology,buy_packages,pop_needs,pop_types,defines}` and `game/map_data/state_regions`. Later add other definitions. [Existing parser architecture](https://shabbygaybar.github.io/victoria3-analysis/project_structure/). Resolve inherited groups, variables, script values, overrides and effective load order. Do not ship copyrighted raw game assets by default.

A useful third source is observed history: archive normalized monthly snapshots; track prior advice and player decisions separately. Do not assume causal effects from before/after comparisons.

## 3. Complexity inventory

Difficulty below is an engineering judgment, not a benchmark. Observing an existing metric is much easier than predicting the result of changing it.

| Domain | Observe/diagnose | What makes forecasts difficult |
|---|---|---|
| Treasury | Balance, credit, revenue/expense changes | Interest, taxation, government vs private construction and shifting revenues |
| Goods markets | Current shortages, buy/sell orders, prices | Demand substitution, price caps, trade, changed production and local market pricing |
| Construction | Queue, capacity, available materials, state constraints | Construction allocation, input costs, hiring delay, private investment and completion time |
| Buildings | Levels, PMs, staffing, productivity, profitability | Multiple PM groups, unlocks, throughput, input shortages, local wages/prices, ownership and modifiers |
| Labor | Unemployment, peasants, vacancies, qualifications | Profession switching, wages, migration, discrimination, competing employers |
| Infrastructure | Infrastructure usage, access | Geography, transport capacity, disruption and local prices |
| Population | Standard of living, radicals, wealth, demographics | Changing consumption baskets, income, taxes, migration and political feedback |
| Technology | Completed research, prerequisites, innovation | Technology spread and indirect downstream unlocks |
| Laws and institutions | Current laws, eligibility and political support | Enactment uncertainty, political movements, amendments, unrest, changing interest-group power |
| Trade/diplomacy | World-market access, tariffs, treaty conditions | AI trade behavior, comparative advantage, treaties, capacity and geopolitical changes |
| Military | Current forces and wars | AI choices, logistics, battle and diplomacy; defer |

Paradox describes the economy as interdependent buildings, goods, wages, owners and population needs in [Dev Diary #57](https://www.paradoxinteractive.com/games/victoria-3/news/dev-diary-57-the-journey-so-far). The 1.9 trade rework removed old manual trade routes in favor of an autonomous world market ([1.9 changelog](https://steamdb.info/patchnotes/18882924/)). 1.15.2 adds tariff/subvention switching restrictions and other mechanical changes ([official announcements](https://steamcommunity.com/app/529340/announcements/)). Therefore old strategy guides cannot be treated as current rules.

## 4. How much math is necessary?

Use deterministic calculations for facts and simple marginal estimates; use the LLM for choosing among candidates, tradeoffs and explanation. Do not let the LLM invent inputs, legal actions or numerical forecasts.

Minimum useful calculations:

- Goods imbalance = buy orders minus sell orders. Relative imbalance must handle zero supply/demand.
- Price deviation = current price / base price minus one, with explicit market and date.
- Estimated marginal goods flows = building-level delta times active production coefficients, adjusted using verified utilization/throughput rules if available.
- Approximate gross operating surplus = output value minus input value, using applicable local prices; subtract verified payroll and other expenses before calling it accounting profit.
- Construction duration estimate = required points / allocated effective points per week, explicitly conditional on current allocation and modifiers.
- Credit runway = remaining credit capacity / current deficit, explicitly constant-deficit, before changes in interest or revenues.
- Check action preconditions: tech, laws, resource/land caps, employment, qualifications, infrastructure, queue and pending production; mark unknown if not extractable.

Do not claim estimates are exact game calculations. Validate the precise price formula, scaling modifiers and zero-order behavior against current `defines` and observed data before using them. A positive producer surplus is not automatically higher treasury income or better living standards. The [production-table documentation](https://shabbygaybar.github.io/victoria3-analysis/data-dictionary/production/) describes fully staffed one-level base configurations, not realized campaign output. [Economy model limitations](https://shabbygaybar.github.io/victoria3-analysis/data-dictionary/) include incomplete local price/MAPI, shortages, wages, transfers and other feedback in relevant solve modes. Its [equilibrium model](https://shabbygaybar.github.io/victoria3-analysis/guides/economy/) makes explicit simplifications, including fixed trade and modeled household income; it is useful as a reference, not as ground truth.

Example: expensive iron is an observation. Adding an iron mine is not automatically correct: verify state resource limit, labor, tools, local prices, infrastructure, current construction and budget. Estimate the additional iron supply and tool demand under clearly stated assumptions. Existing queued mines might already address the issue. After completion, compare subsequent saves with the prior estimate, without attributing all changes to that one mine.

## 5. Proposed architecture

1. Watch local save directory; copy only stable, complete file revisions. Deduplicate by content hash and game date/campaign, handle save branches and manual saves.
2. Decode with Rakaly (subprocess first); reject unsupported versions/unknown tokens where relevant. Prefer streaming/selective parsing over dumping entire world state as LLM input. Benchmark early and late-game saves.
3. Parse static installed game definitions into a versioned catalog; link save IDs to static keys and localized names. Vanilla only initially.
4. Normalize selected state/country/building/market snapshots. Keep date, version, source field, units and missingness. Use SQLite or DuckDB locally; raw snapshots optionally archived.
5. Query and compute tools, e.g. `get_budget`, `get_market`, `get_state`, `get_construction_queue`, `get_building_recipe`, `estimate_building_expansion`. Tool results return source snapshot and assumptions.
6. Deterministic candidate generation and constraint filtering. LLM ranks feasible candidates against explicit user goals and explains evidence, risks and uncertainty. No arbitrary game action execution.
7. UI displays save date/freshness, decisions and evidence. Persist advice separately from the game state. Manual-save refresh if the player needs a more recent snapshot.

Bun + TypeScript is reasonable for orchestration, with Rust CLI for decoding. The Python model is optional; do not add a second runtime unless it provides concrete validated benefit.

## 6. Development gates

**Gate A — data feasibility (1–3 focused days, estimate):** Obtain real vanilla 1.15.2 saves. Test compressed/binary and optionally plaintext. Capture UI at the same date. Map essential metrics, identify missing fields, benchmark decode and ingestion, verify exact installed game catalog. Test both early and late game, small and large countries. If essential state data cannot be reconstructed reliably, narrow the advisor or use UI as a supplement.

**Gate B — basic diagnostic advisor (3–7 focused days after A, estimate):** Budget, market, staffing, infrastructure and construction observations, with evidence-backed explanations. Answer questions such as what is expensive, what is draining money and what needs attention. No precise multi-month forecasting.

**Gate C — bounded economic action evaluator (2–4 additional weeks, estimate):** Feasible building/state selection, recipe-based marginal estimates, queue awareness, versioned regression tests and observed-versus-estimated comparisons. Start with one action at a time; then consider bundles and multiple goals.

**Later:** Technology/laws/institutions, trade strategy, diplomacy, military, multiplayer/mod support, sophisticated forecast or simulation. Broad simulator effort likely months or longer and cannot be estimated credibly without measured coverage and accuracy goals.

Validation criteria: compare extracted metrics with the game UI; fail closed on missing required data; verify action eligibility; compare simple forecasts against controlled saves; cover shortage/zero supply, debt, full employment, low infrastructure, incomplete construction, market changes and patch changes. Every recommendation must cite the snapshot and distinguish observed, calculated and estimated values.

## 7. Important unresolved questions

- No real save was available. Actual field coverage, parse latency/memory, schema and units remain unverified.
- Exact latest-patch Rakaly compatibility and platform installation need testing. Parser support is not equivalent to complete semantic extraction.
- The complete engine rules are not public source code. Script definitions expose much but may omit hardcoded mechanics and runtime intermediate values.
- Autosave cadence and paths should be confirmed in the actual installed game; game time is not wall-clock time. Monthly data can be stale after fast-forward, spending, war or a PM change.
- No formal Paradox authorization for a particular third-party tool was established. Keep read-only and local; check host rules in multiplayer, game terms and redistribution rights before publishing. Local save parsing and public modding tools exist, but that is not blanket approval.
- [Generated script docs](https://github.com/Victoria-3-Modding-Co-op/Modding-Digests) and [modding reference](https://vic3.paradoxwikis.com/wiki/Modding) offer an alternate investigation path for missing data. Do not assume arbitrary real-time telemetry is available.
- Estimates are engineering ranges, not commitments. AI-written code does not eliminate game validation or maintenance.
