# Implementation Plan — Cartographer Splash & Setup Screens

## Goal
Add a retro pixel-art splash screen and game setup flow to the `cartographer` crate (and only `cartographer`), leading into the existing map TUI once configuration is complete.

## Requirements
1. Only modify files under `crates/cartographer/`.
2. Main splash screen:
   - Title: `"Rust in Case's archipelago"` rendered in a large retro pixel-block style (two lines: `RUST IN CASE'S` and `ARCHIPELAGO`) with off-white foreground and teal/cyan shadow, no version number, no music indicator.
   - Buttons: `[ NEW GAME ]`, `[ FROM SEED ]`, `[ QUIT ]`.
3. Secondary setup screen (after `NEW GAME`, or after entering a seed in `FROM SEED`):
   - Mode selection: `[ SEQUENTIAL ]` or `[ SIMULTANEOUS ]`.
   - `[ PARAMETERS ]` section listing all customizable `contractors::shared::Params` fields (`archipelago_radius`, `island_count`, `cliff_count`, `rock_count`, `vision_range`, `bombard_range`, `bombard_damage`, `hull_strength`, `round_cap`, `round_deadline_ms`) with interactive adjustment.
   - Confirmation (`Start`) transitions to the existing map TUI (`App`).
4. Structure the code across multiple focused modules with full doc comments (`clippy::pedantic` + `missing_panics_doc` clean, `cargo fmt --all` clean).

## Architecture
- `crates/cartographer/src/config.rs`: `TurnMode`, `GameConfig`, and uniform parameter descriptor table (`ParamKind` / `ParamSpec`) for `Params`.
- `crates/cartographer/src/title_art.rs`: 5x5 pixel bitmap font and layered shadow renderer for the main title with graceful fallback on small terminals.
- `crates/cartographer/src/splash.rs`: `SplashScreen`, `SplashOutcome`, event handling and Ratatui rendering for `MainMenu`, `SeedInput`, and `GameSetup` screens.
- `crates/cartographer/src/app.rs`: Existing `App` map explorer TUI and `render` function, moved out of `lib.rs` for modularity.
- `crates/cartographer/src/lib.rs`: Module declarations, public re-exports, `tui()` entry point running splash -> `RefChart::generate(seed, &params)` -> `App`, and test module wiring.
- Unit tests covering config/parameter bounds, splash navigation (`NEW GAME`, `FROM SEED`, `QUIT`), mode switching, parameter editing, and existing map rendering.
