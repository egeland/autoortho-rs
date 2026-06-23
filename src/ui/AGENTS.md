# Desktop UI Module

## Purpose

Desktop GUI using iced (elm-inspired MVU): setup wizard, settings, dashboard, dev tools.

## Ownership

- `mod.rs` — `AutoOrthoApp`, MVU `update()` / `view()`, font loading, app lifecycle
- `state.rs` — `AppState`, `Screen`, `ServiceStatus` types
- `simbrief_state.rs` — `SimBriefState` sub-struct for flight plan UI state
- `handlers/` — Message handlers, split by domain:
  - `mod.rs` — simple one-liner state setters (re-exported)
  - `cache.rs` — ClearDdsCache, SaveConfiguration
  - `dev_tools.rs` — FetchTestTile, TestFallbackLookup, SetTestLat/Lon/Zoom
  - `prefetch.rs` — PrefetchRoute, StopPrefetch, PrefetchProgress, PrefetchComplete*
  - `scenery.rs` — DownloadRegion, RefreshAvailableRegions, CancelDownload, UninstallRegion
  - `services.rs` — StartServices, StopServices, ServicesStarted, ServicesFailed
  - `setup.rs` — BrowseXPlanePath, BrowseCacheDir, FolderPicked
  - `simbrief.rs` — FetchSimbrief, SimbriefLoaded, SetTileProvider (coverage check)
  - `window.rs` — WindowOpened, WindowMoved, WindowResized, WindowCloseRequested
- `helpers.rs` — UI helper functions
- `screens/` — Individual screen implementations

## Screens

- `welcome.rs` — First-run wizard
- `setup.rs` — X-Plane path configuration
- `settings.rs` — Config editor (zoom, cache, provider)
- `dashboard.rs` — Live status, tile stats, flight info
- `developer.rs` — Debug tools, test tile generation
- `scenery.rs` — Scenery pack management
- `about.rs` — Version, links

## Local Contracts

- `RUNTIME` — Shared `OnceLock<Arc<Runtime>>` for async tasks
- `SAVED_WINDOW_GEOM` — Window geometry persistence
- `NERD_FONT` — Embedded FiraCode Nerd Font
- Screens return `Element<'_, Message>` from `view()`

## Work Guidance

- MVU: add variants to `Message` enum, handle in `update()`, render in `view()`
- `state.rs` owns all mutable app state
- Complex handlers go in `handlers/<domain>.rs`, simple setters stay in `handlers/mod.rs`
- `update()` should be ~200 LOC of one-liner delegations
- Embedded fonts in `assets/fonts/`

## Verification

- `cargo test --lib ui`
- Visual: run `cargo run -- --gui`

## Child DOX Index

### screens/

Screen implementations for each UI view.
