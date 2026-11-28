# Telemetry mod on paper

Research date: 2026-11-28.
Scope: [issue #17](https://github.com/avidianity/freyya/issues/17), under Spec [#7](https://github.com/avidianity/freyya/issues/7).
Target: Victoria 3 1.15.2 (Ivan-Chai), base game, single player.
This note answers the questions in [product brief section 4.2](./product-brief.md#42-near-real-time-telemetry) and Part 2 item 2.
Nothing was installed, and nothing in the game, the Save folder, or the launcher settings changed.
This note is reference, not instructions.

## Summary

A mod can write economic numbers to `debug.log` once per game month, and a reader can tail that file.
The cost is high for freyya:
the mod changes the game checksum, so it disables achievements on the Owner's existing **Campaign**;
a modded **Campaign** gets **Observations** only under Owner decision 8;
and both linked mods are unmaintained.
Performance cost, flush delay, and the exact log size cap are unknown on paper.

## Sources

| Source | What it gave | Checked |
| ------ | ------------ | ------- |
| [CrazyIvanTR/vicky3-logs-for-data-timelapses](https://github.com/CrazyIvanTR/vicky3-logs-for-data-timelapses) | Log copy method and deduplication scripts | 2026-11-28 |
| [Steam Workshop item 3713168053](https://steamcommunity.com/sharedfiles/filedetails/?id=3713168053) and its [change notes](https://steamcommunity.com/sharedfiles/filedetails/changelog/3713168053) | Mod description, dates, status | 2026-11-28 |
| [Modding-Digests 1.15.2 docs](https://github.com/Victoria-3-Modding-Co-op/Modding-Digests/tree/main/1.15.2/docs) (`effects.log`, `triggers.log`, `on_actions.log`), commit "Added 1.15.2 documentation" of 2026-11-19 | Generated script docs for 1.15.2 | 2026-11-28 |
| [Modding-Digests data types](https://github.com/Victoria-3-Modding-Co-op/Modding-Digests/tree/main/documentation/data_types) (`data_types_uncategorized.txt`, updated 2026-11-19) | Data functions for localization strings | 2026-11-28 |
| Installed Game files, `C:\Program Files (x86)\Steam\steamapps\common\Victoria 3`, version from `launcher/launcher-settings.json` | On actions, checksum manifest, achievement text | 2026-11-28 |
| The Owner's logs folder, `Documents\Paradox Interactive\Victoria 3\logs` (OneDrive) | Real log names, sizes, rotation, line format | 2026-11-28 |

The Victoria 3 wiki and the Paradox forum did not load for automated fetches on 2026-11-28.
Web searches found no Paradox statement on the `debug.log` size cap or on mods and achievements.

## 1. The two linked mods and 1.15.2 compatibility

### Steam Workshop item 3713168053, "Export Data"

- Author: defury.
  Posted and last updated "24 Apr @ 6:44am"; the page shows no year.
  The only comments are from 13 and 14 June, and the linked GitHub repository was created on 2026-06-13, so the year is very likely 2026.
- One change note, with no text.
- Steam shows "This item is incompatible with Victoria 3." and "This item has been removed from the community because it violates Steam Community & Content Guidelines."
  The script files are not downloadable, so this note could not read them.
- The description says the mod writes country, market, world market, and culture data to `debug.log`.
  It says market, world market, and culture data are hardcoded, so new or changed goods or cultures need hand edits.
- The description says "Patch 1.13 limited the size of the debug-file."
- The author says they do not plan to update or maintain the mod.
- No game version tag, no frequency, no performance note, and no achievement note.

Compatibility with 1.15.2: **not compatible as published**.
Steam flags it as incompatible, it is removed, and it predates 1.15.0.

### GitHub repository CrazyIvanTR/vicky3-logs-for-data-timelapses

- Seven commits, all on 2026-06-13; last push 2026-06-13T23:24:07Z.
- It is not a mod.
  It is a companion workflow for the Workshop mod: a PowerShell copy of `debug.log`, plus three Python scripts that find and remove duplicate lines.
- The copy command in its README:

  ```text
  Get-Content debug.log -Wait | Out-File -FilePath "00-full-debug.txt" -Append
  ```

- The README says "Some lines will be duplicates"; it does not say why.
- The scripts read the copy as UTF-16, because PowerShell `Out-File` writes UTF-16.
  The game writes `debug.log` itself as plain ASCII text.
- The README mentions files of about 700 MB and says the author does not plan to maintain it.

Compatibility with 1.15.2: **not applicable on its own**; it depends on the removed Workshop mod.
The copy idea is version independent.

## 2. Script execution: daily and weekly

- The 1.15.2 code on actions have pulses only for month, half year, year, five years, and decade.
  Sources: `game/common/on_actions/_on_actions.md` and the 1.15.2 `on_actions.log` digest.
  Examples: `on_monthly_pulse` (no scope), `on_monthly_pulse_country`, `on_monthly_pulse_state`.
- There is **no daily or weekly pulse** in 1.15.2.
- A weekly run is possible on paper with an event that re-triggers itself.
  The `trigger_event` effect accepts `trigger_event = { id = X days/weeks/months/years = Y }` (1.15.2 `effects.log`).
  This needs a new event file, which is also checksummed.
- A daily run is the same idea with `days = 1`; its cost is unknown and likely the highest.

## 3. Accessible statistics

- The `debug_log` effect exists in 1.15.2: "Log a string to the debug log when this effect executes, debug_log = message, the message can be a localization string with ROOT, SCOPE and PREV available" (1.15.2 `effects.log`).
- The base game uses it in normal play.
  Sample from `game/common/on_actions/00_code_on_actions.txt`:

  ```text
  debug_log = "[TimeKeeper.GetCurrentDate.GetString]: Election Campaign Started - [THIS.GetCountry.GetNameNoFormatting]"
  ```

- Iterators exist to reach the data: `every_country`, `every_state`, `every_market`, `every_market_goods`, `every_scope_building`, `every_scope_pop` (1.15.2 `effects.log`).
  `every_market_goods` iterates "all active (market) goods in a market", so a new mod need not hardcode goods.
- Numeric triggers exist that a mod can read, for example `market_goods_buy_orders`, `market_goods_sell_orders`, `market_goods_production`, `market_goods_consumption`, `market_goods_imports`, `market_goods_exports`, `gold_reserves`, `scaled_debt`, `total_population` (1.15.2 `triggers.log`).
- Data functions exist for localization strings, for example `Country.GetGDP`, `Country.GetGDPPerCapita`, `Country.GetWeeklyBalance`, `Country.GetGoldReserves`, `Country.GetTotalPopulation`, `Goods.GetMarketPrice`, `Building.GetProfitability`, `Building.GetEmploymentPercentage`, `State.GetInfrastructure` (Modding-Digests `data_types_uncategorized.txt`).

**Unknown**: which data functions return real values inside a `debug_log` string.
Many are written for the user interface and may need a selected market or country.
Only a hands-on test answers this.

## 4. Export through `debug_log`

- Each call writes one line.
  A real line from the Owner's `debug.log`:

  ```text
  [06:08:57][jomini_effect_impl.cpp:459]: common/on_actions/00_code_on_actions.txt:4343: April 10, 1837: Election Campaign Ended - Central America
  ```

- The prefix holds wall-clock time, the engine source line, and the script file and line.
  It holds no game date, so the mod must put the game date in the message.
- Other engine messages share the file, so the reader must filter by a fixed tag in the message.
- No structured export exists; the mod formats text, and freyya parses it.
- A forum bug report says that after 1.12.1 every `debug.log` message became a placeholder.
  The Owner's 1.15.2 log shows real messages, so that bug is not present now.
- `error_log` and `debug_log_scopes` also exist; neither gives a better format.

## 5. Incremental log consumption

- The game appends to `debug.log` during a session.
  The CrazyIvanTR method (`Get-Content -Wait`) shows that a reader can follow it live.
- A reader can keep a byte offset and read only new bytes.
- The reader must detect a new session: the file restarts with `Mounted Data` lines and a smaller size.
- **Unknown**: why the CrazyIvanTR copy gets duplicate lines.

## 6. Frequency and latency

- With `on_monthly_pulse`, the export runs once per game month.
- Data delay is at most one game month plus the time the game takes to write the line.
- **Unknown**: the write and flush delay of `debug.log`, and the wall-clock length of a game month at each speed on the Owner's machine.
- Save-based Snapshots already arrive with each autosave.
  **Unknown** on paper: the Owner's autosave interval, so the real gain over Saves is not measured.

## 7. Performance overhead

- **Unknown**.
  No source gives a measurement.
- On paper, the cost grows with markets times goods, or with countries times fields, each pulse.
  The base game already runs many monthly country effects, so one monthly loop is likely small.
  A daily loop over every country or building is likely noticeable.
- Log growth: the README of CrazyIvanTR mentions copies of about 700 MB for a full game.

## 8. Log truncation and rotation

Observed in the Owner's logs folder on 2026-11-28:

| File | Size (bytes) | First line time | Last change |
| ---- | ------------ | --------------- | ----------- |
| `debug.log` | 196,011 | 05:43:25 | 06:15 |
| `debug.1.log` | 15,398 | not read | 05:09 |
| `debug.2.log` | 316,928 | 03:04:58 | 03:54 |
| `debug.3.log` | 207,262 | not read | 01:26 |

- The game rotates logs at each launch: `debug.log` becomes `debug.1.log`, and so on.
- It keeps three older copies; a fourth launch drops the oldest.
  A reader must copy what it needs before then.
- The Workshop author says 1.13 limited the size of `debug.log`.
  **Unknown**: the cap size and what happens at the cap (stop, wrap, or rotate).
  The largest observed file is 316,928 bytes, but no session was long enough to show a cap.
- The logs folder is inside OneDrive here, so OneDrive syncs every write.

## 9. Achievements

- Achievement conditions in 1.15.2 (`game/localization/english/achievements_l_english.yml`, `ACHIEVEMENTS_EXPLANATION`):

  ```text
  Achievements are only unlockable if:
  You play Single Player
  You are not playing with any Game Rules that block achievements
  You are not playing as an observer
  ```

  The text names no rule about mods or Ironman.
- Checksummed folders (`game/checksum_manifest.txt`): `common`, `events`, `map_data`, `gui`, `localization`, plus `paths_checksummed.settings`.
  A telemetry mod adds files in `common/on_actions` and maybe `events`, so it changes the checksum.
- `GAME_CHECKSUM_CHANGED_FROM_SAVE_FILE` in `game/localization/english/frontend/common_l_english.yml`:

  ```text
  This save was valid for Achievements on a different checksum, loading it will disable Achievements in the save.
  ```

- Result: adding the mod to the Owner's existing Sweden **Campaign** **disables achievements for that Campaign**, if it is valid for them now.
- **Unknown**: whether a new campaign started with the mod can earn achievements.
  The explanation text suggests yes, but no primary source confirms it.

## 10. Multiplayer

- `ACHIEVEMENTS_WILL_BE_DISABLED_MULTIPLAYER`: "Opening your game to multiplayer will disable achievements."
- All players need the same checksum, so every player needs the mod.
- Freyya is single player only, so multiplayer is out of scope.

## 11. Conflicts with freyya decisions

- Owner decision 8 says a modded **Campaign** gets **Observations** only, with a warning.
  A telemetry mod makes the **Campaign** modded, so freyya would give no **Advice** for it.
- The read-only boundary forbids writes in the Save folder and the launcher settings.
  Installing and enabling a mod writes to the mod folder and the launcher playset.
  The Owner would do this, not freyya, but the boundary needs an explicit Owner decision.

## Open unknowns

- Which data functions return values inside `debug_log`.
- Flush delay of `debug.log`.
- Performance cost of a monthly, weekly, or daily export.
- The `debug.log` size cap and its behavior.
- Achievements in a new campaign started with a mod.
- Cause of duplicate lines in the CrazyIvanTR copy.

## Recommendation

A hands-on test is **not worth an Owner decision now**.
Ask for it only after the Save investigation shows a concrete gap that Saves cannot fill.
Reasons:

- Both linked mods are unmaintained, and the Workshop mod is removed and flagged incompatible.
- The mod disables achievements on the existing **Campaign**.
- Under decision 8, a modded **Campaign** loses **Advice**, which defeats the purpose.
- The main gain is monthly data between autosaves, and the autosave interval is not yet known.

If the Owner later wants the test, run it on a new throwaway campaign, not the Sweden **Campaign**.
Measure the open unknowns above, and record the decision on decision 8 and the read-only boundary first.
