# AutoOrtho Rust Rewrite - Implementation Plan

## Overall Progress: Phases 1-9 mostly complete, Phase 10-11 remaining

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

### Phase 4b — Windows Dokan FUSE (Deferred)
- [ ] Windows Dokan implementation (deferred - no Windows test machine available)
- [ ] Add `dokan` crate for Windows
- [ ] Create `fuse/dokan.rs` implementing DokanFileSystem trait
- [ ] Add config option `prefer_dokan` (default true)

### Phase 4c — 7z Extraction ✅
- [x] Add `sevenz-rust` crate
- [x] Implement `extract_7z()` in `downloader.rs`
- [x] Also implemented `extract_zip()` and `extract_zip_from_memory()`

### Phase 4d — Seasonal Adjustment UI ✅
- [x] Add Season enum to config: Disabled, Spring, Summer, Autumn, Winter
- [x] Add saturation values for each season (Spring=100%, Summer=100%, Autumn=95%, Winter=85%)
- [x] Add seasonal adjustment to pipeline (assembler.rs with apply_saturation)
- [x] Add UI: pick_list for season selection, saturation sliders (0-200%)

### Phase 4e — Fallback System
- [ ] Add FallbackLevel enum: None, Cache, Full
- [ ] Level 1 (Cache): Check disk cache for lower-zoom version
- [ ] Level 2 (Scale): Scale from lower mipmap
- [ ] Level 3 (Network): Download lower-detail imagery on-demand

### Phase 4f — Provider Coverage Validation + Custom Map Integration

#### Feature 1: Provider Suggestion on SimBrief Fetch ✅
- [x] Add `test_provider_coverage(lat, lon, zoom, provider)` to provider.rs
- [x] On SimBrief fetch, validate coverage at origin and destination
- [x] Test at zoom = near_airport_zoom (default 19)
- [ ] Auto-clear warning when provider changes (deferred)

#### Feature 2: Custom Map Integration (Infrastructure Only)
- [x] Add CustomMapStore field and provider resolution methods to DdsFileSystem
- [ ] Implement actual tile fetching with cell-specific providers (deferred)
- [ ] Skip coverage validation for cells with custom map (deferred)

#### Feature 3: UI Warnings ✅
- [x] Add `simbrief_coverage_warning` field to AppState
- [x] Dashboard: Show warning in SimBrief section
- [x] Settings: Show warning in SimBrief section

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
- [x] `dynamic_zoom.rs` — Altitude-based zoom selection, wired with SimBrief altitude when on-route
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
- [ ] tokio-tungstenite 0.26 → 0.29 (defer until WebSocket used)
- [ ] criterion 0.5 → 0.8 (defer until benchmarks written)

### Phase 9 — Packaging & Distribution ✅ (mostly)
- [x] GitHub Actions CI: format, lint, test (ubuntu), build (ubuntu, macos arm64, windows)
- [x] release-please for conventional-commit semver + changelog
- [x] Release workflow builds and uploads binaries for Linux, macOS, Windows
- [x] Dockerfile.fuse-test for container-based FUSE testing
- [x] `.dockerignore` for clean builds
- [ ] Bundle libispc_texcomp pre-built binaries (optional perf upgrade)
- [ ] scenery_packs.ini auto-configuration

### Phase 10 — Final 🔄
- [ ] Benchmarks with criterion
- [ ] Performance profiling with `cargo-flamegraph` and/or `cargo-profiler`
- [ ] End-to-end integration test: mount, request DDS, byte-compare
- [ ] Documentation pass

### Phase 11 — X-Plane Plugin 🔄
- [ ] **X-Plane Plugin** — Thin plugin in Rust (xplm crate) replacing UDP dataref polling with direct XPLM SDK calls. See [docs/xplane-plugin-plan.md](docs/xplane-plugin-plan.md)
  - Direct dataref reads (zero network latency, no UDP drops)
  - Scenery pack ordering automation
  - Reliable sim start/aircraft loaded detection
  - IPC to autoortho-rs via UDP relay (backward compatible) or shared memory

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

308 tests passing (296 unit + 12 integration)

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
2. **Windows FUSE** — WinFsp implementation not started
3. **Google Maps auth** — May still block under heavy use (ARC/BI recommended)
4. **iced Position::Specific broken on macOS** — Workaround: use move_to() after WindowOpened

---

## Next Steps

1. **Test on Windows**: Build and test with X-Plane on a Windows machine
2. **texpresso BCn** compression upgrade
3. **JPEG chunk disk caching** (lower priority)
4. **SimBrief full integration**
5. **Night exclusion wiring**

---

## Documentation Plan (mirroring autoortho4xplane/docs)

Source: https://github.com/ProgrammingDinosaur/autoortho4xplane/tree/develop/docs

### docs/README.md — Landing / Quick Start
- [ ] Overview of project (Rust rewrite of autoortho4xplane)
- [ ] Feature comparison vs Python original
- [ ] Quick start for each OS: Linux (FUSE), Windows (WinFsp), macOS (macFUSE/Fuse-T)
- [ ] Requirements and compatibility
- [ ] Known issues and limits
- [ ] Links: docs, FAQ, GitHub Discussions, releases
- [ ] License (Apache 2.0 OR GPL-3.0)
- [ ] Donation link (if applicable)

### docs/install.md — Platform Installation Guides
- [ ] **Linux**: FUSE setup, `user_allow_other` in `/etc/fuse.conf`, download + chmod + run
- [ ] **Windows**: WinFsp/Dokan setup, zip extract, run exe, experimental installer
- [ ] **macOS**: MacFUSE or Fuse-T, quarantine removal (`xattr -d com.apple.quarantine`), signed binary note
- [ ] scenery_packs.ini ordering guide (image reference)
- [ ] Build from source instructions (cargo build --release, `cargo install` options)

### docs/config.md — Configuration Reference
- [ ] Scenery install path (custom location, external drives, space requirements)
- [ ] X-Plane install path (must be correct — Custom Scenery dir detection)
- [ ] Download directory (temp storage, Windows Defender exception recommendation)
- [ ] Config file location (`~/.autoortho` or platform equivalent)
- [ ] **Performance tuning settings** (tile time budget, fallback level, spatial prefetching, dynamic zoom)
- [ ] **Native pipeline settings** (ephemeral DDS cache, thread count)
- [ ] SimBrief integration config
- [ ] Dynamic zoom quality steps
- [ ] Time exclusion settings

### docs/details.md — How It Works
- [ ] High-level approach: virtual filesystem intercepting DDS reads
- [ ] X-Plane scenery structure: Custom Scenery, DSF files, mipmaps
- [ ] How this project intercepts and serves imagery on-demand
- [ ] Caching strategy (in-memory LRU, disk cache)
- [ ] Expert usage: custom scenery, Ortho4XP usage tips, skip_downloads config

### docs/faq.md — FAQ & Troubleshooting
- [ ] **Application file structure** (don't delete bundled files)
- [ ] Missing color/green tiles: causes + solutions (lower zoom, increase budget, fallbacks)
- [ ] Long loading at startup: zoom level impact, startup suspend_maxwait
- [ ] Stuttering: time budget, cache-only fallbacks, prefetch tuning
- [ ] Changed settings but no effect (restart, clear cache, fly new area)
- [ ] maxwait vs tile_time_budget difference
- [ ] Time exclusion re-activation after scenery reload (already fixed)
- [ ] X-Plane "Failed to find resource" error (AutoOrtho not running, broken symlinks)
- [ ] Reinstall scenery pack (delete \_info.json)
- [ ] X-Plane doesn't auto-start (must start separately, after AutoOrtho)
- [ ] **Linux**: Too Many Open Files (ulimit -n 8192), FUSE error (user_allow_other), mount cleanup after crash
- [ ] **Windows**: Python version issues, Windows Defender slowness (exclude directory), false positive AV detection, non-NTFS/local drive error
- [ ] **macOS**: Flight crash during loading (disable multithreaded FUSE)
- [ ] Base mesh package issues (redirect to kubilus scenery issues)
- [ ] How to submit bug reports (screenshots, logs, OS, versions)

### docs/info.md — Attributions
- [ ] Ortho4XP attribution
- [ ] Slippy Map Tilenames (OpenStreetMap wiki)
- [ ] Icon attribution (Vital Gorbachev / icons8)
- [ ] 7-Zip / LZMA SDK (GNU LGPL) for DSF compression
- [ ] Rust crate attributions (list key crates)

### docs/performance.md — Performance Tuning Guide
- [ ] Performance presets (Fast / Balanced / Quality / Custom)
- [ ] Zoom level as critical factor (table: ZL vs chunks vs relative resources)
- [ ] Time budget system (use_time_budget, tile_time_budget, how it works)
- [ ] Fallback system (none / cache / full, fallback chain explained, extends_budget)
- [ ] Extended fallback timeout
- [ ] Startup loading behavior (suspend_maxwait, 10× multiplier, stall detection)
- [ ] Dynamic zoom (AGL vs MSL, quality steps table, altitude prediction)
- [ ] Spatial prefetching (prefetch_enabled, lookahead, velocity-based vs SimBrief)
- [ ] Per-chunk timeout (maxwait, when time budget is disabled)
- [ ] Recommended configurations (stutter-free, max quality, slow internet, weak CPU)
- [ ] Statistics reference (mm_count, chunk_budget_skipped, chunk_missing_count)
- [ ] Tile creation time breakdown (download+compose vs compression)
- [ ] Time exclusion (sun elevation, safety features, decision preservation during reload)
- [ ] SimBrief integration deep dive (route consideration radius, deviation threshold, prefetch radius)
- [ ] **Native pipeline architecture** (our Rust pipeline replaces the C aopipeline — document rayon, reqwest, ISPC texcomp equivalent)

### docs/scenery.md — Scenery Setup
- [ ] Using provided scenery packs (must use config utility, don't manually move)
- [ ] scenery_packs.ini ordering (with example: landmarks → yAutoOrtho_Overlays → z_ao_* → z_autoortho → zzz_global_scenery)
- [ ] Installing in custom directories (manual symlinks/shortcuts, scenery_packs.ini editing)

---

## Ideas from autoortho4xplane to Consider

### High Priority
- [ ] **X-Plane Plugin** — Thin Rust plugin replacing UDP dataref polling with direct XPLM SDK calls (zero latency, no drops) + scenery pack management. See [docs/xplane-plugin-plan.md](docs/xplane-plugin-plan.md)
- [ ] **Performance presets UI** — Fast/Balanced/Quality/Custom dropdown in Settings that sets all tuning params at once
- [ ] **Early-build DDS** — Two-phase tile building: build at 90% chunks with placeholder, heal when remaining arrive (reduces first-texture latency)
- [ ] **Stall detection** — Log warnings at 60s and 180s when downloads appear stalled (server throttling indicator)
- [x] **Disk cache cleanup UI** — Cache size, clear button, size slider in Settings
- [ ] **X-Plane diagnostics** — Show last packet time, packet rate in status bar

### Medium Priority
- [ ] **Tile caching monitoring** — Visual view of which tiles are cached (mentioned in autoortho4xplane backlog)
- [ ] **Windows installer** — Experimental NSIS or WiX installer for Windows (like autoortho4xplane's .exe installer)
- [ ] **SimBrief route deviation threshold** — Warn or fall back when 40nm+ off route
- [ ] **Automatic cache eviction** — OS temp directory for ephemeral DDS, OS file cache for hot files (our cache.rs already uses temp)
- [ ] **Seasonal ortho support** — Merge hotbso/autoortho changes for seasonal imagery (in autoortho4xplane backlog)

### Lower Priority
- [ ] **Custom mesh packages** — Rebuild base mesh with newer Ortho4XP for XP12 3D water support (autoortho4xplane planned)
- [ ] **New imagery sources** — Yandex Maps (YNDX), Apple Maps (APPLE — Python fallback required for auth), Firefly (FIREFLY), USGS/NAIP
- [ ] **macOS Fuse-T support** — Alternative to macFUSE (autoortho4xplane tested with both)
- [ ] **Multi-threaded FUSE toggle** — macOS workaround: disable for stability during scenery loading
- [ ] **Windows: prefer WinFsp config** — Override Dokan preference via config file
- [ ] **Ortho4XP export mode** — `skip_downloads = True` for generating scenery without photos
- [ ] **"Reload Scenery" state preservation** — Already implemented for time exclusion, could generalize

### Not Applicable (our Rust approach differs)
- [ ] ~~PyInstaller bundling~~ — We use standard `cargo build --release`
- [ ] ~~C aopipeline~~ — Our Rust pipeline replaces this natively
- [ ] ~~libcurl native HTTP~~ — reqwest handles connection pooling, HTTP/2 natively in Rust
- [ ] ~~ISPC texcomp~~ — We use Rust BCn crate, but could optionally bundle ISPC binaries for faster compression
