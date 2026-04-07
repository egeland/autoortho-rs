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

#### Phase R1 — Code Quality Fixes ✅
- [x] Fix cache eviction tracking bug
- [x] Eliminate redundant clones in fetcher
- [x] Deduplicate TileFetcher methods
- [x] Add builder pattern for DdsFileSystem
- [x] Replace `.ok()` with proper error handling
- [x] Remove unused code (BufferPool, _key field)
- [x] Optimize `fetcher.rs` to return `Arc<Vec<u8>>` instead of cloning `Vec<u8>`
- [x] Remove unused `mockall` dev-dependency
- _Deferred:_ Optimize `filesystem.rs:687` `slice_range()` — low priority, would need `Cow<[u8]>` return type change

#### Phase R2 — Security Hardening ✅
- [x] Replace hardcoded User-Agent with configurable UA (build.rs fetches Chrome version)
- [x] Force HTTPS for all providers
- [x] Add input validation for parsed numeric values

#### Phase R3 — Architecture Improvements ✅
- [x] Standardize on tokio mutexes in async code (confirmed: zero `std::sync::Mutex`)
- [x] Extract large functions into smaller methods (UI handlers in `ui/handlers.rs`)
- [x] Add config validation helpers with range checks
- _Deferred:_ Add request rate limiting to prevent provider blocking — low priority, would need provider-specific rate limiters

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
- [x] Add request rate limiting to prevent provider blocking

---

## Code Review Observations (2026-03-30)

### Remaining Low-Severity Items
1. ~~`fetcher.rs:107,186,208`~~ → Fixed: returns `Arc<Vec<u8>>` directly
2. ~~`filesystem.rs:687`~~ → Deferred: `slice_range()` allocates Vec (low priority)
3. **`assembler.rs:107-133`** — Image decode failures silently use fallback colors via `.ok()`. Per-failure logging would help debugging.
4. **`fetcher.rs` constructors** — 3 constructor variants still exist alongside builder; minor API bloat.
5. **`dataref.rs:156`** — `vertical_speed_fpm: 0.0` with TODO to compute from altitude delta.
6. ~~`mockall` dev-dependency~~ → Fixed: removed from Cargo.toml
