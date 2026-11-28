# Product brief

The Owner wrote this brief and handed it over during the spec grill on 2026-11-28.
Part 1 is the brief as written.
Part 2 records the Owner's answers from that grill, which narrow and order it.
This note is reference: the investigation scope lives in [issue #4](https://github.com/avidianity/freyya/issues/4), rules live in `.rules`, and lasting decisions live in `docs/adr/`.

## Part 1: Victoria 3 AI Companion (as written)

### 1. Objective

Build an AI-powered companion that understands the player's Victoria 3 campaign and helps them make informed strategic decisions.

The companion must function as a strategic advisor capable of understanding the current campaign, identifying problems, discovering opportunities, recommending actions, and explaining their consequences.

The player must retain complete control over gameplay.

### 2. Core Capabilities

- Automatically acquire and analyze campaign data.
- Understand game mechanics, dependencies, and constraints.
- Provide a conversational interface for campaign-specific questions.
- Proactively identify problems, risks, opportunities, and strategic priorities.
- Recommend specific, actionable decisions.
- Explain expected benefits, consequences, uncertainties, and trade-offs.
- Track campaign progression over time.
- Support player-defined strategic objectives.
- Provide near-real-time awareness of the campaign where technically feasible.

### 3. Strategic Objectives

The default objective is to win by maximizing overall campaign success. Consider:

- Economic growth.
- Government finances.
- Industrialization.
- Political stability.
- Technological advancement.
- Military strength.
- Living standards.
- International influence.

Do not assume that maximizing a single metric constitutes winning.

Players must be able to override the default objective using natural language.

Examples:

- Become the world's largest economy.
- Industrialize as quickly as possible.
- Establish military superiority.
- Maximize living standards.
- Become a great power.
- Eliminate government debt.

Support changing objectives during a campaign. Objectives must influence recommendations, prioritization, acceptable risks, and trade-off analysis.

Persist objectives independently for each campaign.

### 4. Campaign Data Acquisition

Support multiple complementary data acquisition mechanisms.

#### 4.1. Save Files

- Automatically detect and parse completed autosaves.
- Support comprehensive campaign snapshots.
- Investigate existing parsers such as Rakaly and Jomini.
- Handle incomplete, corrupted, and incompatible save files safely.
- Avoid unnecessary processing of unchanged data.

#### 4.2. Near-Real-Time Telemetry

Investigate developing a lightweight Victoria 3 companion mod that exports game statistics through supported scripting and logging mechanisms.

Investigate:

- Daily and weekly script execution.
- Accessible internal game statistics.
- Structured data export through `debug_log` or alternative mechanisms.
- Incremental log consumption.
- Export frequency and latency.
- Performance overhead.
- Log truncation and rotation.
- Achievement and multiplayer compatibility.

Review existing implementations:

- <https://github.com/CrazyIvanTR/vicky3-logs-for-data-timelapses>
- <https://steamcommunity.com/sharedfiles/filedetails/?id=3713168053>

Do not assume these implementations remain compatible with current game versions.

Telemetry is a feasibility investigation, not an established capability. Save parsing must remain functional independently.

#### 4.3. Unified Campaign State

Treat save parsing and telemetry as independent, complementary data acquisition adapters.

- Normalize acquired data into a consistent domain model.
- Support complete snapshots and incremental updates.
- Reconcile discrepancies between data sources.
- Track data freshness and provenance.
- Prevent stale information from overwriting newer information.
- Handle missing information gracefully.
- Isolate data from different campaigns.
- Support additional acquisition mechanisms without substantial architectural changes.

Prefer the lowest practical latency without significantly affecting game performance.

### 5. Game Knowledge

Build a version-aware game knowledge system using available sources:

- Installed game files.
- Building definitions.
- Production methods.
- Technologies.
- Laws.
- Resources.
- Modifiers.
- Economic formulas.
- Official documentation.
- Verified community research.

Prefer extracting definitions programmatically rather than hardcoding them.

Account for game versions, DLCs, and modified game rules.

Clearly distinguish verified mechanics from inferred or undocumented mechanics.

### 6. Analysis and Decision Making

Use a hybrid approach combining deterministic computation and LLM reasoning.

#### 6.1. Deterministic Analysis

Use conventional code for:

- Calculations.
- Statistics.
- Trend analysis.
- Dependency analysis.
- Shortage detection.
- Budget analysis.
- Opportunity detection.
- Decision feasibility validation.
- Outcome estimation where reliable formulas exist.

Gradually expand deterministic analytical capabilities as game mechanics become better understood.

#### 6.2. LLM Reasoning

Use the LLM for:

- Understanding player objectives.
- Strategic reasoning.
- Prioritizing opportunities.
- Evaluating competing objectives.
- Understanding unfamiliar situations.
- Recommending actions.
- Explaining decisions.

The LLM must have access to structured campaign data and deterministic analysis capabilities through tools.

The LLM must not replace reliable calculations or invent game mechanics.

Do not attempt to recreate the entire Victoria 3 simulation engine unless demonstrably necessary.

### 7. AI Provider Integration

OpenRouter is the initial AI provider.

- Abstract provider-specific implementations behind appropriate interfaces.
- Support interchangeable providers and models.
- Prevent provider-specific logic from leaking into the domain.
- Account for differences in provider capabilities.
- Support tool calling, structured responses, usage reporting, and configurable reasoning limits.
- Allow different models to serve different responsibilities.

Changing providers must not require substantial changes to the application.

### 8. Token and Cost Optimization

Minimizing AI operating costs is a primary requirement, not a future optimization.

#### 8.1. Selective Inference

- Do not invoke the LLM on every save, telemetry update, or game tick.
- Continuously monitor campaign state using deterministic code.
- Detect significant changes without LLM involvement.
- Invoke the LLM on player request or explicitly configured strategic triggers.
- Deduplicate events and prevent redundant analysis.
- Support cooldown periods between automatic analyses.

Real-time data acquisition must not require real-time AI inference.

#### 8.2. Context Efficiency

- Never send entire save files to the LLM.
- Retrieve only relevant campaign data through tools.
- Filter, aggregate, and paginate large datasets.
- Provide compact strategic summaries where appropriate.
- Reuse previous analysis when still applicable.
- Retrieve relevant game knowledge on demand.
- Utilize provider prompt caching where supported.
- Prevent unbounded conversation history growth.

#### 8.3. Model Routing

Support configurable model routing based on task complexity.

- Low-cost models for routine questions and straightforward analysis.
- More capable models for complex strategic reasoning.
- Configurable limits on reasoning effort, tool calls, and output tokens.
- Avoid unnecessary multi-agent workflows.

#### 8.4. Budget Controls

Provide:

- Configurable daily and per-request spending limits.
- Token and cost tracking.
- Spending estimates before expensive operations.
- Actual provider-reported usage and costs where available.
- Automatic suspension of paid inference when budget limits are reached.
- Explicit player authorization to exceed configured limits.

Consider $0.50 per day as an initial configurable default, subject to validation.

The application must continue acquiring and analyzing campaign data through deterministic capabilities when paid inference is unavailable.

### 9. Architecture and Composability

Design the project as a modular, composable, headless system rather than a tightly coupled standalone application.

Future consumers may include:

- Desktop companion applications.
- Command-line interfaces.
- Web backends.
- REST APIs.
- MCP servers.
- Other applications and integrations.

Architectural requirements:

- Separate domain logic, application logic, infrastructure, and presentation.
- Keep core capabilities independent of UI frameworks, transport protocols, and deployment environments.
- Expose well-defined, typed interfaces.
- Keep modules independently testable and replaceable.
- Abstract external dependencies at appropriate boundaries.
- Prevent circular dependencies.
- Avoid duplicated business logic.
- Avoid unnecessary abstractions and architectural complexity.
- Design modules around cohesive capabilities rather than arbitrary technical layers.

Applications must compose existing capabilities rather than reimplement them.

The following capabilities should have clear module boundaries:

- Data acquisition.
- Campaign state management.
- Game knowledge.
- Deterministic analysis.
- Strategic objectives.
- Decision support.
- AI integration.
- Cost management.
- Persistence.
- Presentation.

These are conceptual boundaries, not a prescribed directory structure.

### 10. Reliability

- Ground recommendations in actual campaign data and verified game mechanics.
- Distinguish observed facts, calculated estimates, assumptions, and predictions.
- Communicate data freshness and missing information.
- Never present uncertain predictions as guaranteed outcomes.
- Validate recommended actions against known prerequisites and constraints.
- Explain why a recommendation is relevant to the player's objective.
- Allow recommendations to be evaluated against subsequent campaign snapshots.
- Support reproducible testing using campaign fixtures.

### 11. Constraints

- Advisory only.
- Do not automate gameplay.
- Do not manipulate game memory.
- Prefer supported game interfaces and scripting mechanisms.
- Minimize impact on game performance.
- Minimize recurring AI operating costs.
- Prefer existing reliable solutions over reinventing them.
- Do not introduce unnecessary infrastructure.
- Do not sacrifice maintainability for short-term development speed.

### 12. Initial Investigation

Before committing to substantial implementation:

- Investigate existing save parsers, telemetry mods, and analysis tools.
- Examine actual save files and installed game definitions.
- Validate extracted information against the in-game UI.
- Investigate scripting capabilities and telemetry limitations.
- Identify available information, missing information, and required calculations.
- Investigate economic formulas, dependencies, and simulation mechanics.
- Determine which recommendations can be made reliably.
- Benchmark parsing and telemetry performance.
- Estimate realistic LLM token consumption and operating costs.
- Identify hidden complexity, compatibility concerns, and maintenance risks.
- Validate assumptions through small feasibility experiments.
- Establish appropriate architectural boundaries.

Inspect existing repository rules and implementation before proposing changes.

Challenge this specification where necessary. Do not treat it as proof of technical feasibility.

Report findings, architectural recommendations, unresolved concerns, and a proposed incremental implementation plan before committing to substantial development.

## Part 2: Owner decisions from the grill (2026-11-28)

Facts found during the grill: Victoria 3 1.15.2 is installed, with no DLC and no mods.
The Owner has one Sweden **Campaign** at 1837: five rotating autosaves of about 12 MB, and the manual Save `sweden_1837_08_16.v3`.

1. **Investigation first.**
   The first milestone is the investigation of section 12, run on the Owner's real Saves and Game files.
   It ends in a findings report that the Owner reads and approves.
   No AI integration, mod, or user interface is built before that.
2. **Telemetry mod on paper only.**
   Read the two linked mods, the modding documentation, and the Game files; report what a mod could export, how often, and whether it disables achievements.
   Nothing is installed, and the read-only rule stays.
   A hands-on test is a separate Owner decision after the report.
3. **AI cost estimate spends nothing.**
   Count the tokens of compact summaries built from real Saves, and multiply by OpenRouter's published prices for a cheap and a capable model.
   No AI account or key is needed before the report.
4. **Headless core, gpui as the main screen.**
   The core (Save reading, the Game files catalog, calculations) knows nothing about screens.
   The gpui window is a thin screen on top of it later.
   The investigation uses the command line only.
5. **UI check.**
   After the grill, the Owner screenshots a checklist of screens from the paused manual Save `sweden_1837_08_16.v3`.
   The screenshots stay outside the repository.
   The Owner keeps that Save untouched until then.
6. **History keeps Snapshots only.**
   Freyya stores the **Snapshot** of each new autosave in its data folder under the local app-data folder, not OneDrive, and keeps no full Save copies beyond the few the investigation studies.
   The report confirms real sizes first.
7. **Game patches.**
   On an unchecked game version, freyya shows **Observations** with a visible "not yet checked for this game version" warning, and gives no **Advice** until that version passes the UI check.
8. **Base game only.**
   No DLC is owned and mods are unsupported; a modded **Campaign** gets **Observations** only, with a warning.
9. **Objective.**
   The Owner states an **Objective** in plain words; freyya repeats it as priorities and limits, and uses it only after the Owner confirms.
   Without one, the **Objective** is broad success.
10. **Deterministic first.**
    Anything that ordinary calculation can find, freyya finds that way.
    Automatic alerts come from deterministic rules on each new Save.
    Paid AI runs only when the Owner asks; automatic AI runs are a later opt-in with a cooldown.
    The report includes a section on the kind and amount of work a fully AI-free freyya needs, and what it loses.
11. **First advice areas.**
    **Advice** covers money, goods markets, buildings, construction, and jobs first.
    Technology, laws, politics, trade, diplomacy, and military are shown as **Observations** only.
12. **Real Saves as fixtures.**
    A chosen set of real Saves, each under 50 MB, is committed as test fixtures, plus tiny hand-made fixtures for missing cases.
    Game files stay out of the repository ([ADR 0005](../adr/0005-commit-real-saves.md)).
13. **Where work lives.**
    This brief keeps the Owner's intent, [issue #4](https://github.com/avidianity/freyya/issues/4) holds the investigation scope, and the findings report arrives in `docs/research/` as a pull request.

Left for after the report: the AI budget (the $0.50 per day default and how the Owner approves going over it), model choice and routing, where the AI key is stored, a hands-on mod test, and the order of features after the investigation.
