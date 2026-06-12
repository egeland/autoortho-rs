# UI Screens

## Purpose

Individual screen implementations for the iced desktop UI.

## Ownership

Each file implements one screen's `view()` and associated message handling:

- `welcome.rs` — First-run welcome wizard
- `setup.rs` — X-Plane path configuration
- `settings.rs` — Config editor (zoom rules, cache, provider, UI scale)
- `dashboard.rs` — Live status display, tile stats, flight info
- `developer.rs` — Debug tools, test tile generation
- `scenery.rs` — Scenery pack management UI
- `about.rs` — Version info, links

## Local Contracts

- Each screen exports a `view()` function returning `Element<'_, Message>`
- Screens use `AppState` from `../state.rs`
- Message variants defined in parent `mod.rs`

## Work Guidance

- Add new screens by creating a file here and adding to `mod.rs`
- Keep screen logic minimal — complex handlers go in `handlers.rs`
- Use `helpers.rs` for shared UI components

## Verification

- `cargo test --lib ui`
- Visual testing via `cargo run -- --gui`
