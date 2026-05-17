# Wiki extraction pipeline

Regenerates every Markdown page in this wiki from a local Solar Expanse install.

## Usage

```bash
./extract.sh
```

That's it. The script:

1. Downloads AssetRipper (first run only, into `tools/`).
2. Spins up AssetRipper in headless mode, loads the game's `Solar Expanse_Data`
   folder, and exports a Unity project tree into `cache/project/`.
3. Builds the three Rust binaries (first run only).
4. Runs the binaries in sequence:
   - `parse-locale` reads `StreamingAssets/Languages/en-US.csv` → `cache/locale.json`
   - `parse-stats` reads `cache/project/ExportedProject/Assets/Scenes/MyScene.unity` → `cache/stats.json`
   - `gen-pages` reads both → writes every wiki page under the repo root

Total runtime: ~30 s on first run (AssetRipper export dominates), ~3 s after that.

### Useful flags

| Variable | Effect |
| --- | --- |
| `SOLAR_EXPANSE_DATA=<path>` | Point at a non-default game install. |
| `FAST=1` | Skip the AssetRipper step and reuse `cache/project`. Use after the first run to iterate on Rust code. |
| `ASSETRIPPER_VERSION=<tag>` | Pin a specific AssetRipper release. |

## Layout

```
extract/
├── extract.sh                  # bash orchestrator
├── Cargo.toml                  # one Rust crate, three binaries
├── src/bin/
│   ├── parse_locale.rs         # CSV → JSON
│   ├── parse_stats.rs          # Unity scene YAML → JSON
│   └── gen_pages.rs            # JSON → Markdown
├── tools/                      # AssetRipper download (gitignored)
└── cache/                      # AssetRipper export + intermediate JSON (gitignored)
```

## Where the numbers come from

| Field | Source MonoBehaviour | Notes |
| --- | --- | --- |
| Planet mass, radius, orbit | `SolarBody` | Stored in proper units (10²⁴ kg, km, AU). |
| Moon mass | `NBody` | Same units as planet mass. |
| Moon orbit | `OrbitUniversal` | Perihelion / eccentricity are scaled (×1000 vs AU). `gen-pages` divides by 1000 before converting to km. |
| Moon parent | `OrbitUniversal.centerNbody` PPtr | Resolved by walking back through the NBody on the parent GameObject. |
| Names + descriptions | `StreamingAssets/Languages/en-US.csv` | Localization keys grouped by namespace. |

## Tests

```bash
cargo test
```

Every binary has unit tests in a `#[cfg(test)]` module at the bottom of its
file. The most load-bearing test —
`parses_block_headers_without_trailing_space` in `parse_stats.rs` — exists
because the first version of the YAML parser tried to match `!u!1 ` (with a
trailing space) against the actual tag `!u!1` (no trailing space) and silently
returned zero bodies. Add a test for every new field you extract.
