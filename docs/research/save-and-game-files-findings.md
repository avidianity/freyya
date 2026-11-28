# Findings: real Saves and Game files

Research date: 2026-11-28.
Spec: [#7](https://github.com/avidianity/freyya/issues/7).
Game: Victoria 3 1.15.2, no DLC, no mods, on the Owner's PC.
This report grows one section per Ticket; the Owner approves it as a whole.

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
Timing and memory are measured properly in a later Ticket.

### Refusals

`freyya_core::read_save` returns a Snapshot or one typed refusal:

| Refusal | When |
| ------- | ---- |
| Damaged | The header is not a Save header, the gamestate CRC32 or size does not match, Rakaly fails, or the text lacks a player or date. |
| Incomplete | The header says zip but no zip end exists, as in a cut or half-written file. |
| UnsupportedVersion | `meta_data.version` is not a checked version; only `1.15.2` is checked. |
| Modded | `meta_data.mods` lists one or more mods. |
| ReaderUnavailable | The Rakaly CLI cannot be started. |

The `mods` field name comes from Pdx-Unlimiter's Victoria 3 editor adapter, which reads `meta_data.mods` and `meta_data.dlcs` ([crschnick/pdx_unlimiter](https://github.com/crschnick/pdx_unlimiter), read 2026-11-28).
The Owner's vanilla Saves have neither field, so a hand-made text Save covers the modded case.

### How to repeat

1. Download `rakaly-0.8.26-x86_64-pc-windows-msvc.zip` from the [0.8.26 release](https://github.com/rakaly/cli/releases/tag/v0.8.26); its SHA-256 is `717aee5d3a2bd67455f77fed6f26ec2aaa6866832fb15baaced0161a00e604dd`.
2. Put `rakaly.exe` on `PATH`, or pass `--rakaly <PATH>`.
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
