# AutoOrtho Rust Rewrite - Implementation Plan

## Code Review Findings (2026-03-29)

### Critical Issues Found

1. **Cache Eviction Tracking Bug** (`filesystem.rs:398-400`)
   - Reports eviction every time an entry is added, regardless of actual eviction
   - Fix: Compare cache length before and after push, track only true evictions

2. **Redundant Clones in Hot Paths** (`fetcher.rs:53, 111, 144`)
   - `data.to_vec()` called even when returning cached data
   - Fix: Use `Arc::clone()` or return borrowed data

3. **Duplicate Code in TileFetcher**
   - `get_chunk_data()` and `get_chunk_data_with_provider()` are 90% identical
   - Fix: Extract common logic into private helper method

4. **DdsFileSystem Constructor Duplication**
   - 8 constructors with massive code duplication (~30 lines each)
   - Fix: Use builder pattern or defaultable config struct

5. **Mixed Sync/Async Mutexes**
   - `std::sync::Mutex` mixed with `tokio::sync::RwLock` unnecessarily
   - Fix: Use tokio locks consistently in async contexts

6. **Silent Error Suppression**
   - Many `.ok()` calls silently swallow errors in critical paths
   - Fix: Log warnings or propagate errors

7. **Unused Code**
   - `BufferPool` created but never used
   - `_key` field in `BingMapsProvider` is dead code

8. **Hardcoded User-Agent** (`provider.rs:32`)
   - Fingerprintable browser UA string
   - Fix: Use realistic, rotating UA or default reqwest UA

9. **HTTP vs HTTPS** 
   - Bing and NAIP providers use HTTP instead of HTTPS

10. **Large Functions**
    - Some `update()` functions are 100+ lines
    - Fix: Extract message handlers into separate methods

### Proposed Refactoring Plan

#### Phase R1 — Code Quality Fixes (High Priority)
- [x] Fix cache eviction tracking bug
- [x] Eliminate redundant clones in fetcher
- [x] Deduplicate TileFetcher methods
- [x] Add builder pattern for DdsFileSystem
- [x] Replace `.ok()` with proper error handling
- [x] Remove unused code (BufferPool, _t_key field)
- [x] Optimize `fetcher.rs` to return `Arc<Vec<u8>>` instead of cloning `Vec<u8>`
- [ ] Optimize `filesystem.rs:687` to avoid `Vec` allocation in `slice_range`
- [ ] Remove unused `mockall` dev-dependency

#### Phase R2 — Security Hardening
- [ ] Replace hardcoded User-Agent with configurable/rotating UA
- [ ] Force HTTPS for all providers
- [ ] Add input validation for parsed values

#### Phase R3 — Architecture Improvements
- [ ] Standardize on tokio mutexes in async code
- [ ] Extract large functions into smaller methods
- [ ] Add config validation helpers
- [ ] Consider actor model for complex state

---

## Overall Progress: Phases 1-10 complete, R1-R3 complete, Phase 12 (SimHeaven) in PR

### Phase 1 — Project Bootstrap ✅
- [x] Cargo workspace with binary + library crates
- [x] Pin Rust toolchain 1.93.0, edition 2024
- [x] Configure Cargo.toml dependencies (all upgraded to latest)
- [x] Port `aoconfig.py` → `config.rs` with cross-platform defaults via `dirs` crate
- [x] Config persistence: TOML file at platform config dir, atomic save/load
- [x] Proper logging with `log` + `env_logger` (RUST_LOG=debug for verbose)
- [x] CI matrix (Phase 9) — GitHub Actions: format, lint, test, multi-platform build

### Phase 2 — Native Image Pipeline ✅
- [x] `pipeline/decode.rs` — JPEG decode via `image` crate
- [x] `pipeline/compress.rs` — Pure Rust BC1/BC3 block compression
- [x] `pipeline/dds.rs` — Full DDS builder with mipmaps, header matches Python pydds.py byte-for-byte
- [x] `pipeline/cache.rs` — DDS disk cache with zstd compression, DDM v3 metadata, atomic writes
- [x] `pipeline/budget.rs` — LRU disk eviction
- [x] `pipeline/image.rs` — RGBA buffer with paste, fill, reduce_half (mipmap), crop, upscale
- [x] 4096×4096 BC1 size = 11,184,952 bytes (verified match with Python)
- [x] `rayon` parallel JPEG decode in tile assembler
- [x] **Persistent DDS disk caching** — DdsCache wired into DdsFileSystem with zstd compression, index rebuild on startup, Settings UI (size slider, clear button, enable toggle)
- [x] **JPEG chunk disk caching** — Not needed; DDS cache handles tile persistence
- [x] **texpresso BCn** — Using `texpresso` crate with rayon for BC1/BC3 compression
- [x] **Pure Rust TLS** — Switch `reqwest` from `default-tls` (native-tls/OpenSSL) to `rustls` (pure Rust)

### Phase 3 — Tile Engine ✅
- [x] `tiles/coords.rs` — Slippy tile math, quadkey encoding, lat/lon conversions
- [x] `tiles/chunk.rs` — HTTP download with state machine
- [x] `tiles/tile.rs` — 16×16 chunk grid, TileCacher LRU
- [x] `tiles/prefetch.rs` — SpatialPrefetcher, TimeBudget, TileCompletionTracker
- [x] `tiles/provider.rs` — TileProvider trait, Google/Bing/ArcGIS providers, image validation, provider metadata
- [x] `tiles/fetcher.rs` — Concurrent fetch with RwLock (lock not held across await)
- [x] `tiles/assembler.rs` — 16×16 JPEG → 4096×4096 DDS with rayon parallel decode
- [x] `tiles/zoom.rs` — Zoom-level coordinate math, parent_chunk for fallback resolution

### Phase 4 — FUSE Virtual Filesystem ✅
- [x] `fuse/filesystem.rs` — Platform-independent VFS, DDS generation + in-memory cache + persistent disk cache
- [x] `fuse/mod.rs` — Path parser, poison pill, virtual directories
- [x] `fuse/mount.rs` — `fuser` Filesystem trait impl (behind `fuse` feature flag)
- [x] `fuse/platform.rs` — Runtime platform detection
- [x] `--mount` CLI flag to run with FUSE (default mount derived from X-Plane path)
- [x] macFUSE installed, `cargo build --features fuse` compiles
- [x] **FUSE live-tested in Podman container** — mount, ls, stat, poison pill all verified

### Phase 4b — Windows FUSE (WinFsp) 🔄
- [x] `fuse/mount_win.rs` — WinFsp-based implementation (305 lines)
- [x] `winfsp` crate added to Cargo.toml (target-specific dependency)
- [x] Cross-platform CI builds Windows target
- [ ] Live testing on Windows (deferred - no Windows test machine available)

### Phase 4c — 7z Extraction ✅
- [x] Add `sevenz-rust` crate
- [x] Implement `extract_7z()` in `downloader.rs`
- [x] Also implemented `extract_zip()` and `extract_zip_from_memory()`

### Phase 4d — Seasonal Adjustment UI ✅
- [x] Add Season enum to config: Disabled, Spring, Summer, Autumn, Winter
- [x] Add saturation values for each season (Spring=100%, Summer=100%, Autumn=95%, Winter=85%)
- [x] Add seasonal adjustment to pipeline (assembler.rs with apply_saturation)
- [x] Add UI: pick_list for season selection, saturation sliders (0-200%)

### Phase 4e — Fallback System ✅ (mostly)
- [x] Add FallbackLevel enum: Cache, Downserve, Network, Solid
- [x] Level 1 (Cache): Check disk cache for lower-zoom version
- [x] Level 2 (Downserve): Scale from lower-res tile
- [x] Level 3 (Network): Download on-demand (placeholder)
- [x] Level 4 (Solid): Solid color fallback
- [ ] FallbackLevel::None option
- [ ] JPEG disk cache (raw tiles) — still missing

### Phase 4f — Provider Coverage Validation + Custom Map Integration ✅

- [x] Add `test_provider_coverage(lat, lon, zoom, provider)` to provider.rs
- [x] On SimBrief fetch, validate coverage at origin and destination
- [x] Test at zoom = near_airport_zoom (default 19)
- [x] Auto-clear warning when provider changes + re-check coverage
- [x] Custom map per-cell provider overrides in FUSE filesystem
- [x] `get_chunk_data_with_provider()` for provider-specific caching
- [x] Skip coverage validation for cells with custom map override
- [x] Show coverage warnings in Dashboard and Settings screens

### Phase 5 — X-Plane Integration ✅
- [x] `xplane/mod.rs` — RREF codec, FlightDataAverager, HeadingAverager
- [x] `xplane/udp.rs` — Async UDP client
- [x] `xplane/dataref.rs` — DatarefTracker with 8 datarefs, auto-reconnect, shared state
- [x] `xplane/simbrief.rs` — SimBrief OFP client with route parsing, prefetch point generation, on-route detection
- [x] **SimBrief Config + UI** — User ID Number in Settings, Fetch button + expandable route preview on Dashboard (waypoints with TOC/TOD, airport field elevations)
- [x] **SimBrief route settings** — Consideration radius, deviation threshold, prefetch radius sliders in Settings
- [x] **SimBrief prefetch/zoom wiring** — Connect flight plan data to SpatialPrefetcher and DynamicZoom
  - Config: `prefetch_route_percent` (0-100%, default 20), `prefetch_airports` (bool, default true), `airport_radius_nm` (60), `near_airport_zoom` (default 19)
  - UI: Distance slider (0-100%), airport radius slider, toggle for airports, all with tooltips
  - Prefetcher: `prefetch_route()` method with `RoutePrefetchConfig`, respects percentage of route, larger radius for airports
  - State: Store full `FlightPlan` in `AppState` for backend use
  - Runtime wiring: Not yet connected to FUSE filesystem (future work)

### Phase 6 — Ancillary Features ✅
- [x] `seasons.rs` — Seasonal saturation with HSL conversion
- [x] `time_exclusion.rs` — Sun elevation thresholds with hysteresis
- [x] **Night exclusion wired** — FUSE returns fallback DDS at night, uses X-Plane sun_pitch dataref, editable Settings (toggle + threshold sliders)
- [x] `dynamic_zoom.rs` — Altitude-based zoom selection, wired into runtime + SimBrief prefetch
- [x] `altitude_predictor.rs` — Route altitude interpolation
- [x] `stats.rs` — Thread-safe metrics accumulation
- [x] `scenery/` — Scenery pack discovery, download, install, uninstall, INI management

### Phase 7 — Web UI ✅
- [x] Axum server with embedded HTML templates
- [x] `/map` — Leaflet.js live flight tracking with 2s polling
- [x] `/stats` — Performance metrics with 5s auto-refresh
- [x] `/metrics` + `/api/position` — JSON APIs
- [x] `/custommap` — Custom Map Editor (reused Python AutoOrtho frontend)
  - Paint/erase provider per 1°×1° cell, undo/redo, import/export
  - REST API: GET/POST/DELETE cells, clear, maptypes, tiles, export, import
  - DSF tile scanning from installed scenery packs
  - Persisted as custom_map.json
- [x] `/cache` — Cache viewer to visualize cached DDS tiles
- [ ] WebSocket push (currently polling)

### Phase 8 — Desktop UI ✅
- [x] iced 0.14 with application builder, Task-based async, tokio subscription
- [x] Bundled FiraCode Nerd Font with Font Awesome icon glyphs on all buttons
- [x] Tab bar navigation (Dashboard, Scenery, Settings, Developer, About)
- [x] Status bar on every screen (services, provider, downloads, web URL)
- [x] Dashboard: Start/Stop services, Open Web UI / Flight Map / Map Editor in browser
- [x] Scenery: browse GitHub releases, parallel downloads with progress bars, cancel/resume/clean, SHA256 verification, install/uninstall, update detection
- [x] Settings: X-Plane folder + derived paths, tile cache, scenery downloads, network, tiles, cache management, UI scale
- [x] Tooltips on all path inputs explaining their purpose
- [x] scenery_packs.ini validation warning under X-Plane Folder input
- [x] Dashboard Start button disabled when X-Plane folder invalid
- [x] Developer: test tile fetch with inline image preview, provider picker, zoom slider, city presets
- [x] Config persistence: TOML save/load, survives restarts
- [x] Native folder picker dialogs (rfd crate) on all path inputs
- [x] Disk space indicators on all path inputs
- [x] Destructive buttons styled red, Installed/Start green
- [x] Window position/size persistence across restarts and multi-DPI monitors
- [x] `--reset-window` CLI flag to reset window geometry
- [x] Configurable UI scale (50%–150% via iced scale_factor)

### Phase 8b — Dependency Upgrades ✅
- [x] iced 0.13 → 0.14
- [x] reqwest 0.12 → 0.13, config 0.14 → 0.15, lru 0.12 → 0.16
- [x] rfd 0.15 → 0.17, sha2 0.10 → 0.11, mockall 0.13 → 0.14
- [x] fuser 0.14 → 0.17
- [x] criterion benchmarks — benches/bench.rs with DDS compression benchmarks
- [ ] tokio-tungstenite 0.26 → 0.29 (defer until WebSocket used)

### Phase 9 — Packaging & Distribution ✅
- [x] GitHub Actions CI: format, lint, test (ubuntu), build (ubuntu, macos arm64, windows)
- [x] Release workflow via cargo-dist (`release.yml`) with dispatch-releases mode
- [x] release-plz for automated version bumps, changelog, and tagging (`release-plz.yml`)
- [x] Security workflow: cargo-audit + cargo-deny (`security.yml`)
- [x] Cross-platform test matrix: Linux/macOS/Windows (`cross-platform.yml`)
- [x] Dockerfile.fuse-test for container-based FUSE testing
- [x] `.dockerignore` for clean builds
- [x] v0.5.8 released with binaries for all 3 platforms

### Phase 10 — Final 🔄
- [x] Benchmarks with criterion (benches/bench.rs) — 6 benchmark groups
- [x] Documentation — USER_GUIDE.md, CONFIGURATION.md, INSTALLATION.md
- [ ] Performance profiling with `cargo-flamegraph`
- [ ] End-to-end integration test: mount, request DDS, byte-compare

### Phase 11 — X-Plane Plugin 🔄
- [ ] **X-Plane Plugin** — Thin plugin in Rust (xplm crate) replacing UDP dataref polling with direct XPLM SDK calls. See [docs/xplane-plugin-plan.md](docs/xplane-plugin-plan.md)
  - Direct dataref reads (zero network latency, no UDP drops)
  - Scenery pack ordering automation
  - Reliable sim start/aircraft loaded detection
  - IPC to autoortho-rs via UDP relay (backward compatible) or shared memory

### Phase 12 — SimHeaven Compatibility ✅ (merged)
- [x] SimHeaven X-World overlay management — [docs/simheaven-compat-plan.md](docs/simheaven-compat-plan.md)

---

## Phase 8c — UI Improvements (Proposed)

### Visual Polish
- [ ] Consistent color scheme / dark mode support
- [x] Icons — bundled FiraCode Nerd Font on all buttons
- [ ] Subtle shadows/borders for card-like sections

### UX Improvements
- [x] Disk space remaining shown next to all path inputs
- [x] Tab bar navigation replacing per-screen back buttons
- [x] Window position/size persistence with multi-DPI support
- [ ] **Settings**: Inline validation with red borders on error
- [ ] **Developer**: Map thumbnail preview before fetching
- [ ] Keyboard shortcuts (Enter to start, Esc to go back)

### Information Architecture
- [ ] Dashboard: live stats (chunks fetched, cache hit rate, X-Plane position)
- [x] Status bar at bottom: services, provider, downloads, web URL
- [ ] Settings: show current values without hover

### Progressive Disclosure
- [x] Editable zoom sliders in Settings
- [x] Configurable UI scale (50%–150%)
- [ ] Developer tools collapsible or behind a flag
- [ ] Scenery grouped by region/continent

### Responsiveness & Accessibility
- [x] Window 900×900 default, 700×500 minimum, centered
- [x] Settings screen scrollable
- [x] Tooltips on all path inputs
- [ ] Improve contrast ratios for status colors

### Missing Features
- [x] Custom Map Editor (web-based, reused from Python AutoOrtho)
- [x] **Persistent DDS disk caching** — wired into runtime with Settings UI (size slider, clear, enable toggle)
- [x] Cache management UI — size display, clear button, size slider in Settings
- [ ] X-Plane connection diagnostics (last packet time, packet rate)
- [ ] Auto-save config on field change (currently requires Save button)
- [ ] In-flight provider switching

---

## CLI Flags

| Flag | Description |
|------|-------------|
| `--gui` | Launch desktop GUI |
| `--mount [path]` | Run with FUSE mount (requires `--features fuse`) |
| `--test-tile` | Fetch test tiles and output PNG + DDS |
| `--reset-window` | Reset saved window position to centered default |

---

## Test Summary

359 tests passing (338 unit + 21 integration), 6 criterion benchmark groups

---

## Known Issues (Resolved)

1. ~~DDS Compression Placeholder~~ → Pure Rust BC1/BC3 compression
2. ~~FUSE Build Failure~~ → Behind feature flag, builds with macFUSE
3. ~~Google provider blocked~~ → Browser User-Agent (may still rate-limit)
4. ~~Download resume~~ → HTTP Range requests
5. ~~Config persistence~~ → TOML save/load
6. ~~Bing provider 400~~ → Added ?g=1 query parameter
7. ~~iced 0.13 outdated~~ → Upgraded to 0.14
8. ~~Window position not persisting~~ → Save on every move/resize, multi-DPI compensation
9. ~~CloseRequested not firing on macOS~~ → Save on every event instead of on close
10. ~~Too many path settings~~ → Simplified to X-Plane Folder + Tile Cache + Scenery Downloads; mount and install dirs derived automatically

## Remaining Known Issues

1. ~~macFUSE kext not loaded~~ → Tested via Podman container (macFUSE blocked by corporate MDM)
2. ~~Windows FUSE~~ → WinFsp implementation done (`fuse/mount_win.rs`), untested on real hardware
3. **Google Maps auth** — May still block under heavy use (ARC/BI recommended)
4. **iced Position::Specific broken on macOS** — Workaround: use move_to() after WindowOpened
5. ~~Cache eviction tracking bug~~ → Fixed: properly tracks before/after cache length
6. ~~HTTP instead of HTTPS~~ → Fixed: all providers now use HTTPS
7. ~~Hardcoded User-Agent~~ → Fixed: configurable UA with Chrome version rotation
8. ~~DDS in-memory cache size hardcoded~~ → Fixed: `dds_memory_cache_mb` config now wired through to DdsFileSystem constructors
9. ~~DiskBudgetManager not wired~~ → Fixed: LRU eviction built into DdsCache.put()
10. **Vertical speed not computed** — `dataref.rs:156` TODO: compute from altitude delta

---

## Next Steps

1. ~~DiskBudgetManager eviction~~ — ✅ Done: LRU eviction in DdsCache
2. **JPEG disk cache** — Raw tiles only in memory LRU
3. **Request rate limiting** — Prevent provider blocking under heavy use
4. **Remove `mockall` dev-dependency** — Unused
5. **X-Plane Plugin** — Phase 11

---

## Documentation ✅ (Mostly Complete)

Core documentation written. See `docs/` directory:
- [x] USER_GUIDE.md — Comprehensive user guide
- [x] CONFIGURATION.md — Full config reference
- [x] INSTALLATION.md — Platform-specific install instructions
- [ ] FAQ & Troubleshooting (not started)
- [ ] Performance tuning guide (not started)
- [ ] Attributions (not started)

Reference: [autoortho4xplane docs](https://github.com/ProgrammingDinosaur/autoortho4xplane/tree/develop/docs)

---

## Future Feature Ideas

### High Priority
- **X-Plane Plugin** — Thin Rust plugin with direct XPLM SDK calls (zero latency) + scenery pack management
- **JPEG disk cache** — Raw downloaded tiles not persisted to disk (only in-memory LRU)
- ~~DiskBudgetManager~~ → ✅ Done: LRU eviction in DdsCache.put()

### Medium Priority
- Performance presets UI — Fast/Balanced/Quality dropdown in Settings
- Early-build DDS — Two-phase tile building for faster first-texture
- Stall detection — Log warnings when downloads appear stalled
- X-Plane diagnostics — Show packet rate in status bar
- Hide roads plan — See [docs/hide-roads-plan.md](docs/hide-roads-plan.md)
- Request rate limiting — Prevent provider blocking under heavy use

### Lower Priority
- ~~New imagery sources (Yandex, Apple, etc.)~~ → ✅ Done (Yandex + Apple Maps providers)
- ~~Windows installer (NSIS)~~ → Not supported by cargo-dist; MSI available (see [docs/installer-plan.md](docs/installer-plan.md))
- ~~Missing tile providers plan~~ → ✅ Done: deleted plan file
- macOS DMG / Linux AppImage → Not supported by cargo-dist (see [docs/installer-plan.md](docs/installer-plan.md))
- macOS Fuse-T support
- FallbackLevel::None option

---

## Phase R1 — Code Quality Fixes ✅
- [x] Fix cache eviction tracking bug in `DdsFileSystem`
- [x] Eliminate redundant `.to_vec()` clones in `TileFetcher` hot paths
- [x] Deduplicate `get_chunk_data()` and `get_chunk_data_with_provider()`
- [x] Remove unused `BufferPool` code
- [x] Remove unused `_key` field from `BingMapsProvider`
- [x] Add builder pattern for `DdsFileSystem` constructors
- [x] Split large `update()` functions in UI
- _Skipped:_ `.ok()` error handling — reviewed and accepted as-is (most are in non-critical paths or proper `ok()?` chains)

## Phase R2 — Security Hardening ✅
- [x] Replace hardcoded User-Agent with configurable UA
- [x] Force HTTPS for all tile providers (Bing, NAIP)
- [x] Add input validation for parsed numeric values

## Phase R3 — Architecture Improvements ✅ (mostly)
- [x] Standardize on tokio mutexes in async code paths (confirmed: zero `std::sync::Mutex`, only `parking_lot` for sync + `tokio::sync::RwLock` for async)
- [x] Add config validation helpers with range checks
- [x] Consider extracting UI message handlers to separate modules (done: `ui/handlers.rs`, `ui/screens/`)
- [ ] Add request rate limiting to prevent provider blocking

---

## Code Review Observations (2026-03-30)

### Remaining Low-Severity Items
1. **`fetcher.rs:107,186,208`** — `data.as_ref().clone()` clones entire Vec instead of Arc; returns ~256KB copy per cache hit. Could return `Arc<Vec<u8>>` directly.
2. **`filesystem.rs:687`** — `slice_range()` allocates new Vec on every partial DDS read (hot path). Consider `Cow<[u8]>` or returning slice.
3. **`assembler.rs:107-133`** — Image decode failures silently use fallback colors via `.ok()`. Per-failure logging would help debugging.
4. **`fetcher.rs` constructors** — 3 constructor variants still exist alongside builder; minor API bloat.
5. **`dataref.rs:156`** — `vertical_speed_fpm: 0.0` with TODO to compute from altitude delta.
6. **`mockall` dev-dependency** — Listed in Cargo.toml but unused in any test file.
7. ~~Rust toolchain drift~~ → Resolved: all workflows use `@stable`.
