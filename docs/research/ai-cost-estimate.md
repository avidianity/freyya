# AI cost estimate and the AI-free option

Research date: 2026-11-28.
Ticket: [issue #16](https://github.com/avidianity/freyya/issues/16), part of [Spec #7](https://github.com/avidianity/freyya/issues/7).
This note is reference, not instructions.
No AI provider was called, no key was made or stored, and no money was spent.

## Result

A cheap model costs 0.003 to 0.11 USD per day for light to heavy use.
This is well under the 0.50 USD per day default in the [product brief](product-brief.md).
A capable model for every question costs 0.12 to 2.50 USD per day.
It goes over 0.50 USD at normal use without prompt caching, and at heavy use in all cases.
Routing 80 percent of questions to the cheap model and 20 percent to the capable model costs 0.03 to 0.58 USD per day.
Compact summaries are the main saving: the full `read-save` output is about 12 times the largest summary and 28 times the average one.

A fully AI-free freyya can keep data, alerts, Candidate actions, Estimates, and tracking.
Almost all of that work is needed for the AI-assisted freyya too, because the AI reaches data only through these same deterministic tools.
The AI-free freyya loses plain-language Objectives, free-form questions, and explanations that join several areas.

## Method

1. Take the `read-save` output for the manual Save `fixtures/saves/sweden_1837_08_16.v3` at integration commit `0057476` (483 lines, 59,536 characters).
2. Build four compact text summaries from it with a throwaway Python script outside the repository: budget, goods shortages, construction, and buildings and jobs.
3. For goods shortages, read each good's base `cost` from the Game files (`game/common/goods/00_goods.txt`, read only) and list goods whose price is at least 10 percent over base.
4. Write a sample system prompt (18 lines) and eight sample tool definitions in OpenAI function format.
5. Count tokens with `tiktoken==0.12.0` (OpenAI's published tokenizer library), encoding `o200k_base`, and cross-check with `cl100k_base`.
6. Read OpenRouter prices from its public model list and multiply them by the token counts in a cost model (see [Assumptions](#assumptions)).

The script ran with `uv run --no-project --python 3.12 --with tiktoken==0.12.0`.
It did not add any dependency or file to the repository.

### Tokenizer caveat

`o200k_base` is the tokenizer of recent OpenAI models.
Anthropic, Google, and DeepSeek models use their own tokenizers, and most of them are not published.
Token counts for the same text differ by model, by roughly 10 to 20 percent.
On these texts, `o200k_base` and `cl100k_base` differ by less than 4 percent.
Cost scales linearly with tokens, so add 20 percent to any figure below for a safe upper bound.

## Token counts

| Text | Characters | Tokens (`o200k_base`) | Tokens (`cl100k_base`) |
| ---- | ---------: | --------------------: | ---------------------: |
| Budget summary | 179 | 65 | 66 |
| Goods shortages summary (13 goods) | 830 | 452 | 458 |
| Construction summary (24 items) | 1,043 | 304 | 307 |
| Buildings and jobs summary (59 buildings) | 2,773 | 1,155 | 1,192 |
| Full `read-save` output | 59,536 | 14,201 | 14,185 |
| Sample system prompt | 1,680 | 372 | 372 |
| Sample tool definitions (8 tools) | 3,161 | 645 | 640 |

The four summaries average about 500 tokens.
The buildings and jobs summary is the largest, because it lists every building with levels.
A tool with a state or building filter returns a fraction of it.

### Summary samples

Budget summary, complete:

```text
Budget SWE 1837-08-16 (Save 1.15.2)
treasury 355999 £; income 22482 £/wk; expenses 18602 £/wk; balance +3880 £/wk; debt missing
weeks of reserve at current balance: n/a (surplus)
```

Goods shortages summary, first lines:

```text
Goods shortages, market 8 (SWE, NOR), 1837-08-16; price at least 10% over base
good | price £ | base £ | over | bldg sell/wk | bldg buy/wk | net exports
Transportation | 52.5 | 30 | +75% | 17.9 | 0.0 | 0.0
Tools | 67.2 | 40 | +68% | 60.6 | 224.2 | -12.0
Fabric | 25.5 | 20 | +27% | 163.9 | 271.7 | -9.0
pop demand: missing (Save stores pop spending as money)
```

Construction summary, first lines:

```text
Construction SWE 1837-08-16; capacity 25.0 pts/wk
government queue:
1. Livestock Ranches s130 left 25 speed 7.8
2. Livestock Ranches s130 left 200 speed -
```

Buildings and jobs summary, first lines:

```text
Salaried workforce       438670 people
Unemployed workforce      36823 people
state | building | levels | staffed levels | employees
Gotland | Subsistence Farms | 3 | 2.1 | 10298
```

Four goods (Transportation, Silk, Porcelain, Fruit) sit at exactly 1.75 times base price.
Silk, Porcelain, and Fruit have no building orders at all, so pop demand alone drives their price.
Pop demand is missing from the Snapshot, so the summary cannot say how large these shortages are.

## Prices

Source: [OpenRouter model list API](https://openrouter.ai/api/v1/models), read 2026-11-28 06:11 UTC, no key needed.
Model pages: [claude-haiku-5.5](https://openrouter.ai/anthropic/claude-haiku-5.5) and [claude-sonnet-5.5](https://openrouter.ai/anthropic/claude-sonnet-5.5).
Caching rules: [OpenRouter prompt caching guide](https://openrouter.ai/docs/guides/best-practices/prompt-caching), read 2026-11-28.

USD per million tokens:

| Model | Role | Input | Output | Cache read | Cache write | Context |
| ----- | ---- | ----: | -----: | ---------: | ----------: | ------: |
| `anthropic/claude-haiku-5.5` | cheap | 0.10 | 0.50 | 0.01 | 0.125 | 1,000,000 |
| `anthropic/claude-sonnet-5.5` | capable | 2.00 | 10.00 | 0.10 | 2.50 | 1,000,000 |

Both models list `tools` in their supported parameters.
One family keeps prompt and tool behavior comparable between the two roles.
The caching guide gives these facts for Anthropic models:

- Caching needs an opt-in: automatic caching or `cache_control` breakpoints.
- A cache read costs 0.1 times input price, and a cache write costs 1.25 times input price.
- The minimum cacheable prompt is 512 tokens for both models.
- The default cache lifetime is 5 minutes.

Other tool-calling models on the same list, for context only:

| Model | Input | Output |
| ----- | ----: | -----: |
| `deepseek/deepseek-v4.1-flash` | 0.051 | 0.567 |
| `openai/gpt-5.6-luna` | 0.20 | 1.20 |
| `google/gemini-4-carbon` | 2.00 | 10.00 |
| `openai/gpt-5.6-terra` | 2.00 | 12.00 |

Prices change often.
Read the list again before any budget decision.
This note does not include any fee for buying OpenRouter credits.

## Assumptions

- Fixed prefix per request: 1,500 tokens (measured system prompt and tools 1,017, plus about 300 for the Objective and a short Campaign memory, plus margin).
- Question: 50 tokens.
- Each tool call: the model writes a 60-token call, and the tool returns a summary.
- Tool result: 500 tokens (light and normal) or 800 tokens (heavy, looser filters).
- Final answer: 500 tokens.
- Reasoning: none for the cheap model, 1,000 output tokens per question for the capable model.
- Each question starts fresh: no chat history is carried, only the short memory in the prefix.
- Each model turn resends the prefix, the question, and all earlier calls and results of that question.
- With caching: the first turn writes the cache, and later turns read everything earlier from cache and write only the new call and result.
  Reuse of the cache between questions is not counted, so the cached figures are conservative.

| Use | Questions per day | Tool calls per question | Model turns per question |
| --- | ----------------: | ----------------------: | -----------------------: |
| Light | 5 | 2 | 3 |
| Normal | 15 | 3 | 4 |
| Heavy | 40 | 5 | 6 |

## Cost per question

USD, without caching / with caching:

| Use | Tokens in / out (cheap) | Cheap | Capable |
| --- | ----------------------- | ----: | ------: |
| Light | 6,330 / 620 | 0.0009 / 0.0007 | 0.029 / 0.023 |
| Normal | 9,560 / 680 | 0.0013 / 0.0008 | 0.036 / 0.026 |
| Heavy | 22,200 / 800 | 0.0026 / 0.0013 | 0.062 / 0.034 |

The capable model writes 1,000 more output tokens per question for reasoning.

## Cost per day

USD, without caching / with caching, compared with the 0.50 USD default:

| Use | Cheap only | Capable only | 80 percent cheap, 20 percent capable |
| --- | ---------: | -----------: | -----------------------------------: |
| Light | 0.005 / 0.003 | 0.14 / 0.12 | 0.033 / 0.026 |
| Normal | 0.019 / 0.012 | **0.54** / 0.38 | 0.12 / 0.086 |
| Heavy | 0.11 / 0.052 | **2.50 / 1.37** | **0.58** / 0.32 |

Bold figures are over 0.50 USD.

At normal use with caching, 0.50 USD buys about 600 cheap questions or about 20 capable questions.
The default is generous for the cheap model and tight for the capable model.

### Why compact summaries matter

If a tool returned the full `read-save` output (14,201 tokens) once per question, every later turn would resend it.
At normal use, three of the four turns carry it, which adds about 42,600 input tokens per question.
That adds about 1.28 USD per day for the capable model and about 0.06 USD per day for the cheap model, without caching.

## AI-free freyya

The brief asks for deterministic analysis first and paid AI only on request (Owner decision 10 in the [product brief](product-brief.md)).
The AI reaches campaign data only through deterministic tools.
So most AI-free work is work the AI-assisted freyya needs anyway.

### What can be fully deterministic

| Brief capability | AI-free form | Kind of work | Amount |
| ---------------- | ------------ | ------------ | ------ |
| Acquire and analyze campaign data | Save to Snapshot, Game files catalog | Parsing (largely done in Spec #7) | Done or small |
| Track progression over time | Snapshot history and change reports per area | Storage and comparison | 2 to 3 Tickets |
| Identify problems and risks | Alert rules on each new Snapshot: deficit and weeks of reserve, goods over base price, unemployment, unstaffed levels, idle or stalled construction | Rules with thresholds | 1 Ticket per Advice area, about 5 |
| Discover opportunities | Rules such as "a good is over base price and a building that makes it can be built here" | Rules joining markets, buildings, and Game files | 2 to 3 Tickets |
| Recommend specific actions | Candidate actions with Estimates (profit per level, jobs, construction cost and time), then a fixed ranking | Formulas from production method inputs, outputs, and prices; each formula needs a UI check | 5 to 8 Tickets, the largest part |
| Explain benefits, costs, and uncertainty | Fixed text templates that fill in sourced numbers and mark missing values | Templates per Candidate action kind | 1 to 2 Tickets |
| Support Objectives | A short list of preset Objectives (for example "grow the economy", "cut debt", "industrialize fast"), each a set of ranking weights | Weight sets and a picker | 1 to 2 Tickets |
| Conversational interface | Fixed commands and reports per area | Command surface | 1 to 2 Tickets |
| Near-real-time awareness | Telemetry, if built | Independent of AI | No change |

In total, an AI-free freyya for the five first Advice areas (money, goods markets, buildings, construction, jobs) is about 17 to 25 Tickets.
Only the preset Objectives, the text templates, and the fixed command surface (about 3 to 6 Tickets) exist only because AI is absent.
The rest is the same deterministic core that the AI calls through tools.

### What is lost without AI

- Plain-language Objectives.
  The Owner picks a preset or sets weights; freyya cannot read "become a great power without debt" and repeat its priorities and limits.
- Free-form questions.
  The Owner reads fixed reports; freyya cannot answer "why did my treasury fall last year?" unless a report already covers it.
- Explanations that join areas.
  Templates explain one Candidate action at a time; they cannot weigh a shortage, a budget limit, and a construction queue together in plain words.
- Unfamiliar situations.
  A rule fires only for a case that someone wrote; a new pattern gets no comment.
- Trade-offs between competing Objectives beyond fixed weights.
- Comments on areas that get Observations only (technology, laws, politics, trade, diplomacy, military).

### Recommendation

Build the deterministic core first, because both options need it.
Add the AI layer as a thin, optional adapter that calls the same tools.
Keep the cheap model as the default and send only questions the Owner marks as hard to the capable model.
Keep 0.50 USD per day as the default; it covers normal use with either the cheap model or the 80/20 mix.

## Limits of this estimate

- The token counts come from one early Save (1837); a late-game Save has more buildings and a larger buildings and jobs summary.
- The system prompt and tools are samples, not the final design.
- The cost model ignores retries, errors, and long chats; a chat that carries history grows each turn.
- The capable model's reasoning length is an assumption; 2,000 reasoning tokens instead of 1,000 adds about 0.01 USD per question.
- The summaries were built from `read-save` text, not from a final summary API in the core.
