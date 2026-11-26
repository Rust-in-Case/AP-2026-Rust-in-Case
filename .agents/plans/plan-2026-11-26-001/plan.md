# Implementation Plan

## Goal
Export `render(map: &impl Map) -> io::Result<()>` from the `cartographer` crate, providing a minimal Ratatui 0.30.2 TUI with:
- A left-side pane containing a single input box (`Command / Input`).
- A right-side pane containing a movable/pannable axial hex map (`+ +` / `+ + +` / `+ +` for radius 1), rendering distinct emojis/symbols per `Terrain` type.
- Leave all other workspace packages untouched (keep `tui()` delegating to `render()` so `crates/game` continues to compile).
- Satisfy all `clippy::pedantic`, `missing_panics_doc`, `missing_errors_doc`, `rustfmt`, and `AGENTS.md` documentation requirements.

## Steps
1. **Refactor `crates/cartographer/src/lib.rs`**:
   - Add `pub fn render(map: &impl Map) -> io::Result<()>` as the main exported entry point.
   - Update `pub fn tui() -> io::Result<()>` to generate the reference chart and call `render(chart.map())` so `crates/game` remains unbroken without touching any other crate.
   - Add doc comments to every public item (`App`, `App::new`, `App::run`, `render`, `tui`), including `# Errors` sections.
   - Fix all 14 `clippy::pedantic` lints (`map_unwrap_or`, `cast_possible_wrap`, `cast_possible_truncation`, `cast_lossless`, `cast_sign_loss`, `missing_errors_doc`).
2. **Add unit tests in `crates/cartographer/src/lib.rs`**:
   - Implement a minimal mock `Map` with `radius = 1` covering `Clear`, `Cliff`, `Island`, `Rock`, and `None` cells.
   - Verify `build_visible_map_lines` produces the staggered hex layout (` + +`, `+ + +`, ` + +`) and expected terrain symbols, as well as camera panning bounds.
3. **Validate**:
   - Run `cargo fmt --all -- --check`
   - Run `cargo clippy --workspace --all-targets`
   - Run `cargo test -p cartographer`
4. **Document**:
   - Write `result.md` in this plan directory.
